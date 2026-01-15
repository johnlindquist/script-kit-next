# Command Bar Refactor Expert Bundle

## Original Goal

> Improving the Actions Window/Dialog into a reusable Command Bar that can be used throughout the applications and other windows. We need the design to stay consistent and the features such as focusing on the input, navigating the list of options, submitting an action, and everything to be fully implemented. The component needs to be flexible enough that the search can be at the top or bottom and that the resize can resize from the top or bottom. There will be concerns around focus and toggling open/closed with cmd+k and exposing the keyboard shortcuts to the be active so that even if the menu is closed, the actions can still be triggered.

## Executive Summary

The Script Kit GPUI application already has a well-architected ActionsDialog component in `src/actions/` that provides a Raycast-style searchable action menu. This bundle documents the existing architecture and identifies specific areas for enhancement to create a fully reusable Command Bar component.

### Key Problems

1. **Current ActionsDialog is tightly coupled to specific contexts** - while it supports multiple factory methods (`with_script`, `with_file`, `with_path`, `with_clipboard_entry`, `with_chat`, `with_config`), making it a truly reusable "Command Bar" requires additional abstraction.

2. **Search position and resize anchor configuration exists but may need refinement** - `ActionsDialogConfig` already has `SearchPosition` (Top/Bottom/Hidden), `SectionStyle`, and `AnchorPosition` enums, but the resize behavior in `window.rs` is partially macOS-specific.

3. **Global hotkey shortcuts when menu is closed** - The current architecture routes keyboard shortcuts through the actions dialog when open, but exposing shortcuts globally (when closed) requires coordination between the `hotkeys.rs` module and the actions system.

### Required Fixes

1. **Abstract CommandBar from ActionsDialog** (`src/actions/`)
   - Create a `CommandBar` wrapper type that encapsulates `ActionsDialog` + window management
   - Add `CommandBarConfig` with all positioning/behavior options
   - Implement `CommandBarHost` trait for contexts that embed a command bar

2. **Unified Focus Management** (`src/actions/window.rs`, `src/app_impl.rs`)
   - Ensure focus can be tracked across the parent window and the vibrancy popup
   - Implement focus restoration when command bar closes
   - Handle Cmd+K toggle consistently across all hosts

3. **Global Shortcut Registration** (`src/hotkeys.rs`, `src/actions/builders.rs`)
   - Extract shortcuts from actions and register them globally
   - Route shortcut triggers to action execution even when command bar is closed
   - Add configuration for which shortcuts should be global vs. command-bar-only

### Files Included

- `src/actions/mod.rs`: Public API exports and module structure
- `src/actions/types.rs`: Core types (Action, ActionCategory, ScriptInfo, ActionsDialogConfig)
- `src/actions/constants.rs`: Layout constants (POPUP_WIDTH, ACTION_ITEM_HEIGHT, etc.)
- `src/actions/dialog.rs`: ActionsDialog struct and full rendering implementation
- `src/actions/window.rs`: Floating vibrancy window management
- `src/notes/actions_panel.rs`: Example integration in Notes window
- `src/prompts/path.rs`: PathPrompt with actions toggle (EventEmitter pattern)
- `.opencode/skill/script-kit-actions-window/SKILL.md`: Existing skill documentation

---

## Architecture Overview

### Current Module Structure

```
src/actions/
├── mod.rs          # Public API re-exports
├── types.rs        # Action, ActionCategory, ScriptInfo, ActionsDialogConfig
├── builders.rs     # Factory functions: get_script_context_actions(), etc.
├── constants.rs    # Layout constants: POPUP_WIDTH, ACTION_ITEM_HEIGHT, etc.
├── dialog.rs       # ActionsDialog struct + rendering logic
└── window.rs       # Separate vibrancy window management
```

### Configuration System

```rust
/// Position of search input
pub enum SearchPosition {
    Top,      // AI chat style - list grows downward
    Bottom,   // Main menu style - list grows upward (default)
    Hidden,   // External search handling
}

/// Section/category display style
pub enum SectionStyle {
    Headers,    // Text headers for sections (AI chat style)
    Separators, // Subtle lines between categories (default)
    None,       // No section indicators
}

/// Dialog anchor during resize
pub enum AnchorPosition {
    Top,    // Content pinned to top, grows down
    Bottom, // Content pinned to bottom, grows up (default)
}

/// Complete configuration
pub struct ActionsDialogConfig {
    pub search_position: SearchPosition,
    pub section_style: SectionStyle,
    pub anchor: AnchorPosition,
    pub show_icons: bool,
    pub show_footer: bool,
}
```

### Usage Patterns

**Basic (no context):**
```rust
let dialog = ActionsDialog::new(focus_handle, callback, theme);
```

**With script context:**
```rust
let script = ScriptInfo::new("my-script", "/path/to/script.ts");
let dialog = ActionsDialog::with_script(focus_handle, callback, Some(script), theme);
```

**With custom config (AI style):**
```rust
let config = ActionsDialogConfig {
    search_position: SearchPosition::Top,
    section_style: SectionStyle::Headers,
    anchor: AnchorPosition::Top,
    show_icons: true,
    show_footer: true,
};
let dialog = ActionsDialog::with_config(focus_handle, callback, actions, theme, config);
```

### Window Management

**Opening:**
```rust
use crate::actions::{open_actions_window, close_actions_window, is_actions_window_open};

let dialog_entity = cx.new(|cx| ActionsDialog::with_script(...));
let main_bounds = window.bounds();
let display_id = window.display().map(|d| d.id());

open_actions_window(cx, main_bounds, display_id, dialog_entity)?;
```

**Resizing after filter changes:**
```rust
resize_actions_window(cx, &dialog_entity);
// Window stays "pinned to bottom" - bottom edge fixed, top moves
```

**Toggle pattern (Cmd+K):**
```rust
fn toggle_actions(&mut self, cx: &mut Context<Self>) {
    let is_showing = is_actions_window_open();
    if is_showing {
        close_actions_window(cx);
    } else {
        self.show_actions(cx);
    }
}
```

---

## AI Window Integration Example

The AI window (`src/ai/window.rs`) shows how to integrate ActionsDialog as a command bar:

```rust
// Using the unified ActionsDialog component for AI command bar (Cmd+K)
use crate::actions::{
    close_actions_window, get_ai_command_bar_actions, notify_actions_window, 
    open_actions_window, ActionsDialog, ActionsDialogConfig, AnchorPosition, 
    SearchPosition, SectionStyle,
};

fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    let theme = Arc::new(crate::theme::load_theme());
    let actions = get_ai_command_bar_actions();

    // Configure for AI-style:
    // - Search at top (like Raycast Cmd+K)
    // - Section headers (not separators)
    // - Icons shown
    let config = ActionsDialogConfig {
        search_position: SearchPosition::Top,
        section_style: SectionStyle::Headers,
        anchor: AnchorPosition::Top,
        show_icons: true,
        show_footer: true,
    };

    let on_select: Arc<dyn Fn(String) + Send + Sync> = Arc::new(|_| {});
    let dialog = cx.new(|cx| {
        ActionsDialog::with_config(cx.focus_handle(), on_select, actions, theme, config)
    });

    let bounds = window.bounds();
    let display_id = window.display(cx).map(|d| d.id());

    self.command_bar_dialog = Some(dialog.clone());
    self.showing_command_bar = true;
    self.focus_handle.focus(window, cx);

    open_actions_window(cx, bounds, display_id, dialog);
}
```

---

## EventEmitter Pattern for Toggle

The `PathPrompt` shows the recommended pattern for toggle behavior using GPUI's EventEmitter:

```rust
/// Events emitted by PathPrompt for parent handling
#[derive(Debug, Clone)]
pub enum PathPromptEvent {
    ShowActions(PathInfo),
    CloseActions,
}

impl EventEmitter<PathPromptEvent> for PathPrompt {}

impl PathPrompt {
    /// Toggle actions dialog - show if hidden, close if showing
    pub fn toggle_actions(&mut self, cx: &mut Context<Self>) {
        let is_showing = self.actions_showing.lock().map(|g| *g).unwrap_or(false);
        if is_showing {
            cx.emit(PathPromptEvent::CloseActions);
        } else {
            if let Some(entry) = self.filtered_entries.get(self.selected_index) {
                let path_info = PathInfo::new(entry.name.clone(), entry.path.clone(), entry.is_dir);
                cx.emit(PathPromptEvent::ShowActions(path_info));
            }
        }
    }
}

// Parent subscribes to events:
cx.subscribe(&path_prompt_entity, |this, _prompt, event, cx| {
    match event {
        PathPromptEvent::ShowActions(info) => this.open_actions_for_path(info, cx),
        PathPromptEvent::CloseActions => this.close_actions(cx),
    }
}).detach();
```

---

## Packx Output

The following section contains the full source code from the relevant files.

This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 8
</notes>
</file_summary>

<directory_structure>
src/prompts/path.rs
src/actions/mod.rs
src/actions/window.rs
src/actions/dialog.rs
src/actions/constants.rs
src/actions/types.rs
src/notes/actions_panel.rs
.opencode/skill/script-kit-actions-window/SKILL.md
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/prompts/path.rs">
//! PathPrompt - File/folder picker prompt
//!
//! Features:
//! - Browse file system starting from optional path
//! - Filter files/folders by name
//! - Navigate with keyboard
//! - Submit selected path
//!
//! Uses GPUI EventEmitter pattern for actions dialog communication:
//! - Parent subscribes to PathPromptEvent::ShowActions / CloseActions
//! - No mutex polling in render - events trigger immediate handling

use gpui::{
    div, prelude::*, uniform_list, Context, EventEmitter, FocusHandle, Focusable, Render,
    UniformListScrollHandle, Window,
};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::components::{
    PromptContainer, PromptContainerColors, PromptContainerConfig, PromptHeader,
    PromptHeaderColors, PromptHeaderConfig,
};
use crate::designs::DesignVariant;
use crate::list_item::{IconKind, ListItem, ListItemColors};
use crate::logging;
use crate::theme;

/// Callback for prompt submission
/// Signature: (id: String, value: Option<String>)
pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;

/// Events emitted by PathPrompt for parent handling
/// Uses GPUI's EventEmitter pattern instead of mutex polling
#[derive(Debug, Clone)]
pub enum PathPromptEvent {
    /// Request to show actions dialog for the given path
    ShowActions(PathInfo),
    /// Request to close actions dialog
    CloseActions,
}

/// Information about a file/folder path for context-aware actions
/// Used for path-specific actions in the actions dialog
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// Display name of the file/folder
    pub name: String,
    /// Full path to the file/folder
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
}

impl PathInfo {
    pub fn new(name: impl Into<String>, path: impl Into<String>, is_dir: bool) -> Self {
        PathInfo {
            name: name.into(),
            path: path.into(),
            is_dir,
        }
    }
}

/// Callback for showing actions dialog
/// Signature: (path_info: PathInfo)
pub type ShowActionsCallback = Arc<dyn Fn(PathInfo) + Send + Sync>;

/// Callback for closing actions dialog (toggle behavior)
/// Signature: ()
pub type CloseActionsCallback = Arc<dyn Fn() + Send + Sync>;

/// PathPrompt - File/folder picker
///
/// Provides a file browser interface for selecting files or directories.
/// Supports starting from a specified path and filtering by name.
pub struct PathPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Starting directory path (defaults to home if None)
    pub start_path: Option<String>,
    /// Hint text to display
    pub hint: Option<String>,
    /// Current directory being browsed
    pub current_path: String,
    /// Filter text for narrowing down results
    pub filter_text: String,
    /// Currently selected index in the list
    pub selected_index: usize,
    /// List of entries in current directory
    pub entries: Vec<PathEntry>,
    /// Filtered entries based on filter_text
    pub filtered_entries: Vec<PathEntry>,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits a selection
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Scroll handle for the list
    pub list_scroll_handle: UniformListScrollHandle,
    /// Optional callback to show actions dialog
    pub on_show_actions: Option<ShowActionsCallback>,
    /// Optional callback to close actions dialog (for toggle behavior)
    pub on_close_actions: Option<CloseActionsCallback>,
    /// Shared state tracking if actions dialog is currently showing
    /// Used by PathPrompt to implement toggle behavior for Cmd+K
    pub actions_showing: Arc<Mutex<bool>>,
    /// Shared state for actions search text (displayed in header when actions showing)
    pub actions_search_text: Arc<Mutex<String>>,
    /// Whether to show blinking cursor (for focused state)
    pub cursor_visible: bool,
}

/// A file system entry (file or directory)
#[derive(Clone, Debug)]
pub struct PathEntry {
    /// Display name
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
}

impl PathPrompt {
    pub fn new(
        id: String,
        start_path: Option<String>,
        hint: Option<String>,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let current_path = start_path.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string())
        });

        logging::log(
            "PROMPTS",
            &format!("PathPrompt::new starting at: {}", current_path),
        );

        // Load entries from current path
        let entries = Self::load_entries(&current_path);
        let filtered_entries = entries.clone();

        PathPrompt {
            id,
            start_path,
            hint,
            current_path,
            filter_text: String::new(),
            selected_index: 0,
            entries,
            filtered_entries,
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            list_scroll_handle: UniformListScrollHandle::new(),
            on_show_actions: None,
            on_close_actions: None,
            actions_showing: Arc::new(Mutex::new(false)),
            actions_search_text: Arc::new(Mutex::new(String::new())),
            cursor_visible: true,
        }
    }

    /// Set the callback for showing actions dialog
    pub fn with_show_actions(mut self, callback: ShowActionsCallback) -> Self {
        self.on_show_actions = Some(callback);
        self
    }

    /// Set the show actions callback (mutable version)
    pub fn set_show_actions(&mut self, callback: ShowActionsCallback) {
        self.on_show_actions = Some(callback);
    }

    /// Set the close actions callback (for toggle behavior)
    pub fn with_close_actions(mut self, callback: CloseActionsCallback) -> Self {
        self.on_close_actions = Some(callback);
        self
    }

    /// Set the shared actions_showing state (for toggle behavior)
    pub fn with_actions_showing(mut self, actions_showing: Arc<Mutex<bool>>) -> Self {
        self.actions_showing = actions_showing;
        self
    }

    /// Set the shared actions_search_text state (for header display)
    pub fn with_actions_search_text(mut self, actions_search_text: Arc<Mutex<String>>) -> Self {
        self.actions_search_text = actions_search_text;
        self
    }

    /// Load directory entries from a path
    fn load_entries(dir_path: &str) -> Vec<PathEntry> {
        let path = Path::new(dir_path);
        let mut entries = Vec::new();

        // No ".." entry - use left arrow to navigate to parent

        // Read directory entries
        if let Ok(read_dir) = std::fs::read_dir(path) {
            let mut dirs: Vec<PathEntry> = Vec::new();
            let mut files: Vec<PathEntry> = Vec::new();

            for entry in read_dir.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files (starting with .)
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = entry_path.is_dir();
                let path_entry = PathEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir,
                };

                if is_dir {
                    dirs.push(path_entry);
                } else {
                    files.push(path_entry);
                }
            }

            // Sort alphabetically (case insensitive)
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Add dirs first, then files
            entries.extend(dirs);
            entries.extend(files);
        }

        logging::log(
            "PROMPTS",
            &format!(
                "PathPrompt loaded {} entries from {}",
                entries.len(),
                dir_path
            ),
        );
        entries
    }

    /// Update filtered entries based on filter text
    fn update_filtered(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_entries = self.entries.clone();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_entries = self
                .entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect();
        }

        // Reset selection to 0 if out of bounds
        if self.selected_index >= self.filtered_entries.len() {
            self.selected_index = 0;
        }
    }

    /// Set the current filter text programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.filter_text == text {
            return;
        }

        self.filter_text = text;
        self.update_filtered();
        self.selected_index = 0;
        self.list_scroll_handle
            .scroll_to_item(0, gpui::ScrollStrategy::Top);
        cx.notify();
    }

    /// Navigate into a directory
    pub fn navigate_to(&mut self, path: &str, cx: &mut Context<Self>) {
        self.current_path = path.to_string();
        self.entries = Self::load_entries(path);
        self.filter_text.clear();
        self.filtered_entries = self.entries.clone();
        self.selected_index = 0;
        cx.notify();
    }

    /// Show actions dialog for the selected entry
    /// Emits PathPromptEvent::ShowActions for parent to handle
    fn show_actions(&mut self, cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            let path_info = PathInfo::new(entry.name.clone(), entry.path.clone(), entry.is_dir);
            logging::log(
                "PROMPTS",
                &format!(
                    "PathPrompt emitting ShowActions for: {} (is_dir={})",
                    path_info.path, path_info.is_dir
                ),
            );
            // Emit event for parent to handle (GPUI pattern)
            cx.emit(PathPromptEvent::ShowActions(path_info.clone()));
            // Also call legacy callback if present (backwards compatibility)
            if let Some(ref callback) = self.on_show_actions {
                (callback)(path_info);
            }
            cx.notify();
        }
    }

    /// Close actions dialog (for toggle behavior)
    /// Emits PathPromptEvent::CloseActions for parent to handle
    fn close_actions(&mut self, cx: &mut Context<Self>) {
        logging::log("PROMPTS", "PathPrompt emitting CloseActions");
        // Emit event for parent to handle (GPUI pattern)
        cx.emit(PathPromptEvent::CloseActions);
        // Also call legacy callback if present (backwards compatibility)
        if let Some(ref callback) = self.on_close_actions {
            (callback)();
        }
        cx.notify();
    }

    /// Toggle actions dialog - show if hidden, close if showing
    pub fn toggle_actions(&mut self, cx: &mut Context<Self>) {
        let is_showing = self.actions_showing.lock().map(|g| *g).unwrap_or(false);

        if is_showing {
            logging::log(
                "PROMPTS",
                "PathPrompt toggle: closing actions (was showing)",
            );
            self.close_actions(cx);
        } else {
            logging::log("PROMPTS", "PathPrompt toggle: showing actions (was hidden)");
            self.show_actions(cx);
        }
    }

    /// Submit the selected path - always submits, never navigates
    /// For files and directories: submit the path (script will handle it)
    /// Navigation into directories is handled by → and Tab keys
    fn submit_selected(&mut self, _cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            // Always submit the path, whether it's a file or directory
            // The calling script or default handler will decide what to do with it
            logging::log(
                "PROMPTS",
                &format!(
                    "PathPrompt submitting path: {} (is_dir={})",
                    entry.path, entry.is_dir
                ),
            );
            (self.on_submit)(self.id.clone(), Some(entry.path.clone()));
        } else if !self.filter_text.is_empty() {
            // If no entry selected but filter has text, submit the filter as a path
            logging::log(
                "PROMPTS",
                &format!(
                    "PathPrompt submitting filter text as path: {}",
                    self.filter_text
                ),
            );
            (self.on_submit)(self.id.clone(), Some(self.filter_text.clone()));
        }
    }

    /// Handle Enter key - always submit the selected path
    /// The calling code (main.rs) will open it with system default via std::process::Command
    pub fn handle_enter(&mut self, cx: &mut Context<Self>) {
        // Always submit directly - no actions dialog on Enter
        // Actions are available via Cmd+K
        self.submit_selected(cx);
    }

    /// Cancel - submit None
    pub fn submit_cancel(&mut self) {
        logging::log(
            "PROMPTS",
            &format!(
                "PathPrompt submit_cancel called - submitting None for id: {}",
                self.id
            ),
        );
        (self.on_submit)(self.id.clone(), None);
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_scroll_handle
                .scroll_to_item(self.selected_index, gpui::ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_entries.len().saturating_sub(1) {
            self.selected_index += 1;
            self.list_scroll_handle
                .scroll_to_item(self.selected_index, gpui::ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Handle character input
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.filter_text.push(ch);
        self.update_filtered();
        cx.notify();
    }

    /// Handle backspace - if filter empty, go up one directory
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.filter_text.is_empty() {
            self.filter_text.pop();
            self.update_filtered();
            cx.notify();
        } else {
            // If filter is empty, navigate up one directory
            let path = Path::new(&self.current_path);
            if let Some(parent) = path.parent() {
                let parent_path = parent.to_string_lossy().to_string();
                self.navigate_to(&parent_path, cx);
            }
        }
    }

    /// Navigate to parent directory (left arrow / shift+tab)
    pub fn navigate_to_parent(&mut self, cx: &mut Context<Self>) {
        let path = Path::new(&self.current_path);
        if let Some(parent) = path.parent() {
            let parent_path = parent.to_string_lossy().to_string();
            logging::log(
                "PROMPTS",
                &format!("PathPrompt navigating to parent: {}", parent_path),
            );
            self.navigate_to(&parent_path, cx);
        }
        // If at root, do nothing
    }

    /// Navigate into selected directory (right arrow / tab)
    pub fn navigate_into_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(entry) = self.filtered_entries.get(self.selected_index) {
            if entry.is_dir {
                let path = entry.path.clone();
                logging::log("PROMPTS", &format!("PathPrompt navigating into: {}", path));
                self.navigate_to(&path, cx);
            }
            // If selected entry is a file, do nothing
        }
    }

    /// Get the currently selected path info (for actions dialog)
    pub fn get_selected_path_info(&self) -> Option<PathInfo> {
        self.filtered_entries
            .get(self.selected_index)
            .map(|entry| PathInfo::new(entry.name.clone(), entry.path.clone(), entry.is_dir))
    }
}

