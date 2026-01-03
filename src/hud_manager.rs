//! HUD Manager - System-level overlay notifications
//!
//! Creates independent floating windows for HUD messages, similar to Raycast's showHUD().
//! HUDs are:
//! - Separate windows (not part of main app window)
//! - Floating above all other windows
//! - Positioned at bottom-center of the screen containing the mouse
//! - Auto-dismiss after configurable duration
//! - Queued if multiple arrive in sequence

use gpui::{
    div, point, prelude::*, px, rgb, size, App, Context, Pixels, Render, Timer, Window,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowOptions,
};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::logging;

/// Default HUD duration in milliseconds
const DEFAULT_HUD_DURATION_MS: u64 = 2000;

/// Gap between stacked HUDs
const HUD_STACK_GAP: f32 = 45.0;

/// Maximum number of simultaneous HUDs
const MAX_SIMULTANEOUS_HUDS: usize = 3;

/// HUD window dimensions - compact pill shape
const HUD_WIDTH: f32 = 200.0;
const HUD_HEIGHT: f32 = 36.0;

/// HUD with action button dimensions (wider to fit button)
#[allow(dead_code)]
const HUD_ACTION_WIDTH: f32 = 300.0;
#[allow(dead_code)]
const HUD_ACTION_HEIGHT: f32 = 40.0;

// =============================================================================
// HUD Actions - Clickable actions for HUD notifications
// =============================================================================

/// Action types that can be triggered from a HUD button click
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum HudAction {
    /// Open a file in the configured editor
    OpenFile(PathBuf),
    /// Open a URL in the default browser
    OpenUrl(String),
    /// Run a shell command
    RunCommand(String),
}

impl HudAction {
    /// Execute the action
    pub fn execute(&self, editor: Option<&str>) {
        match self {
            HudAction::OpenFile(path) => {
                let editor_cmd = editor.unwrap_or("code");
                logging::log(
                    "HUD",
                    &format!("Opening file {:?} with editor: {}", path, editor_cmd),
                );
                match std::process::Command::new(editor_cmd).arg(path).spawn() {
                    Ok(_) => logging::log("HUD", &format!("Opened file: {:?}", path)),
                    Err(e) => logging::log("HUD", &format!("Failed to open file: {}", e)),
                }
            }
            HudAction::OpenUrl(url) => {
                logging::log("HUD", &format!("Opening URL: {}", url));
                if let Err(e) = open::that(url) {
                    logging::log("HUD", &format!("Failed to open URL: {}", e));
                }
            }
            HudAction::RunCommand(cmd) => {
                logging::log("HUD", &format!("Running command: {}", cmd));
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if let Some((program, args)) = parts.split_first() {
                    if let Err(e) = std::process::Command::new(program).args(args).spawn() {
                        logging::log("HUD", &format!("Failed to run command: {}", e));
                    }
                }
            }
        }
    }
}

/// A single HUD notification
#[derive(Clone)]
pub struct HudNotification {
    pub text: String,
    pub duration_ms: u64,
    #[allow(dead_code)]
    pub created_at: Instant,
    /// Optional label for action button (e.g., "Open Logs", "View")
    #[allow(dead_code)]
    pub action_label: Option<String>,
    /// Optional action to execute when button is clicked
    #[allow(dead_code)]
    pub action: Option<HudAction>,
}

impl HudNotification {
    /// Check if this notification has an action button
    #[allow(dead_code)]
    pub fn has_action(&self) -> bool {
        self.action.is_some() && self.action_label.is_some()
    }
}

/// The visual component rendered inside each HUD window
struct HudView {
    text: String,
    #[allow(dead_code)]
    action_label: Option<String>,
    #[allow(dead_code)]
    action: Option<HudAction>,
}

impl HudView {
    fn new(text: String) -> Self {
        Self {
            text,
            action_label: None,
            action: None,
        }
    }

    #[allow(dead_code)]
    fn with_action(text: String, action_label: String, action: HudAction) -> Self {
        Self {
            text,
            action_label: Some(action_label),
            action: Some(action),
        }
    }

