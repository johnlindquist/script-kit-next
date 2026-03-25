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
use tracing::{debug, info, warn};
const RESIZE_MIN_DELTA_PX: f64 = 1.0;
const WINDOW_RESIZE_ANIMATE: bool = false;
const MINI_MAIN_WINDOW_MIN_HEIGHT: f32 = 220.0;
const MINI_MAIN_WINDOW_MAX_HEIGHT: f32 = 420.0;
const MINI_MAIN_WINDOW_HEADER_HEIGHT: f32 = 56.0;
const MINI_MAIN_WINDOW_HINT_STRIP_HEIGHT: f32 = 30.0;
const MINI_MAIN_WINDOW_DIVIDER_HEIGHT: f32 = 1.0;
const MINI_MAIN_WINDOW_SECTION_HEADER_HEIGHT: f32 = 32.0;
pub(crate) const MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS: usize = 8;

/// Available pixel budget for list content in the mini main window.
///
/// Subtracts the fixed chrome (header + divider + hint strip) from `MAX_HEIGHT`.
#[allow(dead_code)] // Called from include!()-ed code in app_impl/ui_window.rs
pub(crate) fn mini_main_window_list_budget_height() -> f32 {
    MINI_MAIN_WINDOW_MAX_HEIGHT
        - MINI_MAIN_WINDOW_HEADER_HEIGHT
        - MINI_MAIN_WINDOW_DIVIDER_HEIGHT
        - MINI_MAIN_WINDOW_HINT_STRIP_HEIGHT
}

/// Maximum number of selectable rows that can fit without clipping, given
/// `visible_section_headers` section headers that each consume
/// `MINI_MAIN_WINDOW_SECTION_HEADER_HEIGHT` pixels of the list budget.
#[allow(dead_code)] // Called from include!()-ed code in app_impl/ui_window.rs
pub(crate) fn capped_mini_main_window_selectable_rows(visible_section_headers: usize) -> usize {
    let remaining_list_height = mini_main_window_list_budget_height()
        - (visible_section_headers as f32 * MINI_MAIN_WINDOW_SECTION_HEADER_HEIGHT);

    if remaining_list_height <= 0.0 {
        0
    } else {
        ((remaining_list_height / LIST_ITEM_HEIGHT).floor() as usize)
            .min(MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS)
    }
}

/// Shared layout constants for the mini main window render branch.
/// Both resize logic and render code consume these so the geometry contract stays in sync.
/// Constants are consumed from the binary target via `include!()` render code.
#[allow(dead_code)]
pub(crate) mod mini_layout {
    /// Horizontal padding for the mini header area.
    pub const HEADER_PADDING_X: f32 = 12.0;
    /// Vertical padding for the mini header area.
    pub const HEADER_PADDING_Y: f32 = 10.0;
    /// Horizontal padding for the mini hint strip footer.
    pub const HINT_STRIP_PADDING_X: f32 = 14.0;
    /// Vertical padding for the mini hint strip footer.
    pub const HINT_STRIP_PADDING_Y: f32 = 8.0;
    /// Height of the hint strip area (matches resize contract).
    pub const HINT_STRIP_HEIGHT: f32 = super::MINI_MAIN_WINDOW_HINT_STRIP_HEIGHT;
    /// Height of the divider between header and list content.
    pub const DIVIDER_HEIGHT: f32 = super::MINI_MAIN_WINDOW_DIVIDER_HEIGHT;
    /// Opacity for hint strip shortcut text (uses OPACITY_TEXT_MUTED from theme/opacity).
    pub const HINT_TEXT_OPACITY: f32 = crate::theme::opacity::OPACITY_TEXT_MUTED;
}
/// Build a content-aware `MiniMainWindowSizing` from grouped items.
///
/// Walks the grouped items list, counting selectable items and section headers
/// visible in the first page.  Uses the header-aware row cap so that section
/// headers explicitly reduce the available selectable-row capacity instead of
/// silently pushing the window height into the max-clamp.
#[allow(dead_code)] // Called from include!()-ed code in app_impl/ui_window.rs
pub(crate) fn mini_main_window_sizing_from_grouped_items(
    grouped_items: &[crate::list_item::GroupedListItem],
) -> MiniMainWindowSizing {
    use crate::list_item::GroupedListItem;

    let mut selectable_items = 0usize;
    let mut visible_section_headers = 0usize;

    for item in grouped_items {
        let selectable_cap = capped_mini_main_window_selectable_rows(visible_section_headers);

        if selectable_items >= selectable_cap {
            break;
        }

        match item {
            GroupedListItem::SectionHeader(..) => {
                visible_section_headers += 1;
            }
            GroupedListItem::Item(_) => {
                selectable_items += 1;
            }
        }
    }

    MiniMainWindowSizing {
        selectable_items,
        visible_section_headers,
        is_empty: grouped_items.is_empty(),
    }
}

