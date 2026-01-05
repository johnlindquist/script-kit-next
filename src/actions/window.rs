//! Actions Window - Separate vibrancy window for actions panel
//!
//! This creates a floating window with its own vibrancy blur effect,
//! similar to Raycast's actions panel. The blur completely obscures
//! the content behind it while still showing the desktop colors through.
//!
//! NOTE: This module is prepared but not yet integrated. The functions
//! are exported but not used until we switch from inline overlay to
//! separate window rendering.

#![allow(dead_code)]

use crate::theme;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, Entity, FocusHandle, Focusable, Pixels, Point,
    Render, Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use gpui_component::Root;
use std::sync::{Arc, Mutex, OnceLock};

use super::dialog::ActionsDialog;
use super::types::ScriptInfo;

/// Global singleton for the actions window handle
static ACTIONS_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

/// Actions window dimensions
const ACTIONS_WINDOW_WIDTH: f32 = 320.0;
const ACTIONS_WINDOW_HEIGHT: f32 = 400.0;

/// ActionsWindow wrapper that holds the dialog and handles window-level concerns
pub struct ActionsWindow {
    pub dialog: Entity<ActionsDialog>,
    pub focus_handle: FocusHandle,
}

impl ActionsWindow {
    pub fn new(
        dialog: Entity<ActionsDialog>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Render the dialog directly - it handles its own styling
        div().size_full().child(self.dialog.clone())
    }
}

/// Open the actions window as a separate floating window with vibrancy
///
/// # Arguments
/// * `cx` - The application context
/// * `anchor_position` - The position to anchor the window (usually main window's top-right)
/// * `focused_script` - Optional script context for actions
/// * `on_select` - Callback when an action is selected
pub fn open_actions_window(
    cx: &mut App,
    anchor_position: Point<Pixels>,
    focused_script: Option<ScriptInfo>,
    on_select: Arc<dyn Fn(String) + Send + Sync + 'static>,
) -> anyhow::Result<WindowHandle<Root>> {
    // Close any existing actions window first
    close_actions_window(cx);

    // Load theme for vibrancy settings
    let theme = Arc::new(theme::load_theme());
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate window bounds - position relative to anchor
    // Offset slightly to the left and down from the anchor point
    let window_x = anchor_position.x - px(ACTIONS_WINDOW_WIDTH + 8.0);
    let window_y = anchor_position.y + px(8.0);

    let bounds = Bounds {
        origin: Point {
            x: window_x,
            y: window_y,
        },
        size: gpui::Size {
            width: px(ACTIONS_WINDOW_WIDTH),
            height: px(ACTIONS_WINDOW_HEIGHT),
        },
    };

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None, // No titlebar for popup
        window_background,
        focus: true,
        show: true,
        kind: WindowKind::PopUp, // Floating popup window
        ..Default::default()
    };

    // Create the window with the actions dialog inside
    let theme_clone = theme.clone();
    let handle = cx.open_window(window_options, |window, cx| {
        // Create the dialog
        let focus_handle = cx.focus_handle();
        let on_select_clone = on_select.clone();

        let dialog = cx.new(|_cx| {
            ActionsDialog::with_script(
                focus_handle.clone(),
                Arc::new(move |action_id| {
                    on_select_clone(action_id);
                }),
                focused_script.clone(),
                theme_clone.clone(),
            )
        });

        // Create the window wrapper
        let actions_window = cx.new(|cx| ActionsWindow::new(dialog, window, cx));

        // Wrap in Root for gpui-component theming
        cx.new(|cx| Root::new(actions_window, window, cx))
    })?;

    // Store the handle globally
    let window_storage = ACTIONS_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }

    // Activate the window
    let _ = handle.update(cx, |_root, window, _cx| {
        window.activate_window();
    });

    Ok(handle)
}

/// Close the actions window if it's open
pub fn close_actions_window(cx: &mut App) {
    if let Some(window_storage) = ACTIONS_WINDOW.get() {
        if let Ok(mut guard) = window_storage.lock() {
            if let Some(handle) = guard.take() {
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
