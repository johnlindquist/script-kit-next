//! Confirm Window - Separate vibrancy window for confirmation dialog
//!
//! This creates a floating popup window with its own vibrancy blur effect,
//! similar to the actions panel. The window is:
//! - Non-draggable (fixed position relative to main window)
//! - Centered over the main window
//! - Auto-closes when choice is made

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

/// Global singleton for the confirm window handle
static CONFIRM_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

/// Global singleton for the confirm dialog entity (for keyboard event dispatch)
static CONFIRM_DIALOG: OnceLock<Mutex<Option<Entity<ConfirmDialog>>>> = OnceLock::new();

/// ConfirmWindow wrapper that renders the ConfirmDialog entity
pub struct ConfirmWindow {
    /// The dialog entity
    pub dialog: Entity<ConfirmDialog>,
    /// Focus handle for this window
    pub focus_handle: FocusHandle,
}

impl ConfirmWindow {
    pub fn new(dialog: Entity<ConfirmDialog>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            dialog,
            focus_handle,
        }
    }
}

impl Focusable for ConfirmWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ConfirmWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.dialog.clone())
    }
}

/// Open the confirm window as a separate floating window with vibrancy
///
/// The window is centered over the main window.
///
/// # Arguments
/// * `cx` - The application context
/// * `main_window_bounds` - The bounds of the main window in SCREEN-RELATIVE coordinates
/// * `display_id` - The display where the main window is located
/// * `message` - The confirmation message
/// * `confirm_text` - Optional text for the confirm button
/// * `cancel_text` - Optional text for the cancel button
/// * `on_choice` - Callback when user makes a choice
///
/// # Returns
/// The window handle and dialog entity on success
pub fn open_confirm_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    message: String,
    confirm_text: Option<String>,
    cancel_text: Option<String>,
    on_choice: ConfirmCallback,
) -> anyhow::Result<(WindowHandle<Root>, Entity<ConfirmDialog>)> {
    // Close any existing confirm window first
    close_confirm_window(cx);

    // Load theme for vibrancy settings
    let theme = theme::load_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        gpui::WindowBackgroundAppearance::Blurred
    } else {
        gpui::WindowBackgroundAppearance::Opaque
    };

    // Calculate window position: centered over main window
    let window_width = px(CONFIRM_WIDTH);
    let window_height = px(CONFIRM_HEIGHT);

    let window_x =
        main_window_bounds.origin.x + (main_window_bounds.size.width - window_width) / 2.0;
    let window_y =
        main_window_bounds.origin.y + (main_window_bounds.size.height - window_height) / 2.0;

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
        "CONFIRM",
        &format!(
            "Opening confirm window at ({:?}, {:?}), size {:?}x{:?}",
            window_x, window_y, window_width, window_height
        ),
    );

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background,
        focus: false, // CRITICAL: Don't take focus - main window keeps it and routes keys
        show: true,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    // Create the dialog entity first, then the window
    let theme_arc = std::sync::Arc::new(theme);
    let mut dialog_entity_holder: Option<Entity<ConfirmDialog>> = None;

    let handle = cx.open_window(window_options, |window, cx| {
        // Create the dialog entity
        let dialog = cx.new(|cx| {
            ConfirmDialog::new(
                message,
                confirm_text,
                cancel_text,
                cx.focus_handle(),
                on_choice,
                theme_arc.clone(),
            )
        });

        dialog_entity_holder = Some(dialog.clone());

        // Create the window wrapper
        let confirm_window = cx.new(|cx| ConfirmWindow::new(dialog, cx));

        // Wrap in Root for gpui-component theming and vibrancy
        cx.new(|cx| Root::new(confirm_window, window, cx))
    })?;

    // Configure the window as non-movable on macOS
    #[cfg(target_os = "macos")]
    {
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
    }

    // Store the handle globally
    let window_storage = CONFIRM_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = window_storage.lock() {
        *guard = Some(handle);
    }

    let dialog_entity = dialog_entity_holder.expect("Dialog entity should have been created");

    // Store the dialog entity globally for keyboard event dispatch
    let dialog_storage = CONFIRM_DIALOG.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = dialog_storage.lock() {
        *guard = Some(dialog_entity.clone());
    }

    crate::logging::log("CONFIRM", "Confirm popup window opened with vibrancy");

    Ok((handle, dialog_entity))
}

/// Close the confirm window if it's open
pub fn close_confirm_window(cx: &mut App) {
    // Clear the dialog entity first
    if let Some(dialog_storage) = CONFIRM_DIALOG.get() {
        if let Ok(mut guard) = dialog_storage.lock() {
            *guard = None;
        }
    }

    // Then close the window
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

/// Check if the confirm window is currently open
#[allow(dead_code)]
pub fn is_confirm_window_open() -> bool {
    if let Some(window_storage) = CONFIRM_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return guard.is_some();
        }
    }
    false
}

/// Get the confirm window handle if it exists
#[allow(dead_code)]
pub fn get_confirm_window_handle() -> Option<WindowHandle<Root>> {
    if let Some(window_storage) = CONFIRM_WINDOW.get() {
        if let Ok(guard) = window_storage.lock() {
            return *guard;
        }
    }
    None
}

/// Notify the confirm window to re-render
#[allow(dead_code)]
pub fn notify_confirm_window(cx: &mut App) {
    if let Some(handle) = get_confirm_window_handle() {
        let _ = handle.update(cx, |_root, _window, cx| {
            cx.notify();
        });
    }
}

/// Dispatch a keyboard event to the confirm dialog
/// Returns true if the event was handled, false otherwise
pub fn dispatch_confirm_key(key: &str, cx: &mut App) -> bool {
    // Get the dialog entity from global storage
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

    crate::logging::log(
        "CONFIRM",
        &format!("Dispatching key to confirm dialog: {}", key),
    );

    match key {
        // Enter = submit current selection and close
        "enter" | "Enter" => {
            dialog.update(cx, |d, _cx| d.submit());
            close_confirm_window(cx);
            true
        }
        // Escape = cancel and close
        "escape" | "Escape" => {
            dialog.update(cx, |d, _cx| d.cancel());
            close_confirm_window(cx);
            true
        }
        // Tab = toggle focus between buttons
        "tab" | "Tab" => {
            dialog.update(cx, |d, cx| d.toggle_focus(cx));
            notify_confirm_window(cx);
            true
        }
        // Left arrow = focus cancel button
        "left" | "arrowleft" | "Left" | "ArrowLeft" => {
            dialog.update(cx, |d, cx| d.focus_cancel(cx));
            notify_confirm_window(cx);
            true
        }
        // Right arrow = focus confirm button
        "right" | "arrowright" | "Right" | "ArrowRight" => {
            dialog.update(cx, |d, cx| d.focus_confirm(cx));
            notify_confirm_window(cx);
            true
        }
        _ => false,
    }
}