impl Focusable for PathPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PathPromptEvent> for PathPrompt {}

impl Render for PathPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check if actions dialog is showing - if so, don't handle most keys
                // The ActionsDialog has its own key handler and will handle them
                let actions_showing = this.actions_showing.lock().map(|g| *g).unwrap_or(false);

                // Cmd+K always toggles actions (whether showing or not)
                if has_cmd && key_str == "k" {
                    this.toggle_actions(cx);
                    return;
                }

                // When actions are showing, let the ActionsDialog handle all other keys
                // The ActionsDialog is focused and has its own on_key_down handler
                if actions_showing {
                    // Don't handle any other keys - let them bubble to ActionsDialog
                    return;
                }

                match key_str.as_str() {
                    "up" | "arrowup" => this.move_up(cx),
                    "down" | "arrowdown" => this.move_down(cx),
                    "left" | "arrowleft" => this.navigate_to_parent(cx),
                    "right" | "arrowright" => this.navigate_into_selected(cx),
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            this.navigate_to_parent(cx);
                        } else {
                            this.navigate_into_selected(cx);
                        }
                    }
                    "enter" => this.handle_enter(cx),
                    "escape" => {
                        logging::log(
                            "PROMPTS",
                            "PathPrompt: Escape key pressed - calling submit_cancel()",
                        );
                        this.submit_cancel();
                    }
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // Use ListItemColors for consistent theming - always use theme
        let list_colors = ListItemColors::from_theme(&self.theme);

        // Clone values needed for the closure
        let filtered_count = self.filtered_entries.len();
        let selected_index = self.selected_index;

        // Clone entries for the closure (uniform_list callback doesn't have access to self)
        let entries_for_list: Vec<(String, bool)> = self
            .filtered_entries
            .iter()
            .map(|e| (e.name.clone(), e.is_dir))
            .collect();

        // Build list items using ListItem component for consistent styling
        let list = uniform_list(
            "path-list",
            filtered_count,
            move |visible_range: std::ops::Range<usize>, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let (name, is_dir) = &entries_for_list[ix];
                        let is_selected = ix == selected_index;

                        // Choose icon based on entry type
                        let icon = if *is_dir {
                            IconKind::Emoji("📁".to_string())
                        } else {
                            IconKind::Emoji("📄".to_string())
                        };

                        // No description needed - folder icon 📁 is sufficient
                        let description: Option<String> = None;

                        // Use ListItem component for consistent styling with main menu
                        ListItem::new(name.clone(), list_colors)
                            .index(ix)
                            .icon_kind(icon)
                            .description_opt(description)
                            .selected(is_selected)
                            .with_accent_bar(true)
                            .into_any_element()
                    })
                    .collect()
            },
        )
        .track_scroll(&self.list_scroll_handle)
        .flex_1()
        .w_full();

        // Get entity handles for click callbacks
        let handle_select = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();

        // Check if actions are currently showing (for CLS-free toggle)
        let show_actions = self.actions_showing.lock().map(|g| *g).unwrap_or(false);

        // Get actions search text from shared state
        let actions_search_text = self
            .actions_search_text
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();

        // Create path prefix for display in search input
        let path_prefix = format!("{}/", self.current_path.trim_end_matches('/'));

        // Create header colors and config using shared components - always use theme
        let header_colors = PromptHeaderColors::from_theme(&self.theme);

        let header_config = PromptHeaderConfig::new()
            .filter_text(self.filter_text.clone())
            .placeholder("Type to filter...")
            .path_prefix(Some(path_prefix))
            .primary_button_label("Select")
            .primary_button_shortcut("↵")
            .show_actions_button(true)
            .cursor_visible(self.cursor_visible)
            .actions_mode(show_actions)
            .actions_search_text(actions_search_text)
            .focused(!show_actions);

        let header = PromptHeader::new(header_config, header_colors)
            .on_primary_click(Box::new(move |_, _window, cx| {
                logging::log("CLICK", "PathPrompt primary button (Select) clicked");
                if let Some(prompt) = handle_select.upgrade() {
                    prompt.update(cx, |this, cx| {
                        this.submit_selected(cx);
                    });
                }
            }))
            .on_actions_click(Box::new(move |_, _window, cx| {
                logging::log("CLICK", "PathPrompt actions button clicked");
                if let Some(prompt) = handle_actions.upgrade() {
                    prompt.update(cx, |this, cx| {
                        this.toggle_actions(cx);
                    });
                }
            }));

        // Create hint text for footer
        let hint_text = self.hint.clone().unwrap_or_else(|| {
            format!("{} items • ↑↓ navigate • ←→ in/out • Enter open • Tab into • ⌘K actions • Esc cancel", filtered_count)
        });

        // Create container colors and config - always use theme
        let container_colors = PromptContainerColors::from_theme(&self.theme);

        let container_config = PromptContainerConfig::new()
            .show_divider(true)
            .hint(hint_text);

        // Build the final container with the outer wrapper for key handling and focus
        div()
            .id(gpui::ElementId::Name("window:path".into()))
            .w_full()
            .h_full()
            .key_context("path_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(
                PromptContainer::new(container_colors)
                    .config(container_config)
                    .header(header)
                    .content(list),
            )
    }
}

</file>

<file path="src/actions/mod.rs">
//! Actions Dialog Module
//!
//! Provides a searchable action menu as a compact overlay popup for quick access
//! to script management and global actions (edit, create, settings, quit, etc.)
//!
//! The dialog can be rendered in two ways:
//! 1. As an inline overlay within the main window (legacy)
//! 2. As a separate floating window with its own vibrancy blur (preferred)
//!
//! ## Module Structure
//! - `types`: Core types (Action, ActionCategory, ScriptInfo)
//! - `builders`: Factory functions for creating action lists
//! - `constants`: Popup dimensions and styling constants
//! - `dialog`: ActionsDialog struct and implementation
//! - `window`: Separate vibrancy window for actions panel

mod builders;
mod constants;
mod dialog;
mod types;
mod window;

// Re-export only the public API that is actually used externally:
// - ScriptInfo: used by main.rs for action context
// - ActionsDialog: the main dialog component
// - Window functions for separate vibrancy window

pub use builders::to_deeplink_name;
#[allow(unused_imports)]
pub use builders::ClipboardEntryInfo;
pub use dialog::ActionsDialog;
pub use types::ScriptInfo;

// Public API for AI window integration (re-exported but may appear unused until integration)
#[allow(unused_imports)]
pub use builders::get_ai_command_bar_actions;
#[allow(unused_imports)]
pub use types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle,
};

// Window functions for separate vibrancy window
pub use window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window,
};
// get_actions_window_handle available but not re-exported (use window:: directly if needed)

#[cfg(test)]
mod tests {
    // Import from submodules directly - these are only used in tests
    use super::builders::{get_global_actions, get_script_context_actions};
    use super::constants::{ACTION_ITEM_HEIGHT, POPUP_MAX_HEIGHT};
    use super::types::{Action, ActionCategory, ScriptInfo};
    use crate::protocol::ProtocolAction;

    #[test]
    fn test_actions_exceed_visible_space() {
        // Verify script context actions count
        // Global actions are now empty (Settings/Quit in main menu only)
        let script = ScriptInfo::new("test-script", "/path/to/test.ts");
        let script_actions = get_script_context_actions(&script);
        let global_actions = get_global_actions();
        let total_actions = script_actions.len() + global_actions.len();

        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;

        // Script context actions: run, edit, add_shortcut (or update+remove),
        // view_logs, reveal_in_finder, copy_path, copy_deeplink = 7 actions
        assert!(
            total_actions >= 7,
            "Should have at least 7 script context actions"
        );
        assert!(global_actions.is_empty(), "Global actions should be empty");

        // Log for visibility
        println!(
            "Total actions: {}, Max visible: {}",
            total_actions, max_visible
        );
    }

    #[test]
    fn test_protocol_action_to_action_conversion() {
        let protocol_action = ProtocolAction {
            name: "Copy".to_string(),
            description: Some("Copy to clipboard".to_string()),
            shortcut: Some("cmd+c".to_string()),
            value: Some("copy-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };

        // Test that ProtocolAction fields are accessible for conversion
        // The actual conversion in dialog.rs copies these to Action struct
        assert_eq!(protocol_action.name, "Copy");
        assert_eq!(
            protocol_action.description,
            Some("Copy to clipboard".to_string())
        );
        assert_eq!(protocol_action.shortcut, Some("cmd+c".to_string()));
        assert_eq!(protocol_action.value, Some("copy-value".to_string()));
        assert!(protocol_action.has_action);

        // Create Action using builder pattern (used by get_*_actions)
        let action = Action::new(
            protocol_action.name.clone(),
            protocol_action.name.clone(),
            protocol_action.description.clone(),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.id, "Copy");
        assert_eq!(action.title, "Copy");
    }

    #[test]
    fn test_protocol_action_has_action_routing() {
        // Action with has_action=true should trigger ActionTriggered to SDK
        let action_with_handler = ProtocolAction {
            name: "Custom Action".to_string(),
            description: None,
            shortcut: None,
            value: Some("custom-value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        };
        assert!(action_with_handler.has_action);

        // Action with has_action=false should submit value directly
        let action_without_handler = ProtocolAction {
            name: "Simple Action".to_string(),
            description: None,
            shortcut: None,
            value: Some("simple-value".to_string()),
            has_action: false,
            visible: None,
            close: None,
        };
        assert!(!action_without_handler.has_action);
    }
}

</file>

<file path="src/actions/window.rs">
//! Actions Window - Separate vibrancy window for actions panel
//!
//! This creates a floating popup window with its own vibrancy blur effect,
//! similar to Raycast's actions panel. The window is:
//! - Non-draggable (fixed position relative to main window)
//! - Positioned below the header, at the right edge of main window
//! - Auto-closes when app loses focus
//! - Shares the ActionsDialog entity with the main app for keyboard routing

use crate::platform;
use crate::theme;
use crate::window_resize::layout::FOOTER_HEIGHT;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, DisplayId, Entity, FocusHandle, Focusable, Pixels,
    Point, Render, Size, Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use gpui_component::Root;
use std::sync::{Mutex, OnceLock};

use super::constants::{ACTION_ITEM_HEIGHT, HEADER_HEIGHT, POPUP_MAX_HEIGHT, SEARCH_INPUT_HEIGHT};
use super::dialog::ActionsDialog;

/// Global singleton for the actions window handle
static ACTIONS_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

/// Actions window width (height is calculated dynamically based on content)
const ACTIONS_WINDOW_WIDTH: f32 = 320.0;
/// Horizontal margin from main window right edge
const ACTIONS_MARGIN_X: f32 = 8.0;
/// Vertical margin from header
const ACTIONS_MARGIN_Y: f32 = 8.0;

/// ActionsWindow wrapper that renders the shared ActionsDialog entity
pub struct ActionsWindow {
    /// The shared dialog entity (created by main app, rendered here)
    pub dialog: Entity<ActionsDialog>,
    /// Focus handle for this window (not actively used - main window keeps focus)
    pub focus_handle: FocusHandle,
}

impl ActionsWindow {
    pub fn new(dialog: Entity<ActionsDialog>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            dialog,
            focus_handle,
        }
    }
}

impl Focusable for ActionsWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ActionsWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Key handler for the actions window
        // Since this is a separate window, it needs its own key handling
        // (the parent window can't route events to us)
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let modifiers = &event.keystroke.modifiers;

            match key {
                "up" | "arrowup" => {
                    this.dialog.update(cx, |d, cx| d.move_up(cx));
                    cx.notify();
                }
                "down" | "arrowdown" => {
                    this.dialog.update(cx, |d, cx| d.move_down(cx));
                    cx.notify();
                }
                "enter" | "return" => {
                    // Get selected action and execute via callback
                    let action_id = this.dialog.read(cx).get_selected_action_id();
                    if let Some(action_id) = action_id {
                        // Execute the action's callback
                        let callback = this.dialog.read(cx).on_select.clone();
                        callback(action_id.clone());
                        // Close the window
                        window.remove_window();
                    }
                }
                "escape" => {
                    // Close the window
                    window.remove_window();
                }
                "backspace" | "delete" => {
                    this.dialog.update(cx, |d, cx| d.handle_backspace(cx));
                    cx.notify();
                }
                _ => {
                    // Handle printable characters for search (when no modifiers)
                    if !modifiers.platform && !modifiers.control && !modifiers.alt {
                        if let Some(ch) = key.chars().next() {
                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_'
                            {
                                this.dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                cx.notify();
                            }
                        }
                    }
                }
            }
        });

        // Render the shared dialog entity with key handling
        // Don't use size_full() - the dialog calculates its own dynamic height
        // This prevents unused window space from showing as a dark area
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(self.dialog.clone())
    }
}

/// Open the actions window as a separate floating window with vibrancy
///
/// The window is positioned at the top-right of the main window, below the header.
/// It does NOT take keyboard focus - the main window keeps focus and routes
/// keyboard events to the shared ActionsDialog entity.
///
/// # Arguments
/// * `cx` - The application context
/// * `main_window_bounds` - The bounds of the main window in SCREEN-RELATIVE coordinates
///   (as returned by GPUI's window.bounds() - top-left origin relative to the window's screen)
/// * `display_id` - The display where the main window is located (actions window will be on same display)
/// * `dialog_entity` - The shared ActionsDialog entity (created by main app)
///
/// # Returns
/// The window handle on success
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
) -> anyhow::Result<WindowHandle<Root>> {
    // Close any existing actions window first
    close_actions_window(cx);

    // Load theme for vibrancy settings
    let theme = theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate dynamic window height based on number of actions
    // This ensures the window fits the content without empty dark space
    let dialog = dialog_entity.read(cx);
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.context_title.is_some();

    let search_box_height = if hide_search {
        0.0
    } else {
        SEARCH_INPUT_HEIGHT
    };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = 2.0; // top + bottom border
    let dynamic_height = items_height + search_box_height + header_height + border_height;

    // Calculate window position:
    // - X: Right edge of main window, minus actions width, minus margin
    // - Y: Bottom-aligned with main window (above footer), minus margin
    //
    // CRITICAL: main_window_bounds must be in SCREEN-RELATIVE coordinates from GPUI's
    // window.bounds(). These are top-left origin, relative to the window's current screen.
    // When we pass display_id to WindowOptions, GPUI will position this window on the
    // same screen as the main window, using these screen-relative coordinates.
    let window_width = px(ACTIONS_WINDOW_WIDTH);
    let window_height = px(dynamic_height);

    let window_x = main_window_bounds.origin.x + main_window_bounds.size.width
        - window_width
        - px(ACTIONS_MARGIN_X);
    // Position popup above the footer (footer is 40px)
    let window_y = main_window_bounds.origin.y + main_window_bounds.size.height
        - window_height
        - px(FOOTER_HEIGHT)
        - px(ACTIONS_MARGIN_Y);

    let bounds = Bounds {
        origin: Point {
            x: window_x,
            y: window_y,
        },
        size: Size {
            width: window_width,
            height: window_height,
        },
    };

    crate::logging::log(
        "ACTIONS",
        &format!(
            "Opening actions window at ({:?}, {:?}), size {:?}x{:?}, display_id={:?}",
            window_x, window_y, window_width, window_height, display_id
        ),
    );

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None, // No titlebar = no drag affordance
        window_background,
        focus: true, // Take focus so we receive keyboard events for navigation
        show: true,
        kind: WindowKind::PopUp, // Floating popup window
        display_id,              // CRITICAL: Position on same display as main window
        ..Default::default()
    };

    // Create the window with the shared dialog entity
    let handle = cx.open_window(window_options, |window, cx| {
        let actions_window = cx.new(|cx| {
            let aw = ActionsWindow::new(dialog_entity, cx);
            // Focus the actions window so it receives keyboard events
            aw.focus_handle.focus(window, cx);
            aw
        });
        // Wrap in Root for gpui-component theming and vibrancy
        cx.new(|cx| Root::new(actions_window, window, cx))
    })?;

    // Configure the window as non-movable on macOS
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSApp;
        use cocoa::base::nil;
        use objc::{msg_send, sel, sel_impl};

        // Get the NSWindow from the app's windows array
        // The popup window should be the most recently created one
        unsafe {
            let app: cocoa::base::id = NSApp();
            let windows: cocoa::base::id = msg_send![app, windows];
            let count: usize = msg_send![windows, count];
            if count > 0 {
                // Get the last window (most recently created)
                let ns_window: cocoa::base::id = msg_send![windows, lastObject];
                if ns_window != nil {
                    platform::configure_actions_popup_window(ns_window);
                }
            }
        }
    }

    // Store the handle globally
    let window_storage = ACTIONS_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }

    crate::logging::log("ACTIONS", "Actions popup window opened with vibrancy");

    Ok(handle)
}

/// Close the actions window if it's open
pub fn close_actions_window(cx: &mut App) {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(mut guard) = window_storage.lock() {
            if let Some(handle) = guard.take() {
                crate::logging::log("ACTIONS", "Closing actions popup window");
                // Close the window
                let _ = handle.update(cx, |_root, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

/// Check if the actions window is currently open
pub fn is_actions_window_open() -> bool {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return guard.is_some();
        }
    }
    false
}

/// Get the actions window handle if it exists
pub fn get_actions_window_handle() -> Option<WindowHandle<Root>> {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return *guard;
        }
    }
    None
}

