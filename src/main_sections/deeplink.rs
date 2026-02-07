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
fn parse_deeplink_url(url: &str) -> Option<String> {
    // Remove the scheme
    let path = url.strip_prefix("scriptkit://")?;

    if let Some(command_id) = path.strip_prefix("commands/") {
        // Format: scriptkit://commands/app/com.apple.Safari
        // or: scriptkit://commands/builtin/clipboard-history
        Some(command_id.to_string())
    } else if let Some(script_name) = path.strip_prefix("run/") {
        // Legacy format: scriptkit://run/my-script -> script/{name}
        Some(format!("script/{}", script_name))
    } else if let Some(note_id) = path.strip_prefix("notes/") {
        // Notes deeplink - handled specially
        Some(format!("notes/{}", note_id))
    } else {
        logging::log("DEEPLINK", &format!("Unknown deeplink format: {}", url));
        None
    }
}

/// Convert our ToastVariant to gpui-component's NotificationType
fn toast_variant_to_notification_type(variant: ToastVariant) -> NotificationType {
    match variant {
        ToastVariant::Success => NotificationType::Success,
        ToastVariant::Warning => NotificationType::Warning,
        ToastVariant::Error => NotificationType::Error,
        ToastVariant::Info => NotificationType::Info,
    }
}

/// Convert a PendingToast to a gpui-component Notification
fn pending_toast_to_notification(toast: &PendingToast) -> Notification {
    let notification_type = toast_variant_to_notification_type(toast.variant);

    let mut notification = Notification::new()
        .message(&toast.message)
        .with_type(notification_type);

    // Add title for errors/warnings (makes them stand out more)
    match toast.variant {
        ToastVariant::Error => {
            notification = notification.title("Error");
        }
        ToastVariant::Warning => {
            notification = notification.title("Warning");
        }
        _ => {}
    }

    // Note: gpui-component Notification has fixed 5s autohide
    // For persistent toasts, set autohide(false)
    if toast.duration_ms.is_none() {
        notification = notification.autohide(false);
    }

    notification
}

/// Check if shutdown has been requested (prevents new script spawns during shutdown)
#[allow(dead_code)]
pub fn is_shutting_down() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}
