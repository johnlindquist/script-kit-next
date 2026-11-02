//! The personal credential vault. New ciphertext uses a random age X25519 identity
//! held by the macOS Keychain, outside the workspace. A readable legacy scrypt vault
//! is re-encrypted on access. A missing/locked key is never treated as an empty vault.

mod vault_key;

use age::secrecy::SecretString;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::iter;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use crate::logging;

/// A secret entry with value and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    /// The secret value
    pub value: String,
    /// When the secret was last modified (ISO 8601)
    pub modified_at: DateTime<Utc>,
}

/// Information about a stored secret (returned to callers)
#[derive(Debug, Clone)]
pub struct SecretInfo {
    /// The secret value
    pub value: String,
    /// When the secret was last modified
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretStoreErrorKind {
    PolicyRefused,
    PathUnavailable,
    KeyUnavailable,
    ReadFailed,
    InvalidFormat,
    UnsupportedEncryption,
    DecryptFailed,
    ParseFailed,
    CacheUnavailable,
}

impl SecretStoreErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecretStoreErrorKind::PolicyRefused => "credentials_refused",
            SecretStoreErrorKind::PathUnavailable => "path_unavailable",
            SecretStoreErrorKind::KeyUnavailable => "key_unavailable",
            SecretStoreErrorKind::ReadFailed => "read_failed",
            SecretStoreErrorKind::InvalidFormat => "invalid_format",
            SecretStoreErrorKind::UnsupportedEncryption => "unsupported_encryption",
            SecretStoreErrorKind::DecryptFailed => "decrypt_failed",
            SecretStoreErrorKind::ParseFailed => "parse_failed",
            SecretStoreErrorKind::CacheUnavailable => "cache_unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretStoreError {
    pub kind: SecretStoreErrorKind,
    pub message: String,
}

impl SecretStoreError {
    fn new(kind: SecretStoreErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind_str(&self) -> &'static str {
        self.kind.as_str()
    }

    pub fn user_message(&self) -> &'static str {
        match self.kind {
            SecretStoreErrorKind::PolicyRefused => "Credential access is forbidden in owned evaluation.",
            SecretStoreErrorKind::PathUnavailable => {
                "Secret storage location is unavailable. Check your home directory and try again."
            }
            SecretStoreErrorKind::KeyUnavailable => "Unlock the macOS Keychain for this workspace and try again. Your saved vault has not been replaced.",
            SecretStoreErrorKind::ReadFailed => {
                "Secret storage could not be read. Check file permissions and try again."
            }
            SecretStoreErrorKind::InvalidFormat
            | SecretStoreErrorKind::UnsupportedEncryption
            | SecretStoreErrorKind::DecryptFailed
            | SecretStoreErrorKind::ParseFailed => {
                "Saved secrets could not be loaded. The store may be corrupt or from another machine."
            }
            SecretStoreErrorKind::CacheUnavailable => {
                "Secret storage is temporarily unavailable. Try again."
            }
        }
    }
}

impl From<SecretEntry> for SecretInfo {
    fn from(entry: SecretEntry) -> Self {
        SecretInfo {
            value: entry.value,
            modified_at: entry.modified_at,
        }
    }
}

/// In-memory cache of decrypted secrets with metadata.
/// Avoids repeated scrypt decryption which takes ~1.3s per call.
type SecretsCache = Option<Result<HashMap<String, SecretEntry>, SecretStoreError>>;

static SECRETS_CACHE: LazyLock<Mutex<SecretsCache>> = LazyLock::new(|| Mutex::new(None));

/// Get the secrets cache mutex.
fn secrets_cache() -> &'static Mutex<SecretsCache> {
    &SECRETS_CACHE
}

/// Get cached secrets, loading from disk if not yet cached.
fn get_cached_secrets() -> Result<HashMap<String, SecretEntry>, SecretStoreError> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials).map_err(
        |error| SecretStoreError::new(SecretStoreErrorKind::PolicyRefused, error.to_string()),
    )?;
    if cfg!(test) || std::env::var("SCRIPT_KIT_NONINTERACTIVE").as_deref() == Ok("1") {
        return Err(SecretStoreError::new(
            SecretStoreErrorKind::PolicyRefused,
            "System credentials are disabled in this process",
        ));
    }
    let mut guard = secrets_cache().lock().map_err(|e| {
        SecretStoreError::new(
            SecretStoreErrorKind::CacheUnavailable,
            format!("Secrets cache lock poisoned: {e}"),
        )
    })?;
    if let Some(ref cached) = *guard {
        return cached.clone();
    }

    // First access - load from disk and cache
    match load_secrets_from_disk() {
        Ok(secrets) => {
            *guard = Some(Ok(secrets.clone()));
            Ok(secrets)
        }
        Err(error) => Err(error),
    }
}