/// Notify the actions window to re-render (call after updating dialog entity)
pub fn notify_actions_window(cx: &mut App) {
    if let Some(handle) = get_actions_window_handle() {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

/// Resize the actions window to fit the current number of filtered actions
/// Call this after filtering changes the action count
///
/// The window is "pinned to bottom" - the search input stays in place and
/// the window shrinks/grows from the top.
pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>) {
    if let Some(handle) = get_actions_window_handle() {
        // Read dialog state to calculate new height
        let dialog = dialog_entity.read(cx);
        let num_actions = dialog.filtered_actions.len();
        let hide_search = dialog.hide_search;
        let has_header = dialog.context_title.is_some();

        // Calculate new height (same logic as open_actions_window)
        let search_box_height = if hide_search {
            0.0
        } else {
            SEARCH_INPUT_HEIGHT
        };
        let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
        let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let border_height = 2.0; // top + bottom border
        let new_height_f32 = items_height + search_box_height + header_height + border_height;

        let _ = handle.update(cx, |_root, window, cx| {
            let current_bounds = window.bounds();
            let current_height_f32: f32 = current_bounds.size.height.into();
            let current_width_f32: f32 = current_bounds.size.width.into();

            // Skip if height hasn't changed
            if (current_height_f32 - new_height_f32).abs() < 1.0 {
                return;
            }

            // "Pin to bottom": keep the bottom edge fixed
            // In macOS screen coords (bottom-left origin), the bottom of the window
            // is at frame.origin.y. When we change height, we keep origin.y the same
            // and only change height - this naturally keeps the bottom fixed.
            #[cfg(target_os = "macos")]
            {
                use cocoa::appkit::NSScreen;
                use cocoa::base::nil;
                use cocoa::foundation::{NSPoint, NSRect, NSSize};
                use objc::{msg_send, sel, sel_impl};

                unsafe {
                    let ns_app: cocoa::base::id = cocoa::appkit::NSApp();
                    let windows: cocoa::base::id = msg_send![ns_app, windows];
                    let count: usize = msg_send![windows, count];

                    // Find our actions window by matching current dimensions
                    for i in 0..count {
                        let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                        if ns_window == nil {
                            continue;
                        }

                        let frame: NSRect = msg_send![ns_window, frame];

                        // Match by width (actions window has unique width)
                        if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                            && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                        {
                            // Get the screen this window is on (NOT primary screen!)
                            let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                            if window_screen == nil {
                                // Fallback to primary if no screen
                                let screens: cocoa::base::id = NSScreen::screens(nil);
                                let _primary: cocoa::base::id =
                                    msg_send![screens, objectAtIndex: 0u64];
                            }

                            // In macOS coords (bottom-left origin):
                            // - frame.origin.y is the BOTTOM of the window
                            // - To keep bottom fixed, we keep origin.y the same
                            // - Only change the height
                            let new_frame = NSRect::new(
                                NSPoint::new(frame.origin.x, frame.origin.y),
                                NSSize::new(frame.size.width, new_height_f32 as f64),
                            );

                            let _: () =
                                msg_send![ns_window, setFrame:new_frame display:true animate:false];

                            crate::logging::log(
                                "ACTIONS",
                                &format!(
                                    "Resized actions window (bottom pinned): height {:.0} -> {:.0}",
                                    current_height_f32, new_height_f32
                                ),
                            );
                            break;
                        }
                    }
                }
            }

            // Also tell GPUI about the new size
            window.resize(Size {
                width: current_bounds.size.width,
                height: px(new_height_f32),
            });
            cx.notify();
        });

        crate::logging::log(
            "ACTIONS",
            &format!(
                "Resized actions window: {} items, height={:.0}",
                num_actions, new_height_f32
            ),
        );
    }
}

</file>

<file path="src/actions/dialog.rs">
//! Actions Dialog
//!
//! The main ActionsDialog struct and its implementation, providing a searchable
//! action menu as a compact overlay popup.

#![allow(dead_code)]

use crate::components::scrollbar::{Scrollbar, ScrollbarColors};
use crate::designs::{get_tokens, DesignColors, DesignVariant};
use crate::logging;
use crate::protocol::ProtocolAction;
use crate::theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, svg, uniform_list, App, BoxShadow, Context, FocusHandle,
    Focusable, Render, ScrollStrategy, SharedString, UniformListScrollHandle, Window,
};
use std::sync::Arc;

use super::builders::{
    get_chat_context_actions, get_clipboard_history_context_actions, get_file_context_actions,
    get_global_actions, get_path_context_actions, get_script_context_actions,
    get_scriptlet_context_actions_with_custom, ChatPromptInfo, ClipboardEntryInfo,
};
use super::constants::{
    ACTION_ITEM_HEIGHT, ACTION_ROW_INSET, HEADER_HEIGHT, KEYCAP_HEIGHT, KEYCAP_MIN_WIDTH,
    POPUP_MAX_HEIGHT, POPUP_WIDTH, SEARCH_INPUT_HEIGHT, SELECTION_RADIUS,
};
use crate::file_search::FileInfo;
use crate::scriptlets::Scriptlet;

// Keep ACCENT_BAR_WIDTH for backwards compatibility during transition
#[allow(unused_imports)]
use super::constants::ACCENT_BAR_WIDTH;
#[allow(unused_imports)] // AnchorPosition reserved for future use
use super::types::{
    Action, ActionCallback, ActionCategory, ActionsDialogConfig, AnchorPosition, ScriptInfo,
    SearchPosition, SectionStyle,
};
use crate::prompts::PathInfo;

/// Helper function to combine a hex color with an alpha value
/// Delegates to DesignColors::hex_with_alpha for DRY
#[inline]
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    DesignColors::hex_with_alpha(hex, alpha)
}

/// ActionsDialog - Compact overlay popup for quick actions
/// Implements Raycast-style design with individual keycap shortcuts
///
/// # Configuration
/// Use `ActionsDialogConfig` to customize appearance:
/// - `search_position`: Top (AI chat style) or Bottom (main menu style)
/// - `section_style`: Headers (text labels) or Separators (subtle lines)
/// - `anchor`: Top (list grows down) or Bottom (list grows up)
/// - `show_icons`: Display icons next to actions
/// - `show_footer`: Show keyboard hint footer
pub struct ActionsDialog {
    pub actions: Vec<Action>,
    pub filtered_actions: Vec<usize>, // Indices into actions
    pub selected_index: usize,        // Index within filtered_actions
    pub search_text: String,
    pub focus_handle: FocusHandle,
    pub on_select: ActionCallback,
    /// Currently focused script for context-aware actions
    pub focused_script: Option<ScriptInfo>,
    /// Currently focused scriptlet (for H3-defined custom actions)
    pub focused_scriptlet: Option<Scriptlet>,
    /// Scroll handle for uniform_list virtualization
    pub scroll_handle: UniformListScrollHandle,
    /// Theme for consistent color styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling (defaults to Default for theme-based styling)
    pub design_variant: DesignVariant,
    /// Cursor visibility for blinking (controlled externally)
    pub cursor_visible: bool,
    /// When true, hide the search input (used when rendered inline in main.rs header)
    pub hide_search: bool,
    /// SDK-provided actions (when present, replaces built-in actions)
    pub sdk_actions: Option<Vec<ProtocolAction>>,
    /// Context title shown in the header (e.g., "Activity Monitor", script name)
    pub context_title: Option<String>,
    /// Configuration for appearance and behavior
    pub config: ActionsDialogConfig,
}

impl ActionsDialog {
    pub fn new(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_script_and_design(focus_handle, on_select, None, theme, DesignVariant::Default)
    }

    pub fn with_script(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
    ) -> Self {
        Self::with_script_and_design(
            focus_handle,
            on_select,
            focused_script,
            theme,
            DesignVariant::Default,
        )
    }

    pub fn with_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        Self::with_script_and_design(focus_handle, on_select, None, theme, design_variant)
    }

    /// Create ActionsDialog for a path (file/folder) with path-specific actions
    pub fn with_path(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        path_info: &PathInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_path_context_actions(path_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for path: {} (is_dir={}) with {} actions",
                path_info.path,
                path_info.is_dir,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            scroll_handle: UniformListScrollHandle::new(),
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(path_info.path.clone()),
            config: ActionsDialogConfig::default(),
        }
    }

    /// Create ActionsDialog for a file search result with file-specific actions
    /// Actions: Open, Show in Finder, Quick Look, Open With..., Show Info, Copy Path
    pub fn with_file(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        file_info: &FileInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_file_context_actions(file_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for file: {} (is_dir={}) with {} actions",
                file_info.path,
                file_info.is_dir,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            scroll_handle: UniformListScrollHandle::new(),
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(file_info.name.clone()),
            config: ActionsDialogConfig::default(),
        }
    }

    /// Create ActionsDialog for a clipboard history entry with clipboard-specific actions
    /// Actions: Paste, Copy, Paste and Keep Open, Share, Attach to AI, Pin/Unpin, Delete, etc.
    pub fn with_clipboard_entry(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        entry_info: &ClipboardEntryInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_clipboard_history_context_actions(entry_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();

        let context_title = if entry_info.preview.len() > 30 {
            format!("{}...", &entry_info.preview[..27])
        } else {
            entry_info.preview.clone()
        };

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for clipboard entry: {} (type={:?}, pinned={}) with {} actions",
                entry_info.id,
                entry_info.content_type,
                entry_info.pinned,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            scroll_handle: UniformListScrollHandle::new(),
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(context_title),
            config: ActionsDialogConfig::default(),
        }
    }

    /// Create ActionsDialog for a chat prompt with chat-specific actions
    /// Actions: Model selection, Continue in Chat, Copy Response, Clear Conversation
    pub fn with_chat(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        chat_info: &ChatPromptInfo,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let actions = get_chat_context_actions(chat_info);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();

        let context_title = chat_info
            .current_model
            .clone()
            .unwrap_or_else(|| "Chat".to_string());

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created for chat prompt: model={:?} with {} actions",
                chat_info.current_model,
                actions.len()
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            scroll_handle: UniformListScrollHandle::new(),
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title: Some(context_title),
            config: ActionsDialogConfig::default(),
        }
    }

    pub fn with_script_and_design(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        focused_script: Option<ScriptInfo>,
        theme: Arc<theme::Theme>,
        design_variant: DesignVariant,
    ) -> Self {
        let actions = Self::build_actions(&focused_script, &None);
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created with {} actions, script: {:?}, design: {:?}",
                actions.len(),
                focused_script.as_ref().map(|s| &s.name),
                design_variant
            ),
        );

        // Log theme color configuration for debugging
        logging::log("ACTIONS_THEME", &format!(
            "Theme colors applied: bg_main=#{:06x}, bg_search=#{:06x}, text_primary=#{:06x}, accent_selected=#{:06x}",
            theme.colors.background.main,
            theme.colors.background.search_box,
            theme.colors.text.primary,
            theme.colors.accent.selected
        ));

        // Extract context title from focused script if available
        let context_title = focused_script.as_ref().map(|s| s.name.clone());

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script,
            focused_scriptlet: None,
            scroll_handle: UniformListScrollHandle::new(),
            theme,
            design_variant,
            cursor_visible: true,
            hide_search: false,
            sdk_actions: None,
            context_title,
            config: ActionsDialogConfig::default(),
        }
    }

    /// Update cursor visibility (called from parent's blink timer)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Hide the search input (for inline mode where header has search)
    pub fn set_hide_search(&mut self, hide: bool) {
        self.hide_search = hide;
    }

    /// Set the context title shown in the header
    pub fn set_context_title(&mut self, title: Option<String>) {
        self.context_title = title;
    }

    /// Set the configuration for appearance and behavior
    pub fn set_config(&mut self, config: ActionsDialogConfig) {
        self.config = config;
        // Update hide_search based on config for backwards compatibility
        self.hide_search = matches!(self.config.search_position, SearchPosition::Hidden);
    }

    /// Create ActionsDialog with custom configuration and actions
    ///
    /// Use this for contexts like AI chat that need different appearance:
    /// - Search at top instead of bottom
    /// - Section headers instead of separators
    /// - Icons next to actions
    pub fn with_config(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        actions: Vec<Action>,
        theme: Arc<theme::Theme>,
        config: ActionsDialogConfig,
    ) -> Self {
        let filtered_actions: Vec<usize> = (0..actions.len()).collect();

        logging::log(
            "ACTIONS",
            &format!(
                "ActionsDialog created with config: {} actions, search={:?}",
                actions.len(),
                config.search_position
            ),
        );

        ActionsDialog {
            actions,
            filtered_actions,
            selected_index: 0,
            search_text: String::new(),
            focus_handle,
            on_select,
            focused_script: None,
            focused_scriptlet: None,
            scroll_handle: UniformListScrollHandle::new(),
            theme,
            design_variant: DesignVariant::Default,
            cursor_visible: true,
            hide_search: matches!(config.search_position, SearchPosition::Hidden),
            sdk_actions: None,
            context_title: None,
            config,
        }
    }

    /// Parse a shortcut string into individual keycap characters
    /// e.g., "⌘↵" → vec!["⌘", "↵"], "⌘I" → vec!["⌘", "I"]
    fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
        let mut keycaps = Vec::new();

        for ch in shortcut.chars() {
            // Handle modifier symbols (single character)
            match ch {
                '⌘' | '⌃' | '⌥' | '⇧' | '↵' | '⎋' | '⇥' | '⌫' | '␣' | '↑' | '↓' | '←' | '→' =>
                {
                    keycaps.push(ch.to_string());
                }
                // Regular characters (letters, numbers)
                _ => {
                    keycaps.push(ch.to_uppercase().to_string());
                }
            }
        }

        keycaps
    }

    /// Set actions from SDK (replaces built-in actions)
    ///
    /// Converts `ProtocolAction` items to internal `Action` format and updates
    /// the actions list. Filters out actions with `visible: false`.
    /// The `has_action` field on each action determines routing:
    /// - `has_action=true`: Send ActionTriggered back to SDK
    /// - `has_action=false`: Submit value directly
    pub fn set_sdk_actions(&mut self, actions: Vec<ProtocolAction>) {
        let total_count = actions.len();
        let visible_actions: Vec<&ProtocolAction> =
            actions.iter().filter(|a| a.is_visible()).collect();
        let visible_count = visible_actions.len();

        let converted: Vec<Action> = visible_actions
            .iter()
            .map(|pa| Action {
                id: pa.name.clone(),
                title: pa.name.clone(),
                description: pa.description.clone(),
                category: ActionCategory::ScriptContext,
                shortcut: pa.shortcut.as_ref().map(|s| Self::format_shortcut_hint(s)),
                has_action: pa.has_action,
                value: pa.value.clone(),
                icon: None,    // SDK actions don't currently have icons
                section: None, // SDK actions don't currently have sections
            })
            .collect();

        logging::log(
            "ACTIONS",
            &format!(
                "SDK actions set: {} visible of {} total",
                visible_count, total_count
            ),
        );

        self.actions = converted;
        self.filtered_actions = (0..self.actions.len()).collect();
        self.selected_index = 0;
        self.search_text.clear();
        self.sdk_actions = Some(actions);
    }

    /// Format a keyboard shortcut for display (e.g., "cmd+c" → "⌘C")
    fn format_shortcut_hint(shortcut: &str) -> String {
        let mut result = String::new();
        let parts: Vec<&str> = shortcut.split('+').collect();

        for (i, part) in parts.iter().enumerate() {
            let part_lower = part.trim().to_lowercase();
            let formatted = match part_lower.as_str() {
                // Modifier keys → symbols
                "cmd" | "command" | "meta" | "super" => "⌘",
                "ctrl" | "control" => "⌃",
                "alt" | "opt" | "option" => "⌥",
                "shift" => "⇧",
                // Special keys
                "enter" | "return" => "↵",
                "escape" | "esc" => "⎋",
                "tab" => "⇥",
                "backspace" | "delete" => "⌫",
                "space" => "␣",
                "up" | "arrowup" => "↑",
                "down" | "arrowdown" => "↓",
                "left" | "arrowleft" => "←",
                "right" | "arrowright" => "→",
                // Regular letters/numbers → uppercase
                _ => {
                    // Check if it's the last part (the actual key)
                    if i == parts.len() - 1 {
                        // Uppercase single characters, keep others as-is
                        result.push_str(&part.trim().to_uppercase());
                        continue;
                    }
                    part.trim()
                }
            };
            result.push_str(formatted);
        }

        result
    }

    /// Clear SDK actions and restore built-in actions
    pub fn clear_sdk_actions(&mut self) {
        if self.sdk_actions.is_some() {
            logging::log(
                "ACTIONS",
                "Clearing SDK actions, restoring built-in actions",
            );
            self.sdk_actions = None;
            self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
            self.filtered_actions = (0..self.actions.len()).collect();
            self.selected_index = 0;
            self.search_text.clear();
        }
    }

    /// Check if SDK actions are currently active
    pub fn has_sdk_actions(&self) -> bool {
        self.sdk_actions.is_some()
    }

    /// Get the currently selected action (for external handling)
    pub fn get_selected_action(&self) -> Option<&Action> {
        self.filtered_actions
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx))
    }

    /// Build the complete actions list based on focused script and optional scriptlet
    fn build_actions(
        focused_script: &Option<ScriptInfo>,
        focused_scriptlet: &Option<Scriptlet>,
    ) -> Vec<Action> {
        let mut actions = Vec::new();

        // Add script-specific actions first if a script is focused
        if let Some(script) = focused_script {
            // If this is a scriptlet with custom actions, use the enhanced builder
            if script.is_scriptlet && focused_scriptlet.is_some() {
                actions.extend(get_scriptlet_context_actions_with_custom(
                    script,
                    focused_scriptlet.as_ref(),
                ));
            } else {
                // Use standard actions for regular scripts
                actions.extend(get_script_context_actions(script));
            }
        }

        // Add global actions
        actions.extend(get_global_actions());

        actions
    }

    /// Update the focused script and rebuild actions
    pub fn set_focused_script(&mut self, script: Option<ScriptInfo>) {
        self.focused_script = script;
        self.focused_scriptlet = None; // Clear scriptlet when only setting script
        self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
        self.refilter();
    }

    /// Update both the focused script and scriptlet for custom actions
    ///
    /// Use this when the focused item is a scriptlet with H3-defined custom actions.
    /// The scriptlet's actions will appear in the Actions Menu.
    pub fn set_focused_scriptlet(
        &mut self,
        script: Option<ScriptInfo>,
        scriptlet: Option<Scriptlet>,
    ) {
        self.focused_script = script;
        self.focused_scriptlet = scriptlet;
        self.actions = Self::build_actions(&self.focused_script, &self.focused_scriptlet);
        self.refilter();

        logging::log(
            "ACTIONS",
            &format!(
                "Set focused scriptlet with {} custom actions",
                self.focused_scriptlet
                    .as_ref()
                    .map(|s| s.actions.len())
                    .unwrap_or(0)
            ),
        );
    }

    /// Update the theme when hot-reloading
    /// Call this from the parent when theme changes to ensure dialog reflects new colors
    pub fn update_theme(&mut self, theme: Arc<theme::Theme>) {
        logging::log("ACTIONS_THEME", "Theme updated in ActionsDialog");
        self.theme = theme;
    }

    /// Refilter actions based on current search_text using ranked fuzzy matching.
    ///
    /// Scoring system:
    /// - Prefix match on title: +100 (strongest signal)
    /// - Fuzzy match on title: +50 + character bonus
    /// - Contains match on description: +25
    /// - Results are sorted by score (descending)
    fn refilter(&mut self) {
        // Preserve selection if possible (track which action was selected)
        let previously_selected = self
            .filtered_actions
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx).map(|a| a.id.clone()));

        if self.search_text.is_empty() {
            self.filtered_actions = (0..self.actions.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();

            // Score each action and collect (index, score) pairs
            let mut scored: Vec<(usize, i32)> = self
                .actions
                .iter()
                .enumerate()
                .filter_map(|(idx, action)| {
                    let score = Self::score_action(action, &search_lower);
                    if score > 0 {
                        Some((idx, score))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by score descending
            scored.sort_by(|a, b| b.1.cmp(&a.1));

            // Extract just the indices
            self.filtered_actions = scored.into_iter().map(|(idx, _)| idx).collect();
        }

        // Preserve selection if the same action is still in results
        if let Some(prev_id) = previously_selected {
            if let Some(new_idx) = self.filtered_actions.iter().position(|&idx| {
                self.actions
                    .get(idx)
                    .map(|a| a.id == prev_id)
                    .unwrap_or(false)
            }) {
                self.selected_index = new_idx;
            } else {
                self.selected_index = 0;
            }
        } else {
            self.selected_index = 0;
        }

        // Only scroll if we have results
        if !self.filtered_actions.is_empty() {
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
        }

        logging::log_debug(
            "ACTIONS_SCROLL",
            &format!(
                "Filter changed: {} results, selected={}",
                self.filtered_actions.len(),
                self.selected_index
            ),
        );
    }

    /// Score an action against a search query.
    /// Returns 0 if no match, higher scores for better matches.
    fn score_action(action: &Action, search_lower: &str) -> i32 {
        let title_lower = action.title.to_lowercase();
        let mut score = 0;

        // Prefix match on title (strongest)
        if title_lower.starts_with(search_lower) {
            score += 100;
        }
        // Contains match on title
        else if title_lower.contains(search_lower) {
            score += 50;
        }
        // Fuzzy match on title (character-by-character subsequence)
        else if Self::fuzzy_match(&title_lower, search_lower) {
            score += 25;
        }

        // Description match (bonus)
        if let Some(ref desc) = action.description {
            let desc_lower = desc.to_lowercase();
            if desc_lower.contains(search_lower) {
                score += 15;
            }
        }

        // Shortcut match (bonus)
        if let Some(ref shortcut) = action.shortcut {
            if shortcut.to_lowercase().contains(search_lower) {
                score += 10;
            }
        }

        score
    }

    /// Simple fuzzy matching: check if all characters in needle appear in haystack in order.
    fn fuzzy_match(haystack: &str, needle: &str) -> bool {
        let mut haystack_chars = haystack.chars();
        for needle_char in needle.chars() {
            loop {
                match haystack_chars.next() {
                    Some(h) if h == needle_char => break,
                    Some(_) => continue,
                    None => return false,
                }
            }
        }
        true
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            logging::log_debug(
                "ACTIONS_SCROLL",
                &format!("Up: selected_index={}", self.selected_index),
            );
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_actions.len().saturating_sub(1) {
            self.selected_index += 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            logging::log_debug(
                "ACTIONS_SCROLL",
                &format!("Down: selected_index={}", self.selected_index),
            );
            cx.notify();
        }
    }

    /// Get the currently selected action ID (for external handling)
    pub fn get_selected_action_id(&self) -> Option<String> {
        if let Some(&action_idx) = self.filtered_actions.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                return Some(action.id.clone());
            }
        }
        None
    }

    /// Get the currently selected ProtocolAction (for checking close behavior)
    /// Returns the original ProtocolAction from sdk_actions if this is an SDK action,
    /// or None for built-in actions.
    pub fn get_selected_protocol_action(&self) -> Option<&ProtocolAction> {
        let action_id = self.get_selected_action_id()?;
        self.sdk_actions
            .as_ref()?
            .iter()
            .find(|a| a.name == action_id)
    }

    /// Check if the currently selected action should close the dialog
    /// Returns true if the action has close: true (or no close field, which defaults to true)
    /// Returns true for built-in actions (they always close)
    pub fn selected_action_should_close(&self) -> bool {
        if let Some(protocol_action) = self.get_selected_protocol_action() {
            protocol_action.should_close()
        } else {
            // Built-in actions always close
            true
        }
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        if let Some(&action_idx) = self.filtered_actions.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                logging::log("ACTIONS", &format!("Action selected: {}", action.id));
                (self.on_select)(action.id.clone());
            }
        }
    }

    /// Cancel - close the dialog
    pub fn submit_cancel(&mut self) {
        logging::log("ACTIONS", "Actions dialog cancelled");
        (self.on_select)("__cancel__".to_string());
    }

    /// Dismiss the dialog when user clicks outside its bounds.
    /// This is a public method called from the parent container's click-outside handler.
    /// Logs the event and triggers the cancel callback.
    pub fn dismiss_on_click_outside(&mut self) {
        tracing::info!(
            target: "script_kit::actions",
            "ActionsDialog dismiss-on-click-outside triggered"
        );
        logging::log("ACTIONS", "Actions dialog dismissed (click outside)");
        self.submit_cancel();
    }

    /// Create box shadow for the overlay popup
    /// When rendered in a separate vibrancy window, no shadow is needed
    /// (the window vibrancy provides visual separation)
    pub(super) fn create_popup_shadow() -> Vec<BoxShadow> {
        // No shadow - vibrancy window provides visual separation
        vec![]
    }

    /// Get colors for the search box based on design variant
    /// Returns: (search_box_bg, border_color, muted_text, dimmed_text, secondary_text)
    pub(super) fn get_search_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Use theme opacity for input background to support vibrancy
        let opacity = self.theme.get_opacity();
        let input_alpha = (opacity.input * 255.0) as u8;

        if self.design_variant == DesignVariant::Default {
            (
                rgba(hex_with_alpha(
                    self.theme.colors.background.search_box,
                    input_alpha,
                )),
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x80)),
                rgb(self.theme.colors.text.muted),
                rgb(self.theme.colors.text.dimmed),
                rgb(self.theme.colors.text.secondary),
            )
        } else {
            (
                rgba(hex_with_alpha(colors.background_secondary, input_alpha)),
                rgba(hex_with_alpha(colors.border, 0x80)),
                rgb(colors.text_muted),
                rgb(colors.text_dimmed),
                rgb(colors.text_secondary),
            )
        }
    }

    /// Get colors for the main container based on design variant
    /// Returns: (main_bg, container_border, container_text)
    pub(super) fn get_container_colors(
        &self,
        colors: &crate::designs::DesignColors,
    ) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        // Use theme opacity for dialog background to support vibrancy
        // The theme's opacity.dialog controls how transparent the popup is
        let opacity = self.theme.get_opacity();
        let dialog_alpha = (opacity.dialog * 255.0) as u8;

        if self.design_variant == DesignVariant::Default {
            (
                rgba(hex_with_alpha(
                    self.theme.colors.background.main,
                    dialog_alpha,
                )),
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x80)),
                rgb(self.theme.colors.text.secondary),
            )
        } else {
            (
                rgba(hex_with_alpha(colors.background, dialog_alpha)),
                rgba(hex_with_alpha(colors.border, 0x80)),
                rgb(colors.text_secondary),
            )
        }
    }
}

