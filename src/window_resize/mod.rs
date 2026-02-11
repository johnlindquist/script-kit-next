//! Dynamic Window Resizing Module
//!
//! Handles window height for different view types in Script Kit GPUI.
//!
//! **Key Rules:**
//! - ScriptList (main window with preview): FIXED at 500px, never resizes
//! - ArgPrompt with choices: Dynamic height based on choice count (capped at 500px)
//! - ArgPrompt without choices (input only): Compact input-only height
//! - Editor/Div/Term: Full height 700px

// --- merged from part_000.rs ---
use crate::config::{self, LayoutConfig};
use crate::list_item::LIST_ITEM_HEIGHT;
use crate::logging;
use crate::window_manager;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSPoint, NSRect, NSSize};
use gpui::{px, Pixels};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
use std::sync::OnceLock;
use tracing::{debug, warn};
const RESIZE_MIN_DELTA_PX: f64 = 1.0;
const WINDOW_RESIZE_ANIMATE: bool = false;
#[derive(Debug, Clone, Copy, PartialEq)]
struct FrameGeometry {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
impl FrameGeometry {
    const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
#[cfg(target_os = "macos")]
impl FrameGeometry {
    fn from_ns_rect(rect: NSRect) -> Self {
        Self::new(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        )
    }

    fn to_ns_rect(self) -> NSRect {
        NSRect::new(
            NSPoint::new(self.x, self.y),
            NSSize::new(self.width, self.height),
        )
    }
}
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
struct ScreenGeometry {
    visible_bounds: FrameGeometry,
    backing_scale: f64,
}
fn sanitize_backing_scale(backing_scale: Option<f64>) -> Option<f64> {
    backing_scale.filter(|scale| scale.is_finite() && *scale > 0.0)
}
fn round_to_backing_scale(value: f64, backing_scale: f64) -> f64 {
    (value * backing_scale).round() / backing_scale
}
fn normalize_frame_to_backing_scale(frame: FrameGeometry, backing_scale: f64) -> FrameGeometry {
    FrameGeometry::new(
        round_to_backing_scale(frame.x, backing_scale),
        round_to_backing_scale(frame.y, backing_scale),
        round_to_backing_scale(frame.width, backing_scale),
        round_to_backing_scale(frame.height, backing_scale),
    )
}
fn clamp_dimension_to_visible_bounds(size: f64, visible_size: f64, edge_margin: f64) -> f64 {
    let max_size = (visible_size - (edge_margin * 2.0)).max(1.0);
    size.min(max_size)
}
fn clamp_axis_to_visible_bounds(
    origin: f64,
    size: f64,
    visible_origin: f64,
    visible_size: f64,
    edge_margin: f64,
) -> f64 {
    let min_origin = visible_origin + edge_margin;
    let max_origin = visible_origin + visible_size - size - edge_margin;

    if min_origin <= max_origin {
        origin.clamp(min_origin, max_origin)
    } else {
        min_origin
    }
}
fn clamp_frame_to_visible_bounds(
    frame: FrameGeometry,
    visible_bounds: FrameGeometry,
    edge_margin: f64,
) -> FrameGeometry {
    let edge_margin = edge_margin.max(0.0);
    let width = clamp_dimension_to_visible_bounds(frame.width, visible_bounds.width, edge_margin);
    let height =
        clamp_dimension_to_visible_bounds(frame.height, visible_bounds.height, edge_margin);
    let x = clamp_axis_to_visible_bounds(
        frame.x,
        width,
        visible_bounds.x,
        visible_bounds.width,
        edge_margin,
    );
    let y = clamp_axis_to_visible_bounds(
        frame.y,
        height,
        visible_bounds.y,
        visible_bounds.height,
        edge_margin,
    );
    FrameGeometry::new(x, y, width, height)
}
fn calculate_resized_frame(
    current_frame: FrameGeometry,
    target_height: f64,
    visible_bounds: Option<FrameGeometry>,
    backing_scale: Option<f64>,
) -> FrameGeometry {
    let height_delta = target_height - current_frame.height;
    let new_origin_y = current_frame.y - height_delta;
    let mut resized = FrameGeometry::new(
        current_frame.x,
        new_origin_y,
        current_frame.width,
        target_height,
    );

    if let Some(backing_scale) = sanitize_backing_scale(backing_scale) {
        resized = normalize_frame_to_backing_scale(resized, backing_scale);
    }

    if let Some(visible_bounds) = visible_bounds {
        resized = clamp_frame_to_visible_bounds(
            resized,
            visible_bounds,
            crate::panel::WINDOW_VISIBLE_EDGE_MARGIN,
        );
    }

    if let Some(backing_scale) = sanitize_backing_scale(backing_scale) {
        resized = normalize_frame_to_backing_scale(resized, backing_scale);
    }

    resized
}
fn should_apply_resize(current_height: f64, target_height: f64) -> bool {
    (current_height - target_height).abs() >= RESIZE_MIN_DELTA_PX
}
#[cfg(target_os = "macos")]
fn ns_rect_contains_point(rect: NSRect, x: f64, y: f64) -> bool {
    x >= rect.origin.x
        && x < rect.origin.x + rect.size.width
        && y >= rect.origin.y
        && y < rect.origin.y + rect.size.height
}
#[cfg(target_os = "macos")]
unsafe fn screen_geometry_from_screen(screen: id) -> Option<ScreenGeometry> {
    if screen == nil {
        return None;
    }

    let visible_frame: NSRect = msg_send![screen, visibleFrame];
    let backing_scale: f64 = msg_send![screen, backingScaleFactor];
    Some(ScreenGeometry {
        visible_bounds: FrameGeometry::from_ns_rect(visible_frame),
        backing_scale,
    })
}
#[cfg(target_os = "macos")]
unsafe fn screen_geometry_for_window_frame(
    window: id,
    frame: FrameGeometry,
) -> Option<ScreenGeometry> {
    let center_x = frame.x + (frame.width / 2.0);
    let center_y = frame.y + (frame.height / 2.0);

    let screens: id = msg_send![class!(NSScreen), screens];
    if screens != nil {
        let count: usize = msg_send![screens, count];
        for index in 0..count {
            let screen: id = msg_send![screens, objectAtIndex:index];
            if screen == nil {
                continue;
            }

            let screen_frame: NSRect = msg_send![screen, frame];
            if ns_rect_contains_point(screen_frame, center_x, center_y) {
                if let Some(geometry) = screen_geometry_from_screen(screen) {
                    return Some(geometry);
                }
            }
        }
    }

    let window_screen: id = msg_send![window, screen];
    if let Some(geometry) = screen_geometry_from_screen(window_screen) {
        return Some(geometry);
    }

    let main_screen: id = msg_send![class!(NSScreen), mainScreen];
    screen_geometry_from_screen(main_screen)
}
/// Layout constants for height calculations
pub mod layout {
    use crate::panel::{CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y, HEADER_PADDING_Y};
    use gpui::{px, Pixels};