/// Content-aware sizing input for the mini main window.
///
/// Instead of passing a flat `item_count` (which conflates section headers with
/// selectable items), callers build this struct so the height formula can account
/// for the different row heights of headers vs items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct MiniMainWindowSizing {
    /// Number of selectable items visible in the first page (capped at MAX_VISIBLE_ROWS).
    pub selectable_items: usize,
    /// Number of section headers visible in the first page.
    pub visible_section_headers: usize,
    /// True when the grouped items list is completely empty.
    pub is_empty: bool,
}

/// Reason for a mini main window resize — used for structured telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants used from include!()-ed code in app_impl/ui_window.rs
pub(crate) enum MiniResizeReason {
    /// Filter text changed → grouped results changed → resize needed.
    FilterChanged,
    /// Grouped results changed (data refresh, view transition, etc.).
    GroupedResultsChanged,
    /// Entering mini window mode.
    ViewModeEntered,
    /// Flat item-count fallback was used instead of content-aware sizing.
    FlatFallback,
}

/// Emit a structured sizing receipt for every mini main window resize.
///
/// All mini resize paths must call this so that every height decision is
/// machine-parseable under the `MINI_WINDOW` tracing target.
pub(crate) fn log_mini_window_sizing(
    reason: MiniResizeReason,
    sizing: MiniMainWindowSizing,
    target_height_px: f32,
) {
    info!(
        target: "MINI_WINDOW",
        ?reason,
        selectable_items = sizing.selectable_items,
        visible_section_headers = sizing.visible_section_headers,
        is_empty = sizing.is_empty,
        target_height_px,
        min_height_px = MINI_MAIN_WINDOW_MIN_HEIGHT,
        max_height_px = MINI_MAIN_WINDOW_MAX_HEIGHT,
        header_height_px = MINI_MAIN_WINDOW_HEADER_HEIGHT,
        divider_height_px = MINI_MAIN_WINDOW_DIVIDER_HEIGHT,
        hint_strip_height_px = MINI_MAIN_WINDOW_HINT_STRIP_HEIGHT,
        section_header_height_px = MINI_MAIN_WINDOW_SECTION_HEADER_HEIGHT,
        "mini window sizing receipt"
    );
}

/// Calculate the target height for the mini main window given content-aware sizing.
///
/// Formula: header + divider + hint_strip + list_content, clamped to [MIN, MAX].
/// List content = selectable_items * LIST_ITEM_HEIGHT + visible_section_headers * SECTION_HEADER_HEIGHT.
pub(crate) fn height_for_mini_main_window(sizing: MiniMainWindowSizing) -> Pixels {
    let list_height = if sizing.is_empty {
        0.0
    } else {
        (sizing.selectable_items as f32 * LIST_ITEM_HEIGHT)
            + (sizing.visible_section_headers as f32 * MINI_MAIN_WINDOW_SECTION_HEADER_HEIGHT)
    };

    let total_height = MINI_MAIN_WINDOW_HEADER_HEIGHT
        + MINI_MAIN_WINDOW_DIVIDER_HEIGHT
        + MINI_MAIN_WINDOW_HINT_STRIP_HEIGHT
        + list_height;

    px(total_height.clamp(MINI_MAIN_WINDOW_MIN_HEIGHT, MINI_MAIN_WINDOW_MAX_HEIGHT))
}

/// Defer a mini main window resize to the end of the current effect cycle.
#[allow(dead_code)] // Called from include!()-ed code in app_impl/ui_window.rs
pub(crate) fn defer_resize_to_mini_main_window(
    sizing: MiniMainWindowSizing,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) {
    let target_height = height_for_mini_main_window(sizing);
    crate::window_ops::queue_resize_with_width(
        f32::from(target_height),
        Some(MINI_MAIN_WINDOW_WIDTH),
        window,
        cx,
    );
}

