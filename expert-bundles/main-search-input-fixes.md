# Main Input Search Issues - Expert Bundle

## Executive Summary
The main script list search input has multiple issues: 1) Search filtering uses a 16ms debounce that causes perceived lag during typing, 2) Deleting all text with backspace doesn't properly clear the search/reset the list, and 3) Standard text input operations (Cmd+V paste, Cmd+A select all, Cmd+C copy, Cmd+X cut) are not implemented for the search input.

### Key Problems:
1. **Debounce over-coalescing**: The two-stage filtering system uses `filter_compute_generation` coalescing with 16ms delay, but the debounce logic may be interfering with proper filter clearing when backspace empties the input.
2. **Missing computed_filter_text sync on empty**: When `filter_text` is cleared (via repeated backspace or clear), `computed_filter_text` may not sync immediately, leaving the UI showing stale results.
3. **No clipboard/selection support**: The main search input handler (`handle_key` in script list render) doesn't handle Cmd+V (paste), Cmd+A (select all), Cmd+C (copy), or Cmd+X (cut) - these are standard text input expectations.
4. **No cursor positioning**: The search input displays text but has no cursor position tracking - text is always appended at the end, with no ability to move cursor left/right or select text ranges.

### Required Fixes:
1. `src/main.rs` (lines 2705-2773): Fix `update_filter()` to ensure proper state sync when clearing
2. `src/main.rs` (lines 7975-8210): Add clipboard operations (paste/copy/cut) and text selection to the script list keyboard handler
3. `src/main.rs`: Add cursor position tracking (`filter_cursor_position: usize`) and selection state (`filter_selection: Option<(usize, usize)>`) to App struct
4. `src/main.rs`: Add left/right arrow key handling for cursor movement in the search input

### Files Included:
- `src/main.rs`: Main app state, filter logic, keyboard handling for script list
- `src/prompts/arg.rs`: Reference implementation for how arg prompt handles text input
- `src/editor.rs`: Reference implementation showing proper copy/paste/select_all patterns

---

## Core Code Sections

### 1. App State Definition (src/main.rs lines 1590-1670)

```rust
    selected_index: usize,
    filter_text: String,
    // ... other fields ...
    
    // P1: Cache for filtered_results() - invalidate on filter_text change only
    cached_filtered_results: Vec<scripts::SearchResult>,
    filter_cache_key: String,
    // P1: Cache for get_grouped_results() - invalidate on filter_text change only
    cached_grouped_items: Arc<[GroupedListItem]>,
    cached_grouped_flat_results: Arc<[scripts::SearchResult]>,
    grouped_cache_key: String,
    // P3: Two-stage filter - display vs search separation with coalescing
    /// What the search cache is built from (may lag behind filter_text during rapid typing)
    computed_filter_text: String,
    /// Generation counter for coalescing filter updates
    filter_compute_generation: u64,
    /// Whether a filter compute task is currently running
    filter_compute_task_running: bool,
```

**Note**: Missing fields for proper text input:
- `filter_cursor_position: usize` - cursor position within filter_text
- `filter_selection: Option<(usize, usize)>` - selection range (start, end) if any

### 2. update_filter() Function (src/main.rs lines 2705-2773)

```rust
    fn update_filter(
        &mut self,
        new_char: Option<char>,
        backspace: bool,
        clear: bool,
        cx: &mut Context<Self>,
    ) {
        // P3: Stage 1 - Update filter_text immediately (displayed in input)
        if clear {
            self.filter_text.clear();
            self.selected_index = 0;
            self.last_scrolled_index = None;
            // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
            self.main_list_state.scroll_to_reveal_item(0);
            self.last_scrolled_index = Some(0);
            // P3: Clear also immediately updates computed text (no coalescing needed)
            self.computed_filter_text.clear();
        } else if backspace && !self.filter_text.is_empty() {
            self.filter_text.pop();
            self.selected_index = 0;
            self.last_scrolled_index = None;
            // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
            self.main_list_state.scroll_to_reveal_item(0);
            self.last_scrolled_index = Some(0);
        } else if let Some(ch) = new_char {
            self.filter_text.push(ch);
            self.selected_index = 0;
            self.last_scrolled_index = None;
            // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
            self.main_list_state.scroll_to_reveal_item(0);
            self.last_scrolled_index = Some(0);
        }

        // P3: Notify immediately so input field updates (responsive typing)
        cx.notify();

        // P3: Stage 2 - Coalesce expensive search work with 16ms delay
        // This prevents multiple search computations during rapid typing
        self.filter_compute_generation = self.filter_compute_generation.wrapping_add(1);
        let current_generation = self.filter_compute_generation;
        let filter_snapshot = self.filter_text.clone();

        // Only spawn a new task if one isn't already running
        if !self.filter_compute_task_running {
            self.filter_compute_task_running = true;
            cx.spawn(async move |this, cx| {
                // Wait 16ms for coalescing window (one frame at 60fps)
                Timer::after(std::time::Duration::from_millis(16)).await;
                
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        app.filter_compute_task_running = false;
                        
                        // Only update if generation still matches (no newer input)
                        if app.filter_compute_generation == current_generation {
                            // Sync computed_filter_text with filter_text snapshot
                            if app.computed_filter_text != filter_snapshot {
                                app.computed_filter_text = filter_snapshot;
                                // This will trigger cache recompute on next get_grouped_results_cached()
                                app.update_window_size();
                                cx.notify();
                            }
                        }
                        // If generation changed, a newer task will handle the update
                    })
                });
            }).detach();
        }
    }
```

