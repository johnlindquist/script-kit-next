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
/// Vertical margin from header/footer
const ACTIONS_MARGIN_Y: f32 = 8.0;
/// Titlebar height (for top-anchored positioning)
#[allow(dead_code)] // Reserved for future TopRight positioning
const TITLEBAR_HEIGHT: f32 = 36.0;

/// Window position relative to the parent window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum WindowPosition {
    /// Bottom-right, above the footer (default for Cmd+K actions)
    #[default]
    BottomRight,
    /// Top-right, below the titlebar (for new chat dropdown)
    TopRight,
    /// Top-center, below the titlebar, horizontally centered (Raycast-style for Notes)
    TopCenter,
}

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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Log focus state AND window focus state
        let is_focused = self.focus_handle.is_focused(window);
        let window_is_active = window.is_window_active();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "ActionsWindow render: focus_handle.is_focused={}, window_is_active={}",
                is_focused, window_is_active
            ),
        );

        // Ensure we have focus on each render
        if !is_focused {
            crate::logging::log(
                "ACTIONS",
                "ActionsWindow: focus_handle NOT focused, re-focusing",
            );
            self.focus_handle.focus(window, cx);
        }

        // Key handler for the actions window
        // Since this is a separate window, it needs its own key handling
        // (the parent window can't route events to us)
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
                    crate::logging::log("ACTIONS", "ActionsWindow: handling UP arrow");

                    this.dialog.update(cx, |d, cx| d.move_up(cx));
                    cx.notify();
                }
                "down" | "arrowdown" => {
                    crate::logging::log("ACTIONS", "ActionsWindow: handling DOWN arrow");
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
                    crate::logging::log("ACTIONS", "ActionsWindow: backspace pressed");
                    this.dialog.update(cx, |d, cx| d.handle_backspace(cx));
                    // Schedule resize after filter changes
                    let dialog = this.dialog.clone();
                    window.defer(cx, move |window, cx| {
                        crate::logging::log("ACTIONS", "ActionsWindow: defer - resizing directly");
                        resize_actions_window_direct(window, cx, &dialog);
                    });
                    cx.notify();
                }
                _ => {
                    // Handle printable characters for search (when no modifiers)
                    if !modifiers.platform && !modifiers.control && !modifiers.alt {
                        if let Some(ch) = key.chars().next() {
                            if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' || ch == '_'
                            {
                                crate::logging::log(
                                    "ACTIONS",
                                    &format!("ActionsWindow: char '{}' pressed", ch),
                                );
                                this.dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                // Schedule resize after filter changes
                                let dialog = this.dialog.clone();
                                window.defer(cx, move |window, cx| {
                                    crate::logging::log(
                                        "ACTIONS",
                                        "ActionsWindow: defer - resizing directly",
                                    );
                                    resize_actions_window_direct(window, cx, &dialog);
                                });
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
/// * `position` - Where to position the window relative to the main window
///
/// # Returns
/// The window handle on success
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    dialog_entity: Entity<ActionsDialog>,
    position: WindowPosition,
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
    // - Y: Depends on position parameter:
    //   - BottomRight: Above footer, aligned to bottom
    //   - TopRight: Below titlebar, aligned to top
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

    // Calculate Y position based on anchor position
    let window_y = match position {
        WindowPosition::BottomRight => {
            // Position popup above the footer (footer is 40px)
            main_window_bounds.origin.y + main_window_bounds.size.height
                - window_height
                - px(FOOTER_HEIGHT)
                - px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopRight => {
            // Position popup below the titlebar
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
        WindowPosition::TopCenter => {
            // Position popup below the titlebar (same Y as TopRight)
            main_window_bounds.origin.y + px(TITLEBAR_HEIGHT) + px(ACTIONS_MARGIN_Y)
        }
    };

    // Override X position for TopCenter - center horizontally in the parent window
    let window_x = match position {
        WindowPosition::TopCenter => {
            // Center horizontally within the parent window
            main_window_bounds.origin.x + (main_window_bounds.size.width - window_width) / 2.0
        }
        _ => window_x, // Keep the right-aligned X for other positions
    };

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
        // DON'T take focus - let the parent AI window keep focus and route keys to us
        // macOS popup windows often don't receive keyboard events properly
        focus: false,
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
    // Use window.defer() to avoid RefCell borrow conflicts - GPUI may still have
    // internal state borrowed immediately after open_window returns.
    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, |_root, window, cx| {
            window.defer(cx, |_window, _cx| {
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
            });
        });
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

/// Resize the actions window directly using the window reference
/// Use this from defer callbacks where we already have access to the window
pub fn resize_actions_window_direct(
    window: &mut Window,
    cx: &mut App,
    dialog_entity: &Entity<ActionsDialog>,
) {
    // Read dialog state to calculate new height
    let dialog = dialog_entity.read(cx);
    let num_actions = dialog.filtered_actions.len();
    let hide_search = dialog.hide_search;
    let has_header = dialog.context_title.is_some();

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct: num_actions={}, hide_search={}, has_header={}",
            num_actions, hide_search, has_header
        ),
    );

    // Calculate new height
    let search_box_height = if hide_search {
        0.0
    } else {
        SEARCH_INPUT_HEIGHT
    };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    // When no actions, still need space for "No actions match" message (use 1 row height)
    let min_items_height = if num_actions == 0 {
        ACTION_ITEM_HEIGHT
    } else {
        0.0
    };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT)
        .max(min_items_height)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    let border_height = 2.0;
    let new_height_f32 = items_height + search_box_height + header_height + border_height;

    let current_bounds = window.bounds();
    let current_height_f32: f32 = current_bounds.size.height.into();
    let current_width_f32: f32 = current_bounds.size.width.into();

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct: current={:.0}x{:.0}, target_height={:.0}",
            current_width_f32, current_height_f32, new_height_f32
        ),
    );

    // Skip if height hasn't changed
    if (current_height_f32 - new_height_f32).abs() < 1.0 {
        crate::logging::log(
            "ACTIONS",
            "resize_actions_window_direct: skipping - height unchanged",
        );
        return;
    }

    // Resize via NSWindow to keep bottom pinned
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

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "NSWindow search: looking for {:.0}x{:.0} among {} windows",
                    current_width_f32, current_height_f32, count
                ),
            );

            for i in 0..count {
                let ns_window: cocoa::base::id = msg_send![windows, objectAtIndex: i];
                if ns_window == nil {
                    continue;
                }

                let frame: NSRect = msg_send![ns_window, frame];

                // Match by width (actions window has unique width of 320)
                if (frame.size.width - current_width_f32 as f64).abs() < 2.0
                    && (frame.size.height - current_height_f32 as f64).abs() < 2.0
                {
                    let window_screen: cocoa::base::id = msg_send![ns_window, screen];
                    if window_screen == nil {
                        let screens: cocoa::base::id = NSScreen::screens(nil);
                        let _primary: cocoa::base::id = msg_send![screens, objectAtIndex: 0u64];
                    }

                    // Keep bottom fixed by keeping origin.y the same
                    let new_frame = NSRect::new(
                        NSPoint::new(frame.origin.x, frame.origin.y),
                        NSSize::new(frame.size.width, new_height_f32 as f64),
                    );

                    let _: () = msg_send![ns_window, setFrame:new_frame display:true animate:false];

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
    window.resize(gpui::Size {
        width: current_bounds.size.width,
        height: px(new_height_f32),
    });

    crate::logging::log(
        "ACTIONS",
        &format!(
            "resize_actions_window_direct complete: {} items, height={:.0}",
            num_actions, new_height_f32
        ),
    );
}

/// Resize the actions window to fit the current number of filtered actions
/// Call this after filtering changes the action count
///
/// The window is "pinned to bottom" - the search input stays in place and
/// the window shrinks/grows from the top.
pub fn resize_actions_window(cx: &mut App, dialog_entity: &Entity<ActionsDialog>) {
    crate::logging::log("ACTIONS", "resize_actions_window called");
    if let Some(handle) = get_actions_window_handle() {
        // Read dialog state to calculate new height
        let dialog = dialog_entity.read(cx);
        let num_actions = dialog.filtered_actions.len();
        let hide_search = dialog.hide_search;
        let has_header = dialog.context_title.is_some();
        crate::logging::log(
            "ACTIONS",
            &format!(
                "resize_actions_window: num_actions={}, hide_search={}, has_header={}",
                num_actions, hide_search, has_header
            ),
        );

        // Calculate new height (same logic as open_actions_window)
        let search_box_height = if hide_search {
            0.0
        } else {
            SEARCH_INPUT_HEIGHT
        };
        let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
        // When no actions, still need space for "No actions match" message
        let min_items_height = if num_actions == 0 {
            ACTION_ITEM_HEIGHT
        } else {
            0.0
        };
        let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT)
            .max(min_items_height)
            .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
        let border_height = 2.0; // top + bottom border
        let new_height_f32 = items_height + search_box_height + header_height + border_height;

        let update_result = handle.update(cx, |_root, window, cx| {
            let current_bounds = window.bounds();
            let current_height_f32: f32 = current_bounds.size.height.into();
            let current_width_f32: f32 = current_bounds.size.width.into();

            crate::logging::log(
                "ACTIONS",
                &format!(
                    "resize_actions_window inside update: current={:.0}x{:.0}, target_height={:.0}",
                    current_width_f32, current_height_f32, new_height_f32
                ),
            );

            // Skip if height hasn't changed
            if (current_height_f32 - new_height_f32).abs() < 1.0 {
                crate::logging::log(
                    "ACTIONS",
                    "resize_actions_window: skipping - height unchanged",
                );
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

                    crate::logging::log(
                        "ACTIONS",
                        &format!(
                            "NSWindow search: looking for {:.0}x{:.0} among {} windows",
                            current_width_f32, current_height_f32, count
                        ),
                    );

                    // Find our actions window by matching current dimensions
                    let mut found = false;
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
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        crate::logging::log(
                            "ACTIONS",
                            &format!(
                                "NSWindow NOT FOUND - no window matches {:.0}x{:.0}",
                                current_width_f32, current_height_f32
                            ),
                        );
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

        if let Err(e) = update_result {
            crate::logging::log("ACTIONS", &format!("handle.update FAILED: {:?}", e));
        }

        crate::logging::log(
            "ACTIONS",
            &format!(
                "Resized actions window: {} items, height={:.0}",
                num_actions, new_height_f32
            ),
        );
    }
}
