# Editor Basics & Find/Replace Expert Bundle

## Executive Summary

This bundle contains everything needed to understand and extend Script Kit GPUI's custom code editor implementation. The editor currently supports basic text editing (insert, delete, backspace), cursor navigation (arrows, home/end, word movement), selection (shift+arrows, cmd+a, mouse), clipboard (cmd+c/v/x), undo/redo (cmd+z, cmd+shift+z), syntax highlighting, and line numbers. **Find/Replace functionality is not yet implemented and is marked as P2 priority in the EDITOR_PLAN.md.**

### Key Problems:
1. **Missing Find/Replace (Cmd+F)**: Users expect standard find/replace functionality - this is P2 in the roadmap but not implemented.
2. **Missing Go to Line (Cmd+G)**: Users expect to jump to a specific line number.
3. **Missing Duplicate Line (Cmd+D or Cmd+Shift+D)**: Common editor action for duplicating current line.
4. **Missing Line Comment Toggle (Cmd+/)**: Toggle comment on current line or selection.
5. **Missing Indent/Outdent Selection (Tab/Shift+Tab on selection)**: Currently Tab just inserts spaces.

### Required Fixes:
1. `src/editor.rs`: Add find/replace state, UI overlay, and keyboard handling (Cmd+F, Cmd+G, etc.)
2. `src/editor.rs`: Implement `find_next()`, `find_prev()`, `replace()`, `replace_all()` methods
3. `src/editor.rs`: Add Go to Line dialog (Cmd+G or Ctrl+G)
4. `src/editor.rs`: Add line operations (duplicate, comment toggle, indent/outdent)
5. Protocol consideration: May need to extend `src/protocol.rs` if SDK needs to control find/replace

### Files Included:
- `src/editor.rs`: Main EditorPrompt component (~1645 lines) - core editor implementation
- `src/syntax.rs`: Syntax highlighting using syntect - provides `highlight_code_lines()`
- `src/snippet.rs`: VSCode snippet/template parsing for tabstop navigation
- `src/actions.rs`: Actions dialog - pattern for adding editor-specific actions (Cmd+K menu)
- `EDITOR_PLAN.md`: Original implementation plan with roadmap and architecture decisions
- `tests/sdk/test-editor.ts`: SDK test for editor functionality
- `tests/smoke/test-editor-actions-keys.ts`: Test for editor actions panel keyboard handling

### Current Editor Features (Already Implemented):
| Feature | Status | Keyboard Shortcut |
|---------|--------|-------------------|
| Insert text | ✅ | Any printable key |
| Delete forward | ✅ | Delete |
| Backspace | ✅ | Backspace |
| Cursor navigation | ✅ | Arrow keys |
| Word navigation | ✅ | Alt/Option + Arrow |
| Line start/end | ✅ | Cmd + Left/Right, Home/End |
| Document start/end | ✅ | Cmd + Up/Down |
| Selection | ✅ | Shift + navigation keys |
| Select all | ✅ | Cmd + A |
| Copy | ✅ | Cmd + C |
| Cut | ✅ | Cmd + X |
| Paste | ✅ | Cmd + V |
| Undo | ✅ | Cmd + Z |
| Redo | ✅ | Cmd + Shift + Z |
| Submit | ✅ | Cmd + Enter |
| Cancel | ✅ | Escape |
| Tab (indent/tabstop) | ✅ | Tab |
| Line numbers | ✅ | Always visible |
| Syntax highlighting | ✅ | Automatic |
| Snippet tabstops | ✅ | Tab/Shift+Tab to navigate |

### Missing Editor Features (Users Would Expect):
| Feature | Priority | Keyboard Shortcut |
|---------|----------|-------------------|
| Find | P2 | Cmd + F |
| Find Next | P2 | Cmd + G or F3 |
| Find Previous | P2 | Cmd + Shift + G or Shift + F3 |
| Replace | P2 | Cmd + H or Cmd + Alt + F |
| Replace All | P2 | Cmd + Shift + H |
| Go to Line | P2 | Cmd + G (when find not open) or Ctrl + G |
| Duplicate Line | P3 | Cmd + D or Cmd + Shift + D |
| Delete Line | P3 | Cmd + Shift + K |
| Move Line Up/Down | P3 | Alt + Up/Down |
| Toggle Comment | P3 | Cmd + / |
| Indent Selection | P3 | Tab (when selection) |
| Outdent Selection | P3 | Shift + Tab (when selection) |
| Multi-cursor | P3 | Cmd + D (add next occurrence) |

