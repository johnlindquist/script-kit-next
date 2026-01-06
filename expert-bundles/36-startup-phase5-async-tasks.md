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

## Bundle: Phase 5 - Async Task Spawning

This bundle covers all the async tasks spawned after window creation that handle events, watchers, and background operations.

---

## Phase 5 Overview

After the window is created, multiple async tasks are spawned to handle various events:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Async Tasks Spawned                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │ Fallback Entry      │  │ Main Hotkey         │  │ Notes Hotkey        │ │
│  │ Point Check         │  │ Listener            │  │ Listener            │ │
│  │ (500ms timeout)     │  │ (event-driven)      │  │ (event-driven)      │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │ AI Hotkey           │  │ Appearance          │  │ Config              │ │
│  │ Listener            │  │ Watcher             │  │ Watcher             │ │
│  │ (event-driven)      │  │ (event-driven)      │  │ (polling 200ms)     │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │ Script              │  │ Scheduler           │  │ Stdin               │ │
│  │ Watcher             │  │ Event Handler       │  │ Command Handler     │ │
│  │ (polling 200ms)     │  │ (std::thread)       │  │ (event-driven)      │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐                          │
│  │ Tray Menu           │  │ Shutdown            │                          │
│  │ Event Handler       │  │ Monitor             │                          │
│  │ (polling 100ms)     │  │ (polling 100ms)     │                          │
│  └─────────────────────┘  └─────────────────────┘                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Task Details

### 1. Fallback Entry Point Check

```rust
cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    // Wait for hotkey registration to complete
    Timer::after(Duration::from_millis(500)).await;

    let hotkey_ok = hotkeys::is_main_hotkey_registered();

    if !hotkey_ok && !tray_ok {
        // Both entry points failed - show window as fallback
        logging::log("APP", "WARNING: Both hotkey and tray failed!");
        let _ = cx.update(|cx| {
            show_main_window_helper(window, app_entity, cx);
        });
    }
}).detach();
```

**Purpose**: Prevents "invisible app" scenario where user has no way to access the launcher.

---

### 2. Main Hotkey Listener

```rust
cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    while let Ok(()) = hotkeys::hotkey_channel().1.recv().await {
        let is_visible = is_main_window_visible();
        
        if is_visible {
            // Toggle: hide
            cx.update(|cx| hide_main_window_helper(app_entity, cx));
        } else {
            // Toggle: show
            cx.update(|cx| show_main_window_helper(window, app_entity, cx));
        }
    }
}).detach();
```

**Key Points**:
- Event-driven via `async_channel` (no polling)
- Implements toggle behavior (show if hidden, hide if visible)
- Uses centralized show/hide helpers

---

### 3. Notes Hotkey Listener

```rust
cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    while let Ok(()) = hotkeys::notes_hotkey_channel().1.recv().await {
        let _ = cx.update(|cx| {
            notes::open_notes_window(cx)?;
        });
    }
}).detach();
```

**Default hotkey**: Cmd+Shift+N

---

### 4. AI Hotkey Listener

```rust
cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    while let Ok(()) = hotkeys::ai_hotkey_channel().1.recv().await {
        let _ = cx.update(|cx| {
            ai::open_ai_window(cx)?;
        });
    }
}).detach();
```

**Default hotkey**: Cmd+Shift+Space

---

### 5. Appearance Watcher Task

```rust
if appearance_watcher_ok {
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(_event) = appearance_rx.recv().await {
            let _ = cx.update(|cx| {
                // Sync gpui-component theme
                theme::sync_gpui_component_theme(cx);
                
                // Update app theme
                app_entity.update(cx, |view, ctx| {
                    view.update_theme(ctx);
                });
            });
        }
    }).detach();
}
```

**Trigger**: macOS system appearance change (light ↔ dark)

---

### 6. Config Watcher Task

```rust
if config_watcher_ok {
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        loop {
            Timer::after(Duration::from_millis(200)).await;

            if config_rx.try_recv().is_ok() {
                let _ = cx.update(|cx| {
                    app_entity.update(cx, |view, ctx| {
                        view.update_config(ctx);
                    });
                });
            }
        }
    }).detach();
}
```

**Note**: Uses polling (200ms) because `try_recv()` is used instead of `recv().await`.

---

### 7. Script Watcher Task

```rust
if script_watcher_ok {
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        loop {
            Timer::after(Duration::from_millis(200)).await;

            while let Ok(event) = script_rx.try_recv() {
                match event {
                    ScriptReloadEvent::FileChanged(path) | 
                    ScriptReloadEvent::FileCreated(path) => {
                        let is_scriptlet = path.extension()
                            .map(|e| e == "md")
                            .unwrap_or(false);

                        if is_scriptlet {
                            // Incremental scriptlet update
                            view.handle_scriptlet_file_change(&path, false, ctx);
                        } else {
                            // Re-scan scheduled scripts
                            scripts::register_scheduled_scripts(&scheduler);
                            // Full script reload
                            view.refresh_scripts(ctx);
                        }
                    }
                    ScriptReloadEvent::FileDeleted(path) => {
                        // Similar handling...
                    }
                    ScriptReloadEvent::FullReload => {
                        view.refresh_scripts(ctx);
                    }
                }
            }
        }
    }).detach();
}
```

