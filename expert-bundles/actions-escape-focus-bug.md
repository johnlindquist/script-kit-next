# Actions Menu Escape Key Focus Bug - Expert Bundle

## Status: FIXES APPLIED

The following fixes have been applied to address two related focus bugs:

### Fix 1: Main Window Shortcut Not Focusing Input
- **Files**: `src/main.rs`, `src/hotkey_pollers.rs`
- **Issue**: When showing the window via hotkey without NEEDS_RESET, `pending_focus` wasn't being set
- **Fix**: Added `view.pending_focus = Some(FocusTarget::MainFilter)` in both locations

### Fix 2: Actions Menu Escape Key Focus
- **Files**: `src/actions/window.rs`, `src/app_impl.rs`, `src/render_builtins.rs`
- **Issue**: `on_close` callback set wrong FocusTarget; window closed before main window could become key
- **Fix**: Changed to `pending_focus = AppRoot` (matching `close_actions_popup`); deferred window removal

## Original Goal

> Pressing Escape in the actions menu should close the menu AND return focus to the main input field (cursor should blink). Currently, Cmd+K to toggle works correctly, but Escape does NOT restore focus.

## Executive Summary

When the user opens the actions menu with `Cmd+K` and closes it by pressing `Cmd+K` again, focus correctly returns to the main input (cursor blinks). However, when closing with `Escape`, the menu closes but **focus is NOT restored** - the cursor doesn't blink and typing doesn't work until the user clicks the input.

### Key Problems:

1. **Two competing escape handlers**: ActionsWindow (separate window) handles escape in `window.rs:173-186`, but the main window's keystroke interceptor (`app_impl.rs:1004-1010`) also tries to handle escape for `show_actions_popup`.

2. **Race condition / timing issue**: When ActionsWindow's escape handler runs:
   - It calls `on_close` callback which sets `pending_focus = AppRoot` and calls `cx.notify()`
   - Then calls `platform::activate_main_window()` to make main window key
   - Then uses `window.defer()` to close the ActionsWindow
   - But `apply_pending_focus()` in the main window's render may not run properly because the window activation is asynchronous

3. **Missing direct window.focus() call**: The working `close_actions_popup()` function calls `window.focus(&self.focus_handle, cx)` directly, but the `on_close` callback doesn't have access to `window` and can only set `pending_focus`.

### Required Fixes:

1. **File: `src/actions/window.rs` (lines 173-186)**: The escape handler needs to ensure focus is properly restored. Current approach of `window.defer()` may not be sufficient.

2. **File: `src/app_impl.rs`**: The `on_close` callback (line ~3030-3045) sets state but can't directly focus. Need a way to either:
   - Have the main window's interceptor handle escape even when ActionsWindow has it
   - Or provide the `on_close` callback with a way to directly apply focus

3. **Architecture decision needed**: Should the ActionsWindow handle escape at all, or should escape be routed through the main window's keystroke interceptor (which already has the working `close_actions_popup()` function)?

### Files Included:

- `src/actions/window.rs`: ActionsWindow struct, escape handler, `open_actions_window()`
- `src/actions/types.rs`: `CloseCallback` type definition
- `src/actions/dialog.rs`: ActionsDialog, `on_close` callback storage, `set_on_close()`, `trigger_on_close()`
- `src/app_impl.rs`: `toggle_actions()`, `close_actions_popup()`, `apply_pending_focus()`, on_close callback setup, keystroke interceptor
- `src/platform.rs`: `activate_main_window()`
- `src/main.rs`: `FocusedInput`, `FocusTarget` enums, render() where `apply_pending_focus()` is called

---

## Code Context

### src/actions/window.rs (FULL FILE - 787 lines)

This is the ActionsWindow that handles keyboard events including Escape:

