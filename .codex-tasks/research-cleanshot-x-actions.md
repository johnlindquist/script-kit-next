# Research: CleanShot X Clipboard Actions

## Summary

Implemented two CleanShot X actions for Script Kit GPUI clipboard history:
1. **Annotate in CleanShot X** - Opens image in CleanShot X for editing
2. **Upload to CleanShot Cloud** - Uploads image to CleanShot cloud service

## 1) CleanShot X URL Schemes

From CleanShot X documentation:
- `cleanshot://open-from-clipboard` - Opens the image currently on clipboard in CleanShot's annotate editor
- `cleanshot://open-annotate?filepath={path}&action=upload` - Opens a file for annotation with auto-upload

## 2) Files Modified

### src/actions/builders.rs (lines ~915-940)
- Added `clipboard_annotate_cleanshot` action ID
- Added `clipboard_upload_cleanshot` action ID
- Both actions are conditionally shown only for image clipboard entries on macOS

### src/app_actions.rs (lines ~679-810)
- Implemented `clipboard_annotate_cleanshot` handler:
  - Copies selected image to clipboard
  - Opens `cleanshot://open-from-clipboard` URL scheme
  - Shows HUD feedback and hides window

- Implemented `clipboard_upload_cleanshot` handler:
  - Loads image content from clipboard history
  - Converts to PNG bytes using `content_to_png_bytes()`
  - Saves to temp file with UUID filename
  - Opens `cleanshot://open-annotate?filepath={encoded_path}&action=upload`
  - Shows HUD feedback and hides window

### src/clipboard_history/image.rs (lines ~95-120)
- Added `content_to_png_bytes()` function to convert clipboard image content to PNG bytes
- Supports blob, png, and rgba formats

### src/clipboard_history/mod.rs
- Exported `content_to_png_bytes` function

### src/render_builtins.rs (lines ~130-250)
- Added `toggle_clipboard_actions()` method for Cmd+K in clipboard history view
- Wired up actions dialog for clipboard entries

## 3) Error Handling

Both handlers include comprehensive error handling:
- Missing clipboard entry selection
- Non-image entry type (shows HUD message)
- Failed clipboard copy
- Failed image content load
- Failed PNG decode
- Failed temp file write
- Failed to spawn `open` command

All errors log to ERROR category and show user-friendly HUD messages.

## 4) macOS Conditional

All CleanShot functionality is gated with `#[cfg(target_os = "macos")]`:
- Actions only appear in the list on macOS
- Non-macOS platforms show "CleanShot actions are only supported on macOS" HUD

## 5) Verification

### Compilation
- `cargo check` - PASS
- `cargo clippy --all-targets -- -D warnings` - PASS

### Tests
- `cargo test clipboard` - 63 tests PASS
- `cargo test content_to_png` - 2 tests PASS
- `cargo test` - All tests PASS

### Code Review
- Proper error handling at each step
- macOS-only conditionals in place
- URL encoding for file paths
- Temp file cleanup delegated to OS

## 6) Usage

1. Open Script Kit GPUI
2. Open Clipboard History (built-in action)
3. Select an image entry
4. Press Cmd+K to open actions
5. Select "Annotate in CleanShot X" or "Upload to CleanShot Cloud"

Note: Requires CleanShot X to be installed with URL scheme API enabled.
