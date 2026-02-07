use crate::designs::icon_variations::{icon_name_from_str, IconName};
use crate::logging;
use crate::ui_foundation::HexColorExt;
use gpui::*;
use std::collections::HashSet;
use std::sync::Arc;
/// Icon type for list items - supports emoji strings, SVG icons, and pre-decoded images
#[derive(Clone)]
pub enum IconKind {
    /// Text/emoji icon (e.g., "📜", "⚡")
    Emoji(String),
    /// Pre-decoded render image (for app icons) - MUST be pre-decoded, not raw PNG bytes
    Image(Arc<RenderImage>),
    /// SVG icon by name (e.g., "File", "Terminal", "Code")
    /// Maps to IconName from designs::icon_variations
    Svg(String),
}
/// Fixed height for list items used in uniform-height virtualized lists.
///
/// IMPORTANT: When using GPUI `uniform_list`, the item closure must render
/// at exactly this height (including padding). If you change visuals, keep the
/// total height stable or update this constant everywhere it is used.
pub const LIST_ITEM_HEIGHT: f32 = 40.0;
/// Fixed height for section headers (RECENT, MAIN, etc.)
/// Total height includes: pt(12px) + text (~12px) + pb(4px) = ~28px content
/// Using 32px (8px grid) for comfortable spacing while maintaining visual compactness.
///
/// ## Performance Note (uniform_list vs list)
/// - Use `uniform_list` when every row has the same fixed height (fast O(1) scroll math).
/// - Use `list()` when you need variable heights (e.g., headers + items); it uses a SumTree
///   and scroll math is O(log n).
pub const SECTION_HEADER_HEIGHT: f32 = 32.0;
// =============================================================================
// Layout & Spacing Constants (8px grid with 4px micro-steps)
// =============================================================================

/// Horizontal padding for list item inner content
const ITEM_PADDING_X: f32 = 14.0;
/// Vertical padding for list item inner content
const ITEM_PADDING_Y: f32 = 4.0;
/// Right padding on outer container (balances accent bar width)
const ITEM_CONTAINER_PADDING_R: f32 = 4.0;
/// Gap between icon and text content
const ITEM_ICON_TEXT_GAP: f32 = 8.0;
/// Gap between name and description lines
const ITEM_NAME_DESC_GAP: f32 = 2.0;
/// Gap between accessory items (badge, tag, hint)
const ITEM_ACCESSORIES_GAP: f32 = 6.0;
// =============================================================================
// Icon Dimensions
// =============================================================================

/// Icon container size (width and height)
const ICON_CONTAINER_SIZE: f32 = 20.0;
/// SVG icon render size (inside the container)
const ICON_SVG_SIZE: f32 = 16.0;
// =============================================================================
// Typography — font sizes & line heights
// =============================================================================

/// Item name font size (14px — dense desktop default for primary labels)
const NAME_FONT_SIZE: f32 = 14.0;
/// Item name line height
const NAME_LINE_HEIGHT: f32 = 20.0;
/// Item description font size (12px — minimum for desktop legibility)
const DESC_FONT_SIZE: f32 = 12.0;
/// Item description line height (16px fits better within 40px item height)
const DESC_LINE_HEIGHT: f32 = 16.0;
/// Keyboard shortcut badge font size
const BADGE_FONT_SIZE: f32 = 11.0;
/// Search-mode shortcut font size (kept compact to reduce clutter)
const SEARCH_SHORTCUT_FONT_SIZE: f32 = 10.0;
/// Tool/language badge font size (e.g. "ts", "bash")
const TOOL_BADGE_FONT_SIZE: f32 = 10.0;
/// Source hint font size (e.g. "main", "cleanshot")
const SOURCE_HINT_FONT_SIZE: f32 = 11.0;
/// Type tag pill font size (e.g. "Script", "Snippet")
const TYPE_TAG_FONT_SIZE: f32 = 11.0;
/// Section header label font size
const SECTION_HEADER_FONT_SIZE: f32 = 12.0;
/// Section header icon size
const SECTION_HEADER_ICON_SIZE: f32 = 10.0;
// =============================================================================
// Badge & Tag Spacing
// =============================================================================

/// Shortcut badge horizontal padding
const BADGE_PADDING_X: f32 = 6.0;
/// Shortcut badge vertical padding
const BADGE_PADDING_Y: f32 = 2.0;
/// Shortcut badge corner radius
const BADGE_RADIUS: f32 = 4.0;
/// Tool badge horizontal padding
const TOOL_BADGE_PADDING_X: f32 = 4.0;
/// Tool badge vertical padding
const TOOL_BADGE_PADDING_Y: f32 = 1.0;
/// Tool badge corner radius
const TOOL_BADGE_RADIUS: f32 = 3.0;
/// Type tag pill horizontal padding
const TYPE_TAG_PADDING_X: f32 = 8.0;
/// Type tag pill vertical padding
const TYPE_TAG_PADDING_Y: f32 = 2.0;
/// Type tag pill corner radius
const TYPE_TAG_RADIUS: f32 = 4.0;
// =============================================================================
// Section Header Spacing
// =============================================================================

