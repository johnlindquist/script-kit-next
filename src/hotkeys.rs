use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    Error as HotkeyError, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

use crate::{config, logging, scripts, shortcuts};

// =============================================================================
// Unified Hotkey Routing System
// =============================================================================
// All hotkey events (main, notes, ai, scripts) are dispatched through a single
// routing table. This ensures:
// 1. Consistent dispatch behavior for all hotkey types
// 2. Proper hot-reload support (routing and registration are coupled)
// 3. No lost hotkeys on failed registration (transactional updates)

/// Action to take when a hotkey is pressed
#[derive(Clone, Debug, PartialEq)]
pub enum HotkeyAction {
    /// Main launcher hotkey
    Main,
    /// Notes window hotkey
    Notes,
    /// AI window hotkey
    Ai,
    /// Toggle log capture hotkey
    ToggleLogs,
    /// Script shortcut - run the script at this path
    Script(String),
}

/// Registered hotkey entry with all needed data for unregistration/updates
#[derive(Clone)]
struct RegisteredHotkey {
    /// The HotKey object (needed for unregister)
    hotkey: HotKey,
    /// What action to take on press
    action: HotkeyAction,
    /// Display string for logging (e.g., "cmd+shift+k")
    display: String,
}

/// Unified routing table for all hotkeys
/// Uses RwLock for fast reads (event dispatch) with occasional writes (updates)
struct HotkeyRoutes {
    /// Maps hotkey ID -> registered hotkey entry
    routes: HashMap<u32, RegisteredHotkey>,
    /// Reverse lookup: script path -> hotkey ID (for script updates)
    script_paths: HashMap<String, u32>,
    /// Current main hotkey ID (for quick lookup)
    main_id: Option<u32>,
    /// Current notes hotkey ID (for quick lookup)
    notes_id: Option<u32>,
    /// Current AI hotkey ID (for quick lookup)
    ai_id: Option<u32>,
    /// Current logs hotkey ID (for quick lookup)
    logs_id: Option<u32>,
}