impl Focusable for ActionsDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ActionsDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get design tokens for the current design variant
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let visual = tokens.visual();

        // NOTE: Key handling is done by the parent (ScriptListApp in main.rs)
        // which routes all keyboard events to this dialog's methods.
        // We do NOT attach our own on_key_down handler to avoid double-processing.

        // Render search input - compact version
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Use helper method for design/theme color extraction
        let (search_box_bg, border_color, _muted_text, dimmed_text, _secondary_text) =
            self.get_search_colors(&colors);

        // Get primary text color for cursor (matches main list styling)
        let primary_text = if self.design_variant == DesignVariant::Default {
            rgb(self.theme.colors.text.primary)
        } else {
            rgb(colors.text_primary)
        };

        // Get accent color for the search input focus indicator
        let accent_color_hex = if self.design_variant == DesignVariant::Default {
            self.theme.colors.accent.selected
        } else {
            colors.accent
        };
        let accent_color = rgb(accent_color_hex);

        // Focus border color (accent with transparency)
        let focus_border_color = rgba(hex_with_alpha(accent_color_hex, 0x60));

        // Input container with fixed height and width to prevent any layout shifts
        // The entire row is constrained to prevent resizing when text is entered
        let input_container = div()
            .w(px(POPUP_WIDTH)) // Match parent width exactly
            .min_w(px(POPUP_WIDTH))
            .max_w(px(POPUP_WIDTH))
            .h(px(SEARCH_INPUT_HEIGHT)) // Fixed height for the input row
            .min_h(px(SEARCH_INPUT_HEIGHT))
            .max_h(px(SEARCH_INPUT_HEIGHT))
            .overflow_hidden() // Prevent any content from causing shifts
            .px(px(spacing.item_padding_x))
            .py(px(spacing.item_padding_y + 2.0)) // Slightly more vertical padding
            .bg(search_box_bg)
            .border_t_1() // Border on top since input is now at bottom
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.gap_md))
            .child(
                // Search icon or indicator - fixed width to prevent shifts
                div()
                    .w(px(24.0)) // Fixed width for the icon container
                    .min_w(px(24.0))
                    .text_color(dimmed_text)
                    .text_xs()
                    .child("⌘K"),
            )
            .child(
                // Search input field with focus indicator
                // CRITICAL: Use flex_shrink_0 to prevent flexbox from shrinking this container
                // The border/bg MUST stay at fixed width regardless of content
                div()
                    .flex_shrink_0() // PREVENT flexbox from shrinking this!
                    .w(px(240.0))
                    .min_w(px(240.0))
                    .max_w(px(240.0))
                    .h(px(28.0)) // Fixed height too
                    .min_h(px(28.0))
                    .max_h(px(28.0))
                    .overflow_hidden()
                    .px(px(spacing.padding_sm))
                    .py(px(spacing.padding_xs))
                    // ALWAYS show background - just vary intensity
                    .bg(if self.design_variant == DesignVariant::Default {
                        rgba(hex_with_alpha(
                            self.theme.colors.background.main,
                            if self.search_text.is_empty() {
                                0x20
                            } else {
                                0x40
                            },
                        ))
                    } else {
                        rgba(hex_with_alpha(
                            colors.background,
                            if self.search_text.is_empty() {
                                0x20
                            } else {
                                0x40
                            },
                        ))
                    })
                    .rounded(px(visual.radius_sm))
                    .border_1()
                    // ALWAYS show border - just vary intensity
                    .border_color(if !self.search_text.is_empty() {
                        focus_border_color
                    } else {
                        border_color
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    .text_color(if self.search_text.is_empty() {
                        dimmed_text
                    } else {
                        primary_text
                    })
                    // ALWAYS render cursor div with consistent margin to prevent layout shift
                    // When empty, cursor is at the start before placeholder text
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .mr(px(2.)) // Use consistent 2px margin
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display.clone())
                    // When has text, cursor is at the end after the text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .ml(px(2.)) // Consistent 2px margin
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

        // Render action list using uniform_list for virtualized scrolling
        let actions_container = if self.filtered_actions.is_empty() {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .child(
                    div()
                        .w_full()
                        .py(px(spacing.padding_lg))
                        .px(px(spacing.item_padding_x))
                        .text_color(dimmed_text)
                        .text_sm()
                        .child("No actions match your search"),
                )
                .into_any_element()
        } else {
            // Clone data needed for the uniform_list closure
            let selected_index = self.selected_index;
            let filtered_len = self.filtered_actions.len();
            let design_variant = self.design_variant;
            // NOTE: Removed per-render log - fires every render frame during cursor blink

            // Calculate scrollbar parameters
            // Container height for actions (excluding search box)
            let search_box_height = if self.hide_search {
                0.0
            } else {
                SEARCH_INPUT_HEIGHT
            };
            let container_height = (filtered_len as f32 * ACTION_ITEM_HEIGHT)
                .min(POPUP_MAX_HEIGHT - search_box_height);
            let visible_items = (container_height / ACTION_ITEM_HEIGHT) as usize;

            // Use selected_index as approximate scroll offset
            // When scrolling, the selected item should be visible, so this gives a reasonable estimate
            let scroll_offset = if selected_index > visible_items.saturating_sub(1) {
                selected_index.saturating_sub(visible_items / 2)
            } else {
                0
            };

            // Get scrollbar colors from theme for consistent styling
            let scrollbar_colors = ScrollbarColors::from_theme(&self.theme);

            // Create scrollbar (only visible if content overflows)
            let scrollbar =
                Scrollbar::new(filtered_len, visible_items, scroll_offset, scrollbar_colors)
                    .container_height(container_height);

            let list = uniform_list(
                "actions-list",
                filtered_len,
                cx.processor(
                    move |this: &mut ActionsDialog, visible_range, _window, _cx| {
                        // NOTE: Removed visible range log - fires per render frame

                        // Get tokens for list item rendering
                        let item_tokens = get_tokens(design_variant);
                        let item_colors = item_tokens.colors();
                        let item_spacing = item_tokens.spacing();
                        let item_visual = item_tokens.visual();

                        // Extract colors for list items - use theme opacity for vibrancy
                        let theme_opacity = this.theme.get_opacity();
                        let selected_alpha = (theme_opacity.selected * 255.0) as u32;
                        let hover_alpha = (theme_opacity.hover * 255.0) as u32;

                        let (selected_bg, hover_bg, primary_text, secondary_text, dimmed_text) =
                            if design_variant == DesignVariant::Default {
                                (
                                    rgba(
                                        (this.theme.colors.accent.selected_subtle << 8)
                                            | selected_alpha,
                                    ),
                                    rgba(
                                        (this.theme.colors.accent.selected_subtle << 8)
                                            | hover_alpha,
                                    ),
                                    rgb(this.theme.colors.text.primary),
                                    rgb(this.theme.colors.text.secondary),
                                    rgb(this.theme.colors.text.dimmed),
                                )
                            } else {
                                (
                                    rgba((item_colors.background_selected << 8) | selected_alpha),
                                    rgba((item_colors.background_selected << 8) | hover_alpha),
                                    rgb(item_colors.text_primary),
                                    rgb(item_colors.text_secondary),
                                    rgb(item_colors.text_dimmed),
                                )
                            };

                        let mut items = Vec::new();

                        // Get border color for category separators
                        let separator_color = if design_variant == DesignVariant::Default {
                            rgba(hex_with_alpha(this.theme.colors.ui.border, 0x40))
                        } else {
                            rgba(hex_with_alpha(item_colors.border_subtle, 0x40))
                        };

                        // Get section style from config
                        let section_style = this.config.section_style;

                        for idx in visible_range {
                            if let Some(&action_idx) = this.filtered_actions.get(idx) {
                                if let Some(action) = this.actions.get(action_idx) {
                                    let action: &Action = action; // Explicit type annotation
                                    let is_selected = idx == selected_index;

                                    // Determine if this is the start of a new section/category
                                    // For SectionStyle::Headers, use action.section
                                    // For SectionStyle::Separators, use action.category
                                    let (is_section_start, section_label) = if idx > 0 {
                                        if let Some(&prev_action_idx) =
                                            this.filtered_actions.get(idx - 1)
                                        {
                                            if let Some(prev_action) =
                                                this.actions.get(prev_action_idx)
                                            {
                                                let prev_action: &Action = prev_action;
                                                match section_style {
                                                    SectionStyle::Headers => {
                                                        // Compare section strings
                                                        let different =
                                                            prev_action.section != action.section;
                                                        (different, action.section.clone())
                                                    }
                                                    SectionStyle::Separators
                                                    | SectionStyle::None => {
                                                        // Compare categories
                                                        let different =
                                                            prev_action.category != action.category;
                                                        (different, None)
                                                    }
                                                }
                                            } else {
                                                (false, None)
                                            }
                                        } else {
                                            (false, None)
                                        }
                                    } else {
                                        // First item - show section header if using Headers style
                                        match section_style {
                                            SectionStyle::Headers => (true, action.section.clone()),
                                            _ => (false, None),
                                        }
                                    };

                                    // Match main list styling: bright text when selected, secondary when not
                                    let title_color = if is_selected {
                                        primary_text
                                    } else {
                                        secondary_text
                                    };

                                    let shortcut_color = dimmed_text;

                                    // Clone strings for SharedString conversion
                                    let title_str: String = action.title.clone();
                                    let shortcut_opt: Option<String> = action.shortcut.clone();

                                    // Note: First/last item rounding is handled by the outer container's overflow_hidden
                                    // Keeping these vars commented for reference in case we need them later
                                    // let is_first_item = idx == 0;
                                    // let is_last_item = idx == filtered_len - 1;
                                    let _ = item_visual.radius_lg; // Suppress unused warning

                                    // Get keycap colors for Raycast-style shortcuts
                                    let keycap_bg = if design_variant == DesignVariant::Default {
                                        rgba(hex_with_alpha(this.theme.colors.ui.border, 0x80))
                                    } else {
                                        rgba(hex_with_alpha(item_colors.border, 0x80))
                                    };
                                    let keycap_border = if design_variant == DesignVariant::Default
                                    {
                                        rgba(hex_with_alpha(this.theme.colors.ui.border, 0xA0))
                                    } else {
                                        rgba(hex_with_alpha(item_colors.border, 0xA0))
                                    };

                                    // Raycast-style: compact rows with pill-style selection
                                    // No left accent bar - using rounded background instead
                                    let mut action_item = div()
                                        .id(idx)
                                        .w_full()
                                        .h(px(ACTION_ITEM_HEIGHT)) // Fixed height for uniform_list
                                        .px(px(ACTION_ROW_INSET)) // Horizontal inset for pill effect
                                        .py(px(2.0)) // Minimal vertical padding for tight spacing
                                        .flex()
                                        .flex_col()
                                        .justify_center();

                                    // Add section indicator based on section_style config
                                    match section_style {
                                        SectionStyle::Separators => {
                                            // Add top border for category separator (non-first items only)
                                            if is_section_start && idx > 0 {
                                                action_item = action_item
                                                    .border_t_1()
                                                    .border_color(separator_color);
                                            }
                                        }
                                        SectionStyle::Headers => {
                                            // Add section header text above the item
                                            if is_section_start {
                                                if let Some(ref label) = section_label {
                                                    action_item = action_item
                                                        .when(idx > 0, |d| {
                                                            d.border_t_1()
                                                                .border_color(separator_color)
                                                        })
                                                        .child(
                                                            div()
                                                                .px(px(16.0))
                                                                .pt(px(if idx > 0 {
                                                                    8.0
                                                                } else {
                                                                    4.0
                                                                }))
                                                                .pb(px(2.0))
                                                                .text_xs()
                                                                .font_weight(
                                                                    gpui::FontWeight::SEMIBOLD,
                                                                )
                                                                .text_color(dimmed_text)
                                                                .child(label.clone()),
                                                        );
                                                }
                                            }
                                        }
                                        SectionStyle::None => {
                                            // No section indicators
                                        }
                                    }

                                    // Inner row fills available height (minus 4px for py(2) top+bottom)
                                    let inner_row = div()
                                        .w_full()
                                        .flex_1() // Fill available height instead of fixed
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .px(px(item_spacing.item_padding_x))
                                        .rounded(px(SELECTION_RADIUS)) // Pill-style rounded corners
                                        .bg(if is_selected {
                                            selected_bg
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .hover(|s| s.bg(hover_bg))
                                        .cursor_pointer();

                                    // Get icon if config enables icons
                                    let show_icons = this.config.show_icons;
                                    let action_icon = action.icon;

                                    // Content row: optional icon + label + shortcuts
                                    let mut left_side =
                                        div().flex().flex_row().items_center().gap(px(12.0));

                                    // Add icon if enabled and present
                                    if show_icons {
                                        if let Some(icon) = action_icon {
                                            left_side = left_side.child(
                                                svg()
                                                    .external_path(icon.external_path())
                                                    .size(px(16.0))
                                                    .text_color(if is_selected {
                                                        primary_text
                                                    } else {
                                                        dimmed_text
                                                    }),
                                            );
                                        }
                                    }

                                    // Add title
                                    left_side = left_side.child(
                                        div()
                                            .text_color(title_color)
                                            .text_sm()
                                            .font_weight(if is_selected {
                                                gpui::FontWeight::MEDIUM
                                            } else {
                                                gpui::FontWeight::NORMAL
                                            })
                                            .child(title_str),
                                    );

                                    let content = div()
                                        .flex_1()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(left_side);

                                    // Right side: keyboard shortcuts as individual keycaps (Raycast-style)
                                    let content = if let Some(shortcut) = shortcut_opt {
                                        // Parse shortcut into individual keycaps
                                        let keycaps =
                                            ActionsDialog::parse_shortcut_keycaps(&shortcut);

                                        // Build keycap row
                                        let mut keycap_row =
                                            div().flex().flex_row().items_center().gap(px(3.));

                                        for keycap in keycaps {
                                            keycap_row = keycap_row.child(
                                                div()
                                                    .min_w(px(KEYCAP_MIN_WIDTH))
                                                    .h(px(KEYCAP_HEIGHT))
                                                    .px(px(6.))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .bg(keycap_bg)
                                                    .border_1()
                                                    .border_color(keycap_border)
                                                    .rounded(px(5.))
                                                    .text_xs()
                                                    .text_color(shortcut_color)
                                                    .child(keycap),
                                            );
                                        }

                                        content.child(keycap_row)
                                    } else {
                                        content
                                    };

                                    // Build final action item with inner row
                                    action_item = action_item.child(inner_row.child(content));

                                    items.push(action_item);
                                }
                            }
                        }
                        items
                    },
                ),
            )
            .flex_1()
            .w_full()
            .track_scroll(&self.scroll_handle);

            // Wrap uniform_list in a relative container with scrollbar overlay
            // NOTE: The wrapper needs flex + h_full for uniform_list to properly calculate visible range
            // overflow_hidden clips children to parent bounds (including rounded corners)
            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .h_full()
                .overflow_hidden()
                .child(list)
                .child(scrollbar)
                .into_any_element()
        };

        // Use helper method for container colors
        let (main_bg, container_border, container_text) = self.get_container_colors(&colors);

        // Calculate dynamic height based on number of items
        // Each item is ACTION_ITEM_HEIGHT, plus search box height (SEARCH_INPUT_HEIGHT), plus padding
        // When hide_search is true, we don't include the search box height
        // Now also includes HEADER_HEIGHT when context_title is set
        // NOTE: Add border_thin * 2 for border (top + bottom from .border_1()) to prevent
        // content from being clipped and causing unnecessary scrolling
        let num_items = self.filtered_actions.len();
        let search_box_height = if self.hide_search {
            0.0
        } else {
            SEARCH_INPUT_HEIGHT
        };
        let header_height = if self.context_title.is_some() {
            HEADER_HEIGHT
        } else {
            0.0
        };
        let border_height = visual.border_thin * 2.0; // top + bottom border
        let items_height = (num_items as f32 * ACTION_ITEM_HEIGHT)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let total_height = items_height + search_box_height + header_height + border_height;

        // Build header row (section header style - non-interactive label)
        // Styled to match render_section_header() from list_item.rs:
        // - Smaller font (text_xs)
        // - Semibold weight
        // - Dimmed color (visually distinct from actionable items)
        let header_container = self.context_title.as_ref().map(|title| {
            let header_text = if self.design_variant == DesignVariant::Default {
                rgb(self.theme.colors.text.dimmed)
            } else {
                rgb(colors.text_dimmed)
            };
            let header_border = if self.design_variant == DesignVariant::Default {
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x40))
            } else {
                rgba(hex_with_alpha(colors.border, 0x40))
            };

            div()
                .w_full()
                .h(px(HEADER_HEIGHT))
                .px(px(16.0)) // Match section header padding from list_item.rs
                .pt(px(8.0)) // Top padding for visual separation
                .pb(px(4.0)) // Bottom padding
                .flex()
                .flex_col()
                .justify_center()
                .border_b_1()
                .border_color(header_border)
                .child(
                    div()
                        .text_xs() // Smaller font like section headers
                        .font_weight(gpui::FontWeight::SEMIBOLD) // Semibold like section headers
                        .text_color(header_text)
                        .child(title.clone()),
                )
        });

        // Main overlay popup container
        // Fixed width, dynamic height based on content, rounded corners, shadow
        // NOTE: Using visual.radius_lg from design tokens for consistency with child item rounding
        //
        // VIBRANCY: When vibrancy is enabled, do NOT apply a background here.
        // The window's native vibrancy blur provides the frosted glass effect.
        // Only apply a background when vibrancy is disabled for a solid fallback.
        let use_vibrancy = self.theme.is_vibrancy_enabled();

        // Build footer with keyboard hints (if enabled)
        let footer_height = if self.config.show_footer { 32.0 } else { 0.0 };
        let footer_container = if self.config.show_footer {
            let footer_text = if self.design_variant == DesignVariant::Default {
                rgb(self.theme.colors.text.dimmed)
            } else {
                rgb(colors.text_dimmed)
            };
            let footer_border = if self.design_variant == DesignVariant::Default {
                rgba(hex_with_alpha(self.theme.colors.ui.border, 0x40))
            } else {
                rgba(hex_with_alpha(colors.border, 0x40))
            };

            Some(
                div()
                    .w_full()
                    .h(px(32.0))
                    .px(px(16.0))
                    .border_t_1()
                    .border_color(footer_border)
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .text_xs()
                    .text_color(footer_text)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("↑↓")
                            .child("Navigate"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("↵")
                            .child("Select"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child("esc")
                            .child("Close"),
                    ),
            )
        } else {
            None
        };

        // Recalculate total height including footer
        let total_height = total_height + footer_height;

        // Get search position from config
        let search_at_top = matches!(self.config.search_position, SearchPosition::Top);
        let show_search =
            !matches!(self.config.search_position, SearchPosition::Hidden) && !self.hide_search;

        // Modify input_container for top position (border at bottom instead of top)
        let input_container_top = if search_at_top && show_search {
            Some(
                div()
                    .w(px(POPUP_WIDTH))
                    .min_w(px(POPUP_WIDTH))
                    .max_w(px(POPUP_WIDTH))
                    .h(px(SEARCH_INPUT_HEIGHT))
                    .min_h(px(SEARCH_INPUT_HEIGHT))
                    .max_h(px(SEARCH_INPUT_HEIGHT))
                    .overflow_hidden()
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.item_padding_y + 2.0))
                    .bg(search_box_bg)
                    .border_b_1() // Border at bottom for top-positioned search
                    .border_color(border_color)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.gap_md))
                    .child(
                        div()
                            .w(px(24.0))
                            .min_w(px(24.0))
                            .text_color(dimmed_text)
                            .text_xs()
                            .child("⌘K"),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .w(px(240.0))
                            .min_w(px(240.0))
                            .max_w(px(240.0))
                            .h(px(28.0))
                            .min_h(px(28.0))
                            .max_h(px(28.0))
                            .overflow_hidden()
                            .px(px(spacing.padding_sm))
                            .py(px(spacing.padding_xs))
                            .bg(if self.design_variant == DesignVariant::Default {
                                rgba(hex_with_alpha(
                                    self.theme.colors.background.main,
                                    if self.search_text.is_empty() {
                                        0x20
                                    } else {
                                        0x40
                                    },
                                ))
                            } else {
                                rgba(hex_with_alpha(
                                    colors.background,
                                    if self.search_text.is_empty() {
                                        0x20
                                    } else {
                                        0x40
                                    },
                                ))
                            })
                            .rounded(px(visual.radius_sm))
                            .border_1()
                            .border_color(if !self.search_text.is_empty() {
                                focus_border_color
                            } else {
                                border_color
                            })
                            .flex()
                            .flex_row()
                            .items_center()
                            .text_sm()
                            .text_color(if self.search_text.is_empty() {
                                dimmed_text
                            } else {
                                primary_text
                            })
                            .when(self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .mr(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            })
                            .child(search_display.clone())
                            .when(!self.search_text.is_empty(), |d| {
                                d.child(
                                    div()
                                        .w(px(2.))
                                        .h(px(16.))
                                        .ml(px(2.))
                                        .rounded(px(1.))
                                        .when(self.cursor_visible, |d| d.bg(accent_color)),
                                )
                            }),
                    ),
            )
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .w(px(POPUP_WIDTH))
            .h(px(total_height)) // Use calculated height including footer
            .when(!use_vibrancy, |d| d.bg(main_bg)) // Only apply bg when vibrancy disabled
            .rounded(px(visual.radius_lg))
            .shadow(Self::create_popup_shadow())
            .border_1()
            .border_color(container_border)
            .overflow_hidden()
            .text_color(container_text)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            // NOTE: No on_key_down here - parent handles all keyboard input
            // Search input at top (if config.search_position == Top)
            .when_some(input_container_top, |d, input| d.child(input))
            // Header row (if context_title is set)
            .when_some(header_container, |d, header| d.child(header))
            // Actions list
            .child(actions_container)
            // Search input at bottom (if config.search_position == Bottom)
            .when(show_search && !search_at_top, |d| d.child(input_container))
            // Footer with keyboard hints (if config.show_footer)
            .when_some(footer_container, |d, footer| d.child(footer))
    }
}

