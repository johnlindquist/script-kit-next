impl ScriptHotkeyManager {
    /// Create a new ScriptHotkeyManager.
    /// NOTE: Must be created on the main thread.
    fn new(manager: GlobalHotKeyManager) -> Self {
        Self {
            manager,
            hotkey_map: HashMap::new(),
            path_to_id: HashMap::new(),
            path_to_hotkey: HashMap::new(),
        }
    }

    /// Register a hotkey for a script.
    /// Returns the hotkey ID on success.
    pub fn register(&mut self, path: &str, shortcut: &str) -> anyhow::Result<u32> {
        // Parse the shortcut
        let (mods, code) = shortcuts::parse_shortcut(shortcut)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))?;

        let hotkey = HotKey::new(Some(mods), code);
        let hotkey_id = hotkey.id();

        // Register with the OS - provide specific error messages based on error type
        if let Err(e) = self.manager.register(hotkey) {
            return Err(match e {
                HotkeyError::AlreadyRegistered(hk) => {
                    anyhow::anyhow!(
                        "Hotkey '{}' is already registered (conflict with another app or script). Hotkey ID: {}",
                        shortcut,
                        hk.id()
                    )
                }
                HotkeyError::FailedToRegister(msg) => {
                    anyhow::anyhow!(
                        "System rejected hotkey '{}': {}. This may be reserved by macOS or another app.",
                        shortcut,
                        msg
                    )
                }
                HotkeyError::OsError(os_err) => {
                    anyhow::anyhow!("OS error registering hotkey '{}': {}", shortcut, os_err)
                }
                other => {
                    anyhow::anyhow!("Failed to register hotkey '{}': {}", shortcut, other)
                }
            });
        }

        // Track the mapping
        self.hotkey_map.insert(hotkey_id, path.to_string());
        self.path_to_id.insert(path.to_string(), hotkey_id);
        self.path_to_hotkey.insert(path.to_string(), hotkey);

        logging::log(
            "HOTKEY",
            &format!(
                "Registered script hotkey '{}' for {} (id: {})",
                shortcut, path, hotkey_id
            ),
        );

        Ok(hotkey_id)
    }

    /// Unregister a hotkey for a script by path.
    /// Returns Ok(()) even if the path wasn't registered (no-op).
    pub fn unregister(&mut self, path: &str) -> anyhow::Result<()> {
        if let Some(hotkey_id) = self.path_to_id.remove(path) {
            // Remove from hotkey_map
            self.hotkey_map.remove(&hotkey_id);

            // Unregister from OS using stored HotKey object
            if let Some(hotkey) = self.path_to_hotkey.remove(path) {
                if let Err(e) = self.manager.unregister(hotkey) {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Warning: Failed to unregister hotkey for {} (id: {}): {}",
                            path, hotkey_id, e
                        ),
                    );
                    // Continue anyway - the internal tracking is already updated
                }
            }

            logging::log(
                "HOTKEY",
                &format!(
                    "Unregistered script hotkey for {} (id: {})",
                    path, hotkey_id
                ),
            );
        }
        // If path wasn't registered, this is a no-op (success)
        Ok(())
    }

    /// Update a script's hotkey.
    /// Handles add (old=None, new=Some), remove (old=Some, new=None), and change (both Some).
    pub fn update(
        &mut self,
        path: &str,
        old_shortcut: Option<&str>,
        new_shortcut: Option<&str>,
    ) -> anyhow::Result<()> {
        match (old_shortcut, new_shortcut) {
            (None, None) => {
                // No change needed
                Ok(())
            }
            (None, Some(new)) => {
                // Add new hotkey
                self.register(path, new)?;
                Ok(())
            }
            (Some(_old), None) => {
                // Remove old hotkey
                self.unregister(path)
            }
            (Some(_old), Some(new)) => {
                // Change: unregister old, register new
                self.unregister(path)?;
                self.register(path, new)?;
                Ok(())
            }
        }
    }

    /// Get the script path for a given hotkey ID.
    pub fn get_script_path(&self, hotkey_id: u32) -> Option<&String> {
        self.hotkey_map.get(&hotkey_id)
    }

    /// Get all registered hotkeys as (path, hotkey_id) pairs.
    pub fn get_registered_hotkeys(&self) -> Vec<(String, u32)> {
        self.path_to_id
            .iter()
            .map(|(path, id)| (path.clone(), *id))
            .collect()
    }

    /// Check if a script has a registered hotkey.
    #[allow(dead_code)]
    pub fn is_registered(&self, path: &str) -> bool {
        self.path_to_id.contains_key(path)
    }
}
/// Global singleton for the ScriptHotkeyManager.
/// Initialized when start_hotkey_listener is called.
static SCRIPT_HOTKEY_MANAGER: OnceLock<Mutex<ScriptHotkeyManager>> = OnceLock::new();
/// Initialize the global ScriptHotkeyManager.
/// Must be called from the main thread.
/// Returns an error if already initialized.
#[allow(dead_code)]
pub fn init_script_hotkey_manager(manager: GlobalHotKeyManager) -> anyhow::Result<()> {
    SCRIPT_HOTKEY_MANAGER
        .set(Mutex::new(ScriptHotkeyManager::new(manager)))
        .map_err(|_| anyhow::anyhow!("ScriptHotkeyManager already initialized"))
}
/// Register a script hotkey dynamically.
/// Returns the hotkey ID on success.
pub fn register_script_hotkey(path: &str, shortcut: &str) -> anyhow::Result<u32> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.register(path, shortcut)
}
/// Unregister a script hotkey by path.
/// Returns Ok(()) even if the path wasn't registered (no-op).
pub fn unregister_script_hotkey(path: &str) -> anyhow::Result<()> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.unregister(path)
}
/// Update a script's hotkey.
/// Handles add (old=None, new=Some), remove (old=Some, new=None), and change (both Some).
pub fn update_script_hotkey(
    path: &str,
    old_shortcut: Option<&str>,
    new_shortcut: Option<&str>,
) -> anyhow::Result<()> {
    let manager = SCRIPT_HOTKEY_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("ScriptHotkeyManager not initialized"))?;

    let mut guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.update(path, old_shortcut, new_shortcut)
}
/// Get the script path for a given hotkey ID.
#[allow(dead_code)]
pub fn get_script_for_hotkey(hotkey_id: u32) -> Option<String> {
    let manager = SCRIPT_HOTKEY_MANAGER.get()?;
    let guard = manager.lock().ok()?;
    guard.get_script_path(hotkey_id).cloned()
}
/// Get all registered script hotkeys.
#[allow(dead_code)]
pub fn get_registered_hotkeys() -> Vec<(String, u32)> {
    SCRIPT_HOTKEY_MANAGER
        .get()
        .and_then(|m| m.lock().ok())
        .map(|guard| guard.get_registered_hotkeys())
        .unwrap_or_default()
}
// =============================================================================
// Dynamic shortcut registration (for shortcuts.json overrides)
// =============================================================================

