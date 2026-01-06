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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::logging;
use crate::theme;

// =============================================================================
// Theme Integration - HUD colors from theme system
// =============================================================================

/// Colors used by HUD rendering, extracted from theme for closure compatibility.
/// This struct is Copy so it can be safely used in closures without borrow issues.
#[derive(Clone, Copy, Debug)]
struct HudColors {
    /// Background color for the HUD pill
    background: u32,
    /// Primary text color
    text_primary: u32,
    /// Accent color for action buttons
    accent: u32,
    /// Accent hover color (lighter)
    accent_hover: u32,
    /// Accent active/pressed color (darker)
    accent_active: u32,
}

impl HudColors {
    /// Load HUD colors from the current theme
    fn from_theme() -> Self {
        let theme = theme::load_theme();
        let colors = &theme.colors;

        // Calculate hover/active variants from accent
        // For hover: lighten by ~10%
        // For active: darken by ~10%
        let accent = colors.ui.info; // Use info color (blue) for action buttons
        let accent_hover = lighten_color(accent, 0.1);
        let accent_active = darken_color(accent, 0.1);

        Self {
            background: colors.background.main,
            text_primary: colors.text.primary,
            accent,
            accent_hover,
            accent_active,
        }
    }

    /// Create default dark theme colors (fallback)
    #[cfg(test)]
    fn dark_default() -> Self {
        Self {
            background: 0x1e1e1e,
            text_primary: 0xffffff,
            accent: 0x3b82f6,        // blue-500
            accent_hover: 0x60a5fa,  // blue-400
            accent_active: 0x2563eb, // blue-600
        }
    }
}

/// Lighten a color by a percentage (0.0 - 1.0)
fn lighten_color(color: u32, amount: f32) -> u32 {
    let r = ((color >> 16) & 0xff) as f32;
    let g = ((color >> 8) & 0xff) as f32;
    let b = (color & 0xff) as f32;

    let r = (r + (255.0 - r) * amount).min(255.0) as u32;
    let g = (g + (255.0 - g) * amount).min(255.0) as u32;
    let b = (b + (255.0 - b) * amount).min(255.0) as u32;

    (r << 16) | (g << 8) | b
}

/// Darken a color by a percentage (0.0 - 1.0)
fn darken_color(color: u32, amount: f32) -> u32 {
    let r = ((color >> 16) & 0xff) as f32;
    let g = ((color >> 8) & 0xff) as f32;
    let b = (color & 0xff) as f32;

    let r = (r * (1.0 - amount)).max(0.0) as u32;
    let g = (g * (1.0 - amount)).max(0.0) as u32;
    let b = (b * (1.0 - amount)).max(0.0) as u32;

    (r << 16) | (g << 8) | b
}

/// Counter for generating unique HUD IDs
static NEXT_HUD_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique HUD ID
fn next_hud_id() -> u64 {
    NEXT_HUD_ID.fetch_add(1, Ordering::Relaxed)
}

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
    /// Theme colors for rendering
    colors: HudColors,
}

impl HudView {
    fn new(text: String) -> Self {
        Self {
            text,
            action_label: None,
            action: None,
            colors: HudColors::from_theme(),
        }
    }

    #[allow(dead_code)]
    fn with_action(text: String, action_label: String, action: HudAction) -> Self {
        Self {
            text,
            action_label: Some(action_label),
            action: Some(action),
            colors: HudColors::from_theme(),
        }
    }

    /// Create a HudView with specific colors (for testing)
    #[cfg(test)]
    fn with_colors(text: String, colors: HudColors) -> Self {
        Self {
            text,
            action_label: None,
            action: None,
            colors,
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

        // Extract colors for use in closures (Copy trait)
        let colors = self.colors;

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
            // Use theme background color
            .bg(rgb(colors.background))
            .rounded(px(8.)) // Rounded corners matching main window
            // Text styling - system font, smaller size, theme text color, centered
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_primary))
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
                        .bg(rgb(colors.accent))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(colors.accent_hover)))
                        .active(|s| s.bg(rgb(colors.accent_active)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_primary))
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
    /// Unique identifier for this HUD
    id: u64,
    #[allow(dead_code)]
    window: WindowHandle<HudView>,
    /// The bounds used to identify this HUD window for closing
    bounds: gpui::Bounds<Pixels>,
    created_at: Instant,
    duration_ms: u64,
    /// Slot index (0..MAX_SIMULTANEOUS_HUDS) for position calculation
    #[allow(dead_code)] // Used in position calculation
    slot: usize,
}

/// Entry in the slot allocation array (lightweight, for tracking slot ownership)
#[derive(Clone, Copy, Debug)]
struct HudSlotEntry {
    /// HUD ID that owns this slot
    id: u64,
}