</file>

<file path="src/actions/constants.rs">
//! Actions dialog constants
//!
//! Overlay popup dimensions and styling constants used by the ActionsDialog.

/// Popup width for the actions dialog
pub const POPUP_WIDTH: f32 = 320.0;

/// Maximum height for the actions dialog popup
pub const POPUP_MAX_HEIGHT: f32 = 400.0;

/// Fixed height for action items (required for uniform_list virtualization)
/// Standardized to 44px for consistent touch targets (matches iOS guidelines, Notes panel)
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;

/// Fixed height for the search input row (matches Notes panel PANEL_SEARCH_HEIGHT)
pub const SEARCH_INPUT_HEIGHT: f32 = 44.0;

/// Width of the left accent bar for selected items (legacy, kept for reference)
pub const ACCENT_BAR_WIDTH: f32 = 3.0;

/// Height for the header row showing context title (matches section header style)
pub const HEADER_HEIGHT: f32 = 24.0;

/// Horizontal inset for action rows (creates rounded pill appearance)
pub const ACTION_ROW_INSET: f32 = 6.0;

/// Corner radius for selected row background (pill style)
pub const SELECTION_RADIUS: f32 = 8.0;

/// Minimum width for keycap badges
pub const KEYCAP_MIN_WIDTH: f32 = 22.0;

/// Height for keycap badges
pub const KEYCAP_HEIGHT: f32 = 22.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_constants() {
        assert_eq!(POPUP_WIDTH, 320.0);
        assert_eq!(POPUP_MAX_HEIGHT, 400.0);
    }

    #[test]
    fn test_action_item_height_constant() {
        // Fixed height is required for uniform_list virtualization
        // Standardized to 44px for consistent touch targets (matches iOS guidelines)
        assert_eq!(ACTION_ITEM_HEIGHT, 44.0);
        // Ensure item height is positive and reasonable
        const _: () = assert!(ACTION_ITEM_HEIGHT > 0.0);
        const _: () = assert!(ACTION_ITEM_HEIGHT < POPUP_MAX_HEIGHT);
    }

    #[test]
    fn test_max_visible_items() {
        // Calculate max visible items that can fit in the popup
        // This helps verify scroll virtualization is worthwhile
        let max_visible = (POPUP_MAX_HEIGHT / ACTION_ITEM_HEIGHT) as usize;
        // With 400px max height and 44px items, ~9 items fit
        assert!(max_visible >= 8, "Should fit at least 8 items");
        assert!(max_visible <= 15, "Sanity check on max visible");
    }
}

</file>

<file path="src/actions/types.rs">
//! Action types and data structures
//!
//! Core types for the actions system including Action, ActionCategory, and ScriptInfo.

use crate::designs::icon_variations::IconName;
use std::sync::Arc;

/// Callback for action selection
/// Signature: (action_id: String)
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Information about the currently focused/selected script
/// Used for context-aware actions in the actions dialog
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    /// Display name of the script
    pub name: String,
    // Note: path is written during construction for completeness but currently
    // action handlers read directly from ProtocolAction. Kept for API consistency.
    #[allow(dead_code)]
    /// Full path to the script file
    pub path: String,
    /// Whether this is a real script file (true) or a built-in command (false)
    /// Built-in commands (like Clipboard History, App Launcher) have limited actions
    pub is_script: bool,
    /// Whether this is a scriptlet (snippet from markdown file)
    /// Scriptlets have their own actions (Edit Scriptlet, etc.) that work with the markdown file
    pub is_scriptlet: bool,
    /// The verb to use for the primary action (e.g., "Run", "Launch", "Switch to")
    /// Defaults to "Run" for scripts
    pub action_verb: String,
    /// Current keyboard shortcut assigned to this script/item (if any)
    /// Used to determine which shortcut actions to show in the actions menu
    pub shortcut: Option<String>,
    /// Current alias assigned to this script/item (if any)
    /// Used to determine which alias actions to show in the actions menu
    pub alias: Option<String>,
    /// Whether this item appears in the "Suggested" section (has frecency data)
    /// Used to show/hide the "Reset Ranking" action
    pub is_suggested: bool,
    /// The frecency path used to track this item's usage
    /// Used by "Reset Ranking" to know which frecency entry to remove
    pub frecency_path: Option<String>,
}

