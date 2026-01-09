# Expert Bundle: File Search - Tab Navigation Issues

## Issue Summary

Users report that Tab key navigation in the File Search view feels inconsistent or doesn't work as expected. When pressing Tab on a focused item, sometimes nothing happens, sometimes behavior is unexpected.

## Symptoms

1. Pressing Tab on a file (not a directory) does nothing
2. Tab navigation only works on directories, not files
3. Shift+Tab sometimes doesn't go up a directory level
4. After Tab-navigating into a directory, selection resets unexpectedly
5. Filter text in the input doesn't properly extract when typing after Tab
6. Tab works but the selected_index doesn't reset to 0 after directory change

---

## Architecture Overview

### Tab Navigation Flow

```
User Presses Tab (or Shift+Tab)
         │
         ▼
cx.intercept_keystrokes() in app_impl.rs:361-497
(Tab interceptor fires BEFORE Input component)
         │
         ├─ Shift+Tab: parse_directory_path() → go up to parent directory
         │   └─ gpui_input_state.set_value(parent_path)
         │   └─ handle_filter_input_change() triggers async load
         │
         └─ Tab (no shift): Check if selected item is a directory
             ├─ NOT a directory → Tab does NOTHING (intentional)
             │
             └─ IS a directory → Tab enters that directory
                 └─ gpui_input_state.set_value(new_path + "/")
                 └─ handle_filter_input_change() triggers async load
```

### Key Components

1. **Tab Interceptor** (`src/app_impl.rs:361-497`)
   - Uses `cx.intercept_keystrokes()` to catch Tab BEFORE Input component
   - Only handles Tab in `FileSearchView` or `ScriptList` (Ask AI feature)
   - Tab only navigates INTO directories, not files

2. **Directory Path Parser** (`src/file_search.rs:parse_directory_path()`)
   - Extracts directory and optional filter from query
   - e.g., `~/dev/fin` → `{directory: "~/dev/", filter: Some("fin")}`

3. **Input Change Handler** (`src/main.rs:handle_filter_input_change()`)
   - Detects when directory changes
   - Triggers async directory listing via `list_directory()`
   - Resets `selected_index` to 0 when directory changes

4. **Arrow Key Interceptor** (`src/app_impl.rs:500-652`)
   - Handles Up/Down arrow keys for list navigation
   - Also intercepts arrows when actions popup is open

---

## Complete Source Code

### File: src/app_impl.rs - Tab Interceptor (Lines 357-497)

```rust
        // Add Tab key interceptor for "Ask AI" feature and file search directory navigation
        // This fires BEFORE normal key handling, allowing us to intercept Tab
        // even when the Input component has focus
        let app_entity_for_tab = cx.entity().downgrade();
        let tab_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_tab;
            move |event, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let has_shift = event.keystroke.modifiers.shift;
                // Check for Tab key (no cmd/alt/ctrl modifiers, but shift is allowed)
                if key == "tab"
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Handle Tab in FileSearchView for directory navigation
                            if let AppView::FileSearchView {
                                query,
                                selected_index,
                            } = &mut this.current_view
                            {
                                if has_shift {
                                    // Shift+Tab: Go up one directory level
                                    if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        let current_dir = parsed.directory.trim_end_matches('/');

                                        if let Some(last_slash) = current_dir.rfind('/') {
                                            let parent_path = if last_slash == 0 {
                                                "/".to_string()
                                            } else if current_dir.starts_with('~')
                                                && last_slash == 1
                                            {
                                                "~/".to_string()
                                            } else {
                                                format!("{}/", &current_dir[..last_slash])
                                            };

                                            crate::logging::log(
                                                "KEY",
                                                &format!("Shift+Tab: Going up to: {}", parent_path),
                                            );

                                            // Just update the input - handle_filter_input_change will:
                                            // - Update query
                                            // - Reset selected_index to 0
                                            // - Detect directory change
                                            // - Trigger async directory load
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(parent_path, window, cx);
                                            });

                                            cx.notify();
                                            cx.stop_propagation();
                                        }
                                    }
                                } else {
                                    // Tab: Enter selected directory
                                    // Get filtered results to find selected item
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    let filtered_results: Vec<_> =
                                        if let Some(ref pattern) = filter_pattern {
                                            crate::file_search::filter_results_nucleo_simple(
                                                &this.cached_file_results,
                                                pattern,
                                            )
                                        } else {
                                            this.cached_file_results.iter().enumerate().collect()
                                        };

                                    if let Some((_, file)) = filtered_results.get(*selected_index) {
                                        if file.file_type == crate::file_search::FileType::Directory
                                        {
                                            // Use shorten_path to keep ~ instead of expanding to /Users/...
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            let new_path = format!("{}/", shortened);
                                            crate::logging::log(
                                                "KEY",
                                                &format!("Tab: Entering directory: {}", new_path),
                                            );

                                            // Just update the input - handle_filter_input_change will:
                                            // - Update query
                                            // - Reset selected_index to 0
                                            // - Detect directory change
                                            // - Trigger async directory load
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(new_path, window, cx);
                                            });

                                            cx.notify();
                                            cx.stop_propagation();
                                        }
                                    }
                                }
                                return;
                            }

                            // Handle Tab in ScriptList view for Ask AI feature
                            if matches!(this.current_view, AppView::ScriptList)
                                && !this.filter_text.is_empty()
                                && !this.show_actions_popup
                                && !has_shift
                            {
                                let query = this.filter_text.clone();

                                // Open AI window and submit query
                                if let Err(e) = crate::ai::open_ai_window(cx) {
                                    crate::logging::log(
                                        "ERROR",
                                        &format!("Failed to open AI window: {}", e),
                                    );
                                } else {
                                    crate::ai::set_ai_input(cx, &query, true);
                                }

                                // Clear filter text directly (close_and_reset_window will reset the input)
                                this.filter_text.clear();
                                this.close_and_reset_window(cx);

                                // Stop propagation so Input doesn't handle it
                                cx.stop_propagation();
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(tab_interceptor);
```