/// Update the cache after a write operation.
fn update_cache(secrets: HashMap<String, SecretEntry>) -> anyhow::Result<()> {
    let mut guard = secrets_cache()
        .lock()
        .map_err(|e| anyhow::anyhow!("Secrets cache lock poisoned: {e}"))?;
    *guard = Some(Ok(secrets));
    Ok(())
}

/// Read-only compatibility with the original scrypt format; never used to encrypt a new file.
const APP_IDENTIFIER: &str = "com.scriptkit.secrets";

/// Get the path to the secrets file
fn secrets_path() -> anyhow::Result<PathBuf> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials)?;
    Ok(crate::setup::get_kit_path().join("secrets.age"))
}

/// Derive a machine-specific passphrase
///
/// Combines the system hostname with the app identifier to create a passphrase
/// that is unique to this machine but consistent across app restarts.
fn legacy_passphrase() -> SecretString {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    // Combine hostname + app identifier for machine-specific passphrase
    SecretString::from(format!("{}:{}", hostname, APP_IDENTIFIER))
}

type Entries = HashMap<String, SecretEntry>;
type KeySource = fn(&std::path::Path, bool) -> Result<age::x25519::Identity, SecretStoreError>;

/// Read but do not follow links or silently reinterpret a corrupt/empty vault.
fn read_vault(path: &std::path::Path, key: KeySource) -> Result<(Entries, bool), SecretStoreError> {
    use std::fs::OpenOptions;
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials)
        .map_err(|e| SecretStoreError::new(SecretStoreErrorKind::PolicyRefused, e.to_string()))?;
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let failure = || {
        SecretStoreError::new(
            SecretStoreErrorKind::ReadFailed,
            "Couldn't read the private vault",
        )
    };
    let file = match opts.open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Entries::new(), false)),
        Err(_) => return Err(failure()),
    };
    let meta = file.metadata().map_err(|_| failure())?;
    if !meta.is_file() || meta.len() > 16 * 1024 * 1024 {
        return Err(failure());
    }
    let mut encrypted_data = vec![];
    file.take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut encrypted_data)
        .map_err(|_| failure())?;
    if encrypted_data.len() > 16 * 1024 * 1024 {
        return Err(failure());
    }
    let decryptor = match age::Decryptor::new(&encrypted_data[..]) {
        Ok(d) => d,
        Err(e) => {
            return Err(SecretStoreError::new(
                SecretStoreErrorKind::InvalidFormat,
                format!("Failed to create decryptor: {e}"),
            ));
        }
    };

    let legacy = decryptor.is_scrypt();
    let identity: Box<dyn age::Identity> = if legacy {
        Box::new(age::scrypt::Identity::new(legacy_passphrase()))
    } else {
        Box::new(key(path, false)?)
    };
    let mut decrypted = vec![];
    let mut reader = match decryptor.decrypt(iter::once(identity.as_ref())) {
        Ok(r) => r,
        Err(e) => {
            return Err(SecretStoreError::new(
                SecretStoreErrorKind::DecryptFailed,
                format!("Failed to decrypt secrets: {e}"),
            ));
        }
    };

    if let Err(e) = reader.read_to_end(&mut decrypted) {
        return Err(SecretStoreError::new(
            SecretStoreErrorKind::DecryptFailed,
            format!("Failed to read decrypted data: {e}"),
        ));
    }

    // Try to parse as new format first
    if let Ok(secrets) = serde_json::from_slice::<HashMap<String, SecretEntry>>(&decrypted) {
        logging::log("SECRETS", "Loaded secrets in new format with metadata");
        return Ok((secrets, legacy));
    }

    // Fall back to old format and migrate
    if let Ok(old_secrets) = serde_json::from_slice::<HashMap<String, String>>(&decrypted) {
        logging::log(
            "SECRETS",
            &format!(
                "Migrating {} secrets from old format to new format with timestamps",
                old_secrets.len()
            ),
        );
        let now = Utc::now();
        let migrated: HashMap<String, SecretEntry> = old_secrets
            .into_iter()
            .map(|(key, value)| {
                (
                    key,
                    SecretEntry {
                        value,
                        modified_at: now,
                    },
                )
            })
            .collect();
        return Ok((migrated, legacy));
    }

    Err(SecretStoreError::new(
        SecretStoreErrorKind::ParseFailed,
        "Failed to parse secrets JSON in any format",
    ))
}

