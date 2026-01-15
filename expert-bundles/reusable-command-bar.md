# Reusable Command Bar Expert Bundle

## Original Goal

> Improving the Actions Window/Dialog into a reusable Command Bar that can be used throughout the applications and other windows. We need the design to stay consistent and the features such as focusing on the input, navigating the list of options, submitting an action, and everything to be fully implemented. The component needs to be flexible enough that the search can be at the top or bottom and that the resize can resize from the top or bottom. There will be concerns around focus and toggling open/closed with cmd+k and exposing the keyboard shortcuts to the be active so that even if the menu is closed, the actions can still be triggered.
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The codebase already contains a well-structured `ActionsDialog` component in `src/actions/` that functions as a reusable Command Bar. It supports configurable search position (top/bottom), dynamic resizing, fuzzy search, keyboard navigation, and can be opened as either an inline overlay or a separate vibrancy window. The AI window (`src/ai/window.rs`) demonstrates the pattern of using `ActionsDialog` as a Cmd+K command bar.

### Key Problems:
1. **Inconsistent usage patterns**: The main script list and AI window use slightly different approaches to open/close the command bar, which could be unified.
2. **Focus management complexity**: Each host (main window, AI window, prompts) implements its own keyboard routing to the dialog, requiring careful focus handle management.
3. **Shortcut activation when closed**: Shortcuts defined in actions work when the dialog is open, but global shortcuts (when dialog is closed) must be handled separately by each host.

### Required Fixes (for full reusability):
1. **Create a unified `CommandBar` wrapper** that encapsulates the `ActionsDialog` entity + window management + keyboard routing into a single reusable component.
2. **Extract global shortcut handling** into a registry pattern so shortcuts work even when the command bar is closed.
3. **Standardize the focus restoration logic** across all hosts using `FocusTarget` enum.

### Files Included:
- `src/actions/mod.rs`: Module re-exports and public API
- `src/actions/types.rs`: Core types (`Action`, `ActionCategory`, `ScriptInfo`, `ActionsDialogConfig`)
- `src/actions/dialog.rs`: Main `ActionsDialog` struct with search, filtering, selection, keyboard handling
- `src/actions/window.rs`: Separate vibrancy window management (`open_actions_window`, `close_actions_window`, `resize_actions_window`)
- `src/actions/constants.rs`: Layout dimensions (`POPUP_WIDTH`, `ACTION_ITEM_HEIGHT`, etc.)
- `src/actions/builders.rs`: Factory functions for creating context-specific action lists
- `src/ai/window.rs` (excerpts): Example of using ActionsDialog as a Cmd+K command bar
- `src/render_script_list.rs` (excerpts): Example of toggle_actions and keyboard routing
- `src/main.rs` (excerpts): Focus management patterns (`FocusTarget`, `FocusedInput`)
- `src/components/scrollbar.rs`: Custom scrollbar component used by the dialog

---

## Core Architecture

### Configuration Types (src/actions/types.rs)

```rust
/// Configuration for how the search input is positioned
pub enum SearchPosition {
    Top,    // AI chat style - list grows downward
    Bottom, // Main menu style - list grows upward (default)
    Hidden, // External search handling
}

/// Configuration for how sections/categories are displayed
pub enum SectionStyle {
    Headers,    // Text headers for sections
    Separators, // Subtle lines between categories (default)
    None,
}

/// Configuration for dialog anchor position during resize
pub enum AnchorPosition {
    Top,    // Dialog grows/shrinks from top
    Bottom, // Dialog grows/shrinks from bottom (default)
}

/// Complete configuration for ActionsDialog appearance and behavior
pub struct ActionsDialogConfig {
    pub search_position: SearchPosition,
    pub section_style: SectionStyle,
    pub anchor: AnchorPosition,
    pub show_icons: bool,
    pub show_footer: bool,
}
```

### Key API Methods (src/actions/dialog.rs)

```rust
impl ActionsDialog {
    // Create with custom configuration (most flexible)
    pub fn with_config(
        focus_handle: FocusHandle,
        on_select: ActionCallback,
        actions: Vec<Action>,
        theme: Arc<Theme>,
        config: ActionsDialogConfig,
    ) -> Self;
    
    // Input handling
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>);
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>);
    
    // Navigation
    pub fn move_up(&mut self, cx: &mut Context<Self>);
    pub fn move_down(&mut self, cx: &mut Context<Self>);
    
    // Selection
    pub fn get_selected_action_id(&self) -> Option<String>;
    pub fn get_selected_action(&self) -> Option<&Action>;
    pub fn submit_selected(&mut self);
    
    // SDK actions support
    pub fn set_sdk_actions(&mut self, actions: Vec<ProtocolAction>);
    pub fn clear_sdk_actions(&mut self);
    pub fn has_sdk_actions(&self) -> bool;
}
```

