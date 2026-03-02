<!-- markdownlint-disable MD013 MD022 MD031 -->
# Agent Cookbook — Right/Wrong Patterns
Concrete, code-first patterns for this repo. Prefer every `Right` pattern shown here.

## Vendored Dependencies
`gpui` (core GPUI framework) is vendored at `vendor/gpui/` from Zed rev
`03416097`.
`gpui-component` (UI component library) is vendored at
`vendor/gpui-component/` from its own repository.
Both are local copies so agents can patch integration details directly
in this repo.
Typical local patches include diagnostics and layout-debugging hooks.

## GPUI Rendering (most common agent mistakes)
### `render()` is READ-ONLY
```rust
impl Render for PromptView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // WRONG: render mutates state
        self.items.push("late item".to_string());
        // RIGHT: only read state in render
        div().child(format!("{}", self.items.len()))
    }
}
impl PromptView {
    fn on_new_item(&mut self, value: String, cx: &mut Context<Self>) {
        self.items.push(value); // RIGHT: mutate in handler/action
        cx.notify();
    }
}
```
### `cx.notify()` is MANDATORY
```rust
impl PromptView {
    fn set_query(&mut self, query: String, _cx: &mut Context<Self>) {
        self.query = query;
        // WRONG: no notify => no redraw
    }
    fn set_query_right(&mut self, query: String, cx: &mut Context<Self>) {
        self.query = query;
        cx.notify(); // RIGHT: render-affecting mutation always notifies
    }
}
```
### Subscriptions MUST be stored
```rust
pub struct PromptView {
    model: Entity<Model>,
    model_sub: Option<Subscription>,
}
impl PromptView {
    fn wire_model(&mut self, cx: &mut Context<Self>) {
        // WRONG: dropped immediately
        cx.subscribe(&self.model, |this, _m, evt, cx| this.on_evt(evt, cx));
        // RIGHT: keep handle alive on struct
        let sub = cx.subscribe(&self.model, |this, _m, evt, cx| this.on_evt(evt, cx));
        self.model_sub = Some(sub);
    }
}
```
### `cx.spawn()` gives `WeakEntity`
```rust
impl PromptView {
    fn refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let result = fetch_items().await;
            // RIGHT: re-enter entity with update + .ok()
            this.update(cx, |this, cx| {
                if let Ok(items) = result { this.items = items; }
                cx.notify();
            }).ok();
        }).detach();
    }
}
```

### Tasks must be `.detach()`ed or stored
```rust
pub struct PromptView {
    poll_task: Option<Task<()>>,
}

impl PromptView {
    fn poll_once_wrong(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |_this, _cx| { poll_once().await; }); // WRONG: dropped
    }

    fn poll_once_right(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |_this, _cx| { poll_once().await; }).detach(); // RIGHT
    }

    fn poll_loop_right(&mut self, cx: &mut Context<Self>) {
        self.poll_task = Some(cx.spawn(async move |_this, _cx| { poll_loop().await; })); // RIGHT
    }
}
```

### Use `cx.spawn()` not `std::thread::spawn` for UI work
```rust
impl PromptView {
    fn kickoff_wrong(&mut self) {
        std::thread::spawn(|| {
            // WRONG: GPUI state/entity work from unmanaged thread
            println!("updating ui");
        });
    }

    fn kickoff_right(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let status = compute_status().await;
            this.update(cx, |this, cx| { this.status = status; cx.notify(); }).ok();
        }).detach();
    }
}
```

## ObjC Interop (objc 0.2 — NOT objc2)

### Wrong API family vs correct API family
```rust
// WRONG in this repo: objc2 APIs
use objc2::{declare_class, msg_send_id};

// RIGHT in this repo: objc 0.2 macros
use objc::{class, msg_send, sel, sel_impl};
use objc::runtime::Object;

let app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };
let window: *mut Object = unsafe { msg_send![app, keyWindow] };
let _: () = unsafe { msg_send![window, makeKeyAndOrderFront: std::ptr::null::<Object>()] };
let _s = sel!(setLevel:);
let _si = sel_impl!(setLevel:);
```

### Null-check `msg_send!` returns
```rust
use anyhow::bail;
use objc::{class, msg_send};
use objc::runtime::Object;

let panel: *mut Object = unsafe { msg_send![class!(NSPanel), alloc] };
if panel.is_null() {
    bail!("NSPanel alloc returned null"); // RIGHT: guard before next send
}
let _: () = unsafe { msg_send![panel, orderFront: std::ptr::null::<Object>()] };
```