/// Load and decrypt the secrets store from disk.
/// This is slow (~1.3s) due to scrypt. Use get_cached_secrets() instead.
///
/// Handles migration from old format (HashMap<String, String>) to new format
/// (HashMap<String, SecretEntry>) automatically.
fn load_secrets_from_disk() -> Result<HashMap<String, SecretEntry>, SecretStoreError> {
    if cfg!(test) || std::env::var("SCRIPT_KIT_NONINTERACTIVE").as_deref() == Ok("1") {
        return Err(SecretStoreError::new(
            SecretStoreErrorKind::PolicyRefused,
            "System credentials are disabled in this process",
        ));
    }
    let path = secrets_path().map_err(|e| {
        SecretStoreError::new(
            SecretStoreErrorKind::PathUnavailable,
            format!("Failed to resolve secrets path when loading from disk: {e}"),
        )
    })?;

    if matches!(fs::symlink_metadata(&path),Err(e) if e.kind()==std::io::ErrorKind::NotFound) {
        return Ok(Entries::new());
    }
    let _lock = lock_vault(&path)?;
    open_and_upgrade(&path, vault_key::identity)
}

fn open_and_upgrade(path: &std::path::Path, key: KeySource) -> Result<Entries, SecretStoreError> {
    let (entries, legacy) = read_vault(path, key)?;
    if legacy {
        let id = key(path, true)?;
        save_secrets_to_path(&entries, path, &id)
            .map_err(|e| SecretStoreError::new(SecretStoreErrorKind::ReadFailed, e))?;
    }
    Ok(entries)
}
fn lock_vault(path: &std::path::Path) -> Result<fs::File, SecretStoreError> {
    let failed = || {
        SecretStoreError::new(
            SecretStoreErrorKind::CacheUnavailable,
            "Vault is busy or inaccessible",
        )
    };
    let parent = path.parent().ok_or_else(failed)?;
    fs::create_dir_all(parent).map_err(|_| failed())?;
    let mut opts = fs::OpenOptions::new();
    opts.create(true).truncate(false).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let fd = opts
        .open(parent.join(".secrets-lock"))
        .map_err(|_| failed())?;
    if !fd.metadata().is_ok_and(|m| m.is_file()) {
        return Err(failed());
    }
    fd.try_lock().map_err(|_| failed())?;
    Ok(fd)
}
fn mutate_vault(change: impl FnOnce(&mut Entries) -> bool) -> Result<(), String> {
    if cfg!(test) || std::env::var("SCRIPT_KIT_NONINTERACTIVE").as_deref() == Ok("1") {
        return Err("System credentials are disabled in this process".into());
    }
    let path = secrets_path().map_err(|e| e.to_string())?;
    let _lock = lock_vault(&path).map_err(|e| e.message)?;
    let (mut entries, legacy) = read_vault(&path, vault_key::identity).map_err(|e| e.message)?;
    let changed = change(&mut entries);
    if changed || legacy {
        let id = vault_key::identity(&path, legacy || !path.exists()).map_err(|e| e.message)?;
        save_secrets_to_path(&entries, &path, &id)?;
    }
    update_cache(entries).map_err(|e| e.to_string())
}