---

### 8. Scheduler Event Handler

```rust
// NOTE: Uses std::thread, not GPUI spawn
// Because scheduler uses std::sync::mpsc (not async_channel)
std::thread::spawn(move || {
    loop {
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            break;
        }

        match scheduler_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(SchedulerEvent::RunScript(path)) => {
                // Spawn bun to run the script
                let child = Command::new(&bun_path)
                    .arg("run")
                    .arg("--preload")
                    .arg(&sdk_path)
                    .arg(&path)
                    .spawn()?;
                
                // Track process
                PROCESS_MANAGER.register_process(child.id(), &path);
                
                // Wait in separate thread (don't block scheduler)
                std::thread::spawn(move || {
                    let output = child.wait_with_output();
                    PROCESS_MANAGER.unregister_process(pid);
                });
            }
            Ok(SchedulerEvent::Error(msg)) => {
                logging::log("SCHEDULER", &format!("Error: {}", msg));
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
});
```

---

### 9. Stdin Command Handler

```rust
let stdin_rx = start_stdin_listener();

cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    while let Ok(cmd) = stdin_rx.recv().await {
        match cmd {
            ExternalCommand::Run { path, request_id } => {
                // Show window and run script
                show_window_and_run_script(&path, ...);
            }
            ExternalCommand::Show { .. } => {
                show_main_window_helper(window, app_entity, cx);
            }
            ExternalCommand::Hide { .. } => {
                hide_main_window_helper(app_entity, cx);
            }
            ExternalCommand::SetFilter { text, .. } => {
                view.set_filter(&text, ctx);
            }
            ExternalCommand::OpenNotes { .. } => {
                notes::open_notes_window(cx)?;
            }
            ExternalCommand::OpenAi { .. } => {
                ai::open_ai_window(cx)?;
            }
            // ... more commands
        }
    }
}).detach();
```

---

### 10. Shutdown Monitor Task

```rust
cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    loop {
        Timer::after(Duration::from_millis(100)).await;
        
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            // Cleanup
            PROCESS_MANAGER.kill_all_scripts();
            PROCESS_MANAGER.remove_pid_file();
            
            // Exit
            cx.update(|cx| cx.quit());
            break;
        }
    }
}).detach();
```

**Why polling?** Signal handlers can only set atomic flags. Actual cleanup must happen on the main thread.

---

## Event-Driven vs Polling

| Task | Pattern | Interval | Why |
|------|---------|----------|-----|
| Main hotkey | Event-driven | - | `async_channel` supports `.await` |
| Notes hotkey | Event-driven | - | `async_channel` supports `.await` |
| AI hotkey | Event-driven | - | `async_channel` supports `.await` |
| Appearance watcher | Event-driven | - | `async_channel` supports `.await` |
| Config watcher | Polling | 200ms | Uses `try_recv()` pattern |
| Script watcher | Polling | 200ms | Drains multiple events per tick |
| Scheduler | Polling | 1s | Uses `std::sync::mpsc` (no async) |
| Stdin | Event-driven | - | `async_channel` supports `.await` |
| Tray menu | Polling | 100ms | Menu clicks are polled |
| Shutdown | Polling | 100ms | Checks atomic flag |

---

## Task Communication Pattern

```
┌─────────────────┐     channel      ┌─────────────────┐
│  Background     │ ───────────────▶ │  GPUI Task      │
│  Thread/Source  │                  │  (cx.spawn)     │
└─────────────────┘                  └─────────────────┘
        │                                    │
        │                                    │ cx.update(|cx| {...})
        │                                    ▼
        │                            ┌─────────────────┐
        │                            │  App Entity     │
        │                            │  Update         │
        │                            └─────────────────┘
```

All GPUI entity updates MUST go through `cx.update()` to ensure thread safety.

---

## detach() Pattern

All tasks use `.detach()`:

```rust
cx.spawn(async move |cx| {
    // task logic
}).detach();
```

**Why?** Detached tasks run independently. If not detached, the task handle would need to be stored somewhere to prevent the task from being dropped.

---

## Error Handling in Tasks

Tasks that can fail use Result handling:

```rust
cx.spawn(async move |cx| {
    while let Ok(cmd) = stdin_rx.recv().await {
        let _ = cx.update(|cx| {
            // The `let _ =` ignores the Result
            // Errors are logged inside the update closure
        });
    }
}).detach();
```

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
