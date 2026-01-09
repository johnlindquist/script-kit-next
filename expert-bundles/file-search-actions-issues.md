# Expert Bundle: File Search - Actions Not Working

## Issue Summary

Users report that when selecting actions from the actions dialog (opened via Cmd+K) in File Search view, nothing happens. Actions like "Copy Path", "Reveal in Finder", "Quick Look" etc. don't execute.

## Symptoms

1. User opens File Search, navigates to a file
2. Presses Cmd+K to open actions dialog
3. Actions dialog appears with list of actions
4. User selects an action (e.g., "Copy Path") and presses Enter
5. Action dialog closes but the action doesn't execute
6. No path copied to clipboard, no Finder window opens, etc.

---

## Architecture Overview

### Actions Flow

```
User Presses Cmd+K on File
         │
         ▼
toggle_file_search_actions() [src/render_builtins.rs:8-102]
    │
    ├─ Sets: self.file_search_actions_path = Some(file.path.clone())  ← CRITICAL!
    ├─ Creates: ActionsDialog with get_file_context_actions(file_info)
    └─ Opens: Actions popup window
         │
         ▼
User Selects Action + Presses Enter
         │
         ▼
Actions Interceptor [src/app_impl.rs:654-786]
    │
    ├─ Gets selected action_id from dialog
    └─ Calls: this.handle_action(action_id, cx)
         │
         ▼
handle_action() [src/app_actions.rs:46-463]
    │
    ├─ Match action_id:
    │   ├─ "open_file"|"open_directory" → check file_search_actions_path
    │   ├─ "quick_look" → check file_search_actions_path
    │   ├─ "copy_path" → First checks get_selected_result() (script list)
    │   │                THEN falls through to file_search_actions_path check
    │   └─ etc.
    │
    └─ If file_search_actions_path is None → ACTION FAILS SILENTLY!
```

### The Critical Variable

```rust
// In ScriptListApp:
pub file_search_actions_path: Option<String>,  // Must be Some(path) for actions to work!
```

This variable MUST be set when the actions dialog is opened, and MUST still be Some when the action executes.

---

## Complete Source Code

### File: src/render_builtins.rs - toggle_file_search_actions() (Lines 6-102)

```rust
impl ScriptListApp {
    /// Toggle the actions dialog for file search results
    /// Opens a popup with file-specific actions: Open, Show in Finder, Quick Look, etc.
    fn toggle_file_search_actions(
        &mut self,
        file: &file_search::FileResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log("KEY", "Toggling file search actions popup");

        if self.show_actions_popup || is_actions_window_open() {
            // Close the actions popup
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.file_search_actions_path = None;  // ← Cleared on close

            // Restore focus state - file search uses the main filter input
            self.focused_input = FocusedInput::MainFilter;
            self.gpui_input_focused = true;

            // Close the actions window via spawn
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_actions_window(cx);
                })
                .ok();
            })
            .detach();

            // Refocus the file search input
            self.focus_main_filter(window, cx);
            logging::log(
                "FOCUS",
                "File search actions closed, focus returned to file search input",
            );
        } else {
            // Open actions popup for the selected file
            self.show_actions_popup = true;

            // CRITICAL: Transfer focus from Input to main focus_handle
            // This prevents the Input from receiving text (which would go to file search filter)
            // while keeping keyboard focus in main window for routing to actions dialog
            self.focus_handle.focus(window, cx);
            self.gpui_input_focused = false;
            self.focused_input = FocusedInput::ActionsSearch;

            // *** CRITICAL LINE - THIS MUST BE SET ***
            // Store the file path for action handling
            self.file_search_actions_path = Some(file.path.clone());

            // Create file info from the result
            let file_info = file_search::FileInfo::from_result(file);

            // Create the dialog entity
            let theme_arc = std::sync::Arc::new(self.theme.clone());
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_file(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
                    &file_info,
                    theme_arc,
                )
            });

            // Store the dialog entity for keyboard routing
            self.actions_dialog = Some(dialog.clone());

            // Get main window bounds and display_id for positioning
            let main_bounds = window.bounds();
            let display_id = window.display(cx).map(|d| d.id());

            logging::log(
                "ACTIONS",
                &format!(
                    "Opening file search actions for: {} (is_dir={})",
                    file_info.name, file_info.is_dir
                ),
            );

            // Open the actions window
            cx.spawn(async move |_this, cx| {
                cx.update(
                    |cx| match open_actions_window(cx, main_bounds, display_id, dialog) {
                        Ok(_handle) => {
                            logging::log("ACTIONS", "File search actions popup window opened");
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open actions window: {}", e));
                        }
                    },
                )
                .ok();
            })
            .detach();
        }
        cx.notify();
    }
```