### Window Management (src/actions/window.rs)

```rust
// Open as a separate vibrancy window
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
) -> anyhow::Result<WindowHandle<Root>>;

// Close the window
pub fn close_actions_window(cx: &mut App);

// Check if open
pub fn is_actions_window_open() -> bool;

// Update after filter changes
pub fn notify_actions_window(cx: &mut App);
pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>);
```

---

## Usage Pattern 1: AI Window Command Bar (Cmd+K)

From `src/ai/window.rs`:

```rust
fn show_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    let theme = Arc::new(crate::theme::load_theme());
    let actions = get_ai_command_bar_actions();
    
    // Configure for AI-style command bar:
    // - Search at top (like Raycast Cmd+K)
    // - Section headers (not separators)
    let config = ActionsDialogConfig {
        search_position: SearchPosition::Top,
        section_style: SectionStyle::Headers,
        anchor: AnchorPosition::Top,
        show_icons: true,
        show_footer: true,
    };
    
    let on_select: Arc<dyn Fn(String) + Send + Sync> = Arc::new(|_| {});
    
    // Create the ActionsDialog
    let dialog = cx.new(|cx| {
        ActionsDialog::with_config(cx.focus_handle(), on_select, actions, theme, config)
    });
    
    // Store for keyboard routing
    self.command_bar_dialog = Some(dialog.clone());
    self.showing_command_bar = true;
    
    // Open in separate vibrancy window
    let bounds = window.bounds();
    let display_id = window.display(cx).map(|d| d.id());
    
    cx.spawn(async move |this, cx| {
        cx.update(|cx| {
            open_actions_window(cx, bounds, display_id, dialog).ok();
        }).ok();
    }).detach();
    
    cx.notify();
}

fn hide_command_bar(&mut self, cx: &mut Context<Self>) {
    self.showing_command_bar = false;
    self.command_bar_dialog = None;
    close_actions_window(cx);
    cx.notify();
}
```

### Keyboard Routing in Host Window

```rust
.on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
    let key = event.keystroke.key.as_str();
    let modifiers = &event.keystroke.modifiers;
    
    // Handle command bar navigation when open
    if this.showing_command_bar {
        match key {
            "up" | "arrowup" => {
                this.command_bar_select_prev(cx);
                return;
            }
            "down" | "arrowdown" => {
                this.command_bar_select_next(cx);
                return;
            }
            "enter" | "return" => {
                this.execute_command_bar_action(window, cx);
                return;
            }
            "escape" => {
                this.hide_command_bar(cx);
                return;
            }
            "backspace" | "delete" => {
                this.command_bar_handle_backspace(cx);
                return;
            }
            _ => {
                // Handle printable characters for search
                if !modifiers.platform && !modifiers.control && !modifiers.alt {
                    if let Some(ch) = key.chars().next() {
                        if ch.is_alphanumeric() || ch.is_whitespace() {
                            this.command_bar_handle_char(ch, cx);
                            return;
                        }
                    }
                }
            }
        }
    }
    
    // Toggle command bar with Cmd+K
    if modifiers.platform && key == "k" {
        if this.showing_command_bar {
            this.hide_command_bar(cx);
        } else {
            this.show_command_bar(window, cx);
        }
    }
}))
```

---

## Usage Pattern 2: Main Script List Actions

From `src/render_script_list.rs`:

```rust
// Toggle with Cmd+K
"k" => {
    this.toggle_actions(cx, window);
    return;
}

// Route keyboard events when popup is open
if this.show_actions_popup {
    if let Some(ref dialog) = this.actions_dialog {
        match key_str.as_str() {
            "up" | "arrowup" => {
                dialog.update(cx, |d, cx| d.move_up(cx));
                cx.spawn(async move |_this, cx| {
                    cx.update(notify_actions_window).ok();
                }).detach();
                return;
            }
            "down" | "arrowdown" => {
                dialog.update(cx, |d, cx| d.move_down(cx));
                cx.spawn(async move |_this, cx| {
                    cx.update(notify_actions_window).ok();
                }).detach();
                return;
            }
            "enter" => {
                let action_id = dialog.read(cx).get_selected_action_id();
                let should_close = dialog.read(cx).selected_action_should_close();
                if let Some(action_id) = action_id {
                    if should_close {
                        this.show_actions_popup = false;
                        this.actions_dialog = None;
                        cx.spawn(async move |_this, cx| {
                            cx.update(close_actions_window).ok();
                        }).detach();
                        this.focus_main_filter(window, cx);
                    }
                    this.handle_action(action_id, cx);
                }
                cx.notify();
                return;
            }
            // ... backspace, escape handling
        }
    }
}
```

