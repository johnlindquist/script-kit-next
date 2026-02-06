# Research Notes: Notes Window Mouse Cursor Hiding

## 1) Files investigated

### A. Notes window implementation (`src/notes/window.rs`)

- `NotesApp` state struct (candidate place for new flag): `src/notes/window.rs:77-190`
  - Existing UI state flags cluster is around `src/notes/window.rs:99-116`.
- Notes render path: `src/notes/window.rs:1941-2249`
  - Root element chain begins at `src/notes/window.rs:1971`.
  - Existing mouse handlers near target insertion area:
    - `on_mouse_down(...)`: `src/notes/window.rs:1983-1993`
    - `on_hover(...)`: `src/notes/window.rs:1995-2002`
  - Keyboard capture hook:
    - `.capture_key_down(...)`: `src/notes/window.rs:2005-2240`

### B. GPUI cursor APIs (from Zed's GPUI checkout)

- `CursorStyle` enum includes hidden-pointer option:
  - `~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/gpui/src/platform.rs:1494-1583`
  - `CursorStyle::None` specifically at `.../platform.rs:1581-1582`
- Window-level cursor API:
  - `set_window_cursor_style(...)`: `~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/gpui/src/window.rs:2516-2525`
  - Note: docs in code say this should be called during paint/render phase.
- macOS platform behavior for hidden cursor:
  - `CursorStyle::None` maps to `NSCursor.setHiddenUntilMouseMoves:YES` in
    `~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/gpui/src/platform/mac/platform.rs:953-957`

### C. Zed editor reference implementation

- State + config:
  - `mouse_cursor_hidden` field: `~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/editor/src/editor.rs:1202`
  - Initialized false: `.../editor.rs:2397`
  - `hide_mouse_mode` setting initialization: `.../editor.rs:2399-2401`
- State transitions:
  - `show_mouse_cursor(...)`: `.../editor.rs:2727-2731`
  - `hide_mouse_cursor(...)`: `.../editor.rs:2734-2749`
- Mouse-move unhide behavior:
  - `mouse_moved(...)` calls `editor.show_mouse_cursor(cx)`: 
    `~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/editor/src/element.rs:1185-1199`
- Render-time cursor application:
  - If hidden, set window cursor style to none:
    `~/.cargo/git/checkouts/zed-a70e2ad075855582/94faaeb/crates/editor/src/element.rs:6502-6505`

## 2) Current behavior

### Notes window

- There is currently no mouse-cursor hiding logic in `src/notes/window.rs`.
- Search for cursor-style APIs in Notes returns no matches (`CursorStyle`, `set_window_cursor_style`, `mouse_cursor_hidden`, `on_mouse_move`).
- Current input/event handling in Notes focuses on panel toggles and keyboard routing (`capture_key_down`) but does not manage OS pointer visibility.

### Main Script Kit window

- Main window currently has text-caret blink state, not mouse-pointer hiding.
- Caret blink timer toggles `cursor_visible` in:
  - `src/app_impl.rs:138-177` (timer + `app.cursor_visible = !app.cursor_visible` at `:161`)
  - initial state `cursor_visible: true` at `src/app_impl.rs:318`

## 3) Root cause analysis

Feature is not yet implemented for Notes.

Specifically:
- No Notes state flag tracks whether pointer should be hidden.
- No render-time call to `window.set_window_cursor_style(CursorStyle::None)` in Notes render path.
- No keyboard-input path in Notes marks cursor hidden.
- No mouse-move handler in Notes clears hidden state.

So, Notes cannot currently reproduce Zed-style "hide pointer while typing, show on mouse move" behavior.

## 4) Proposed solution approach

### 4.1 Add state flag to `NotesApp`

- Add `mouse_cursor_hidden: bool` to `NotesApp` near existing UI flags (`src/notes/window.rs:99-116`).
- Initialize to `false` in `NotesApp::new(...)` where other fields are initialized.

### 4.2 Apply hidden cursor in render

- In `Render::render` (`src/notes/window.rs:1941+`, around root construction at `:1971`), set cursor style when hidden:
  - `window.set_window_cursor_style(CursorStyle::None)` when `self.mouse_cursor_hidden` is `true`.
- This follows GPUI's intended usage during paint phase (`gpui/src/window.rs:2516-2525`) and mirrors Zed's render-time pattern (`editor/src/element.rs:6502-6505`).

### 4.3 Hide cursor on keyboard input

- In `.capture_key_down(...)` (`src/notes/window.rs:2005+`), set `self.mouse_cursor_hidden = true` when processing keyboard input.
- If state changes, call `cx.notify()` to trigger re-render.

### 4.4 Restore cursor on mouse movement

- Add `.on_mouse_move(...)` handler in root render chain near existing mouse handlers (`src/notes/window.rs:1983-2002` area).
- In handler: if `self.mouse_cursor_hidden` is true, set it to false and `cx.notify()`.
- This mirrors Zed's pattern (`editor/src/element.rs:1185-1199` + `editor/src/editor.rs:2727-2731`).

## Suggested implementation sketch (not applied in this research task)

```rust
// in NotesApp state
mouse_cursor_hidden: bool,

// in render()
if self.mouse_cursor_hidden {
    window.set_window_cursor_style(CursorStyle::None);
}

// in capture_key_down
if !self.mouse_cursor_hidden {
    self.mouse_cursor_hidden = true;
    cx.notify();
}

// in on_mouse_move
if self.mouse_cursor_hidden {
    self.mouse_cursor_hidden = false;
    cx.notify();
}
```

## Verification

### 1) What was changed

- Added imports: `CursorStyle`, `MouseMoveEvent`.
- Added state field: `mouse_cursor_hidden`.
- Added helper methods:
  - `hide_mouse_cursor`
  - `show_mouse_cursor`
  - `set_mouse_cursor_hidden_state`
- Added render-time cursor style application so hidden state sets `CursorStyle::None`.
- Added `on_mouse_move` handler to restore/show cursor on mouse movement.
- Added hide call in `capture_key_down` so typing hides the mouse cursor.

### 2) Test results

- `cargo check` passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- All 34 notes tests passed.

### 3) Before/after comparison

- Before: mouse cursor remained visible while typing in Notes.
- After: cursor hides on keyboard input and shows on mouse move.

### 4) Pattern alignment

- Behavior matches the cursor-hiding pattern used by the Zed editor.
