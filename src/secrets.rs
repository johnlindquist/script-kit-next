//! Secrets storage using age encryption
//!
//! Provides secure secret storage using age (https://age-encryption.org) with
//! scrypt passphrase-based encryption. Secrets are stored as encrypted JSON
//! in ~/.scriptkit/secrets.age.
//!
//! The passphrase is derived from machine-specific identifiers (hostname + app ID)
//! for transparent encryption without requiring user interaction.
//!
//! ## Performance
//!
//! Secrets are cached in memory after first load to avoid repeated scrypt
//! decryption (~1.3s per call). The cache is invalidated on write operations.
//!
//! ## Security
//!
//! - At-rest: Secrets encrypted with age/scrypt in ~/.scriptkit/secrets.age
//! - In-memory: Decrypted cache (standard practice for desktop apps)
//! - Cache cleared on app exit (process memory reclaimed by OS)
//!
//! API matches the keyring functions in prompts/env.rs for easy migration:
//! - `get_secret(key: &str) -> Option<String>`
//! - `set_secret(key: &str, value: &str) -> Result<(), String>`
//! - `delete_secret(key: &str) -> Result<(), String>`

use age::secrecy::SecretString;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::iter;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::logging;

/// In-memory cache of decrypted secrets.
/// Avoids repeated scrypt decryption which takes ~1.3s per call.
static SECRETS_CACHE: OnceLock<Mutex<Option<HashMap<String, String>>>> = OnceLock::new();

/// Get the secrets cache mutex, initializing it if needed.
fn secrets_cache() -> &'static Mutex<Option<HashMap<String, String>>> {
    SECRETS_CACHE.get_or_init(|| Mutex::new(None))
}

/// Get cached secrets, loading from disk if not yet cached.
fn get_cached_secrets() -> HashMap<String, String> {
    let mut guard = secrets_cache().lock().expect("Secrets cache lock poisoned");
    if let Some(ref secrets) = *guard {
        return secrets.clone();
    }

    // First access - load from disk and cache
    let secrets = load_secrets_from_disk();
    *guard = Some(secrets.clone());
    secrets
}

/// Update the cache after a write operation.
fn update_cache(secrets: HashMap<String, String>) {
    let mut guard = secrets_cache().lock().expect("Secrets cache lock poisoned");
    *guard = Some(secrets);
}

/// Warm up the secrets cache (call at app startup).
/// Loads and decrypts secrets in the background so they're ready when needed.
pub fn warmup_cache() {
    std::thread::spawn(|| {
        let start = std::time::Instant::now();
        let secrets = get_cached_secrets();
        let elapsed = start.elapsed();
        logging::log(
            "SECRETS",
            &format!(
                "Warmed up secrets cache: {} keys in {:.2}s",
                secrets.len(),
                elapsed.as_secs_f64()
            ),
        );
    });
}

/// App identifier used in passphrase derivation
const APP_IDENTIFIER: &str = "com.scriptkit.secrets";

/// Get the path to the secrets file
fn secrets_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".scriptkit").join("secrets.age")
}

/// Derive a machine-specific passphrase
///
/// Combines the system hostname with the app identifier to create a passphrase
/// that is unique to this machine but consistent across app restarts.
fn derive_passphrase() -> SecretString {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    // Combine hostname + app identifier for machine-specific passphrase
    SecretString::from(format!("{}:{}", hostname, APP_IDENTIFIER))
}

/// Load and decrypt the secrets store from disk.
/// This is slow (~1.3s) due to scrypt. Use get_cached_secrets() instead.
fn load_secrets_from_disk() -> HashMap<String, String> {
    let path = secrets_path();

    if !path.exists() {
        return HashMap::new();
    }

    let encrypted_data = match fs::read(&path) {
        Ok(data) => data,
        Err(e) => {
            logging::log("SECRETS", &format!("Failed to read secrets file: {}", e));
            return HashMap::new();
        }
    };

    if encrypted_data.is_empty() {
        return HashMap::new();
    }

    let passphrase = derive_passphrase();
    let identity = age::scrypt::Identity::new(passphrase);

    let decryptor = match age::Decryptor::new(&encrypted_data[..]) {
        Ok(d) => d,
        Err(e) => {
            logging::log("SECRETS", &format!("Failed to create decryptor: {}", e));
            return HashMap::new();
        }
    };

    // Verify it's a passphrase-encrypted file
    if !decryptor.is_scrypt() {
        logging::log("SECRETS", "Secrets file is not passphrase-encrypted");
        return HashMap::new();
    }

    let mut decrypted = vec![];
    let mut reader = match decryptor.decrypt(iter::once(&identity as &dyn age::Identity)) {
        Ok(r) => r,
        Err(e) => {
            logging::log("SECRETS", &format!("Failed to decrypt secrets: {}", e));
            return HashMap::new();
        }
    };

    if let Err(e) = reader.read_to_end(&mut decrypted) {
        logging::log("SECRETS", &format!("Failed to read decrypted data: {}", e));
        return HashMap::new();
    }

    match serde_json::from_slice(&decrypted) {
        Ok(secrets) => secrets,
        Err(e) => {
            logging::log("SECRETS", &format!("Failed to parse secrets JSON: {}", e));
            HashMap::new()
        }
    }
}