/// Section header horizontal padding (matches item padding for alignment)
const SECTION_PADDING_X: f32 = 14.0;
/// Section header top padding (visual separation from above item)
const SECTION_PADDING_TOP: f32 = 12.0;
/// Section header bottom padding
const SECTION_PADDING_BOTTOM: f32 = 4.0;
/// Gap between header elements (icon, label, count)
const SECTION_GAP: f32 = 6.0;
// =============================================================================
// Opacity tokens — named for intent, hex for GPUI rgba() bit-packing
// =============================================================================

/// 85% opacity — used for selected description text
const ALPHA_STRONG: u32 = 0xD9;
/// 70% opacity — selected description remains secondary to the item title
const ALPHA_DESC_SELECTED: u32 = 0xB3;
/// 72% opacity — used for non-selected item names
/// Softer than ALPHA_STRONG to let non-selected items recede (Raycast/Spotlight pattern)
const ALPHA_NAME_QUIET: u32 = 0xB8;
/// 50% opacity — used for non-selected item icons
/// Low enough that icons don't compete for attention; selected items restore full color
const ALPHA_ICON_QUIET: u32 = 0x80;
/// 80% opacity — used for shortcut badge text and position indicator
pub(crate) const ALPHA_READABLE: u32 = 0xCC;
/// 75% opacity — used for header icon, tool badge text
/// (Bumped from 70% for better legibility on vibrancy backgrounds)
const ALPHA_MUTED: u32 = 0xBF;
/// 35% opacity — used for non-selected description text
/// Makes descriptions visible on hover/focus but clearly recedes in the list
const ALPHA_DESC_QUIET: u32 = 0x59;
/// 70% opacity — used for source hint text
/// (Bumped from 65% for WCAG-friendlier contrast on vibrancy)
const ALPHA_HINT: u32 = 0xB3;
/// 60% opacity — subtle type labels during search
const ALPHA_TYPE_LABEL: u32 = 0x99;
/// 75% opacity — subtle matched-character tint without high-contrast flash
const ALPHA_MATCH_HIGHLIGHT: u32 = 0xBF;
/// 65% opacity — used for section header count
/// (Bumped from 60% for better readability of numeric info)
const ALPHA_SUBTLE: u32 = 0xA6;
/// 30% opacity — used for shortcut badge border
/// (Bumped from 25% so kbd borders pass ≥3:1 non-text contrast on vibrancy)
const ALPHA_BORDER: u32 = 0x4D;
/// 22% opacity — reserved for strong separators (currently unused, kept for design variants)
const _ALPHA_SEPARATOR_STRONG: u32 = 0x38;
/// 20% opacity — used for type tag pill background
/// (Higher than separator for visible badge fill on vibrancy)
const ALPHA_TAG_BG: u32 = 0x33;
/// 35% opacity — used for type tag pill border
const ALPHA_TAG_BORDER: u32 = 0x59;
/// 8% opacity — used for section separator
/// Barely-there divider; the section label itself provides enough grouping signal
const ALPHA_SEPARATOR: u32 = 0x14;
/// 7% opacity — used for tool badge background
/// (Bumped from 6% for slightly more visible badge pills)
const ALPHA_TINT_MEDIUM: u32 = 0x12;
/// 6% opacity — used for shortcut badge background
/// (Bumped from 5% for subtle but visible kbd backgrounds)
const ALPHA_TINT_LIGHT: u32 = 0x0F;
/// 7% opacity — used for section header background tint
/// (Bumped from 5% for more visible section grouping)
const ALPHA_TINT_FAINT: u32 = 0x12;
// =============================================================================
// Empty State Constants
// =============================================================================

/// Gap between empty state icon and text elements
pub(crate) const EMPTY_STATE_GAP: f32 = 12.0;
/// Empty state icon size (Code / MagnifyingGlass)
pub(crate) const EMPTY_STATE_ICON_SIZE: f32 = 32.0;
/// Empty state primary message font size
pub(crate) const EMPTY_STATE_MESSAGE_FONT_SIZE: f32 = 14.0;
/// Empty state filter tips top margin
pub(crate) const EMPTY_STATE_TIPS_MARGIN_TOP: f32 = 8.0;
// Empty state opacity tokens — deliberately lower than list content
// to keep the empty state understated while still legible

