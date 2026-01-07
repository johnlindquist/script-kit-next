# State Management Architecture - Expert Bundle

## Overview

Script Kit uses a combination of GPUI's reactive model, global singletons, and message channels for state management across the application.

## GPUI Reactive State

### View State Pattern

```rust
pub struct App {
    // Core UI state - triggers re-render on change
    filter_text: String,
    selected_index: usize,
    prompt_state: PromptState,
    
    // Computed/derived state
    filtered_scripts: Vec<Arc<Script>>,
    
    // Resources (don't trigger re-render directly)
    theme: Theme,
    config: Arc<Config>,
    
    // Focus management
    focus_handle: FocusHandle,
}

impl App {
    fn set_filter(&mut self, text: String, cx: &mut Context<Self>) {
        self.filter_text = text.clone();
        self.recompute_filtered_scripts();
        cx.notify(); // CRITICAL: Must notify after state changes
    }
    
    fn recompute_filtered_scripts(&mut self) {
        self.filtered_scripts = self.all_scripts
            .iter()
            .filter(|s| s.matches_filter(&self.filter_text))
            .cloned()
            .collect();
    }
}
```

### When to Call cx.notify()

```rust
// ALWAYS call after:
// 1. Selection changes
fn move_selection_down(&mut self, cx: &mut Context<Self>) {
    if self.selected_index < self.filtered_scripts.len() - 1 {
        self.selected_index += 1;
        cx.notify(); // Selection changed
    }
}

// 2. Filter text changes
fn handle_input(&mut self, text: &str, cx: &mut Context<Self>) {
    self.filter_text = text.to_string();
    self.recompute_filtered_scripts();
    cx.notify(); // List content changed
}

// 3. Visibility changes
fn show(&mut self, cx: &mut Context<Self>) {
    self.visible = true;
    cx.notify(); // Visibility changed
}

// 4. Focus changes that affect styling
fn on_focus_change(&mut self, focused: bool, cx: &mut Context<Self>) {
    if self.is_focused != focused {
        self.is_focused = focused;
        cx.notify(); // Focus styling needs update
    }
}
```

## Global Singleton Pattern

### Process Manager (src/process_manager.rs)

```rust
use std::sync::{LazyLock, RwLock};

/// Global singleton process manager
pub static PROCESS_MANAGER: LazyLock<ProcessManager> = LazyLock::new(ProcessManager::new);

pub struct ProcessManager {
    active_processes: RwLock<HashMap<u32, ProcessInfo>>,
    main_pid_path: PathBuf,
    active_pids_path: PathBuf,
}

impl ProcessManager {
    pub fn register_process(&self, pid: u32, script_path: &str) {
        if let Ok(mut processes) = self.active_processes.write() {
            processes.insert(pid, ProcessInfo {
                pid,
                script_path: script_path.to_string(),
                started_at: Utc::now(),
            });
        }
        // Persist to disk for crash recovery
        let _ = self.persist_active_pids();
    }
    
    pub fn unregister_process(&self, pid: u32) {
        if let Ok(mut processes) = self.active_processes.write() {
            processes.remove(&pid);
        }
        let _ = self.persist_active_pids();
    }
}
```

### Hotkey Routes (src/hotkeys.rs)

```rust
/// Global routing table - protected by RwLock for fast reads
static HOTKEY_ROUTES: OnceLock<RwLock<HotkeyRoutes>> = OnceLock::new();

struct HotkeyRoutes {
    routes: HashMap<u32, RegisteredHotkey>,
    script_paths: HashMap<String, u32>,
    main_id: Option<u32>,
    notes_id: Option<u32>,
    ai_id: Option<u32>,
}

fn routes() -> &'static RwLock<HotkeyRoutes> {
    HOTKEY_ROUTES.get_or_init(|| RwLock::new(HotkeyRoutes::new()))
}

// Fast read path for event dispatch
fn handle_hotkey_event(id: u32) {
    let action = {
        let routes_guard = routes().read().unwrap();
        routes_guard.get_action(id)
    }; // Lock released here
    
    // Process action outside lock
    if let Some(action) = action {
        dispatch_action(action);
    }
}
```

## Message Channel Pattern

### Async Channels for Cross-Thread Communication

```rust
use async_channel::{Sender, Receiver, bounded};
use std::sync::OnceLock;

// Hotkey channel - bounded to prevent memory growth
static HOTKEY_CHANNEL: OnceLock<(Sender<()>, Receiver<()>)> = OnceLock::new();

pub fn hotkey_channel() -> &'static (Sender<()>, Receiver<()>) {
    HOTKEY_CHANNEL.get_or_init(|| bounded(10))
}

// Script hotkey channel - sends script path
static SCRIPT_HOTKEY_CHANNEL: OnceLock<(Sender<String>, Receiver<String>)> = OnceLock::new();

pub fn script_hotkey_channel() -> &'static (Sender<String>, Receiver<String>) {
    SCRIPT_HOTKEY_CHANNEL.get_or_init(|| bounded(10))
}
```

### Using Channels in GPUI

```rust
impl App {
    fn poll_hotkey_channel(&mut self, cx: &mut Context<Self>) {
        // Non-blocking poll in render cycle
        let receiver = &hotkey_channel().1;
        
        while let Ok(()) = receiver.try_recv() {
            self.toggle_visibility(cx);
        }
    }
    
    fn spawn_channel_listener(&self, cx: &mut Context<Self>) {
        let receiver = hotkey_channel().1.clone();
        
        cx.spawn(|this, mut cx| async move {
            loop {
                if receiver.recv().await.is_ok() {
                    let _ = this.update(&mut cx, |app, cx| {
                        app.toggle_visibility(cx);
                    });
                }
            }
        }).detach();
    }
}
```