/// Encrypt and save the secrets store
fn save_secrets(secrets: &HashMap<String, String>) -> Result<(), String> {
    let path = secrets_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create secrets directory: {}", e))?;
    }

    let json =
        serde_json::to_vec(secrets).map_err(|e| format!("Failed to serialize secrets: {}", e))?;

    let passphrase = derive_passphrase();

    // Use with_user_passphrase for simple passphrase-based encryption
    let encryptor = age::Encryptor::with_user_passphrase(passphrase);

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

    fs::write(&path, &encrypted).map_err(|e| format!("Failed to write secrets file: {}", e))?;

    logging::log(
        "SECRETS",
        &format!("Saved {} secrets to {:?}", secrets.len(), path),
    );
    Ok(())
}

/// Get a secret from the encrypted store
///
/// Returns `Some(value)` if the secret exists, `None` otherwise.
///
/// # Example
/// ```ignore
/// if let Some(api_key) = get_secret("OPENAI_API_KEY") {
///     // Use the key
/// }
/// ```
pub fn get_secret(key: &str) -> Option<String> {
    let secrets = get_cached_secrets();
    let result = secrets.get(key).cloned();

    if result.is_some() {
        logging::log("SECRETS", &format!("Retrieved secret for key: {}", key));
    } else {
        logging::log("SECRETS", &format!("No secret found for key: {}", key));
    }

    result
}

/// Set a secret in the encrypted store
///
/// Creates or updates the secret with the given key.
///
/// # Example
/// ```ignore
/// set_secret("OPENAI_API_KEY", "sk-...")?;
/// ```
pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
    let mut secrets = get_cached_secrets();
    secrets.insert(key.to_string(), value.to_string());
    save_secrets(&secrets)?;

    // Update the in-memory cache
    update_cache(secrets);

    logging::log("SECRETS", &format!("Stored secret for key: {}", key));
    Ok(())
}

/// Delete a secret from the encrypted store
///
/// Removes the secret if it exists. Returns Ok even if the key doesn't exist.
///
/// # Example
/// ```ignore
/// delete_secret("OPENAI_API_KEY")?;
/// ```
#[allow(dead_code)]
pub fn delete_secret(key: &str) -> Result<(), String> {
    let mut secrets = get_cached_secrets();

    if secrets.remove(key).is_some() {
        save_secrets(&secrets)?;
        // Update the in-memory cache
        update_cache(secrets);
        logging::log("SECRETS", &format!("Deleted secret for key: {}", key));
    } else {
        logging::log("SECRETS", &format!("No secret to delete for key: {}", key));
    }

    Ok(())
}

/// Check if a secret exists in the store
///
/// # Example
/// ```ignore
/// if has_secret("OPENAI_API_KEY") {
///     // Key exists
/// }
/// ```
#[allow(dead_code)]
pub fn has_secret(key: &str) -> bool {
    let secrets = get_cached_secrets();
    secrets.contains_key(key)
}

/// List all secret keys (not values)
///
/// Returns the keys of all stored secrets. Useful for UI to show which
/// secrets are configured.
///
/// # Example
/// ```ignore
/// for key in list_secret_keys() {
///     println!("Have secret: {}", key);
/// }
/// ```
#[allow(dead_code)]
pub fn list_secret_keys() -> Vec<String> {
    let secrets = get_cached_secrets();
    secrets.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passphrase_derivation() {
        let passphrase = derive_passphrase();
        // SecretString doesn't expose its content easily in tests,
        // but we can verify it was created successfully
        drop(passphrase);
    }

    #[test]
    fn test_secrets_path() {
        let path = secrets_path();
        assert!(path.ends_with("secrets.age"));
        assert!(path.to_string_lossy().contains(".scriptkit"));
    }

    // Integration tests that actually read/write would go here
    // but should be feature-gated to avoid modifying user data
}