/// 28% opacity — empty state icon tint
/// (Bumped from 22% for better visibility; still understated vs list content)
pub(crate) const ALPHA_EMPTY_ICON: u32 = 0x47;
/// 45% opacity — empty state primary message text
/// (Bumped from 38% to meet minimum readable contrast on vibrancy)
pub(crate) const ALPHA_EMPTY_MESSAGE: u32 = 0x73;
/// 33% opacity — empty state secondary hint text
/// (Bumped from 27% for improved legibility of help text)
pub(crate) const ALPHA_EMPTY_HINT: u32 = 0x54;
/// 22% opacity — empty state filter tips text
/// (Bumped from 17% — was barely visible; now subtly legible)
pub(crate) const ALPHA_EMPTY_TIPS: u32 = 0x38;
// =============================================================================
// Header Area Constants (Ask AI button, Tab badge, indicators)
// =============================================================================

/// Gap between "Ask AI" text and "Tab" badge
pub(crate) const ASK_AI_BUTTON_GAP: f32 = 6.0;
/// Ask AI button horizontal padding
pub(crate) const ASK_AI_BUTTON_PADDING_X: f32 = 6.0;
/// Ask AI button vertical padding
pub(crate) const ASK_AI_BUTTON_PADDING_Y: f32 = 4.0;
/// Ask AI button corner radius
pub(crate) const ASK_AI_BUTTON_RADIUS: f32 = 4.0;
/// Tab badge horizontal padding
pub(crate) const TAB_BADGE_PADDING_X: f32 = 6.0;
/// Tab badge vertical padding
pub(crate) const TAB_BADGE_PADDING_Y: f32 = 2.0;
/// Tab badge corner radius
pub(crate) const TAB_BADGE_RADIUS: f32 = 4.0;
/// 15% opacity — hover accent background on interactive buttons
pub(crate) const ALPHA_HOVER_ACCENT: u32 = 0x26;
/// 30% opacity — Tab badge background tint
pub(crate) const ALPHA_TAB_BADGE_BG: u32 = 0x4D;
/// 80% opacity — library size count hint (boosted for vibrancy readability)
pub(crate) const ALPHA_COUNT_HINT: u32 = 0xCC;
// =============================================================================
// Divider & Scroll Constants
// =============================================================================

/// Default horizontal margin for the header/list divider
pub(crate) const DIVIDER_MARGIN_DEFAULT: f32 = 16.0;
/// Default border width for the header/list divider (1px hairline)
pub(crate) const DIVIDER_BORDER_WIDTH_DEFAULT: f32 = 1.0;
/// Maximum height for the log panel area
pub(crate) const LOG_PANEL_MAX_HEIGHT: f32 = 120.0;
/// 50% opacity — divider line between header and list (visible separation)
pub(crate) const ALPHA_DIVIDER: u32 = 0x80;
/// Estimated visible list container height for scrollbar calculations
/// Window is 500px, header is ~60px, footer is ~34px → ~406px for list area
pub(crate) const ESTIMATED_LIST_CONTAINER_HEIGHT: f32 = 436.0;
/// Average item height for scroll-wheel delta-to-index conversion
/// Weighted: most items are 40px (LIST_ITEM_HEIGHT), headers are 32px (SECTION_HEADER_HEIGHT)
pub(crate) const AVERAGE_ITEM_HEIGHT_FOR_SCROLL: f32 = 44.0;
// =============================================================================
// Font Family Tokens
// =============================================================================