---

## Packx Output

This file contains 7 filtered files from the repository.

## Files

### src/syntax.rs

```rs
//! Syntax highlighting module using syntect
//!
//! Provides syntax highlighting for code strings with colors that integrate
//! with the existing theme system. Colors are returned as hex u32 values.
//!
//! NOTE: syntect's default syntax set doesn't include TypeScript, so we use
//! JavaScript syntax for .ts files (which works well for highlighting).

#![allow(dead_code)]

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A highlighted span of text with its associated color
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedSpan {
    /// The text content of this span
    pub text: String,
    /// The color as a hex u32 value (0xRRGGBB format)
    pub color: u32,
    /// Whether this span ends a line (contains newline)
    pub is_line_end: bool,
}

impl HighlightedSpan {
    /// Create a new highlighted span
    pub fn new(text: impl Into<String>, color: u32) -> Self {
        let text_str = text.into();
        let is_line_end = text_str.ends_with('\n');
        Self {
            text: text_str,
            color,
            is_line_end,
        }
    }
}

/// A complete highlighted line with all its spans
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    pub spans: Vec<HighlightedSpan>,
}

/// Convert a syntect Style color to a hex u32 value
fn style_to_hex_color(style: &Style) -> u32 {
    let fg = style.foreground;
    ((fg.r as u32) << 16) | ((fg.g as u32) << 8) | (fg.b as u32)
}

/// Map language name/extension to syntect syntax name
fn map_language_to_syntax(language: &str) -> &str {
    match language.to_lowercase().as_str() {
        "typescript" | "ts" => "JavaScript",
        "javascript" | "js" => "JavaScript",
        "markdown" | "md" => "Markdown",
        "json" => "JSON",
        "rust" | "rs" => "Rust",
        "python" | "py" => "Python",
        "html" => "HTML",
        "css" => "CSS",
        "shell" | "sh" | "bash" => "Bourne Again Shell (bash)",
        "yaml" | "yml" => "YAML",
        "toml" => "Makefile",
        _ => language,
    }
}

/// Highlight code with syntax coloring, returning lines of spans
pub fn highlight_code_lines(code: &str, language: &str) -> Vec<HighlightedLine> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-eighties.dark"];
    let default_color = 0xcccccc_u32;
    let syntax_name = map_language_to_syntax(language);

    let syntax = ps
        .find_syntax_by_name(syntax_name)
        .or_else(|| ps.find_syntax_by_extension(language))
        .or_else(|| ps.find_syntax_by_name("JavaScript"))
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in LinesWithEndings::from(code) {
        let mut line_spans = Vec::new();
        match highlighter.highlight_line(line, &ps) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    if !text.is_empty() {
                        let clean_text = text.trim_end_matches('\n');
                        if !clean_text.is_empty() {
                            line_spans.push(HighlightedSpan::new(clean_text, style_to_hex_color(&style)));
                        }
                    }
                }
            }
            Err(_) => {
                let clean_line = line.trim_end_matches('\n');
                if !clean_line.is_empty() {
                    line_spans.push(HighlightedSpan::new(clean_line, default_color));
                }
            }
        }
        result.push(HighlightedLine { spans: line_spans });
    }
    result
}

pub fn supported_languages() -> Vec<&'static str> {
    vec!["typescript", "ts", "javascript", "js", "markdown", "md", "json", "rust", "rs", "python", "py", "html", "css", "shell", "sh", "bash", "yaml", "yml", "toml"]
}
```

### src/editor.rs (Key Sections)

