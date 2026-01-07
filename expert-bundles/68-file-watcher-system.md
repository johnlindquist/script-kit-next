# File Watcher System - Expert Bundle

## Overview

Script Kit uses file watchers for hot-reload of scripts, themes, and configuration, with debouncing to handle rapid file changes.

## Watcher Setup

### Core Watcher

```rust
use notify::{
    recommended_watcher, Event, EventKind, RecursiveMode, Watcher,
};
use std::sync::mpsc;
use std::time::Duration;

pub fn setup_file_watchers() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    
    let mut watcher = recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;
    
    // Watch key directories
    let home = dirs::home_dir().unwrap();
    
    // Scripts
    watcher.watch(
        &home.join(".scriptkit/scripts"),
        RecursiveMode::Recursive,
    )?;
    
    // Snippets
    watcher.watch(
        &home.join(".scriptkit/snippets"),
        RecursiveMode::Recursive,
    )?;
    
    // Theme (single file)
    watcher.watch(
        &home.join(".scriptkit/theme.json"),
        RecursiveMode::NonRecursive,
    )?;
    
    // Config (single file)
    watcher.watch(
        &home.join(".scriptkit/config.ts"),
        RecursiveMode::NonRecursive,
    )?;
    
    // Process events in background thread
    std::thread::spawn(move || {
        process_file_events(rx);
    });
    
    Ok(())
}
```

## Event Processing

### Debounced Processing

```rust
use std::collections::HashMap;
use std::time::Instant;

struct DebouncedWatcher {
    pending: HashMap<PathBuf, Instant>,
    debounce_duration: Duration,
}

impl DebouncedWatcher {
    fn new(debounce_ms: u64) -> Self {
        Self {
            pending: HashMap::new(),
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }

    fn should_process(&mut self, path: &Path) -> bool {
        let now = Instant::now();
        
        if let Some(last_time) = self.pending.get(path) {
            if now.duration_since(*last_time) < self.debounce_duration {
                // Update timestamp, skip processing
                self.pending.insert(path.to_path_buf(), now);
                return false;
            }
        }
        
        self.pending.insert(path.to_path_buf(), now);
        true
    }

    fn cleanup_old(&mut self) {
        let now = Instant::now();
        self.pending.retain(|_, time| {
            now.duration_since(*time) < self.debounce_duration * 2
        });
    }
}

fn process_file_events(rx: mpsc::Receiver<Event>) {
    let mut debouncer = DebouncedWatcher::new(100); // 100ms debounce
    
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                for path in &event.paths {
                    if debouncer.should_process(path) {
                        handle_file_change(path, &event.kind);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                debouncer.cleanup_old();
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}
```

## Change Handlers

### Route by Path

```rust
fn handle_file_change(path: &Path, kind: &EventKind) {
    let path_str = path.to_string_lossy();
    
    match kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            if path_str.contains("/scripts/") {
                handle_script_change(path);
            } else if path_str.contains("/snippets/") {
                handle_snippet_change(path);
            } else if path_str.ends_with("theme.json") {
                handle_theme_change(path);
            } else if path_str.ends_with("config.ts") {
                handle_config_change(path);
            }
        }
        EventKind::Remove(_) => {
            if path_str.contains("/scripts/") {
                handle_script_removed(path);
            } else if path_str.contains("/snippets/") {
                handle_snippet_removed(path);
            }
        }
        _ => {}
    }
}
```

### Script Change Handler

```rust
fn handle_script_change(path: &Path) {
    logging::log("WATCHER", &format!("Script changed: {:?}", path));
    
    // Invalidate cache
    SCRIPTLET_CACHE.invalidate(&path.to_string_lossy());
    
    // Re-parse script for hotkey changes
    if let Some(script) = parse_script(path) {
        // Check if shortcut changed
        let old_shortcut = get_registered_shortcut(&path.to_string_lossy());
        let new_shortcut = script.shortcut.as_deref();
        
        if old_shortcut != new_shortcut {
            // Update hotkey registration
            if let Err(e) = update_script_hotkey(
                &path.to_string_lossy(),
                old_shortcut,
                new_shortcut,
            ) {
                logging::log_warn("WATCHER", &format!(
                    "Failed to update hotkey: {}", e
                ));
            }
        }
    }
    
    // Notify UI to refresh
    send_refresh_event(RefreshEvent::Scripts);
}

fn handle_script_removed(path: &Path) {
    logging::log("WATCHER", &format!("Script removed: {:?}", path));
    
    // Remove from cache
    SCRIPTLET_CACHE.invalidate(&path.to_string_lossy());
    
    // Unregister hotkey
    let _ = unregister_script_hotkey(&path.to_string_lossy());
    
    // Notify UI
    send_refresh_event(RefreshEvent::Scripts);
}
```