### File: src/app_impl.rs - Actions Interceptor (Lines 654-786)

```rust
        // Add interceptor for actions popup in FileSearchView
        // This handles Cmd+K (toggle), Escape (close), Enter (submit), and typing
        let app_entity_for_actions = cx.entity().downgrade();
        let actions_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_actions;
            move |event, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                let key_char = event.keystroke.key_char.as_deref();

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Only handle when in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            return;
                        }

                        // Handle Cmd+K to toggle actions popup
                        if has_cmd && key == "k" {
                            if let AppView::FileSearchView { selected_index, .. } =
                                &this.current_view
                            {
                                // Get the selected file for toggle
                                let filter_pattern =
                                    if let AppView::FileSearchView { query, .. } =
                                        &this.current_view
                                    {
                                        if let Some(parsed) =
                                            crate::file_search::parse_directory_path(query)
                                        {
                                            parsed.filter
                                        } else if !query.is_empty() {
                                            Some(query.clone())
                                        } else {
                                            None
                                        }
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
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Only handle remaining keys if actions popup is open
                        if !this.show_actions_popup {
                            return;
                        }

                        // Handle Escape to close actions popup
                        if key == "escape" {
                            this.close_actions_popup(ActionsDialogHost::FileSearch, window, cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Enter to submit selected action
                        if key == "enter" {
                            if let Some(ref dialog) = this.actions_dialog {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();

                                if let Some(action_id) = action_id {
                                    crate::logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "FileSearch actions executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );

                                    if should_close {
                                        this.close_actions_popup(
                                            ActionsDialogHost::FileSearch,
                                            window,
                                            cx,
                                        );
                                    }

                                    // Use handle_action instead of trigger_action_by_name
                                    // handle_action supports both built-in actions (open_file, quick_look, etc.)
                                    // and SDK actions
                                    this.handle_action(action_id, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Backspace for actions search
                        if key == "backspace" {
                            if let Some(ref dialog) = this.actions_dialog {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                crate::actions::notify_actions_window(cx);
                                crate::actions::resize_actions_window(cx, dialog);
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle printable character input for actions search
                        if let Some(chars) = key_char {
                            if let Some(ch) = chars.chars().next() {
                                if ch.is_ascii_graphic() || ch == ' ' {
                                    if let Some(ref dialog) = this.actions_dialog {
                                        dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        crate::actions::notify_actions_window(cx);
                                        crate::actions::resize_actions_window(cx, dialog);
                                    }
                                    cx.stop_propagation();
                                }
                            }
                        }
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(actions_interceptor);
```

### File: src/app_actions.rs - handle_action() (Complete File, 559 lines)