```rs
//! GPUI Editor Prompt Component
//!
//! A full-featured code editor for Script Kit with:
//! - Text editing (insert, delete, backspace)
//! - Cursor navigation (arrows, home/end, word movement)
//! - Selection (shift+arrows, cmd+a, mouse)
//! - Clipboard (cmd+c/v/x)
//! - Undo/redo (cmd+z, cmd+shift+z)
//! - Syntax highlighting
//! - Line numbers
//! - Monospace font

use gpui::{
    div, prelude::*, px, rgb, rgba, uniform_list, ClipboardItem, Context, FocusHandle, Focusable,
    Render, SharedString, UniformListScrollHandle, Window,
};
use ropey::Rope;
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use crate::config::Config;
use crate::logging;
use crate::snippet::ParsedSnippet;
use crate::syntax::{highlight_code_lines, HighlightedLine};
use crate::theme::Theme;

pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;

const BASE_CHAR_WIDTH: f32 = 8.4;
const BASE_FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT_MULTIPLIER: f32 = 1.43;
const GUTTER_WIDTH: f32 = 50.0;
const MAX_UNDO_HISTORY: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: CursorPosition,
    pub head: CursorPosition,
}

impl Selection {
    pub fn is_empty(&self) -> bool { self.anchor == self.head }
    pub fn ordered(&self) -> (CursorPosition, CursorPosition) {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.column <= self.head.column)
        {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

/// EditorPrompt - Full-featured code editor
pub struct EditorPrompt {
    pub id: String,
    rope: Rope,
    language: String,
    cursor: CursorPosition,
    selection: Selection,
    cursor_visible: bool,
    highlighted_lines: Vec<HighlightedLine>,
    needs_rehighlight: bool,
    scroll_handle: UniformListScrollHandle,
    undo_stack: VecDeque<EditorSnapshot>,
    redo_stack: VecDeque<EditorSnapshot>,
    focus_handle: FocusHandle,
    on_submit: SubmitCallback,
    theme: Arc<Theme>,
    config: Arc<Config>,
    content_height: Option<gpui::Pixels>,
    snippet_state: Option<SnippetState>,
    pub suppress_keys: bool,
    // TODO: Add find/replace state here
    // find_state: Option<FindReplaceState>,
}

// Key method: handle_key_event - this is where keyboard shortcuts are processed
fn handle_key_event(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
    if self.suppress_keys { return; }

    let key = event.keystroke.key.to_lowercase();
    let cmd = event.keystroke.modifiers.platform;
    let shift = event.keystroke.modifiers.shift;
    let alt = event.keystroke.modifiers.alt;

    match (key.as_str(), cmd, shift, alt) {
        // Submit/Cancel
        ("enter", true, false, false) => self.submit(),
        ("escape", _, _, _) => self.cancel(),

        // Undo/Redo
        ("z", true, false, false) => self.undo(),
        ("z", true, true, false) => self.redo(),

        // Clipboard
        ("c", true, false, false) => self.copy(cx),
        ("x", true, false, false) => self.cut(cx),
        ("v", true, false, false) => self.paste(cx),

        // Select all
        ("a", true, false, false) => self.select_all(),

        // Navigation
        ("left" | "arrowleft", false, _, false) => self.move_left(shift),
        ("right" | "arrowright", false, _, false) => self.move_right(shift),
        ("up" | "arrowup", false, _, false) => self.move_up(shift),
        ("down" | "arrowdown", false, _, false) => self.move_down(shift),

        // Word navigation (Alt + arrow)
        ("left" | "arrowleft", false, _, true) => self.move_word_left(shift),
        ("right" | "arrowright", false, _, true) => self.move_word_right(shift),

        // Line start/end
        ("left" | "arrowleft", true, _, false) => self.move_to_line_start(shift),
        ("right" | "arrowright", true, _, false) => self.move_to_line_end(shift),
        ("home", false, _, _) => self.move_to_line_start(shift),
        ("end", false, _, _) => self.move_to_line_end(shift),

        // Document start/end
        ("up" | "arrowup", true, _, false) => self.move_to_document_start(shift),
        ("down" | "arrowdown", true, _, false) => self.move_to_document_end(shift),

        // Editing
        ("backspace", _, _, _) => self.backspace(),
        ("delete", _, _, _) => self.delete(),
        ("enter", false, _, _) => self.insert_newline(),

        // Tab handling
        ("tab", false, false, false) => {
            if self.snippet_state.is_some() {
                self.next_tabstop();
            } else {
                self.insert_text("    ");
            }
        }
        ("tab", false, true, false) => {
            if self.snippet_state.is_some() {
                self.prev_tabstop();
            }
        }

        // TODO: Add find/replace shortcuts here
        // ("f", true, false, false) => self.show_find_dialog(cx),
        // ("h", true, false, false) => self.show_replace_dialog(cx),
        // ("g", true, false, false) => self.find_next(cx),
        // ("g", true, true, false) => self.find_prev(cx),

        // Character input
        _ => {
            if let Some(ref key_char) = event.keystroke.key_char {
                if let Some(ch) = key_char.chars().next() {
                    if !ch.is_control() && !cmd {
                        self.insert_char(ch);
                    }
                }
            }
        }
    }
    cx.notify();
}

// Text manipulation methods (already implemented)
fn insert_text(&mut self, text: &str) { /* ... */ }
fn backspace(&mut self) { /* ... */ }
fn delete(&mut self) { /* ... */ }

// Navigation methods (already implemented)
fn move_left(&mut self, extend_selection: bool) { /* ... */ }
fn move_right(&mut self, extend_selection: bool) { /* ... */ }
fn move_up(&mut self, extend_selection: bool) { /* ... */ }
fn move_down(&mut self, extend_selection: bool) { /* ... */ }
fn move_word_left(&mut self, extend_selection: bool) { /* ... */ }
fn move_word_right(&mut self, extend_selection: bool) { /* ... */ }
fn move_to_line_start(&mut self, extend_selection: bool) { /* ... */ }
fn move_to_line_end(&mut self, extend_selection: bool) { /* ... */ }

// Selection methods (already implemented)
fn select_all(&mut self) { /* ... */ }
fn get_selected_text(&self) -> String { /* ... */ }

// Clipboard methods (already implemented)
fn copy(&self, cx: &mut Context<Self>) { /* ... */ }
fn cut(&mut self, cx: &mut Context<Self>) { /* ... */ }
fn paste(&mut self, cx: &mut Context<Self>) { /* ... */ }

// Undo/redo (already implemented)
fn undo(&mut self) { /* ... */ }
fn redo(&mut self) { /* ... */ }
```