/// Encrypt and atomically persist the secrets store to `path`.
///
/// The encrypted bytes are written to a sibling temp file (created 0o600 on
/// Unix) and then renamed over the destination. This matters for two reasons:
///
/// 1. **Atomicity.** A crash or SIGKILL during a direct in-place `truncate` +
///    `write_all` could leave `secrets.age` truncated and undecryptable —
///    losing *every* stored secret. A temp-file rename is a same-filesystem
///    atomic replace, so an interrupted save leaves the previous store intact.
/// 2. **Permission enforcement.** `OpenOptions::mode(0o600)` is ignored when the
///    destination already exists, so the previous in-place write silently left a
///    pre-existing loose-permission file untouched. A fresh temp file is created
///    0o600 every time and then replaces the destination, so the secrets file is
///    always owner-only after a save.
fn save_secrets_to_path(
    secrets: &HashMap<String, SecretEntry>,
    path: &std::path::Path,
    identity: &age::x25519::Identity,
) -> Result<(), String> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials)
        .map_err(|error| error.to_string())?;
    // Ensure parent directory exists
    let parent = path
        .parent()
        .ok_or_else(|| format!("Secrets path has no parent directory: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create secrets directory: {}", e))?;

    let json =
        serde_json::to_vec(secrets).map_err(|e| format!("Failed to serialize secrets: {}", e))?;

    let recipient = identity.to_public();
    let encryptor = age::Encryptor::with_recipients(iter::once(&recipient as &dyn age::Recipient))
        .map_err(|e| e.to_string())?;

    let mut encrypted = vec![];
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(|e| format!("Failed to create encryption writer: {}", e))?;

    writer
        .write_all(&json)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    writer
        .finish()
        .map_err(|e| format!("Failed to finish encryption: {}", e))?;

    // Encrypt into a sibling temp file, then atomically rename it over the
    // destination.
    let mut temp_file = tempfile::Builder::new()
        .prefix(".secrets-")
        .suffix(".age.tmp")
        .tempfile_in(parent)
        .map_err(|e| format!("Failed to create temp secrets file: {}", e))?;

    // Write secrets file with restrictive permissions (0o600 - owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = temp_file
            .as_file()
            .metadata()
            .map_err(|e| format!("Failed to read temp secrets metadata: {}", e))?
            .permissions();
        perms.set_mode(0o600);
        temp_file.as_file().set_permissions(perms).map_err(|e| {
            format!(
                "Failed to set secure permissions on temp secrets file: {}",
                e
            )
        })?;
    }

    temp_file
        .write_all(&encrypted)
        .map_err(|e| format!("Failed to write encrypted secrets to temp file: {}", e))?;
    temp_file
        .as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temp secrets file: {}", e))?;

    temp_file
        .persist(path)
        .map_err(|e| format!("Failed to atomically replace secrets file: {}", e.error))?;

    logging::log(
        "SECRETS",
        &format!("Saved {} secrets to {:?} (atomic)", secrets.len(), path),
    );
    Ok(())
}

/// Get a secret value from the encrypted store
///
/// Returns `Some(value)` if the secret exists, `None` otherwise.
/// For metadata (modification time), use `get_secret_info()` instead.
///
/// # Example
/// ```ignore
/// if let Some(api_key) = get_secret("OPENAI_API_KEY") {
///     // Use the key
/// }
/// ```
pub fn get_secret(key: &str) -> Option<String> {
    match get_secret_result(key) {
        Ok(result) => result,
        Err(error) => {
            tracing::error!(
                key,
                kind = error.kind_str(),
                "Failed to get cached secrets for get_secret: {}",
                error.message
            );
            None
        }
    }
}

pub fn get_secret_result(key: &str) -> Result<Option<String>, SecretStoreError> {
    let secrets = get_cached_secrets()?;
    let result = secrets.get(key).map(|entry| entry.value.clone());

    if result.is_some() {
        logging::log("SECRETS", &format!("Retrieved secret for key: {}", key));
    } else {
        logging::log("SECRETS", &format!("No secret found for key: {}", key));
    }

    Ok(result)
}

/// Get a secret with its metadata from the encrypted store
///
/// Returns `Some(SecretInfo)` if the secret exists, including the value
/// and when it was last modified. Returns `None` if not found.
///
/// # Example
/// ```ignore
/// if let Some(info) = get_secret_info("OPENAI_API_KEY") {
///     println!("Key set on: {}", info.modified_at);
///     // Use info.value
/// }
/// ```
pub fn get_secret_info(key: &str) -> Option<SecretInfo> {
    match get_secret_info_result(key) {
        Ok(result) => result,
        Err(error) => {
            tracing::error!(
                key,
                kind = error.kind_str(),
                "Failed to get cached secrets for get_secret_info: {}",
                error.message
            );
            None
        }
    }
}