### Use `c""` string literals
```rust
// WRONG: runtime CString assembly for fixed symbol names
let _wrong = std::ffi::CString::new("CGSMainConnectionID")?;

// RIGHT: compile-time C string literal
let symbol = c"CGSMainConnectionID";
let symbol_ptr = symbol.as_ptr();
```

### Private APIs: `dlsym` + `OnceLock` fallback
```rust
use std::sync::OnceLock;

type CgsMainConnectionId = unsafe extern "C" fn() -> i32;
static CGS_MAIN_CONNECTION_ID: OnceLock<Option<CgsMainConnectionId>> = OnceLock::new();

fn resolve_main_connection_id() -> Option<CgsMainConnectionId> {
    *CGS_MAIN_CONNECTION_ID.get_or_init(|| {
        let ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, c"CGSMainConnectionID".as_ptr()) };
        if ptr.is_null() { None } else { Some(unsafe { std::mem::transmute(ptr) }) }
    })
}
```

## Serde / Protocol Messages

### Protocol structs: `rename_all = "camelCase"`
```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct PromptUpdateWrong {
    prompt_id: String, // WRONG: serializes as prompt_id
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptUpdateRight {
    prompt_id: String, // RIGHT: serializes as promptId
}
```

### Optional protocol fields: always `#[serde(default)]`
```rust
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptConfigWrong {
    debounce_ms: Option<u64>, // WRONG: missing field can fail in strict flows
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptConfigRight {
    #[serde(default)]
    debounce_ms: Option<u64>, // RIGHT: absent => None
}
```

### Message routing: tagged enums with `#[serde(tag = "type")]`
```rust
#[derive(serde::Deserialize)]
struct RawMessageWrong {
    action: String,
    payload: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ProtocolMessageRight {
    Show { script_path: String },
    Hide,
    SetInput { value: String },
}
```

## async_channel — Always bounded in production

```rust
use async_channel::{bounded, Receiver, Sender};

fn make_queue_wrong() -> (Sender<Job>, Receiver<Job>) {
    async_channel::unbounded() // WRONG: no backpressure, unbounded growth
}

const JOB_QUEUE_CAPACITY: usize = 256;

fn make_queue_right() -> (Sender<Job>, Receiver<Job>) {
    bounded(JOB_QUEUE_CAPACITY) // RIGHT: bounded and predictable
}
```

## Theme System

### Color access: `theme.colors.*`, never hardcoded `rgb(0x...)`
```rust
fn render_badge_wrong() -> impl IntoElement {
    div().text_color(gpui::rgb(0xFF3B30)) // WRONG
}

fn render_badge_right() -> impl IntoElement {
    let theme = crate::theme::get_cached_theme();
    div().text_color(theme.colors.text_danger) // RIGHT
}
```

### Vibrancy: NEVER opaque `.bg()` on prompt containers
```rust
fn render_prompt_wrong() -> impl IntoElement {
    div().bg(gpui::rgb(0x101010)).child(render_prompt_body()) // WRONG
}

fn render_prompt_right() -> impl IntoElement {
    div().child(render_prompt_body()) // RIGHT: let root vibrancy show through
}
```

### Render path: use `get_cached_theme()`, never `load_theme()`
```rust
fn render_wrong(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    let theme = crate::theme::load_theme().unwrap(); // WRONG in render hot path
    div().text_color(theme.colors.text)
}

fn render_right(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    let theme = crate::theme::get_cached_theme(); // RIGHT: no args, returns cached Theme
    div().text_color(theme.colors.text)
}
```

## Error Handling

### Add context to error chains with `anyhow::Context`
```rust
use anyhow::{Context, Result};
use std::path::Path;

fn load_script_wrong(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?) // WRONG: no operation context
}

fn load_script_right(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read script file: {}", path.display())) // RIGHT
}
```

### Use `bail!` for preconditions
```rust
use anyhow::{bail, Result};

fn ensure_window_size_wrong(width: u32, height: u32) -> Result<()> {
    if width == 0 || height == 0 { return Ok(()); } // WRONG: silently accepts invalid input
    Ok(())
}

fn ensure_window_size_right(width: u32, height: u32) -> Result<()> {
    if width == 0 || height == 0 {
        bail!("window size must be non-zero (width={width}, height={height})"); // RIGHT
    }
    Ok(())
}
```