```rust
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

/// Actions window width
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

        if !is_focused {
            crate::logging::log(
                "ACTIONS",
                "ActionsWindow: focus_handle NOT focused, re-focusing",
            );
            self.focus_handle.focus(window, cx);
        }

        // Key handler for the actions window
        let handle_key = cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let modifiers = &event.keystroke.modifiers;

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "ActionsWindow on_key_down received: key='{}', modifiers={:?}",
                    key, modifiers
                ),
            );

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
                // ============================================================
                // ESCAPE HANDLER - THIS IS THE PROBLEMATIC CODE PATH
                // ============================================================
                "escape" => {
                    // Notify main app to restore focus before closing
                    let on_close = this.dialog.read(cx).on_close.clone();
                    if let Some(callback) = on_close {
                        callback(cx);  // Sets pending_focus = AppRoot, calls cx.notify()
                    }
                    // Activate the main window so it can receive focus
                    platform::activate_main_window();
                    // Defer window removal to give the main window time to become key
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

/// Open the actions window as a separate floating window with vibrancy
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

    // Calculate dynamic window height based on content
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
        focus: false,  // DON'T take focus - let parent keep it? But we DO focus it in new()
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    let handle = cx.open_window(window_options, |window, cx| {
        let actions_window = cx.new(|cx| {
            let aw = ActionsWindow::new(dialog_entity, cx);
            aw.focus_handle.focus(window, cx);  // Actually focus the actions window
            aw
        });
        cx.new(|cx| Root::new(actions_window, window, cx))
    })?;

    // Configure the window as non-movable on macOS
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

/// Close the actions window if it's open
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

/// Check if the actions window is currently open
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

/// Resize the actions window directly using the window reference
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
    let current_width_f32: f32 = current_bounds.size.width.into();

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

// resize_actions_window() function omitted for brevity - similar to resize_actions_window_direct
```

---

### src/actions/types.rs (Key Types)

```rust
/// Callback for dialog close (escape pressed, window dismissed)
/// Used to notify the main app to restore focus
/// Takes &mut App so the callback can update the main app entity
pub type CloseCallback = Arc<dyn Fn(&mut gpui::App) + Send + Sync>;
```

---

### src/actions/dialog.rs (Key Methods)

```rust
pub struct ActionsDialog {
    // ... other fields ...
    
    /// Callback for when the dialog is closed (escape pressed, window dismissed)
    /// Used to notify the main app to restore focus
    pub on_close: Option<CloseCallback>,
}

impl ActionsDialog {
    /// Set the callback for when the dialog is closed (escape pressed, window dismissed)
    /// Used to notify the main app to restore focus
    pub fn set_on_close(&mut self, callback: CloseCallback) {
        self.on_close = Some(callback);
    }

    /// Call the on_close callback if set
    /// Returns true if a callback was called, false otherwise
    pub fn trigger_on_close(&self, cx: &mut gpui::App) -> bool {
        if let Some(ref callback) = self.on_close {
            callback(cx);
            true
        } else {
            false
        }
    }
}
```

---

### src/main.rs (Focus Enums and apply_pending_focus call)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedInput {
    /// Main script list filter input
    MainFilter,
    /// Actions dialog search input
    ActionsSearch,
    /// Arg prompt input (when running a script)
    ArgPrompt,
    /// No input focused (e.g., terminal prompt)
    None,
}

/// Pending focus target - identifies which element should receive focus
/// when window access becomes available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    MainFilter,
    AppRoot,
    ActionsDialog,
    PathPrompt,
    FormPrompt,
    EditorPrompt,
    SelectPrompt,
    EnvPrompt,
    DropPrompt,
    TemplatePrompt,
    TermPrompt,
    ChatPrompt,
}