/// Resize the mini main window synchronously.
#[allow(dead_code)] // Called from include!()-ed code in app_impl/ui_window.rs
pub(crate) fn resize_to_mini_main_window_sync(sizing: MiniMainWindowSizing) {
    let target_height = height_for_mini_main_window(sizing);
    resize_first_window_to_size(target_height, Some(MINI_MAIN_WINDOW_WIDTH));
}

/// Width for mini main window (compact launcher)
const MINI_MAIN_WINDOW_WIDTH: f32 = 480.0;
/// Width for full main window (standard launcher)
const FULL_MAIN_WINDOW_WIDTH: f32 = 750.0;
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
    calculate_resized_frame_with_width(
        current_frame,
        target_height,
        None,
        visible_bounds,
        backing_scale,
    )
}
fn calculate_resized_frame_with_width(
    current_frame: FrameGeometry,
    target_height: f64,
    target_width: Option<f64>,
    visible_bounds: Option<FrameGeometry>,
    backing_scale: Option<f64>,
) -> FrameGeometry {
    let height_delta = target_height - current_frame.height;
    let new_origin_y = current_frame.y - height_delta;
    let new_width = target_width.unwrap_or(current_frame.width);
    // Center horizontally when width changes
    let new_origin_x = if target_width.is_some() {
        let width_delta = new_width - current_frame.width;
        current_frame.x - (width_delta / 2.0)
    } else {
        current_frame.x
    };
    let mut resized = FrameGeometry::new(new_origin_x, new_origin_y, new_width, target_height);

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
        ViewType::MiniMainWindow => {
            // Flat item_count fallback: assumes all items are selectable (no section headers).
            // Prefer height_for_mini_main_window(MiniMainWindowSizing) for content-aware sizing.
            let visible_items = item_count.min(MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS);
            let sizing = MiniMainWindowSizing {
                selectable_items: visible_items,
                visible_section_headers: 0,
                is_empty: item_count == 0,
            };
            let height = height_for_mini_main_window(sizing);
            warn!(
                target: "MINI_WINDOW",
                item_count,
                "flat mini sizing fallback used; content-aware sizing should be preferred"
            );
            log_mini_window_sizing(MiniResizeReason::FlatFallback, sizing, f32::from(height));
            height
        }
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
    /// Script list in compact main-window mode - dynamic height based on item count
    MiniMainWindow,
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
/// Get the target width for a specific view type, if it differs from current.
///
/// Returns `Some(width)` when the view needs a specific width (e.g. mini mode),
/// or `None` when the current width should be preserved.
pub fn width_for_view(view_type: ViewType) -> Option<f32> {
    match view_type {
        ViewType::MiniMainWindow => Some(MINI_MAIN_WINDOW_WIDTH),
        // When leaving mini mode, restore full width
        ViewType::ScriptList => Some(FULL_MAIN_WINDOW_WIDTH),
        _ => None,
    }
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
    let target_width = width_for_view(view_type);
    crate::window_ops::queue_resize_with_width(f32::from(target_height), target_width, window, cx);
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
    let target_width = width_for_view(view_type);
    if matches!(view_type, ViewType::MiniMainWindow) {
        let visible_rows = item_count.clamp(4, MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS);
        debug!(
            view_type = ?view_type,
            width_px = target_width.unwrap_or(0.0),
            height_px = f32::from(target_height),
            item_count = item_count,
            visible_row_count = visible_rows,
            "mini_main_window sizing selected"
        );
    }
    if target_width.is_some() {
        resize_first_window_to_size(target_height, target_width);
    } else {
        resize_first_window_to_height(target_height);
    }
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
/// Resize the main window to a new height and optionally a new width, keeping the top edge fixed.
/// When width changes, the window is re-centered horizontally around its midpoint.
#[cfg(target_os = "macos")]
pub fn resize_first_window_to_size(target_height: Pixels, target_width: Option<f32>) {
    let height_f64: f64 = f32::from(target_height) as f64;
    let width_f64: Option<f64> = target_width.map(|w| w as f64);

    let window = match window_manager::get_main_window() {
        Some(w) => w,
        None => {
            warn!("Main window not registered in WindowManager, cannot resize");
            return;
        }
    };

    // SAFETY: NSWindow frame/setFrame are standard AppKit APIs called on the main thread.
    // The window pointer is validated as non-nil by get_main_window().
    unsafe {
        let current_frame: NSRect = msg_send![window, frame];
        let current_height = current_frame.size.height;
        let current_width = current_frame.size.width;

        let height_changed = should_apply_resize(current_height, height_f64);
        let width_changed = width_f64
            .map(|w| (current_width - w).abs() >= RESIZE_MIN_DELTA_PX)
            .unwrap_or(false);

        if !height_changed && !width_changed {
            return;
        }

        let correlation_id = format!("resize:{}", uuid::Uuid::new_v4());

        debug!(
            from_height = current_height,
            to_height = height_f64,
            from_width = current_width,
            to_width = ?width_f64,
            correlation_id = %correlation_id,
            "Resizing window (height+width)"
        );

        let current_geometry = FrameGeometry::from_ns_rect(current_frame);
        let screen_geometry = screen_geometry_for_window_frame(window, current_geometry);
        let new_frame = calculate_resized_frame_with_width(
            current_geometry,
            height_f64,
            width_f64,
            screen_geometry.map(|g| g.visible_bounds),
            screen_geometry.map(|g| g.backing_scale),
        )
        .to_ns_rect();

        // SAFETY: setFrame:display:animate: is a standard NSWindow method.
        let _: () = msg_send![
            window,
            setFrame:new_frame
            display:true
            animate:WINDOW_RESIZE_ANIMATE
        ];

        logging::log(
            "RESIZE",
            &format!(
                "[RESIZE_SIZE_END] correlation_id={} height={:.0} width={:.0}",
                correlation_id,
                height_f64,
                width_f64.unwrap_or(current_width)
            ),
        );
    }
}
/// Non-macOS stub for resize_first_window_to_size
#[cfg(not(target_os = "macos"))]
pub fn resize_first_window_to_size(_target_height: Pixels, _target_width: Option<f32>) {
    logging::log("RESIZE", "Window resize is only supported on macOS");
}

// --- merged from part_001.rs ---
#[cfg(test)]
mod tests;

#[cfg(test)]
mod resize_tests {
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
    fn test_mini_main_window_dynamic_height() {
        let layout = default_layout();
        // Content-aware formula: header(56) + divider(1) + hint_strip(30) + list_content
        // 0 items: empty → clamped to MIN_HEIGHT (220)
        assert_eq!(
            height_for_view_with_layout(ViewType::MiniMainWindow, 0, &layout),
            px(MINI_MAIN_WINDOW_MIN_HEIGHT)
        );
        // 4 items: 56 + 1 + 30 + 4*40 = 247
        assert_eq!(
            height_for_view_with_layout(ViewType::MiniMainWindow, 4, &layout),
            px(247.0)
        );
        // 8 items: 56 + 1 + 30 + 8*40 = 407
        assert_eq!(
            height_for_view_with_layout(ViewType::MiniMainWindow, 8, &layout),
            px(407.0)
        );
        // 100 items: capped at 8 visible → same as 8 items = 407
        assert_eq!(
            height_for_view_with_layout(ViewType::MiniMainWindow, 100, &layout),
            px(407.0)
        );
    }

    #[test]
    fn test_mini_height_content_aware_with_section_headers() {
        // 3 items, no headers: 56 + 1 + 30 + 3*40 = 207 → clamped to MIN 220
        assert_eq!(
            f32::from(height_for_mini_main_window(MiniMainWindowSizing {
                selectable_items: 3,
                visible_section_headers: 0,
                is_empty: false,
            })),
            MINI_MAIN_WINDOW_MIN_HEIGHT
        );

        // 6 items + 1 section header: 56 + 1 + 30 + 6*40 + 1*32 = 359
        assert_eq!(
            f32::from(height_for_mini_main_window(MiniMainWindowSizing {
                selectable_items: 6,
                visible_section_headers: 1,
                is_empty: false,
            })),
            359.0
        );

        // 8 items + 2 section headers: 56 + 1 + 30 + 8*40 + 2*32 = 471 → clamped to MAX 420
        assert_eq!(
            f32::from(height_for_mini_main_window(MiniMainWindowSizing {
                selectable_items: 8,
                visible_section_headers: 2,
                is_empty: false,
            })),
            MINI_MAIN_WINDOW_MAX_HEIGHT
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
        let flag = WINDOW_RESIZE_ANIMATE;
        assert!(
            !flag,
            "Window resize must stay instant with animation disabled"
        );
    }

    #[test]
    fn test_width_for_view_mini_main_window() {
        assert_eq!(
            width_for_view(ViewType::MiniMainWindow),
            Some(MINI_MAIN_WINDOW_WIDTH)
        );
    }

    #[test]
    fn test_width_for_view_script_list_restores_full() {
        assert_eq!(
            width_for_view(ViewType::ScriptList),
            Some(FULL_MAIN_WINDOW_WIDTH)
        );
    }

    #[test]
    fn test_width_for_view_other_types_none() {
        assert_eq!(width_for_view(ViewType::ArgPromptWithChoices), None);
        assert_eq!(width_for_view(ViewType::ArgPromptNoChoices), None);
        assert_eq!(width_for_view(ViewType::DivPrompt), None);
        assert_eq!(width_for_view(ViewType::EditorPrompt), None);
        assert_eq!(width_for_view(ViewType::TermPrompt), None);
    }

    #[test]
    fn test_mini_main_window_width_constant() {
        assert_eq!(MINI_MAIN_WINDOW_WIDTH, 480.0);
        assert_eq!(FULL_MAIN_WINDOW_WIDTH, 750.0);
    }

    #[test]
    fn test_calculate_resized_frame_with_width_centers_horizontally() {
        let current = FrameGeometry::new(100.0, 200.0, 750.0, 500.0);
        let resized = calculate_resized_frame_with_width(current, 400.0, Some(480.0), None, None);

        // Width should be 480
        assert!((resized.width - 480.0).abs() < 0.001);
        // Height should be 400
        assert!((resized.height - 400.0).abs() < 0.001);
        // X should be centered: 100 - (480 - 750) / 2 = 100 + 135 = 235
        assert!((resized.x - 235.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_with_width_none_preserves_width() {
        let current = FrameGeometry::new(100.0, 200.0, 750.0, 500.0);
        let resized = calculate_resized_frame_with_width(current, 400.0, None, None, None);

        // Width should be preserved
        assert!((resized.width - 750.0).abs() < 0.001);
        // X should be preserved
        assert!((resized.x - 100.0).abs() < 0.001);
    }
}

#[cfg(test)]
mod mini_main_window_layout_tests {
    use super::{
        capped_mini_main_window_selectable_rows, height_for_mini_main_window, MiniMainWindowSizing,
        MINI_MAIN_WINDOW_MAX_HEIGHT, MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS,
    };

    #[test]
    fn capped_rows_account_for_section_headers() {
        assert_eq!(
            capped_mini_main_window_selectable_rows(0),
            MINI_MAIN_WINDOW_MAX_VISIBLE_ROWS
        );
        assert_eq!(capped_mini_main_window_selectable_rows(1), 7);
        assert_eq!(capped_mini_main_window_selectable_rows(2), 6);
    }

    #[test]
    fn two_headers_and_capped_rows_stay_below_max_height() {
        let height = height_for_mini_main_window(MiniMainWindowSizing {
            selectable_items: capped_mini_main_window_selectable_rows(2),
            visible_section_headers: 2,
            is_empty: false,
        });

        assert_eq!(f32::from(height), 391.0);
    }

    #[test]
    fn uncapped_two_headers_and_eight_rows_would_hit_max_clamp() {
        let height = height_for_mini_main_window(MiniMainWindowSizing {
            selectable_items: 8,
            visible_section_headers: 2,
            is_empty: false,
        });

        assert_eq!(f32::from(height), MINI_MAIN_WINDOW_MAX_HEIGHT);
    }

    // --- Source-audit regression tests ---
    // These verify that structured tracing targets exist so that agentic
    // verification can rely on machine-parseable log lines.

    #[test]
    fn source_audit_mini_resize_receipt_log_exists() {
        let source = std::fs::read_to_string("src/window_resize/mod.rs")
            .expect("should read window_resize/mod.rs");
        assert!(
            source.contains("target: \"MINI_WINDOW\""),
            "mini resize flow should emit structured MINI_WINDOW logs"
        );
    }

    #[test]
    fn source_audit_scroll_reveal_reason_is_logged() {
        let source = std::fs::read_to_string("src/app_navigation/impl_scroll.rs")
            .expect("should read impl_scroll.rs");
        assert!(
            source.contains("reason,"),
            "scroll reveal should log the caller-provided reason"
        );
    }

    #[test]
    fn source_audit_mini_resize_reason_enum_exists() {
        let source = std::fs::read_to_string("src/window_resize/mod.rs")
            .expect("should read window_resize/mod.rs");
        assert!(
            source.contains("MiniResizeReason"),
            "MiniResizeReason enum should exist for structured resize receipts"
        );
    }
}