    /// List container vertical padding (top + bottom, matches default padding_xs)
    pub const ARG_LIST_PADDING_Y: f32 = 8.0;
    /// Divider thickness (matches default design border_thin)
    pub const ARG_DIVIDER_HEIGHT: f32 = 1.0;
    /// Input row text height (cursor height + margins)
    pub const ARG_INPUT_LINE_HEIGHT: f32 = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);
    /// Footer height (matches PromptFooter)
    pub const FOOTER_HEIGHT: f32 = 30.0;
    /// Total input-only height (header only, no list, but with footer)
    /// Uses HEADER_PADDING_Y from panel.rs for accurate height calculation
    pub const ARG_HEADER_HEIGHT: f32 =
        (HEADER_PADDING_Y * 2.0) + ARG_INPUT_LINE_HEIGHT + FOOTER_HEIGHT;

    /// Minimum window height (input only) - for input-only prompts
    pub const MIN_HEIGHT: Pixels = px(ARG_HEADER_HEIGHT);

    /// Standard height for views with preview panel (script list, arg with choices)
    /// This is FIXED - these views do NOT resize dynamically
    pub const STANDARD_HEIGHT: Pixels = px(crate::config::defaults::DEFAULT_LAYOUT_STANDARD_HEIGHT);

    /// Maximum window height for full-content views (editor, div, term)
    pub const MAX_HEIGHT: Pixels = px(crate::config::defaults::DEFAULT_LAYOUT_MAX_HEIGHT);
}
fn sanitize_dimension(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}
fn sanitize_layout_config(layout: LayoutConfig) -> LayoutConfig {
    let defaults = LayoutConfig::default();
    let min_height = f32::from(layout::MIN_HEIGHT);

    let standard_height =
        sanitize_dimension(layout.standard_height, defaults.standard_height).max(min_height);
    let max_height =
        sanitize_dimension(layout.max_height, defaults.max_height).max(standard_height);

    LayoutConfig {
        standard_height,
        max_height,
    }
}
fn runtime_layout_config() -> LayoutConfig {
    static LAYOUT_CONFIG_CACHE: OnceLock<LayoutConfig> = OnceLock::new();

    LAYOUT_CONFIG_CACHE
        .get_or_init(|| sanitize_layout_config(config::load_user_preferences().layout))
        .clone()
}
fn height_for_view_with_layout(
    view_type: ViewType,
    item_count: usize,
    layout_config: &LayoutConfig,
) -> Pixels {
    use layout::*;

    let standard_height = px(layout_config.standard_height);
    let max_height = px(layout_config.max_height);

    let clamp_height = |height: Pixels| -> Pixels {
        let height_f = f32::from(height);
        let min_f = f32::from(MIN_HEIGHT);
        let max_f = f32::from(standard_height);
        px(height_f.clamp(min_f, max_f))
    };

    match view_type {
        // Views with preview panel - FIXED height, no dynamic resizing
        // DivPrompt also uses standard height to match main window
        ViewType::ScriptList | ViewType::DivPrompt => standard_height,
        ViewType::ArgPromptWithChoices => {
            let visible_items = item_count.max(1) as f32;
            let list_height =
                (visible_items * LIST_ITEM_HEIGHT) + ARG_LIST_PADDING_Y + ARG_DIVIDER_HEIGHT;
            let total_height = ARG_HEADER_HEIGHT + list_height;
            clamp_height(px(total_height))
        }
        // Input-only prompt - compact
        ViewType::ArgPromptNoChoices => MIN_HEIGHT,
        // Full content views (editor, terminal) - max height
        ViewType::EditorPrompt | ViewType::TermPrompt => max_height,
    }
}
fn initial_window_height_with_layout(layout_config: &LayoutConfig) -> Pixels {
    px(layout_config.standard_height)
}
/// View types for height calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    /// Script list view (main launcher) - has preview panel, FIXED height
    ScriptList,
    /// Arg prompt with choices - dynamic height based on item count
    ArgPromptWithChoices,
    /// Arg prompt without choices (input only) - compact height
    ArgPromptNoChoices,
    /// Div prompt (HTML display) - full height
    DivPrompt,
    /// Editor prompt (code editor) - full height
    EditorPrompt,
    /// Terminal prompt - full height
    TermPrompt,
}
/// Get the target height for a specific view type
///
/// # Arguments
/// * `view_type` - The type of view being displayed
/// * `item_count` - Number of items in the current view (used for dynamic sizing)
///
/// # Returns
/// The window height for this view type
pub fn height_for_view(view_type: ViewType, item_count: usize) -> Pixels {
    let layout_config = runtime_layout_config();
    height_for_view_with_layout(view_type, item_count, &layout_config)
}
/// Calculate the initial window height for app startup
pub fn initial_window_height() -> Pixels {
    let layout_config = runtime_layout_config();
    initial_window_height_with_layout(&layout_config)
}
/// Defer a window resize to the end of the current effect cycle.
///
/// This version uses `Window::defer()` for coalesced, deferred execution.
/// Use when you have direct Window access (e.g., in window update closures, hotkey handlers).
///
/// # Arguments
/// * `view_type` - The type of view to resize for
/// * `item_count` - Item count (used for some view types)
/// * `window` - The GPUI Window reference
/// * `cx` - The GPUI App context
///
pub fn defer_resize_to_view(
    view_type: ViewType,
    item_count: usize,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) {
    let target_height = height_for_view(view_type, item_count);
    crate::window_ops::queue_resize(f32::from(target_height), window, cx);
}
/// Resize window synchronously based on view type.
///
/// Use this version when you only have ViewContext access (e.g., in prompt message handlers
/// running from async tasks via `cx.spawn`). These handlers run outside the render cycle,
/// so direct resize is safe and won't cause RefCell borrow conflicts.
///
/// # Arguments
/// * `view_type` - The type of view to resize for
/// * `item_count` - Item count (used for some view types)
///
/// # Example
/// ```rust,ignore
/// // In handle_prompt_message or similar ViewContext methods:
/// resize_to_view_sync(ViewType::ArgPromptWithChoices, choices.len());
/// ```
pub fn resize_to_view_sync(view_type: ViewType, item_count: usize) {
    let target_height = height_for_view(view_type, item_count);
    resize_first_window_to_height(target_height);
}
/// Force reset the debounce timer (kept for API compatibility)
pub fn reset_resize_debounce() {
    // No-op - we removed debouncing since resizes are now rare
}
/// Resize the main window to a new height, keeping the top edge fixed.
///
/// # Arguments
/// * `target_height` - The desired window height in pixels
///
/// # Platform
/// This function only works on macOS. On other platforms, it's a no-op.
#[cfg(target_os = "macos")]
pub fn resize_first_window_to_height(target_height: Pixels) {
    let height_f64: f64 = f32::from(target_height) as f64;

    // Get the main window from WindowManager
    let window = match window_manager::get_main_window() {
        Some(w) => w,
        None => {
            warn!("Main window not registered in WindowManager, cannot resize");
            logging::log(
                "RESIZE",
                "WARNING: Main window not registered in WindowManager.",
            );
            return;
        }
    };

    unsafe {
        // Get current window frame
        let current_frame: NSRect = msg_send![window, frame];

        // Skip if height is already correct (within 1px tolerance)
        let current_height = current_frame.size.height;
        if !should_apply_resize(current_height, height_f64) {
            return;
        }

        let correlation_id = format!("resize:{}", uuid::Uuid::new_v4());

        // Log actual resizes at debug level (these are rare events, not hot-path).
        debug!(
            from_height = current_height,
            to_height = height_f64,
            correlation_id = %correlation_id,
            "Resizing window instantly"
        );
        logging::log(
            "RESIZE",
            &format!(
                "[RESIZE_START] correlation_id={} from={:.0} to={:.0} animate={}",
                correlation_id, current_height, height_f64, WINDOW_RESIZE_ANIMATE
            ),
        );

        let current_geometry = FrameGeometry::from_ns_rect(current_frame);
        let screen_geometry = screen_geometry_for_window_frame(window, current_geometry);
        let new_frame = calculate_resized_frame(
            current_geometry,
            height_f64,
            screen_geometry.map(|geometry| geometry.visible_bounds),
            screen_geometry.map(|geometry| geometry.backing_scale),
        )
        .to_ns_rect();

        // Apply the new frame instantly to avoid any native resize animation.
        let _: () = msg_send![
            window,
            setFrame:new_frame
            display:true
            animate:WINDOW_RESIZE_ANIMATE
        ];

        logging::log(
            "RESIZE",
            &format!(
                "[RESIZE_END] correlation_id={} applied_height={:.0}",
                correlation_id, height_f64
            ),
        );
    }
}
/// Get the current height of the main window
#[allow(dead_code)]
#[cfg(target_os = "macos")]
pub fn get_first_window_height() -> Option<Pixels> {
    let window = window_manager::get_main_window()?;

    unsafe {
        let frame: NSRect = msg_send![window, frame];
        Some(px(frame.size.height as f32))
    }
}
/// Non-macOS stub for resize function
#[cfg(not(target_os = "macos"))]
pub fn resize_first_window_to_height(_target_height: Pixels) {
    logging::log("RESIZE", "Window resize is only supported on macOS");
}
/// Non-macOS stub for get_first_window_height
#[allow(dead_code)]
#[cfg(not(target_os = "macos"))]
pub fn get_first_window_height() -> Option<Pixels> {
    None
}