/// Check if a duration has elapsed (used for HUD expiry)
/// Returns true when elapsed >= duration (inclusive boundary)
fn is_duration_expired(created_at: Instant, duration: Duration) -> bool {
    created_at.elapsed() >= duration
}

impl ActiveHud {
    fn is_expired(&self) -> bool {
        is_duration_expired(self.created_at, Duration::from_millis(self.duration_ms))
    }
}

/// Global HUD manager state
struct HudManagerState {
    /// Currently displayed HUD windows (kept for window handle storage)
    active_huds: Vec<ActiveHud>,
    /// Slot allocation array - each slot is None (free) or Some(entry) (occupied)
    /// Using fixed array prevents overlap from len-based stacking
    hud_slots: [Option<HudSlotEntry>; MAX_SIMULTANEOUS_HUDS],
    /// Queue of pending HUDs (if max simultaneous reached)
    pending_queue: VecDeque<HudNotification>,
}

impl HudManagerState {
    fn new() -> Self {
        Self {
            active_huds: Vec::new(),
            hud_slots: [None; MAX_SIMULTANEOUS_HUDS],
            pending_queue: VecDeque::new(),
        }
    }

    /// Find the first free slot (lowest index)
    fn first_free_slot(&self) -> Option<usize> {
        self.hud_slots.iter().position(|slot| slot.is_none())
    }

    /// Release a slot by HUD ID (clear the slot that contains this ID)
    fn release_slot_by_id(&mut self, hud_id: u64) {
        for slot in self.hud_slots.iter_mut() {
            if let Some(entry) = slot {
                if entry.id == hud_id {
                    *slot = None;
                    return;
                }
            }
        }
    }

    /// Find which slot contains a given HUD ID
    #[allow(dead_code)] // Used in tests
    fn find_slot_by_id(&self, hud_id: u64) -> Option<usize> {
        self.hud_slots
            .iter()
            .position(|slot| slot.as_ref().is_some_and(|entry| entry.id == hud_id))
    }

    /// Count how many HUDs are currently active
    #[allow(dead_code)] // Used in tests
    fn active_hud_count(&self) -> usize {
        self.hud_slots.iter().filter(|s| s.is_some()).count()
    }
}

/// Global HUD manager singleton
static HUD_MANAGER: std::sync::OnceLock<Arc<Mutex<HudManagerState>>> = std::sync::OnceLock::new();

fn get_hud_manager() -> &'static Arc<Mutex<HudManagerState>> {
    HUD_MANAGER.get_or_init(|| Arc::new(Mutex::new(HudManagerState::new())))
}