// In render() method:
impl Render for ScriptListApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ... window focus tracking ...
        
        // Apply pending focus request (if any). This is the new "apply once" mechanism
        // that replaces the old "perpetually enforce focus in render()" pattern.
        // Focus is applied exactly once when pending_focus is set, then cleared.
        self.apply_pending_focus(window, cx);
        
        // ... rest of render ...
    }
}
```

---

### src/app_impl.rs (Key Methods)

#### apply_pending_focus (lines 1247-1356)
```rust
/// Apply pending focus if set. Called at the start of render() when window
/// is focused. This applies focus exactly once, then clears pending_focus.
fn apply_pending_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
    let Some(target) = self.pending_focus.take() else {
        return false;
    };

    logging::log("FOCUS", &format!("Applying pending focus: {:?}", target));

    match target {
        FocusTarget::MainFilter => {
            let input_state = self.gpui_input_state.clone();
            input_state.update(cx, |state, cx| {
                state.focus(window, cx);
            });
            self.focused_input = FocusedInput::MainFilter;
        }
        FocusTarget::ActionsDialog => {
            if let Some(ref dialog) = self.actions_dialog {
                let fh = dialog.read(cx).focus_handle.clone();
                window.focus(&fh, cx);
                self.focused_input = FocusedInput::ActionsSearch;
            }
        }
        // ... other cases ...
        FocusTarget::ChatPrompt => {
            if let AppView::ChatPrompt { entity, .. } = &self.current_view {
                let fh = entity.read(cx).focus_handle(cx);
                window.focus(&fh, cx);
                self.focused_input = FocusedInput::None;
            }
        }
        FocusTarget::AppRoot => {
            window.focus(&self.focus_handle, cx);
            // Don't reset focused_input here - the caller already set it appropriately.
            // For example, ArgPrompt sets focused_input = ArgPrompt before setting
            // pending_focus = AppRoot, and we want to preserve that so the cursor blinks.
        }
    }

    true
}
```

#### toggle_actions - THE WORKING CMD+K PATH (lines 2958-3097)
```rust
fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
    let popup_state = self.show_actions_popup;
    let window_open = is_actions_window_open();
    logging::log(
        "KEY",
        &format!(
            "Toggling actions popup (show_actions_popup={}, is_actions_window_open={})",
            popup_state, window_open
        ),
    );
    if self.show_actions_popup || is_actions_window_open() {
        // Close - return focus to main filter
        self.show_actions_popup = false;
        self.actions_dialog = None;
        self.focused_input = FocusedInput::MainFilter;
        self.pending_focus = Some(FocusTarget::MainFilter);

        // Close the separate actions window via spawn
        cx.spawn(async move |_this, cx| {
            cx.update(|cx| {
                close_actions_window(cx);
            })
            .ok();
        })
        .detach();

        // Refocus main filter - THIS IS THE KEY DIFFERENCE!
        self.focus_main_filter(window, cx);  // <-- DIRECT FOCUS CALL
        logging::log("FOCUS", "Actions closed, focus returned to MainFilter");
    } else {
        // Open actions as a separate window with vibrancy blur
        self.show_actions_popup = true;
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
        self.focused_input = FocusedInput::ActionsSearch;

        // ... create dialog ...

        // Set up the on_close callback
        let app_entity = cx.entity().clone();
        dialog.update(cx, |d, _cx| {
            d.set_on_close(std::sync::Arc::new(move |cx| {
                app_entity.update(cx, |app, cx| {
                    app.show_actions_popup = false;
                    app.actions_dialog = None;
                    // Match what close_actions_popup does for MainList host:
                    app.focused_input = FocusedInput::MainFilter;
                    app.pending_focus = Some(FocusTarget::AppRoot);
                    logging::log(
                        "FOCUS",
                        "Actions closed via escape, pending_focus=AppRoot, focused_input=MainFilter",
                    );
                    cx.notify();
                });
            }));
        });

        // ... open window ...
    }
    cx.notify();
}
```

#### close_actions_popup - THE CENTRALIZED CLOSE FUNCTION (lines 3543-3599)
```rust
/// Close the actions popup and restore focus based on host type.
///
/// This centralizes close behavior, ensuring cx.notify() is always called
/// and focus is correctly restored based on which prompt hosted the dialog.
fn close_actions_popup(
    &mut self,
    host: ActionsDialogHost,
    window: &mut Window,
    cx: &mut Context<Self>,
) {
    self.show_actions_popup = false;
    self.actions_dialog = None;

    // Close the separate actions window if open
    if is_actions_window_open() {
        cx.spawn(async move |_this, cx| {
            cx.update(|cx| {
                close_actions_window(cx);
            })
            .ok();
        })
        .detach();
    }

    // Restore focus based on host type
    match host {
        ActionsDialogHost::ArgPrompt => {
            self.focused_input = FocusedInput::ArgPrompt;
            self.pending_focus = Some(FocusTarget::AppRoot);
        }
        ActionsDialogHost::DivPrompt
        | ActionsDialogHost::EditorPrompt
        | ActionsDialogHost::TermPrompt
        | ActionsDialogHost::FormPrompt => {
            self.focused_input = FocusedInput::None;
        }
        ActionsDialogHost::ChatPrompt => {
            self.focused_input = FocusedInput::None;
            self.pending_focus = Some(FocusTarget::AppRoot);
        }
        ActionsDialogHost::MainList => {
            self.focused_input = FocusedInput::MainFilter;
            self.pending_focus = Some(FocusTarget::AppRoot);
        }
        ActionsDialogHost::FileSearch => {
            self.focused_input = FocusedInput::MainFilter;
            self.pending_focus = Some(FocusTarget::AppRoot);
        }
    }

    // THIS IS THE KEY - DIRECT window.focus() CALL
    window.focus(&self.focus_handle, cx);
    logging::log(
        "FOCUS",
        &format!("Actions popup closed, focus restored for {:?}", host),
    );
    cx.notify();
}
```

#### Main window keystroke interceptor (lines 1004-1010)
```rust
// For ScriptList with actions open, handle Escape/Enter/typing
if matches!(this.current_view, AppView::ScriptList) && this.show_actions_popup {
    // Handle Escape to close actions popup
    if key == "escape" {
        this.close_actions_popup(ActionsDialogHost::MainList, window, cx);
        cx.stop_propagation();
        return;
    }
    // ... handle enter, typing ...
}
```

**PROBLEM**: This interceptor checks `this.show_actions_popup`, but by the time it runs, the ActionsWindow has already called the `on_close` callback which sets `show_actions_popup = false`. So this code path is NEVER reached when pressing Escape in ActionsWindow.

---

### src/platform.rs (activate_main_window)

```rust
/// Activate the main window and bring it to front.
///
/// This makes the main window the key window and activates the application.
/// Used when returning focus to the main window after closing overlays like the actions popup.
#[cfg(target_os = "macos")]
pub fn activate_main_window() {
    debug_assert_main_thread();
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log("PANEL", "activate_main_window: Main window not registered");
                return;
            }
        };

        let app: id = NSApp();
        
        // Activate the application, ignoring other apps
        let _: () = msg_send![app, activateIgnoringOtherApps: true];

        // Make our window key and bring it to front
        let _: () = msg_send![window, makeKeyAndOrderFront: nil];

        logging::log(
            "PANEL",
            "Main window activated (activateIgnoringOtherApps + makeKeyAndOrderFront)",
        );
    }
}
```

---

## Implementation Guide

### The Core Problem

When Escape is pressed in ActionsWindow:

1. `on_close` callback runs → sets `pending_focus = AppRoot`, `focused_input = MainFilter`, calls `cx.notify()`
2. `platform::activate_main_window()` runs → makes main window key (async operation)
3. `window.defer()` schedules `window.remove_window()`
4. Main window should re-render and call `apply_pending_focus()`

**But**: The main window may not re-render with proper window access because:
- `cx.notify()` was called while ActionsWindow context was active
- `platform::activate_main_window()` is async - window may not be key yet
- `apply_pending_focus()` only runs in render, but render timing is uncertain

**Compare to Cmd+K toggle path** (which works):
1. Main window's interceptor catches Cmd+K
2. `toggle_actions()` calls `self.focus_main_filter(window, cx)` **DIRECTLY**
3. Main window has direct `window` reference, focus is applied immediately

### Step 1: Option A - Remove escape handling from ActionsWindow

The cleanest fix is to let the main window handle ALL escape events via the global keystroke interceptor.

**Problem**: The main window's interceptor checks `this.show_actions_popup` AFTER `on_close` has already set it to false.

**Solution**: Check `is_actions_window_open()` instead:

```rust
// File: src/app_impl.rs
// Location: keystroke interceptor, around line 1004

