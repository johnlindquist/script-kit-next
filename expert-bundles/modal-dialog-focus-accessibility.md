# Modal Dialog Focus & Accessibility Expert Bundle

## Original Goal

> Please create an expert bundle around all of the modal dialog logic and how we can improve these. I want to make sure we're using the GPUI component library as best as possible, that the modals and the buttons inside of them have focus, meaning that if someone were to tab through the buttons, the appropriate button would be focused, that things like hitting the spacebar would trigger a button that's focused, and that the general behavior of modals just match what everyone would expect from other applications.
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The modal dialog system (ConfirmDialog, ActionsDialog) uses **manual div-based button rendering** instead of the `gpui_component::Button` component that the rest of the codebase uses. This results in:
- Inconsistent focus handling between modals and other parts of the app
- Manual focus state tracking (`focused_button: usize`) instead of GPUI's native focus system
- Missing accessibility features like proper focus rings from the component library

### Key Problems:

1. **ConfirmDialog uses raw `div()` elements for buttons** (`src/confirm/dialog.rs:189-254`) instead of `gpui_component::button::Button`. Focus is tracked manually via `focused_button: usize` rather than using GPUI's `FocusHandle` per button.

2. **Buttons don't have individual focus handles** - The current implementation has a single `focus_handle` for the entire dialog, with manual selection tracking. This prevents proper Tab navigation where each button would be individually focusable.

3. **Keyboard handling is split across files** - Space/Enter/Tab/Arrow handling happens in `ConfirmWindow` (`window.rs`) but the visual focus indication is in `ConfirmDialog` (`dialog.rs`). This separation makes the code harder to maintain.

### Required Fixes:

1. **Replace div-based buttons with `gpui_component::Button`** in `src/confirm/dialog.rs`:
   - Use `Button::new("Cancel", colors).variant(ButtonVariant::Ghost)` 
   - Use `Button::new("OK", colors).variant(ButtonVariant::Primary)`
   - Apply `.focused(is_focused)` to show focus ring

2. **Use gpui_component::Button's focus system** instead of manual `focused_button` tracking:
   - Each button should have its own `FocusHandle` 
   - Use `.track_focus()` for proper Tab navigation
   - Let GPUI handle the focus ring styling

3. **Add proper focus management for modal buttons**:
   - Create a `ConfirmButton` component with `FocusHandle`
   - Implement `Focusable` trait for each button
   - Handle Tab/Shift+Tab, Space, Enter at the button level

### Files Included:

- `src/confirm/dialog.rs`: ConfirmDialog struct - **main target for refactor**
- `src/confirm/window.rs`: ConfirmWindow keyboard handling
- `src/confirm/mod.rs`: Public API exports
- `src/confirm/constants.rs`: Layout constants
- `src/actions/dialog.rs`: ActionsDialog (uses list items, not buttons)
- `src/actions/window.rs`: ActionsWindow keyboard handling pattern
- `src/actions/types.rs`: Action type definitions
- `src/actions/constants.rs`: Layout constants
- `src/components/button.rs`: Custom Button component (good reference)
- `src/focus_coordinator.rs`: Focus management system

### Reference Files (not in bundle but important):

- `src/ai/window.rs`: Uses `gpui_component::button::Button` correctly
- `src/notes/window.rs`: Uses `gpui_component::button::Button` correctly
- `src/components/shortcut_recorder.rs`: Uses custom Button with focus
- `src/components/alias_input.rs`: Uses custom Button with focus

---

## Current Implementation Analysis

### Problem 1: Manual Button Rendering in ConfirmDialog

```rust
// src/confirm/dialog.rs:189-220 - CURRENT (problematic)
let cancel_button = div()
    .id("cancel-btn")
    .flex_1()
    .h(px(44.0))
    // ... lots of manual styling ...
    .bg(if is_cancel_focused { focused_bg } else { unfocused_bg })
    .border_2()
    .border_color(if is_cancel_focused { focus_border } else { unfocused_border })
    .on_click(cx.listener(|this, _e, _window, _cx| {
        this.cancel();
    }))
    .child(self.cancel_text.clone());
```

Compare with how Notes/AI windows use gpui_component::Button:
```rust
// src/notes/window.rs:1240-1246 - CORRECT
Button::new("bold")
    .ghost()
    .xsmall()
    .label("B")
    .on_click(cx.listener(|this, _, _, cx| {
        this.insert_formatting("**", "**", cx);
    }))
```

### Problem 2: Manual Focus Tracking

```rust
// src/confirm/dialog.rs:41-42
/// Which button is currently focused (0 = cancel, 1 = confirm)
pub focused_button: usize,
```

This manual tracking means:
- No integration with GPUI's focus system
- Tab order isn't managed by GPUI
- No automatic focus ring styling from gpui_component

### Problem 3: Split Keyboard Handling

Keyboard events are handled in `ConfirmWindow::render()`:
```rust
// src/confirm/window.rs:76-116
match key {
    "enter" | "return" => {
        this.dialog.update(cx, |d, _cx| d.submit());
    }
    " " | "space" => {
        this.dialog.update(cx, |d, _cx| d.submit());
    }
    "tab" => {
        this.dialog.update(cx, |d, cx| d.toggle_focus(cx));
    }
    // ...
}
```

But focus visual updates are in `ConfirmDialog::render()`:
```rust
// src/confirm/dialog.rs:167-168
let is_cancel_focused = self.focused_button == 0;
let is_confirm_focused = self.focused_button == 1;
```

---

This section contains the contents of the repository's files.

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

use super::constants::{
    ACTION_ITEM_HEIGHT, HEADER_HEIGHT, POPUP_MAX_HEIGHT, SEARCH_INPUT_HEIGHT, SECTION_HEADER_HEIGHT,
};
use super::dialog::ActionsDialog;
use super::types::{Action, SectionStyle};

/// Count the number of section headers in the filtered action list
fn count_section_headers(actions: &[Action], filtered_indices: &[usize]) -> usize {
    if filtered_indices.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut prev_section: Option<&Option<String>> = None;

    for &idx in filtered_indices {
        if let Some(action) = actions.get(idx) {
            let current_section = &action.section;
            if current_section.is_some() {
                match prev_section {
                    None => count += 1,
                    Some(prev) if prev != current_section => count += 1,
                    _ => {}
                }
            }
            prev_section = Some(current_section);
        }
    }

    count
}