**BUG IDENTIFIED**: When backspace empties `filter_text`, the code does NOT immediately sync `computed_filter_text`. The sync only happens inside the 16ms async task. But if `filter_compute_task_running` is already true from a previous keystroke, no new task is spawned, and `computed_filter_text` never gets cleared.

### 3. Script List Keyboard Handler (src/main.rs lines 7975-8210)

```rust
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check SDK action shortcuts FIRST (before built-in shortcuts)
                if !this.action_shortcuts.is_empty() {
                    let key_combo =
                        Self::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                    if let Some(action_name) = this.action_shortcuts.get(&key_combo).cloned() {
                        if this.trigger_action_by_name(&action_name, cx) {
                            return;
                        }
                    }
                }

                if has_cmd {
                    let has_shift = event.keystroke.modifiers.shift;

                    match key_str.as_str() {
                        "l" => {
                            this.toggle_logs(cx);
                            return;
                        }
                        "k" => {
                            this.toggle_actions(cx, window);
                            return;
                        }
                        // ... other Cmd shortcuts ...
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            this.handle_action("copy_path".to_string(), cx);
                            return;
                        }
                        // ... more shortcuts ...
                        _ => {}
                    }
                }

                // ... actions popup handling ...

                match key_str.as_str() {
                    "up" | "arrowup" => {
                        // ... navigation ...
                    }
                    "down" | "arrowdown" => {
                        // ... navigation ...
                    }
                    "enter" => this.execute_selected(cx),
                    "escape" => {
                        if !this.filter_text.is_empty() {
                            this.update_filter(None, false, true, cx);
                        } else {
                            // ... hide window ...
                        }
                    }
                    "backspace" => this.update_filter(None, true, false, cx),
                    "space" | " " => {
                        // ... alias matching ...
                        this.update_filter(Some(' '), false, false, cx);
                    }
                    _ => {
                        // Allow all printable characters
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.update_filter(Some(ch), false, false, cx);
                                }
                            }
                        }
                    }
                }
            },
        );
```

**MISSING FUNCTIONALITY**:
- No `"a"` with `has_cmd` for select all
- No `"v"` with `has_cmd` for paste  
- No `"c"` with `has_cmd` (without shift) for copy
- No `"x"` with `has_cmd` for cut
- No `"left"/"arrowleft"` or `"right"/"arrowright"` for cursor movement

### 4. Reference: Editor Copy/Paste/Select (src/editor.rs lines 1017-1069)

```rust
    /// Select all text
    fn select_all(&mut self) {
        self.selection.anchor = CursorPosition::start();
        let last_line = self.line_count().saturating_sub(1);
        self.cursor = CursorPosition::new(last_line, self.line_len(last_line));
        self.selection.head = self.cursor;
    }

    /// Copy selection to clipboard
    fn copy(&self, cx: &mut Context<Self>) {
        let text = self.get_selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            logging::log("EDITOR", "Copied to clipboard");
        }
    }

    /// Cut selection to clipboard
    fn cut(&mut self, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            return;
        }
        let text = self.get_selected_text();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.save_undo_state();
        self.delete_selection_internal();
        self.needs_rehighlight = true;
    }

    /// Paste from clipboard
    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.insert_text(&text);
                logging::log("EDITOR", &format!("Pasted {} chars", text.len()));
            }
        }
    }
```

### 5. Editor Keyboard Handler Pattern (src/editor.rs lines 2031-2043)

```rust
            // Undo/Redo
            ("z", true, false, false) => self.undo(),
            ("z", true, true, false) => self.redo(),

            // Clipboard
            ("c", true, false, false) => self.copy(cx),
            ("x", true, false, false) => self.cut(cx),
            ("v", true, false, false) => self.paste(cx),

            // Select all
            ("a", true, false, false) => self.select_all(),
```