/// Register a shortcut dynamically for a command (scriptlet, builtin, app).
/// This adds the shortcut to the unified routing table and registers with the OS.
///
/// # Arguments
/// * `command_id` - Unique identifier (e.g., "scriptlet/my-scriptlet", "builtin/ai-chat")
/// * `shortcut` - Shortcut string (e.g., "cmd+shift+k")
/// * `display_name` - Human-readable name for logging
///
/// # Returns
/// The hotkey ID on success, or an error if registration fails.
pub fn register_dynamic_shortcut(
    command_id: &str,
    shortcut: &str,
    display_name: &str,
) -> anyhow::Result<u32> {
    let manager = MAIN_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("Hotkey manager not initialized"))?;

    let manager_guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

    let (mods, code) = shortcuts::parse_shortcut(shortcut)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))?;

    let hotkey = HotKey::new(Some(mods), code);
    let id = hotkey.id();

    // Check if already registered
    {
        let routes_guard = routes().read();
        if routes_guard.get_action(id).is_some() {
            return Err(anyhow::anyhow!(
                "Shortcut '{}' is already registered",
                shortcut
            ));
        }
    }

    // Register with OS
    manager_guard
        .register(hotkey)
        .map_err(|e| anyhow::anyhow!("Failed to register hotkey '{}': {}", shortcut, e))?;

    // Add to routing table
    {
        let mut routes_guard = routes().write();
        routes_guard.add_route(
            id,
            RegisteredHotkey {
                hotkey,
                action: HotkeyAction::Script(command_id.to_string()),
                display: shortcut.to_string(),
            },
        );
    }

    logging::log(
        "HOTKEY",
        &format!(
            "Registered dynamic shortcut '{}' for {} (id: {})",
            shortcut, display_name, id
        ),
    );

    Ok(id)
}
/// Unregister a dynamic shortcut by command_id.
/// Returns Ok(()) even if the shortcut wasn't registered (no-op).
#[allow(dead_code)]
pub fn unregister_dynamic_shortcut(command_id: &str) -> anyhow::Result<()> {
    let manager = MAIN_MANAGER
        .get()
        .ok_or_else(|| anyhow::anyhow!("Hotkey manager not initialized"))?;

    let manager_guard = manager
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

    // Find the hotkey ID for this command
    let (id, hotkey) = {
        let routes_guard = routes().read();
        routes_guard
            .get_script_id(command_id)
            .and_then(|id| routes_guard.routes.get(&id).map(|entry| (id, entry.hotkey)))
    }
    .ok_or_else(|| anyhow::anyhow!("No shortcut registered for {}", command_id))?;

    // Unregister from OS
    if let Err(e) = manager_guard.unregister(hotkey) {
        logging::log(
            "HOTKEY",
            &format!(
                "Warning: Failed to unregister hotkey for {}: {}",
                command_id, e
            ),
        );
    }

    // Remove from routing table
    {
        let mut routes_guard = routes().write();
        routes_guard.remove_route(id);
    }

    logging::log(
        "HOTKEY",
        &format!(
            "Unregistered dynamic shortcut for {} (id: {})",
            command_id, id
        ),
    );

    Ok(())
}
// =============================================================================
// GCD dispatch for immediate main-thread execution (bypasses async runtime)
// =============================================================================