    #[allow(dead_code)]
    fn has_action(&self) -> bool {
        self.action.is_some() && self.action_label.is_some()
    }
}

impl Render for HudView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_action = self.has_action();

        // HUD pill styling: matches main window theme, minimal and clean
        // Similar to Raycast's HUD - simple, elegant, non-intrusive
        div()
            .id("hud-pill")
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .px(px(16.))
            .py(px(8.))
            .gap(px(12.))
            // Use theme-matching dark background (0x1e1e1e with full opacity)
            .bg(rgb(0x1e1e1e))
            .rounded(px(8.)) // Rounded corners matching main window
            // Text styling - system font, smaller size, white text, centered
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xFFFFFF))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.text.clone()),
            )
            // Action button (only if action is present)
            .when(has_action, |el| {
                let label = self.action_label.clone().unwrap_or_default();
                let action = self.action.clone();
                el.child(
                    div()
                        .id("hud-action-button")
                        .px(px(10.))
                        .py(px(4.))
                        .bg(rgb(0x0078d4)) // Accent blue
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x1084d8))) // Lighter on hover
                        .active(|s| s.bg(rgb(0x006cbe))) // Darker on press
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xFFFFFF))
                                .child(label),
                        )
                        .on_click(cx.listener(move |_this, _event, _window, _cx| {
                            if let Some(ref action) = action {
                                action.execute(None); // TODO: Get editor from config
                            }
                        })),
                )
            })
    }
}

/// Tracks an active HUD window
struct ActiveHud {
    #[allow(dead_code)]
    window: WindowHandle<HudView>,
    created_at: Instant,
    duration_ms: u64,
}

impl ActiveHud {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_millis() as u64 > self.duration_ms
    }
}

/// Global HUD manager state
struct HudManagerState {
    /// Currently displayed HUD windows
    active_huds: Vec<ActiveHud>,
    /// Queue of pending HUDs (if max simultaneous reached)
    pending_queue: VecDeque<HudNotification>,
}

impl HudManagerState {
    fn new() -> Self {
        Self {
            active_huds: Vec::new(),
            pending_queue: VecDeque::new(),
        }
    }
}

/// Global HUD manager singleton
static HUD_MANAGER: std::sync::OnceLock<Arc<Mutex<HudManagerState>>> = std::sync::OnceLock::new();

fn get_hud_manager() -> &'static Arc<Mutex<HudManagerState>> {
    HUD_MANAGER.get_or_init(|| Arc::new(Mutex::new(HudManagerState::new())))
}

/// Show a HUD notification
///
/// This creates a new floating window positioned at the bottom-center of the
/// screen containing the mouse cursor. The HUD auto-dismisses after the
/// specified duration.
///
/// # Arguments
/// * `text` - The message to display
/// * `duration_ms` - Optional duration in milliseconds (default: 2000ms)
/// * `cx` - GPUI App context
pub fn show_hud(text: String, duration_ms: Option<u64>, cx: &mut App) {
    let duration = duration_ms.unwrap_or(DEFAULT_HUD_DURATION_MS);

    logging::log(
        "HUD",
        &format!("Showing HUD: '{}' for {}ms", text, duration),
    );

    // Check if we can show immediately or need to queue
    let should_queue = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.active_huds.len() >= MAX_SIMULTANEOUS_HUDS
    };

    if should_queue {
        logging::log("HUD", "Max HUDs reached, queueing");
        let manager = get_hud_manager();
        let mut state = manager.lock();
        state.pending_queue.push_back(HudNotification {
            text,
            duration_ms: duration,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        });
        return;
    }

    // Calculate position - bottom center of screen with mouse
    let (hud_x, hud_y) = calculate_hud_position(cx);

    // Calculate vertical offset for stacking
    let stack_offset = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.active_huds.len() as f32 * HUD_STACK_GAP
    };

    let hud_width: Pixels = px(HUD_WIDTH);
    let hud_height: Pixels = px(HUD_HEIGHT);

    let bounds = gpui::Bounds {
        origin: point(px(hud_x), px(hud_y - stack_offset)),
        size: size(hud_width, hud_height),
    };

    let text_for_log = text.clone();
    let expected_bounds = bounds;

    // Create the HUD window with specific options for overlay behavior
    let window_result = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            focus: false, // Don't steal focus
            show: true,   // Show immediately
            ..Default::default()
        },
        |_, cx| cx.new(|_| HudView::new(text)),
    );

    match window_result {
        Ok(window_handle) => {
            // Configure the window as a floating overlay
            configure_hud_window_by_bounds(expected_bounds);

            // Clone window handle for the cleanup timer
            let window_for_cleanup = window_handle;

            // Track the active HUD
            {
                let manager = get_hud_manager();
                let mut state = manager.lock();
                state.active_huds.push(ActiveHud {
                    window: window_handle,
                    created_at: Instant::now(),
                    duration_ms: duration,
                });
            }

            // Schedule cleanup after duration - close HUD windows directly via NSWindow
            let duration_duration = Duration::from_millis(duration);
            let cleanup_bounds = expected_bounds;
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                Timer::after(duration_duration).await;

                // Close the NSWindow directly (don't use window_handle to avoid borrow issues)
                close_hud_window_by_bounds(cleanup_bounds);

                // Then clean up the tracking state
                let _ = cx.update(|cx| {
                    cleanup_expired_huds(cx);
                });

                // Drop the window handle reference
                let _ = window_for_cleanup;
            })
            .detach();

            logging::log(
                "HUD",
                &format!("HUD window created for: '{}'", text_for_log),
            );
        }
        Err(e) => {
            logging::log("HUD", &format!("Failed to create HUD window: {:?}", e));
        }
    }
}