---

## Implementation Guide

### Step 1: Add Cursor Position and Selection State to App

```rust
// File: src/main.rs
// Location: App struct definition (around line 1594, after filter_text)

    filter_text: String,
    /// Cursor position within filter_text (byte offset)
    filter_cursor: usize,
    /// Selection range (start, end byte offsets) - None if no selection
    filter_selection: Option<(usize, usize)>,
```

Initialize these in the `new()` function (around line 1966):
```rust
            filter_text: String::new(),
            filter_cursor: 0,
            filter_selection: None,
```

### Step 2: Fix update_filter() to Immediately Sync on Empty

```rust
// File: src/main.rs
// Location: fn update_filter() around line 2705
// Replace the entire function with:

    fn update_filter(
        &mut self,
        new_char: Option<char>,
        backspace: bool,
        clear: bool,
        cx: &mut Context<Self>,
    ) {
        // P3: Stage 1 - Update filter_text immediately (displayed in input)
        if clear {
            self.filter_text.clear();
            self.filter_cursor = 0;
            self.filter_selection = None;
            self.selected_index = 0;
            self.last_scrolled_index = None;
            self.main_list_state.scroll_to_reveal_item(0);
            self.last_scrolled_index = Some(0);
            // P3: Clear IMMEDIATELY updates computed text - no coalescing needed
            self.computed_filter_text.clear();
            // Invalidate grouped cache immediately on clear
            self.grouped_cache_key = String::from("\0_CLEAR_\0");
        } else if backspace && !self.filter_text.is_empty() {
            // Handle backspace with cursor position
            if let Some((start, end)) = self.filter_selection.take() {
                // Delete selection
                let min = start.min(end);
                let max = start.max(end);
                self.filter_text.drain(min..max);
                self.filter_cursor = min;
            } else if self.filter_cursor > 0 {
                // Delete char before cursor
                let prev_char_boundary = self.filter_text[..self.filter_cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.filter_text.drain(prev_char_boundary..self.filter_cursor);
                self.filter_cursor = prev_char_boundary;
            }
            self.selected_index = 0;
            self.last_scrolled_index = None;
            self.main_list_state.scroll_to_reveal_item(0);
            self.last_scrolled_index = Some(0);
            
            // CRITICAL FIX: If filter_text is now empty, sync immediately
            if self.filter_text.is_empty() {
                self.computed_filter_text.clear();
                self.grouped_cache_key = String::from("\0_CLEARED_\0");
            }
        } else if let Some(ch) = new_char {
            // Delete selection first if any
            if let Some((start, end)) = self.filter_selection.take() {
                let min = start.min(end);
                let max = start.max(end);
                self.filter_text.drain(min..max);
                self.filter_cursor = min;
            }
            // Insert char at cursor
            self.filter_text.insert(self.filter_cursor, ch);
            self.filter_cursor += ch.len_utf8();
            self.selected_index = 0;
            self.last_scrolled_index = None;
            self.main_list_state.scroll_to_reveal_item(0);
            self.last_scrolled_index = Some(0);
        }

        // P3: Notify immediately so input field updates (responsive typing)
        cx.notify();

        // P3: Stage 2 - Coalesce expensive search work with 16ms delay
        // Skip coalescing if filter is empty (already synced above)
        if !self.filter_text.is_empty() {
            self.filter_compute_generation = self.filter_compute_generation.wrapping_add(1);
            let current_generation = self.filter_compute_generation;
            let filter_snapshot = self.filter_text.clone();

            // Only spawn a new task if one isn't already running
            if !self.filter_compute_task_running {
                self.filter_compute_task_running = true;
                cx.spawn(async move |this, cx| {
                    Timer::after(std::time::Duration::from_millis(16)).await;
                    
                    let _ = cx.update(|cx| {
                        this.update(cx, |app, cx| {
                            app.filter_compute_task_running = false;
                            
                            if app.filter_compute_generation == current_generation {
                                if app.computed_filter_text != filter_snapshot {
                                    app.computed_filter_text = filter_snapshot;
                                    app.update_window_size();
                                    cx.notify();
                                }
                            }
                        })
                    });
                }).detach();
            }
        }
    }
```

### Step 3: Add Paste/Copy/Cut/Select All to Keyboard Handler

