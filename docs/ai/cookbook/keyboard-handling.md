# Keyboard Handling (GPUI)

## When to use
- You are adding key navigation or submit/cancel behavior to a prompt/window.
- You need key matching that works across GPUI key string variants (`up` and `arrowup`, `enter` and `return`, etc.).
- You are wiring keyboard handling in render code with `cx.listener(...)` and `.on_key_down(...)`.

## Do not do
- Do not normalize keys with `to_lowercase()` in hot paths.
- Do not match only one spelling/casing of a key.
- Do not attach `.on_key_down(...)` without ensuring the element can receive focus (`.track_focus(...)` where needed).

## Canonical files
- `src/ui_foundation/part_001.rs:12` defines allocation-free key helpers (`is_key_up`, `is_key_down`, `is_key_enter`, `is_key_escape`) using `eq_ignore_ascii_case`, including arrow/return aliases.
- `src/ui_foundation/part_001.rs:176` has tests proving accepted key variants (`up`/`arrowup`, `enter`/`return`, `escape`/`esc`).
- `src/prompts/path/render.rs:13` shows canonical `cx.listener` key handler setup with `event.keystroke.key.as_str()`.
- `src/prompts/path/render.rs:46` shows explicit variant matching (`"up" | "Up" | "ArrowUp" | "arrowup"`, etc.).
- `src/prompts/path/render.rs:210` shows listener wiring with `.on_key_down(handle_key)` and focus wiring at `src/prompts/path/render.rs:209`.
- `src/app_impl/actions_dialog.rs:24` shows routing through shared key helpers instead of duplicating variant lists.

## Minimal snippet
```rust
use crate::ui_foundation::{is_key_down, is_key_enter, is_key_escape, is_key_up};

let handle_key = cx.listener(|this: &mut Self, event: &gpui::KeyDownEvent, _window, cx| {
    let key = event.keystroke.key.as_str();
    if is_key_up(key) {
        this.move_up(cx);
    } else if is_key_down(key) {
        this.move_down(cx);
    } else if is_key_enter(key) {
        this.submit();
    } else if is_key_escape(key) {
        this.submit_cancel();
    }
});

div()
    .track_focus(&self.focus_handle)
    .on_key_down(handle_key)
```