### EDITOR_PLAN.md (Key Sections)

From the plan document, the feature priorities are:

| Feature | Priority | Complexity |
|---------|----------|------------|
| Display code with syntax highlighting | P0 | Low |
| Basic text editing (insert, delete) | P0 | Medium |
| Cursor navigation | P0 | Medium |
| Selection | P0 | Medium |
| Copy/Paste | P0 | Low |
| Undo/Redo | P1 | Medium |
| Line numbers | P1 | Low |
| **Find/Replace (cmd+f)** | **P2** | **High** |
| Multi-cursor | P3 | Very High |
| Scroll with virtualization | P0 | Medium |
| Actions bar | P2 | Medium |

---

## Implementation Guide

### Step 1: Add Find/Replace State to EditorPrompt

```rust
// File: src/editor.rs
// Location: Add to EditorPrompt struct fields

/// State for find/replace dialog
#[derive(Debug, Clone)]
pub struct FindReplaceState {
    /// Search query
    pub query: String,
    /// Replacement text
    pub replacement: String,
    /// Whether find dialog is visible
    pub is_visible: bool,
    /// Whether replace input is visible
    pub show_replace: bool,
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Use regex
    pub use_regex: bool,
    /// All match ranges (start_char_idx, end_char_idx)
    pub matches: Vec<(usize, usize)>,
    /// Currently highlighted match index
    pub current_match_idx: Option<usize>,
}

impl Default for FindReplaceState {
    fn default() -> Self {
        Self {
            query: String::new(),
            replacement: String::new(),
            is_visible: false,
            show_replace: false,
            case_sensitive: false,
            use_regex: false,
            matches: Vec::new(),
            current_match_idx: None,
        }
    }
}

// Add to EditorPrompt struct:
pub struct EditorPrompt {
    // ... existing fields ...
    
    /// Find/replace state
    find_state: FindReplaceState,
}
```