```rust
// Actions handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: handle_action, trigger_action_by_name

impl ScriptListApp {
    /// Helper to hide main window and set reset flag
    fn hide_main_and_reset(&self, cx: &mut Context<Self>) {
        set_main_window_visible(false);
        NEEDS_RESET.store(true, Ordering::SeqCst);
        cx.hide();
    }

    /// Helper to reveal a path in Finder (macOS)
    fn reveal_in_finder(&self, path: &std::path::Path) {
        let path_str = path.to_string_lossy().to_string();
        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new("open").arg("-R").arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Revealed in Finder: {}", path_str)),
                Err(e) => logging::log("ERROR", &format!("Failed to reveal in Finder: {}", e)),
            }
        });
    }

    /// Copy text to clipboard using pbcopy on macOS.
    /// Critical: This properly closes stdin before waiting to prevent hangs.
    #[cfg(target_os = "macos")]
    fn pbcopy(&self, text: &str) -> Result<(), std::io::Error> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

        // Take ownership of stdin, write, then drop to signal EOF
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
            // stdin is dropped here => EOF delivered to pbcopy
        }

        // Now it's safe to wait - pbcopy has received EOF
        child.wait()?;
        Ok(())
    }

    /// Handle action selection from the actions dialog
    fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
        logging::log("UI", &format!("Action selected: {}", action_id));

        // Close the dialog and return to script list
        self.current_view = AppView::ScriptList;
        self.pending_focus = Some(FocusTarget::MainFilter);

        match action_id.as_str() {
            "create_script" => {
                logging::log("UI", "Create script action - opening scripts folder");
                let scripts_dir = shellexpand::tilde("~/.scriptkit/scripts").to_string();
                std::thread::spawn(move || {
                    use std::process::Command;
                    match Command::new("open").arg(&scripts_dir).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened scripts folder: {}", scripts_dir))
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open scripts folder: {}", e))
                        }
                    }
                });
                self.last_output = Some(SharedString::from("Opened scripts folder"));
                self.hide_main_and_reset(cx);
            }
            "run_script" => {
                logging::log("UI", "Run script action");
                self.execute_selected(cx);
            }
            "view_logs" => {
                logging::log("UI", "View logs action");
                self.toggle_logs(cx);
            }
            "reveal_in_finder" => {
                logging::log("UI", "Reveal in Finder action");
                // *** THIS BLOCK HANDLES SCRIPT LIST reveal_in_finder ***
                // It checks get_selected_result() which only works for scripts/apps/etc.
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::App(m) => Some(m.app.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal scriptlets in Finder"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal built-in features"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal windows in Finder"));
                            None
                        }
                        scripts::SearchResult::Fallback(_) => {
                            self.last_output = Some(SharedString::from(
                                "Cannot reveal fallback commands in Finder",
                            ));
                            None
                        }
                    };

                    if let Some(path) = path_opt {
                        self.reveal_in_finder(&path);
                        self.last_output = Some(SharedString::from("Revealed in Finder"));
                        self.hide_main_and_reset(cx);
                    }
                } else {
                    // *** BUG: Falls through to "No item selected" instead of checking
                    // file_search_actions_path! ***
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "copy_path" => {
                logging::log("UI", "Copy path action");
                // *** SAME BUG: Checks get_selected_result() first, which fails for file search ***
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(m) => Some(m.script.path.clone()),
                        scripts::SearchResult::App(m) => Some(m.app.path.clone()),
                        scripts::SearchResult::Agent(m) => Some(m.agent.path.clone()),
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy scriptlet path"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy built-in path"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot copy window path"));
                            None
                        }
                        scripts::SearchResult::Fallback(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy fallback command path"));
                            None
                        }
                    };

                    if let Some(path) = path_opt {
                        let path_str = path.to_string_lossy().to_string();

                        #[cfg(target_os = "macos")]
                        {
                            match self.pbcopy(&path_str) {
                                Ok(_) => {
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_str),
                                    );
                                    self.last_output =
                                        Some(SharedString::from(format!("Copied: {}", path_str)));
                                }
                                Err(e) => {
                                    logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }

                        #[cfg(not(target_os = "macos"))]
                        {
                            use arboard::Clipboard;
                            match Clipboard::new().and_then(|mut c| c.set_text(&path_str)) {
                                Ok(_) => {
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_str),
                                    );
                                    self.last_output =
                                        Some(SharedString::from(format!("Copied: {}", path_str)));
                                }
                                Err(e) => {
                                    logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }
                    }
                } else {
                    // *** BUG: Falls through to "No item selected" ***
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            // ... other script list actions ...
            
            "__cancel__" => {
                logging::log("UI", "Actions dialog cancelled");
                // Clear file search actions path on cancel
                self.file_search_actions_path = None;
            }
            
            // *** FILE SEARCH SPECIFIC ACTIONS ***
            // These correctly check file_search_actions_path
            "open_file" | "open_directory" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Opening file: {}", path));
                    let _ = crate::file_search::open_file(path);
                    self.file_search_actions_path = None;
                    self.close_and_reset_window(cx);
                }
                // *** BUG: If file_search_actions_path is None, silently does nothing! ***
            }
            "quick_look" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Quick Look: {}", path));
                    let _ = crate::file_search::quick_look(path);
                    self.file_search_actions_path = None;
                    // Don't close window for Quick Look - user may want to continue
                }
                // *** BUG: If file_search_actions_path is None, silently does nothing! ***
            }
            "open_with" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Open With: {}", path));
                    let _ = crate::file_search::open_with(path);
                    self.file_search_actions_path = None;
                }
            }
            "show_info" => {
                if let Some(ref path) = self.file_search_actions_path {
                    logging::log("UI", &format!("Show Info: {}", path));
                    let _ = crate::file_search::show_info(path);
                    self.file_search_actions_path = None;
                }
            }
            "copy_filename" => {
                if let Some(ref path) = self.file_search_actions_path {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    logging::log("UI", &format!("Copy filename: {}", filename));
                    #[cfg(target_os = "macos")]
                    {
                        let _ = self.pbcopy(filename);
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        let _ = Clipboard::new().and_then(|mut c| c.set_text(filename));
                    }
                    self.last_output = Some(SharedString::from(format!("Copied: {}", filename)));
                    self.file_search_actions_path = None;
                }
            }
            _ => {
                // *** FALLBACK HANDLER FOR file search reveal_in_finder and copy_path ***
                // This is where file search actions ACTUALLY get handled!
                if let Some(path) = self.file_search_actions_path.clone() {
                    match action_id.as_str() {
                        "reveal_in_finder" => {
                            logging::log(
                                "UI",
                                &format!("Reveal in Finder (file search): {}", path),
                            );
                            self.reveal_in_finder(std::path::Path::new(&path));
                            self.file_search_actions_path = None;
                            cx.notify();
                            return;
                        }
                        "copy_path" => {
                            logging::log("UI", &format!("Copy path (file search): {}", path));
                            #[cfg(target_os = "macos")]
                            {
                                let _ = self.pbcopy(&path);
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                use arboard::Clipboard;
                                let _ = Clipboard::new().and_then(|mut c| c.set_text(&path));
                            }
                            self.last_output =
                                Some(SharedString::from(format!("Copied: {}", path)));
                            self.file_search_actions_path = None;
                            cx.notify();
                            return;
                        }
                        _ => {}
                    }
                }
                // Handle SDK actions using shared helper
                self.trigger_sdk_action_internal(&action_id);
            }
        }

        cx.notify();
    }
}
```

