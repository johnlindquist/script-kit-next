# Research: Clipboard Share Sheet

## Files investigated
- `Cargo.toml`
- `src/actions/builders.rs`
- `src/platform.rs`
- `src/window_manager.rs`
- `src/clipboard_history/change_detection.rs`

## Current behavior
- The clipboard history actions include a `clipboard_share` action definition (label: "Share...") and tests assert its presence in the action list.
- No implementation exists that handles `clipboard_share` when an action is executed. A repo-wide search for `clipboard_share` only finds the action definition and its tests, so the action currently does nothing when selected.

## Root cause analysis
- There is no macOS share sheet implementation in the platform layer. The project already depends on `cocoa` and `objc` (see `Cargo.toml`), and uses Objective-C runtime patterns in `src/platform.rs`, but there is no `NSSharingServicePicker` integration.
- Result: the action is defined but never mapped to a native share sheet invocation.

## Proposed solution
### 1) Add a macOS share sheet helper
Implement a `show_share_sheet()` function in `src/platform.rs` (or a new platform submodule) that:
- Uses `NSSharingServicePicker` with an `NSArray` of items.
- Supports both content types:
  - Text: create an `NSString` and pass it as the share item.
  - Image: create an `NSImage` from PNG data (e.g., `NSImage::initWithData:` using an `NSData` payload) and pass it as the share item.
- Anchors the picker to the main window's `contentView`:
  - Use `window_manager::get_main_window()` to get the `NSWindow`.
  - Retrieve `contentView` and call `showRelativeToRect:ofView:preferredEdge:` (anchor to the view bounds or a centered rect).

### 2) Wire up the action handler
- In the action execution path (the same place other built-in actions like `clipboard_copy`, `clipboard_paste`, etc. are handled), add a `clipboard_share` match arm.
- On selection:
  - Resolve the currently selected clipboard entry.
  - For text entries, pass the entry text to `show_share_sheet()`.
  - For image entries, load image bytes from the clipboard history cache/database, then pass PNG bytes to `show_share_sheet()`.

### 3) Use existing dependencies and patterns
- No new dependencies are required; `cocoa` + `objc` are already in `Cargo.toml`.
- `src/platform.rs` already contains Objective-C interop patterns and is a natural home for the helper.
- `src/window_manager.rs` already provides main window tracking, which is needed for anchoring the share sheet.

## Verification
### What was changed
- Added `show_share_sheet()` in `src/platform.rs` with a `ShareSheetItem` enum (`Text` and `ImagePng`).
- Uses `NSSharingServicePicker` with `NSString` for text and `NSImage` created from `NSData` for PNG images.
- Anchors the picker to the main window `contentView`.
- Wired the `clipboard_share` action handler in `src/app_actions.rs`.

### Test results
- `cargo check` passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- `cargo test`: 2557 tests passed with 2 unrelated pre-existing failures (`clipboard_attach_to_ai` tests).

### Before/after comparison
- Before: `clipboard_share` action was defined but had no handler.
- After: selecting `clipboard_share` triggers the native macOS share sheet.

### Proposed solution alignment
- The implementation follows the proposed solution exactly.