/// Global singleton for the actions window handle
static ACTIONS_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

const ACTIONS_WINDOW_WIDTH: f32 = 320.0;
const ACTIONS_MARGIN_X: f32 = 8.0;
const ACTIONS_MARGIN_Y: f32 = 8.0;
#[allow(dead_code)]
const TITLEBAR_HEIGHT: f32 = 36.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum WindowPosition {
    #[default]
    BottomRight,
    TopRight,
    TopCenter,
}

/// ActionsWindow wrapper that renders the shared ActionsDialog entity
pub struct ActionsWindow {
    pub dialog: Entity<ActionsDialog>,
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        let window_is_active = window.is_window_active();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "ActionsWindow render: focus_handle.is_focused={}, window_is_active={}",
                is_focused, window_is_active
            ),
        );

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
                    let action_id = this.dialog.read(cx).get_selected_action_id();
                    if let Some(action_id) = action_id {
                        let callback = this.dialog.read(cx).on_select.clone();
                        callback(action_id.clone());
                        let on_close = this.dialog.read(cx).on_close.clone();
                        if let Some(callback) = on_close {
                            callback(cx);
                        }
                        platform::activate_main_window();
                        window.defer(cx, |window, _cx| {
                            window.remove_window();
                        });
                    }
                }
                "escape" => {
                    let on_close = this.dialog.read(cx).on_close.clone();
                    if let Some(callback) = on_close {
                        callback(cx);
                    }
                    platform::activate_main_window();
                    window.defer(cx, |window, _cx| {
                        window.remove_window();
                    });
                }
                "backspace" | "delete" => {
                    this.dialog.update(cx, |d, cx| d.handle_backspace(cx));
                    let dialog = this.dialog.clone();
                    window.defer(cx, move |window, cx| {
                        resize_actions_window_direct(window, cx, &dialog);
                    });
                    cx.notify();
                }
                _ => {
                    if !modifiers.platform && !modifiers.control && !modifiers.alt {
                        if let Some(ch) = key.chars().next() {
                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_' {
                                this.dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                let dialog = this.dialog.clone();
                                window.defer(cx, move |window, cx| {
                                    resize_actions_window_direct(window, cx, &dialog);
                                });
                                cx.notify();
                            }
                        }
                    }
                }
            }
        });

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(self.dialog.clone())
    }
}

pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
    position: WindowPosition,
) -> anyhow::Result<WindowHandle<Root>> {
    close_actions_window(cx);

    let theme = theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let dialog = dialog_entity.read(cx);
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.context_title.is_some();

    let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
        count_section_headers(&dialog.actions, &dialog.filtered_actions)
    } else {
        0
    };

    let search_box_height = if hide_search { 0.0 } else { SEARCH_INPUT_HEIGHT };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = 2.0;
    let dynamic_height = items_height + search_box_height + header_height + border_height;

    let window_width = px(ACTIONS_WINDOW_WIDTH);
    let window_height = px(dynamic_height);

    let window_x = main_window_bounds.origin.x + main_window_bounds.size.width
        - window_width
        - px(ACTIONS_MARGIN_X);

    let window_y = match position {
        WindowPosition::BottomRight => {
            main_window_bounds.origin.y + main_window_bounds.size.height
                - window_height
                - px(FOOTER_HEIGHT)
                - px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopRight => {
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopCenter => {
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
    };

    let window_x = match position {
        WindowPosition::TopCenter => {
            main_window_bounds.origin.x + (main_window_bounds.size.width - window_width) / 2.0
        }
        _ => window_x,
    };

    let bounds = Bounds {
        origin: Point { x: window_x, y: window_y },
        size: Size { width: window_width, height: window_height },
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: false,
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    let handle = cx.open_window(window_options, |window, cx| {
        let actions_window = cx.new(|cx| ActionsWindow::new(dialog_entity, cx));
        cx.new(|cx| Root::new(actions_window, window, cx))
    })?;

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, |_root, window, cx| {
            window.defer(cx, |_window, _cx| {
                use cocoa::appkit::NSApp;
                use cocoa::base::nil;
                use objc::{msg_send, sel, sel_impl};

                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let windows: cocoa::base::id = msg_send![app, windows];
                    let count: usize = msg_send![windows, count];
                    if count > 0 {
                        let ns_window: cocoa::base::id = msg_send![windows, lastObject];
                        if ns_window != nil {
                            platform::configure_actions_popup_window(ns_window);
                        }
                    }
                }
            });
        });
    }

    let window_storage = ACTIONS_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }

    Ok(handle)
}

pub fn close_actions_window(cx: &mut App) {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(mut guard) = window_storage.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.update(cx, |_root, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

pub fn is_actions_window_open() -> bool {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return guard.is_some();
        }
    }
    false
}

pub fn get_actions_window_handle() -> Option<WindowHandle<Root>> {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return *guard;
        }
    }
    None
}

pub fn notify_actions_window(cx: &mut App) {
    if let Some(handle) = get_actions_window_handle() {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

pub fn resize_actions_window_direct(
    window: &mut Window,
    cx: &mut App,
    dialog_entity: &Entity<ActionsDialog>,
) {
    let dialog = dialog_entity.read(cx);
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.context_title.is_some();

    let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
        count_section_headers(&dialog.actions, &dialog.filtered_actions)
    } else {
        0
    };

    let search_box_height = if hide_search { 0.0 } else { SEARCH_INPUT_HEIGHT };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
    let min_items_height = if num_actions == 0 { ACTION_ITEM_HEIGHT } else { 0.0 };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
        .max(min_items_height)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = 2.0;
    let new_height_f32 = items_height + search_box_height + header_height + border_height;

    let current_bounds = window.bounds();
    let current_height_f32: f32 = current_bounds.size.height.into();

    if (current_height_f32 - new_height_f32).abs() < 1.0 {
        return;
    }

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
            let current_width_f32: f32 = current_bounds.size.width.into();

            for i in 0..count {
                let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                if ns_window == nil { continue; }

                let frame: NSRect = msg_send![ns_window, frame];

                if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                    && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                {
                    let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                    if window_screen == nil {
                        let screens: cocoa::base::id = NSScreen::screens(nil);
                        let _primary: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
                    }

                    let new_frame = NSRect::new(
                        NSPoint::new(frame.origin.x, frame.origin.y),
                        NSSize::new(frame.size.width, new_height_f32 as f64),
                    );

                    let _: () = msg_send![ns_window, setFrame:new_frame display:true animate:false];
                    break;
                }
            }
        }
    }

    window.resize(gpui::Size {
        width: current_bounds.size.width,
        height: px(new_height_f32),
    });
}

pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>) {
    if let Some(handle) = get_actions_window_handle() {
        let dialog = dialog_entity.read(cx);
        let num_actions = dialog.filtered_actions.len();
        let hide_search = dialog.hide_search;
        let has_header = dialog.context_title.is_some();

        let section_header_count = if dialog.config.section_style == SectionStyle::Headers {
            count_section_headers(&dialog.actions, &dialog.filtered_actions)
        } else {
            0
        };

        let search_box_height = if hide_search { 0.0 } else { SEARCH_INPUT_HEIGHT };
        let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
        let section_headers_height = section_header_count as f32 * SECTION_HEADER_HEIGHT;
        let min_items_height = if num_actions == 0 { ACTION_ITEM_HEIGHT } else { 0.0 };
        let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT + section_headers_height)
            .max(min_items_height)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let border_height = 2.0;
        let new_height_f32 = items_height + search_box_height + header_height + border_height;

        let _ = handle.update(cx, |_root, window, cx| {
            let current_bounds = window.bounds();
            let current_height_f32: f32 = current_bounds.size.height.into();

            if (current_height_f32 - new_height_f32).abs() < 1.0 {
                return;
            }

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
                    let current_width_f32: f32 = current_bounds.size.width.into();

                    for i in 0..count {
                        let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                        if ns_window == nil { continue; }

                        let frame: NSRect = msg_send![ns_window, frame];

                        if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                            && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                        {
                            let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                            if window_screen == nil {
                                let screens: cocoa::base::id = NSScreen::screens(nil);
                                let _primary: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
                            }

                            let new_frame = NSRect::new(
                                NSPoint::new(frame.origin.x, frame.origin.y),
                                NSSize::new(frame.size.width, new_height_f32 as f64),
                            );

                            let _: () = msg_send![ns_window, setFrame:new_frame display:true animate:false];
                            break;
                        }
                    }
                }
            }

            window.resize(Size {
                width: current_bounds.size.width,
                height: px(new_height_f32),
            });
            cx.notify();
        });
    }
}
</file>

<file path="src/confirm/dialog.rs">
//! Confirm Dialog
//!
//! A simple confirmation dialog with a message and two buttons (Cancel/Confirm).
//! Supports keyboard shortcuts: Enter = confirm, Escape = cancel.
//! Tab/Arrow keys navigate between buttons with visual focus indication.

use crate::logging;
use crate::theme;
use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, FontWeight, Render,
    SharedString, Window,
};
use std::sync::Arc;

use super::constants::{BUTTON_GAP, CONFIRM_PADDING, CONFIRM_WIDTH, DIALOG_RADIUS};

/// Callback for confirm/cancel selection
pub type ConfirmCallback = Arc<dyn Fn(bool) + Send + Sync>;

#[inline]
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    (hex << 8) | (alpha as u32)
}

/// ConfirmDialog - Simple confirmation modal with message and two buttons
pub struct ConfirmDialog {
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub focus_handle: FocusHandle,
    pub on_choice: ConfirmCallback,
    pub theme: Arc<theme::Theme>,
    /// Which button is currently focused (0 = cancel, 1 = confirm)
    pub focused_button: usize,
}

impl ConfirmDialog {
    pub fn new(
        message: impl Into<String>,
        confirm_text: Option<String>,
        cancel_text: Option<String>,
        focus_handle: FocusHandle,
        on_choice: ConfirmCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let message_str = message.into();
        logging::log("CONFIRM", &format!("ConfirmDialog created: {:?}", message_str));

        Self {
            message: message_str,
            confirm_text: confirm_text.unwrap_or_else(|| "OK".to_string()),
            cancel_text: cancel_text.unwrap_or_else(|| "Cancel".to_string()),
            focus_handle,
            on_choice,
            theme,
            focused_button: 1, // Default focus on confirm button
        }
    }

    pub fn focus_cancel(&mut self, cx: &mut Context<Self>) {
        if self.focused_button != 0 {
            self.focused_button = 0;
            cx.notify();
        }
    }

    pub fn focus_confirm(&mut self, cx: &mut Context<Self>) {
        if self.focused_button != 1 {
            self.focused_button = 1;
            cx.notify();
        }
    }

    pub fn toggle_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_button = 1 - self.focused_button;
        cx.notify();
    }

    pub fn submit(&mut self) {
        let confirmed = self.focused_button == 1;
        logging::log("CONFIRM", &format!("User chose: {}", if confirmed { "confirm" } else { "cancel" }));
        (self.on_choice)(confirmed);
    }

    pub fn cancel(&mut self) {
        logging::log("CONFIRM", "User cancelled");
        (self.on_choice)(false);
    }

    pub fn confirm(&mut self) {
        logging::log("CONFIRM", "User confirmed");
        (self.on_choice)(true);
    }
}

