# Hotkey System Architecture - Expert Bundle

## Overview

Script Kit uses a unified hotkey routing system that handles main launcher, Notes, AI, and script shortcuts through a single dispatch mechanism.

## Architecture (src/hotkeys.rs)

### Hotkey Actions

```rust
/// Action to take when a hotkey is pressed
#[derive(Clone, Debug, PartialEq)]
pub enum HotkeyAction {
    /// Main launcher hotkey
    Main,
    /// Notes window hotkey
    Notes,
    /// AI window hotkey
    Ai,
    /// Script shortcut - run the script at this path
    Script(String),
}
```

### Registered Hotkey Entry

```rust
/// Entry with all needed data for unregistration/updates
#[derive(Clone)]
struct RegisteredHotkey {
    /// The HotKey object (needed for unregister)
    hotkey: HotKey,
    /// What action to take on press
    action: HotkeyAction,
    /// Display string for logging (e.g., "cmd+shift+k")
    display: String,
}
```

### Unified Routing Table

```rust
/// Routing table for all hotkeys
struct HotkeyRoutes {
    /// Maps hotkey ID -> registered hotkey entry
    routes: HashMap<u32, RegisteredHotkey>,
    /// Reverse lookup: script path -> hotkey ID
    script_paths: HashMap<String, u32>,
    /// Quick lookups for builtin hotkeys
    main_id: Option<u32>,
    notes_id: Option<u32>,
    ai_id: Option<u32>,
}

/// Global routing table - RwLock for fast reads
static HOTKEY_ROUTES: OnceLock<RwLock<HotkeyRoutes>> = OnceLock::new();

fn routes() -> &'static RwLock<HotkeyRoutes> {
    HOTKEY_ROUTES.get_or_init(|| RwLock::new(HotkeyRoutes::new()))
}
```

## Hotkey Configuration

### Config Structure

```rust
// In config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,  // ["meta", "shift"]
    pub key: String,             // "Semicolon", "KeyK", etc.
}
```

### Parsing Config to HotKey

```rust
fn parse_hotkey_config(hk: &HotkeyConfig) -> Option<(Modifiers, Code)> {
    let code = match hk.key.as_str() {
        "Semicolon" => Code::Semicolon,
        "Space" => Code::Space,
        "KeyK" => Code::KeyK,
        "KeyN" => Code::KeyN,
        // ... all key codes
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
```

## Transactional Hot-Reload

### Safe Rebinding

```rust
/// Transactional hotkey rebind: register new BEFORE unregistering old
/// Prevents losing a working hotkey if new registration fails
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
        let routes_guard = routes().read().unwrap();
        match &action {
            HotkeyAction::Main => routes_guard.main_id,
            HotkeyAction::Notes => routes_guard.notes_id,
            HotkeyAction::Ai => routes_guard.ai_id,
            HotkeyAction::Script(path) => routes_guard.get_script_id(path),
        }
    };

    if current_id == Some(new_id) {
        return true; // No change needed
    }

    // TRANSACTIONAL: Register new FIRST
    if let Err(e) = manager.register(new_hotkey) {
        logging::log("HOTKEY", &format!(
            "Failed to register {}: {} - keeping existing", display, e
        ));
        return false;
    }

    // New succeeded - now safe to update routing and unregister old
    let old_entry = {
        let mut routes_guard = routes().write().unwrap();
        let old_id = match &action {
            HotkeyAction::Main => routes_guard.main_id,
            HotkeyAction::Notes => routes_guard.notes_id,
            HotkeyAction::Ai => routes_guard.ai_id,
            HotkeyAction::Script(path) => routes_guard.get_script_id(path),
        };
        let old_entry = old_id.and_then(|id| routes_guard.remove_route(id));

        // Add new route
        routes_guard.add_route(new_id, RegisteredHotkey {
            hotkey: new_hotkey,
            action: action.clone(),
            display: display.to_string(),
        });

        old_entry
    };

    // Unregister old (best-effort)
    if let Some(old) = old_entry {
        let _ = manager.unregister(old.hotkey);
    }

    true
}
```

## Event Loop

### Hotkey Listener Thread

