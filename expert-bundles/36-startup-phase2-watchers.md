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

## Bundle: Phase 2 - Watcher Initialization

This bundle covers the file watchers and scheduler that are created BEFORE the GPUI event loop but whose poll loops are spawned AFTER.

---

## Phase 2 Sequence (main.rs lines 1356-1413)

```rust
// Start watchers and track which ones succeeded
// We only spawn poll loops for watchers that successfully started

// Appearance watcher - system light/dark mode changes
let (mut appearance_watcher, appearance_rx) = watcher::AppearanceWatcher::new();
let appearance_watcher_ok = match appearance_watcher.start() {
    Ok(()) => { logging::log("APP", "Appearance watcher started"); true }
    Err(e) => { logging::log("APP", &format!("Failed: {}", e)); false }
};

// Config watcher - ~/.scriptkit/kit/config.ts changes
let (mut config_watcher, config_rx) = watcher::ConfigWatcher::new();
let config_watcher_ok = match config_watcher.start() {
    Ok(()) => { logging::log("APP", "Config watcher started"); true }
    Err(e) => { logging::log("APP", &format!("Failed: {}", e)); false }
};

// Script watcher - ~/.scriptkit/kit/main/scripts/ and extensions/
let (mut script_watcher, script_rx) = watcher::ScriptWatcher::new();
let script_watcher_ok = match script_watcher.start() {
    Ok(()) => { logging::log("APP", "Script watcher started"); true }
    Err(e) => { logging::log("APP", &format!("Failed: {}", e)); false }
};

// Initialize script scheduler
let (mut scheduler, scheduler_rx) = scheduler::Scheduler::new();
let scheduled_count = scripts::register_scheduled_scripts(&scheduler);
if scheduled_count > 0 {
    scheduler.start()?;
}

// Wrap scheduler in Arc<Mutex<>> for thread-safe access
let scheduler = Arc::new(Mutex::new(scheduler));
```

---

## Watcher Architecture

Each watcher follows the same pattern:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Watcher Pattern                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────┐    create     ┌─────────────────┐                    │
│   │  Watcher::new() │ ─────────────▶│  (watcher, rx)  │                    │
│   └─────────────────┘               └─────────────────┘                    │
│                                            │                                │
│                                            │ start()                        │
│                                            ▼                                │
│                            ┌───────────────────────────┐                   │
│                            │  OS file system watcher   │                   │
│                            │  (notify/FSEvents)        │                   │
│                            └───────────────────────────┘                   │
│                                            │                                │
│                                            │ file change event              │
│                                            ▼                                │
│                            ┌───────────────────────────┐                   │
│                            │   async_channel::send()   │                   │
│                            └───────────────────────────┘                   │
│                                            │                                │
│                                            │ (rx lives in GPUI task)       │
│                                            ▼                                │
│                            ┌───────────────────────────┐                   │
│                            │  GPUI spawn task polls   │                   │
│                            │  rx.recv().await         │                   │
│                            └───────────────────────────┘                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Watcher Types

### 1. AppearanceWatcher

Watches for macOS system appearance changes (light/dark mode).

```rust
pub struct AppearanceWatcher {
    // Uses macOS NSDistributedNotificationCenter
    // Listens for "AppleInterfaceThemeChangedNotification"
}

// Events:
pub enum AppearanceEvent {
    Changed,  // System appearance changed
}
```

**Usage**: When the system switches between light/dark mode, the app reloads the theme and syncs all windows.

---

### 2. ConfigWatcher

Watches `~/.scriptkit/kit/config.ts` for changes.

```rust
pub struct ConfigWatcher {
    // Uses notify crate (FSEvents on macOS)
    // Watches: ~/.scriptkit/kit/config.ts
}

// Events:
pub enum ConfigEvent {
    Changed,  // Config file modified
}
```

**Usage**: User edits config.ts → app reloads configuration (hotkey, font sizes, etc.)

---

### 3. ScriptWatcher

Watches script directories for file changes.

```rust
pub struct ScriptWatcher {
    // Uses notify crate (FSEvents on macOS)
    // Watches: 
    //   ~/.scriptkit/kit/main/scripts/
    //   ~/.scriptkit/kit/main/extensions/
}

// Events:
pub enum ScriptReloadEvent {
    FileChanged(PathBuf),   // Script/scriptlet modified
    FileCreated(PathBuf),   // New script/scriptlet added
    FileDeleted(PathBuf),   // Script/scriptlet removed
    FullReload,             // Trigger complete reload
}
```

**Usage**: 
- Script `.ts` files → full script list reload
- Scriptlet `.md` files → incremental update

---

### 4. Scheduler

Cron-based script scheduling.

```rust
pub struct Scheduler {
    // Tracks scripts with `// Cron:` or `// Schedule:` metadata
    // Background thread checks every 30 seconds
}

// Events:
pub enum SchedulerEvent {
    RunScript(PathBuf),     // Time to execute this script
    Error(String),          // Scheduler error
}
```

**Example script metadata:**
```typescript
// Cron: 0 9 * * *     (Run at 9 AM daily)
// Schedule: every 5 minutes
```

---

## Channel Usage

Watchers use `async_channel` (async-compatible):
```rust
let (tx, rx) = async_channel::bounded::<Event>(100);

// Watcher thread sends:
tx.send(Event::Changed).await;

// GPUI task receives (event-driven, no polling):
while let Ok(event) = rx.recv().await {
    handle_event(event);
}
```

Scheduler uses `std::sync::mpsc` (sync-only):
```rust
let (tx, rx) = std::sync::mpsc::channel::<SchedulerEvent>();

// Scheduler thread sends:
tx.send(SchedulerEvent::RunScript(path))?;

// Handler thread polls (not in GPUI):
loop {
    match rx.recv_timeout(Duration::from_secs(1)) {
        Ok(event) => handle_event(event),
        Err(Timeout) => continue,
        Err(Disconnected) => break,
    }
}
```

---

## Why Watchers Start Before GPUI

1. **OS watcher registration** happens immediately
2. **Channel receivers** are passed into GPUI tasks later
3. This ensures no events are missed during GPUI initialization

---

## Error Handling

Watchers are optional - the app continues if they fail:

```rust
let config_watcher_ok = match config_watcher.start() {
    Ok(()) => true,
    Err(e) => {
        logging::log("APP", &format!("Failed to start config watcher: {}", e));
        false  // Continue without live config reload
    }
};

// Later, only spawn poll loop if watcher started successfully:
if config_watcher_ok {
    cx.spawn(async move |cx| {
        // Poll config_rx
    }).detach();
}
```

---

## Watched Paths Summary

| Watcher | Watched Paths |
|---------|---------------|
| AppearanceWatcher | macOS NSDistributedNotificationCenter |
| ConfigWatcher | `~/.scriptkit/kit/config.ts` |
| ScriptWatcher | `~/.scriptkit/kit/main/scripts/*.ts` |
| ScriptWatcher | `~/.scriptkit/kit/main/extensions/*.md` |
| Scheduler | Scripts with `// Cron:` metadata |

---

## File Change Hot Reload Behavior

| File Type | Reload Behavior |
|-----------|-----------------|
| `config.ts` | Reload config, update UI settings |
| `theme.json` | Reload theme, sync all windows |
| `*.ts` (scripts) | Full script list reload |
| `*.md` (scriptlets) | Incremental update |
| System appearance | Reload theme |

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
