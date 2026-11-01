//! The private age identity never goes in a Kit workspace, git repository, or process argument.
use super::{SecretStoreError, SecretStoreErrorKind};
use age::x25519::Identity;
use std::path::Path;

pub(super) fn identity(_path: &Path, create: bool) -> Result<Identity, SecretStoreError> {
    let refused = || {
        SecretStoreError::new(
            SecretStoreErrorKind::KeyUnavailable,
            "The private vault key is unavailable",
        )
    };
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Credentials)
        .map_err(|_| refused())?;
    if std::env::var("SCRIPT_KIT_NONINTERACTIVE").as_deref() == Ok("1") {
        return Err(refused());
    }
    #[cfg(all(target_os = "macos", not(test)))]
    {
        use age::secrecy::ExposeSecret;
        use security_framework::os::macos::keychain::SecKeychain;
        // The same macOS user can move their encrypted workspace without losing its key.
        const ACCOUNT: &str = "personal-vault-identity";
        const SERVICE: &str = "com.scriptkit.app.age-vault.v2";
        let kc = SecKeychain::default().map_err(|_| refused())?;
        let read = || kc.find_generic_password(SERVICE, ACCOUNT);
        match read() {
            Ok((bytes, _)) => std::str::from_utf8(&bytes)
                .ok()
                .and_then(|s| s.parse().ok())
                .ok_or_else(refused),
            Err(e) if create && e.code() == -25300 => {
                let id = Identity::generate();
                match kc.add_generic_password(
                    SERVICE,
                    ACCOUNT,
                    id.to_string().expose_secret().as_bytes(),
                ) {
                    Ok(()) => Ok(id),
                    // Another creator won. Read its key; never update/replace it.
                    Err(e) if e.code() == -25299 => {
                        let (bytes, _) = read().map_err(|_| refused())?;
                        std::str::from_utf8(&bytes)
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .ok_or_else(refused)
                    }
                    Err(_) => Err(refused()),
                }
            }
            Err(_) => Err(refused()),
        }
    }
    #[cfg(any(not(target_os = "macos"), test))]
    {
        let _ = create;
        Err(refused())
    }
}