### Step 2: Implement Find Methods

```rust
// File: src/editor.rs
// Location: Add as impl methods on EditorPrompt

impl EditorPrompt {
    /// Show find dialog (Cmd+F)
    fn show_find(&mut self, cx: &mut Context<Self>) {
        self.find_state.is_visible = true;
        self.find_state.show_replace = false;
        // If there's a selection, use it as the search query
        let selected = self.get_selected_text();
        if !selected.is_empty() && !selected.contains('\n') {
            self.find_state.query = selected;
            self.perform_find();
        }
        cx.notify();
    }
    
    /// Show find and replace dialog (Cmd+H or Cmd+Alt+F)
    fn show_find_replace(&mut self, cx: &mut Context<Self>) {
        self.find_state.is_visible = true;
        self.find_state.show_replace = true;
        let selected = self.get_selected_text();
        if !selected.is_empty() && !selected.contains('\n') {
            self.find_state.query = selected;
            self.perform_find();
        }
        cx.notify();
    }
    
    /// Hide find dialog
    fn hide_find(&mut self, cx: &mut Context<Self>) {
        self.find_state.is_visible = false;
        self.find_state.matches.clear();
        self.find_state.current_match_idx = None;
        cx.notify();
    }
    
    /// Perform search and populate matches
    fn perform_find(&mut self) {
        self.find_state.matches.clear();
        self.find_state.current_match_idx = None;
        
        if self.find_state.query.is_empty() {
            return;
        }
        
        let content = self.rope.to_string();
        let query = &self.find_state.query;
        
        // Simple substring search (can be extended to regex)
        let search_content = if self.find_state.case_sensitive {
            content.clone()
        } else {
            content.to_lowercase()
        };
        let search_query = if self.find_state.case_sensitive {
            query.clone()
        } else {
            query.to_lowercase()
        };
        
        let mut start = 0;
        while let Some(pos) = search_content[start..].find(&search_query) {
            let match_start = start + pos;
            let match_end = match_start + query.len();
            self.find_state.matches.push((match_start, match_end));
            start = match_start + 1;
        }
        
        // Jump to first match after cursor
        if !self.find_state.matches.is_empty() {
            let cursor_idx = self.cursor_to_char_idx(self.cursor);
            self.find_state.current_match_idx = self.find_state.matches
                .iter()
                .position(|(start, _)| *start >= cursor_idx)
                .or(Some(0));
        }
    }
    
    /// Find next occurrence (Cmd+G or F3)
    fn find_next(&mut self, cx: &mut Context<Self>) {
        if self.find_state.matches.is_empty() {
            return;
        }
        
        let next_idx = match self.find_state.current_match_idx {
            Some(idx) => (idx + 1) % self.find_state.matches.len(),
            None => 0,
        };
        
        self.find_state.current_match_idx = Some(next_idx);
        self.jump_to_current_match(cx);
    }
    
    /// Find previous occurrence (Cmd+Shift+G or Shift+F3)
    fn find_prev(&mut self, cx: &mut Context<Self>) {
        if self.find_state.matches.is_empty() {
            return;
        }
        
        let prev_idx = match self.find_state.current_match_idx {
            Some(idx) if idx > 0 => idx - 1,
            _ => self.find_state.matches.len() - 1,
        };
        
        self.find_state.current_match_idx = Some(prev_idx);
        self.jump_to_current_match(cx);
    }
    
    /// Jump cursor to current match and select it
    fn jump_to_current_match(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.find_state.current_match_idx {
            if let Some(&(start, end)) = self.find_state.matches.get(idx) {
                let start_pos = self.char_idx_to_cursor(start);
                let end_pos = self.char_idx_to_cursor(end);
                
                self.cursor = end_pos;
                self.selection = Selection::new(start_pos, end_pos);
                
                // Scroll to make match visible
                self.scroll_handle.scroll_to_item(
                    start_pos.line,
                    gpui::ScrollStrategy::Center,
                );
                
                cx.notify();
            }
        }
    }
    
    /// Replace current match
    fn replace_current(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.find_state.current_match_idx {
            if let Some(&(start, end)) = self.find_state.matches.get(idx) {
                self.save_undo_state();
                
                // Delete the match
                self.rope.remove(start..end);
                
                // Insert replacement
                self.rope.insert(start, &self.find_state.replacement);
                
                self.needs_rehighlight = true;
                
                // Re-run find to update matches
                self.perform_find();
                
                cx.notify();
            }
        }
    }
    
    /// Replace all matches
    fn replace_all(&mut self, cx: &mut Context<Self>) {
        if self.find_state.matches.is_empty() {
            return;
        }
        
        self.save_undo_state();
        
        // Replace from end to start to preserve indices
        let matches: Vec<_> = self.find_state.matches.iter().rev().cloned().collect();
        
        for (start, end) in matches {
            self.rope.remove(start..end);
            self.rope.insert(start, &self.find_state.replacement);
        }
        
        self.needs_rehighlight = true;
        self.perform_find(); // Will clear matches since text changed
        
        cx.notify();
    }
}
```

