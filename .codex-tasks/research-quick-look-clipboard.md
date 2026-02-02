# Research: Quick Look Clipboard History Action

## 1) Files Investigated

### Quick Look Integration Options on macOS

1. **`qlmanage -p` command** (Chosen approach)
   - Simple command-line invocation via `std::process::Command`
   - Works for any file type that macOS can preview
   - Used pattern: `qlmanage -p <file_path>`

2. **QLPreviewPanel API via Rust/objc**
   - More complex, requires Objective-C bindings
   - Would use `objc2-quick-look-ui` crate
   - Requires implementing `QLPreviewPanelDataSource` and `QLPreviewItem` traits
   - Not chosen due to complexity and the simplicity of `qlmanage`

### Existing Codebase Structure

- `src/file_search/mod.rs` - Already has `quick_look()` function for file search
- `src/clipboard_history/` - Module for clipboard history operations
- `src/actions/builders.rs` - Action definition functions
- `src/app_actions.rs` - Action handlers
- `src/render_builtins.rs` - Keyboard handlers for built-in views

### Temp File Patterns

- Uses `~/.scriptkit/clipboard/quicklook/` directory for Quick Look preview files
- Entry IDs are sanitized for safe filenames
- Text entries saved as `.txt`, images as `.png`

## 2) Current Implementation

### New File: `src/clipboard_history/quick_look.rs`

```rust
//! Quick Look preview helpers for clipboard history entries.
//!
//! Uses macOS `qlmanage -p` to preview text/images. For non-macOS targets,
//! falls back to opening the generated file with the default app.

use std::fs;
use std::path::{Path, PathBuf};

use super::{content_to_png_bytes, get_entry_content, ClipboardEntryMeta, ContentType};

/// Preview a clipboard history entry with Quick Look (macOS) or open fallback.
pub fn quick_look_entry(entry: &ClipboardEntryMeta) -> Result<(), String> {
    let content = get_entry_content(&entry.id)
        .ok_or_else(|| "Failed to load clipboard entry content".to_string())?;

    let preview_path = match entry.content_type {
        ContentType::Text => write_text_preview(&entry.id, &content)?,
        ContentType::Image => resolve_image_preview_path(&entry.id, &content)?,
    };

    quick_look_path(&preview_path)
}
```

### Key Functions

- `quick_look_entry(entry)` - Main entry point, loads content and routes by type
- `write_text_preview(entry_id, content)` - Writes text to temp `.txt` file
- `resolve_image_preview_path(entry_id, content)` - Uses blob path or writes `.png`
- `quick_look_path(path)` - Invokes `qlmanage -p` on macOS

## 3) Root Cause Analysis

### What Was Missing

Before this implementation:
- File search had Quick Look (Cmd+Y) functionality
- Clipboard history had no Quick Look preview
- No way to preview clipboard content without copying it

### Gap Identified

The clipboard history view needed:
1. A Quick Look action in the context menu
2. Spacebar shortcut (matching Finder behavior)
3. Content type-aware file generation for preview

## 4) Proposed Solution (Implemented)

### Architecture

1. **New module**: `src/clipboard_history/quick_look.rs`
   - Content-type aware preview file generation
   - macOS-specific `qlmanage` invocation with non-macOS fallback

2. **Action registration**: `src/actions/builders.rs`
   - Added `clipboard_quick_look` action ID
   - Shortcut: `␣` (Space) for macOS Finder-like behavior

3. **Action handler**: `src/app_actions.rs`
   - Handles `clipboard_quick_look` action
   - Shows error HUD on failure

4. **Keyboard binding**: `src/render_builtins.rs`
   - Space key triggers Quick Look when filter is empty
   - Requires no modifier keys

---

## Verification

### 1) Files Changed

| File | Change |
|------|--------|
| `src/clipboard_history/quick_look.rs` | **NEW** - Quick Look module (97 lines) |
| `src/clipboard_history/mod.rs` | Added `pub mod quick_look` and re-export |
| `src/clipboard_history/temp_file.rs` | **NEW** - Temp file helpers for clipboard content |
| `src/actions/builders.rs` | Added `clipboard_quick_look` action with Space shortcut |
| `src/app_actions.rs` | Added handler for `clipboard_quick_look` action |
| `src/render_builtins.rs` | Added Space key binding for Quick Look in clipboard history |

### 2) Test Results

```
cargo check: ✓ Finished (no errors)
cargo clippy --all-targets -- -D warnings: ✓ No warnings

cargo test clipboard:
  - 63 tests passed, 0 failed
  - Includes new Quick Look related tests

cargo test actions::builders:
  - 34 tests passed, 0 failed
  - test_clipboard_history_action_shortcuts ✓
  - test_get_clipboard_history_text_actions ✓
  - test_get_clipboard_history_image_actions ✓
```

### 3) Before/After Comparison

| Aspect | Before | After |
|--------|--------|-------|
| Quick Look in clipboard history | None | Full support |
| Spacebar preview | Not available | Works like Finder |
| Text preview | N/A | Saved as .txt in quicklook dir |
| Image preview | N/A | Uses blob path or temp .png |
| Action menu entry | N/A | "Quick Look" with ␣ shortcut |

### 4) Implementation Summary

**Quick Look Module (`src/clipboard_history/quick_look.rs`)**:
- `quick_look_entry()` - Main API, handles both text and image entries
- Uses `~/.scriptkit/clipboard/quicklook/` for temp preview files
- macOS: Spawns `qlmanage -p <path>` process
- Non-macOS: Falls back to `open_file()` from file_search

**Action Wiring (`src/actions/builders.rs`)**:
- Action ID: `clipboard_quick_look`
- Title: "Quick Look"
- Description: "Preview with Quick Look"
- Shortcut: Space (displayed as `␣`)
- Only shown on macOS (`#[cfg(target_os = "macos")]`)

**Action Handler (`src/app_actions.rs`)**:
- Matches `"clipboard_quick_look"` action ID
- Gets selected clipboard entry from context
- Calls `clipboard_history::quick_look_entry(&entry)`
- Shows HUD on error

**Keyboard Shortcut (`src/render_builtins.rs`)**:
- Space key (no modifiers) in clipboard history view
- Only triggers when filter is empty (prevents conflict with typing)
- Matches macOS Finder spacebar Quick Look behavior