impl Focusable for ConfirmDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConfirmDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        logging::log("CONFIRM", &format!("ConfirmDialog::render() called, focused_button={}", self.focused_button));

        let colors = &self.theme.colors;

        let use_vibrancy = self.theme.is_vibrancy_enabled();
        let dialog_alpha = if use_vibrancy {
            let opacity = self.theme.opacity.as_ref().map(|o| o.main).unwrap_or(0.37).clamp(0.25, 0.50);
            (opacity * 255.0) as u8
        } else {
            (0.95 * 255.0) as u8
        };
        let main_bg = rgba(hex_with_alpha(colors.background.main, dialog_alpha));
        let primary_text = rgb(colors.text.primary);
        let border_color = rgba(hex_with_alpha(colors.ui.border, 0x60));
        let message_str: SharedString = self.message.clone().into();

        let is_cancel_focused = self.focused_button == 0;
        let is_confirm_focused = self.focused_button == 1;

        let accent_hex = colors.accent.selected;
        let accent_color = rgb(accent_hex);
        let focused_bg = rgba(hex_with_alpha(accent_hex, 0x66));
        let focused_text = rgb(0xFFFFFF);
        let unfocused_bg = rgba(0xffffff10);
        let hover_bg = rgba(0xffffff20);
        let focus_border = rgba(hex_with_alpha(accent_hex, 0xFF));
        let unfocused_border = rgba(0xffffff30);

        // PROBLEM: Using div() instead of gpui_component::Button
        let cancel_button = div()
            .id("cancel-btn")
            .flex_1()
            .h(px(44.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.0))
            .cursor_pointer()
            .font_weight(FontWeight::MEDIUM)
            .bg(if is_cancel_focused { focused_bg } else { unfocused_bg })
            .text_color(if is_cancel_focused { focused_text } else { accent_color })
            .border_2()
            .border_color(if is_cancel_focused { focus_border } else { unfocused_border })
            .hover(|style| style.bg(hover_bg))
            .on_click(cx.listener(|this, _e, _window, _cx| {
                this.cancel();
            }))
            .child(self.cancel_text.clone());

        let confirm_button = div()
            .id("confirm-btn")
            .flex_1()
            .h(px(44.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.0))
            .cursor_pointer()
            .font_weight(FontWeight::MEDIUM)
            .bg(if is_confirm_focused { focused_bg } else { unfocused_bg })
            .text_color(if is_confirm_focused { focused_text } else { accent_color })
            .border_2()
            .border_color(if is_confirm_focused { focus_border } else { unfocused_border })
            .hover(|style| style.bg(hover_bg))
            .on_click(cx.listener(|this, _e, _window, _cx| {
                this.confirm();
            }))
            .child(self.confirm_text.clone());

        let button_row = div()
            .w_full()
            .flex()
            .flex_row()
            .gap(px(BUTTON_GAP))
            .child(cancel_button)
            .child(confirm_button);

        let _ = main_bg;
        div()
            .w(px(CONFIRM_WIDTH))
            .flex()
            .flex_col()
            .p(px(CONFIRM_PADDING))
            .gap(px(CONFIRM_PADDING))
            .rounded(px(DIALOG_RADIUS))
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .text_color(primary_text)
                    .text_base()
                    .text_center()
                    .child(message_str),
            )
            .child(button_row)
    }
}
</file>

<file path="src/confirm/window.rs">
//! Confirm Window - Separate vibrancy window for confirmation dialog

use crate::platform;
use crate::theme;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, DisplayId, Entity, FocusHandle, Focusable, Pixels,
    Point, Render, Size, Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use gpui_component::Root;
use std::sync::{Mutex, OnceLock};

use super::constants::{CONFIRM_HEIGHT, CONFIRM_WIDTH};
use super::dialog::{ConfirmCallback, ConfirmDialog};

static CONFIRM_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();
static CONFIRM_DIALOG: OnceLock<Mutex<Option<Entity<ConfirmDialog>>>> = OnceLock::new();

pub struct ConfirmWindow {
    pub dialog: Entity<ConfirmDialog>,
    pub focus_handle: FocusHandle,
}

impl ConfirmWindow {
    pub fn new(dialog: Entity<ConfirmDialog>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self { dialog, focus_handle }
    }
}

impl Focusable for ConfirmWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConfirmWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_handle.is_focused(window) {
            crate::logging::log("CONFIRM", "ConfirmWindow: focus_handle NOT focused, re-focusing");
            self.focus_handle.focus(window, cx);
        }

        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();

            crate::logging::log("CONFIRM", &format!("ConfirmWindow on_key_down received: key='{}'", key));

            match key {
                "enter" | "return" => {
                    crate::logging::log("CONFIRM", "Enter pressed - submitting");
                    this.dialog.update(cx, |d, _cx| d.submit());
                }
                " " | "space" => {
                    crate::logging::log("CONFIRM", "Space pressed - submitting");
                    this.dialog.update(cx, |d, _cx| d.submit());
                }
                "escape" => {
                    crate::logging::log("CONFIRM", "Escape pressed - cancelling");
                    this.dialog.update(cx, |d, _cx| d.cancel());
                }
                "tab" => {
                    this.dialog.update(cx, |d, cx| {
                        d.toggle_focus(cx);
                        crate::logging::log("CONFIRM", &format!("Tab pressed, focused_button now: {}", d.focused_button));
                    });
                    cx.notify();
                }
                "left" | "arrowleft" => {
                    crate::logging::log("CONFIRM", "Left arrow - focusing cancel");
                    this.dialog.update(cx, |d, cx| d.focus_cancel(cx));
                    cx.notify();
                }
                "right" | "arrowright" => {
                    crate::logging::log("CONFIRM", "Right arrow - focusing confirm");
                    this.dialog.update(cx, |d, cx| d.focus_confirm(cx));
                    cx.notify();
                }
                _ => {}
            }
        });

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(self.dialog.clone())
    }
}

pub fn init_confirm_bindings(_cx: &mut App) {
    crate::logging::log("CONFIRM", "init_confirm_bindings called (no-op, keys handled via on_key_down)");
}