### File: src/actions/builders.rs - get_file_context_actions() (Lines 9-109)

```rust
/// Get actions specific to a file search result
/// Actions: Open (default), Show in Finder, Quick Look, Open With..., Show Info
pub fn get_file_context_actions(file_info: &FileInfo) -> Vec<Action> {
    let mut actions = Vec::new();

    // Primary action - Open file
    if file_info.is_dir {
        actions.push(
            Action::new(
                "open_directory",
                format!("Open \"{}\"", file_info.name),
                Some("Open this folder".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    } else {
        actions.push(
            Action::new(
                "open_file",
                format!("Open \"{}\"", file_info.name),
                Some("Open with default application".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("↵"),
        );
    }

    // Show in Finder (Cmd+Enter)
    actions.push(
        Action::new(
            "reveal_in_finder",
            "Show in Finder",
            Some("Reveal in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘↵"),
    );

    // Quick Look (Cmd+Y) - macOS only
    #[cfg(target_os = "macos")]
    if !file_info.is_dir {
        actions.push(
            Action::new(
                "quick_look",
                "Quick Look",
                Some("Preview with Quick Look".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("⌘Y"),
        );
    }

    // Open With... (Cmd+O) - macOS only
    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "open_with",
            "Open With...",
            Some("Choose application to open with".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘O"),
    );

    // Show Info in Finder (Cmd+I) - macOS only
    #[cfg(target_os = "macos")]
    actions.push(
        Action::new(
            "show_info",
            "Get Info",
            Some("Show file information in Finder".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘I"),
    );

    // Copy Path
    actions.push(
        Action::new(
            "copy_path",
            "Copy Path",
            Some("Copy the full path to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘⇧C"),
    );

    // Copy Filename
    actions.push(
        Action::new(
            "copy_filename",
            "Copy Filename",
            Some("Copy just the filename to clipboard".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘C"),
    );

    actions
}
```