use std::sync::Arc;
/// Callback type for hotkey actions - uses Arc<dyn Fn()> for repeated invocation
pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;
/// Static storage for handlers to be invoked on main thread
static NOTES_HANDLER: OnceLock<std::sync::Mutex<Option<HotkeyHandler>>> = OnceLock::new();
static AI_HANDLER: OnceLock<std::sync::Mutex<Option<HotkeyHandler>>> = OnceLock::new();
/// Register a handler to be invoked when the Notes hotkey is pressed.
/// This handler will be executed on the main thread via GCD dispatch_async.
/// The handler can be called multiple times (it's not consumed).
#[allow(dead_code)]
pub fn set_notes_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = NOTES_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    *storage.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(handler));
}
/// Register a handler to be invoked when the AI hotkey is pressed.
/// This handler will be executed on the main thread via GCD dispatch_async.
/// The handler can be called multiple times (it's not consumed).
#[allow(dead_code)]
pub fn set_ai_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = AI_HANDLER.get_or_init(|| std::sync::Mutex::new(None));
    *storage.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(handler));
}
fn clone_hotkey_handler_with_poison_recovery(
    storage: &std::sync::Mutex<Option<HotkeyHandler>>,
    handler_kind: &'static str,
) -> Option<HotkeyHandler> {
    match storage.lock() {
        Ok(handler) => handler.clone(),
        Err(poisoned) => {
            tracing::warn!(
                handler_kind = %handler_kind,
                attempted = "clone_hotkey_handler",
                state = "poisoned_lock",
                "Recovered from poisoned hotkey handler mutex"
            );
            poisoned.into_inner().clone()
        }
    }
}
#[cfg(target_os = "macos")]
mod gcd {
    use std::ffi::c_void;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    // Link to libSystem for GCD functions
    // Note: dispatch_get_main_queue is actually a macro that returns &_dispatch_main_q
    // We use the raw symbol directly instead
    #[link(name = "System", kind = "framework")]
    extern "C" {
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        // The main dispatch queue is a global static symbol, not a function
        #[link_name = "_dispatch_main_q"]
        static DISPATCH_MAIN_QUEUE: c_void;
    }

    /// Dispatch a closure to the main thread via GCD.
    /// This is the key to making hotkeys work before the GPUI event loop is "warmed up".
    ///
    /// SAFETY: The trampoline uses catch_unwind to prevent panics from unwinding
    /// across the FFI boundary, which would be undefined behavior.
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        let boxed: Box<dyn FnOnce() + Send> = Box::new(f);
        let raw = Box::into_raw(Box::new(boxed));

        extern "C" fn trampoline(context: *mut c_void) {
            unsafe {
                let boxed: Box<Box<dyn FnOnce() + Send>> = Box::from_raw(context as *mut _);
                // CRITICAL: Catch panics to prevent UB from unwinding across FFI boundary
                let result = catch_unwind(AssertUnwindSafe(|| {
                    boxed();
                }));
                if let Err(e) = result {
                    // Log the panic but don't propagate it across FFI
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    eprintln!("[HOTKEY] PANIC in GCD dispatch: {}", msg);
                }
            }
        }

        unsafe {
            let main_queue = &DISPATCH_MAIN_QUEUE as *const c_void;
            dispatch_async_f(main_queue, raw as *mut c_void, trampoline);
        }
    }
}
#[cfg(not(target_os = "macos"))]
mod gcd {
    /// Fallback for non-macOS: just call the closure directly (in the current thread)
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        f();
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HotkeyEvent {
    pub correlation_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptHotkeyEvent {
    pub command_id: String,
    pub correlation_id: String,
}
