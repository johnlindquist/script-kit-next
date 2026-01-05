//! Actions Window - Separate vibrancy window for actions panel
//!
//! This creates a floating popup window with its own vibrancy blur effect,
//! similar to Raycast's actions panel. The window is:
//! - Non-draggable (fixed position relative to main window)
//! - Positioned below the header, at the right edge of main window
//! - Auto-closes when app loses focus
//! - Shares the ActionsDialog entity with the main app for keyboard routing

use crate::panel::HEADER_TOTAL_HEIGHT;
use crate::platform;
use gpui::{
    div, prelude::*, px, App, Bounds, Context, Entity, FocusHandle, Focusable, Pixels, Point,
    Render, Size, Window, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use std::sync::{Mutex, OnceLock};

use super::dialog::ActionsDialog;

/// Global singleton for the actions window handle
/// NOTE: Uses ActionsWindow directly (not Root) to avoid Root's opaque background
static ACTIONS_WINDOW: OnceLock<Mutex<Option<WindowHandle<ActionsWindow>>>> = OnceLock::new();

/// Actions window dimensions
const ACTIONS_WINDOW_WIDTH: f32 = 320.0;
const ACTIONS_WINDOW_HEIGHT: f32 = 400.0;
/// Margin from main window edges
const ACTIONS_MARGIN: f32 = 8.0;

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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Render the shared dialog entity - it handles its own sizing
        // Don't use size_full() - the dialog calculates its own dynamic height
        // This prevents unused window space from showing as a dark area
        div().child(self.dialog.clone())
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
/// * `main_window_bounds` - The bounds of the main window (for positioning)
/// * `dialog_entity` - The shared ActionsDialog entity (created by main app)
///
/// # Returns
/// The window handle on success
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    dialog_entity: Entity<ActionsDialog>,
) -> anyhow::Result<WindowHandle<ActionsWindow>> {
    // Close any existing actions window first
    close_actions_window(cx);

    // Use transparent window background - the dialog renders its own
    // vibrancy/blur styling. This prevents unused window area from showing
    // as a dark rectangle when the content is smaller than max height.
    let window_background = gpui::WindowBackgroundAppearance::Transparent;

    // Calculate window position:
    // - X: Right edge of main window, minus actions width, minus margin
    // - Y: Below the header (HEADER_TOTAL_HEIGHT), plus margin
    let window_width = px(ACTIONS_WINDOW_WIDTH);
    let window_height = px(ACTIONS_WINDOW_HEIGHT);

    let window_x = main_window_bounds.origin.x + main_window_bounds.size.width
        - window_width
        - px(ACTIONS_MARGIN);
    let window_y = main_window_bounds.origin.y + px(HEADER_TOTAL_HEIGHT) + px(ACTIONS_MARGIN);

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
            "Opening actions window at ({:?}, {:?}), size {:?}x{:?}",
            window_x, window_y, window_width, window_height
        ),
    );

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None, // No titlebar = no drag affordance
        window_background,
        focus: false, // CRITICAL: Don't take focus - main window keeps it
        show: true,
        kind: WindowKind::PopUp, // Floating popup window
        ..Default::default()
    };

    // Create the window with the shared dialog entity
    // NOTE: We don't use Root wrapper here because Root has .bg(theme.background).size_full()
    // which creates an opaque background for the entire window. Instead, we render the
    // ActionsWindow directly - the ActionsDialog handles its own styling.
    let handle = cx.open_window(window_options, |_window, cx| {
        cx.new(|cx| ActionsWindow::new(dialog_entity, cx))
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
pub fn get_actions_window_handle() -> Option<WindowHandle<ActionsWindow>> {
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