---

## Focus Management Patterns

From `src/main.rs`:

```rust
/// Tracks which input currently has focus
enum FocusedInput {
    MainFilter,     // Main script list filter input
    ActionsSearch,  // Actions dialog search input
    ArgPrompt,      // Arg prompt input
    None,           // No input focused
}

/// Pending focus target - applied once then cleared
enum FocusTarget {
    MainFilter,
    AppRoot,
    ActionsDialog,
    PathPrompt,
    FormPrompt,
    EditorPrompt,
    // ... other prompt types
}

// In ScriptListApp:
struct ScriptListApp {
    focused_input: FocusedInput,
    pending_focus: Option<FocusTarget>,
    actions_dialog: Option<Entity<ActionsDialog>>,
    show_actions_popup: bool,
    // ...
}
```

---

## Dynamic Height Calculation (Bottom-Pinned Resize)

From `src/actions/window.rs`:

```rust
pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>) {
    if let Some(handle) = get_actions_window_handle() {
        let dialog = dialog_entity.read(cx);
        let num_actions = dialog.filtered_actions.len();
        
        // Calculate new height
        let search_box_height = if dialog.hide_search { 0.0 } else { SEARCH_INPUT_HEIGHT };
        let header_height = if dialog.context_title.is_some() { HEADER_HEIGHT } else { 0.0 };
        let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let new_height = items_height + search_box_height + header_height + 2.0;
        
        // "Pin to bottom": keep the bottom edge fixed
        // On macOS, frame.origin.y is the BOTTOM of the window
        #[cfg(target_os = "macos")]
        unsafe {
            // ... NSWindow frame manipulation
            let new_frame = NSRect::new(
                NSPoint::new(frame.origin.x, frame.origin.y), // Keep origin.y same
                NSSize::new(frame.size.width, new_height),
            );
            msg_send![ns_window, setFrame:new_frame display:true animate:false];
        }
    }
}
```

---

## Creating Custom Action Lists

From `src/actions/builders.rs`:

```rust
/// Get actions for the AI chat command bar (Cmd+K menu)
pub fn get_ai_command_bar_actions() -> Vec<Action> {
    vec![
        // Response section
        Action::new("copy_response", "Copy Response", Some("Copy the last AI response".to_string()), ActionCategory::ScriptContext)
            .with_shortcut("⇧⌘C")
            .with_icon(IconName::Copy)
            .with_section("Response"),
        
        Action::new("copy_chat", "Copy Chat", Some("Copy the entire conversation".to_string()), ActionCategory::ScriptContext)
            .with_shortcut("⌥⇧⌘C")
            .with_icon(IconName::Copy)
            .with_section("Response"),
        
        // Actions section
        Action::new("submit", "Submit", Some("Send your message".to_string()), ActionCategory::ScriptContext)
            .with_shortcut("↵")
            .with_icon(IconName::ArrowUp)
            .with_section("Actions"),
        
        Action::new("new_chat", "New Chat", Some("Start a new conversation".to_string()), ActionCategory::ScriptContext)
            .with_shortcut("⌘N")
            .with_icon(IconName::Plus)
            .with_section("Actions"),
        
        // ... more actions
    ]
}
```

---

## Packed Source Files

<files>
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
pub use builders::to_deeplink_name;
pub use builders::ClipboardEntryInfo;
pub use dialog::ActionsDialog;
pub use types::ScriptInfo;

// Public API for AI window integration
pub use builders::get_ai_command_bar_actions;
pub use types::{
    Action, ActionCategory, ActionsDialogConfig, AnchorPosition, SearchPosition, SectionStyle,
};

// Window functions for separate vibrancy window
pub use window::{
    close_actions_window, is_actions_window_open, notify_actions_window, open_actions_window,
    resize_actions_window,
};
</file>

<file path="src/actions/types.rs">
//! Action types and data structures

use crate::designs::icon_variations::IconName;
use std::sync::Arc;

/// Callback for action selection
pub type ActionCallback = Arc<dyn Fn(String) + Send + Sync>;

/// Information about the currently focused/selected script
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    pub name: String,
    pub path: String,
    pub is_script: bool,
    pub is_scriptlet: bool,
    pub action_verb: String,
    pub shortcut: Option<String>,
    pub alias: Option<String>,
    pub is_suggested: bool,
    pub frecency_path: Option<String>,
}

