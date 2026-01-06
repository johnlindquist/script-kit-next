# Script Kit GPUI - Expert Review Request

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Architecture:**
- **GPUI** for UI rendering (custom immediate-mode reactive UI framework from Zed)
- **Bun** as the TypeScript runtime for user scripts
- **Stdin/stdout JSON protocol** for bidirectional script ↔ app communication
- **SQLite** for persistence (clipboard history, notes, chat)
- **macOS-first** with floating panel window behavior

**Key Constraints:**
- Must maintain backwards compatibility with existing Script Kit scripts
- Performance-critical: launcher must appear instantly, list scrolling at 60fps
- Multi-window: main launcher + Notes window + AI chat window (all independent)
- Theme hot-reload across all windows

---

## Bundle: Phase 1 - Pre-GPUI Initialization

This bundle covers all initialization that happens BEFORE the GPUI event loop starts.

---

## Phase 1 Sequence (main.rs lines 1159-1414)

```rust
fn main() {
    // 1. Logging initialization
    logging::init();

    // 2. Legacy migration (~/.kenv → ~/.scriptkit)
    if setup::migrate_from_kenv() {
        logging::log("APP", "Migrated from ~/.kenv to ~/.scriptkit");
    }

    // 3. Environment setup
    let setup_result = setup::ensure_kit_setup();
    // Creates directories, extracts SDK, creates default files

    // 4. PID file for orphan detection
    PROCESS_MANAGER.write_main_pid();

    // 5. Orphan process cleanup
    PROCESS_MANAGER.cleanup_orphans();

    // 6. Signal handlers (Unix only)
    #[cfg(unix)]
    {
        extern "C" fn handle_signal(_sig: libc::c_int) {
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        }
        unsafe {
            libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGHUP, handle_signal as libc::sighandler_t);
        }
    }

    // 7. Config loading (bun eval - ~100-300ms)
    let loaded_config = config::load_config();

    // 8. Clipboard history initialization
    clipboard_history::init_clipboard_history();

    // 9. Text expansion system (macOS, background thread)
    #[cfg(target_os = "macos")]
    {
        std::thread::spawn(|| { ExpandManager::new().enable(); });
    }

    // 10. MCP server for AI agents
    mcp_server::McpServer::with_defaults()?.start();

    // 11. Hotkey listener (separate thread)
    hotkeys::start_hotkey_listener(loaded_config);

    // ... continues to Phase 2 (watchers)
}
```

---

## Component Details

### 1. Logging System (src/logging.rs)

```rust
pub fn init() -> LoggingGuard {
    // Initialize in-memory log buffer for UI display
    let _ = LOG_BUFFER.set(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));

    // Check for AI compact log mode
    let ai_log_mode = std::env::var("SCRIPT_KIT_AI_LOG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // Create log directory
    let log_dir = get_log_dir(); // ~/.scriptkit/logs/
    fs::create_dir_all(&log_dir)?;

    // Open log file with append mode
    let log_path = log_dir.join("script-kit-gpui.jsonl");

    // Create non-blocking writer (prevents UI freeze)
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file);

    // Configure tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)         // JSONL to file
        .with(pretty_layer)       // Pretty to stderr (or compact AI layer)
        .init();

    LoggingGuard { _file_guard: file_guard }
}
```