---

## Root Cause Analysis

### Primary Issue: View Reset Before Action Execution

**Location:** `src/app_actions.rs:46-51`

```rust
fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
    logging::log("UI", &format!("Action selected: {}", action_id));

    // *** BUG: This resets the view BEFORE checking file_search_actions_path! ***
    self.current_view = AppView::ScriptList;
    self.pending_focus = Some(FocusTarget::MainFilter);
    // ...
}
```

**Problem:** The view is reset to `ScriptList` at the top of `handle_action()`, before the action is processed. This may interfere with context that the action handler needs.

### Secondary Issue: Dual Path for reveal_in_finder and copy_path

These actions are handled in TWO places:

1. **First:** In the main `match` block (lines 79-191), checking `get_selected_result()` which works for script list items
2. **Second:** In the `_ =>` fallback (lines 423-456), checking `file_search_actions_path`

**The Bug:** For file search, `get_selected_result()` returns `None` (because there's no selected script), so the first handler sets `last_output = "No item selected"` and exits. The fallback is never reached!

**Code Path for `reveal_in_finder` from File Search:**

```rust
"reveal_in_finder" => {
    if let Some(result) = self.get_selected_result() {
        // This is None for file search!
        // ...
    } else {
        self.last_output = Some(SharedString::from("No item selected"));
        // Returns here WITHOUT checking file_search_actions_path!
    }
}
```

**The Fix Needed:** Check `file_search_actions_path` FIRST for actions that support both contexts:

```rust
"reveal_in_finder" => {
    // Check file search context FIRST
    if let Some(ref path) = self.file_search_actions_path {
        logging::log("UI", &format!("Reveal in Finder (file search): {}", path));
        self.reveal_in_finder(std::path::Path::new(path));
        self.file_search_actions_path = None;
        self.close_and_reset_window(cx);
    } else if let Some(result) = self.get_selected_result() {
        // Fall back to script list context
        // ... existing script list code ...
    } else {
        self.last_output = Some(SharedString::from("No item selected"));
    }
}
```

### Tertiary Issue: close_actions_popup May Clear Path

**Location:** `src/app_impl.rs:741-746`

```rust
if should_close {
    this.close_actions_popup(
        ActionsDialogHost::FileSearch,
        window,
        cx,
    );
}

// Then call handle_action
this.handle_action(action_id, cx);
```

If `close_actions_popup()` clears `file_search_actions_path`, the action won't work.

**Check `close_actions_popup()` implementation** to see if it clears the path.

### Issue with file_search_actions_path Being None

Scenarios where `file_search_actions_path` could be None when action executes:

1. **Never set:** `toggle_file_search_actions()` wasn't called (actions opened differently?)
2. **Already cleared:** Action was executed once, cleared the path, user somehow submits again
3. **Race condition:** Async spawn in `toggle_file_search_actions()` hasn't completed when action runs
4. **Cleared by close:** `close_actions_popup()` cleared it before `handle_action()` ran

---

## Debugging Strategies

### 1. Verify file_search_actions_path is Set

Add logging at the critical points:

```rust
// In toggle_file_search_actions, when setting:
self.file_search_actions_path = Some(file.path.clone());
logging::log("ACTIONS", &format!(
    "Set file_search_actions_path to: {:?}",
    self.file_search_actions_path
));

// In handle_action, at the top:
logging::log("ACTIONS", &format!(
    "handle_action called with action_id={}, file_search_actions_path={:?}",
    action_id, self.file_search_actions_path
));
```

### 2. Trace the Full Flow

```bash
# Run with logging
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'ACTIONS|file_search|copy_path|reveal'
```

### 3. Check Order of Operations

The sequence should be:
1. `toggle_file_search_actions()` sets `file_search_actions_path`
2. User presses Enter
3. Actions interceptor gets action_id
4. (Optional) `close_actions_popup()` is called
5. `handle_action()` is called
6. `handle_action()` checks `file_search_actions_path`

Add timestamps to verify order:
```rust
logging::log("ACTIONS", &format!(
    "[{}] toggle_file_search_actions setting path",
    std::time::Instant::now().elapsed().as_millis()
));
```

### 4. Test Each Action Type

Create a test that tries each file search action:

```typescript
// tests/smoke/test-file-search-actions.ts
import '../../scripts/kit-sdk';

console.log("File Search Actions Test");
console.log("========================");
console.log("");
console.log("Manual test steps:");
console.log("1. Type ~/dev/ to list home directory");
console.log("2. Arrow down to select a file");
console.log("3. Press Cmd+K to open actions");
console.log("4. Select 'Copy Path' and press Enter");
console.log("5. Check clipboard - should have file path");
console.log("6. If clipboard is empty, check logs for 'file_search_actions_path'");

await new Promise(r => setTimeout(r, 1000));
process.exit(0);
```

---

## Potential Fixes

### Fix 1: Check file_search_actions_path First for Shared Actions

```rust
// In handle_action(), for reveal_in_finder:
"reveal_in_finder" => {
    // Check file search context FIRST
    if let Some(ref path) = self.file_search_actions_path {
        logging::log("UI", &format!("Reveal in Finder (file search): {}", path));
        self.reveal_in_finder(std::path::Path::new(path));
        self.file_search_actions_path = None;
        self.hide_main_and_reset(cx);
    } else if let Some(result) = self.get_selected_result() {
        // Existing script list handling
        let path_opt = match result { /* ... */ };
        if let Some(path) = path_opt {
            self.reveal_in_finder(&path);
            self.last_output = Some(SharedString::from("Revealed in Finder"));
            self.hide_main_and_reset(cx);
        }
    } else {
        self.last_output = Some(SharedString::from("No item selected"));
    }
}

// Same pattern for copy_path:
"copy_path" => {
    // Check file search context FIRST
    if let Some(path) = self.file_search_actions_path.clone() {
        logging::log("UI", &format!("Copy path (file search): {}", path));
        #[cfg(target_os = "macos")]
        { let _ = self.pbcopy(&path); }
        self.last_output = Some(SharedString::from(format!("Copied: {}", path)));
        self.file_search_actions_path = None;
    } else if let Some(result) = self.get_selected_result() {
        // Existing script list handling
        // ...
    } else {
        self.last_output = Some(SharedString::from("No item selected"));
    }
}
```

### Fix 2: Don't Close Popup Before Action for Non-Closing Actions

```rust
// In actions interceptor:
if key == "enter" {
    if let Some(ref dialog) = this.actions_dialog {
        let action_id = dialog.read(cx).get_selected_action_id();
        let should_close = dialog.read(cx).selected_action_should_close();

        if let Some(action_id) = action_id {
            // Execute action FIRST
            this.handle_action(action_id.clone(), cx);
            
            // THEN close if needed
            if should_close {
                this.close_actions_popup(ActionsDialogHost::FileSearch, window, cx);
            }
        }
    }
}
```

### Fix 3: Don't Reset View at Start of handle_action

```rust
fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
    logging::log("UI", &format!("Action selected: {}", action_id));

    // DON'T reset view here - let specific handlers do it
    // self.current_view = AppView::ScriptList;  // REMOVE THIS
    
    match action_id.as_str() {
        // Actions that should return to script list:
        "run_script" | "view_logs" | "edit_script" => {
            self.current_view = AppView::ScriptList;
            self.pending_focus = Some(FocusTarget::MainFilter);
            // ... rest of handler
        }
        
        // File search actions - don't change view until window closes
        "open_file" | "open_directory" | "reveal_in_finder" | "copy_path" => {
            // View change handled by close_and_reset_window()
            // ...
        }
        // ...
    }
}
```

### Fix 4: Add Error Logging for Silent Failures

```rust
"open_file" | "open_directory" => {
    if let Some(ref path) = self.file_search_actions_path {
        logging::log("UI", &format!("Opening file: {}", path));
        let _ = crate::file_search::open_file(path);
        self.file_search_actions_path = None;
        self.close_and_reset_window(cx);
    } else {
        // *** ADD THIS: Log the failure! ***
        logging::log("ERROR", "open_file action failed: file_search_actions_path is None");
        self.last_output = Some(SharedString::from("Error: No file path stored"));
    }
}
```

---

## Testing Commands

### Build and Test

```bash
# Build
cargo build

# Run with file search
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Filter for actions-related logs
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-file-search.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'ACTIONS|copy_path|reveal|file_search_actions_path|handle_action'
```

### Check Clipboard After Copy Path

```bash
# After running copy_path action, verify clipboard:
pbpaste
```

---

## Action ID Reference

File search actions (from `get_file_context_actions()`):

| Action ID | Description | Shortcut |
|-----------|-------------|----------|
| `open_file` | Open file with default app | ↵ |
| `open_directory` | Open folder | ↵ |
| `reveal_in_finder` | Show in Finder | ⌘↵ |
| `quick_look` | Preview with Quick Look | ⌘Y |
| `open_with` | Choose app to open with | ⌘O |
| `show_info` | Get Info in Finder | ⌘I |
| `copy_path` | Copy full path | ⌘⇧C |
| `copy_filename` | Copy filename only | ⌘C |

---

## State Variables Reference

```rust
// Critical state for file search actions:
pub file_search_actions_path: Option<String>,  // THE CRITICAL VARIABLE
pub show_actions_popup: bool,                  // Is popup currently shown
pub actions_dialog: Option<Entity<ActionsDialog>>,  // The dialog entity
pub current_view: AppView,  // FileSearchView vs ScriptList
```

---

## Summary

The root cause of "actions not working" is likely one of:

1. **`reveal_in_finder` and `copy_path` check script list context first** - they call `get_selected_result()` which returns None for file search, then fall through to "No item selected" without checking `file_search_actions_path`

2. **`file_search_actions_path` is None when action executes** - either never set, cleared by close_actions_popup, or race condition

3. **View reset at top of handle_action()** - may clear or interfere with context

**Recommended Fix Priority:**

1. **HIGH:** Reorder `reveal_in_finder` and `copy_path` handlers to check `file_search_actions_path` FIRST
2. **MEDIUM:** Add error logging for all silent failure paths
3. **LOW:** Consider not resetting view at start of `handle_action()`

The fix is relatively straightforward - just need to check the file search context before the script list context for actions that support both.