/// System UI font for all list item text
pub(crate) const FONT_SYSTEM_UI: &str = ".AppleSystemUIFont";
/// Monospace font for keyboard shortcuts and code badges
pub(crate) const FONT_MONO: &str = "SF Mono";
/// Enum for grouped list items - supports both regular items and section headers
///
/// Used with GPUI's `list()` component when rendering grouped results (e.g., frecency with RECENT/MAIN sections).
/// The usize in Item variant is the index into the flat results array.
#[derive(Clone, Debug)]
pub enum GroupedListItem {
    /// A section header (e.g., "SUGGESTED", "MAIN")
    SectionHeader(String, Option<String>),
    /// A regular list item - usize is the index in the flat results array
    Item(usize),
}
/// Coerce a selection index to land on a selectable (non-header) row.
///
/// When the given index lands on a header or is out of bounds:
/// 1. First tries searching DOWN to find the next Item
/// 2. If not found, searches UP to find the previous Item
/// 3. If still not found (list has no items), returns None
///
/// This is the canonical way to ensure selection never lands on a header.
///
/// # Performance
/// O(n) worst case, but typically O(1) since headers are sparse.
///
/// # Returns
/// - `Some(index)` - Valid selectable index
/// - `None` - No selectable items exist (list is empty or contains only headers)
pub fn coerce_selection(rows: &[GroupedListItem], ix: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    // Clamp to valid range first
    let ix = ix.min(rows.len() - 1);

    // If already on a selectable item, done
    if matches!(rows[ix], GroupedListItem::Item(_)) {
        return Some(ix);
    }

    // Search down for next selectable
    for (j, item) in rows.iter().enumerate().skip(ix + 1) {
        if matches!(item, GroupedListItem::Item(_)) {
            return Some(j);
        }
    }

    // Search up for previous selectable
    for (j, item) in rows.iter().enumerate().take(ix).rev() {
        if matches!(item, GroupedListItem::Item(_)) {
            return Some(j);
        }
    }

    // No selectable items found
    None
}
/// Pre-computed grouped list state for efficient navigation
///
/// This struct caches header positions and total counts to avoid expensive
/// recalculation on every keypress. Build it once when the list data changes,
/// then reuse for navigation.
///
/// ## Performance
/// - `is_header()`: O(1) lookup via HashSet
/// - `next_selectable()` / `prev_selectable()`: O(k) where k is consecutive headers
/// - Memory: O(h) where h is number of headers (typically < 10)
///
#[derive(Clone, Debug)]
pub struct GroupedListState {
    /// Set of indices that are headers (for O(1) lookup)
    header_indices: std::collections::HashSet<usize>,
    /// Total number of visual items (headers + entries)
    pub total_items: usize,
    /// Index of first selectable item (skips leading header)
    pub first_selectable: usize,
}
impl GroupedListState {
    /// Create from a list of (group_name, item_count) pairs
    ///
    /// Each group gets a header at the start, followed by its items.
    /// Empty groups are skipped (no header for empty groups).
    pub fn from_groups(groups: &[(&str, usize)]) -> Self {
        let mut header_indices = std::collections::HashSet::new();
        let mut idx = 0;

        for (_, count) in groups {
            if *count > 0 {
                header_indices.insert(idx); // Header position
                idx += 1 + count; // Header + items
            }
        }

        let first_selectable = if header_indices.contains(&0) { 1 } else { 0 };

        Self {
            header_indices,
            total_items: idx,
            first_selectable,
        }
    }

    /// Create from pre-built GroupedListItem vec (when you already have the items)
    pub fn from_items(items: &[GroupedListItem]) -> Self {
        let mut header_indices = std::collections::HashSet::new();

        for (idx, item) in items.iter().enumerate() {
            if matches!(item, GroupedListItem::SectionHeader(..)) {
                header_indices.insert(idx);
            }
        }

        let first_selectable = if header_indices.contains(&0) { 1 } else { 0 };

        Self {
            header_indices,
            total_items: items.len(),
            first_selectable,
        }
    }

    /// Create an empty state (no headers, for flat lists)
    pub fn flat(item_count: usize) -> Self {
        Self {
            header_indices: std::collections::HashSet::new(),
            total_items: item_count,
            first_selectable: 0,
        }
    }

    /// Check if an index is a header (O(1))
    #[inline]
    pub fn is_header(&self, index: usize) -> bool {
        self.header_indices.contains(&index)
    }

    /// Get next selectable index (skips headers), or None if at end
    pub fn next_selectable(&self, current: usize) -> Option<usize> {
        let mut next = current + 1;
        while next < self.total_items && self.is_header(next) {
            next += 1;
        }
        if next < self.total_items {
            Some(next)
        } else {
            None
        }
    }

    /// Get previous selectable index (skips headers), or None if at start
    pub fn prev_selectable(&self, current: usize) -> Option<usize> {
        if current == 0 {
            return None;
        }
        let mut prev = current - 1;
        while prev > 0 && self.is_header(prev) {
            prev -= 1;
        }
        if !self.is_header(prev) {
            Some(prev)
        } else {
            None
        }
    }

    /// Get number of headers
    pub fn header_count(&self) -> usize {
        self.header_indices.len()
    }
}
/// Pre-computed colors for ListItem rendering
///
/// This struct holds the primitive color values needed for list item rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy)]
pub struct ListItemColors {
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub text_dimmed: u32,
    pub accent_selected: u32,
    pub accent_selected_subtle: u32,
    pub background: u32,
    pub background_selected: u32,
    /// Opacity for selected item background (from theme.opacity.selected)
    pub selected_opacity: f32,
    /// Opacity for hovered item background (from theme.opacity.hover)
    pub hover_opacity: f32,
    /// Warning background color (for confirmation overlays, alerts)
    pub warning_bg: u32,
    /// Text color for content displayed on accent/warning backgrounds
    pub text_on_accent: u32,
}