### Step 3: Add Keyboard Shortcuts

```rust
// File: src/editor.rs
// Location: In handle_key_event match statement, add these cases:

match (key.as_str(), cmd, shift, alt) {
    // ... existing shortcuts ...
    
    // Find/Replace
    ("f", true, false, false) => self.show_find(cx),
    ("h", true, false, false) => self.show_find_replace(cx),
    ("g", true, false, false) => {
        if self.find_state.is_visible {
            self.find_next(cx);
        } else {
            self.show_go_to_line(cx);
        }
    }
    ("g", true, true, false) => self.find_prev(cx),
    ("f3", false, false, false) => self.find_next(cx),
    ("f3", false, true, false) => self.find_prev(cx),
    
    // When find dialog is visible, Escape closes it
    ("escape", _, _, _) if self.find_state.is_visible => self.hide_find(cx),
    
    // ... rest of existing shortcuts ...
}
```

### Step 4: Render Find Dialog Overlay

```rust
// File: src/editor.rs
// Location: Add to render method, after the editor content

fn render_find_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let colors = &self.theme.colors;
    
    div()
        .absolute()
        .top(px(8.))
        .right(px(8.))
        .w(px(320.))
        .bg(rgb(colors.background.search_box))
        .rounded(px(8.))
        .border_1()
        .border_color(rgb(colors.ui.border))
        .shadow_lg()
        .p(px(8.))
        .flex()
        .flex_col()
        .gap(px(8.))
        // Find input
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .child(
                    div()
                        .flex_1()
                        .h(px(28.))
                        .px(px(8.))
                        .bg(rgb(colors.background.main))
                        .rounded(px(4.))
                        .flex()
                        .items_center()
                        .child(SharedString::from(
                            if self.find_state.query.is_empty() {
                                "Find...".to_string()
                            } else {
                                self.find_state.query.clone()
                            }
                        ))
                )
                .child(
                    // Match count
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text.muted))
                        .child(SharedString::from(format!(
                            "{}/{}",
                            self.find_state.current_match_idx.map(|i| i + 1).unwrap_or(0),
                            self.find_state.matches.len()
                        )))
                )
        )
        // Replace input (conditionally shown)
        .when(self.find_state.show_replace, |d| {
            d.child(
                div()
                    .h(px(28.))
                    .px(px(8.))
                    .bg(rgb(colors.background.main))
                    .rounded(px(4.))
                    .flex()
                    .items_center()
                    .child(SharedString::from(
                        if self.find_state.replacement.is_empty() {
                            "Replace...".to_string()
                        } else {
                            self.find_state.replacement.clone()
                        }
                    ))
            )
        })
        // Buttons
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(4.))
                .child(self.render_find_button("↑", "Find Previous"))
                .child(self.render_find_button("↓", "Find Next"))
                .when(self.find_state.show_replace, |d| {
                    d.child(self.render_find_button("Replace", "Replace"))
                     .child(self.render_find_button("All", "Replace All"))
                })
        )
}

fn render_find_button(&self, label: &str, _tooltip: &str) -> impl IntoElement {
    let colors = &self.theme.colors;
    
    div()
        .px(px(8.))
        .py(px(4.))
        .bg(rgb(colors.background.main))
        .rounded(px(4.))
        .text_xs()
        .text_color(rgb(colors.text.secondary))
        .hover(|s| s.bg(rgb(colors.accent.selected_subtle)))
        .cursor_pointer()
        .child(SharedString::from(label.to_string()))
}
```