### Theme Change Handler

```rust
fn handle_theme_change(path: &Path) {
    logging::log("WATCHER", "Theme changed, reloading");
    
    match load_theme_from_path(path) {
        Ok(theme) => {
            // Update shared theme
            *SHARED_THEME.write().unwrap() = theme;
            
            // Notify all windows
            send_refresh_event(RefreshEvent::Theme);
        }
        Err(e) => {
            logging::log_warn("WATCHER", &format!(
                "Failed to reload theme: {}", e
            ));
        }
    }
}
```

### Config Change Handler

```rust
fn handle_config_change(path: &Path) {
    logging::log("WATCHER", "Config changed, reloading");
    
    // Small delay to ensure file is fully written
    std::thread::sleep(Duration::from_millis(50));
    
    let new_config = load_config();
    
    // Update hotkeys
    crate::hotkeys::update_hotkeys(&new_config);
    
    // Update shared config
    *SHARED_CONFIG.write().unwrap() = Arc::new(new_config);
    
    // Notify UI
    send_refresh_event(RefreshEvent::Config);
}
```

## Refresh Events

### Event Channel

```rust
pub enum RefreshEvent {
    Scripts,
    Snippets,
    Theme,
    Config,
}

static REFRESH_CHANNEL: OnceLock<(
    async_channel::Sender<RefreshEvent>,
    async_channel::Receiver<RefreshEvent>,
)> = OnceLock::new();

fn refresh_channel() -> &'static (
    async_channel::Sender<RefreshEvent>,
    async_channel::Receiver<RefreshEvent>,
) {
    REFRESH_CHANNEL.get_or_init(|| async_channel::bounded(10))
}

fn send_refresh_event(event: RefreshEvent) {
    let _ = refresh_channel().0.try_send(event);
}
```

### UI Integration

```rust
impl App {
    fn setup_refresh_listener(&self, cx: &mut Context<Self>) {
        let receiver = refresh_channel().1.clone();
        
        cx.spawn(|this, mut cx| async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        let _ = this.update(&mut cx, |app, cx| {
                            app.handle_refresh(event, cx);
                        });
                    }
                    Err(_) => break,
                }
            }
        }).detach();
    }

    fn handle_refresh(&mut self, event: RefreshEvent, cx: &mut Context<Self>) {
        match event {
            RefreshEvent::Scripts => {
                self.reload_scripts();
                cx.notify();
            }
            RefreshEvent::Snippets => {
                self.reload_snippets();
                cx.notify();
            }
            RefreshEvent::Theme => {
                self.theme = shared_theme().read().unwrap().clone();
                cx.notify();
            }
            RefreshEvent::Config => {
                self.config = shared_config().read().unwrap().clone();
                cx.notify();
            }
        }
    }
}
```

## Watched Paths

| Path | Mode | Events |
|------|------|--------|
| `~/.scriptkit/scripts/` | Recursive | Create, Modify, Remove |
| `~/.scriptkit/snippets/` | Recursive | Create, Modify, Remove |
| `~/.scriptkit/theme.json` | Single | Modify |
| `~/.scriptkit/config.ts` | Single | Modify |

## Best Practices

### 1. Always Debounce

```rust
// Good - debounce rapid changes
if debouncer.should_process(path) {
    handle_change(path);
}

// Bad - process every event (causes spam)
handle_change(path);
```

### 2. Handle File Write Race

```rust
// Good - delay before reading
std::thread::sleep(Duration::from_millis(50));
let content = fs::read_to_string(path)?;

// Bad - read immediately (may get partial content)
let content = fs::read_to_string(path)?;
```

### 3. Graceful Error Handling

```rust
// Good - log and continue
match load_theme_from_path(path) {
    Ok(theme) => update_theme(theme),
    Err(e) => {
        logging::log_warn("WATCHER", &format!("Theme error: {}", e));
        // Keep using existing theme
    }
}

// Bad - panic on error
let theme = load_theme_from_path(path).unwrap();
```

### 4. Cleanup Resources

```rust
impl Drop for FileWatcherManager {
    fn drop(&mut self) {
        // Watcher is dropped automatically
        // Receiver channel will close
        logging::log("WATCHER", "File watcher stopped");
    }
}
```

## Summary

1. **Use `notify` crate** for cross-platform watching
2. **Debounce events** (100ms typical)
3. **Delay before reading** (50ms for write completion)
4. **Route by path** to appropriate handlers
5. **Invalidate caches** on file changes
6. **Update hotkeys** when scripts change
7. **Notify UI** via channel events