impl HotkeyRoutes {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
            script_paths: HashMap::new(),
            main_id: None,
            notes_id: None,
            ai_id: None,
            logs_id: None,
        }
    }

    /// Get the action for a hotkey ID
    fn get_action(&self, id: u32) -> Option<HotkeyAction> {
        self.routes.get(&id).map(|r| r.action.clone())
    }

    /// Add a route (internal - doesn't register with OS)
    fn add_route(&mut self, id: u32, entry: RegisteredHotkey) {
        match &entry.action {
            HotkeyAction::Main => self.main_id = Some(id),
            HotkeyAction::Notes => self.notes_id = Some(id),
            HotkeyAction::Ai => self.ai_id = Some(id),
            HotkeyAction::ToggleLogs => self.logs_id = Some(id),
            HotkeyAction::Script(path) => {
                self.script_paths.insert(path.clone(), id);
            }
        }
        self.routes.insert(id, entry);
    }

    /// Remove a route by ID (internal - doesn't unregister from OS)
    fn remove_route(&mut self, id: u32) -> Option<RegisteredHotkey> {
        if let Some(entry) = self.routes.remove(&id) {
            match &entry.action {
                HotkeyAction::Main => {
                    if self.main_id == Some(id) {
                        self.main_id = None;
                    }
                }
                HotkeyAction::Notes => {
                    if self.notes_id == Some(id) {
                        self.notes_id = None;
                    }
                }
                HotkeyAction::Ai => {
                    if self.ai_id == Some(id) {
                        self.ai_id = None;
                    }
                }
                HotkeyAction::ToggleLogs => {
                    if self.logs_id == Some(id) {
                        self.logs_id = None;
                    }
                }
                HotkeyAction::Script(path) => {
                    self.script_paths.remove(path);
                }
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Get script hotkey ID by path
    fn get_script_id(&self, path: &str) -> Option<u32> {
        self.script_paths.get(path).copied()
    }

    /// Get the hotkey entry for an action type
    #[allow(dead_code)]
    fn get_builtin_entry(&self, action: &HotkeyAction) -> Option<&RegisteredHotkey> {
        let id = match action {
            HotkeyAction::Main => self.main_id?,
            HotkeyAction::Notes => self.notes_id?,
            HotkeyAction::Ai => self.ai_id?,
            HotkeyAction::ToggleLogs => self.logs_id?,
            HotkeyAction::Script(path) => *self.script_paths.get(path)?,
        };
        self.routes.get(&id)
    }
}

/// Global routing table - protected by RwLock for fast reads
static HOTKEY_ROUTES: OnceLock<RwLock<HotkeyRoutes>> = OnceLock::new();

fn routes() -> &'static RwLock<HotkeyRoutes> {
    HOTKEY_ROUTES.get_or_init(|| RwLock::new(HotkeyRoutes::new()))
}

/// The main GlobalHotKeyManager - stored globally so update_hotkeys can access it
static MAIN_MANAGER: OnceLock<Mutex<GlobalHotKeyManager>> = OnceLock::new();

/// Parse a HotkeyConfig into (Modifiers, Code)
fn parse_hotkey_config(hk: &config::HotkeyConfig) -> Option<(Modifiers, Code)> {
    let code = match hk.key.as_str() {
        "Semicolon" => Code::Semicolon,
        "KeyK" => Code::KeyK,
        "KeyP" => Code::KeyP,
        "Space" => Code::Space,
        "Enter" => Code::Enter,
        "Digit0" => Code::Digit0,
        "Digit1" => Code::Digit1,
        "Digit2" => Code::Digit2,
        "Digit3" => Code::Digit3,
        "Digit4" => Code::Digit4,
        "Digit5" => Code::Digit5,
        "Digit6" => Code::Digit6,
        "Digit7" => Code::Digit7,
        "Digit8" => Code::Digit8,
        "Digit9" => Code::Digit9,
        "KeyA" => Code::KeyA,
        "KeyB" => Code::KeyB,
        "KeyC" => Code::KeyC,
        "KeyD" => Code::KeyD,
        "KeyE" => Code::KeyE,
        "KeyF" => Code::KeyF,
        "KeyG" => Code::KeyG,
        "KeyH" => Code::KeyH,
        "KeyI" => Code::KeyI,
        "KeyJ" => Code::KeyJ,
        "KeyL" => Code::KeyL,
        "KeyM" => Code::KeyM,
        "KeyN" => Code::KeyN,
        "KeyO" => Code::KeyO,
        "KeyQ" => Code::KeyQ,
        "KeyR" => Code::KeyR,
        "KeyS" => Code::KeyS,
        "KeyT" => Code::KeyT,
        "KeyU" => Code::KeyU,
        "KeyV" => Code::KeyV,
        "KeyW" => Code::KeyW,
        "KeyX" => Code::KeyX,
        "KeyY" => Code::KeyY,
        "KeyZ" => Code::KeyZ,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        _ => return None,
    };

    let mut modifiers = Modifiers::empty();
    for modifier in &hk.modifiers {
        match modifier.as_str() {
            "meta" => modifiers |= Modifiers::META,
            "ctrl" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            _ => {}
        }
    }

    Some((modifiers, code))
}

/// Convert a HotkeyConfig to a display string (e.g., "meta+shift+N")
fn hotkey_config_to_display(hk: &config::HotkeyConfig) -> String {
    format!(
        "{}{}{}",
        hk.modifiers.join("+"),
        if hk.modifiers.is_empty() { "" } else { "+" },
        hk.key
    )
}

/// Transactional hotkey rebind: register new BEFORE unregistering old
/// This prevents losing a working hotkey if the new registration fails
#[tracing::instrument(skip(manager, display), fields(action = ?action))]
fn rebind_hotkey_transactional(
    manager: &GlobalHotKeyManager,
    action: HotkeyAction,
    mods: Modifiers,
    code: Code,
    display: &str,
) -> bool {
    let new_hotkey = HotKey::new(Some(mods), code);
    let new_id = new_hotkey.id();

    // Check if already registered with same ID (no change needed)
    let current_id = {
        let routes_guard = routes().read();
        match &action {
            HotkeyAction::Main => routes_guard.main_id,
            HotkeyAction::Notes => routes_guard.notes_id,
            HotkeyAction::Ai => routes_guard.ai_id,
            HotkeyAction::ToggleLogs => routes_guard.logs_id,
            HotkeyAction::Script(path) => routes_guard.get_script_id(path),
        }
    };

    if current_id == Some(new_id) {
        return true; // No change needed
    }

    // TRANSACTIONAL: Register new FIRST, before unregistering old
    // This ensures we never lose a working hotkey on registration failure
    if let Err(e) = manager.register(new_hotkey) {
        logging::log(
            "HOTKEY",
            &format!("Failed to register {}: {} - keeping existing", display, e),
        );
        return false;
    }

    // New registration succeeded - now safe to update routing and unregister old
    let old_entry = {
        let mut routes_guard = routes().write();

        // Get old entry before adding new (they might have same action type)
        let old_id = match &action {
            HotkeyAction::Main => routes_guard.main_id,
            HotkeyAction::Notes => routes_guard.notes_id,
            HotkeyAction::Ai => routes_guard.ai_id,
            HotkeyAction::ToggleLogs => routes_guard.logs_id,
            HotkeyAction::Script(path) => routes_guard.get_script_id(path),
        };
        let old_entry = old_id.and_then(|id| routes_guard.remove_route(id));

        // Add new route
        routes_guard.add_route(
            new_id,
            RegisteredHotkey {
                hotkey: new_hotkey,
                action: action.clone(),
                display: display.to_string(),
            },
        );

        old_entry
    };

    // Unregister old hotkey (best-effort - it's already removed from routing)
    if let Some(old) = old_entry {
        if let Err(e) = manager.unregister(old.hotkey) {
            logging::log(
                "HOTKEY",
                &format!(
                    "Warning: failed to unregister old {} hotkey: {}",
                    old.display, e
                ),
            );
            // Continue anyway - new hotkey is working
        }
    }

    logging::log(
        "HOTKEY",
        &format!(
            "Hot-reloaded {:?} hotkey: {} (id: {})",
            action, display, new_id
        ),
    );
    true
}

/// Update hotkeys from config - call this when config changes
/// Uses transactional updates: register new before unregistering old
#[tracing::instrument(skip_all)]
pub fn update_hotkeys(cfg: &config::Config) {
    let manager_guard = match MAIN_MANAGER.get() {
        Some(m) => match m.lock() {
            Ok(g) => g,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to lock manager: {}", e));
                return;
            }
        },
        None => {
            logging::log("HOTKEY", "Manager not initialized - hotkeys not updated");
            return;
        }
    };

    // Update main hotkey
    let main_config = &cfg.hotkey;
    if let Some((mods, code)) = parse_hotkey_config(main_config) {
        let display = hotkey_config_to_display(main_config);
        let success =
            rebind_hotkey_transactional(&manager_guard, HotkeyAction::Main, mods, code, &display);
        MAIN_HOTKEY_REGISTERED.store(success, Ordering::Relaxed);
    }

    // Update notes hotkey (only if configured - no default)
    if let Some(notes_config) = cfg.get_notes_hotkey() {
        if let Some((mods, code)) = parse_hotkey_config(&notes_config) {
            let display = hotkey_config_to_display(&notes_config);
            rebind_hotkey_transactional(&manager_guard, HotkeyAction::Notes, mods, code, &display);
        }
    }

    // Update AI hotkey
    let ai_config = cfg.get_ai_hotkey();
    if let Some((mods, code)) = parse_hotkey_config(&ai_config) {
        let display = hotkey_config_to_display(&ai_config);
        rebind_hotkey_transactional(&manager_guard, HotkeyAction::Ai, mods, code, &display);
    }

    // Update logs hotkey
    let logs_config = cfg.get_logs_hotkey();
    if let Some((mods, code)) = parse_hotkey_config(&logs_config) {
        let display = hotkey_config_to_display(&logs_config);
        rebind_hotkey_transactional(
            &manager_guard,
            HotkeyAction::ToggleLogs,
            mods,
            code,
            &display,
        );
    }
}

