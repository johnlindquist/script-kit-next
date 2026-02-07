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
    if let Some(ai_config) = cfg.get_ai_hotkey() {
        if let Some((mods, code)) = parse_hotkey_config(&ai_config) {
            let display = hotkey_config_to_display(&ai_config);
            rebind_hotkey_transactional(&manager_guard, HotkeyAction::Ai, mods, code, &display);
        }
    }

    // Update logs hotkey
    if let Some(logs_config) = cfg.get_logs_hotkey() {
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
