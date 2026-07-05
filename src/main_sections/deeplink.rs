// DEEPLINK_CHANNEL: Channel for handling scriptkit:// URL scheme events
// URLs are sent from on_open_urls callback and processed inside the app
static DEEPLINK_CHANNEL: std::sync::OnceLock<(
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
)> = std::sync::OnceLock::new();

/// Get the deeplink channel, initializing it on first access.
fn deeplink_channel() -> &'static (
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
) {
    DEEPLINK_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

/// Parse a scriptkit:// URL and extract the command ID
/// Supported formats:
/// - scriptkit://commands/{command_id} - Execute any command (app/builtin/script/scriptlet)
/// - scriptkit://run/{script_name} - Execute a script by name (legacy)
/// - scriptkit://notes/{note_id} - Open a specific note
/// - scriptkit://agent-chat/{thread_id} - Open Agent Chat (provenance links)
fn parse_deeplink_url(url: &str) -> Option<String> {
    // Remove the scheme
    let path = url.strip_prefix("scriptkit://")?;

    if path.starts_with("commands/") {
        // Format: scriptkit://commands/app/com.apple.Safari
        // or: scriptkit://commands/builtin/clipboard-history
        // Validate through the canonical command_id_from_deeplink helper
        match crate::config::command_id_from_deeplink(url) {
            Ok(command_id) => return Some(command_id),
            Err(_) => {
                // If it doesn't parse as a supported category, still pass through
                // (runtime-only namespaces like notes/ may appear here)
                let command_id = path.strip_prefix("commands/")?;
                return Some(command_id.to_string());
            }
        }
    }

    if let Some(script_name) = path.strip_prefix("run/") {
        // Legacy format: scriptkit://run/my-script -> script/{name}
        return crate::config::build_command_id(
            crate::config::CommandCategory::Script,
            script_name,
        )
        .ok();
    }

    if let Some(note_id) = path.strip_prefix("notes/") {
        // Notes deeplink - handled specially (runtime-only namespace)
        return Some(format!("notes/{}", note_id));
    }

    if let Some(thread_id) = path.strip_prefix("agent-chat/") {
        // Agent Chat provenance deeplink (from note `source:` frontmatter)
        return Some(format!("agent-chat/{}", thread_id));
    }

    tracing::warn!(url = %url, "unknown_deeplink_format");
    None
}

/// Check if shutdown has been requested (prevents new script spawns during shutdown)
#[allow(dead_code)]
pub fn is_shutting_down() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// Opt-in escape hatch to allow web-triggerable deeplinks to execute scripts.
const ALLOW_DEEPLINK_SCRIPT_EXECUTION_ENV: &str = "SCRIPT_KIT_ALLOW_DEEPLINK_SCRIPTS";

/// Whether a resolved deeplink `command_id` is allowed to run.
///
/// The `scriptkit://` scheme is registered via `on_open_urls`, so ANY web page
/// the user visits can fire one and the OS hands it to the app. UI-opening
/// deeplinks (`app/`, `builtin/`, `notes/`, `agent-chat/`) are low risk, but a
/// deeplink that resolves to a `script/` or `scriptlet/` command — or a bare
/// filesystem path — would silently execute code on a drive-by, which is a real
/// escalation given how many installed scripts touch the filesystem or shells.
///
/// Default-deny those code-executing forms. Users who intentionally rely on
/// `scriptkit://run/...` can opt back in by setting
/// `SCRIPT_KIT_ALLOW_DEEPLINK_SCRIPTS=1`.
pub fn deeplink_execution_allowed(command_id: &str) -> bool {
    if !deeplink_command_executes_code(command_id) {
        return true;
    }
    std::env::var(ALLOW_DEEPLINK_SCRIPT_EXECUTION_ENV)
        .map(|v| {
            let v = v.trim();
            !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
        })
        .unwrap_or(false)
}

/// Whether a resolved deeplink `command_id` would execute user code (rather than
/// just opening a UI surface or launching an already-installed app).
fn deeplink_command_executes_code(command_id: &str) -> bool {
    // Known UI-opening / app-launch categories are safe to trigger from a link.
    const SAFE_PREFIXES: &[&str] = &["app/", "builtin/", "notes/", "agent-chat/"];
    if SAFE_PREFIXES
        .iter()
        .any(|prefix| command_id.starts_with(prefix))
    {
        return false;
    }
    // Everything else — `script/`, `scriptlet/`, and bare paths that
    // `execute_by_command_id_or_path` would run — is treated as code execution.
    true
}

#[cfg(test)]
mod deeplink_gate_tests {
    use super::deeplink_command_executes_code;

    #[test]
    fn ui_opening_deeplinks_do_not_execute_code() {
        for id in [
            "app/com.apple.Safari",
            "builtin/clipboard-history",
            "notes/abc123",
            "agent-chat/thread-1",
        ] {
            assert!(
                !deeplink_command_executes_code(id),
                "{id} should be treated as safe/UI-opening"
            );
        }
    }

    #[test]
    fn script_and_path_deeplinks_execute_code() {
        for id in [
            "script/my-script",
            "scriptlet/some/scriptlet",
            "/Users/me/dev/evil.sh",
            "unknown-namespace/thing",
        ] {
            assert!(
                deeplink_command_executes_code(id),
                "{id} should be treated as code execution"
            );
        }
    }
}