## Coalescing Pattern

### Filter Coalescer (src/filter_coalescer.rs)

```rust
/// Coalesces rapid updates into single operations
#[derive(Debug, Default)]
pub struct FilterCoalescer {
    pending: bool,
    latest: Option<String>,
}

impl FilterCoalescer {
    pub fn queue(&mut self, value: impl Into<String>) -> bool {
        self.latest = Some(value.into());
        if self.pending {
            false // Already have pending work
        } else {
            self.pending = true;
            true // First in batch, start work
        }
    }

    pub fn take_latest(&mut self) -> Option<String> {
        if !self.pending {
            return None;
        }
        self.pending = false;
        self.latest.take()
    }
}
```

### Usage in Keyboard Handling

```rust
impl App {
    fn handle_key_event(&mut self, key: &str, cx: &mut Context<Self>) {
        // Coalesce rapid key events
        if key == "up" || key == "arrowup" {
            self.queue_scroll(ScrollDirection::Up, cx);
        }
    }
    
    fn queue_scroll(&mut self, dir: ScrollDirection, cx: &mut Context<Self>) {
        let now = Instant::now();
        
        if now.duration_since(self.last_scroll_time) < Duration::from_millis(20)
           && self.pending_scroll_dir == Some(dir) {
            // Coalesce with pending scroll
            self.pending_scroll_delta += 1;
            return;
        }
        
        // Flush pending and start new batch
        self.flush_pending_scroll(cx);
        self.pending_scroll_dir = Some(dir);
        self.pending_scroll_delta = 1;
        self.last_scroll_time = now;
        
        // Schedule flush
        cx.spawn(|this, mut cx| async move {
            Timer::after(Duration::from_millis(20)).await;
            let _ = this.update(&mut cx, |app, cx| {
                app.flush_pending_scroll(cx);
            });
        }).detach();
    }
}
```

## Database-Backed State

### SQLite for Persistence (src/menu_cache.rs)

```rust
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex, OnceLock};

static MENU_CACHE_DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

pub fn init_menu_cache_db() -> Result<()> {
    if MENU_CACHE_DB.get().is_some() {
        return Ok(()); // Idempotent
    }
    
    let conn = Connection::open(get_db_path())?;
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS menu_cache (
            bundle_id TEXT PRIMARY KEY,
            menu_json TEXT NOT NULL,
            last_scanned INTEGER NOT NULL
        );
    "#)?;
    
    let _ = MENU_CACHE_DB.get_or_init(|| Arc::new(Mutex::new(conn)));
    Ok(())
}

pub fn get_cached_menu(bundle_id: &str) -> Result<Option<Vec<MenuBarItem>>> {
    let db = MENU_CACHE_DB.get()
        .ok_or_else(|| anyhow!("DB not initialized"))?;
    
    let conn = db.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
    
    let result: Option<String> = conn
        .query_row(
            "SELECT menu_json FROM menu_cache WHERE bundle_id = ?1",
            params![bundle_id],
            |row| row.get(0),
        )
        .optional()?;
    
    match result {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}
```

## Arc-Wrapped Data

### Sharing Immutable Data

```rust
pub struct App {
    // Scripts are wrapped in Arc for cheap cloning
    all_scripts: Vec<Arc<Script>>,
    filtered_scripts: Vec<Arc<Script>>,
    config: Arc<Config>,
}

impl App {
    fn select_script(&mut self) -> Option<Arc<Script>> {
        self.filtered_scripts.get(self.selected_index).cloned()
    }
    
    fn render_list_item(&self, script: &Arc<Script>) -> impl IntoElement {
        // Arc allows cheap reads without cloning script data
        let name = &script.name;
        let desc = script.description.as_deref().unwrap_or("");
        
        div()
            .child(name.clone())
            .child(desc.to_string())
    }
}
```

### Copyable Color Structs

```rust
/// Extract copyable subset of colors for closures
#[derive(Clone, Copy)]
pub struct ListItemColors {
    pub background: u32,
    pub selected_bg: u32,
    pub hover_bg: u32,
    pub text: u32,
    pub secondary_text: u32,
}

impl Theme {
    pub fn list_item_colors(&self) -> ListItemColors {
        ListItemColors {
            background: self.colors.background.main,
            selected_bg: self.colors.ui.selection,
            hover_bg: self.colors.ui.hover,
            text: self.colors.text.primary,
            secondary_text: self.colors.text.secondary,
        }
    }
}

// Usage in closures
fn render(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let colors = self.theme.list_item_colors(); // Copy
    
    uniform_list("items", self.items.len(), move |_this, range, _w, _cx| {
        // colors is Copy, so this closure can be Fn
        range.map(|i| {
            div().bg(rgb(colors.background))
        }).collect()
    })
}
```

## State Synchronization

### Focus-Aware State

```rust
impl App {
    fn check_focus_changed(&mut self, window: &Window, cx: &mut Context<Self>) {
        let is_focused = self.focus_handle.is_focused(window);
        
        if self.was_focused != is_focused {
            self.was_focused = is_focused;
            cx.notify(); // Trigger re-render for focus styling
        }
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.check_focus_changed(window, cx);
        
        let colors = self.theme.get_colors(self.was_focused);
        // ... render with focus-appropriate colors
    }
}
```

## Best Practices Summary

1. **Always call `cx.notify()`** after state changes that affect rendering
2. **Use `RwLock`** for globals with frequent reads, infrequent writes
3. **Use bounded channels** to prevent memory growth
4. **Coalesce rapid updates** (keyboard, scroll) to prevent lag
5. **Use `Arc`** for sharing immutable data across views
6. **Use copyable structs** in closures to avoid borrow issues
7. **Persist critical state** to SQLite for crash recovery
8. **Check focus state** in render and notify on changes
