//! Sign-in recovery: open `goose configure` against Script Kit's goose root.
//!
//! goose owns provider sign-in (API keys, ChatGPT and other OAuth flows). When
//! a turn fails for missing or rejected credentials, the recovery action opens
//! a Terminal running goose's own configuration flow for the same state
//! directory the sidecar uses, so the next turn picks up the new sign-in.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthRecoveryAction {
    SignInAgain,
    /// Forget the provider's stored sign-in first, then configure again.
    SwitchAccount,
}

fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// The provider half of the session's `provider/model` selection.
pub(crate) fn resolve_auth_recovery_provider(selected_model_id: Option<&str>) -> Option<String> {
    selected_model_id
        .and_then(|model_id| model_id.split_once('/').map(|(provider, _)| provider))
        .filter(|provider| valid_provider_id(provider))
        .map(str::to_string)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn build_auth_recovery_command(
    goose_binary: &Path,
    goose_root: &Path,
    provider: &str,
    action: AuthRecoveryAction,
) -> Result<String, String> {
    if !valid_provider_id(provider) {
        return Err("invalid provider id".to_string());
    }
    let env = format!(
        "GOOSE_PATH_ROOT={} GOOSE_DISABLE_KEYRING=1 GOOSE_PROVIDER={}",
        shell_quote(&goose_root.to_string_lossy()),
        shell_quote(provider)
    );
    let configure = format!(
        "{env} {} configure",
        shell_quote(&goose_binary.to_string_lossy())
    );
    Ok(match action {
        AuthRecoveryAction::SignInAgain => configure,
        AuthRecoveryAction::SwitchAccount => {
            // OAuth providers keep their tokens under config/<provider>/.
            let tokens = goose_root.join("config").join(provider);
            format!(
                "rm -rf {} && {configure}",
                shell_quote(&tokens.to_string_lossy())
            )
        }
    })
}

pub(crate) fn launch_auth_recovery(
    goose_binary: PathBuf,
    provider: String,
    action: AuthRecoveryAction,
) -> Result<(), String> {
    let root = super::launch_spec::goose_root(&crate::setup::get_kit_path());
    let command = build_auth_recovery_command(&goose_binary, &root, &provider, action)?;

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell application \"Terminal\"\nactivate\ndo script \"{}\"\nend tell",
            crate::utils::escape_applescript_string(&command)
        );
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .spawn()
            .map_err(|error| format!("failed to open provider sign-in terminal: {error}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (command, goose_binary, provider, action);
        Err("provider sign-in recovery is not supported on this platform".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_comes_from_the_namespaced_model() {
        assert_eq!(
            resolve_auth_recovery_provider(Some("chatgpt_codex/gpt-5.6-sol")).as_deref(),
            Some("chatgpt_codex")
        );
        assert_eq!(resolve_auth_recovery_provider(Some("gpt-5.6-sol")), None);
    }

    #[test]
    fn sign_in_runs_goose_configure_against_script_kits_goose_root() {
        let command = build_auth_recovery_command(
            Path::new("/Applications/Script Kit.app/Contents/MacOS/goose"),
            Path::new("/Users/me/.scriptkit/goose"),
            "chatgpt_codex",
            AuthRecoveryAction::SignInAgain,
        )
        .unwrap();
        assert!(command.starts_with("GOOSE_PATH_ROOT='/Users/me/.scriptkit/goose'"));
        assert!(command.ends_with("'/Applications/Script Kit.app/Contents/MacOS/goose' configure"));
    }

    #[test]
    fn switch_account_forgets_only_that_providers_tokens() {
        let command = build_auth_recovery_command(
            Path::new("/tmp/goose"),
            Path::new("/tmp/root"),
            "chatgpt_codex",
            AuthRecoveryAction::SwitchAccount,
        )
        .unwrap();
        assert!(command.starts_with("rm -rf '/tmp/root/config/chatgpt_codex' && "));
    }

    #[test]
    fn provider_id_rejects_shell_metacharacters() {
        assert!(build_auth_recovery_command(
            Path::new("/tmp/goose"),
            Path::new("/tmp/root"),
            "chatgpt_codex; rm -rf /",
            AuthRecoveryAction::SignInAgain,
        )
        .is_err());
    }
}
