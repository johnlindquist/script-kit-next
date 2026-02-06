# Research: Pointer Cursor + Setup Buttons Hit Testing

## Scope
- File: `src/ai/window.rs`
- Focused range: lines `3174-3247`
- Question: pointer cursor styling vs click hit testing for setup buttons

## Findings

### 1) Button code in `src/ai/window.rs:3174-3247`
The setup view defines two clickable button `div`s:

- **Configure Vercel AI Gateway** (`configure-vercel-btn`)
  - Starts at line `3174`
  - Uses `.on_click(...)` to call `show_api_key_input(...)`
- **Connect to Claude Code** (`connect-claude-code-btn`)
  - Starts at line `3215`
  - Uses `.on_click(...)` to call `enable_claude_code(...)`

### 2) Both buttons have `.cursor_pointer()` but hit testing may depend on proper `.id()` placement
- First button has `.cursor_pointer()` at line `3185`
- Second button has `.cursor_pointer()` at line `3226`
- Both buttons should have stable `.id(...)` on the same clickable root element used for `.on_click(...)`

### 3) No `pointer_events` blocking found
A scan of `src/ai/window.rs` found no `pointer_events` usage, so there is no explicit pointer-event blocking in this file.

### 4) Proposed solution
Ensure both setup button root `div`s have `.id(...)` defined **before** `.cursor_pointer()` to satisfy GPUI hit-testing expectations.

Target pattern:

```rust
div()
    .id("configure-vercel-btn")
    .cursor_pointer()
    .on_click(...)

div()
    .id("connect-claude-code-btn")
    .cursor_pointer()
    .on_click(...)
```

This keeps the hit-testable identity attached to the same element that sets pointer cursor and click handler.

## Verification

1. Added `.id("setup-card")` to the setup card container at `src/ai/window.rs:3137`.
2. Both setup button root `div`s already had `.id(...)` before `.cursor_pointer()`:
   - `configure-vercel-btn` at `src/ai/window.rs:3181` before `.cursor_pointer()` at `src/ai/window.rs:3190`
   - `connect-claude-code-btn` at `src/ai/window.rs:3225` before `.cursor_pointer()` at `src/ai/window.rs:3234`
3. `cargo check` passes (`Finished dev profile`, exit code 0).
4. Code review confirms `on_click` handlers are in place at:
   - `src/ai/window.rs:3197` (`show_api_key_input(...)`)
   - `src/ai/window.rs:3241` (`enable_claude_code(...)`)