// BEFORE:
if matches!(this.current_view, AppView::ScriptList) && this.show_actions_popup {

// AFTER:
if matches!(this.current_view, AppView::ScriptList) 
    && (this.show_actions_popup || is_actions_window_open()) 
{
```

Then remove the escape handler from ActionsWindow:

```rust
// File: src/actions/window.rs
// Location: match key block, around line 173

// REMOVE or comment out the entire "escape" => { ... } block
// Let the main window's interceptor handle escape
```

### Step 2: Option B - Give on_close callback window access

If Option A doesn't work (interceptor may not fire when ActionsWindow has focus), we need to give the callback a way to directly focus.

One approach: Store a closure that captures the main window handle:

```rust
// File: src/app_impl.rs
// Location: on_close callback setup, around line 3029

// Store the main window's entity for later focus restoration
let app_entity = cx.entity().clone();
let main_focus_handle = self.focus_handle.clone();

dialog.update(cx, |d, _cx| {
    d.set_on_close(std::sync::Arc::new(move |cx| {
        app_entity.update(cx, |app, cx| {
            app.show_actions_popup = false;
            app.actions_dialog = None;
            app.focused_input = FocusedInput::MainFilter;
            
            // Instead of setting pending_focus, we need to somehow
            // trigger close_actions_popup which has window access
            // This is the architectural challenge - cx is &mut App, not Context<Self>
            
            cx.notify();
        });
    }));
});
```

**Architectural challenge**: The `on_close` callback receives `&mut gpui::App`, not `&mut Context<ScriptListApp>` with `Window` access. This makes it impossible to call `window.focus()` directly.

### Step 3: Option C - Use a channel/flag to signal escape

Set a flag when escape is pressed, then handle it in the main window's render:

```rust
// File: src/app_impl.rs (add to ScriptListApp struct)
pub actions_escape_pressed: bool,

// File: src/actions/window.rs (escape handler)
"escape" => {
    // Set flag instead of calling on_close
    let app_entity = /* need to get this somehow */;
    app_entity.update(cx, |app, _cx| {
        app.actions_escape_pressed = true;
    });
    platform::activate_main_window();
    window.defer(cx, |window, _cx| {
        window.remove_window();
    });
}

// File: src/main.rs (in render, before apply_pending_focus)
if self.actions_escape_pressed {
    self.actions_escape_pressed = false;
    self.close_actions_popup(ActionsDialogHost::MainList, window, cx);
}
```

### Step 4: Recommended Approach - Hybrid

1. Keep the `on_close` callback for state cleanup (`show_actions_popup = false`, etc.)
2. Add `actions_needs_close: bool` flag to ScriptListApp
3. In `on_close`, set `actions_needs_close = true` and call `cx.notify()`
4. In main window's render (before `apply_pending_focus`), check:
   ```rust
   if self.actions_needs_close {
       self.actions_needs_close = false;
       // Don't call close_actions_popup (window already closed)
       // Just restore focus directly:
       window.focus(&self.focus_handle, cx);
       self.focused_input = FocusedInput::MainFilter;
   }
   ```

---

## Instructions for the Next AI Agent

### Your Task

Fix the escape key focus restoration bug in the Script Kit GPUI actions menu.

### What You Need to Do

1. **Understand the two code paths**:
   - Working: Cmd+K toggle → `toggle_actions()` → `focus_main_filter(window, cx)` (DIRECT)
   - Broken: Escape in ActionsWindow → `on_close` callback → `pending_focus` (INDIRECT)

2. **Choose an implementation approach** from the options above (Option A is cleanest)

3. **Verify the fix**:
   ```bash
   cargo check && cargo clippy --all-targets -- -D warnings && cargo test --lib -- --skip logging
   ```

4. **Test manually**:
   - Open app with `echo '{"type":"show"}' | ./target/debug/script-kit-gpui`
   - Press Cmd+K to open actions
   - Press Escape to close
   - Verify cursor blinks in main input

5. **Watch the logs** for:
   ```
   FOCUS|Actions closed via escape
   FOCUS|Applying pending focus: AppRoot
   ```

### Key Insight

The fundamental issue is that the `on_close` callback doesn't have access to `Window`, so it can't call `window.focus()` directly. The working Cmd+K path has direct window access.

The fix must either:
- Route escape through the main window's interceptor (which has window access)
- Or defer focus restoration to the next render cycle where window IS available

### Files to Modify

1. `src/app_impl.rs` - Add flag or modify interceptor logic
2. `src/actions/window.rs` - Possibly remove escape handler
3. `src/main.rs` - Possibly add check in render() before apply_pending_focus()

### Do NOT

- Add more `cx.notify()` calls hoping they'll trigger a render - the issue is architectural
- Use `platform::activate_main_window()` alone - it doesn't focus the input, just makes the window key
- Try to pass `Window` to the callback - it's not `Send + Sync`

OUTPUT_FILE_PATH: expert-bundles/actions-escape-focus-bug.md