impl ScriptInfo {
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
}

/// An action in the menu
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ActionCategory,
    pub shortcut: Option<String>,
    pub has_action: bool,
    pub value: Option<String>,
    pub icon: Option<IconName>,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchPosition {
    Top,
    #[default]
    Bottom,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SectionStyle {
    Headers,
    #[default]
    Separators,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnchorPosition {
    Top,
    #[default]
    Bottom,
}

#[derive(Debug, Clone, Default)]
pub struct ActionsDialogConfig {
    pub search_position: SearchPosition,
    pub section_style: SectionStyle,
    pub anchor: AnchorPosition,
    pub show_icons: bool,
    pub show_footer: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionCategory {
    ScriptContext,
    ScriptOps,
    GlobalOps,
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

    pub fn with_icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_section(mut self, section: impl Into<String>) -> Self {
        self.section = Some(section.into());
        self
    }
}
</file>

<file path="src/actions/constants.rs">
//! Actions dialog constants

pub const POPUP_WIDTH: f32 = 320.0;
pub const POPUP_MAX_HEIGHT: f32 = 400.0;
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;
pub const SEARCH_INPUT_HEIGHT: f32 = 44.0;
pub const ACCENT_BAR_WIDTH: f32 = 3.0;
pub const HEADER_HEIGHT: f32 = 24.0;
pub const ACTION_ROW_INSET: f32 = 6.0;
pub const SELECTION_RADIUS: f32 = 8.0;
pub const KEYCAP_MIN_WIDTH: f32 = 22.0;
pub const KEYCAP_HEIGHT: f32 = 22.0;
</file>
</files>

---

## Implementation Guide

### Step 1: Create a Unified CommandBar Wrapper

Create `src/command_bar.rs`:

```rust
//! Unified Command Bar component
//! 
//! Wraps ActionsDialog + window management + keyboard routing into a single
//! reusable component that can be embedded in any window.

use crate::actions::{
    ActionsDialog, ActionsDialogConfig, Action,
    open_actions_window, close_actions_window, notify_actions_window, resize_actions_window,
};
use gpui::{App, Context, Entity, FocusHandle, Window};
use std::sync::Arc;

pub struct CommandBar {
    dialog: Option<Entity<ActionsDialog>>,
    is_open: bool,
    config: ActionsDialogConfig,
    actions: Vec<Action>,
    on_execute: Arc<dyn Fn(String) + Send + Sync>,
}

impl CommandBar {
    pub fn new(
        actions: Vec<Action>,
        config: ActionsDialogConfig,
        on_execute: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            dialog: None,
            is_open: false,
            config,
            actions,
            on_execute: Arc::new(on_execute),
        }
    }
    
    pub fn toggle(&mut self, window: &mut Window, cx: &mut App) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }
    
    pub fn open(&mut self, window: &mut Window, cx: &mut App) {
        let theme = Arc::new(crate::theme::load_theme());
        let on_select = self.on_execute.clone();
        
        let dialog = cx.new(|cx| {
            ActionsDialog::with_config(
                cx.focus_handle(),
                on_select,
                self.actions.clone(),
                theme,
                self.config.clone(),
            )
        });
        
        let bounds = window.bounds();
        let display_id = window.display(cx).map(|d| d.id());
        
        if let Ok(_) = open_actions_window(cx, bounds, display_id, dialog.clone()) {
            self.dialog = Some(dialog);
            self.is_open = true;
        }
    }
    
    pub fn close(&mut self, cx: &mut App) {
        close_actions_window(cx);
        self.dialog = None;
        self.is_open = false;
    }
    
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Handle keyboard event. Returns true if handled.
    pub fn handle_key(&mut self, key: &str, modifiers: &gpui::Modifiers, cx: &mut App) -> bool {
        if !self.is_open {
            return false;
        }
        
        let Some(dialog) = &self.dialog else { return false };
        
        match key {
            "up" | "arrowup" => {
                dialog.update(cx, |d, cx| d.move_up(cx));
                notify_actions_window(cx);
                true
            }
            "down" | "arrowdown" => {
                dialog.update(cx, |d, cx| d.move_down(cx));
                notify_actions_window(cx);
                true
            }
            "enter" | "return" => {
                if let Some(action_id) = dialog.read(cx).get_selected_action_id() {
                    let callback = self.on_execute.clone();
                    self.close(cx);
                    callback(action_id);
                }
                true
            }
            "escape" => {
                self.close(cx);
                true
            }
            "backspace" | "delete" => {
                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                resize_actions_window(cx, dialog);
                true
            }
            _ => {
                if !modifiers.platform && !modifiers.control && !modifiers.alt {
                    if let Some(ch) = key.chars().next() {
                        if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                            resize_actions_window(cx, dialog);
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}
```

### Step 2: Add Global Shortcut Registry

Create `src/shortcut_registry.rs`:

```rust
//! Global shortcut registry for actions that work even when command bar is closed

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

type ShortcutHandler = Arc<dyn Fn() + Send + Sync>;

static REGISTRY: OnceLock<Mutex<HashMap<String, ShortcutHandler>>> = OnceLock::new();

pub fn register_shortcut(shortcut: &str, handler: impl Fn() + Send + Sync + 'static) {
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut map) = registry.lock() {
        map.insert(shortcut.to_lowercase(), Arc::new(handler));
    }
}

pub fn try_handle_shortcut(shortcut: &str) -> bool {
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(map) = registry.lock() {
        if let Some(handler) = map.get(&shortcut.to_lowercase()) {
            handler();
            return true;
        }
    }
    false
}

pub fn clear_shortcuts() {
    if let Some(registry) = REGISTRY.get() {
        if let Ok(mut map) = registry.lock() {
            map.clear();
        }
    }
}
```

### Step 3: Integrate in Host Window

```rust
impl MyWindow {
    fn new(cx: &mut Context<Self>) -> Self {
        let command_bar = CommandBar::new(
            get_my_actions(),
            ActionsDialogConfig {
                search_position: SearchPosition::Top,
                section_style: SectionStyle::Headers,
                anchor: AnchorPosition::Top,
                show_icons: true,
                show_footer: true,
            },
            |action_id| {
                // Handle action execution
                match action_id.as_str() {
                    "copy" => { /* ... */ }
                    "paste" => { /* ... */ }
                    _ => {}
                }
            },
        );
        
        Self {
            command_bar,
            // ...
        }
    }
}

impl Render for MyWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;
                
                // First try command bar
                if this.command_bar.handle_key(key, modifiers, cx) {
                    return;
                }
                
                // Then try global shortcuts
                let shortcut = format_shortcut(key, modifiers);
                if try_handle_shortcut(&shortcut) {
                    return;
                }
                
                // Toggle with Cmd+K
                if modifiers.platform && key == "k" {
                    this.command_bar.toggle(window, cx);
                }
            }))
            .child(/* ... */)
    }
}
```

---

## Instructions for the Next AI Agent

### Context
You are implementing a reusable Command Bar component for a GPUI (Zed's GPU-accelerated UI framework) desktop application. The existing `ActionsDialog` component provides the core functionality but requires integration with window management, focus handling, and keyboard routing.

### Key Requirements
1. **Search Position Flexibility**: Support `SearchPosition::Top` (AI-style, list grows down) and `SearchPosition::Bottom` (main menu style, list grows up).
2. **Resize Anchoring**: When filtering changes the action count, resize from top (bottom-pinned) or from bottom (top-pinned) based on `AnchorPosition`.
3. **Focus Management**: 
   - Track `FocusedInput` enum to know which input has focus
   - Use `FocusTarget` enum for pending focus requests
   - Apply focus once per request, then clear (avoid "perpetual focus enforcement")
4. **Keyboard Routing**: When command bar is open, route all relevant keys to it. When closed, allow global shortcuts to still work.
5. **Vibrancy Window**: Open as a separate macOS vibrancy window for the floating popup effect.

### Critical Implementation Details
- Always match both arrow key variants: `"up" | "arrowup"` and `"down" | "arrowdown"`
- After updating dialog state, call `notify_actions_window(cx)` to re-render
- After filtering (backspace/char input), call `resize_actions_window(cx, &dialog_entity)`
- Use `cx.spawn().detach()` for async window operations
- The `ActionsDialog` does NOT handle its own keyboard events - the parent routes them

### Testing Approach
Test via stdin JSON protocol:
```bash
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-command-bar.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### Files to Modify
1. Create `src/command_bar.rs` for unified wrapper
2. Create `src/shortcut_registry.rs` for global shortcut handling
3. Update `src/lib.rs` to export new modules
4. Integrate in target windows (AI, main, prompts)

### Success Criteria
- [ ] Command bar opens with Cmd+K in any host window
- [ ] Search filters actions correctly
- [ ] Arrow keys navigate selection
- [ ] Enter executes selected action
- [ ] Escape closes without executing
- [ ] Window resizes dynamically based on filtered count
- [ ] Focus returns to appropriate input after close
- [ ] Shortcuts work even when command bar is closed

OUTPUT_FILE_PATH: expert-bundles/reusable-command-bar.md