pub fn get_secret_info_result(key: &str) -> Result<Option<SecretInfo>, SecretStoreError> {
    let secrets = get_cached_secrets()?;
    let result = secrets
        .get(key)
        .map(|entry| SecretInfo::from(entry.clone()));

    if result.is_some() {
        logging::log(
            "SECRETS",
            &format!("Retrieved secret info for key: {}", key),
        );
    } else {
        logging::log("SECRETS", &format!("No secret found for key: {}", key));
    }

    Ok(result)
}

/// Set a secret in the encrypted store
///
/// Creates or updates the secret with the given key.
/// Updates the modification timestamp to now.
///
/// # Example
/// ```ignore
/// set_secret("OPENAI_API_KEY", "sk-...")?;
/// ```
pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
    mutate_vault(|entries| {
        entries.insert(
            key.to_owned(),
            SecretEntry {
                value: value.to_owned(),
                modified_at: Utc::now(),
            },
        );
        true
    })
}

/// Delete a secret from the encrypted store
///
/// Removes the secret if it exists. Returns Ok even if the key doesn't exist.
///
/// # Example
/// ```ignore
/// delete_secret("OPENAI_API_KEY")?;
/// ```
pub fn delete_secret(key: &str) -> Result<(), String> {
    mutate_vault(|entries| entries.remove(key).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    static TEST_ID: LazyLock<age::x25519::Identity> =
        LazyLock::new(age::x25519::Identity::generate);
    fn fixture_key(
        _: &std::path::Path,
        _: bool,
    ) -> Result<age::x25519::Identity, SecretStoreError> {
        Ok(TEST_ID.clone())
    }
    fn load_secrets_from_path(p: &std::path::Path) -> Result<Entries, SecretStoreError> {
        read_vault(p, fixture_key).map(|(v, _)| v)
    }

    #[test]
    fn legacy_upgrade_preserves_bytes_when_key_denied_and_rekeys_without_hostname() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("secrets.age");
        let legacy = encrypted_test_payload(br#"{"fixture":"example"}"#);
        fs::write(&path, &legacy).unwrap();
        let denied = |_: &std::path::Path, _| {
            Err(SecretStoreError::new(
                SecretStoreErrorKind::KeyUnavailable,
                "locked",
            ))
        };
        assert_eq!(
            open_and_upgrade(&path, denied).unwrap_err().kind,
            SecretStoreErrorKind::KeyUnavailable
        );
        assert_eq!(fs::read(&path).unwrap(), legacy);
        assert_eq!(
            open_and_upgrade(&path, fixture_key).unwrap()["fixture"].value,
            "example"
        );
        let upgraded = fs::read(&path).unwrap();
        assert!(!age::Decryptor::new(upgraded.as_slice())
            .unwrap()
            .is_scrypt());
        assert_eq!(
            read_vault(&path, denied).unwrap_err().kind,
            SecretStoreErrorKind::KeyUnavailable
        );
        assert_eq!(fs::read(&path).unwrap(), upgraded);
        assert_eq!(
            read_vault(&path, fixture_key).unwrap().0["fixture"].value,
            "example"
        );
    }
    #[test]
    fn test_passphrase_derivation() {
        let passphrase = legacy_passphrase();
        // SecretString doesn't expose its content easily in tests,
        // but we can verify it was created successfully
        drop(passphrase);
    }

    #[test]
    fn test_secrets_path() {
        let path = secrets_path().expect("secrets path should resolve in tests");
        assert!(path.ends_with("secrets.age"));
        assert_eq!(path.parent(), Some(crate::setup::get_kit_path().as_path()));
    }

    fn encrypted_test_payload(bytes: &[u8]) -> Vec<u8> {
        let passphrase = legacy_passphrase();
        let encryptor = age::Encryptor::with_user_passphrase(passphrase);
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
        writer.write_all(bytes).unwrap();
        writer.finish().unwrap();
        encrypted
    }

    #[test]
    fn missing_secrets_file_loads_as_empty_store() {
        let temp_dir = TempDir::new().unwrap();
        let missing_path = temp_dir.path().join("missing-secrets.age");

        let secrets = load_secrets_from_path(&missing_path).expect("missing file is not an error");

        assert!(secrets.is_empty());
    }

    #[test]
    fn invalid_secret_store_format_is_not_missing_secret() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid-secrets.age");
        fs::write(&path, b"not an age file").unwrap();

        let error = load_secrets_from_path(&path).expect_err("invalid store should be distinct");

        assert_eq!(error.kind, SecretStoreErrorKind::InvalidFormat);
    }

    #[test]
    fn unparsable_decrypted_secret_store_is_not_missing_secret() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid-json-secrets.age");
        fs::write(&path, encrypted_test_payload(b"not-json")).unwrap();

        let error = load_secrets_from_path(&path).expect_err("parse failure should be distinct");

        assert_eq!(error.kind, SecretStoreErrorKind::ParseFailed);
    }

    #[test]
    fn unreadable_secret_store_path_is_not_missing_secret() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("secret-store-directory.age");
        fs::create_dir(&path).unwrap();

        let error = load_secrets_from_path(&path).expect_err("read failure should be distinct");

        assert_eq!(error.kind, SecretStoreErrorKind::ReadFailed);
    }

    #[test]
    #[cfg(unix)]
    fn test_secrets_file_has_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("test-secrets.age");

        // Create test secrets
        let mut test_secrets = HashMap::new();
        test_secrets.insert(
            "TEST_KEY".to_string(),
            SecretEntry {
                value: "test_value".to_string(),
                modified_at: chrono::Utc::now(),
            },
        );

        // Manually save secrets to temp path (copy save_secrets logic)
        let json = serde_json::to_vec(&test_secrets).unwrap();
        let passphrase = legacy_passphrase();
        let encryptor = age::Encryptor::with_user_passphrase(passphrase);
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
        writer.write_all(&json).unwrap();
        writer.finish().unwrap();

        // Write with secure permissions
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temp_path)
            .and_then(|mut file| file.write_all(&encrypted))
            .unwrap();

        // Verify file permissions
        let metadata = fs::metadata(&temp_path).unwrap();
        let mode = metadata.permissions().mode();
        let file_perms = mode & 0o777;

        assert_eq!(
            file_perms, 0o600,
            "Secrets file should have 0o600 permissions, got 0o{:o}",
            file_perms
        );
    }

    #[test]
    #[cfg(unix)]
    fn save_secrets_to_path_is_atomic_and_reenforces_0600_on_existing_file() {
        use std::fs::OpenOptions;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("secrets.age");

        // Simulate a stale / loose pre-existing store (e.g. written by an older
        // build, or chmod'd by the user). Then empirically characterize the bug
        // the fix addresses: re-opening an *existing* file with `mode(0o600)`
        // does NOT tighten it — which is exactly what the previous in-place
        // `save_secrets` did, silently leaving loose permissions in place.
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o644)
            .open(&path)
            .unwrap()
            .write_all(b"stale")
            .unwrap();
        // An owner-only process umask also narrows mode at creation. This fake
        // stale file (inside the private temp directory) must actually be loose.
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .unwrap()
            .write_all(b"still-loose")
            .unwrap();
        let leftover = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            leftover, 0o644,
            "characterization: OpenOptions::mode is ignored on an existing file (the bug)"
        );

        // The fixed save must re-establish 0o600 and round-trip the contents.
        let mut secrets = HashMap::new();
        secrets.insert(
            "API_KEY".to_string(),
            SecretEntry {
                value: "sk-secret".to_string(),
                modified_at: Utc::now(),
            },
        );
        save_secrets_to_path(&secrets, &path, &TEST_ID).expect("atomic save should succeed");

        // (a) Permissions are re-tightened to 0o600 even though the destination
        // already existed at 0o644. Fails if the fix regresses to an in-place write.
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "secrets file must be 0o600 after save, got 0o{mode:o}"
        );

        // (b) Contents decrypt through the real load path.
        let loaded = load_secrets_from_path(&path).expect("saved store must decrypt");
        assert_eq!(
            loaded.get("API_KEY").map(|entry| entry.value.as_str()),
            Some("sk-secret")
        );

        // (c) The atomic rename leaves no temp file litter behind.
        let leftovers: Vec<String> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name != "secrets.age")
            .collect();
        assert!(
            leftovers.is_empty(),
            "no temp files should remain after atomic save, found: {leftovers:?}"
        );
    }

    // Integration tests that actually read/write would go here
    // but should be feature-gated to avoid modifying user data
}
