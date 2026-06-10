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