### File: src/app_impl.rs - Arrow Key Interceptor (Lines 500-652)

```rust
        // Add arrow key interceptor for builtin views with Input components
        // This fires BEFORE Input component handles arrow keys, allowing list navigation
        let app_entity_for_arrows = cx.entity().downgrade();
        let arrow_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_arrows;
            move |event, _window, cx| {
                let key = event.keystroke.key.to_lowercase();
                // Check for Up/Down arrow keys (no modifiers except shift for selection)
                if (key == "up" || key == "arrowup" || key == "down" || key == "arrowdown")
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Only intercept in views that use Input + list navigation
                            match &mut this.current_view {
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Compute filtered length using same logic as render
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    // Use Nucleo fuzzy matching for consistent filtering with render
                                    let filtered_len = if let Some(ref pattern) = filter_pattern {
                                        crate::file_search::filter_results_nucleo_simple(
                                            &this.cached_file_results,
                                            pattern,
                                        )
                                        .len()
                                    } else {
                                        this.cached_file_results.len()
                                    };

                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    // Stop propagation so Input doesn't handle it
                                    cx.stop_propagation();
                                }
                                // ... other views (ClipboardHistoryView, AppLauncherView, WindowSwitcherView)
                                _ => {
                                    // Don't intercept arrows for other views (let normal handling work)
                                }
                            }
                        });
                    }
                }
            }
        });
        app.gpui_input_subscriptions.push(arrow_interceptor);
```

### File: src/file_search.rs - parse_directory_path() Function

```rust
/// Parsed directory path with optional filter
#[derive(Debug, Clone)]
pub struct ParsedDirectoryPath {
    /// The directory portion (always ends with /)
    pub directory: String,
    /// Optional filter after the directory
    pub filter: Option<String>,
}

/// Parse a directory path with optional filter
/// Examples:
/// - "~/dev/" -> ParsedDirectoryPath { directory: "~/dev/", filter: None }
/// - "~/dev/fin" -> ParsedDirectoryPath { directory: "~/dev/", filter: Some("fin") }
/// - "/usr/local/" -> ParsedDirectoryPath { directory: "/usr/local/", filter: None }
/// - "/usr/local/b" -> ParsedDirectoryPath { directory: "/usr/local/", filter: Some("b") }
/// - "./src/" -> ParsedDirectoryPath { directory: "./src/", filter: None }
/// - "../" -> ParsedDirectoryPath { directory: "../", filter: None }
///
/// Returns None if the input doesn't look like a directory path
pub fn parse_directory_path(input: &str) -> Option<ParsedDirectoryPath> {
    // Must start with ~/, /, ./, or ../
    if !input.starts_with("~/")
        && !input.starts_with('/')
        && !input.starts_with("./")
        && !input.starts_with("../")
    {
        return None;
    }

    // Find the last slash
    if let Some(last_slash_pos) = input.rfind('/') {
        let directory = &input[..=last_slash_pos]; // Include the slash
        let remainder = &input[last_slash_pos + 1..];

        let filter = if remainder.is_empty() {
            None
        } else {
            Some(remainder.to_string())
        };

        Some(ParsedDirectoryPath {
            directory: directory.to_string(),
            filter,
        })
    } else {
        None
    }
}
```