### Step 5: Add Go to Line Dialog

```rust
// File: src/editor.rs
// Location: Add new state and methods

#[derive(Debug, Clone, Default)]
pub struct GoToLineState {
    pub is_visible: bool,
    pub line_input: String,
}

impl EditorPrompt {
    fn show_go_to_line(&mut self, cx: &mut Context<Self>) {
        self.go_to_line_state.is_visible = true;
        self.go_to_line_state.line_input.clear();
        cx.notify();
    }
    
    fn go_to_line(&mut self, cx: &mut Context<Self>) {
        if let Ok(line_num) = self.go_to_line_state.line_input.parse::<usize>() {
            let target_line = (line_num.saturating_sub(1)).min(self.line_count().saturating_sub(1));
            self.cursor = CursorPosition::new(target_line, 0);
            self.selection = Selection::caret(self.cursor);
            self.scroll_handle.scroll_to_item(target_line, gpui::ScrollStrategy::Center);
        }
        self.go_to_line_state.is_visible = false;
        cx.notify();
    }
}
```

### Testing

After implementing these changes, verify with:

1. **Find Dialog (Cmd+F)**:
   - Opens find overlay
   - Typing in find input searches document
   - Shows match count (e.g., "3/10")
   - F3 / Cmd+G cycles through matches
   - Escape closes dialog

2. **Replace Dialog (Cmd+H)**:
   - Opens with replace input visible
   - "Replace" replaces current match
   - "Replace All" replaces all matches

3. **Go to Line (Cmd+G when find closed, or Ctrl+G)**:
   - Opens line number input
   - Enter jumps to specified line
   - Escape cancels

4. **Run tests**:
   ```bash
   cargo check && cargo clippy && cargo test
   echo '{"type": "run", "path": "'$(pwd)'/tests/sdk/test-editor.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   ```

---

## Instructions For The Next AI Agent

You are reading the "Editor Basics & Find/Replace Expert Bundle". This file is self-contained and includes all the context you should assume you have.

Your job:

* Design and describe the minimal, safe changes needed to add Find/Replace and other expected editor functionality.
* Operate **only** on the files and code snippets included in this bundle. If you need additional files or context, clearly say so.

When you propose changes, follow these rules strictly:

1. Always provide **precise code snippets** that can be copy-pasted directly into the repo.
2. Always include **exact file paths** (e.g. `src/editor.rs`) and, when possible, line numbers or a clear description of the location (e.g. "add to handle_key_event match statement").
3. Never describe code changes only in prose. Show the full function or block as it should look **after** the change, or show both "before" and "after" versions.
4. Keep instructions **unmistakable and unambiguous**. A human or tool following your instructions should not need to guess what to do.
5. Assume you cannot see any files outside this bundle. If you must rely on unknown code, explicitly note assumptions and risks.

Key architectural decisions to maintain:
- Use `ropey::Rope` for text buffer operations (efficient for large files)
- Use `syntect` for syntax highlighting via `highlight_code_lines()`
- Follow the existing pattern of `cx.notify()` after state changes
- Use theme colors from `self.theme.colors` - never hardcode colors
- Add keyboard shortcuts to the `handle_key_event` match statement
- Use `uniform_list` for virtualized rendering of lines

When you answer, you do not need to restate this bundle. Work directly with the code and instructions it contains and return a clear, step-by-step plan plus exact code edits.
