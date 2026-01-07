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
        focus: false, // CRITICAL: Don't take focus - main window keeps it
        show: true,
        kind: WindowKind::PopUp, // Floating popup window
        display_id,              // CRITICAL: Position on same display as main window
        ..Default::default()
    };

    // Create the window with the shared dialog entity
    let handle = cx.open_window(window_options, |window, cx| {
        let actions_window = cx.new(|cx| ActionsWindow::new(dialog_entity, cx));
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