### File: src/file_search.rs - shorten_path() Function

```rust
/// Shorten a path for display by replacing home directory with ~
pub fn shorten_path(path: &str) -> String {
    let home = shellexpand::tilde("~").to_string();
    if path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    }
}
```

### File: src/file_search.rs - filter_results_nucleo_simple() Function

```rust
/// Simple Nucleo-based fuzzy filtering
/// Returns (original_index, &FileResult) pairs sorted by match quality
pub fn filter_results_nucleo_simple<'a>(
    results: &'a [FileResult],
    pattern: &str,
) -> Vec<(usize, &'a FileResult)> {
    use nucleo_matcher::{
        pattern::{AtomKind, CaseMatching, Normalization, Pattern},
        Config, Matcher, Utf32Str,
    };

    if pattern.is_empty() {
        return results.iter().enumerate().collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    let pattern = Pattern::new(
        pattern,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let mut scored: Vec<(usize, &FileResult, u32)> = results
        .iter()
        .enumerate()
        .filter_map(|(idx, file)| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(&file.name, &mut buf);
            pattern.score(haystack, &mut matcher).map(|score| (idx, file, score))
        })
        .collect();

    // Sort by score descending (higher is better match)
    scored.sort_by(|a, b| b.2.cmp(&a.2));

    scored.into_iter().map(|(idx, file, _)| (idx, file)).collect()
}
```

### File: src/main.rs - handle_filter_input_change() (FileSearch portion)

```rust
// In handle_filter_input_change() method:

AppView::FileSearchView { query, selected_index } => {
    let new_text = self.gpui_input_state.read(cx).text().to_string();
    
    // Detect if directory changed (for resetting selection)
    let old_dir = parse_directory_path(query).map(|p| p.directory);
    let new_dir = parse_directory_path(&new_text).map(|p| p.directory);
    let directory_changed = old_dir != new_dir;
    
    // Update the query
    *query = new_text.clone();
    
    // Reset selection when directory changes
    if directory_changed {
        *selected_index = 0;
    }
    
    // Determine if we should do directory listing or search
    if is_directory_path(&new_text) {
        // Directory path mode - list directory contents
        if let Some(parsed) = parse_directory_path(&new_text) {
            // Only trigger load if directory actually changed
            if directory_changed {
                self.file_search_loading = true;
                self.trigger_directory_list(&parsed.directory, cx);
            }
            // If just filter changed, cached_file_results stays same, filtered in render
        }
    } else if !new_text.is_empty() {
        // Search mode - debounce and search via mdfind
        self.trigger_debounced_search(&new_text, cx);
    } else {
        // Empty - clear results
        self.cached_file_results.clear();
        self.file_search_loading = false;
    }
    
    cx.notify();
}
```

### File: src/render_builtins.rs - Key Handler in render_file_search() (Lines 2033-2198)

```rust
        // Key handler for file search
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    ActionsDialogHost::FileSearch,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {
                        // Actions dialog not open - continue to file search key handling
                    }
                    ActionsRoute::Handled => {
                        // Key was consumed by actions dialog
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        // User selected an action - execute it
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC goes back to main menu (not close window)
                if key_str == "escape" {
                    logging::log("KEY", "ESC in FileSearch - returning to main menu");
                    this.file_search_debounce_task = None;
                    this.file_search_loading = false;
                    this.cached_file_results.clear();
                    this.current_view = AppView::ScriptList;
                    this.filter_text.clear();
                    this.selected_index = 0;
                    this.gpui_input_state.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                        state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
                    });
                    this.update_window_size();
                    cx.notify();
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    let filter_pattern =
                        if let Some(parsed) = crate::file_search::parse_directory_path(query) {
                            parsed.filter
                        } else if !query.is_empty() {
                            Some(query.clone())
                        } else {
                            None
                        };

                    let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
                        crate::file_search::filter_results_nucleo_simple(
                            &this.cached_file_results,
                            pattern,
                        )
                    } else {
                        this.cached_file_results.iter().enumerate().collect()
                    };
                    let filtered_len = filtered_results.len();

                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                                this.file_search_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if *selected_index + 1 < filtered_len {
                                *selected_index += 1;
                                this.file_search_scroll_handle
                                    .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                                cx.notify();
                            }
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        "enter" => {
                            if has_cmd {
                                // Cmd+Enter: reveal in finder
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Enter: open file with default app
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_file(&file.path);
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
                }
            },
        );
```