/// Show a HUD notification with a clickable action button
///
/// This creates a HUD with a button that executes an action when clicked.
/// The HUD is wider to accommodate the button.
///
/// # Arguments
/// * `text` - The message to display
/// * `duration_ms` - Optional duration in milliseconds (default: 3000ms for action HUDs)
/// * `action_label` - Label for the action button (e.g., "Open Logs")
/// * `action` - The action to execute when the button is clicked
/// * `cx` - GPUI App context
#[allow(dead_code)]
pub fn show_hud_with_action(
    text: String,
    duration_ms: Option<u64>,
    action_label: String,
    action: HudAction,
    cx: &mut App,
) {
    // Action HUDs have longer default duration (3s) since user might click
    let duration = duration_ms.unwrap_or(3000);

    logging::log(
        "HUD",
        &format!(
            "Showing HUD with action: '{}' [{}] for {}ms",
            text, action_label, duration
        ),
    );

    // Check if we can show immediately or need to queue
    let should_queue = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.active_huds.len() >= MAX_SIMULTANEOUS_HUDS
    };

    if should_queue {
        logging::log("HUD", "Max HUDs reached, queueing action HUD");
        let manager = get_hud_manager();
        let mut state = manager.lock();
        state.pending_queue.push_back(HudNotification {
            text,
            duration_ms: duration,
            created_at: Instant::now(),
            action_label: Some(action_label),
            action: Some(action),
        });
        return;
    }

    // Calculate position - bottom center of screen with mouse
    let (hud_x, hud_y) = calculate_hud_position(cx);

    // Calculate vertical offset for stacking
    let stack_offset = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.active_huds.len() as f32 * HUD_STACK_GAP
    };

    // Use wider dimensions for action HUDs
    let hud_width: Pixels = px(HUD_ACTION_WIDTH);
    let hud_height: Pixels = px(HUD_ACTION_HEIGHT);

    // Adjust x position for wider HUD
    let adjusted_x = hud_x - (HUD_ACTION_WIDTH - HUD_WIDTH) / 2.0;

    let bounds = gpui::Bounds {
        origin: point(px(adjusted_x), px(hud_y - stack_offset)),
        size: size(hud_width, hud_height),
    };

    let text_for_log = text.clone();
    let expected_bounds = bounds;

    // Create the HUD window with action button
    let window_result = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            is_movable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            focus: false, // Don't steal focus
            show: true,   // Show immediately
            ..Default::default()
        },
        |_, cx| cx.new(|_| HudView::with_action(text, action_label, action)),
    );

    match window_result {
        Ok(window_handle) => {
            // Configure the window as a floating overlay
            configure_hud_window_by_bounds(expected_bounds);

            // Clone window handle for the cleanup timer
            let window_for_cleanup = window_handle;

            // Track the active HUD
            {
                let manager = get_hud_manager();
                let mut state = manager.lock();
                state.active_huds.push(ActiveHud {
                    window: window_handle,
                    created_at: Instant::now(),
                    duration_ms: duration,
                });
            }

            // Schedule cleanup after duration
            let duration_duration = Duration::from_millis(duration);
            let cleanup_bounds = expected_bounds;
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                Timer::after(duration_duration).await;

                // Close the NSWindow directly
                close_hud_window_by_bounds(cleanup_bounds);

                // Then clean up the tracking state
                let _ = cx.update(|cx| {
                    cleanup_expired_huds(cx);
                });

                // Drop the window handle reference
                let _ = window_for_cleanup;
            })
            .detach();

            logging::log(
                "HUD",
                &format!("Action HUD window created for: '{}'", text_for_log),
            );
        }
        Err(e) => {
            logging::log("HUD", &format!("Failed to create action HUD window: {:?}", e));
        }
    }
}