pub fn open_confirm_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    message: String,
    confirm_text: Option<String>,
    cancel_text: Option<String>,
    on_choice: ConfirmCallback,
) -> anyhow::Result<(WindowHandle<Root>, Entity<ConfirmDialog>)> {
    close_confirm_window(cx);

    let theme = theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    let window_width = px(CONFIRM_WIDTH);
    let window_height = px(CONFIRM_HEIGHT);

    let window_x = main_window_bounds.origin.x + (main_window_bounds.size.width - window_width) / 2.0;
    let window_y = main_window_bounds.origin.y + (main_window_bounds.size.height - window_height) / 2.0;

    let bounds = Bounds {
        origin: Point { x: window_x, y: window_y },
        size: Size { width: window_width, height: window_height },
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: true,
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    let theme_arc = std::sync::Arc::new(theme);
    let mut dialog_entity_holder: Option<Entity<ConfirmDialog>> = None;

    let handle = cx.open_window(window_options, |window, cx| {
        let dialog = cx.new(|cx| {
            ConfirmDialog::new(message, confirm_text, cancel_text, cx.focus_handle(), on_choice, theme_arc.clone())
        });

        dialog_entity_holder = Some(dialog.clone());

        let confirm_window = cx.new(|cx| {
            let cw = ConfirmWindow::new(dialog, cx);
            cw.focus_handle.focus(window, cx);
            cw
        });

        cx.new(|cx| Root::new(confirm_window, window, cx))
    })?;

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, |_root, window, cx| {
            window.defer(cx, |_window, _cx| {
                use cocoa::appkit::NSApp;
                use cocoa::base::nil;
                use objc::{msg_send, sel, sel_impl};

                unsafe {
                    let app: cocoa::base::id = NSApp();
                    let windows: cocoa::base::id = msg_send![app, windows];
                    let count: usize = msg_send![windows, count];
                    if count > 0 {
                        let ns_window: cocoa::base::id = msg_send![windows, lastObject];
                        if ns_window != nil {
                            platform::configure_actions_popup_window(ns_window);
                        }
                    }
                }
            });
        });
    }

    let window_storage = CONFIRM_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }

    let dialog_entity = dialog_entity_holder.expect("Dialog entity should have been created");

    let dialog_storage = CONFIRM_DIALOG.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = dialog_storage.lock() {
        *guard = Some(dialog_entity.clone());
    }

    crate::logging::log("CONFIRM", "Confirm popup window opened with vibrancy");

    Ok((handle, dialog_entity))
}

pub fn close_confirm_window(cx: &mut App) {
    if let Some(dialog_storage) = CONFIRM_DIALOG.get() {
        if let Ok(mut guard) = dialog_storage.lock() {
            *guard = None;
        }
    }

    if let Some(window_storage) = CONFIRM_WINDOW.get() {
        if let Ok(mut guard) = window_storage.lock() {
            if let Some(handle) = guard.take() {
                crate::logging::log("CONFIRM", "Closing confirm popup window");
                let _ = handle.update(cx, |_root, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

#[allow(dead_code)]
pub fn is_confirm_window_open() -> bool {
    if let Some(window_storage) = CONFIRM_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return guard.is_some();
        }
    }
    false
}

#[allow(dead_code)]
pub fn get_confirm_window_handle() -> Option<WindowHandle<Root>> {
    if let Some(window_storage) = CONFIRM_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return *guard;
        }
    }
    None
}

#[allow(dead_code)]
pub fn notify_confirm_window(cx: &mut App) {
    if let Some(handle) = get_confirm_window_handle() {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

pub fn dispatch_confirm_key(key: &str, cx: &mut App) -> bool {
    let dialog_opt = if let Some(storage) = CONFIRM_DIALOG.get() {
        if let Ok(guard) = storage.lock() {
            guard.clone()
        } else {
            None
        }
    } else {
        None
    };

    let Some(dialog) = dialog_opt else {
        return false;
    };

    crate::logging::log("CONFIRM", &format!("Dispatching key to confirm dialog: {}", key));

    match key {
        "enter" | "Enter" => {
            dialog.update(cx, |d, _cx| d.submit());
            close_confirm_window(cx);
            true
        }
        "space" | "Space" | " " => {
            dialog.update(cx, |d, _cx| d.submit());
            close_confirm_window(cx);
            true
        }
        "escape" | "Escape" => {
            dialog.update(cx, |d, _cx| d.cancel());
            close_confirm_window(cx);
            true
        }
        "tab" | "Tab" => {
            dialog.update(cx, |d, cx| {
                d.toggle_focus(cx);
                crate::logging::log("CONFIRM", &format!("Tab pressed, focused_button now: {}", d.focused_button));
            });
            notify_confirm_window(cx);
            true
        }
        "left" | "arrowleft" | "Left" | "ArrowLeft" => {
            dialog.update(cx, |d, cx| d.focus_cancel(cx));
            notify_confirm_window(cx);
            true
        }
        "right" | "arrowright" | "Right" | "ArrowRight" => {
            dialog.update(cx, |d, cx| d.focus_confirm(cx));
            notify_confirm_window(cx);
            true
        }
        _ => false,
    }
}
</file>

<file path="src/confirm/mod.rs">
//! Confirm Module
//!
//! A modal confirmation dialog that appears as a floating window.
//! Used by the SDK `confirm()` function to get user confirmation for actions.

mod constants;
mod dialog;
mod window;

pub use dialog::ConfirmCallback;
pub use window::{
    close_confirm_window, dispatch_confirm_key, init_confirm_bindings, is_confirm_window_open,
    open_confirm_window,
};
</file>

<file path="src/confirm/constants.rs">
//! Confirm dialog constants

pub const CONFIRM_WIDTH: f32 = 340.0;
pub const CONFIRM_HEIGHT: f32 = 140.0;
pub const CONFIRM_PADDING: f32 = 20.0;
pub const BUTTON_GAP: f32 = 12.0;
pub const DIALOG_RADIUS: f32 = 12.0;
</file>

<file path="src/components/button.rs">
//! Reusable Button component for GPUI Script Kit
//!
//! This module provides a theme-aware button component with multiple variants
//! and support for hover states, click handlers, and keyboard shortcuts.

#![allow(dead_code)]

use gpui::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Ghost,
    Icon,
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonColors {
    pub text_color: u32,
    #[allow(dead_code)]
    pub text_hover: u32,
    pub background: u32,
    pub background_hover: u32,
    pub accent: u32,
    pub border: u32,
    pub focus_ring: u32,
    pub focus_tint: u32,
}

impl ButtonColors {
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        Self {
            text_color: theme.colors.accent.selected,
            text_hover: theme.colors.text.primary,
            background: theme.colors.accent.selected_subtle,
            background_hover: theme.colors.accent.selected_subtle,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
            focus_ring: theme.colors.accent.selected,
            focus_tint: theme.colors.accent.selected_subtle,
        }
    }

    pub fn from_design(colors: &crate::designs::DesignColors) -> Self {
        Self {
            text_color: colors.accent,
            text_hover: colors.text_primary,
            background: colors.background_selected,
            background_hover: colors.background_hover,
            accent: colors.accent,
            border: colors.border,
            focus_ring: colors.accent,
            focus_tint: colors.background_selected,
        }
    }
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            text_color: 0xfbbf24,
            text_hover: 0xffffff,
            background: 0x2a2a2a,
            background_hover: 0x323232,
            accent: 0xfbbf24,
            border: 0x464647,
            focus_ring: 0xfbbf24,
            focus_tint: 0x2a2a2a,
        }
    }
}