**Key Points:**
- Dual output: JSONL file + stderr
- Non-blocking file writes (won't freeze UI)
- AI compact mode for token-efficient logs
- Guard must be kept alive for entire program

---

### 2. Environment Setup (src/setup.rs)

```rust
pub fn ensure_kit_setup() -> SetupResult {
    let kit_dir = get_kit_path(); // ~/.scriptkit or $SK_PATH

    // Create directory structure
    let required_dirs = [
        kit_dir.join("kit").join("main").join("scripts"),
        kit_dir.join("kit").join("main").join("extensions"),
        kit_dir.join("kit").join("main").join("agents"),
        kit_dir.join("sdk"),
        kit_dir.join("db"),
        kit_dir.join("logs"),
        kit_dir.join("cache").join("app-icons"),
    ];

    // SDK extraction (compile-time embedded)
    let sdk_path = kit_dir.join("sdk").join("kit-sdk.ts");
    write_string_if_changed(&sdk_path, EMBEDDED_SDK, ...);

    // User config (only create if missing)
    let config_path = kit_dir.join("kit").join("config.ts");
    write_string_if_missing(&config_path, EMBEDDED_CONFIG_TEMPLATE, ...);

    // App-managed files (refresh if changed)
    ensure_tsconfig_paths(...);  // TypeScript path mappings
    write_string_if_changed(&gitignore_path, ...);

    // Check for bun availability
    let bun_available = bun_is_discoverable();

    // Sample files on fresh install
    if is_fresh_install {
        create_sample_files(&kit_dir, ...);
    }

    SetupResult { is_fresh_install, kit_path: kit_dir, bun_available, warnings }
}
```

**File Categories:**
- **User-owned**: Never overwritten (config.ts, theme.json)
- **App-managed**: Refreshed if changed (SDK, tsconfig.json, .gitignore)
- **Fresh-install only**: Sample scripts, README

---

### 3. Process Management

```rust
// Write PID for orphan detection
PROCESS_MANAGER.write_main_pid();  // ~/.scriptkit/main.pid

// Kill orphaned processes from previous crash
let orphans_killed = PROCESS_MANAGER.cleanup_orphans();
```

This prevents zombie script processes from accumulating after crashes.

---

### 4. Signal Handlers

```rust
// ASYNC-SIGNAL-SAFE: Only set atomic flag
// All cleanup happens in GPUI shutdown monitor task
extern "C" fn handle_signal(_sig: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}
```

**Critical**: Signal handlers can only safely call async-signal-safe functions.
We ONLY set an atomic flag; actual cleanup happens in a GPUI task.

---

### 5. Config Loading

```rust
let loaded_config = config::load_config();
// Uses bun to eval ~/.scriptkit/kit/config.ts
// Returns Config struct with hotkey, fonts, builtins settings
// Falls back to defaults if parsing fails
```

This is the **slowest startup operation** (~100-300ms) because it spawns bun.

---

### 6. Clipboard History

```rust
clipboard_history::init_clipboard_history();
// Spawns background thread for clipboard monitoring
// Stores history in SQLite: ~/.scriptkit/db/clipboard-history.db
```

---

### 7. Text Expansion (macOS)

```rust
std::thread::spawn(move || {
    // Check accessibility permissions
    if !ExpandManager::has_accessibility_permission() {
        return;  // Disabled without permissions
    }

    let mut manager = ExpandManager::new();
    manager.load_scriptlets()?;  // Load expand triggers
    manager.enable()?;           // Start keyboard monitoring

    // Manager runs until process exits
    std::mem::forget(manager);
});
```

---

### 8. MCP Server

```rust
let _mcp_handle = mcp_server::McpServer::with_defaults()?.start();
// HTTP server on localhost:43210
// Bearer token authentication
// Discovery file: ~/.scriptkit/server.json
```

Enables AI agents to interact with Script Kit.

---

### 9. Hotkey Listener

```rust
hotkeys::start_hotkey_listener(loaded_config);
// Runs in separate thread (not on GPUI event loop)
// Dispatches via async_channel when hotkey pressed
```

**Why separate thread?** Global hotkeys must be registered with the OS before GPUI starts, and the listener must run independently.

---

## Error Handling Pattern

Each component handles failures gracefully:

```rust
// Example: clipboard history
if let Err(e) = clipboard_history::init_clipboard_history() {
    logging::log("APP", &format!("Failed to initialize clipboard history: {}", e));
    // Continue without clipboard history
} else {
    logging::log("APP", "Clipboard history monitoring initialized");
}
```

The app continues with degraded functionality rather than crashing.

---

## Performance Considerations

| Operation | Blocking? | Duration |
|-----------|-----------|----------|
| Logging init | No | <1ms |
| Directory setup | Yes (disk I/O) | <5ms |
| Signal handlers | No | <1ms |
| Config loading | Yes (bun spawn) | 100-300ms |
| Clipboard init | No (spawns thread) | <1ms |
| Text expansion | No (spawns thread) | <1ms |
| MCP server | No (spawns thread) | <1ms |
| Hotkey listener | No (spawns thread) | <1ms |

**Bottleneck**: Config loading is the main blocking operation.

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
