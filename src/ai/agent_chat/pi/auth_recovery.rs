use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PiAuthRecoveryAction {
    SignInAgain,
    SwitchAccount,
}

fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn provider_from_error(raw_error: &str) -> Option<&str> {
    let normalized = raw_error.to_ascii_lowercase();
    let marker = "provider error:";
    let marker_index = normalized.find(marker)?;
    let provider = raw_error[marker_index + marker.len()..]
        .trim_start()
        .split(|ch: char| ch == ':' || ch.is_whitespace())
        .next()?;
    valid_provider_id(provider).then_some(provider)
}

pub(crate) fn resolve_auth_recovery_provider(
    selected_model_id: Option<&str>,
    raw_error: Option<&str>,
) -> Option<String> {
    selected_model_id
        .and_then(|model_id| model_id.split_once('/').map(|(provider, _)| provider))
        .filter(|provider| valid_provider_id(provider))
        .or_else(|| raw_error.and_then(provider_from_error))
        .map(str::to_string)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn build_pi_auth_recovery_command(
    pi_binary: &Path,
    provider: &str,
    action: PiAuthRecoveryAction,
) -> Result<String, String> {
    if !valid_provider_id(provider) {
        return Err("invalid provider id".to_string());
    }

    let binary = shell_quote(&pi_binary.to_string_lossy());
    let provider_arg = shell_quote(provider);
    let login = shell_quote(&format!("/login {provider}"));
    let command = match action {
        PiAuthRecoveryAction::SignInAgain => {
            format!("{binary} --provider {provider_arg} {login}")
        }
        PiAuthRecoveryAction::SwitchAccount => {
            let logout = shell_quote(&format!("/logout {provider}"));
            format!("{binary} --provider {provider_arg} {logout} {login}")
        }
    };
    Ok(command)
}

pub(crate) fn launch_pi_auth_recovery(
    pi_binary: PathBuf,
    provider: String,
    action: PiAuthRecoveryAction,
) -> Result<(), String> {
    let command = build_pi_auth_recovery_command(&pi_binary, &provider, action)?;

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
        let _ = (command, pi_binary, provider, action);
        Err("provider sign-in recovery is not supported on this platform".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_prefers_namespaced_model_and_falls_back_to_provider_error() {
        assert_eq!(
            resolve_auth_recovery_provider(
                Some("openai-codex/gpt-5.3-codex-spark"),
                Some("Provider error: anthropic: nope")
            )
            .as_deref(),
            Some("openai-codex")
        );
        assert_eq!(
            resolve_auth_recovery_provider(
                Some("gpt-5.3-codex-spark"),
                Some("Provider error: openai-codex: OpenAI API error (HTTP 429)")
            )
            .as_deref(),
            Some("openai-codex")
        );
    }

    #[test]
    fn switch_account_runs_logout_then_login_for_the_same_provider() {
        let command = build_pi_auth_recovery_command(
            Path::new("/Applications/Script Kit.app/Contents/MacOS/pi"),
            "openai-codex",
            PiAuthRecoveryAction::SwitchAccount,
        )
        .expect("valid command");

        assert!(command.contains("'/logout openai-codex' '/login openai-codex'"));
        assert!(command.contains("--provider 'openai-codex'"));
    }

    #[test]
    fn provider_id_rejects_shell_metacharacters() {
        assert!(build_pi_auth_recovery_command(
            Path::new("/tmp/pi"),
            "openai-codex; rm -rf /",
            PiAuthRecoveryAction::SignInAgain,
        )
        .is_err());
    }
}
