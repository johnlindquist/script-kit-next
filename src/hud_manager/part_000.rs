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
use crate::components::button::{Button, ButtonColors, ButtonVariant};
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
    #[allow(dead_code)] // Reserved for future use
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

    /// Create default light theme colors (fallback)
    #[cfg(test)]
    fn light_default() -> Self {
        Self {
            background: 0xfafafa,
            text_primary: 0x000000,
            accent: 0x2563eb,        // blue-600 (darker for light mode)
            accent_hover: 0x3b82f6,  // blue-500
            accent_active: 0x1d4ed8, // blue-700
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                // Theme-aware hover overlay: white for dark mode, black for light mode
                // Determined by checking if background color is dark (luminance < 0.5)
                let hover_overlay = {
                    let bg = colors.background;
                    let r = ((bg >> 16) & 0xFF) as f32 / 255.0;
                    let g = ((bg >> 8) & 0xFF) as f32 / 255.0;
                    let b = (bg & 0xFF) as f32 / 255.0;
                    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    if luminance < 0.5 {
                        0xffffff26 // White at ~15% alpha for dark backgrounds
                    } else {
                        0x00000026 // Black at ~15% alpha for light backgrounds
                    }
                };
                // Create button colors from HUD colors
                let button_colors = ButtonColors {
                    text_color: colors.text_primary,
                    text_hover: colors.text_primary,
                    background: colors.accent,
                    background_hover: colors.accent_hover,
                    accent: colors.text_primary, // Text on accent background
                    border: colors.accent,
                    focus_ring: colors.accent_hover,
                    focus_tint: colors.accent,
                    hover_overlay,
                };
                el.child(
                    Button::new(label, button_colors)
                        .variant(ButtonVariant::Primary)
                        .on_click(Box::new(move |_event, _window, _cx| {
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
    /// Window handle for closing via GPUI's proper API
    window: WindowHandle<HudView>,
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
    if let (Some(action_label), Some(action)) = (notif.action_label, notif.action) {
        show_hud_with_action(
            notif.text,
            Some(notif.duration_ms),
            action_label,
            action,
            cx,
        );
    } else {
        show_hud(notif.text, Some(notif.duration_ms), cx);
    }
}
