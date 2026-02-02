# Unfinished Tasks from Today's Sessions - Script Kit GPUI

## Task 1: Syntax Highlighting Light Theme Fix (HIGHEST PRIORITY)
**Root Cause Identified:** `src/syntax.rs` uses hardcoded dark theme:
```rust
fn get_theme() -> &'static Theme {
    THEME.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        ts.themes["base16-eighties.dark"].clone()  // Always dark!
    })
}
```

**Fix Required:**
1. Add `LIGHT_THEME` and `DARK_THEME` statics
2. Make `get_theme(is_dark: bool)` return appropriate theme
3. Use "InspiredGitHub" or "Solarized (light)" for light mode
4. Update `highlight_code_lines()` to accept `is_dark: bool`
5. Update all callers in `src/app_render.rs` to pass theme mode
6. Update default foreground color for light mode (0x333333 vs 0xcccccc)

## Task 2: Notes Window - Quick Capture Implementation
**Research Finding:** Quick capture is incomplete per codebase analysis.
**Files:** `src/notes/window.rs`, `src/notes/storage.rs`

Implement:
1. Global hotkey quick capture (Fn+Q style)
2. Floating capture window that auto-saves
3. Context-aware capture (clipboard, selection)

## Task 3: Notes Window - Inline Markdown Preview
**Research Finding:** Typora-style inline preview recommended.
**File:** `src/notes/window.rs`

Implement:
1. Inline markdown rendering behind cursor
2. Auto-format headers, bold, italic as you type
3. Preserve raw markdown when cursor is on line

## Task 4: AI Chat - Replace Polling with Channels
**Research Finding:** 50ms polling causes overhead.
**File:** `src/ai/window.rs`

Fix:
1. Replace 50ms polling with async_channel
2. Use event-driven updates instead of timer
3. Improve streaming responsiveness

## Task 5: AI Chat - Add Syntax Highlighting to Code Blocks
**Research Finding:** Code blocks lack highlighting.
**Files:** `src/ai/window.rs`, integrate with `src/syntax.rs`

Implement:
1. Detect code blocks in markdown
2. Apply syntect highlighting
3. Add copy button to code blocks
4. Add language badge

## Task 6: AI Chat - Context Variables (@mentions)
**Research Finding:** Copilot/Cursor pattern is highly effective.
**File:** `src/ai/window.rs`

Implement:
1. `@file:` to reference files
2. `@clipboard` for clipboard content
3. `@selection` for selected text
4. Autocomplete dropdown when typing @

## Task 7: AI Chat - Slash Commands
**Research Finding:** /explain, /fix, /tests pattern from Copilot.
**File:** `src/ai/window.rs`

Implement:
1. `/explain` - explain selected code
2. `/fix` - suggest fixes
3. `/tests` - generate tests
4. Command palette on `/` keystroke

## Task 8: Notes Window - Tag System with Autocomplete
**Research Finding:** Inline #tag detection with autocomplete.
**Files:** `src/notes/window.rs`, `src/notes/storage.rs`

Implement:
1. Detect #hashtags inline
2. Tag autocomplete (8-10 suggestions max)
3. Store tags in SQLite
4. Filter notes by tag

## Task 9: AI Chat - Message Edit/Delete
**Research Finding:** Basic feature missing.
**Files:** `src/ai/window.rs`, `src/ai/storage.rs`

Implement:
1. Edit button on user messages
2. Delete button with confirmation
3. Re-generate from edited message
4. Update SQLite storage

## Task 10: Notes Window - Always-on-Top Pin Toggle
**Research Finding:** Standard feature in floating note apps.
**File:** `src/notes/window.rs`

Implement:
1. Pin button in header
2. Toggle NSWindowLevel between normal and floating
3. Persist pin state per note
4. Visual indicator when pinned