impl ScriptInfo {
    /// Create a ScriptInfo for a real script file
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo for a real script file with shortcut info
    #[allow(dead_code)]
    pub fn with_shortcut(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo for a scriptlet (snippet from markdown file)
    /// Scriptlets have their own actions that work with the source markdown file
    pub fn scriptlet(
        name: impl Into<String>,
        markdown_path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: markdown_path.into(),
            is_script: false,
            is_scriptlet: true,
            action_verb: "Run".to_string(),
            shortcut,
            alias,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo for a real script file with shortcut and alias info
    #[allow(dead_code)]
    pub fn with_shortcut_and_alias(
        name: impl Into<String>,
        path: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script: true,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut,
            alias,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo for a built-in command (not a real script)
    /// Built-ins have limited actions (no edit, view logs, reveal in finder, copy path, configure shortcut)
    #[allow(dead_code)]
    pub fn builtin(name: impl Into<String>) -> Self {
        ScriptInfo {
            name: name.into(),
            path: String::new(),
            is_script: false,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo with explicit is_script flag and custom action verb
    #[allow(dead_code)]
    pub fn with_is_script(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: "Run".to_string(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo with all options including custom action verb
    pub fn with_action_verb(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: action_verb.into(),
            shortcut: None,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo with all options including custom action verb and shortcut
    #[allow(dead_code)]
    pub fn with_action_verb_and_shortcut(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: action_verb.into(),
            shortcut,
            alias: None,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Create a ScriptInfo with all options including custom action verb, shortcut, and alias
    #[allow(dead_code)]
    pub fn with_all(
        name: impl Into<String>,
        path: impl Into<String>,
        is_script: bool,
        action_verb: impl Into<String>,
        shortcut: Option<String>,
        alias: Option<String>,
    ) -> Self {
        ScriptInfo {
            name: name.into(),
            path: path.into(),
            is_script,
            is_scriptlet: false,
            action_verb: action_verb.into(),
            shortcut,
            alias,
            is_suggested: false,
            frecency_path: None,
        }
    }

    /// Set whether this item is suggested (has frecency data) and its frecency path
    pub fn with_frecency(mut self, is_suggested: bool, frecency_path: Option<String>) -> Self {
        self.is_suggested = is_suggested;
        self.frecency_path = frecency_path;
        self
    }
}

/// Available actions in the actions menu
///
/// Note: The `has_action` and `value` fields are populated from ProtocolAction
/// for consistency, but the actual routing logic reads from the original
/// ProtocolAction. These fields are kept for future use cases where Action
/// might need independent behavior.
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ActionCategory,
    /// Optional keyboard shortcut hint (e.g., "⌘E")
    pub shortcut: Option<String>,
    /// If true, send ActionTriggered to SDK; if false, submit value directly
    #[allow(dead_code)]
    pub has_action: bool,
    /// Optional value to submit when action is triggered
    #[allow(dead_code)]
    pub value: Option<String>,
    /// Optional icon to display next to the action
    pub icon: Option<IconName>,
    /// Section/group name for display (used with SectionStyle::Headers)
    pub section: Option<String>,
}

/// Configuration for how the search input is positioned
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchPosition {
    /// Search input at top (AI chat style - list grows downward)
    Top,
    /// Search input at bottom (main menu style - list grows upward)
    #[default]
    Bottom,
    /// No search input (external search handling)
    Hidden,
}

/// Configuration for how sections/categories are displayed
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SectionStyle {
    /// Show text headers for sections (AI chat style)
    Headers,
    /// Show subtle separators between categories (main menu style)
    #[default]
    Separators,
    /// No section indicators
    None,
}

/// Configuration for dialog anchor position during resize
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnchorPosition {
    /// Dialog grows/shrinks from top (content pinned to top)
    Top,
    /// Dialog grows/shrinks from bottom (content pinned to bottom)
    #[default]
    Bottom,
}

/// Complete configuration for ActionsDialog appearance and behavior
#[allow(dead_code)] // Public API - will be used by AI window integration
#[derive(Debug, Clone, Default)]
pub struct ActionsDialogConfig {
    /// Position of search input
    pub search_position: SearchPosition,
    /// How to display section/category divisions
    pub section_style: SectionStyle,
    /// Which edge the dialog anchors to during resize
    pub anchor: AnchorPosition,
    /// Whether to show icons for actions (if available)
    pub show_icons: bool,
    /// Whether to show the footer with keyboard hints
    pub show_footer: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    ScriptContext, // Actions specific to the focused script
    #[allow(dead_code)]
    ScriptOps, // Edit, Create, Delete script operations (reserved for future use)
    #[allow(dead_code)]
    GlobalOps, // Settings, Quit, etc.
}

impl Action {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: Option<String>,
        category: ActionCategory,
    ) -> Self {
        Action {
            id: id.into(),
            title: title.into(),
            description,
            category,
            shortcut: None,
            has_action: false,
            value: None,
            icon: None,
            section: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    #[allow(dead_code)] // Public API - used by get_ai_command_bar_actions
    pub fn with_icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    #[allow(dead_code)] // Public API - used by get_ai_command_bar_actions
    pub fn with_section(mut self, section: impl Into<String>) -> Self {
        self.section = Some(section.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_info_creation() {
        let script = ScriptInfo::new("test-script", "/path/to/test-script.ts");
        assert_eq!(script.name, "test-script");
        assert_eq!(script.path, "/path/to/test-script.ts");
        assert!(script.is_script);
        assert!(!script.is_scriptlet);
        assert!(script.shortcut.is_none());
        assert!(script.alias.is_none());
    }

    #[test]
    fn test_script_info_with_shortcut() {
        let script = ScriptInfo::with_shortcut(
            "test-script",
            "/path/to/test-script.ts",
            Some("cmd+shift+t".to_string()),
        );
        assert_eq!(script.name, "test-script");
        assert!(script.is_script);
        assert!(!script.is_scriptlet);
        assert_eq!(script.shortcut, Some("cmd+shift+t".to_string()));
    }

    #[test]
    fn test_script_info_scriptlet() {
        let scriptlet = ScriptInfo::scriptlet(
            "Open GitHub",
            "/path/to/url.md#open-github",
            Some("cmd+g".to_string()),
            Some("gh".to_string()),
        );
        assert_eq!(scriptlet.name, "Open GitHub");
        assert_eq!(scriptlet.path, "/path/to/url.md#open-github");
        assert!(!scriptlet.is_script);
        assert!(scriptlet.is_scriptlet);
        assert_eq!(scriptlet.shortcut, Some("cmd+g".to_string()));
        assert_eq!(scriptlet.alias, Some("gh".to_string()));
        assert_eq!(scriptlet.action_verb, "Run");
    }

    #[test]
    fn test_script_info_builtin() {
        let builtin = ScriptInfo::builtin("Clipboard History");
        assert_eq!(builtin.name, "Clipboard History");
        assert_eq!(builtin.path, "");
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
        assert!(builtin.shortcut.is_none());
        assert!(builtin.alias.is_none());
    }

    #[test]
    fn test_script_info_with_is_script() {
        let script = ScriptInfo::with_is_script("my-script", "/path/to/script.ts", true);
        assert!(script.is_script);
        assert!(!script.is_scriptlet);
        assert!(script.shortcut.is_none());

        let builtin = ScriptInfo::with_is_script("App Launcher", "", false);
        assert!(!builtin.is_script);
        assert!(!builtin.is_scriptlet);
    }

    #[test]
    fn test_script_info_with_action_verb_and_shortcut() {
        let script = ScriptInfo::with_action_verb_and_shortcut(
            "test",
            "/path",
            true,
            "Launch",
            Some("cmd+k".to_string()),
        );
        assert_eq!(script.action_verb, "Launch");
        assert!(!script.is_scriptlet);
        assert_eq!(script.shortcut, Some("cmd+k".to_string()));
    }

    #[test]
    fn test_action_with_shortcut() {
        let action =
            Action::new("test", "Test Action", None, ActionCategory::GlobalOps).with_shortcut("⌘T");
        assert_eq!(action.shortcut, Some("⌘T".to_string()));
    }

    #[test]
    fn test_action_new_defaults() {
        let action = Action::new(
            "id",
            "title",
            Some("desc".to_string()),
            ActionCategory::ScriptContext,
        );
        assert_eq!(action.id, "id");
        assert_eq!(action.title, "title");
        assert_eq!(action.description, Some("desc".to_string()));
        assert_eq!(action.category, ActionCategory::ScriptContext);
        assert!(action.shortcut.is_none());
    }

    #[test]
    fn test_script_info_with_shortcut_and_alias() {
        let script = ScriptInfo::with_shortcut_and_alias(
            "test-script",
            "/path/to/test-script.ts",
            Some("cmd+shift+t".to_string()),
            Some("ts".to_string()),
        );
        assert_eq!(script.name, "test-script");
        assert_eq!(script.shortcut, Some("cmd+shift+t".to_string()));
        assert_eq!(script.alias, Some("ts".to_string()));
    }

    #[test]
    fn test_script_info_with_all() {
        let script = ScriptInfo::with_all(
            "App Launcher",
            "builtin:app-launcher",
            false,
            "Open",
            Some("cmd+space".to_string()),
            Some("apps".to_string()),
        );
        assert_eq!(script.name, "App Launcher");
        assert_eq!(script.path, "builtin:app-launcher");
        assert!(!script.is_script);
        assert_eq!(script.action_verb, "Open");
        assert_eq!(script.shortcut, Some("cmd+space".to_string()));
        assert_eq!(script.alias, Some("apps".to_string()));
    }

    #[test]
    fn test_script_info_with_frecency() {
        // Test with_frecency builder method
        let script = ScriptInfo::new("test-script", "/path/to/script.ts")
            .with_frecency(true, Some("/path/to/script.ts".to_string()));

        assert!(script.is_suggested);
        assert_eq!(script.frecency_path, Some("/path/to/script.ts".to_string()));
    }

    #[test]
    fn test_script_info_default_frecency_values() {
        // Test that default values are correct (not suggested, no frecency path)
        let script = ScriptInfo::new("test-script", "/path/to/script.ts");
        assert!(!script.is_suggested);
        assert!(script.frecency_path.is_none());

        let scriptlet = ScriptInfo::scriptlet("Open GitHub", "/path/to/url.md", None, None);
        assert!(!scriptlet.is_suggested);
        assert!(scriptlet.frecency_path.is_none());

        let builtin = ScriptInfo::builtin("Clipboard History");
        assert!(!builtin.is_suggested);
        assert!(builtin.frecency_path.is_none());
    }

    #[test]
    fn test_script_info_frecency_chaining() {
        // Test that with_frecency can be chained with other constructors
        let script = ScriptInfo::with_shortcut_and_alias(
            "test-script",
            "/path/to/test.ts",
            Some("cmd+t".to_string()),
            Some("ts".to_string()),
        )
        .with_frecency(true, Some("frecency:path".to_string()));

        // Original fields preserved
        assert_eq!(script.shortcut, Some("cmd+t".to_string()));
        assert_eq!(script.alias, Some("ts".to_string()));

        // Frecency fields set
        assert!(script.is_suggested);
        assert_eq!(script.frecency_path, Some("frecency:path".to_string()));
    }
}

</file>

<file path="src/notes/actions_panel.rs">
//! Notes Actions Panel
//!
//! Modal overlay panel triggered by Cmd+K in the Notes window.
//! Provides searchable action list for note operations.
//!
//! ## Actions
//! - New Note (⌘N) - Create a new note
//! - Duplicate Note (⌘D) - Create a copy of the current note
//! - Browse Notes (⌘P) - Open note browser/picker
//! - Find in Note (⌘F) - Search within current note
//! - Copy Note As... (⇧⌘C) - Copy note in a chosen format
//! - Copy Deeplink (⇧⌘D) - Copy a deeplink to the note
//! - Create Quicklink (⇧⌘L) - Copy a quicklink to the note
//! - Export... (⇧⌘E) - Export note content
//! - Move List Item Up (⌃⌘↑) - Reorder notes list (disabled)
//! - Move List Item Down (⌃⌘↓) - Reorder notes list (disabled)
//! - Format... (⇧⌘T) - Formatting commands
//!
//! ## Keyboard Navigation
//! - Arrow Up/Down: Navigate actions
//! - Enter: Execute selected action
//! - Escape: Close panel
//! - Type to search/filter actions

use crate::designs::icon_variations::IconName;
use gpui::{
    div, point, prelude::*, px, rgba, svg, uniform_list, App, BoxShadow, Context, FocusHandle,
    Focusable, Hsla, MouseButton, Render, ScrollStrategy, SharedString, UniformListScrollHandle,
    Window,
};
use gpui_component::theme::{ActiveTheme, Theme};
use std::sync::Arc;
use tracing::debug;

/// Callback type for action execution
/// The String parameter is the action ID
pub type NotesActionCallback = Arc<dyn Fn(NotesAction) + Send + Sync>;

/// Available actions in the Notes actions panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotesAction {
    /// Create a new note
    NewNote,
    /// Duplicate the current note
    DuplicateNote,
    /// Open the note browser/picker
    BrowseNotes,
    /// Search within the current note
    FindInNote,
    /// Copy note content as a formatted export
    CopyNoteAs,
    /// Copy deeplink to the current note
    CopyDeeplink,
    /// Copy quicklink to the current note
    CreateQuicklink,
    /// Export note content
    Export,
    /// Move list item up (disabled placeholder)
    MoveListItemUp,
    /// Move list item down (disabled placeholder)
    MoveListItemDown,
    /// Open formatting commands
    Format,
    /// Enable auto-sizing (window grows/shrinks with content)
    EnableAutoSizing,
    /// Panel was cancelled (Escape pressed)
    Cancel,
}

impl NotesAction {
    /// Get all available actions (excluding Cancel)
    pub fn all() -> &'static [NotesAction] {
        &[
            NotesAction::NewNote,
            NotesAction::DuplicateNote,
            NotesAction::BrowseNotes,
            NotesAction::FindInNote,
            NotesAction::CopyNoteAs,
            NotesAction::CopyDeeplink,
            NotesAction::CreateQuicklink,
            NotesAction::Export,
            NotesAction::MoveListItemUp,
            NotesAction::MoveListItemDown,
            NotesAction::Format,
        ]
    }

    /// Get the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "New Note",
            NotesAction::DuplicateNote => "Duplicate Note",
            NotesAction::BrowseNotes => "Browse Notes",
            NotesAction::FindInNote => "Find in Note",
            NotesAction::CopyNoteAs => "Copy Note As...",
            NotesAction::CopyDeeplink => "Copy Deeplink",
            NotesAction::CreateQuicklink => "Create Quicklink",
            NotesAction::Export => "Export...",
            NotesAction::MoveListItemUp => "Move List Item Up",
            NotesAction::MoveListItemDown => "Move List Item Down",
            NotesAction::Format => "Format...",
            NotesAction::EnableAutoSizing => "Enable Auto-Sizing",
            NotesAction::Cancel => "Cancel",
        }
    }

    /// Get the keyboard shortcut key (without modifier)
    pub fn shortcut_key(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "N",
            NotesAction::DuplicateNote => "D",
            NotesAction::BrowseNotes => "P",
            NotesAction::FindInNote => "F",
            NotesAction::CopyNoteAs => "C",
            NotesAction::CopyDeeplink => "D",
            NotesAction::CreateQuicklink => "L",
            NotesAction::Export => "E",
            NotesAction::MoveListItemUp => "↑",
            NotesAction::MoveListItemDown => "↓",
            NotesAction::Format => "T",
            NotesAction::EnableAutoSizing => "A",
            NotesAction::Cancel => "Esc",
        }
    }

    /// Get shortcut keys for keycap rendering
    pub fn shortcut_keys(&self) -> &'static [&'static str] {
        const CMD_N: [&str; 2] = ["⌘", "N"];
        const CMD_D: [&str; 2] = ["⌘", "D"];
        const CMD_P: [&str; 2] = ["⌘", "P"];
        const CMD_F: [&str; 2] = ["⌘", "F"];
        const SHIFT_CMD_C: [&str; 3] = ["⇧", "⌘", "C"];
        const SHIFT_CMD_D: [&str; 3] = ["⇧", "⌘", "D"];
        const SHIFT_CMD_L: [&str; 3] = ["⇧", "⌘", "L"];
        const SHIFT_CMD_E: [&str; 3] = ["⇧", "⌘", "E"];
        const CTRL_CMD_UP: [&str; 3] = ["⌃", "⌘", "↑"];
        const CTRL_CMD_DOWN: [&str; 3] = ["⌃", "⌘", "↓"];
        const SHIFT_CMD_T: [&str; 3] = ["⇧", "⌘", "T"];
        const CMD_A: [&str; 2] = ["⌘", "A"];
        const ESC: [&str; 1] = ["Esc"];

        match self {
            NotesAction::NewNote => &CMD_N,
            NotesAction::DuplicateNote => &CMD_D,
            NotesAction::BrowseNotes => &CMD_P,
            NotesAction::FindInNote => &CMD_F,
            NotesAction::CopyNoteAs => &SHIFT_CMD_C,
            NotesAction::CopyDeeplink => &SHIFT_CMD_D,
            NotesAction::CreateQuicklink => &SHIFT_CMD_L,
            NotesAction::Export => &SHIFT_CMD_E,
            NotesAction::MoveListItemUp => &CTRL_CMD_UP,
            NotesAction::MoveListItemDown => &CTRL_CMD_DOWN,
            NotesAction::Format => &SHIFT_CMD_T,
            NotesAction::EnableAutoSizing => &CMD_A,
            NotesAction::Cancel => &ESC,
        }
    }

    /// Get the formatted shortcut display string
    pub fn shortcut_display(&self) -> String {
        if self.shortcut_keys().is_empty() {
            return String::new();
        }

        self.shortcut_keys().join("")
    }

    /// Get the icon for this action (uses local IconName from designs module)
    pub fn icon(&self) -> IconName {
        match self {
            NotesAction::NewNote => IconName::Plus,
            NotesAction::DuplicateNote => IconName::Copy,
            NotesAction::BrowseNotes => IconName::FolderOpen,
            NotesAction::FindInNote => IconName::MagnifyingGlass,
            NotesAction::CopyNoteAs => IconName::Copy,
            NotesAction::CopyDeeplink => IconName::ArrowRight,
            NotesAction::CreateQuicklink => IconName::Star,
            NotesAction::Export => IconName::ArrowRight,
            NotesAction::MoveListItemUp => IconName::ArrowUp,
            NotesAction::MoveListItemDown => IconName::ArrowDown,
            NotesAction::Format => IconName::Code,
            NotesAction::EnableAutoSizing => IconName::ArrowRight,
            NotesAction::Cancel => IconName::Close,
        }
    }

    /// Get action ID for lookup
    pub fn id(&self) -> &'static str {
        match self {
            NotesAction::NewNote => "new_note",
            NotesAction::DuplicateNote => "duplicate_note",
            NotesAction::BrowseNotes => "browse_notes",
            NotesAction::FindInNote => "find_in_note",
            NotesAction::CopyNoteAs => "copy_note_as",
            NotesAction::CopyDeeplink => "copy_deeplink",
            NotesAction::CreateQuicklink => "create_quicklink",
            NotesAction::Export => "export",
            NotesAction::MoveListItemUp => "move_list_item_up",
            NotesAction::MoveListItemDown => "move_list_item_down",
            NotesAction::Format => "format",
            NotesAction::EnableAutoSizing => "enable_auto_sizing",
            NotesAction::Cancel => "cancel",
        }
    }
}

/// Action list sections for visual grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotesActionSection {
    Primary,
    Actions,
    Move,
    Format,
    Utility,
}

impl NotesActionSection {
    fn for_action(action: NotesAction) -> Self {
        match action {
            NotesAction::NewNote | NotesAction::DuplicateNote | NotesAction::BrowseNotes => {
                NotesActionSection::Primary
            }
            NotesAction::FindInNote
            | NotesAction::CopyNoteAs
            | NotesAction::CopyDeeplink
            | NotesAction::CreateQuicklink
            | NotesAction::Export => NotesActionSection::Actions,
            NotesAction::MoveListItemUp | NotesAction::MoveListItemDown => NotesActionSection::Move,
            NotesAction::Format => NotesActionSection::Format,
            NotesAction::EnableAutoSizing | NotesAction::Cancel => NotesActionSection::Utility,
        }
    }
}

/// Action entry with enabled state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotesActionItem {
    pub action: NotesAction,
    pub enabled: bool,
}

impl NotesActionItem {
    fn section(&self) -> NotesActionSection {
        NotesActionSection::for_action(self.action)
    }
}

/// Panel dimensions and styling constants (matches main ActionsDialog)
pub const PANEL_WIDTH: f32 = 320.0;
/// Standardized to match main ActionsDialog POPUP_MAX_HEIGHT (was 580.0)
pub const PANEL_MAX_HEIGHT: f32 = 400.0;
pub const PANEL_CORNER_RADIUS: f32 = 12.0;
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;
pub const PANEL_SEARCH_HEIGHT: f32 = 44.0;
pub const PANEL_BORDER_HEIGHT: f32 = 2.0;
/// Horizontal inset for action rows (creates rounded pill appearance)
pub const ACTION_ROW_INSET: f32 = 6.0;
/// Corner radius for selected row background
pub const SELECTION_RADIUS: f32 = 8.0;

pub fn panel_height_for_rows(row_count: usize) -> f32 {
    let items_height = (row_count as f32 * ACTION_ITEM_HEIGHT)
        .min(PANEL_MAX_HEIGHT - (PANEL_SEARCH_HEIGHT + 16.0));
    items_height + PANEL_SEARCH_HEIGHT + PANEL_BORDER_HEIGHT
}

/// Notes Actions Panel - Modal overlay for note operations
pub struct NotesActionsPanel {
    /// Available actions
    actions: Vec<NotesActionItem>,
    /// Filtered action indices
    filtered_indices: Vec<usize>,
    /// Currently selected index (within filtered)
    selected_index: usize,
    /// Search text
    search_text: String,
    /// Focus handle
    focus_handle: FocusHandle,
    /// Callback for action selection
    on_action: NotesActionCallback,
    /// Scroll handle for virtualization
    scroll_handle: UniformListScrollHandle,
    /// Cursor blink visibility
    cursor_visible: bool,
}

impl NotesActionsPanel {
    /// Create a new NotesActionsPanel
    pub fn new(
        focus_handle: FocusHandle,
        actions: Vec<NotesActionItem>,
        on_action: NotesActionCallback,
    ) -> Self {
        let filtered_indices: Vec<usize> = (0..actions.len()).collect();
        let selected_index = actions.iter().position(|item| item.enabled).unwrap_or(0);

        debug!(action_count = actions.len(), "Notes actions panel created");

        Self {
            actions,
            filtered_indices,
            selected_index,
            search_text: String::new(),
            focus_handle,
            on_action,
            scroll_handle: UniformListScrollHandle::new(),
            cursor_visible: true,
        }
    }

    /// Set cursor visibility (for blink animation)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        self.move_selection(-1, cx);
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        self.move_selection(1, cx);
    }

    /// Submit the selected action
    pub fn submit_selected(&mut self) {
        if let Some(&action_idx) = self.filtered_indices.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                if action.enabled {
                    debug!(action = ?action.action, "Notes action selected");
                    (self.on_action)(action.action);
                }
            }
        }
    }

    /// Cancel and close
    pub fn cancel(&mut self) {
        debug!("Notes actions panel cancelled");
        (self.on_action)(NotesAction::Cancel);
    }

    /// Get currently selected action
    pub fn get_selected_action(&self) -> Option<NotesAction> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx))
            .and_then(|item| {
                if item.enabled {
                    Some(item.action)
                } else {
                    None
                }
            })
    }

    /// Refilter actions based on search text
    fn refilter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_indices = (0..self.actions.len()).collect();
        } else {
            let search_lower = self.search_text.to_lowercase();
            self.filtered_indices = self
                .actions
                .iter()
                .enumerate()
                .filter(|(_, action)| action.action.label().to_lowercase().contains(&search_lower))
                .map(|(idx, _)| idx)
                .collect();
        }

        self.ensure_valid_selection();

        // Scroll to keep selection visible
        if !self.filtered_indices.is_empty() {
            self.scroll_handle
                .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
        }
    }

    fn ensure_valid_selection(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected_index = 0;
            return;
        }

        if self.selected_index >= self.filtered_indices.len()
            || !self.is_selectable(self.selected_index)
        {
            if let Some(index) =
                (0..self.filtered_indices.len()).find(|&idx| self.is_selectable(idx))
            {
                self.selected_index = index;
            } else {
                self.selected_index = 0;
            }
        }
    }

    fn is_selectable(&self, filtered_idx: usize) -> bool {
        self.filtered_indices
            .get(filtered_idx)
            .and_then(|&idx| self.actions.get(idx))
            .map(|item| item.enabled)
            .unwrap_or(false)
    }

    fn move_selection(&mut self, delta: i32, cx: &mut Context<Self>) {
        let filtered_len = self.filtered_indices.len();
        if filtered_len == 0 {
            return;
        }

        let mut next_index = self.selected_index as i32;
        loop {
            next_index += delta;
            if next_index < 0 || next_index >= filtered_len as i32 {
                break;
            }

            let next = next_index as usize;
            if self.is_selectable(next) {
                self.selected_index = next;
                self.scroll_handle
                    .scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
                cx.notify();
                return;
            }
        }
    }

    // =====================================================
    // Vibrancy Helper Functions
    // =====================================================
    // NOTE: hex_to_rgba_with_opacity moved to crate::ui_foundation (centralized)

    /// Get background color with vibrancy opacity applied
    fn get_vibrancy_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.main;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.main,
        ))
    }

    /// Get search box background with vibrancy opacity
    fn get_vibrancy_search_background() -> gpui::Rgba {
        let sk_theme = crate::theme::load_theme();
        let opacity = sk_theme.get_opacity();
        let bg_hex = sk_theme.colors.background.search_box;
        rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
            bg_hex,
            opacity.search_box,
        ))
    }

    /// Create box shadow for the overlay
    fn create_shadow() -> Vec<BoxShadow> {
        vec![
            BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.3,
                },
                offset: point(px(0.0), px(4.0)),
                blur_radius: px(16.0),
                spread_radius: px(0.0),
            },
            BoxShadow {
                color: Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 0.15,
                },
                offset: point(px(0.0), px(8.0)),
                blur_radius: px(32.0),
                spread_radius: px(-4.0),
            },
        ]
    }
}