---

## Root Cause Analysis

### Issue 1: Tab Does Nothing on Files

**Expected:** User expects Tab to do something (maybe autocomplete? open file?)
**Actual:** Tab only works on directories (by design)

**Root Cause Location:** `src/app_impl.rs:440-462`

```rust
if let Some((_, file)) = filtered_results.get(*selected_index) {
    if file.file_type == crate::file_search::FileType::Directory
    {
        // Only enter directories
        // ...
    }
    // ELSE: Nothing happens for files!
}
```

**Analysis:** This is intentional behavior - Tab navigates INTO directories. But users may expect Tab to autocomplete or open files. The UX is unclear.

**Potential Fix:**
1. Add visual hint that Tab only works on directories (e.g., show "Tab" hint only on directory items)
2. Or make Tab open files (like Enter), with Shift+Tab going up
3. Or show a tooltip/toast "Tab to enter directory" when on a directory

### Issue 2: Shift+Tab Doesn't Go Up a Level

**Symptoms:** Shift+Tab sometimes fails to go up a directory level

**Root Cause Location:** `src/app_impl.rs:380-415`

```rust
if has_shift {
    // Shift+Tab: Go up one directory level
    if let Some(parsed) = crate::file_search::parse_directory_path(query)
    {
        // Only works if query IS a directory path!
        // ...
    }
    // ELSE: Nothing happens if query doesn't parse as directory path
}
```

**Conditions for Failure:**
1. Query doesn't start with `~/`, `/`, `./`, or `../` → `parse_directory_path()` returns None
2. Query is just a search term like "readme" → Not a path, Shift+Tab does nothing
3. At root level (`/`) with no parent to go to

**Debug Logging:** Check for `[KEY] Shift+Tab: Going up to:` in logs. If not present, parsing failed.

### Issue 3: Selection Not Resetting After Tab Navigation

**Symptoms:** After pressing Tab to enter a directory, the selection doesn't reset to index 0

**Root Cause Location:** The flow is:
1. Tab interceptor calls `gpui_input_state.set_value(new_path)`
2. This triggers `handle_filter_input_change()`
3. `handle_filter_input_change()` should reset `selected_index` to 0

**Potential Bug:** If `handle_filter_input_change()` doesn't detect directory change correctly:

```rust
// In handle_filter_input_change:
let old_dir = parse_directory_path(query).map(|p| p.directory);
let new_dir = parse_directory_path(&new_text).map(|p| p.directory);
let directory_changed = old_dir != new_dir;

if directory_changed {
    *selected_index = 0;  // This might not be executing
}
```

**Debug:** Add logging in `handle_filter_input_change()` to verify:
```rust
logging::log("NAV", &format!(
    "Directory change detection: old={:?} new={:?} changed={}",
    old_dir, new_dir, directory_changed
));
```

### Issue 4: Filter Text Extraction Issues

**Symptoms:** After typing `~/dev/fin`, the filter "fin" doesn't work correctly

**Root Cause Location:** `src/file_search.rs:parse_directory_path()`

The filter extraction depends on the last `/` position:
```rust
if let Some(last_slash_pos) = input.rfind('/') {
    let directory = &input[..=last_slash_pos]; // Include the slash
    let remainder = &input[last_slash_pos + 1..];

    let filter = if remainder.is_empty() {
        None
    } else {
        Some(remainder.to_string())
    };
    // ...
}
```

**Edge Cases:**
- `~/dev/` → directory="~/dev/", filter=None (correct)
- `~/dev/fin` → directory="~/dev/", filter=Some("fin") (correct)
- `~/dev/sub/` → directory="~/dev/sub/", filter=None (correct)
- `~/dev/sub/f` → directory="~/dev/sub/", filter=Some("f") (correct)

**Potential Issue:** If `cached_file_results` hasn't loaded yet, filtering against empty list returns nothing.

---

## Debugging Strategies

### 1. Add Comprehensive Logging

```rust
// In Tab interceptor (app_impl.rs:416-464):
if key == "tab" && !has_shift {
    logging::log("KEY", &format!(
        "Tab pressed: query={:?} selected_index={} file_count={}",
        query, selected_index, this.cached_file_results.len()
    ));
    
    if let Some((idx, file)) = filtered_results.get(*selected_index) {
        logging::log("KEY", &format!(
            "Selected item: name={} type={:?} is_dir={}",
            file.name, file.file_type, file.file_type == FileType::Directory
        ));
    } else {
        logging::log("KEY", "No item at selected_index!");
    }
}
```