```rust
// File: src/main.rs
// Location: Inside handle_key closure (around line 8002-8058)
// Add these cases inside the `if has_cmd { match key_str.as_str() { ... } }` block:

                        // Text editing shortcuts for search input
                        "a" => {
                            // Cmd+A - Select all filter text
                            if !this.filter_text.is_empty() {
                                this.filter_selection = Some((0, this.filter_text.len()));
                                this.filter_cursor = this.filter_text.len();
                                cx.notify();
                            }
                            return;
                        }
                        "c" if !has_shift => {
                            // Cmd+C - Copy selected filter text (or all if no selection)
                            if let Some((start, end)) = this.filter_selection {
                                let min = start.min(end);
                                let max = start.max(end);
                                let selected = &this.filter_text[min..max];
                                if !selected.is_empty() {
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected.to_string()));
                                    logging::log("UI", &format!("Copied {} chars from filter", selected.len()));
                                }
                            } else if !this.filter_text.is_empty() {
                                // No selection - copy all
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(this.filter_text.clone()));
                                logging::log("UI", &format!("Copied all filter text: {} chars", this.filter_text.len()));
                            }
                            return;
                        }
                        "x" => {
                            // Cmd+X - Cut selected filter text
                            if let Some((start, end)) = this.filter_selection.take() {
                                let min = start.min(end);
                                let max = start.max(end);
                                let selected = this.filter_text[min..max].to_string();
                                if !selected.is_empty() {
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected));
                                    this.filter_text.drain(min..max);
                                    this.filter_cursor = min;
                                    // Re-trigger filter update for the changed text
                                    this.selected_index = 0;
                                    if this.filter_text.is_empty() {
                                        this.computed_filter_text.clear();
                                        this.grouped_cache_key = String::from("\0_CUT_\0");
                                    }
                                    cx.notify();
                                    logging::log("UI", "Cut filter text");
                                }
                            }
                            return;
                        }
                        "v" => {
                            // Cmd+V - Paste into filter
                            if let Some(item) = cx.read_from_clipboard() {
                                if let Some(text) = item.text() {
                                    // Delete selection first if any
                                    if let Some((start, end)) = this.filter_selection.take() {
                                        let min = start.min(end);
                                        let max = start.max(end);
                                        this.filter_text.drain(min..max);
                                        this.filter_cursor = min;
                                    }
                                    // Filter out control characters and newlines
                                    let clean_text: String = text.chars()
                                        .filter(|c| !c.is_control() || *c == ' ')
                                        .take(500) // Limit paste length
                                        .collect();
                                    if !clean_text.is_empty() {
                                        this.filter_text.insert_str(this.filter_cursor, &clean_text);
                                        this.filter_cursor += clean_text.len();
                                        this.selected_index = 0;
                                        // Trigger filter update
                                        this.filter_compute_generation = this.filter_compute_generation.wrapping_add(1);
                                        cx.notify();
                                        logging::log("UI", &format!("Pasted {} chars into filter", clean_text.len()));
                                    }
                                }
                            }
                            return;
                        }
```

### Step 4: Add Cursor Movement with Arrow Keys

```rust
// File: src/main.rs
// Location: Inside handle_key match block (around line 8119-8210)
// Add these cases for arrow key cursor movement:

                match key_str.as_str() {
                    "left" | "arrowleft" => {
                        if has_cmd {
                            // Cmd+Left - move to start
                            this.filter_cursor = 0;
                            if !event.keystroke.modifiers.shift {
                                this.filter_selection = None;
                            } else {
                                // Shift+Cmd+Left - select to start
                                let old_cursor = this.filter_cursor;
                                this.filter_selection = Some((old_cursor, 0));
                            }
                            cx.notify();
                            return;
                        }
                        if this.filter_cursor > 0 {
                            let prev = this.filter_text[..this.filter_cursor]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            if event.keystroke.modifiers.shift {
                                // Extend/create selection
                                match this.filter_selection {
                                    Some((anchor, _)) => {
                                        this.filter_selection = Some((anchor, prev));
                                    }
                                    None => {
                                        this.filter_selection = Some((this.filter_cursor, prev));
                                    }
                                }
                            } else {
                                this.filter_selection = None;
                            }
                            this.filter_cursor = prev;
                            cx.notify();
                        }
                        return;
                    }
                    "right" | "arrowright" => {
                        if has_cmd {
                            // Cmd+Right - move to end
                            this.filter_cursor = this.filter_text.len();
                            if !event.keystroke.modifiers.shift {
                                this.filter_selection = None;
                            } else {
                                let old_cursor = this.filter_cursor;
                                this.filter_selection = Some((old_cursor, this.filter_text.len()));
                            }
                            cx.notify();
                            return;
                        }
                        if this.filter_cursor < this.filter_text.len() {
                            let next = this.filter_text[this.filter_cursor..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| this.filter_cursor + i)
                                .unwrap_or(this.filter_text.len());
                            if event.keystroke.modifiers.shift {
                                match this.filter_selection {
                                    Some((anchor, _)) => {
                                        this.filter_selection = Some((anchor, next));
                                    }
                                    None => {
                                        this.filter_selection = Some((this.filter_cursor, next));
                                    }
                                }
                            } else {
                                this.filter_selection = None;
                            }
                            this.filter_cursor = next;
                            cx.notify();
                        }
                        return;
                    }
                    "up" | "arrowup" => {
                        // ... existing navigation code ...
```