```rust
pub fn start_hotkey_listener(config: Config) {
    std::thread::spawn(move || {
        let manager = GlobalHotKeyManager::new().expect("Failed to create manager");
        
        // Store manager for hot-reload access
        let _ = MAIN_MANAGER.set(Mutex::new(manager));
        
        let manager_guard = MAIN_MANAGER.get().unwrap().lock().unwrap();

        // Register builtin hotkeys
        register_builtin_hotkey(&manager_guard, HotkeyAction::Main, &config.hotkey);
        register_builtin_hotkey(&manager_guard, HotkeyAction::Notes, &config.get_notes_hotkey());
        register_builtin_hotkey(&manager_guard, HotkeyAction::Ai, &config.get_ai_hotkey());

        // Register script shortcuts
        for script in scripts::read_scripts() {
            if let Some(ref shortcut) = script.shortcut {
                register_script_hotkey_internal(
                    &manager_guard, 
                    &script.path.to_string_lossy(), 
                    shortcut, 
                    &script.name
                );
            }
        }

        drop(manager_guard);
        
        // Event loop
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.state != HotKeyState::Pressed {
                    continue;
                }

                // Look up action in routing table (fast read lock)
                let action = {
                    let routes_guard = routes().read().unwrap();
                    routes_guard.get_action(event.id)
                };

                match action {
                    Some(HotkeyAction::Main) => {
                        hotkey_channel().0.try_send(()).ok();
                    }
                    Some(HotkeyAction::Notes) => {
                        dispatch_notes_hotkey();
                    }
                    Some(HotkeyAction::Ai) => {
                        dispatch_ai_hotkey();
                    }
                    Some(HotkeyAction::Script(path)) => {
                        script_hotkey_channel().0.try_send(path).ok();
                    }
                    None => {
                        logging::log("HOTKEY", &format!(
                            "Unknown hotkey event id={}", event.id
                        ));
                    }
                }
            }
        }
    });
}
```

## GCD Dispatch (macOS)

### Direct Main Thread Dispatch

```rust
#[cfg(target_os = "macos")]
mod gcd {
    use std::ffi::c_void;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[link(name = "System", kind = "framework")]
    extern "C" {
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        #[link_name = "_dispatch_main_q"]
        static DISPATCH_MAIN_QUEUE: c_void;
    }

    /// Dispatch a closure to the main thread via GCD
    pub fn dispatch_to_main<F: FnOnce() + Send + 'static>(f: F) {
        let boxed: Box<dyn FnOnce() + Send> = Box::new(f);
        let raw = Box::into_raw(Box::new(boxed));

        extern "C" fn trampoline(context: *mut c_void) {
            unsafe {
                let boxed: Box<Box<dyn FnOnce() + Send>> = 
                    Box::from_raw(context as *mut _);
                // Catch panics to prevent UB across FFI
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    boxed();
                }));
            }
        }

        unsafe {
            let main_queue = &DISPATCH_MAIN_QUEUE as *const c_void;
            dispatch_async_f(main_queue, raw as *mut c_void, trampoline);
        }
    }
}
```

### Dispatch Handlers

```rust
/// Handler storage
static NOTES_HANDLER: OnceLock<Mutex<Option<HotkeyHandler>>> = OnceLock::new();
static AI_HANDLER: OnceLock<Mutex<Option<HotkeyHandler>>> = OnceLock::new();

pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;

/// Set handler for Notes hotkey
pub fn set_notes_hotkey_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let storage = NOTES_HANDLER.get_or_init(|| Mutex::new(None));
    *storage.lock().unwrap() = Some(Arc::new(handler));
}

/// Dispatch Notes hotkey
fn dispatch_notes_hotkey() {
    let handler = NOTES_HANDLER
        .get_or_init(|| Mutex::new(None))
        .lock().unwrap()
        .clone();

    if let Some(handler) = handler {
        // Direct GCD dispatch (skip channel to avoid double-fire)
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        // Fallback to channel
        notes_hotkey_channel().0.try_send(()).ok();
        gcd::dispatch_to_main(|| {}); // Wake GPUI event loop
    }
}
```

## Channel Communication

### Bounded Channels

```rust
// Main hotkey channel
static HOTKEY_CHANNEL: OnceLock<(Sender<()>, Receiver<()>)> = OnceLock::new();

pub fn hotkey_channel() -> &'static (Sender<()>, Receiver<()>) {
    HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

// Script hotkey channel (sends script path)
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(Sender<String>, Receiver<String>)> = 
    OnceLock::new();

pub fn script_hotkey_channel() -> &'static (Sender<String>, Receiver<String>) {
    SCRIPT_HOTKEY_CHANNEL.get_or_init(|| async_channel::bounded(10))
}
```

## Dynamic Script Hotkeys

### ScriptHotkeyManager