/// Close a HUD window by finding it based on its expected bounds
/// This avoids borrowing issues by not using WindowHandle
#[cfg(target_os = "macos")]
fn close_hud_window_by_bounds(expected_bounds: gpui::Bounds<Pixels>) {
    use cocoa::appkit::NSApp;
    use cocoa::base::id;
    use cocoa::foundation::NSRect;

    let expected_x: f32 = expected_bounds.origin.x.into();
    let expected_w: f32 = expected_bounds.size.width.into();
    let expected_h: f32 = expected_bounds.size.height.into();

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let ns_window: id = msg_send![windows, objectAtIndex: i];
            let frame: NSRect = msg_send![ns_window, frame];

            // Match by size and x position (HUD windows have distinctive dimensions)
            let w_match = (frame.size.width - expected_w as f64).abs() < 5.0;
            let h_match = (frame.size.height - expected_h as f64).abs() < 5.0;
            let x_match = (frame.origin.x - expected_x as f64).abs() < 5.0;

            if w_match && h_match && x_match {
                logging::log(
                    "HUD",
                    &format!(
                        "Closing HUD window at ({:.0}, {:.0})",
                        frame.origin.x, frame.origin.y
                    ),
                );
                let _: () = msg_send![ns_window, close];
                return;
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn close_hud_window_by_bounds(_expected_bounds: gpui::Bounds<Pixels>) {
    logging::log("HUD", "Non-macOS: HUD window cleanup not implemented");
}

/// Calculate HUD position - bottom center of screen containing mouse
fn calculate_hud_position(cx: &App) -> (f32, f32) {
    let displays = cx.displays();

    // Try to get mouse position
    let mouse_pos = crate::platform::get_global_mouse_position();

    // Find display containing mouse
    let target_display = if let Some((mouse_x, mouse_y)) = mouse_pos {
        displays.iter().find(|display| {
            let bounds = display.bounds();
            let x: f64 = bounds.origin.x.into();
            let y: f64 = bounds.origin.y.into();
            let w: f64 = bounds.size.width.into();
            let h: f64 = bounds.size.height.into();

            mouse_x >= x && mouse_x < x + w && mouse_y >= y && mouse_y < y + h
        })
    } else {
        None
    };

    // Use found display or primary
    let display = target_display.or_else(|| displays.first());

    if let Some(display) = display {
        let bounds = display.bounds();
        let screen_x: f32 = bounds.origin.x.into();
        let screen_y: f32 = bounds.origin.y.into();
        let screen_width: f32 = bounds.size.width.into();
        let screen_height: f32 = bounds.size.height.into();

        // Center horizontally, position at 85% down the screen
        let hud_x = screen_x + (screen_width - HUD_WIDTH) / 2.0;
        let hud_y = screen_y + screen_height * 0.85;

        (hud_x, hud_y)
    } else {
        // Fallback position
        (500.0, 800.0)
    }
}

/// Configure a HUD window by finding it based on expected bounds
#[cfg(target_os = "macos")]
fn configure_hud_window_by_bounds(expected_bounds: gpui::Bounds<Pixels>) {
    use cocoa::appkit::NSApp;
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSRect;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        let expected_x: f32 = expected_bounds.origin.x.into();
        let expected_width: f32 = expected_bounds.size.width.into();
        let expected_height: f32 = expected_bounds.size.height.into();

        // Find the window with matching dimensions (HUD-sized windows)
        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let frame: NSRect = msg_send![window, frame];

            // Check if this looks like our HUD window (by size)
            let width_matches = (frame.size.width - expected_width as f64).abs() < 5.0;
            let height_matches = (frame.size.height - expected_height as f64).abs() < 5.0;
            let x_matches = (frame.origin.x - expected_x as f64).abs() < 5.0;

            if width_matches && height_matches && x_matches {
                // Found it! Configure as HUD overlay

                // Set window level very high (NSPopUpMenuWindowLevel = 101)
                let hud_level: i32 = 101;
                let _: () = msg_send![window, setLevel: hud_level];

                // Collection behaviors for HUD:
                // - CanJoinAllSpaces (1): appear on all spaces
                // - Stationary (16): don't move with spaces
                // - IgnoresCycle (64): cmd-tab ignores this window
                let collection_behavior: u64 = 1 | 16 | 64;
                let _: () = msg_send![window, setCollectionBehavior: collection_behavior];

                // Ignore mouse events - click-through
                let _: () = msg_send![window, setIgnoresMouseEvents: true];

                // Don't show in window menu
                let _: () = msg_send![window, setExcludedFromWindowsMenu: true];

                // Order to front without activating the app
                let _: () = msg_send![window, orderFront: nil];

                logging::log(
                    "HUD",
                    &format!(
                        "Configured HUD NSWindow at ({:.0}, {:.0}): level={}, click-through, orderFront",
                        frame.origin.x, frame.origin.y, hud_level
                    ),
                );
                return;
            }
        }

        let expected_y: f32 = expected_bounds.origin.y.into();
        logging::log(
            "HUD",
            &format!(
                "Could not find HUD window with bounds ({:.0}, {:.0})",
                expected_x, expected_y
            ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_hud_window_by_bounds(_expected_bounds: gpui::Bounds<Pixels>) {
    logging::log(
        "HUD",
        "Non-macOS platform, skipping HUD window configuration",
    );
}

/// Clean up expired HUD windows and show pending ones
fn cleanup_expired_huds(cx: &mut App) {
    let manager = get_hud_manager();
    let mut state = manager.lock();

    // Remove expired HUDs from tracking
    let before_count = state.active_huds.len();
    state.active_huds.retain(|hud| !hud.is_expired());
    let removed = before_count - state.active_huds.len();

    if removed > 0 {
        logging::log("HUD", &format!("Cleaned up {} expired HUD(s)", removed));
    }

    // Show pending HUDs if we have room
    while state.active_huds.len() < MAX_SIMULTANEOUS_HUDS {
        if let Some(pending) = state.pending_queue.pop_front() {
            // Drop lock before showing HUD (show_hud will acquire it)
            drop(state);
            show_hud(pending.text, Some(pending.duration_ms), cx);
            // Re-acquire for next iteration
            state = manager.lock();
        } else {
            break;
        }
    }
}

/// Dismiss all active HUDs immediately
#[allow(dead_code)]
pub fn dismiss_all_huds() {
    let manager = get_hud_manager();
    let mut state = manager.lock();

    let count = state.active_huds.len();
    state.active_huds.clear();
    state.pending_queue.clear();

    if count > 0 {
        logging::log("HUD", &format!("Dismissed {} active HUD(s)", count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_notification_creation() {
        let notif = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        };
        assert_eq!(notif.text, "Test");
        assert_eq!(notif.duration_ms, 2000);
    }

    #[test]
    fn test_hud_manager_state_creation() {
        let state = HudManagerState::new();
        assert!(state.active_huds.is_empty());
        assert!(state.pending_queue.is_empty());
    }
}