impl Focusable for NotesActionsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesActionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Vibrancy-aware colors using Script Kit theme hex values
        let bg_color = Self::get_vibrancy_background();
        let search_bg_color = Self::get_vibrancy_search_background();
        let border_color = theme.border;
        let text_primary = theme.foreground;
        let text_muted = theme.muted_foreground;
        let accent_color = theme.accent;

        // Search display
        let search_display = if self.search_text.is_empty() {
            SharedString::from("Search for actions...")
        } else {
            SharedString::from(self.search_text.clone())
        };

        // Build search input row - Raycast style: no search icon, just placeholder with cursor
        let search_input = div()
            .w_full()
            .h(px(PANEL_SEARCH_HEIGHT))
            .px(px(12.0))
            .py(px(8.0))
            .bg(search_bg_color) // Vibrancy-aware search area
            .border_b_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            // Search field - full width, no icon
            .child(
                div()
                    .flex_1()
                    .h(px(28.0))
                    .px(px(8.0))
                    .bg(search_bg_color) // Vibrancy-aware input
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(if self.search_text.is_empty() {
                        border_color
                    } else {
                        accent_color
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    .text_color(if self.search_text.is_empty() {
                        text_muted
                    } else {
                        text_primary
                    })
                    // Cursor when empty
                    .when(self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .mr(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    })
                    .child(search_display)
                    // Cursor when has text
                    .when(!self.search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .w(px(2.))
                                .h(px(16.))
                                .ml(px(2.))
                                .rounded(px(1.))
                                .when(self.cursor_visible, |d| d.bg(accent_color)),
                        )
                    }),
            );

        // Build actions list
        let selected_index = self.selected_index;
        let filtered_len = self.filtered_indices.len();

        let actions_list = if self.filtered_indices.is_empty() {
            div()
                .flex_1()
                .w_full()
                .py(px(16.0))
                .px(px(12.0))
                .text_color(text_muted)
                .text_sm()
                .child("No actions match your search")
                .into_any_element()
        } else {
            uniform_list(
                "notes-actions-list",
                filtered_len,
                cx.processor(
                    move |this: &mut NotesActionsPanel, visible_range, _window, cx| {
                        let theme = cx.theme();
                        let mut items = Vec::new();

                        for idx in visible_range {
                            if let Some(&action_idx) = this.filtered_indices.get(idx) {
                                if let Some(action) = this.actions.get(action_idx) {
                                    let action: &NotesActionItem = action;
                                    let is_enabled = action.enabled;
                                    let is_selected = idx == selected_index && is_enabled;
                                    let is_section_start = if idx > 0 {
                                        this.filtered_indices
                                            .get(idx - 1)
                                            .and_then(|&prev_idx| this.actions.get(prev_idx))
                                            .map(|prev: &NotesActionItem| {
                                                prev.section() != action.section()
                                            })
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    // Transparent Hsla for unselected state
                                    let transparent = Hsla {
                                        h: 0.0,
                                        s: 0.0,
                                        l: 0.0,
                                        a: 0.0,
                                    };

                                    // Raycast-style: rounded pill selection, no left accent bar
                                    // Outer wrapper provides horizontal inset for the rounded background
                                    let action_row = div()
                                        .id(idx)
                                        .w_full()
                                        .h(px(ACTION_ITEM_HEIGHT))
                                        .px(px(ACTION_ROW_INSET))
                                        .flex()
                                        .flex_col()
                                        .justify_center()
                                        // Section divider as top border
                                        .when(is_section_start, |d| {
                                            d.border_t_1().border_color(theme.border)
                                        })
                                        // Inner row with rounded background
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(ACTION_ITEM_HEIGHT - 8.0))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .px(px(8.0))
                                                .rounded(px(SELECTION_RADIUS))
                                                .bg(if is_selected {
                                                    theme.list_active
                                                } else {
                                                    transparent
                                                })
                                                .when(is_enabled, |d| {
                                                    d.hover(|s| s.bg(theme.list_hover))
                                                })
                                                .when(is_enabled, |d| d.cursor_pointer())
                                                .when(!is_enabled, |d| d.opacity(0.5))
                                                // Content row: icon + label + shortcuts
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        // Left: icon + label
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .gap(px(10.0))
                                                                // Icon
                                                                .child(
                                                                    svg()
                                                                        .external_path(action.action.icon().external_path())
                                                                        .size(px(16.))
                                                                        .text_color(if is_enabled {
                                                                            theme.foreground
                                                                        } else {
                                                                            theme.muted_foreground
                                                                        }),
                                                                )
                                                                // Label
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .text_color(if is_enabled {
                                                                            theme.foreground
                                                                        } else {
                                                                            theme.muted_foreground
                                                                        })
                                                                        .font_weight(
                                                                            if is_selected {
                                                                                gpui::FontWeight::MEDIUM
                                                                            } else {
                                                                                gpui::FontWeight::NORMAL
                                                                            },
                                                                        )
                                                                        .child(action.action.label()),
                                                                ),
                                                        )
                                                        // Right: shortcut badge
                                                        .child(render_shortcut_keys(
                                                            action.action.shortcut_keys(),
                                                            theme,
                                                        )),
                                                ),
                                        )
                                        .when(is_enabled, |d| {
                                            d.on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.selected_index = idx;
                                                    this.submit_selected();
                                                    cx.notify();
                                                }),
                                            )
                                        });

                                    items.push(action_row);
                                }
                            }
                        }
                        items
                    },
                ),
            )
            .flex_1()
            .w_full()
            .track_scroll(&self.scroll_handle)
            .into_any_element()
        };

        // Calculate dynamic height
        let items_height = (filtered_len as f32 * ACTION_ITEM_HEIGHT)
            .min(PANEL_MAX_HEIGHT - (PANEL_SEARCH_HEIGHT + 16.0));
        let total_height = items_height + PANEL_SEARCH_HEIGHT + PANEL_BORDER_HEIGHT;

        // Main container
        div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(total_height))
            .bg(bg_color)
            .rounded(px(PANEL_CORNER_RADIUS))
            .shadow(Self::create_shadow())
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .child(search_input)
            .child(actions_list)
    }
}

