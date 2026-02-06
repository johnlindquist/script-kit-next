# Research: Setup Card Button Cursor / Hit Testing (Chat Prompt)

Date: 2026-02-02

## 1) Files investigated

Primary file (requested):
- `src/prompts/chat.rs`
  - `render_setup_card`: lines ~1883-2050
  - Setup button container: lines ~1937-1945 (`id("setup-buttons-container")`)
  - Configure button: lines ~1946-1985 (`id("configure-button")`)
  - Claude Code button: lines ~1994-2033 (`id("configure-claude-code-btn")`)

## 2) Current behavior

Observed in `render_setup_card`:
- Both setup buttons apply `.cursor_pointer()` (`configure-button` at ~1959, `configure-claude-code-btn` at ~2007).
- Both buttons attach `.on_click(...)` listeners directly (not conditional wrappers), so click handlers are present.
- Both buttons also include `.hover(...)` and `.active(...)` styling, and container/button IDs are present for hit testing.

## 3) Root cause analysis

No implementation bug was found in this section. The setup card button cursor/hit testing code is already correct in the inspected range.

## 4) Proposed solution

No code changes are required in `src/prompts/chat.rs` for this issue.

## Verification

What was checked:
- `src/prompts/chat.rs` lines 1940-2040 in clean state.

Test results:
- `cargo check` passes successfully.

Findings:
- Both buttons (`configure-button` and `configure-claude-code-btn`) already have:
  - `.id()` for hit testing
  - `.cursor_pointer()` for hover cursor
  - `.hover()` for visual feedback
  - `.active()` for press feedback
  - `.on_click()` handlers
- Parent container `setup-buttons-container` does not block mouse events.

Conclusion:
- No changes needed. The `cursor_pointer()` implementation is correct.
- GPUI does not have an `overflow_visible()` method; default overflow is already visible.