### Step 5: Update Filter Display to Show Cursor Position

```rust
// File: src/main.rs
// Location: Around line 8305-8362 (search input rendering)
// Replace the filter display section with cursor-aware rendering:

                    // Search input with cursor at correct position
                    .child({
                        let before_cursor = &self.filter_text[..self.filter_cursor];
                        let after_cursor = &self.filter_text[self.filter_cursor..];
                        let has_selection = self.filter_selection.is_some();
                        
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_lg()
                            .text_color(if filter_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(filter_is_empty, |d| {
                                // Empty: cursor + placeholder
                                d.child(
                                    div()
                                        .w(px(CURSOR_WIDTH))
                                        .h(px(CURSOR_HEIGHT_LG))
                                        .my(px(CURSOR_MARGIN_Y))
                                        .mr(px(CURSOR_GAP_X))
                                        .when(
                                            self.focused_input == FocusedInput::MainFilter
                                                && self.cursor_visible,
                                            |d| d.bg(rgb(text_primary)),
                                        ),
                                )
                                .child(
                                    div()
                                        .ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))
                                        .child(SharedString::from(DEFAULT_PLACEHOLDER)),
                                )
                            })
                            .when(!filter_is_empty, |d| {
                                // Has text: show text with cursor at position
                                d.child(SharedString::from(before_cursor.to_string()))
                                    .child(
                                        div()
                                            .w(px(CURSOR_WIDTH))
                                            .h(px(CURSOR_HEIGHT_LG))
                                            .my(px(CURSOR_MARGIN_Y))
                                            .mx(px(1.))
                                            .when(
                                                self.focused_input == FocusedInput::MainFilter
                                                    && self.cursor_visible,
                                                |d| d.bg(rgb(text_primary)),
                                            ),
                                    )
                                    .child(SharedString::from(after_cursor.to_string()))
                            })
                    })
```

### Testing

1. **Test backspace clearing**: 
   - Type several characters in the search
   - Press backspace repeatedly until empty
   - Verify the script list shows all items again (not filtered)

2. **Test paste (Cmd+V)**:
   - Copy some text to clipboard
   - Focus the search input
   - Press Cmd+V
   - Verify text appears and filtering works

3. **Test select all (Cmd+A)**:
   - Type some text in search
   - Press Cmd+A
   - Verify all text is selected (when cursor display is implemented)
   - Press any character to replace selection

4. **Test copy (Cmd+C)**:
   - Type text in search, select with Cmd+A
   - Press Cmd+C
   - Paste elsewhere to verify it worked

5. **Test cursor movement**:
   - Type "hello world"
   - Press Left arrow 5 times
   - Type "X" - should insert at cursor position
   - Verify "hello Xworld" appears

6. **Test shift-selection**:
   - Type text
   - Hold Shift + press Left arrow
   - Verify text becomes selected
   - Type to replace selection

---

## Instructions For The Next AI Agent

You are reading the "Main Input Search Issues Expert Bundle". This file is self-contained and includes all the context you should assume you have.

Your job:

* Design and describe the minimal, safe changes needed to fully resolve the issues described in the Executive Summary and Key Problems.
* Operate **only** on the files and code snippets included in this bundle. If you need additional files or context, clearly say so.

When you propose changes, follow these rules strictly:

1. Always provide **precise code snippets** that can be copy-pasted directly into the repo.
2. Always include **exact file paths** (e.g. `src/main.rs`) and, when possible, line numbers or a clear description of the location (e.g. "replace the existing `update_filter` function").
3. Never describe code changes only in prose. Show the full function or block as it should look **after** the change, or show both "before" and "after" versions.
4. Keep instructions **unmistakable and unambiguous**. A human or tool following your instructions should not need to guess what to do.
5. Assume you cannot see any files outside this bundle. If you must rely on unknown code, explicitly note assumptions and risks.

When you answer, you do not need to restate this bundle. Work directly with the code and instructions it contains and return a clear, step-by-step plan plus exact code edits.

---