// =============================================================================
// Dynamic Script Hotkey Manager
// =============================================================================

/// Manages dynamic registration/unregistration of script hotkeys.
/// Uses a thread-safe global singleton pattern for access from multiple contexts.
pub struct ScriptHotkeyManager {
    /// The underlying global hotkey manager
    manager: GlobalHotKeyManager,
    /// Maps hotkey ID -> script path
    hotkey_map: HashMap<u32, String>,
    /// Maps script path -> hotkey ID (reverse lookup for unregistration)
    path_to_id: HashMap<String, u32>,
    /// Maps script path -> HotKey object (needed for proper unregistration)
    path_to_hotkey: HashMap<String, HotKey>,
}

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

/// Dispatch the Notes hotkey handler to the main thread.
///
/// Strategy (mutually exclusive to prevent double-fire):
/// - If a handler is registered: use it directly via GCD dispatch
/// - Otherwise: send to channel for async polling
///
/// This works even before the main window is activated because GCD dispatch
/// directly integrates with the NSApplication run loop that GPUI uses.
#[tracing::instrument(skip_all)]
fn dispatch_notes_hotkey() {
    // Check if a direct handler is registered (takes priority over channel)
    let handler = NOTES_HANDLER
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap()
        .clone();

    if let Some(handler) = handler {
        // Handler is set - use direct GCD dispatch (skip channel to avoid double-fire)
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // No handler - use channel approach for async polling
        if notes_hotkey_channel().0.try_send(()).is_err() {
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
fn dispatch_ai_hotkey() {
    // Check if a direct handler is registered (takes priority over channel)
    let handler = AI_HANDLER
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .unwrap()
        .clone();

    if let Some(handler) = handler {
        // Handler is set - use direct GCD dispatch (skip channel to avoid double-fire)
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // No handler - use channel approach for async polling
        if ai_hotkey_channel().0.try_send(()).is_err() {
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
static HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn hotkey_channel() -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>)
{
    HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// SCRIPT_HOTKEY_CHANNEL: Channel for script shortcut events (sends script path)
#[allow(dead_code)]
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
)> = OnceLock::new();

/// Get the script hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn script_hotkey_channel() -> &'static (
    async_channel::Sender<String>,
    async_channel::Receiver<String>,
) {
    SCRIPT_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// NOTES_HOTKEY_CHANNEL: Channel for notes hotkey events
#[allow(dead_code)]
static NOTES_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the notes hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn notes_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    NOTES_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// AI_HOTKEY_CHANNEL: Channel for AI hotkey events
#[allow(dead_code)]
static AI_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the AI hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn ai_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
    AI_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// LOGS_HOTKEY_CHANNEL: Channel for log capture toggle events
#[allow(dead_code)]
static LOGS_HOTKEY_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
    OnceLock::new();

/// Get the logs hotkey channel, initializing it on first access.
#[allow(dead_code)]
pub(crate) fn logs_hotkey_channel(
) -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
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

#[allow(dead_code)]
pub(crate) fn start_hotkey_listener(config: config::Config) {
    std::thread::spawn(move || {
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to create hotkey manager: {}", e));
                return;
            }
        };

        if MAIN_MANAGER.set(Mutex::new(manager)).is_err() {
            logging::log("HOTKEY", "Manager already initialized (unexpected)");
            return;
        }

        let manager_guard = match MAIN_MANAGER.get().unwrap().lock() {
            Ok(g) => g,
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to lock manager: {}", e));
                return;
            }
        };

        // Register main hotkey using unified registration
        if register_builtin_hotkey(&manager_guard, HotkeyAction::Main, &config.hotkey).is_some() {
            MAIN_HOTKEY_REGISTERED.store(true, Ordering::Relaxed);
        }

        // Register notes hotkey (only if configured - no default)
        if let Some(notes_hotkey) = config.get_notes_hotkey() {
            register_builtin_hotkey(&manager_guard, HotkeyAction::Notes, &notes_hotkey);
        }
        // Register AI and logs hotkeys
        register_builtin_hotkey(&manager_guard, HotkeyAction::Ai, &config.get_ai_hotkey());
        register_builtin_hotkey(
            &manager_guard,
            HotkeyAction::ToggleLogs,
            &config.get_logs_hotkey(),
        );

        // Register script shortcuts
        let mut script_count = 0;

        let all_scripts = scripts::read_scripts();
        for script in &all_scripts {
            if let Some(ref shortcut) = script.shortcut {
                let path = script.path.to_string_lossy().to_string();
                if register_script_hotkey_internal(&manager_guard, &path, shortcut, &script.name)
                    .is_some()
                {
                    script_count += 1;
                }
            }
        }

        let all_scriptlets = scripts::load_scriptlets();
        for scriptlet in &all_scriptlets {
            if let Some(ref shortcut) = scriptlet.shortcut {
                let path = scriptlet
                    .file_path
                    .clone()
                    .unwrap_or_else(|| scriptlet.name.clone());
                if register_script_hotkey_internal(&manager_guard, &path, shortcut, &scriptlet.name)
                    .is_some()
                {
                    script_count += 1;
                }
            }
        }

        logging::log(
            "HOTKEY",
            &format!("Registered {} script/scriptlet shortcuts", script_count),
        );

        // Track which command IDs have been registered to avoid duplicates
        let mut registered_commands: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // Priority 1: Load shortcuts from config.ts commands field (highest priority)
        let mut config_count = 0;
        if let Some(commands) = &config.commands {
            for (command_id, cmd_config) in commands {
                if let Some(hotkey_config) = &cmd_config.shortcut {
                    let shortcut_str = hotkey_config.to_shortcut_string();
                    if register_script_hotkey_internal(
                        &manager_guard,
                        command_id,
                        &shortcut_str,
                        command_id,
                    )
                    .is_some()
                    {
                        registered_commands.insert(command_id.clone());
                        config_count += 1;
                    }
                }
            }
            if config_count > 0 {
                logging::log(
                    "HOTKEY",
                    &format!(
                        "Registered {} command shortcuts from config.ts",
                        config_count
                    ),
                );
            }
        }

        // Priority 2: Load user shortcut overrides from shortcuts.json
        // (skips commands already registered from config.ts)
        let mut override_count = 0;
        match crate::shortcuts::load_shortcut_overrides() {
            Ok(overrides) => {
                for (command_id, shortcut) in overrides {
                    // Skip if already registered from config.ts
                    if registered_commands.contains(&command_id) {
                        logging::log(
                            "HOTKEY",
                            &format!(
                                "Skipping shortcuts.json entry for '{}' (config.ts takes priority)",
                                command_id
                            ),
                        );
                        continue;
                    }

                    let shortcut_str = shortcut.to_canonical_string();
                    if register_script_hotkey_internal(
                        &manager_guard,
                        &command_id,
                        &shortcut_str,
                        &command_id,
                    )
                    .is_some()
                    {
                        override_count += 1;
                    }
                }
                if override_count > 0 {
                    logging::log(
                        "HOTKEY",
                        &format!(
                            "Registered {} user shortcut overrides from shortcuts.json",
                            override_count
                        ),
                    );
                }
            }
            Err(e) => {
                logging::log("HOTKEY", &format!("Failed to load shortcuts.json: {}", e));
            }
        }

        // Log routing table summary
        {
            let routes_guard = routes().read();
            logging::log(
                "HOTKEY",
                &format!(
                    "Routing table: main={:?}, notes={:?}, ai={:?}, logs={:?}, scripts={}",
                    routes_guard.main_id,
                    routes_guard.notes_id,
                    routes_guard.ai_id,
                    routes_guard.logs_id,
                    routes_guard.script_paths.len()
                ),
            );
        }

        drop(manager_guard);
        let receiver = GlobalHotKeyEvent::receiver();

        loop {
            if let Ok(event) = receiver.recv() {
                if event.state != HotKeyState::Pressed {
                    continue;
                }

                // Look up action in unified routing table (fast read lock)
                let action = {
                    let routes_guard = routes().read();
                    routes_guard.get_action(event.id)
                };

                match action {
                    Some(HotkeyAction::Main) => {
                        // Set correlation ID for this hotkey event
                        let _guard =
                            logging::set_correlation_id(format!("hotkey:main:{}", Uuid::new_v4()));

                        let count = HOTKEY_TRIGGER_COUNT.fetch_add(1, Ordering::Relaxed);
                        // NON-BLOCKING: Use try_send to prevent hotkey thread from blocking
                        if hotkey_channel().0.try_send(()).is_err() {
                            logging::log("HOTKEY", "Main hotkey channel full/closed");
                        }
                        logging::log(
                            "HOTKEY",
                            &format!("Main hotkey pressed (trigger #{})", count + 1),
                        );
                    }
                    Some(HotkeyAction::Notes) => {
                        // Set correlation ID for this hotkey event
                        let _guard =
                            logging::set_correlation_id(format!("hotkey:notes:{}", Uuid::new_v4()));

                        logging::log(
                            "HOTKEY",
                            "Notes hotkey pressed - dispatching to main thread",
                        );
                        dispatch_notes_hotkey();
                    }
                    Some(HotkeyAction::Ai) => {
                        // Set correlation ID for this hotkey event
                        let _guard =
                            logging::set_correlation_id(format!("hotkey:ai:{}", Uuid::new_v4()));

                        logging::log("HOTKEY", "AI hotkey pressed - dispatching to main thread");
                        dispatch_ai_hotkey();
                    }
                    Some(HotkeyAction::ToggleLogs) => {
                        // Set correlation ID for this hotkey event
                        let _guard =
                            logging::set_correlation_id(format!("hotkey:logs:{}", Uuid::new_v4()));

                        logging::log("HOTKEY", "Logs hotkey pressed - toggling log capture");
                        // Toggle capture immediately (no need for main thread dispatch)
                        let (is_capturing, path) = logging::toggle_capture();
                        if is_capturing {
                            if let Some(p) = path {
                                logging::log(
                                    "HOTKEY",
                                    &format!("Log capture STARTED: {}", p.display()),
                                );
                            }
                        } else if let Some(p) = path {
                            logging::log(
                                "HOTKEY",
                                &format!("Log capture STOPPED: {}", p.display()),
                            );
                        }
                        // Send to channel for UI notification (HUD)
                        if logs_hotkey_channel().0.try_send(()).is_err() {
                            logging::log("HOTKEY", "Logs hotkey channel full/closed");
                        }
                    }
                    Some(HotkeyAction::Script(path)) => {
                        // Set correlation ID for this hotkey event
                        let _guard = logging::set_correlation_id(format!(
                            "hotkey:script:{}:{}",
                            path,
                            Uuid::new_v4()
                        ));

                        // Start benchmark timing for hotkey → chat latency analysis
                        logging::bench_start(&format!("hotkey:{}", path));
                        logging::log("HOTKEY", &format!("Script shortcut triggered: {}", path));
                        // NON-BLOCKING: Use try_send to prevent hotkey thread from blocking
                        if script_hotkey_channel().0.try_send(path.clone()).is_err() {
                            logging::log(
                                "HOTKEY",
                                &format!("Script channel full/closed for {}", path),
                            );
                        }
                    }
                    None => {
                        // Set correlation ID even for unknown hotkey events
                        let _guard = logging::set_correlation_id(format!(
                            "hotkey:unknown:{}",
                            Uuid::new_v4()
                        ));

                        // Unknown hotkey ID - can happen during hot-reload transitions
                        logging::log("HOTKEY", &format!("Unknown hotkey event id={}", event.id));
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_channel::TryRecvError;

    // =============================================================================
    // Unified Routing Table Tests
    // =============================================================================
    mod routing_table_tests {
        use super::*;

        #[test]
        fn test_hotkey_routes_new() {
            let routes = HotkeyRoutes::new();
            assert!(routes.routes.is_empty());
            assert!(routes.script_paths.is_empty());
            assert!(routes.main_id.is_none());
            assert!(routes.notes_id.is_none());
            assert!(routes.ai_id.is_none());
        }

        #[test]
        fn test_add_main_route() {
            let mut routes = HotkeyRoutes::new();
            let hotkey = HotKey::new(Some(Modifiers::META), Code::Semicolon);
            let entry = RegisteredHotkey {
                hotkey,
                action: HotkeyAction::Main,
                display: "cmd+;".to_string(),
            };
            routes.add_route(hotkey.id(), entry);

            assert_eq!(routes.main_id, Some(hotkey.id()));
            assert!(routes.routes.contains_key(&hotkey.id()));
            assert_eq!(routes.get_action(hotkey.id()), Some(HotkeyAction::Main));
        }

        #[test]
        fn test_add_script_route() {
            let mut routes = HotkeyRoutes::new();
            let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyT);
            let path = "/test/script.ts".to_string();
            let entry = RegisteredHotkey {
                hotkey,
                action: HotkeyAction::Script(path.clone()),
                display: "cmd+shift+t".to_string(),
            };
            routes.add_route(hotkey.id(), entry);

            assert_eq!(routes.script_paths.get(&path), Some(&hotkey.id()));
            assert_eq!(routes.get_script_id(&path), Some(hotkey.id()));
            assert_eq!(
                routes.get_action(hotkey.id()),
                Some(HotkeyAction::Script(path))
            );
        }

        #[test]
        fn test_remove_route() {
            let mut routes = HotkeyRoutes::new();
            let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyN);
            let entry = RegisteredHotkey {
                hotkey,
                action: HotkeyAction::Notes,
                display: "cmd+n".to_string(),
            };
            routes.add_route(hotkey.id(), entry);
            assert!(routes.notes_id.is_some());

            let removed = routes.remove_route(hotkey.id());
            assert!(removed.is_some());
            assert!(routes.notes_id.is_none());
            assert!(routes.get_action(hotkey.id()).is_none());
        }

        #[test]
        fn test_remove_script_route() {
            let mut routes = HotkeyRoutes::new();
            let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyS);
            let path = "/test/script.ts".to_string();
            let entry = RegisteredHotkey {
                hotkey,
                action: HotkeyAction::Script(path.clone()),
                display: "cmd+s".to_string(),
            };
            routes.add_route(hotkey.id(), entry);
            assert!(routes.script_paths.contains_key(&path));

            routes.remove_route(hotkey.id());
            assert!(!routes.script_paths.contains_key(&path));
        }

        #[test]
        fn test_hotkey_action_equality() {
            assert_eq!(HotkeyAction::Main, HotkeyAction::Main);
            assert_eq!(HotkeyAction::Notes, HotkeyAction::Notes);
            assert_eq!(HotkeyAction::Ai, HotkeyAction::Ai);
            assert_eq!(HotkeyAction::ToggleLogs, HotkeyAction::ToggleLogs);
            assert_eq!(
                HotkeyAction::Script("/a.ts".to_string()),
                HotkeyAction::Script("/a.ts".to_string())
            );
            assert_ne!(HotkeyAction::Main, HotkeyAction::Notes);
            assert_ne!(HotkeyAction::Main, HotkeyAction::ToggleLogs);
            assert_ne!(
                HotkeyAction::Script("/a.ts".to_string()),
                HotkeyAction::Script("/b.ts".to_string())
            );
        }
    }

    #[test]
    fn hotkey_channels_are_independent() {
        while hotkey_channel().1.try_recv().is_ok() {}
        while script_hotkey_channel().1.try_recv().is_ok() {}

        hotkey_channel().0.send_blocking(()).expect("send hotkey");
        assert!(matches!(
            script_hotkey_channel().1.try_recv(),
            Err(TryRecvError::Empty)
        ));
        assert!(hotkey_channel().1.try_recv().is_ok());

        script_hotkey_channel()
            .0
            .send_blocking("script".to_string())
            .expect("send script hotkey");
        assert_eq!(
            script_hotkey_channel()
                .1
                .try_recv()
                .expect("recv script hotkey"),
            "script"
        );
    }

    // =============================================================================
    // ScriptHotkeyManager Unit Tests
    // =============================================================================
    // Note: These tests cannot actually register system hotkeys in the test environment
    // because GlobalHotKeyManager requires a running event loop and proper OS permissions.
    // Instead, we test the logic of the manager's internal tracking.

    mod script_hotkey_manager_tests {
        use super::*;

        /// Helper to create a manager for testing.
        /// Note: Registration will fail without an event loop, but we can test tracking logic.
        fn create_test_manager() -> Option<ScriptHotkeyManager> {
            // GlobalHotKeyManager::new() may fail in test environment
            GlobalHotKeyManager::new()
                .ok()
                .map(ScriptHotkeyManager::new)
        }

        #[test]
        fn test_manager_creation() {
            // Just verify we can create the struct (manager creation may fail in CI)
            if let Some(manager) = create_test_manager() {
                assert!(manager.hotkey_map.is_empty());
                assert!(manager.path_to_id.is_empty());
            }
        }

        #[test]
        fn test_get_registered_hotkeys_empty() {
            if let Some(manager) = create_test_manager() {
                assert!(manager.get_registered_hotkeys().is_empty());
            }
        }

        #[test]
        fn test_is_registered_false_for_unknown_path() {
            if let Some(manager) = create_test_manager() {
                assert!(!manager.is_registered("/some/unknown/path.ts"));
            }
        }

        #[test]
        fn test_unregister_nonexistent_is_noop() {
            if let Some(mut manager) = create_test_manager() {
                // Should not error when unregistering a path that was never registered
                let result = manager.unregister("/nonexistent/path.ts");
                assert!(result.is_ok());
            }
        }

        #[test]
        fn test_update_none_to_none_is_noop() {
            if let Some(mut manager) = create_test_manager() {
                // No old, no new -> no-op, should succeed
                let result = manager.update("/some/path.ts", None, None);
                assert!(result.is_ok());
            }
        }

        // Note: The following tests would require a working GlobalHotKeyManager
        // which may not be available in all test environments.
        // In a real CI environment, these would be integration tests.

        #[test]
        fn test_register_tracks_mapping() {
            if let Some(mut manager) = create_test_manager() {
                // Try to register - this may fail in test environment, that's OK
                let result = manager.register("/test/script.ts", "cmd+shift+t");
                if result.is_ok() {
                    // If registration succeeded, verify tracking
                    assert!(manager.is_registered("/test/script.ts"));
                    let hotkeys = manager.get_registered_hotkeys();
                    assert_eq!(hotkeys.len(), 1);
                    assert_eq!(hotkeys[0].0, "/test/script.ts");
                }
                // If it failed (no event loop), that's expected in test env
            }
        }

        #[test]
        fn test_unregister_removes_tracking() {
            if let Some(mut manager) = create_test_manager() {
                // Try to register first
                if manager.register("/test/script.ts", "cmd+shift+u").is_ok() {
                    assert!(manager.is_registered("/test/script.ts"));

                    // Now unregister
                    let result = manager.unregister("/test/script.ts");
                    assert!(result.is_ok());
                    assert!(!manager.is_registered("/test/script.ts"));
                }
            }
        }

        #[test]
        fn test_update_add_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // None -> Some = add
                let result = manager.update("/test/add.ts", None, Some("cmd+shift+a"));
                if result.is_ok() {
                    assert!(manager.is_registered("/test/add.ts"));
                }
            }
        }

        #[test]
        fn test_update_remove_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // First register
                if manager.register("/test/remove.ts", "cmd+shift+r").is_ok() {
                    // Some -> None = remove
                    let result = manager.update("/test/remove.ts", Some("cmd+shift+r"), None);
                    assert!(result.is_ok());
                    assert!(!manager.is_registered("/test/remove.ts"));
                }
            }
        }

        #[test]
        fn test_update_change_hotkey() {
            if let Some(mut manager) = create_test_manager() {
                // First register with old shortcut
                if manager.register("/test/change.ts", "cmd+shift+c").is_ok() {
                    // Some -> Some (different) = change
                    let result =
                        manager.update("/test/change.ts", Some("cmd+shift+c"), Some("cmd+alt+c"));
                    if result.is_ok() {
                        // Should still be registered (with new shortcut)
                        assert!(manager.is_registered("/test/change.ts"));
                    }
                }
            }
        }

        #[test]
        fn test_get_script_path() {
            if let Some(mut manager) = create_test_manager() {
                if let Ok(hotkey_id) = manager.register("/test/lookup.ts", "cmd+shift+l") {
                    let path = manager.get_script_path(hotkey_id);
                    assert_eq!(path, Some(&"/test/lookup.ts".to_string()));

                    // Unknown ID returns None
                    assert!(manager.get_script_path(99999).is_none());
                }
            }
        }
    }
}