### 2. Verify Directory Detection

```rust
// In handle_filter_input_change:
logging::log("NAV", &format!(
    "FileSearch input change: '{}' -> '{}' | is_dir_path={} | dir_changed={}",
    old_query, new_text,
    is_directory_path(&new_text),
    directory_changed
));
```

### 3. Test Tab Navigation Flow

```bash
# Run with logging enabled
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search-nav.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'KEY|NAV|Tab|directory'
```

### 4. Verify Interceptor Order

The Tab interceptor MUST fire before the Input component handles Tab (which would insert a tab character or move focus). Verify with:

```rust
// At start of tab_interceptor closure:
logging::log("INTERCEPT", "Tab interceptor fired");
```

If this doesn't appear when pressing Tab, the interceptor isn't registered correctly.

---

## Potential Fixes

### Fix 1: Visual Feedback for Tab-able Items

Only show Tab hint on directories:

```rust
// In list item rendering:
.when(file.file_type == FileType::Directory, |el| {
    el.child(
        div()
            .text_xs()
            .text_color(rgb(text_dimmed))
            .child("Tab →")
    )
})
```

### Fix 2: Make Tab Work on Files (Open Them)

```rust
// In Tab interceptor, after directory check:
if file.file_type == crate::file_search::FileType::Directory {
    // Enter directory
    // ... existing code ...
} else {
    // Tab on file = open it
    let _ = crate::file_search::open_file(&file.path);
    this.close_and_reset_window(cx);
}
cx.notify();
cx.stop_propagation();
```

### Fix 3: Ensure Selection Reset on Directory Change

```rust
// In handle_filter_input_change, be more explicit:
if directory_changed {
    *selected_index = 0;
    this.file_search_scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
    logging::log("NAV", "Selection reset to 0 due to directory change");
}
```

### Fix 4: Handle Edge Case - Shift+Tab at Root

```rust
// In Shift+Tab handler:
if current_dir == "/" || current_dir == "~/" {
    logging::log("KEY", "Already at root, cannot go up");
    // Maybe show a toast?
    cx.stop_propagation();
    return;
}
```

---

## Testing Commands

### Build and Test

```bash
# Build
cargo build

# Run file search test
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Filter for navigation logs
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'KEY|NAV|Tab|directory|selected'
```

### Test Script for Tab Navigation

```typescript
// tests/smoke/test-file-search-tab-nav.ts
import '../../scripts/kit-sdk';

// Trigger file search
await new Promise(r => setTimeout(r, 500));

// Type a directory path
// (Would need to simulate keystrokes to test Tab)

console.log("Tab navigation test - manual testing required");
console.log("Steps:");
console.log("1. Type ~/dev/ in file search");
console.log("2. Arrow down to a directory");
console.log("3. Press Tab - should enter that directory");
console.log("4. Press Shift+Tab - should go back up");
console.log("5. Arrow down to a FILE");
console.log("6. Press Tab - should do nothing (currently) or open file (if fixed)");

process.exit(0);
```

---

## State Variables Reference

These are the key state variables involved in Tab navigation:

```rust
// In ScriptListApp:
pub current_view: AppView,  // AppView::FileSearchView { query, selected_index }
pub cached_file_results: Vec<FileResult>,  // Directory contents or search results
pub file_search_loading: bool,  // True during async load
pub file_search_current_dir: Option<String>,  // Last loaded directory (for caching)
pub file_search_scroll_handle: UniformListScrollHandle,  // For scrolling list
pub gpui_input_state: Entity<InputState>,  // The search input text
```

```rust
// AppView enum:
pub enum AppView {
    FileSearchView {
        query: String,          // Current input text (e.g., "~/dev/fin")
        selected_index: usize,  // Currently highlighted item (0-based)
    },
    // ... other views
}
```

---

## Summary

The Tab navigation system uses keystroke interception to capture Tab before the Input component. Key behaviors:

1. **Tab** = Enter selected directory (only works on directories)
2. **Shift+Tab** = Go up one directory level
3. **Arrow Up/Down** = Move selection (also intercepted)

Common issues stem from:
- Unclear UX (Tab does nothing on files)
- Directory path parsing edge cases
- Selection not resetting on directory change
- Race conditions between input update and async directory load

The fix strategy should focus on:
1. Better visual feedback (show what Tab will do)
2. More robust directory change detection
3. Consistent selection reset behavior
4. Clear logging for debugging