```rust
pub struct ScriptHotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey_map: HashMap<u32, String>,      // ID -> path
    path_to_id: HashMap<String, u32>,       // path -> ID
    path_to_hotkey: HashMap<String, HotKey>, // path -> HotKey object
}

impl ScriptHotkeyManager {
    pub fn register(&mut self, path: &str, shortcut: &str) -> anyhow::Result<u32> {
        let (mods, code) = parse_shortcut(shortcut)
            .ok_or_else(|| anyhow!("Failed to parse: {}", shortcut))?;

        let hotkey = HotKey::new(Some(mods), code);
        let id = hotkey.id();

        if let Err(e) = self.manager.register(hotkey) {
            return Err(match e {
                HotkeyError::AlreadyRegistered(hk) => anyhow!(
                    "Hotkey '{}' already registered (ID: {})", shortcut, hk.id()
                ),
                HotkeyError::FailedToRegister(msg) => anyhow!(
                    "System rejected '{}': {}", shortcut, msg
                ),
                other => anyhow!("Failed to register '{}': {}", shortcut, other),
            });
        }

        self.hotkey_map.insert(id, path.to_string());
        self.path_to_id.insert(path.to_string(), id);
        self.path_to_hotkey.insert(path.to_string(), hotkey);

        Ok(id)
    }

    pub fn unregister(&mut self, path: &str) -> anyhow::Result<()> {
        if let Some(id) = self.path_to_id.remove(path) {
            self.hotkey_map.remove(&id);
            if let Some(hotkey) = self.path_to_hotkey.remove(path) {
                let _ = self.manager.unregister(hotkey);
            }
        }
        Ok(())
    }

    pub fn update(
        &mut self,
        path: &str,
        old: Option<&str>,
        new: Option<&str>,
    ) -> anyhow::Result<()> {
        match (old, new) {
            (None, None) => Ok(()),
            (None, Some(new)) => { self.register(path, new)?; Ok(()) }
            (Some(_), None) => self.unregister(path),
            (Some(_), Some(new)) => {
                self.unregister(path)?;
                self.register(path, new)?;
                Ok(())
            }
        }
    }
}
```

## Hot-Reload Support

```rust
/// Update hotkeys from config - call when config changes
pub fn update_hotkeys(cfg: &Config) {
    let manager_guard = match MAIN_MANAGER.get() {
        Some(m) => match m.lock() {
            Ok(g) => g,
            Err(_) => return,
        },
        None => return,
    };

    // Update each builtin hotkey transactionally
    if let Some((mods, code)) = parse_hotkey_config(&cfg.hotkey) {
        let display = hotkey_config_to_display(&cfg.hotkey);
        rebind_hotkey_transactional(&manager_guard, HotkeyAction::Main, mods, code, &display);
    }

    if let Some((mods, code)) = parse_hotkey_config(&cfg.get_notes_hotkey()) {
        let display = hotkey_config_to_display(&cfg.get_notes_hotkey());
        rebind_hotkey_transactional(&manager_guard, HotkeyAction::Notes, mods, code, &display);
    }

    if let Some((mods, code)) = parse_hotkey_config(&cfg.get_ai_hotkey()) {
        let display = hotkey_config_to_display(&cfg.get_ai_hotkey());
        rebind_hotkey_transactional(&manager_guard, HotkeyAction::Ai, mods, code, &display);
    }
}
```

## Error Handling

```rust
fn format_hotkey_error(e: &HotkeyError, shortcut: &str) -> String {
    match e {
        HotkeyError::AlreadyRegistered(hk) => format!(
            "Hotkey '{}' already registered by another app (ID: {})",
            shortcut, hk.id()
        ),
        HotkeyError::FailedToRegister(msg) => format!(
            "System rejected '{}': {} (may be reserved by macOS)",
            shortcut, msg
        ),
        HotkeyError::OsError(err) => format!(
            "OS error for '{}': {}",
            shortcut, err
        ),
        other => format!("Failed to register '{}': {}", shortcut, other),
    }
}
```

## Best Practices

1. **Use transactional rebinding** - register new before unregistering old
2. **Use RwLock** for routing table (fast reads, infrequent writes)
3. **Bounded channels** prevent memory growth
4. **Non-blocking try_send** in hotkey thread
5. **GCD dispatch** for immediate main-thread execution on macOS
6. **Log all registration/unregistration** for debugging
7. **Handle conflicts gracefully** with clear error messages

## Summary

- Unified routing table for all hotkey types
- Transactional hot-reload prevents losing working hotkeys
- GCD dispatch bypasses async runtime for immediate response
- Bounded channels for cross-thread communication
- Clear error messages for conflicts and failures
