<!-- markdownlint-disable MD013 MD022 MD031 -->
# Agent Cookbook — Right/Wrong Patterns
Concrete, code-first patterns for this repo. Prefer every `Right` pattern shown here.
## GPUI Rendering (most common agent mistakes)
### `render()` is READ-ONLY
```rust
impl Render for PromptView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        // WRONG: render mutates state
        self.items.push("late item".to_string());
        // RIGHT: only read state in render
        div().child(format!("{}", self.items.len()))
    }
}
impl PromptView {
    fn on_new_item(&mut self, value: String, cx: &mut ViewContext<Self>) {
        self.items.push(value); // RIGHT: mutate in handler/action
        cx.notify();
    }
}
```
### `cx.notify()` is MANDATORY
```rust
impl PromptView {
    fn set_query(&mut self, query: String, _cx: &mut ViewContext<Self>) {
        self.query = query;
        // WRONG: no notify => no redraw
    }
    fn set_query_right(&mut self, query: String, cx: &mut ViewContext<Self>) {
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
    fn wire_model(&mut self, cx: &mut ViewContext<Self>) {
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
    fn refresh(&mut self, cx: &mut ViewContext<Self>) {
        cx.spawn(|this, mut cx| async move {
            let result = fetch_items().await;
            // RIGHT: re-enter entity with update + .ok()
            this.update(&mut cx, |this, cx| {
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
    fn poll_once_wrong(&mut self, cx: &mut ViewContext<Self>) {
        cx.spawn(|_this, _cx| async move { poll_once().await; }); // WRONG: dropped
    }

    fn poll_once_right(&mut self, cx: &mut ViewContext<Self>) {
        cx.spawn(|_this, _cx| async move { poll_once().await; }).detach(); // RIGHT
    }

    fn poll_loop_right(&mut self, cx: &mut ViewContext<Self>) {
        self.poll_task = Some(cx.spawn(|_this, _cx| async move { poll_loop().await; })); // RIGHT
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

    fn kickoff_right(&mut self, cx: &mut ViewContext<Self>) {
        cx.spawn(|this, mut cx| async move {
            let status = compute_status().await;
            this.update(&mut cx, |this, cx| { this.status = status; cx.notify(); }).ok();
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

fn render_badge_right(cx: &mut ViewContext<Self>) -> impl IntoElement {
    let theme = crate::theme::get_cached_theme(cx);
    div().text_color(theme.colors.text_danger) // RIGHT
}
```

### Vibrancy: NEVER opaque `.bg()` on prompt containers
```rust
fn render_prompt_wrong(cx: &mut ViewContext<Self>) -> impl IntoElement {
    div().bg(gpui::rgb(0x101010)).child(render_prompt_body(cx)) // WRONG
}

fn render_prompt_right(cx: &mut ViewContext<Self>) -> impl IntoElement {
    div().child(render_prompt_body(cx)) // RIGHT: let root vibrancy show through
}
```

### Render path: use `get_cached_theme()`, never `load_theme()`
```rust
fn render_wrong(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    let theme = crate::theme::load_theme().unwrap(); // WRONG in render hot path
    div().text_color(theme.colors.text)
}

fn render_right(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    let theme = crate::theme::get_cached_theme(cx); // RIGHT
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
