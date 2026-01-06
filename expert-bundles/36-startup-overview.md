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

## Bundle: Initial Startup Process Overview

This bundle documents the complete application startup sequence from `fn main()` through the GPUI event loop initialization.

---

## Startup Sequence Summary

The startup process has **5 distinct phases**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Pre-GPUI Initialization                                            │
│   logging::init() → migrate_from_kenv() → ensure_kit_setup()               │
│   → PID file → orphan cleanup → signal handlers → config loading            │
│   → clipboard init → text expansion → MCP server → hotkey listener          │
├─────────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Watcher Creation                                                   │
│   AppearanceWatcher → ConfigWatcher → ScriptWatcher → Scheduler             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Phase 3: GPUI Application Startup                                           │
│   Application::new().run() → configure_as_accessory_app()                   │
│   → frontmost_app_tracker → fonts → gpui_component::init()                  │
│   → TrayManager::new()                                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Main Window Creation                                               │
│   calculate_bounds → load_theme → open_window() → ScriptListApp::new()     │
│   → register_main_window() → swizzle_gpui_blurred_view()                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Async Task Spawning                                                │
│   fallback_entry_point → main_hotkey_listener → notes_hotkey_listener      │
│   → ai_hotkey_listener → appearance_watcher → config_watcher               │
│   → script_watcher → scheduler_handler → stdin_listener → tray_handler     │
│   → shutdown_monitor                                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Files Involved

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, orchestrates entire startup |
| `src/logging.rs` | Dual-output logging initialization (JSONL + stderr) |
| `src/setup.rs` | Environment setup, directory creation, SDK extraction |
| `src/config/mod.rs` | Configuration loading from `~/.scriptkit/kit/config.ts` |
| `src/theme/mod.rs` | Theme loading from `~/.scriptkit/kit/theme.json` |
| `src/hotkeys.rs` | Global hotkey registration |
| `src/watcher.rs` | File watchers for config, scripts, appearance |
| `src/tray.rs` | System tray icon and menu |
| `src/stdin_commands.rs` | Stdin JSON protocol listener |
| `src/app_impl.rs` | ScriptListApp initialization |
| `src/platform.rs` | macOS-specific platform configuration |
| `src/scripts/mod.rs` | Script and scriptlet loading |
| `src/clipboard_history/mod.rs` | Clipboard monitoring initialization |
| `src/scheduler.rs` | Cron-based script scheduler |

---

## Critical Design Decisions

### 1. Pre-GPUI vs Post-GPUI Initialization

Some initialization MUST happen before GPUI starts:
- Logging (to capture early errors)
- Signal handlers (async-signal-safe requirement)
- Config loading (needed for hotkey registration)
- Hotkey listener thread (runs independently of GPUI)

### 2. Window Starts Hidden

The main window is created but NOT shown at startup:
```rust
WindowOptions {
    show: false,     // Start hidden - only show on hotkey press
    focus: false,    // Don't focus on creation
    // ...
}
```

This is intentional for "launcher" UX - the app lives in the background until summoned.

### 3. Dual Entry Points

Users can access the app via:
- **Global hotkey** (default: Cmd+;)
- **System tray icon**

If BOTH fail, a fallback shows the window at startup (prevents "invisible" app).

### 4. Event-Driven Architecture

The startup spawns multiple async tasks that communicate via channels:
- `async_channel` for hotkey events, stdin commands, appearance changes
- `std::sync::mpsc` for scheduler events (requires polling)

This avoids polling in the render loop (performance-critical).

---

## Timing Expectations

| Operation | Expected Duration |
|-----------|-------------------|
| Logging init | <1ms |
| Directory setup | <5ms (SSD) |
| Config loading (bun eval) | 100-300ms |
| Script loading (331 scripts) | ~5ms |
| Scriptlet loading | ~5ms |
| Theme loading | <1ms |
| Window creation | Near-instant |
| App scanning (background) | 300-500ms |
| **Total to first hotkey response** | ~300-400ms |

---

## Error Handling Philosophy

Each component can fail independently without crashing the app:

| Component | Failure Behavior |
|-----------|------------------|
| Logging | Falls back to /dev/null |
| Setup | Collects warnings, continues |
| Config | Uses defaults |
| Theme | Uses system appearance defaults |
| Hotkeys | Falls back to tray-only |
| Tray | Falls back to hotkey-only |
| Watchers | Continues without live reload |
| MCP Server | Continues without AI integration |
| Clipboard | Continues without history |

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