fn render_shortcut_keys(keys: &[&'static str], theme: &Theme) -> impl IntoElement {
    if keys.is_empty() {
        return div().into_any_element();
    }

    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for key in keys {
        row = row.child(
            div()
                .min_w(px(18.0))
                .px(px(6.0))
                .py(px(2.0))
                .bg(theme.muted)
                .border_1()
                .border_color(theme.border)
                .rounded(px(5.0))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(*key),
        );
    }

    row.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_action_labels() {
        assert_eq!(NotesAction::NewNote.label(), "New Note");
        assert_eq!(NotesAction::DuplicateNote.label(), "Duplicate Note");
        assert_eq!(NotesAction::BrowseNotes.label(), "Browse Notes");
        assert_eq!(NotesAction::FindInNote.label(), "Find in Note");
        assert_eq!(NotesAction::CopyNoteAs.label(), "Copy Note As...");
        assert_eq!(NotesAction::CopyDeeplink.label(), "Copy Deeplink");
        assert_eq!(NotesAction::CreateQuicklink.label(), "Create Quicklink");
        assert_eq!(NotesAction::Export.label(), "Export...");
        assert_eq!(NotesAction::MoveListItemUp.label(), "Move List Item Up");
        assert_eq!(NotesAction::MoveListItemDown.label(), "Move List Item Down");
        assert_eq!(NotesAction::Format.label(), "Format...");
    }

    #[test]
    fn test_notes_action_shortcuts() {
        assert_eq!(NotesAction::NewNote.shortcut_display(), "⌘N");
        assert_eq!(NotesAction::DuplicateNote.shortcut_display(), "⌘D");
        assert_eq!(NotesAction::BrowseNotes.shortcut_display(), "⌘P");
        assert_eq!(NotesAction::FindInNote.shortcut_display(), "⌘F");
        assert_eq!(NotesAction::CopyNoteAs.shortcut_display(), "⇧⌘C");
        assert_eq!(NotesAction::CopyDeeplink.shortcut_display(), "⇧⌘D");
        assert_eq!(NotesAction::CreateQuicklink.shortcut_display(), "⇧⌘L");
        assert_eq!(NotesAction::Export.shortcut_display(), "⇧⌘E");
        assert_eq!(NotesAction::MoveListItemUp.shortcut_display(), "⌃⌘↑");
        assert_eq!(NotesAction::MoveListItemDown.shortcut_display(), "⌃⌘↓");
        assert_eq!(NotesAction::Format.shortcut_display(), "⇧⌘T");
    }

    #[test]
    fn test_notes_action_all() {
        let all = NotesAction::all();
        assert_eq!(all.len(), 11);
        assert!(all.contains(&NotesAction::NewNote));
        assert!(all.contains(&NotesAction::DuplicateNote));
        assert!(all.contains(&NotesAction::BrowseNotes));
        assert!(all.contains(&NotesAction::FindInNote));
        assert!(all.contains(&NotesAction::CopyNoteAs));
        assert!(all.contains(&NotesAction::CopyDeeplink));
        assert!(all.contains(&NotesAction::CreateQuicklink));
        assert!(all.contains(&NotesAction::Export));
        assert!(all.contains(&NotesAction::MoveListItemUp));
        assert!(all.contains(&NotesAction::MoveListItemDown));
        assert!(all.contains(&NotesAction::Format));
    }

    #[test]
    fn test_notes_action_ids() {
        assert_eq!(NotesAction::NewNote.id(), "new_note");
        assert_eq!(NotesAction::DuplicateNote.id(), "duplicate_note");
        assert_eq!(NotesAction::BrowseNotes.id(), "browse_notes");
        assert_eq!(NotesAction::FindInNote.id(), "find_in_note");
        assert_eq!(NotesAction::CopyNoteAs.id(), "copy_note_as");
        assert_eq!(NotesAction::CopyDeeplink.id(), "copy_deeplink");
        assert_eq!(NotesAction::CreateQuicklink.id(), "create_quicklink");
        assert_eq!(NotesAction::Export.id(), "export");
        assert_eq!(NotesAction::MoveListItemUp.id(), "move_list_item_up");
        assert_eq!(NotesAction::MoveListItemDown.id(), "move_list_item_down");
        assert_eq!(NotesAction::Format.id(), "format");
    }

    #[test]
    fn test_panel_constants() {
        // Verify panel matches main ActionsDialog dimensions
        assert_eq!(PANEL_WIDTH, 320.0);
        assert_eq!(PANEL_MAX_HEIGHT, 400.0); // Standardized to match main dialog
        assert_eq!(PANEL_CORNER_RADIUS, 12.0);
        assert_eq!(ACTION_ITEM_HEIGHT, 44.0);
        assert_eq!(ACTION_ROW_INSET, 6.0);
        assert_eq!(SELECTION_RADIUS, 8.0);
    }
}

</file>

<file path=".opencode/skill/script-kit-actions-window/SKILL.md">
---
name: script-kit-actions-window
description: Actions window/dialog system for Script Kit GPUI. A Raycast-style searchable action menu supporting script context actions, file actions, path actions, and SDK-provided custom actions. Use when working with ActionsDialog, ActionsWindow, action builders, or the actions panel UI.
tags:
  - gpui
  - actions
  - dialog
  - popup
  - menu
  - vibrancy
---

# Script Kit Actions Window

Actions window/dialog system for Script Kit GPUI. A Raycast-style searchable action menu 
supporting script context actions, file actions, path actions, and SDK-provided custom actions.

**Use when:**
- Building or modifying the actions popup UI
- Adding new action types or categories
- Working with keyboard shortcut hints/keycaps
- Integrating SDK-provided custom actions
- Positioning/resizing the floating actions window

## Architecture Overview

```
src/actions/
├── mod.rs          # Public API re-exports
├── types.rs        # Action, ActionCategory, ScriptInfo, ActionCallback
├── builders.rs     # Factory functions: get_script_context_actions(), etc.
├── constants.rs    # Layout constants: POPUP_WIDTH, ACTION_ITEM_HEIGHT, etc.
├── dialog.rs       # ActionsDialog struct + rendering logic
└── window.rs       # Separate vibrancy window management
```

### Key Types

```rust
// src/actions/types.rs

/// Information about the currently focused script/item
pub struct ScriptInfo {
    pub name: String,           // Display name
    pub path: String,           // Full path (empty for built-ins)
    pub is_script: bool,        // false for built-ins like "Clipboard History"
    pub action_verb: String,    // "Run", "Launch", "Switch to"
    pub shortcut: Option<String>, // Current assigned shortcut
}

/// An action in the menu
pub struct Action {
    pub id: String,              // Unique identifier (e.g., "edit_script")
    pub title: String,           // Display text
    pub description: Option<String>,
    pub category: ActionCategory,
    pub shortcut: Option<String>, // Keyboard hint (e.g., "⌘E")
    pub has_action: bool,        // SDK routing: true = ActionTriggered, false = submit value
    pub value: Option<String>,   // Value to submit when triggered
}

pub enum ActionCategory {
    ScriptContext,  // Actions for the focused script/file
    ScriptOps,      // Edit, Create, Delete (reserved)
    GlobalOps,      // Settings, Quit (now in main menu)
}

/// Callback signature for action selection
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;
```

### ActionsDialog Entity

```rust
// src/actions/dialog.rs

pub struct ActionsDialog {
    pub actions: Vec<Action>,
    pub filtered_actions: Vec<usize>,  // Indices into actions (after search filter)
    pub selected_index: usize,         // Index within filtered_actions
    pub search_text: String,
    pub focus_handle: FocusHandle,
    pub on_select: ActionCallback,
    pub focused_script: Option<ScriptInfo>,
    pub scroll_handle: UniformListScrollHandle,
    pub theme: Arc<Theme>,
    pub design_variant: DesignVariant,
    pub cursor_visible: bool,          // For blinking cursor
    pub hide_search: bool,             // True when embedded in header
    pub sdk_actions: Option<Vec<ProtocolAction>>,  // SDK-provided actions
    pub context_title: Option<String>, // Header title
}
```

## Creating ActionsDialog

### For Scripts (main use case)

```rust
use crate::actions::{ActionsDialog, ScriptInfo};

// Basic - no script context
let dialog = ActionsDialog::new(focus_handle, on_select_callback, theme);

// With script context - shows script-specific actions
let script_info = ScriptInfo::new("my-script", "/path/to/my-script.ts");
let dialog = ActionsDialog::with_script(focus_handle, callback, Some(script_info), theme);

// With shortcut info (affects which shortcut actions show)
let script_info = ScriptInfo::with_shortcut(
    "my-script",
    "/path/to/script.ts", 
    Some("cmd+shift+m".to_string())  // Has shortcut → shows Update/Remove
);

// For built-in commands (limited actions - no edit, reveal, copy path)
let builtin = ScriptInfo::builtin("Clipboard History");
let dialog = ActionsDialog::with_script(focus_handle, callback, Some(builtin), theme);
```

### For File Search Results

```rust
use crate::actions::ActionsDialog;
use crate::file_search::FileInfo;

let file_info = FileInfo {
    path: "/Users/test/document.pdf".to_string(),
    name: "document.pdf".to_string(),
    file_type: FileType::Document,
    is_dir: false,
};

let dialog = ActionsDialog::with_file(focus_handle, callback, &file_info, theme);
// Actions: Open, Show in Finder, Quick Look, Open With..., Get Info, Copy Path
```

### For Path Prompt

```rust
use crate::actions::ActionsDialog;
use crate::prompts::PathInfo;

let path_info = PathInfo {
    path: "/Users/test/folder".to_string(),
    name: "folder".to_string(),
    is_dir: true,
};

let dialog = ActionsDialog::with_path(focus_handle, callback, &path_info, theme);
// Actions: Open, Copy Path, Open in Finder, Open in Editor, Open in Terminal, etc.
```

## Window Management

The actions dialog can render in two modes:
1. **Inline overlay** (legacy) - rendered within main window
2. **Separate vibrancy window** (preferred) - floating popup with blur

### Opening the Floating Window

```rust
use crate::actions::{open_actions_window, close_actions_window, is_actions_window_open};

// Create dialog entity first
let dialog_entity = cx.new(|cx| ActionsDialog::with_script(...));

// Get main window bounds (SCREEN-RELATIVE coordinates)
let main_bounds = window.bounds();
let display_id = window.display().map(|d| d.id());

// Open floating window - positions at bottom-right of main window
match open_actions_window(cx, main_bounds, display_id, dialog_entity) {
    Ok(handle) => { /* window opened */ }
    Err(e) => { /* handle error */ }
}

// Check if open
if is_actions_window_open() { ... }

// Close window
close_actions_window(cx);
```

### Resizing After Filter Changes

```rust
use crate::actions::resize_actions_window;

// After search text changes and filtered_actions.len() changes:
resize_actions_window(cx, &dialog_entity);
// Window stays "pinned to bottom" - bottom edge stays fixed, top moves
```

### Notifying for Re-render

```rust
use crate::actions::notify_actions_window;

// After updating dialog entity state:
dialog_entity.update(cx, |dialog, cx| {
    dialog.set_cursor_visible(!dialog.cursor_visible);
    cx.notify();
});
notify_actions_window(cx);  // Also notify the window
```

## Action Builders

Factory functions that create action lists for different contexts:

```rust
// src/actions/builders.rs

/// Script context actions (most common)
/// Actions vary based on is_script and shortcut presence
pub fn get_script_context_actions(script: &ScriptInfo) -> Vec<Action>;
// Returns: Run, Add/Update/Remove Shortcut, Edit, View Logs, Reveal, Copy Path, Copy Deeplink

/// File search result actions
pub fn get_file_context_actions(file_info: &FileInfo) -> Vec<Action>;
// Returns: Open, Show in Finder, Quick Look, Open With, Get Info, Copy Path/Filename

/// Path prompt actions
pub fn get_path_context_actions(path_info: &PathInfo) -> Vec<Action>;
// Returns: Open/Select, Copy Path, Open in Finder/Editor/Terminal, Copy Filename, Move to Trash

/// Global actions (currently empty - Settings/Quit in main menu)
pub fn get_global_actions() -> Vec<Action>;
```

## SDK Actions Integration

Scripts can provide custom actions via the SDK:

```rust
// Set SDK actions (replaces built-in actions)
dialog.set_sdk_actions(vec![
    ProtocolAction {
        name: "Custom Action".to_string(),
        description: Some("Does something custom".to_string()),
        shortcut: Some("cmd+k".to_string()),
        value: Some("custom-value".to_string()),
        has_action: true,  // true = send ActionTriggered to SDK
        visible: None,     // None or Some(true) = visible
        close: None,       // Whether to close dialog after action
    },
]);

// Check if SDK actions are active
if dialog.has_sdk_actions() { ... }

// Clear SDK actions (restores built-in)
dialog.clear_sdk_actions();
```

### Action Routing Logic

```rust
// In action handler:
if action.has_action {
    // Send ActionTriggered event to SDK
    send_to_sdk(ActionTriggered { name: action.id, value: action.value });
} else {
    // Submit value directly (like selecting a choice)
    submit_value(action.value.unwrap_or(action.id));
}
```

## Layout Constants

```rust
// src/actions/constants.rs

pub const POPUP_WIDTH: f32 = 320.0;
pub const POPUP_MAX_HEIGHT: f32 = 400.0;
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;  // iOS-style touch target
pub const SEARCH_INPUT_HEIGHT: f32 = 44.0;
pub const HEADER_HEIGHT: f32 = 44.0;
pub const ACTION_ROW_INSET: f32 = 6.0;     // Pill-style row padding
pub const SELECTION_RADIUS: f32 = 8.0;     // Selected row corner radius
pub const KEYCAP_MIN_WIDTH: f32 = 22.0;
pub const KEYCAP_HEIGHT: f32 = 22.0;
pub const ACCENT_BAR_WIDTH: f32 = 3.0;     // Legacy, kept for reference
```

## Keyboard Shortcut Formatting

```rust
// Convert SDK shortcut format to display symbols
ActionsDialog::format_shortcut_hint("cmd+shift+e") // → "⌘⇧E"
ActionsDialog::format_shortcut_hint("ctrl+c")      // → "⌃C"
ActionsDialog::format_shortcut_hint("enter")       // → "↵"

// Parse shortcut into individual keycaps for rendering
ActionsDialog::parse_shortcut_keycaps("⌘⇧E") // → vec!["⌘", "⇧", "E"]
```

### Symbol Mappings

| Input | Symbol |
|-------|--------|
| `cmd`, `command`, `meta` | ⌘ |
| `ctrl`, `control` | ⌃ |
| `alt`, `opt`, `option` | ⌥ |
| `shift` | ⇧ |
| `enter`, `return` | ↵ |
| `escape`, `esc` | ⎋ |
| `tab` | ⇥ |
| `backspace`, `delete` | ⌫ |
| `space` | ␣ |
| `up`, `arrowup` | ↑ |
| `down`, `arrowdown` | ↓ |
| `left`, `arrowleft` | ← |
| `right`, `arrowright` | → |

## Search/Filter Behavior

The dialog implements ranked fuzzy matching:

```rust
// Scoring system (higher = better match):
// - Prefix match on title: +100
// - Contains match on title: +50  
// - Fuzzy subsequence match on title: +25
// - Contains match on description: +15

dialog.refilter();  // Called automatically when search_text changes
```

## ActionsDialogHost Enum

Tracks where the actions dialog was opened from (for proper close handling):

```rust
// src/main.rs
enum ActionsDialogHost {
    MainList,      // Main script list
    ArgPrompt,     // Arg prompt
    DivPrompt,     // Div prompt
    EditorPrompt,  // Editor prompt
    TermPrompt,    // Terminal prompt
    FormPrompt,    // Form prompt
    FileSearch,    // File search results
}
```

## Rendering Pattern

```rust
impl Render for ActionsDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = get_tokens(self.design_variant).colors;
        
        div()
            .w(px(POPUP_WIDTH))
            .flex()
            .flex_col()
            .bg(rgb(colors.background))
            .rounded(px(SELECTION_RADIUS))
            .shadow(/* vibrancy-compatible shadow */)
            // Optional header with context title
            .when_some(self.context_title.clone(), |el, title| {
                el.child(self.render_header(&title, &colors))
            })
            // Search input (unless hide_search)
            .when(!self.hide_search, |el| {
                el.child(self.render_search_input(&colors))
            })
            // Virtualized action list
            .child(
                uniform_list("actions-list", self.filtered_actions.len(), |this, range, _, _| {
                    this.render_action_items(range)
                })
                .track_scroll(&self.scroll_handle)
                .h(px(items_height))
            )
    }
}
```

## Common Patterns

### Opening Actions from a Prompt

```rust
// In render_prompts/arg.rs, div.rs, editor.rs, etc.
fn show_actions_popup(&mut self, host: ActionsDialogHost, window: &mut Window, cx: &mut Context<Self>) {
    let dialog = ActionsDialog::with_script(
        cx.focus_handle(),
        Arc::new(|_| {}),  // Callback handled by main app
        self.get_script_info(),
        self.theme.clone(),
    );
    
    let dialog_entity = cx.new(|_| dialog);
    let main_bounds = window.bounds();
    let display_id = window.display().map(|d| d.id());
    
    open_actions_window(cx, main_bounds, display_id, dialog_entity);
}
```

### Closing on Escape/Click Outside

```rust
// In key handler
"escape" | "Escape" => {
    if is_actions_window_open() {
        close_actions_window(cx);
    }
}

// Click-outside detection (in ActionsDialog render)
.on_mouse_down_out(cx.listener(|this, _, _, cx| {
    logging::log("ACTIONS", "dismiss-on-click-outside triggered");
    // Parent handles actual close via callback
}))
```

## Testing

```rust
#[test]
fn test_get_script_context_actions_no_shortcut() {
    let script = ScriptInfo::new("my-script", "/path/to/script.ts");
    let actions = get_script_context_actions(&script);
    
    assert!(actions.iter().any(|a| a.id == "run_script"));
    assert!(actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(!actions.iter().any(|a| a.id == "update_shortcut"));
}

#[test]
fn test_get_script_context_actions_with_shortcut() {
    let script = ScriptInfo::with_shortcut("my-script", "/path", Some("cmd+m".to_string()));
    let actions = get_script_context_actions(&script);
    
    assert!(!actions.iter().any(|a| a.id == "add_shortcut"));
    assert!(actions.iter().any(|a| a.id == "update_shortcut"));
    assert!(actions.iter().any(|a| a.id == "remove_shortcut"));
}
```

## Related Files

- `src/main.rs` - ActionsDialogHost enum, action routing
- `src/app_impl.rs` - Action handling, theme propagation
- `src/render_script_list.rs` - Opening actions from script list
- `src/render_prompts/*.rs` - Opening actions from prompts
- `src/notes/actions_panel.rs` - Similar pattern for Notes window
- `src/stories/actions_window_stories.rs` - Storybook stories

</file>

</files>
---

## Supplementary: AI Window Integration (src/ai/window.rs)

The AI window demonstrates the full integration pattern with ActionsDialogConfig:

```rust
// From src/ai/window.rs

// Using the unified ActionsDialog component for AI command bar (Cmd+K)
use crate::actions::{
    close_actions_window, get_ai_command_bar_actions, notify_actions_window, open_actions_window,
    ActionsDialog, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle,
};

// In AiApp struct:
/// The command bar dialog entity (uses the unified ActionsDialog component)
command_bar_dialog: Option<Entity<ActionsDialog>>,

/// Show the command bar as a separate vibrancy window (Cmd+K)
/// Creates a new ActionsDialog entity and opens it in a floating window with macOS blur.
fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    let theme = std::sync::Arc::new(crate::theme::load_theme());
    let actions = get_ai_command_bar_actions();

    // Configure for AI-style command bar:
    // - Search at top (like Raycast Cmd+K)
    // - Section headers (not separators)
    // - Icons shown
    // - Footer with keyboard hints
    let config = ActionsDialogConfig {
        search_position: SearchPosition::Top,
        section_style: SectionStyle::Headers,
        anchor: AnchorPosition::Top,
        show_icons: true,
        show_footer: true,
    };

    let on_select: std::sync::Arc<dyn Fn(String) + Send + Sync> =
        std::sync::Arc::new(|_action_id: String| {
            // Action handling is done via execute_command_bar_action()
        });

    let dialog = cx.new(|cx| {
        ActionsDialog::with_config(cx.focus_handle(), on_select, actions, theme, config)
    });

    let ai_window_bounds = window.bounds();
    let display_id = window.display(cx).map(|d| d.id());

    self.command_bar_dialog = Some(dialog.clone());
    self.showing_command_bar = true;

    // CRITICAL: Focus main focus_handle so keyboard events route to us
    self.focus_handle.focus(window, cx);

    // Open the command bar in a separate vibrancy window
    cx.spawn(async move |this, cx| {
        cx.update(|cx| {
            match open_actions_window(cx, ai_window_bounds, display_id, dialog) {
                Ok(_) => crate::logging::log("AI", "Command bar window opened"),
                Err(e) => crate::logging::log("AI", &format!("Failed: {}", e)),
            }
        }).ok();
        
        this.update(cx, |this, cx| {
            this.needs_command_bar_focus = true;
            cx.notify();
        }).ok();
    }).detach();
}
```

---

## Supplementary: Action Builder Pattern (src/actions/builders.rs)

```rust
/// Get actions for the AI chat command bar (Cmd+K menu)
///
/// Returns actions with icons and sections for:
/// - Response: Copy Response, Copy Chat, Copy Last Code Block
/// - Actions: Submit, New Chat, Delete Chat
/// - Attachments: Add Attachments, Paste Image
/// - Settings: Change Model
pub fn get_ai_command_bar_actions() -> Vec<Action> {
    vec![
        // Response section
        Action::new(
            "copy_response",
            "Copy Response",
            Some("Copy the last AI response".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        
        Action::new(
            "copy_chat",
            "Copy Chat",
            Some("Copy the entire conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌥⇧⌘C")
        .with_icon(IconName::Copy)
        .with_section("Response"),
        
        // Actions section
        Action::new(
            "submit",
            "Submit",
            Some("Send your message".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("↵")
        .with_icon(IconName::ArrowUp)
        .with_section("Actions"),
        
        Action::new(
            "new_chat",
            "New Chat",
            Some("Start a new conversation".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("⌘N")
        .with_icon(IconName::Plus)
        .with_section("Actions"),
        // ... more actions
    ]
}
```

---

## Implementation Guide

### Step 1: Create CommandBar Wrapper Type

```rust
// File: src/actions/command_bar.rs
// Location: New file in src/actions/

use super::{ActionsDialog, ActionsDialogConfig, Action};
use gpui::{Entity, FocusHandle, Window, Context, App, Bounds, Pixels, DisplayId};
use std::sync::Arc;

/// Reusable Command Bar component that wraps ActionsDialog
/// with consistent window management and focus handling.
pub struct CommandBar {
    /// The underlying dialog entity
    dialog: Entity<ActionsDialog>,
    /// Configuration for appearance and behavior
    config: ActionsDialogConfig,
    /// Whether the command bar is currently visible
    is_open: bool,
    /// Callback when an action is selected
    on_action: Arc<dyn Fn(&str) + Send + Sync>,
}

impl CommandBar {
    pub fn new(
        actions: Vec<Action>,
        config: ActionsDialogConfig,
        on_action: impl Fn(&str) + Send + Sync + 'static,
        theme: Arc<crate::theme::Theme>,
        cx: &mut Context<Self>,
    ) -> Self {
        let on_select = Arc::new(move |id: String| {});
        let dialog = cx.new(|cx| {
            ActionsDialog::with_config(
                cx.focus_handle(),
                on_select,
                actions,
                theme,
                config.clone(),
            )
        });
        
        Self {
            dialog,
            config,
            is_open: false,
            on_action: Arc::new(on_action),
        }
    }
    
    /// Toggle open/close state (for Cmd+K binding)
    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }
    
    /// Open the command bar
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open { return; }
        
        let bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());
        
        if let Ok(_) = super::open_actions_window(cx, bounds, display_id, self.dialog.clone()) {
            self.is_open = true;
            cx.notify();
        }
    }
    
    /// Close the command bar
    pub fn close(&mut self, cx: &mut App) {
        if !self.is_open { return; }
        
        super::close_actions_window(cx);
        self.is_open = false;
    }
    
    /// Check if open
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Update the actions list
    pub fn set_actions(&mut self, actions: Vec<Action>, cx: &mut App) {
        self.dialog.update(cx, |dialog, cx| {
            // Clear and rebuild
            dialog.actions = actions;
            dialog.filtered_actions = (0..dialog.actions.len()).collect();
            dialog.selected_index = 0;
            dialog.search_text.clear();
            cx.notify();
        });
        
        if self.is_open {
            super::resize_actions_window(cx, &self.dialog);
        }
    }
}
```

### Step 2: Add CommandBarHost Trait

```rust
// File: src/actions/command_bar.rs (continued)

/// Trait for views that can host a command bar
pub trait CommandBarHost {
    /// Get the command bar entity
    fn command_bar(&self) -> Option<&Entity<CommandBar>>;
    
    /// Get mutable command bar entity
    fn command_bar_mut(&mut self) -> Option<&mut Entity<CommandBar>>;
    
    /// Get actions for the current context
    fn get_context_actions(&self) -> Vec<Action>;
    
    /// Handle action execution
    fn execute_action(&mut self, action_id: &str, cx: &mut Context<Self>);
}
```

### Step 3: Update ActionsDialogConfig for Flexible Resize

```rust
// File: src/actions/types.rs
// Location: ActionsDialogConfig struct

pub struct ActionsDialogConfig {
    pub search_position: SearchPosition,
    pub section_style: SectionStyle,
    pub anchor: AnchorPosition,
    pub show_icons: bool,
    pub show_footer: bool,
    /// NEW: Custom width (default: POPUP_WIDTH = 320.0)
    pub width: Option<f32>,
    /// NEW: Maximum height (default: POPUP_MAX_HEIGHT = 400.0)
    pub max_height: Option<f32>,
    /// NEW: Margin from parent window edge
    pub margin: f32,
}
```

### Step 4: Global Shortcut Registration

```rust
// File: src/actions/shortcuts.rs (new file)

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Registry of global shortcuts from actions
static GLOBAL_SHORTCUTS: OnceLock<Mutex<HashMap<String, ShortcutBinding>>> = OnceLock::new();

pub struct ShortcutBinding {
    /// The shortcut string (e.g., "cmd+shift+e")
    pub shortcut: String,
    /// The action ID to execute
    pub action_id: String,
    /// Context for this shortcut (e.g., "script_list", "ai_chat")
    pub context: String,
    /// Whether this shortcut is active when command bar is closed
    pub global: bool,
}

/// Register shortcuts from an action list
pub fn register_shortcuts(actions: &[Action], context: &str) {
    let registry = GLOBAL_SHORTCUTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = registry.lock().unwrap();
    
    for action in actions {
        if let Some(ref shortcut) = action.shortcut {
            let binding = ShortcutBinding {
                shortcut: shortcut.clone(),
                action_id: action.id.clone(),
                context: context.to_string(),
                global: true, // Configure per-action later
            };
            guard.insert(format!("{}:{}", context, action.id), binding);
        }
    }
}

/// Try to execute a shortcut in the given context
pub fn try_execute_shortcut(shortcut: &str, context: &str) -> Option<String> {
    let registry = GLOBAL_SHORTCUTS.get()?;
    let guard = registry.lock().ok()?;
    
    for (_, binding) in guard.iter() {
        if binding.shortcut == shortcut 
            && (binding.context == context || binding.global) {
            return Some(binding.action_id.clone());
        }
    }
    None
}
```

---

## Instructions for the Next AI Agent

### Context
You are working on Script Kit GPUI, a Rust desktop application using the GPUI framework. The goal is to refactor the existing `ActionsDialog` component into a more reusable `CommandBar` component.

### Key Files to Modify
1. `src/actions/mod.rs` - Add new exports
2. `src/actions/command_bar.rs` - New file with CommandBar wrapper
3. `src/actions/types.rs` - Extend ActionsDialogConfig
4. `src/actions/window.rs` - May need resize anchor refinement
5. `src/hotkeys.rs` - For global shortcut registration

### Critical Patterns to Follow

**Focus Management:**
- When opening command bar: `self.focus_handle.focus(window, cx)` to keep keyboard routing
- The vibrancy window has its own key handler but parent can route via `dialog.update()`
- Always call `cx.notify()` after state changes

**GPUI Key Handling:**
- Match both arrow key variants: `"up" | "arrowup"`, `"down" | "arrowdown"`
- Use `cx.listener()` for event handlers
- Coalesce rapid key events (20ms window) to prevent lag

**Testing Protocol:**
- Never test via CLI args - use stdin JSON protocol:
  ```bash
  echo '{"type":"run","path":"..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
  ```
- Before commits: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

### What NOT to Do
- Don't create new window types - use the existing `open_actions_window()` pattern
- Don't hardcode colors - use `theme.colors.*` or design tokens
- Don't skip `cx.notify()` after render-affecting changes
- Don't use blocking calls on the main thread

### Success Criteria
1. CommandBar can be instantiated with any action list and config
2. Cmd+K toggles consistently across all host contexts
3. Search position (top/bottom) works correctly
4. Resize anchor (top/bottom pin) works correctly
5. Shortcuts can be registered globally and triggered when command bar is closed
6. All existing ActionsDialog functionality continues to work