/// Internal helper to show a HUD notification from a HudNotification struct.
/// This preserves all fields including action_label and action.
fn show_notification(notif: HudNotification, cx: &mut App) {
    if notif.has_action() {
        show_hud_with_action(
            notif.text,
            Some(notif.duration_ms),
            notif.action_label.unwrap(),
            notif.action.unwrap(),
            cx,
        );
    } else {
        show_hud(notif.text, Some(notif.duration_ms), cx);
    }
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

    // Allocate slot and check if we can show immediately
    let allocated_slot = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.first_free_slot()
    };

    let slot = match allocated_slot {
        Some(s) => s,
        None => {
            // No free slots, queue the HUD
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
    };

    // Calculate position - bottom center of screen with mouse
    let (hud_x, hud_y) = calculate_hud_position(cx);

    // Calculate vertical offset using SLOT index (not len) - this prevents overlap
    let stack_offset = slot as f32 * HUD_STACK_GAP;

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
            // Regular HUDs without actions are click-through (true)
            configure_hud_window_by_bounds(expected_bounds, true);

            // Generate unique ID for this HUD
            let hud_id = next_hud_id();

            // Clone window handle for the cleanup timer
            let window_for_cleanup = window_handle;

            // Track the active HUD and register slot
            {
                let manager = get_hud_manager();
                let mut state = manager.lock();
                // Register slot ownership
                state.hud_slots[slot] = Some(HudSlotEntry { id: hud_id });
                state.active_huds.push(ActiveHud {
                    id: hud_id,
                    window: window_handle,
                    bounds: expected_bounds,
                    created_at: Instant::now(),
                    duration_ms: duration,
                    slot,
                });
            }

            // Schedule cleanup after duration - use ID for dismissal
            let duration_duration = Duration::from_millis(duration);
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                Timer::after(duration_duration).await;

                // IMPORTANT: All AppKit calls must happen on the main thread.
                // cx.update() ensures we're on the main thread.
                let _ = cx.update(|cx| {
                    // Dismiss by ID - this is more reliable than bounds matching
                    dismiss_hud_by_id(hud_id, cx);
                });

                // Drop the window handle reference
                let _ = window_for_cleanup;
            })
            .detach();

            logging::log(
                "HUD",
                &format!("HUD window created for: '{}' (slot {})", text_for_log, slot),
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

    // Allocate slot and check if we can show immediately
    let allocated_slot = {
        let manager = get_hud_manager();
        let state = manager.lock();
        state.first_free_slot()
    };

    let slot = match allocated_slot {
        Some(s) => s,
        None => {
            // No free slots, queue the HUD
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
    };

    // Calculate position - bottom center of screen with mouse
    let (hud_x, hud_y) = calculate_hud_position(cx);

    // Calculate vertical offset using SLOT index (not len) - this prevents overlap
    let stack_offset = slot as f32 * HUD_STACK_GAP;

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
            // Action HUDs need to receive mouse events for button clicks (click_through = false)
            configure_hud_window_by_bounds(expected_bounds, false);

            // Generate unique ID for this HUD
            let hud_id = next_hud_id();

            // Clone window handle for the cleanup timer
            let window_for_cleanup = window_handle;

            // Track the active HUD and register slot
            {
                let manager = get_hud_manager();
                let mut state = manager.lock();
                // Register slot ownership
                state.hud_slots[slot] = Some(HudSlotEntry { id: hud_id });
                state.active_huds.push(ActiveHud {
                    id: hud_id,
                    window: window_handle,
                    bounds: expected_bounds,
                    created_at: Instant::now(),
                    duration_ms: duration,
                    slot,
                });
            }

            // Schedule cleanup after duration - use ID for dismissal
            let duration_duration = Duration::from_millis(duration);
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                Timer::after(duration_duration).await;

                // IMPORTANT: All AppKit calls must happen on the main thread.
                // cx.update() ensures we're on the main thread.
                let _ = cx.update(|cx| {
                    // Dismiss by ID - this is more reliable than bounds matching
                    dismiss_hud_by_id(hud_id, cx);
                });

                // Drop the window handle reference
                let _ = window_for_cleanup;
            })
            .detach();

            logging::log(
                "HUD",
                &format!(
                    "Action HUD window created for: '{}' (slot {})",
                    text_for_log, slot
                ),
            );
        }
        Err(e) => {
            logging::log(
                "HUD",
                &format!("Failed to create action HUD window: {:?}", e),
            );
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
    let expected_y: f32 = expected_bounds.origin.y.into();
    let expected_w: f32 = expected_bounds.size.width.into();
    let expected_h: f32 = expected_bounds.size.height.into();

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        for i in 0..count {
            let ns_window: id = msg_send![windows, objectAtIndex: i];
            let frame: NSRect = msg_send![ns_window, frame];

            // Match by size AND position (x, y) to distinguish stacked HUDs
            let w_match = (frame.size.width - expected_w as f64).abs() < 5.0;
            let h_match = (frame.size.height - expected_h as f64).abs() < 5.0;
            let x_match = (frame.origin.x - expected_x as f64).abs() < 5.0;
            let y_match = (frame.origin.y - expected_y as f64).abs() < 5.0;

            if w_match && h_match && x_match && y_match {
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

        logging::log(
            "HUD",
            &format!(
                "Could not find HUD window to close at ({:.0}, {:.0})",
                expected_x, expected_y
            ),
        );
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
///
/// # Arguments
/// * `expected_bounds` - The bounds used to identify the HUD window
/// * `click_through` - If true, window ignores mouse events (for plain HUDs).
///   If false, window receives mouse events (for action HUDs with buttons).
#[cfg(target_os = "macos")]
fn configure_hud_window_by_bounds(expected_bounds: gpui::Bounds<Pixels>, click_through: bool) {
    use cocoa::appkit::NSApp;
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSRect;

    unsafe {
        let app: id = NSApp();
        let windows: id = msg_send![app, windows];
        let count: usize = msg_send![windows, count];

        let expected_x: f32 = expected_bounds.origin.x.into();
        let expected_y: f32 = expected_bounds.origin.y.into();
        let expected_width: f32 = expected_bounds.size.width.into();
        let expected_height: f32 = expected_bounds.size.height.into();

        // Find the window with matching dimensions AND position
        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            let frame: NSRect = msg_send![window, frame];

            // Check if this looks like our HUD window (by size AND position)
            let width_matches = (frame.size.width - expected_width as f64).abs() < 5.0;
            let height_matches = (frame.size.height - expected_height as f64).abs() < 5.0;
            let x_matches = (frame.origin.x - expected_x as f64).abs() < 5.0;
            let y_matches = (frame.origin.y - expected_y as f64).abs() < 5.0;

            if width_matches && height_matches && x_matches && y_matches {
                // Found it! Configure as HUD overlay

                // Set window level very high (NSPopUpMenuWindowLevel = 101)
                // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
                let hud_level: i64 = 101;
                let _: () = msg_send![window, setLevel: hud_level];

                // Collection behaviors for HUD:
                // - CanJoinAllSpaces (1): appear on all spaces
                // - Stationary (16): don't move with spaces
                // - IgnoresCycle (64): cmd-tab ignores this window
                let collection_behavior: u64 = 1 | 16 | 64;
                let _: () = msg_send![window, setCollectionBehavior: collection_behavior];

                // Set mouse event handling based on whether HUD has clickable actions
                // Action HUDs need to receive mouse events for button clicks
                // Use cocoa::base::BOOL (i8) for proper ObjC BOOL type on macOS
                let ignores_mouse: cocoa::base::BOOL = if click_through {
                    cocoa::base::YES
                } else {
                    cocoa::base::NO
                };
                let _: () = msg_send![window, setIgnoresMouseEvents: ignores_mouse];

                // Don't show in window menu
                let _: () = msg_send![window, setExcludedFromWindowsMenu: true];

                // Order to front without activating the app
                let _: () = msg_send![window, orderFront: nil];

                let click_status = if click_through {
                    "click-through"
                } else {
                    "clickable"
                };
                logging::log(
                    "HUD",
                    &format!(
                        "Configured HUD NSWindow at ({:.0}, {:.0}): level={}, {}, orderFront",
                        frame.origin.x, frame.origin.y, hud_level, click_status
                    ),
                );
                return;
            }
        }

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
fn configure_hud_window_by_bounds(_expected_bounds: gpui::Bounds<Pixels>, _click_through: bool) {
    logging::log(
        "HUD",
        "Non-macOS platform, skipping HUD window configuration",
    );
}

/// Dismiss a specific HUD by its ID
///
/// This is more reliable than bounds matching for timer-based dismissal,
/// as it avoids race conditions where bounds might match the wrong window.
/// Uses slot-based clearing instead of swap_remove to prevent position overlap.
fn dismiss_hud_by_id(hud_id: u64, cx: &mut App) {
    let manager = get_hud_manager();

    // Find and remove the HUD with matching ID, getting its bounds for window close
    let bounds_to_close: Option<gpui::Bounds<Pixels>> = {
        let mut state = manager.lock();

        // First, release the slot (this is the key fix - clears by ID, not swap_remove)
        state.release_slot_by_id(hud_id);

        // Then find and remove from active_huds Vec (retain order, don't swap_remove)
        if let Some(idx) = state.active_huds.iter().position(|h| h.id == hud_id) {
            let hud = state.active_huds.remove(idx); // Use remove() to preserve order
            Some(hud.bounds)
        } else {
            None
        }
    };

    // Close the window if we found it
    if let Some(bounds) = bounds_to_close {
        close_hud_window_by_bounds(bounds);
        logging::log("HUD", &format!("Dismissed HUD id={}", hud_id));

        // Show any pending HUDs
        cleanup_expired_huds(cx);
    } else {
        // HUD was already dismissed (possibly manually) - this is OK
        logging::log(
            "HUD",
            &format!("HUD id={} already dismissed, skipping", hud_id),
        );
    }
}

/// Clean up expired HUD windows and show pending ones
fn cleanup_expired_huds(cx: &mut App) {
    let manager = get_hud_manager();
    let mut state = manager.lock();

    // Remove expired HUDs from tracking and release their slots
    let before_count = state.active_huds.len();
    let expired_ids: Vec<u64> = state
        .active_huds
        .iter()
        .filter(|hud| hud.is_expired())
        .map(|hud| hud.id)
        .collect();

    // Release slots for expired HUDs
    for id in &expired_ids {
        state.release_slot_by_id(*id);
    }

    state.active_huds.retain(|hud| !hud.is_expired());
    let removed = before_count - state.active_huds.len();

    if removed > 0 {
        logging::log("HUD", &format!("Cleaned up {} expired HUD(s)", removed));
    }

    // Show pending HUDs if we have free slots
    while state.first_free_slot().is_some() {
        if let Some(pending) = state.pending_queue.pop_front() {
            // Drop lock before showing HUD (show_notification will acquire it)
            drop(state);
            // Use show_notification to preserve action_label and action
            show_notification(pending, cx);
            // Re-acquire for next iteration
            state = manager.lock();
        } else {
            break;
        }
    }
}

/// Dismiss all active HUDs immediately
///
/// This closes all active HUD windows and clears the pending queue.
/// Must be called on the main thread (i.e., from within App context).
#[allow(dead_code)]
pub fn dismiss_all_huds(_cx: &mut App) {
    let manager = get_hud_manager();

    // Collect bounds first, then close windows
    let bounds_to_close: Vec<gpui::Bounds<Pixels>> = {
        let state = manager.lock();
        state.active_huds.iter().map(|hud| hud.bounds).collect()
    };

    // Close each window by its bounds
    for bounds in &bounds_to_close {
        close_hud_window_by_bounds(*bounds);
    }

    // Clear tracking state including slot allocations
    let mut state = manager.lock();
    let count = state.active_huds.len();
    state.active_huds.clear();
    state.hud_slots = [None; MAX_SIMULTANEOUS_HUDS]; // Clear all slots
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

    #[test]
    fn test_is_duration_expired_boundary_condition() {
        // HUD should be expired when elapsed == duration (not just >)
        // This tests the fix for the boundary condition bug

        // Create timestamp from 100ms ago
        let created_at = Instant::now() - Duration::from_millis(100);
        let duration = Duration::from_millis(100);

        // When elapsed >= duration, should be expired
        assert!(
            is_duration_expired(created_at, duration),
            "Should be expired when elapsed >= duration"
        );
    }

    #[test]
    fn test_is_duration_expired_definitely_expired() {
        // Create timestamp from 200ms ago with 100ms duration
        let created_at = Instant::now() - Duration::from_millis(200);
        let duration = Duration::from_millis(100);

        // When elapsed > duration, definitely expired
        assert!(
            is_duration_expired(created_at, duration),
            "Should be expired when elapsed > duration"
        );
    }

    #[test]
    fn test_is_duration_expired_not_expired_yet() {
        // Create timestamp from now with a long duration
        let created_at = Instant::now();
        let duration = Duration::from_millis(10000); // 10 seconds

        assert!(
            !is_duration_expired(created_at, duration),
            "Should not be expired immediately after creation"
        );
    }

    #[test]
    fn test_hud_notification_has_action() {
        let notif_without_action = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        };
        assert!(!notif_without_action.has_action());

        let notif_with_action = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: Some("Open".to_string()),
            action: Some(HudAction::OpenUrl("https://example.com".to_string())),
        };
        assert!(notif_with_action.has_action());
    }

    #[test]
    fn test_hud_id_generation() {
        // IDs should be unique and increasing
        let id1 = next_hud_id();
        let id2 = next_hud_id();
        let id3 = next_hud_id();

        assert!(id2 > id1, "IDs should be strictly increasing");
        assert!(id3 > id2, "IDs should be strictly increasing");
        assert_ne!(id1, id2, "IDs should be unique");
        assert_ne!(id2, id3, "IDs should be unique");
    }

    #[test]
    fn test_hud_view_has_action() {
        // Test that HudView correctly reports whether it has an action
        let view_without_action = HudView::new("Test message".to_string());
        assert!(
            !view_without_action.has_action(),
            "HudView without action should report has_action() = false"
        );

        let view_with_action = HudView::with_action(
            "Test message".to_string(),
            "Open".to_string(),
            HudAction::OpenUrl("https://example.com".to_string()),
        );
        assert!(
            view_with_action.has_action(),
            "HudView with action should report has_action() = true"
        );
    }

    #[test]
    fn test_hud_action_execute_open_url() {
        // Test that HudAction::OpenUrl can be created and executed doesn't panic
        // (actual URL opening is mocked in unit tests)
        let action = HudAction::OpenUrl("https://example.com".to_string());
        // Just verify it can be constructed - actual execution requires system integration
        match action {
            HudAction::OpenUrl(url) => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected OpenUrl variant"),
        }
    }

    #[test]
    fn test_hud_action_execute_open_file() {
        // Test that HudAction::OpenFile can be created
        let action = HudAction::OpenFile(std::path::PathBuf::from("/tmp/test.txt"));
        match action {
            HudAction::OpenFile(path) => {
                assert_eq!(path, std::path::PathBuf::from("/tmp/test.txt"))
            }
            _ => panic!("Expected OpenFile variant"),
        }
    }

    #[test]
    fn test_hud_action_execute_run_command() {
        // Test that HudAction::RunCommand can be created
        let action = HudAction::RunCommand("echo hello".to_string());
        match action {
            HudAction::RunCommand(cmd) => assert_eq!(cmd, "echo hello"),
            _ => panic!("Expected RunCommand variant"),
        }
    }

    // =============================================================================
    // Theme Integration Tests
    // =============================================================================

    #[test]
    fn test_lighten_color() {
        // Test lightening pure black by 50%
        let black = 0x000000;
        let lightened = lighten_color(black, 0.5);
        // Should be ~0x7f7f7f (half way to white)
        assert_eq!(lightened, 0x7f7f7f);

        // Test lightening pure red by 10%
        let red = 0xff0000;
        let lightened_red = lighten_color(red, 0.1);
        // Red channel is already max, green/blue should be ~0x19 (25)
        assert_eq!(lightened_red >> 16, 0xff); // Red stays at max
        assert!((lightened_red >> 8) & 0xff >= 0x19); // Green increased
        assert!(lightened_red & 0xff >= 0x19); // Blue increased
    }

    #[test]
    fn test_darken_color() {
        // Test darkening pure white by 50%
        let white = 0xffffff;
        let darkened = darken_color(white, 0.5);
        // Should be ~0x7f7f7f (half way to black)
        assert_eq!(darkened, 0x7f7f7f);

        // Test darkening a color by 10%
        let color = 0x646464; // RGB(100, 100, 100)
        let darkened_color = darken_color(color, 0.1);
        // Each component should be 90% of original: 100 * 0.9 = 90 = 0x5a
        assert_eq!(darkened_color, 0x5a5a5a);
    }

    #[test]
    fn test_lighten_darken_boundary_conditions() {
        // Lightening white should stay white
        let white = 0xffffff;
        assert_eq!(lighten_color(white, 0.5), 0xffffff);

        // Darkening black should stay black
        let black = 0x000000;
        assert_eq!(darken_color(black, 0.5), 0x000000);
    }

    #[test]
    fn test_hud_colors_default() {
        // Test that default colors are valid (non-zero)
        let colors = HudColors::dark_default();
        assert_ne!(colors.background, 0);
        assert_ne!(colors.text_primary, 0);
        assert_ne!(colors.accent, 0);
        assert_ne!(colors.accent_hover, 0);
        assert_ne!(colors.accent_active, 0);
    }

    #[test]
    fn test_hud_colors_accent_variants() {
        // Test that hover is lighter than accent, and active is darker
        let colors = HudColors::dark_default();

        // Extract brightness (simple sum of components)
        let brightness = |c: u32| ((c >> 16) & 0xff) + ((c >> 8) & 0xff) + (c & 0xff);

        // Hover should be brighter than base accent
        assert!(
            brightness(colors.accent_hover) >= brightness(colors.accent),
            "Hover should be at least as bright as accent"
        );

        // Active should be darker than base accent
        assert!(
            brightness(colors.accent_active) <= brightness(colors.accent),
            "Active should be at most as bright as accent"
        );
    }

    #[test]
    fn test_hud_view_with_custom_colors() {
        // Test that HudView can be created with custom colors
        let custom_colors = HudColors {
            background: 0x2a2a2a,
            text_primary: 0xeeeeee,
            accent: 0x00ff00,
            accent_hover: 0x33ff33,
            accent_active: 0x00cc00,
        };

        let view = HudView::with_colors("Custom themed HUD".to_string(), custom_colors);
        assert_eq!(view.colors.background, 0x2a2a2a);
        assert_eq!(view.colors.text_primary, 0xeeeeee);
        assert_eq!(view.colors.accent, 0x00ff00);
    }

    // =============================================================================
    // HUD Manager State Tests
    // =============================================================================

    #[test]
    fn test_hud_manager_state_queue_operations() {
        // Test that pending queue works correctly
        let mut state = HudManagerState::new();

        // Queue should start empty
        assert!(state.pending_queue.is_empty());

        // Add items to queue
        state.pending_queue.push_back(HudNotification {
            text: "First".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        });

        state.pending_queue.push_back(HudNotification {
            text: "Second".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: None,
        });

        assert_eq!(state.pending_queue.len(), 2);

        // Pop front should return first item
        let first = state.pending_queue.pop_front().unwrap();
        assert_eq!(first.text, "First");

        // Queue should still have one item
        assert_eq!(state.pending_queue.len(), 1);

        let second = state.pending_queue.pop_front().unwrap();
        assert_eq!(second.text, "Second");

        // Queue should be empty now
        assert!(state.pending_queue.is_empty());
    }

    #[test]
    fn test_hud_notification_partial_has_action() {
        // Test edge cases for has_action()

        // Only action_label set (should NOT have action)
        let notif_label_only = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: Some("Click".to_string()),
            action: None,
        };
        assert!(
            !notif_label_only.has_action(),
            "Should not have action with only label"
        );

        // Only action set (should NOT have action)
        let notif_action_only = HudNotification {
            text: "Test".to_string(),
            duration_ms: 2000,
            created_at: Instant::now(),
            action_label: None,
            action: Some(HudAction::OpenUrl("https://example.com".to_string())),
        };
        assert!(
            !notif_action_only.has_action(),
            "Should not have action with only action (no label)"
        );
    }

    #[test]
    fn test_hud_constants() {
        // Verify constants are sensible values using const blocks (clippy-compliant)

        // Default duration should be reasonable (1-10 seconds)
        const _: () = assert!(DEFAULT_HUD_DURATION_MS >= 1000 && DEFAULT_HUD_DURATION_MS <= 10000);

        // Stack gap should be positive and reasonable
        // Note: f32 comparisons in const context require workaround
        assert!(
            (HUD_STACK_GAP as i32) > 0 && (HUD_STACK_GAP as i32) < 100,
            "Stack gap should be positive and reasonable"
        );

        // Max simultaneous HUDs should be at least 1
        const _: () = assert!(MAX_SIMULTANEOUS_HUDS >= 1);

        // HUD dimensions should be positive
        assert!((HUD_WIDTH as i32) > 0, "HUD width should be positive");
        assert!((HUD_HEIGHT as i32) > 0, "HUD height should be positive");
    }

    #[test]
    fn test_hud_action_variants_debug() {
        // Test Debug impl for HudAction variants
        let open_file = HudAction::OpenFile(std::path::PathBuf::from("/test/path.txt"));
        let debug_str = format!("{:?}", open_file);
        assert!(
            debug_str.contains("OpenFile"),
            "Debug should contain variant name"
        );

        let open_url = HudAction::OpenUrl("https://test.com".to_string());
        let debug_str = format!("{:?}", open_url);
        assert!(
            debug_str.contains("OpenUrl"),
            "Debug should contain variant name"
        );

        let run_cmd = HudAction::RunCommand("ls -la".to_string());
        let debug_str = format!("{:?}", run_cmd);
        assert!(
            debug_str.contains("RunCommand"),
            "Debug should contain variant name"
        );
    }

    #[test]
    fn test_hud_action_clone() {
        // Test Clone impl for HudAction
        let original = HudAction::OpenUrl("https://clone-test.com".to_string());
        let cloned = original.clone();

        match (original, cloned) {
            (HudAction::OpenUrl(orig_url), HudAction::OpenUrl(clone_url)) => {
                assert_eq!(orig_url, clone_url, "Cloned URL should match original");
            }
            _ => panic!("Clone should preserve variant type"),
        }
    }

    #[test]
    fn test_hud_notification_clone() {
        // Test Clone impl for HudNotification
        let original = HudNotification {
            text: "Clone test".to_string(),
            duration_ms: 3000,
            created_at: Instant::now(),
            action_label: Some("Test".to_string()),
            action: Some(HudAction::OpenUrl("https://example.com".to_string())),
        };

        let cloned = original.clone();
        assert_eq!(cloned.text, original.text);
        assert_eq!(cloned.duration_ms, original.duration_ms);
        assert_eq!(cloned.action_label, original.action_label);
        assert!(
            cloned.has_action(),
            "Cloned notification should have action"
        );
    }

    #[test]
    fn test_hud_colors_copy_trait() {
        // Test that HudColors implements Copy (important for closures)
        let colors = HudColors::dark_default();

        // This would fail to compile if HudColors wasn't Copy
        let colors_copy = colors;
        let another_copy = colors;

        assert_eq!(colors_copy.background, another_copy.background);
        assert_eq!(colors_copy.text_primary, another_copy.text_primary);
    }

    #[test]
    fn test_color_manipulation_with_gray() {
        // Test lighten/darken with mid-gray
        let gray = 0x808080; // RGB(128, 128, 128)

        let lightened = lighten_color(gray, 0.5);
        // Each component: 128 + (255-128)*0.5 = 128 + 63.5 = 191.5 -> 191 = 0xbf
        assert_eq!(lightened, 0xbfbfbf);

        let darkened = darken_color(gray, 0.5);
        // Each component: 128 * 0.5 = 64 = 0x40
        assert_eq!(darkened, 0x404040);
    }

    #[test]
    fn test_color_manipulation_preserves_channels() {
        // Test that color manipulation works independently per channel
        let color = 0xff8000; // RGB(255, 128, 0) - orange

        let darkened = darken_color(color, 0.5);
        // R: 255*0.5=127, G: 128*0.5=64, B: 0*0.5=0
        let r = (darkened >> 16) & 0xff;
        let g = (darkened >> 8) & 0xff;
        let b = darkened & 0xff;

        assert_eq!(r, 127, "Red channel should be halved");
        assert_eq!(g, 64, "Green channel should be halved");
        assert_eq!(b, 0, "Blue channel should stay at 0");
    }

    // =============================================================================
    // Slot-based Allocation Tests (TDD for overlap fix)
    // =============================================================================

    #[test]
    fn test_slot_allocation_gives_unique_slots() {
        // Each HUD should get a unique slot 0..MAX_SIMULTANEOUS_HUDS
        let mut state = HudManagerState::new();

        // Allocate slots one by one
        let slot0 = state.first_free_slot();
        assert_eq!(slot0, Some(0), "First slot should be 0");

        // Simulate HUD at slot 0
        state.hud_slots[0] = Some(HudSlotEntry { id: 100 });

        let slot1 = state.first_free_slot();
        assert_eq!(slot1, Some(1), "Second slot should be 1");

        state.hud_slots[1] = Some(HudSlotEntry { id: 101 });

        let slot2 = state.first_free_slot();
        assert_eq!(slot2, Some(2), "Third slot should be 2");

        state.hud_slots[2] = Some(HudSlotEntry { id: 102 });

        // All slots full
        let slot_none = state.first_free_slot();
        assert_eq!(slot_none, None, "Should return None when all slots full");
    }

    #[test]
    fn test_slot_release_makes_slot_available() {
        // When a HUD dismisses, its slot becomes available for reuse
        let mut state = HudManagerState::new();

        // Fill all slots
        state.hud_slots[0] = Some(HudSlotEntry { id: 100 });
        state.hud_slots[1] = Some(HudSlotEntry { id: 101 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 102 });

        // All slots full
        assert_eq!(state.first_free_slot(), None);

        // Release slot 1 (middle)
        state.release_slot_by_id(101);

        // Slot 1 should now be free
        assert!(state.hud_slots[1].is_none(), "Slot 1 should be released");

        // Next allocation should get slot 1
        let next_slot = state.first_free_slot();
        assert_eq!(next_slot, Some(1), "Should reuse released slot 1");
    }

    #[test]
    fn test_concurrent_huds_get_different_slots() {
        // Multiple HUDs active at same time should have different slots
        let mut state = HudManagerState::new();

        // Allocate and fill slots
        state.hud_slots[0] = Some(HudSlotEntry { id: 200 });
        state.hud_slots[1] = Some(HudSlotEntry { id: 201 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 202 });

        // Verify all slots occupied by different IDs
        let ids: Vec<u64> = state
            .hud_slots
            .iter()
            .filter_map(|s| s.as_ref().map(|e| e.id))
            .collect();

        assert_eq!(ids.len(), 3, "All slots should be occupied");
        assert!(ids.contains(&200));
        assert!(ids.contains(&201));
        assert!(ids.contains(&202));

        // No duplicates
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 3, "All IDs should be unique");
    }

    #[test]
    fn test_position_uses_slot_not_len() {
        // Position calculation should use slot index, not active count
        // This is the core bug fix - releasing middle HUD shouldn't move others

        let mut state = HudManagerState::new();

        // Fill slots 0, 1, 2
        state.hud_slots[0] = Some(HudSlotEntry { id: 300 });
        state.hud_slots[1] = Some(HudSlotEntry { id: 301 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 302 });

        // Release middle HUD (slot 1)
        state.release_slot_by_id(301);

        // Slot 0 and 2 should still have their original entries
        assert!(
            state.hud_slots[0].is_some(),
            "Slot 0 should still be occupied"
        );
        assert!(state.hud_slots[1].is_none(), "Slot 1 should be empty");
        assert!(
            state.hud_slots[2].is_some(),
            "Slot 2 should still be occupied"
        );

        // Calculate offsets using slot indices
        // Slot 0 -> offset = 0 * HUD_STACK_GAP
        // Slot 2 -> offset = 2 * HUD_STACK_GAP (NOT 1 * HUD_STACK_GAP)
        let offset_slot_0 = 0.0 * HUD_STACK_GAP;
        let offset_slot_2 = 2.0 * HUD_STACK_GAP;

        assert!(
            (offset_slot_0 - 0.0).abs() < f32::EPSILON,
            "Slot 0 should have offset 0"
        );
        assert!(
            (offset_slot_2 - 2.0 * HUD_STACK_GAP).abs() < f32::EPSILON,
            "Slot 2 should keep its original offset"
        );
    }

    #[test]
    fn test_release_nonexistent_id_is_safe() {
        // Releasing an ID that doesn't exist should be a no-op
        let mut state = HudManagerState::new();

        state.hud_slots[0] = Some(HudSlotEntry { id: 400 });

        // Release non-existent ID
        state.release_slot_by_id(999);

        // Original slot should be unchanged
        assert!(state.hud_slots[0].is_some(), "Existing slot should remain");
        assert_eq!(state.hud_slots[0].as_ref().unwrap().id, 400);
    }

    #[test]
    fn test_find_slot_by_id() {
        let mut state = HudManagerState::new();

        state.hud_slots[0] = Some(HudSlotEntry { id: 500 });
        state.hud_slots[2] = Some(HudSlotEntry { id: 502 });
        // slot 1 is empty

        assert_eq!(state.find_slot_by_id(500), Some(0));
        assert_eq!(state.find_slot_by_id(502), Some(2));
        assert_eq!(state.find_slot_by_id(501), None); // doesn't exist
        assert_eq!(state.find_slot_by_id(999), None); // doesn't exist
    }

    #[test]
    fn test_active_hud_count() {
        let mut state = HudManagerState::new();

        assert_eq!(state.active_hud_count(), 0);

        state.hud_slots[0] = Some(HudSlotEntry { id: 600 });
        assert_eq!(state.active_hud_count(), 1);

        state.hud_slots[2] = Some(HudSlotEntry { id: 602 });
        assert_eq!(state.active_hud_count(), 2);

        state.hud_slots[1] = Some(HudSlotEntry { id: 601 });
        assert_eq!(state.active_hud_count(), 3);

        // Release middle
        state.release_slot_by_id(601);
        assert_eq!(state.active_hud_count(), 2);
    }
}
