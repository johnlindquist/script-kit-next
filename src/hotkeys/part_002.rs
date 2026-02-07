/// Dispatch the Notes hotkey handler to the main thread.
///
/// Strategy (mutually exclusive to prevent double-fire):
/// - If a handler is registered: use it directly via GCD dispatch
/// - Otherwise: send to channel for async polling
///
/// This works even before the main window is activated because GCD dispatch
/// directly integrates with the NSApplication run loop that GPUI uses.
#[tracing::instrument(skip_all)]
fn dispatch_notes_hotkey(event: HotkeyEvent) {
    // Check if a direct handler is registered (takes priority over channel)
    let handler_storage = NOTES_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    let handler = clone_hotkey_handler_with_poison_recovery(handler_storage, "notes");

    if let Some(handler) = handler {
        // Handler is set - use direct GCD dispatch (skip channel to avoid double-fire)
        let correlation_id = event.correlation_id.clone();
        gcd::dispatch_to_main(move || {
            let _guard = logging::set_correlation_id(correlation_id);
            handler();
        });
    } else {
        // No handler - use channel approach for async polling
        if notes_hotkey_channel().0.try_send(event).is_err() {
            logging::log("HOTKEY", "Notes hotkey channel full/closed");
        }
        // Dispatch an empty closure to wake GPUI's event loop
        // This ensures the channel message gets processed even if GPUI was idle
        gcd::dispatch_to_main(|| {
            // Empty closure - just wakes the run loop
        });
    }
}
/// Dispatch the AI hotkey handler to the main thread.
///
/// Strategy (mutually exclusive to prevent double-fire):
/// - If a handler is registered: use it directly via GCD dispatch
/// - Otherwise: send to channel for async polling
#[tracing::instrument(skip_all)]
fn dispatch_ai_hotkey(event: HotkeyEvent) {
    // Check if a direct handler is registered (takes priority over channel)
    let handler_storage = AI_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    let handler = clone_hotkey_handler_with_poison_recovery(handler_storage, "ai");

    if let Some(handler) = handler {
        // Handler is set - use direct GCD dispatch (skip channel to avoid double-fire)
        let correlation_id = event.correlation_id.clone();
        gcd::dispatch_to_main(move || {
            let _guard = logging::set_correlation_id(correlation_id);
            handler();
        });
    } else {
        // No handler - use channel approach for async polling
        if ai_hotkey_channel().0.try_send(event).is_err() {
            logging::log("HOTKEY", "AI hotkey channel full/closed");
        }
        // Dispatch an empty closure to wake GPUI's event loop
        gcd::dispatch_to_main(|| {
            // Empty closure - just wakes the run loop
        });
    }
}
// HOTKEY_CHANNEL: Event-driven async_channel for hotkey events (replaces AtomicBool polling)
#[allow(dead_code)]
static HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
)> = OnceLock::new();
/// Get the hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn hotkey_channel() -> &'static (
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
) {
    HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}
// SCRIPT_HOTKEY_CHANNEL: Channel for script shortcut events (sends script path)
#[allow(dead_code)]
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<ScriptHotkeyEvent>,
    async_channel::Receiver<ScriptHotkeyEvent>,
)> = OnceLock::new();
/// Get the script hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn script_hotkey_channel() -> &'static (
    async_channel::Sender<ScriptHotkeyEvent>,
    async_channel::Receiver<ScriptHotkeyEvent>,
) {
    SCRIPT_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}
// NOTES_HOTKEY_CHANNEL: Channel for notes hotkey events
#[allow(dead_code)]
static NOTES_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
)> = OnceLock::new();
/// Get the notes hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn notes_hotkey_channel() -> &'static (
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
) {
    NOTES_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}
// AI_HOTKEY_CHANNEL: Channel for AI hotkey events
#[allow(dead_code)]
static AI_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
)> = OnceLock::new();
/// Get the AI hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn ai_hotkey_channel() -> &'static (
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
) {
    AI_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}