// --- merged from part_001.rs ---
#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    fn default_layout() -> LayoutConfig {
        LayoutConfig::default()
    }

    #[test]
    fn test_script_list_fixed_height() {
        let layout = default_layout();

        // Script list should always be STANDARD_HEIGHT regardless of item count
        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 0, &layout),
            layout::STANDARD_HEIGHT
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 5, &layout),
            layout::STANDARD_HEIGHT
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 100, &layout),
            layout::STANDARD_HEIGHT
        );
    }

    #[test]
    fn test_arg_with_choices_dynamic_height() {
        let layout = default_layout();

        // Arg with choices should size to items, clamped to STANDARD_HEIGHT
        let base_height =
            layout::ARG_HEADER_HEIGHT + layout::ARG_DIVIDER_HEIGHT + layout::ARG_LIST_PADDING_Y;
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptWithChoices, 1, &layout),
            px(base_height + LIST_ITEM_HEIGHT)
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptWithChoices, 2, &layout),
            px(base_height + (2.0 * LIST_ITEM_HEIGHT))
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptWithChoices, 100, &layout),
            layout::STANDARD_HEIGHT
        );
    }

    #[test]
    fn test_arg_no_choices_compact() {
        let layout = default_layout();

        // Arg without choices should be MIN_HEIGHT
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptNoChoices, 0, &layout),
            layout::MIN_HEIGHT
        );
    }

    #[test]
    fn test_full_height_views() {
        let layout = default_layout();

        // Editor and Terminal use MAX_HEIGHT (700px)
        assert_eq!(
            height_for_view_with_layout(ViewType::EditorPrompt, 0, &layout),
            layout::MAX_HEIGHT
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::TermPrompt, 0, &layout),
            layout::MAX_HEIGHT
        );
    }

    #[test]
    fn test_div_prompt_standard_height() {
        let layout = default_layout();

        // DivPrompt uses STANDARD_HEIGHT (500px) to match main window
        assert_eq!(
            height_for_view_with_layout(ViewType::DivPrompt, 0, &layout),
            layout::STANDARD_HEIGHT
        );
    }

    #[test]
    fn test_initial_window_height() {
        let layout = default_layout();
        assert_eq!(
            initial_window_height_with_layout(&layout),
            layout::STANDARD_HEIGHT
        );
        assert_eq!(
            initial_window_height(),
            height_for_view(ViewType::ScriptList, 0)
        );
    }

    #[test]
    fn test_height_constants() {
        assert_eq!(layout::MIN_HEIGHT, px(layout::ARG_HEADER_HEIGHT));
        assert_eq!(layout::STANDARD_HEIGHT, px(500.0));
        assert_eq!(layout::MAX_HEIGHT, px(700.0));
    }

    #[test]
    fn test_layout_uses_configured_standard_and_max_height() {
        let custom_layout = LayoutConfig {
            standard_height: 540.0,
            max_height: 860.0,
        };

        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 0, &custom_layout),
            px(540.0)
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::EditorPrompt, 0, &custom_layout),
            px(860.0)
        );
        assert_eq!(initial_window_height_with_layout(&custom_layout), px(540.0));
    }

    #[test]
    fn test_sanitize_layout_config_enforces_bounds() {
        let sanitized = sanitize_layout_config(LayoutConfig {
            standard_height: 10.0,
            max_height: 5.0,
        });

        assert_eq!(sanitized.standard_height, f32::from(layout::MIN_HEIGHT));
        assert_eq!(sanitized.max_height, f32::from(layout::MIN_HEIGHT));
    }

    #[test]
    fn test_calculate_resized_frame_keeps_top_edge_fixed() {
        let current = FrameGeometry::new(100.0, 200.0, 750.0, 500.0);
        let resized = calculate_resized_frame(current, 700.0, None, None);

        assert!((resized.y - 0.0).abs() < 0.001);
        assert!((resized.height - 700.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_clamps_bottom_to_visible_bounds() {
        let current = FrameGeometry::new(100.0, 200.0, 750.0, 500.0);
        let visible = FrameGeometry::new(0.0, 50.0, 1920.0, 800.0);
        let resized = calculate_resized_frame(current, 700.0, Some(visible), None);

        assert!((resized.y - 54.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_caps_height_to_visible_bounds() {
        let current = FrameGeometry::new(100.0, 300.0, 750.0, 400.0);
        let visible = FrameGeometry::new(0.0, 0.0, 1920.0, 700.0);
        let resized = calculate_resized_frame(current, 900.0, Some(visible), None);

        assert!((resized.height - 692.0).abs() < 0.001);
        assert!((resized.y - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_rounds_to_backing_scale() {
        let current = FrameGeometry::new(10.1, 20.2, 749.7, 500.3);
        let resized = calculate_resized_frame(current, 700.7, None, Some(2.0));

        assert!((resized.x - 10.0).abs() < 0.001);
        assert!((resized.y - -180.0).abs() < 0.001);
        assert!((resized.width - 749.5).abs() < 0.001);
        assert!((resized.height - 700.5).abs() < 0.001);
    }

    #[test]
    fn test_should_apply_resize_true_when_height_changes() {
        assert!(should_apply_resize(500.0, 700.0));
    }

    #[test]
    fn test_should_apply_resize_false_when_height_is_effectively_unchanged() {
        assert!(!should_apply_resize(500.0, 500.4));
    }

    #[test]
    fn test_window_resize_animation_flag_is_disabled() {
        assert!(
            !WINDOW_RESIZE_ANIMATE,
            "Window resize must stay instant with animation disabled"
        );
    }
}