pub type OnClickCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Button {
    label: SharedString,
    colors: ButtonColors,
    variant: ButtonVariant,
    shortcut: Option<String>,
    disabled: bool,
    focused: bool,
    on_click: Option<Rc<OnClickCallback>>,
}

impl Button {
    pub fn new(label: impl Into<SharedString>, colors: ButtonColors) -> Self {
        Self {
            label: label.into(),
            colors,
            variant: ButtonVariant::default(),
            shortcut: None,
            disabled: false,
            focused: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn shortcut_opt(mut self, shortcut: Option<String>) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn on_click(mut self, callback: OnClickCallback) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

const FOCUS_BORDER_WIDTH: f32 = 2.0;

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let variant = self.variant;
        let disabled = self.disabled;
        let focused = self.focused;
        let on_click_callback = self.on_click;
        let label_for_log = self.label.clone();

        let hover_overlay = rgba(0xffffff26);
        let focus_ring_color = rgba((colors.focus_ring << 8) | 0xA0);
        let focus_tint = rgba((colors.focus_tint << 8) | 0x20);
        let unfocused_border = rgba((colors.border << 8) | 0x40);

        let (text_color, bg_color, hover_bg) = match variant {
            ButtonVariant::Primary => {
                let base_bg = rgba((colors.background << 8) | 0x80);
                let bg = if focused {
                    rgba((colors.background << 8) | 0xA0)
                } else {
                    base_bg
                };
                (rgb(colors.accent), bg, rgba((colors.background_hover << 8) | 0xB0))
            }
            ButtonVariant::Ghost => {
                let bg = if focused { focus_tint } else { rgba(0x00000000) };
                (rgb(colors.accent), bg, hover_overlay)
            }
            ButtonVariant::Icon => {
                let bg = if focused { focus_tint } else { rgba(0x00000000) };
                (rgb(colors.accent), bg, hover_overlay)
            }
        };

        let shortcut_element = if let Some(sc) = self.shortcut {
            div().flex().items_center().text_xs().ml(rems(0.25)).child(sc)
        } else {
            div()
        };

        let (px_val, py_val) = match variant {
            ButtonVariant::Primary => (rems(0.75), rems(0.375)),
            ButtonVariant::Ghost => (rems(0.5), rems(0.25)),
            ButtonVariant::Icon => (rems(0.375), rems(0.375)),
        };

        let mut button = div()
            .id(ElementId::Name(self.label.clone()))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(rems(0.125))
            .px(px_val)
            .py(py_val)
            .rounded(px(6.))
            .bg(bg_color)
            .text_color(text_color)
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .font_family(".AppleSystemUIFont")
            .cursor_pointer()
            .child(self.label)
            .child(shortcut_element);

        if focused {
            button = button.border(px(FOCUS_BORDER_WIDTH)).border_color(focus_ring_color);
        } else {
            button = button.border_1().border_color(unfocused_border);
        }

        if !disabled {
            button = button.hover(move |s| s.bg(hover_bg));
        } else {
            button = button.opacity(0.5).cursor_default();
        }

        if let Some(callback) = on_click_callback {
            if !disabled {
                button = button.on_click(move |event, window, cx| {
                    tracing::debug!(button = %label_for_log, "Button clicked");
                    callback(event, window, cx);
                });
            }
        }

        button
    }
}
</file>

<file path="src/focus_coordinator.rs">
//! Focus Coordinator - Centralized focus management for Script Kit GPUI

use crate::logging;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorOwner {
    MainFilter,
    ActionsSearch,
    ArgPrompt,
    ChatPrompt,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FocusTarget {
    MainFilter,
    ActionsDialog,
    ArgPrompt,
    PathPrompt,
    FormPrompt,
    EditorPrompt,
    SelectPrompt,
    EnvPrompt,
    DropPrompt,
    TemplatePrompt,
    TermPrompt,
    ChatPrompt,
    DivPrompt,
    ScratchPad,
    QuickTerminal,
}

impl FocusTarget {
    pub fn default_cursor_owner(self) -> CursorOwner {
        match self {
            FocusTarget::MainFilter => CursorOwner::MainFilter,
            FocusTarget::ActionsDialog => CursorOwner::ActionsSearch,
            FocusTarget::ArgPrompt => CursorOwner::ArgPrompt,
            FocusTarget::ChatPrompt => CursorOwner::ChatPrompt,
            FocusTarget::PathPrompt
            | FocusTarget::FormPrompt
            | FocusTarget::EditorPrompt
            | FocusTarget::SelectPrompt
            | FocusTarget::EnvPrompt
            | FocusTarget::DropPrompt
            | FocusTarget::TemplatePrompt
            | FocusTarget::TermPrompt
            | FocusTarget::DivPrompt
            | FocusTarget::ScratchPad
            | FocusTarget::QuickTerminal => CursorOwner::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusRequest {
    pub target: FocusTarget,
    pub cursor: CursorOwner,
}

impl FocusRequest {
    #[allow(dead_code)]
    pub fn new(target: FocusTarget, cursor: CursorOwner) -> Self {
        Self { target, cursor }
    }

    pub fn with_default_cursor(target: FocusTarget) -> Self {
        Self { target, cursor: target.default_cursor_owner() }
    }

    pub fn main_filter() -> Self { Self::with_default_cursor(FocusTarget::MainFilter) }
    pub fn actions_dialog() -> Self { Self::with_default_cursor(FocusTarget::ActionsDialog) }
    pub fn arg_prompt() -> Self { Self::with_default_cursor(FocusTarget::ArgPrompt) }
    pub fn chat_prompt() -> Self { Self::with_default_cursor(FocusTarget::ChatPrompt) }
    #[allow(dead_code)]
    pub fn div_prompt() -> Self { Self::with_default_cursor(FocusTarget::DivPrompt) }
    #[allow(dead_code)]
    pub fn form_prompt() -> Self { Self::with_default_cursor(FocusTarget::FormPrompt) }
    #[allow(dead_code)]
    pub fn path_prompt() -> Self { Self::with_default_cursor(FocusTarget::PathPrompt) }
    #[allow(dead_code)]
    pub fn editor_prompt() -> Self { Self::with_default_cursor(FocusTarget::EditorPrompt) }
    #[allow(dead_code)]
    pub fn select_prompt() -> Self { Self::with_default_cursor(FocusTarget::SelectPrompt) }
    #[allow(dead_code)]
    pub fn env_prompt() -> Self { Self::with_default_cursor(FocusTarget::EnvPrompt) }
    #[allow(dead_code)]
    pub fn drop_prompt() -> Self { Self::with_default_cursor(FocusTarget::DropPrompt) }
    #[allow(dead_code)]
    pub fn template_prompt() -> Self { Self::with_default_cursor(FocusTarget::TemplatePrompt) }
    #[allow(dead_code)]
    pub fn term_prompt() -> Self { Self::with_default_cursor(FocusTarget::TermPrompt) }
    #[allow(dead_code)]
    pub fn scratchpad() -> Self { Self::with_default_cursor(FocusTarget::ScratchPad) }
    #[allow(dead_code)]
    pub fn quick_terminal() -> Self { Self::with_default_cursor(FocusTarget::QuickTerminal) }
}

#[derive(Debug, Default)]
pub struct FocusCoordinator {
    pending: Option<FocusRequest>,
    restore_stack: Vec<FocusRequest>,
    current_cursor_owner: CursorOwner,
}

impl FocusCoordinator {
    #[allow(dead_code)]
    pub fn new() -> Self { Self::default() }

    pub fn with_main_filter_focus() -> Self {
        Self {
            pending: Some(FocusRequest::main_filter()),
            restore_stack: Vec::new(),
            current_cursor_owner: CursorOwner::MainFilter,
        }
    }

    pub fn request(&mut self, request: FocusRequest) {
        logging::log("FOCUS", &format!("Focus request: target={:?}, cursor={:?}", request.target, request.cursor));
        self.pending = Some(request);
    }

    #[allow(dead_code)]
    pub fn request_target(&mut self, target: FocusTarget) {
        self.request(FocusRequest::with_default_cursor(target));
    }

    #[allow(dead_code)]
    pub fn take_pending(&mut self) -> Option<FocusRequest> {
        let request = self.pending.take();
        if let Some(ref req) = request {
            self.current_cursor_owner = req.cursor;
            logging::log("FOCUS", &format!("Applying pending focus: target={:?}, cursor={:?}", req.target, req.cursor));
        }
        request
    }

    #[allow(dead_code)]
    pub fn has_pending(&self) -> bool { self.pending.is_some() }

    pub fn peek_pending(&self) -> Option<&FocusRequest> { self.pending.as_ref() }

    pub fn push_overlay(&mut self, overlay_request: FocusRequest) {
        let saved = self.infer_current_request();
        logging::log("FOCUS", &format!("Pushing overlay: {:?} (saving: {:?})", overlay_request.target, saved.target));
        self.restore_stack.push(saved);
        self.request(overlay_request);
    }

    pub fn pop_overlay(&mut self) {
        let restored = self.restore_stack.pop().unwrap_or_else(|| {
            logging::log("FOCUS", "Restore stack empty, falling back to MainFilter");
            FocusRequest::main_filter()
        });
        logging::log("FOCUS", &format!("Popping overlay, restoring to: {:?}", restored.target));
        self.request(restored);
    }

    pub fn clear_overlays(&mut self) {
        if !self.restore_stack.is_empty() {
            logging::log("FOCUS", &format!("Clearing {} overlay(s) from stack", self.restore_stack.len()));
        }
        self.restore_stack.clear();
        self.request(FocusRequest::main_filter());
    }

    #[allow(dead_code)]
    pub fn overlay_depth(&self) -> usize { self.restore_stack.len() }
    #[allow(dead_code)]
    pub fn has_overlay(&self) -> bool { !self.restore_stack.is_empty() }
    pub fn cursor_owner(&self) -> CursorOwner { self.current_cursor_owner }

    #[allow(dead_code)]
    pub fn set_cursor_owner(&mut self, owner: CursorOwner) {
        self.current_cursor_owner = owner;
    }

    fn infer_current_request(&self) -> FocusRequest {
        match self.current_cursor_owner {
            CursorOwner::MainFilter => FocusRequest::main_filter(),
            CursorOwner::ActionsSearch => FocusRequest::actions_dialog(),
            CursorOwner::ArgPrompt => FocusRequest::arg_prompt(),
            CursorOwner::ChatPrompt => FocusRequest::chat_prompt(),
            CursorOwner::None => FocusRequest::main_filter(),
        }
    }
}

impl CursorOwner {
    #[allow(dead_code)]
    pub fn from_legacy(s: &str) -> Self {
        match s {
            "MainFilter" => CursorOwner::MainFilter,
            "ActionsSearch" => CursorOwner::ActionsSearch,
            "ArgPrompt" => CursorOwner::ArgPrompt,
            "None" => CursorOwner::None,
            _ => CursorOwner::None,
        }
    }
}
</file>

---

## Implementation Guide

### Step 1: Replace div-based buttons with gpui_component::Button in ConfirmDialog

**File: `src/confirm/dialog.rs`**

Replace the manual button rendering with the proper Button component:

```rust
// File: src/confirm/dialog.rs
// Location: Render impl, lines ~189-264

// Add imports at top:
use crate::components::{Button, ButtonColors, ButtonVariant};

// In render(), replace the manual div-based buttons with:

let button_colors = ButtonColors::from_theme(&self.theme);

let cancel_button = Button::new(&self.cancel_text, button_colors)
    .variant(ButtonVariant::Ghost)
    .focused(is_cancel_focused)
    .shortcut("Esc")
    .on_click(Box::new({
        let on_choice = self.on_choice.clone();
        move |_, _, _| {
            (on_choice)(false);
        }
    }));

let confirm_button = Button::new(&self.confirm_text, button_colors)
    .variant(ButtonVariant::Primary)
    .focused(is_confirm_focused)
    .shortcut("↵")
    .on_click(Box::new({
        let on_choice = self.on_choice.clone();
        move |_, _, _| {
            (on_choice)(true);
        }
    }));
```

### Step 2: Consider creating FocusableButton for proper Tab navigation

For full keyboard accessibility, each button needs its own FocusHandle:

```rust
// File: src/components/focusable_button.rs (new file)

use gpui::{App, Context, Entity, FocusHandle, Focusable, Render, Window};
use crate::components::{Button, ButtonColors, ButtonVariant};

pub struct FocusableButton {
    focus_handle: FocusHandle,
    label: String,
    colors: ButtonColors,
    variant: ButtonVariant,
    on_click: Box<dyn Fn() + Send + Sync>,
}

impl FocusableButton {
    pub fn new(
        label: impl Into<String>,
        colors: ButtonColors,
        on_click: Box<dyn Fn() + Send + Sync>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            label: label.into(),
            colors,
            variant: ButtonVariant::Primary,
            on_click,
        }
    }
    
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Focusable for FocusableButton {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FocusableButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, _| {
                match event.keystroke.key.as_str() {
                    " " | "space" | "enter" | "return" => {
                        (this.on_click)();
                    }
                    _ => {}
                }
            }))
            .child(
                Button::new(&self.label, self.colors)
                    .variant(self.variant)
                    .focused(is_focused)
            )
    }
}
```

### Step 3: Update ConfirmDialog to use FocusableButton entities

```rust
// File: src/confirm/dialog.rs

pub struct ConfirmDialog {
    pub message: String,
    pub focus_handle: FocusHandle,  // Dialog-level focus for Escape
    pub cancel_button: Entity<FocusableButton>,
    pub confirm_button: Entity<FocusableButton>,
    // Remove: focused_button: usize
}

impl ConfirmDialog {
    pub fn new(..., cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        
        let cancel_button = cx.new(|cx| {
            FocusableButton::new("Cancel", colors, on_cancel_callback, cx)
                .variant(ButtonVariant::Ghost)
        });
        
        let confirm_button = cx.new(|cx| {
            FocusableButton::new("OK", colors, on_confirm_callback, cx)
                .variant(ButtonVariant::Primary)
        });
        
        Self { message, focus_handle, cancel_button, confirm_button }
    }
}
```

### Step 4: Handle Tab navigation between buttons

With proper FocusHandle per button, Tab navigation works automatically via GPUI's focus system:

```rust
// File: src/confirm/window.rs

impl Render for ConfirmWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handle_key = cx.listener(|this, event: &KeyDownEvent, window, cx| {
            match event.keystroke.key.as_str() {
                "escape" => {
                    // Close dialog, trigger cancel
                    this.dialog.update(cx, |d, _| d.cancel());
                }
                "tab" => {
                    // GPUI handles Tab navigation automatically when buttons
                    // have their own FocusHandle and use track_focus()
                    // No manual handling needed!
                }
                _ => {}
            }
        });
        
        // ... rest of render
    }
}
```

### Expected Behavior After Implementation

1. **Tab** cycles focus between Cancel and OK buttons (automatic via GPUI)
2. **Shift+Tab** cycles in reverse (automatic via GPUI)
3. **Space** triggers the focused button
4. **Enter** triggers the focused button (or always triggers OK)
5. **Escape** closes the dialog with Cancel
6. **Arrow Left/Right** can optionally move focus (for horizontal button layouts)
7. Visual focus ring appears on focused button using gpui_component styling

---

## Instructions for the Next AI Agent

### Your Task

Refactor the `ConfirmDialog` (`src/confirm/dialog.rs`) to use proper GPUI focus handling and the `gpui_component::Button` or custom `Button` component.

### Key Files to Modify

1. **`src/confirm/dialog.rs`** - Main refactor target
2. **`src/confirm/window.rs`** - Update keyboard handling
3. **`src/components/button.rs`** - Reference for Button API

### Reference Files (not in bundle)

- `src/ai/window.rs` - Uses `gpui_component::button::Button` with `.ghost()`, `.on_click()`
- `src/notes/window.rs` - Uses `gpui_component::button::Button` correctly
- `src/components/shortcut_recorder.rs` - Uses custom `Button` with focus styling

### Don't Forget

1. **Run verification gate before committing**:
   ```bash
   cargo check && cargo clippy --all-targets -- -D warnings && cargo test
   ```

2. **Test with stdin protocol**:
   ```bash
   # Test the confirm dialog visually
   echo '{"type":"confirm","message":"Test focus?"}' | \
     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
   ```

3. **Check for these behaviors**:
   - Tab moves focus between buttons
   - Focused button has visible focus ring
   - Space triggers the focused button
   - Enter triggers the focused button
   - Escape cancels and closes

### Success Criteria

- [ ] ConfirmDialog uses `Button` component instead of raw `div()`
- [ ] Buttons have visible focus ring when focused
- [ ] Tab/Shift+Tab navigates between buttons
- [ ] Space/Enter triggers the focused button
- [ ] Focus ring styling matches rest of app
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings

### Anti-Patterns to Avoid

- Don't use `focused_button: usize` manual tracking with proper FocusHandle per button
- Don't duplicate focus ring styling - use Button's built-in `.focused()` method
- Don't handle Tab manually if using proper FocusHandle - GPUI handles it