// LOGS_HOTKEY_CHANNEL: Channel for log capture toggle events
#[allow(dead_code)]
static LOGS_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
)> = OnceLock::new();
/// Get the logs hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn logs_hotkey_channel() -> &'static (
    async_channel::Sender<HotkeyEvent>,
    async_channel::Receiver<HotkeyEvent>,
) {
    LOGS_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}
/// Tracks whether the main hotkey was successfully registered
/// Used by main.rs to detect if the app has an alternate entry point
static MAIN_HOTKEY_REGISTERED: AtomicBool = AtomicBool::new(false);
/// Check if the main hotkey was successfully registered
pub fn is_main_hotkey_registered() -> bool {
    MAIN_HOTKEY_REGISTERED.load(Ordering::Relaxed)
}
#[allow(dead_code)]
static HOTKEY_TRIGGER_COUNT: AtomicU64 = AtomicU64::new(0);
/// Format a hotkey registration error with helpful context
fn format_hotkey_error(e: &HotkeyError, shortcut_display: &str) -> String {
    match e {
        HotkeyError::AlreadyRegistered(hk) => {
            format!(
                "Hotkey '{}' is already registered by another application or script (ID: {}). \
                 Try a different shortcut or close the conflicting app.",
                shortcut_display,
                hk.id()
            )
        }
        HotkeyError::FailedToRegister(msg) => {
            format!(
                "System rejected hotkey '{}': {}. This shortcut may be reserved by macOS.",
                shortcut_display, msg
            )
        }
        HotkeyError::OsError(os_err) => {
            format!(
                "OS error registering '{}': {}. Check system hotkey settings.",
                shortcut_display, os_err
            )
        }
        other => format!(
            "Failed to register hotkey '{}': {}",
            shortcut_display, other
        ),
    }
}
/// Register a builtin hotkey (main/notes/ai) and add to unified routing table
fn register_builtin_hotkey(
    manager: &GlobalHotKeyManager,
    action: HotkeyAction,
    cfg: &config::HotkeyConfig,
) -> Option<u32> {
    let (mods, code) = parse_hotkey_config(cfg)?;
    let hotkey = HotKey::new(Some(mods), code);
    let id = hotkey.id();
    let display = hotkey_config_to_display(cfg);

    match manager.register(hotkey) {
        Ok(()) => {
            let mut routes_guard = routes().write();
            routes_guard.add_route(
                id,
                RegisteredHotkey {
                    hotkey,
                    action: action.clone(),
                    display: display.clone(),
                },
            );
            logging::log(
                "HOTKEY",
                &format!("Registered {:?} hotkey {} (id: {})", action, display, id),
            );
            Some(id)
        }
        Err(e) => {
            logging::log("HOTKEY", &format_hotkey_error(&e, &display));
            None
        }
    }
}
/// Register a script hotkey and add to unified routing table
fn register_script_hotkey_internal(
    manager: &GlobalHotKeyManager,
    path: &str,
    shortcut: &str,
    name: &str,
) -> Option<u32> {
    let (mods, code) = shortcuts::parse_shortcut(shortcut)?;
    let hotkey = HotKey::new(Some(mods), code);
    let id = hotkey.id();

    match manager.register(hotkey) {
        Ok(()) => {
            let mut routes_guard = routes().write();
            routes_guard.add_route(
                id,
                RegisteredHotkey {
                    hotkey,
                    action: HotkeyAction::Script(path.to_string()),
                    display: shortcut.to_string(),
                },
            );
            logging::log(
                "HOTKEY",
                &format!(
                    "Registered script shortcut '{}' for {} (id: {})",
                    shortcut, name, id
                ),
            );
            Some(id)
        }
        Err(e) => {
            logging::log(
                "HOTKEY",
                &format!("{} (script: {})", format_hotkey_error(&e, shortcut), name),
            );
            None
        }
    }
}