## Before You Edit — Checklist

Before modifying any file, verify:

1. Is this file include!()'d into main.rs? If yes: no use statements, no mod declarations.
2. Is this file in the High-Risk Files list (CLAUDE.md)? If yes: read the full file first.
3. Does this file have unsafe blocks? Add // SAFETY: comments to any you touch.
4. Does this file use ObjC interop? Use objc 0.2 APIs only (msg_send!, class!, sel!, sel_impl!).
5. Are you adding a new protocol message? Follow the Protocol Message Checklist below.

## Protocol Message Addition Checklist

To add a new stdin command / protocol message:

1. Create variant file: src/protocol/message/variants/your_command.rs
2. Add variant to Message enum in src/protocol/message/mod.rs with #[serde(rename = "yourCommand")]
3. Add handler in src/stdin_commands/mod.rs
4. Use #[serde(rename_all = "camelCase")] on the variant struct
5. Add #[serde(default)] on all optional fields
6. Note: ExternalCommand uses deny_unknown_fields; Message does NOT
7. Test: send JSON via stdin pipe, verify round-trip with cargo test

## Keyboard Event Propagation
```rust
// Register key handler via cx.listener() in render():
div()
  .track_focus(&self.focus_handle)
  .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
    let key = event.keystroke.key.as_str();
    if is_key_enter(key) {
      this.confirm(window, cx);
      cx.stop_propagation(); // RIGHT: prevent parent from also handling Enter
      return;
    }
    if is_key_escape(key) {
      this.cancel(window, cx);
      cx.stop_propagation();
      return;
    }
    // WRONG: bare return in _ arm — key silently swallowed
    // RIGHT: propagate unhandled keys to parent
    cx.propagate();
  }))

// Dispatch actions from key handlers:
// WRONG: cx.dispatch_action(action)
// RIGHT:
window.dispatch_action(action);
```

## Entity Creation Anti-Pattern
```rust
// WRONG: creating entity inside render() — new entity every frame, leaked subs
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let child = cx.new(|cx| ChildView::new(cx)); // WRONG: per-frame allocation
    div().child(child)
}

// RIGHT: create entity once in constructor, store on struct
fn new(cx: &mut Context<Self>) -> Self {
    let child = cx.new(|cx| ChildView::new(cx));
    Self { child }
}
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().child(self.child.clone()) // RIGHT: reuse stored entity
}
```

## rusqlite Patterns
```rust
use std::sync::{Arc, OnceLock};
use parking_lot::Mutex;
use rusqlite::Connection;

// Global connection singleton
static DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

fn get_db() -> anyhow::Result<Arc<Mutex<Connection>>> {
    Ok(DB.get_or_init(|| {
        let conn = Connection::open(db_path()).expect("DB open");
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")
            .expect("DB pragmas");
        Arc::new(Mutex::new(conn))
    }).clone())
}

// WRONG: prepare() reprepares every call
let mut stmt = conn.prepare("SELECT ...")?;
// RIGHT: prepare_cached() reuses statement
let mut stmt = conn.prepare_cached("SELECT ...")?;

// WRONG: lock().unwrap()
let guard = db.lock().unwrap();
// RIGHT: parking_lot Mutex never poisons
let guard = db.lock();
```

## ResultExt Error Patterns
```rust
use gpui::ResultExt;

// In event handlers where you can't propagate errors with ?:
// WRONG: silently ignoring
let _ = do_thing();
// RIGHT: log and continue
do_thing().log_err();
do_thing().warn_on_err();

// For recoverable operations in render-adjacent code:
let theme = load_custom_theme().unwrap_or_else(|e| {
    tracing::warn!(error = %e, "falling back to default theme");
    default_theme()
});
```

## Top 10 Agent Anti-Patterns (Quick Reference)

1. Using `ViewContext<Self>` — correct type is `Context<Self>`
2. Mutating state in render() — use event handlers
3. Forgetting cx.notify() after state changes — UI won't update
4. Using objc2 APIs — this project uses objc 0.2
5. Bare cx.subscribe() without storing — subscription dies immediately
6. Using get_cached_theme(cx) — it takes zero args: get_cached_theme()
7. Creating entities in render() — per-frame allocation leak
8. Missing cx.stop_propagation() — parent handlers also fire
9. Adding use statements in include!() files — they share main.rs scope
10. Using std::thread::spawn for UI work — use cx.spawn()
