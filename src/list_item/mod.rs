//! Shared ListItem component for script list and arg prompt choice list
//!
//! This module provides a reusable, theme-aware list item component that can be
//! used in both the main script list and arg prompt choice lists.

#![allow(dead_code)]

// --- merged from part_000.rs ---
use crate::designs::icon_variations::{icon_name_from_str, IconName};
use crate::logging;
use crate::ui_foundation::HexColorExt;
use gpui::*;
use gpui_component::tooltip::Tooltip;
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

impl IconKind {
    /// Resolve icon metadata into an IconKind for list rendering.
    ///
    /// Supports:
    /// - Known SVG names/aliases via `icon_name_from_str` (e.g., "Terminal", "file-code")
    /// - Emoji/symbol glyphs (e.g., "📄", "⚡")
    pub fn from_icon_hint(icon_hint: &str) -> Option<Self> {
        let trimmed = icon_hint.trim();
        if trimmed.is_empty() {
            return None;
        }

        if icon_name_from_str(trimmed).is_some() {
            return Some(Self::Svg(trimmed.to_string()));
        }

        if looks_like_symbol_icon_hint(trimmed) {
            return Some(Self::Emoji(trimmed.to_string()));
        }

        None
    }
}

fn looks_like_symbol_icon_hint(icon_hint: &str) -> bool {
    let has_ascii_alnum = icon_hint.chars().any(|ch| ch.is_ascii_alphanumeric());
    let char_count = icon_hint.chars().count();

    !has_ascii_alnum && char_count <= 4
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
pub(crate) const SECTION_PADDING_TOP: f32 = 12.0;
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
pub(crate) const ALPHA_SEPARATOR: u32 = 0x14;
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

#[cfg(test)]
mod icon_kind_tests {
    use super::IconKind;

    #[test]
    fn test_icon_kind_from_icon_hint_returns_svg_when_known_icon_name() {
        match IconKind::from_icon_hint("terminal") {
            Some(IconKind::Svg(name)) => assert_eq!(name, "terminal"),
            _ => panic!("expected SVG icon from known icon hint"),
        }
    }

    #[test]
    fn test_icon_kind_from_icon_hint_returns_emoji_when_symbol_glyph() {
        match IconKind::from_icon_hint("📄") {
            Some(IconKind::Emoji(emoji)) => assert_eq!(emoji, "📄"),
            _ => panic!("expected emoji icon for symbol glyph"),
        }
    }

    #[test]
    fn test_icon_kind_from_icon_hint_returns_none_for_unknown_ascii_word() {
        assert!(IconKind::from_icon_hint("unknown-icon-name").is_none());
    }
}
// --- merged from part_001.rs ---
impl ListItemColors {
    /// Create from theme reference
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        let opacity = theme.get_opacity();
        Self {
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            accent_selected: theme.colors.accent.selected,
            accent_selected_subtle: theme.colors.accent.selected_subtle,
            background: theme.colors.background.main,
            background_selected: theme.colors.accent.selected_subtle,
            selected_opacity: opacity.selected,
            hover_opacity: opacity.hover,
            warning_bg: theme.colors.ui.warning,
            // White text provides good contrast on warning/accent backgrounds in dark themes
            text_on_accent: theme.colors.text.primary,
        }
    }

    /// Create from design colors for GLOBAL theming support
    /// Uses same opacity values as from_theme() for consistent vibrancy-compatible styling
    ///
    /// NOTE: This defaults to dark mode opacity values. For light mode support,
    /// use `from_design_with_dark_mode()` instead.
    pub fn from_design(colors: &crate::designs::DesignColors) -> Self {
        // Default to dark mode
        Self::from_design_with_dark_mode(colors, true)
    }

    /// Create from design colors with explicit dark/light mode
    ///
    /// Light mode needs higher opacity values because low opacity on light backgrounds
    /// (e.g., white at 7-12%) is too subtle to be visible. Dark mode uses lower opacity
    /// because white overlays are more visible on dark backgrounds.
    ///
    /// # Arguments
    /// * `colors` - Design colors to use
    /// * `is_dark` - True for dark mode (lower opacity), false for light mode (higher opacity)
    pub fn from_design_with_dark_mode(
        colors: &crate::designs::DesignColors,
        is_dark: bool,
    ) -> Self {
        // Dark mode: low opacity works well (white at 7-12% visible on dark bg)
        // Light mode: needs higher opacity for visibility (black overlay on light bg)
        // Values aligned with Material Design elevation overlay model (~4-6dp)
        let (selected_opacity, hover_opacity) = if is_dark {
            (0.14, 0.08) // Dark mode: improved selection/hover visibility
        } else {
            (0.20, 0.12) // Light mode: stronger overlay for visibility on vibrancy
        };

        Self {
            text_primary: colors.text_primary,
            text_secondary: colors.text_secondary,
            text_muted: colors.text_muted,
            text_dimmed: colors.text_dimmed,
            accent_selected: colors.accent,
            accent_selected_subtle: colors.background_selected,
            background: colors.background,
            background_selected: colors.background_selected,
            selected_opacity,
            hover_opacity,
            warning_bg: colors.warning,
            text_on_accent: colors.text_on_accent,
        }
    }
}
/// Format a keyboard shortcut string using macOS-native modifier symbols.
///
/// Converts common shortcut formats to native macOS symbols:
/// - "cmd+shift+k" → "⌘⇧K"
/// - "ctrl+c" → "⌃C"
/// - "alt+enter" → "⌥↩"
///
/// If the input already contains macOS symbols (⌘, ⇧, ⌥, ⌃), returns as-is.
pub fn format_shortcut_display(shortcut: &str) -> String {
    // If already contains macOS modifier symbols, return as-is
    if shortcut.contains('⌘')
        || shortcut.contains('⇧')
        || shortcut.contains('⌥')
        || shortcut.contains('⌃')
    {
        return shortcut.to_string();
    }

    // Normalize: replace '+' with space, then split on whitespace.
    // This handles both space-delimited ("opt i", "cmd shift k") and
    // plus-delimited ("cmd+shift+k") shortcut formats from Script Kit metadata.
    let normalized = shortcut.replace('+', " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    let mut result = String::new();

    for part in &parts {
        match part.to_lowercase().as_str() {
            "cmd" | "command" | "meta" | "super" => result.push('⌘'),
            "shift" => result.push('⇧'),
            "alt" | "option" | "opt" => result.push('⌥'),
            "ctrl" | "control" => result.push('⌃'),
            "enter" | "return" => result.push('↩'),
            "escape" | "esc" => result.push('⎋'),
            "tab" => result.push('⇥'),
            "space" => result.push('␣'),
            "backspace" | "delete" => result.push('⌫'),
            "up" | "arrowup" => result.push('↑'),
            "down" | "arrowdown" => result.push('↓'),
            "left" | "arrowleft" => result.push('←'),
            "right" | "arrowright" => result.push('→'),
            key => {
                // Uppercase single-character keys, preserve multi-char keys as-is
                if key.len() == 1 {
                    result.push_str(&key.to_uppercase());
                } else {
                    result.push_str(key);
                }
            }
        }
    }

    result
}
/// Search rows keep shortcuts only on the actively focused row to avoid right-side noise.
pub(crate) fn should_show_search_shortcut(
    is_filtering: bool,
    selected: bool,
    hovered: bool,
) -> bool {
    if !is_filtering {
        return true;
    }
    selected || hovered
}
/// Search rows keep descriptions only when they add context for the current focus or match.
pub(crate) fn should_show_search_description(
    selected: bool,
    hovered: bool,
    has_description_match: bool,
) -> bool {
    selected || hovered || has_description_match
}
/// Callback type for hover events on list items.
/// The callback receives the item index and a boolean indicating hover state (true = entered, false = left).
pub type OnHoverCallback = Box<dyn Fn(usize, bool) + 'static>;
/// A reusable list item component for displaying selectable items
///
/// Supports:
/// - Name (required)
/// - Description (optional, shown below name)
/// - Icon (optional, emoji or PNG image displayed left of name)
/// - Shortcut badge (optional, right-aligned)
/// - Selection state with themed colors (full focus styling)
/// - Hover state with subtle visual feedback (separate from selection)
/// - Hover callback for mouse interaction (optional)
/// - Semantic ID for AI-driven targeting (optional)
///
#[derive(IntoElement)]
pub struct ListItem {
    name: SharedString,
    description: Option<String>,
    shortcut: Option<String>,
    icon: Option<IconKind>,
    selected: bool,
    /// Whether this item is being hovered (subtle visual feedback, separate from selected)
    hovered: bool,
    colors: ListItemColors,
    /// Index of this item in the list (needed for hover callback)
    index: Option<usize>,
    /// Optional callback triggered when mouse enters/leaves this item
    on_hover: Option<OnHoverCallback>,
    /// Semantic ID for AI-driven UX targeting. Format: {type}:{index}:{value}
    semantic_id: Option<String>,
    /// Show left accent bar when selected (3px colored bar on left edge)
    show_accent_bar: bool,
    /// Whether to enable instant hover effects (via GPUI .hover() pseudo-class)
    /// When false, the .hover() modifier is not applied, preventing visual feedback
    /// Used to disable hover when user is navigating with keyboard
    enable_hover_effect: bool,
    /// Character indices in the name that match the search query (for fuzzy highlight)
    /// When present, matched characters are rendered with accent color for visual emphasis
    highlight_indices: Option<Vec<usize>>,
    /// Character indices in the description that match the search query (for fuzzy highlight)
    /// When present, matched characters are rendered with accent color for visual emphasis
    description_highlight_indices: Option<Vec<usize>>,
    /// Type tag shown as subtle colored text (e.g., "Script", "Snippet", "App")
    /// Only shown during search mode to help distinguish mixed result types
    type_tag: Option<TypeTag>,
    /// Source/kit name (e.g., "main", "cleanshot") shown as subtle text during search
    source_hint: Option<String>,
    /// Tool/language badge for scriptlets (e.g., "ts", "bash", "paste")
    /// Shown as a subtle monospace badge in the accessories area
    tool_badge: Option<String>,
}
/// Type tag displayed as subtle colored text on list items during search
#[derive(Clone, Debug)]
pub struct TypeTag {
    /// Display label (e.g., "Script", "Snippet", "App")
    pub label: &'static str,
    /// Color for the tag (u32 hex, e.g., 0x3B82F6 for blue)
    pub color: u32,
}
/// Width of the left accent bar for selected items
pub const ACCENT_BAR_WIDTH: f32 = 3.0;
impl ListItem {
    /// Create a new list item with the given name and pre-computed colors
    pub fn new(name: impl Into<SharedString>, colors: ListItemColors) -> Self {
        Self {
            name: name.into(),
            description: None,
            shortcut: None,
            icon: None,
            selected: false,
            hovered: false,
            colors,
            index: None,
            on_hover: None,
            semantic_id: None,
            show_accent_bar: false,
            enable_hover_effect: true, // Default to enabled
            highlight_indices: None,
            description_highlight_indices: None,
            type_tag: None,
            source_hint: None,
            tool_badge: None,
        }
    }

    /// Enable the left accent bar (3px colored bar shown when selected)
    pub fn with_accent_bar(mut self, show: bool) -> Self {
        self.show_accent_bar = show;
        self
    }

    /// Enable or disable instant hover effects (GPUI .hover() pseudo-class)
    /// When disabled, no visual feedback is shown on mouse hover
    /// Used to prevent hover effects during keyboard navigation
    pub fn with_hover_effect(mut self, enable: bool) -> Self {
        self.enable_hover_effect = enable;
        self
    }

    /// Set the index of this item in the list (required for hover callback to work)
    pub fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Set a callback to be triggered when mouse enters or leaves this item.
    /// The callback receives (index, is_hovered) where is_hovered is true when entering.
    pub fn on_hover(mut self, callback: OnHoverCallback) -> Self {
        self.on_hover = Some(callback);
        self
    }

    /// Set the semantic ID for AI-driven UX targeting.
    /// Format: {type}:{index}:{value} (e.g., "choice:0:apple")
    pub fn semantic_id(mut self, id: impl Into<String>) -> Self {
        self.semantic_id = Some(id.into());
        self
    }

    /// Set an optional semantic ID (convenience for Option<String>)
    pub fn semantic_id_opt(mut self, id: Option<String>) -> Self {
        self.semantic_id = id;
        self
    }

    /// Set the description text (shown below the name)
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// Set an optional description (convenience for Option<String>)
    pub fn description_opt(mut self, d: Option<String>) -> Self {
        self.description = d;
        self
    }

    /// Set the shortcut badge text (shown right-aligned)
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }

    /// Set an optional shortcut (convenience for Option<String>)
    pub fn shortcut_opt(mut self, s: Option<String>) -> Self {
        self.shortcut = s;
        self
    }

    /// Set the icon (emoji) to display on the left side
    pub fn icon(mut self, i: impl Into<String>) -> Self {
        self.icon = Some(IconKind::Emoji(i.into()));
        self
    }

    /// Set an optional emoji icon (convenience for Option<String>)
    pub fn icon_opt(mut self, i: Option<String>) -> Self {
        self.icon = i.map(IconKind::Emoji);
        self
    }

    /// Set a pre-decoded RenderImage icon
    pub fn icon_image(mut self, image: Arc<RenderImage>) -> Self {
        self.icon = Some(IconKind::Image(image));
        self
    }

    /// Set an optional pre-decoded image icon
    pub fn icon_image_opt(mut self, image: Option<Arc<RenderImage>>) -> Self {
        self.icon = image.map(IconKind::Image);
        self
    }

    /// Set icon from IconKind enum (for mixed icon types)
    pub fn icon_kind(mut self, kind: IconKind) -> Self {
        self.icon = Some(kind);
        self
    }

    /// Set an optional icon from IconKind
    pub fn icon_kind_opt(mut self, kind: Option<IconKind>) -> Self {
        self.icon = kind;
        self
    }

    /// Set whether this item is selected
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set whether this item is hovered (visual feedback)
    ///
    /// Hovered items show a visible background tint (25% opacity).
    /// This is separate from `selected` which shows full focus styling
    /// (35% opacity background + accent bar).
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Set character indices for fuzzy match highlighting
    /// When set, matched characters in the name are rendered with accent color
    pub fn highlight_indices(mut self, indices: Vec<usize>) -> Self {
        if !indices.is_empty() {
            self.highlight_indices = Some(indices);
        }
        self
    }

    /// Set optional highlight indices (convenience for Option<Vec<usize>>)
    pub fn highlight_indices_opt(mut self, indices: Option<Vec<usize>>) -> Self {
        self.highlight_indices = indices.filter(|v| !v.is_empty());
        self
    }

    /// Set character indices for fuzzy match highlighting in description
    /// When set, matched characters in the description are rendered with accent color
    pub fn description_highlight_indices(mut self, indices: Vec<usize>) -> Self {
        if !indices.is_empty() {
            self.description_highlight_indices = Some(indices);
        }
        self
    }

    /// Set optional description highlight indices (convenience for Option<Vec<usize>>)
    pub fn description_highlight_indices_opt(mut self, indices: Option<Vec<usize>>) -> Self {
        self.description_highlight_indices = indices.filter(|v| !v.is_empty());
        self
    }

    /// Set a type tag to show as subtle colored text (e.g., "Script", "Snippet")
    /// Only used during search mode to distinguish mixed result types
    pub fn type_tag(mut self, tag: TypeTag) -> Self {
        self.type_tag = Some(tag);
        self
    }

    /// Set an optional type tag
    pub fn type_tag_opt(mut self, tag: Option<TypeTag>) -> Self {
        self.type_tag = tag;
        self
    }

    /// Set the source/kit name hint (shown during search to indicate origin)
    pub fn source_hint(mut self, hint: impl Into<String>) -> Self {
        self.source_hint = Some(hint.into());
        self
    }

    /// Set an optional source hint
    pub fn source_hint_opt(mut self, hint: Option<String>) -> Self {
        self.source_hint = hint;
        self
    }

    /// Set the tool/language badge (e.g., "ts", "bash", "paste")
    /// Displayed as a subtle monospace badge for scriptlets
    pub fn tool_badge(mut self, badge: impl Into<String>) -> Self {
        self.tool_badge = Some(badge.into());
        self
    }

    /// Set the tool/language badge from an option
    pub fn tool_badge_opt(mut self, badge: Option<String>) -> Self {
        self.tool_badge = badge;
        self
    }
}
// --- merged from part_002.rs ---
impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let index = self.index;
        let item_index = index.unwrap_or(0);
        let on_hover_callback = self.on_hover;
        let semantic_id = self.semantic_id;

        // Selection colors with alpha from theme opacity settings
        // This allows vibrancy blur to show through selected/hovered items
        // Use rgba8() helper (same pattern as footer) to ensure consistent Hsla conversion
        let selected_alpha = (colors.selected_opacity * 255.0) as u8;
        let hover_alpha = (colors.hover_opacity * 255.0) as u8;
        let selected_bg = colors.accent_selected_subtle.rgba8(selected_alpha);
        let hover_bg = colors.accent_selected_subtle.rgba8(hover_alpha);

        // Icon element (if present) - displayed on the left
        // Supports both emoji strings and PNG image data
        // Icons use slightly muted color to maintain text hierarchy
        let icon_text_color = if self.selected {
            rgb(colors.text_primary)
        } else {
            rgba((colors.text_primary << 8) | ALPHA_ICON_QUIET) // Quiet icons let names lead
        };
        let icon_size = px(ICON_CONTAINER_SIZE);
        let svg_size = px(ICON_SVG_SIZE);
        let icon_element = match &self.icon {
            Some(IconKind::Emoji(emoji)) => div()
                .w(icon_size)
                .h(icon_size)
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(icon_text_color)
                .flex_shrink_0()
                .child(emoji.clone()),
            Some(IconKind::Image(render_image)) => {
                // Render pre-decoded image directly (no decoding on render - critical for perf)
                let image = render_image.clone();
                div()
                    .w(icon_size)
                    .h(icon_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .child(
                        img(move |_window: &mut Window, _cx: &mut App| Some(Ok(image.clone())))
                            .w(icon_size)
                            .h(icon_size)
                            .object_fit(ObjectFit::Contain),
                    )
            }
            Some(IconKind::Svg(name)) => {
                // Convert string to IconName and render SVG
                // Use external_path() for file system SVGs (not path() which is for embedded assets)
                if let Some(icon_name) = icon_name_from_str(name) {
                    let svg_path = icon_name.external_path();
                    div()
                        .w(icon_size)
                        .h(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            svg()
                                .external_path(svg_path)
                                .size(svg_size)
                                .text_color(icon_text_color),
                        )
                } else {
                    // Fallback to Code icon if name not recognized
                    let svg_path = IconName::Code.external_path();
                    div()
                        .w(icon_size)
                        .h(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            svg()
                                .external_path(svg_path)
                                .size(svg_size)
                                .text_color(icon_text_color),
                        )
                }
            }
            None => {
                div().w(px(0.)).h(px(0.)) // No space if no icon
            }
        };

        // Progressive disclosure: detect if search/filter is active
        // Used to conditionally show descriptions and accessories
        let is_filtering =
            self.highlight_indices.is_some() || self.description_highlight_indices.is_some();

        // Build content with name + description (compact with small gap)
        let mut item_content = div()
            .flex_1()
            .min_w(px(0.))
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap(px(ITEM_NAME_DESC_GAP))
            .justify_center();

        // Name rendering - 14px font size for better balance with description
        // Medium weight for unselected, semibold when selected for clear emphasis
        // When highlight_indices are present, use StyledText to highlight matched characters
        // Otherwise, render as plain text
        let name_weight = if self.selected {
            FontWeight::MEDIUM // Subtle emphasis — launchers rely on background, not weight
        } else {
            FontWeight::NORMAL // Lighter weight reduces visual density
        };
        let name_element = if let Some(ref indices) = self.highlight_indices {
            // Build StyledText with highlighted matched characters
            let index_set: HashSet<usize> = indices.iter().copied().collect();
            let highlight_color = if self.selected {
                rgb(colors.text_primary)
            } else {
                rgba((colors.text_primary << 8) | ALPHA_MATCH_HIGHLIGHT)
            };
            let highlight_style = HighlightStyle {
                color: Some(highlight_color.into()),
                ..Default::default()
            };

            // Convert character indices to byte ranges for StyledText
            let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
            for (char_idx, (byte_offset, ch)) in self.name.char_indices().enumerate() {
                if index_set.contains(&char_idx) {
                    highlights.push((byte_offset..byte_offset + ch.len_utf8(), highlight_style));
                }
            }

            // Base text color is more muted when highlighting to create contrast
            let base_color = if self.selected {
                rgba((colors.text_secondary << 8) | ALPHA_HINT)
            } else {
                rgba((colors.text_muted << 8) | ALPHA_NAME_QUIET)
            };

            let full_name = self.name.to_string();
            let styled = StyledText::new(full_name.clone()).with_highlights(highlights);

            div()
                .text_size(px(NAME_FONT_SIZE))
                .font_weight(name_weight)
                .overflow_hidden()
                .text_ellipsis()
                .id(ElementId::NamedInteger("list-name".into(), item_index as u64))
                .tooltip(move |window, cx| Tooltip::new(full_name.clone()).build(window, cx))
                .whitespace_nowrap()
                .line_height(px(NAME_LINE_HEIGHT))
                .text_color(base_color)
                .child(styled)
        } else {
            // Plain text rendering (no search active)
            // Mute non-selected names to let selected item stand out
            let name_color = if self.selected {
                rgb(colors.text_primary)
            } else {
                rgba((colors.text_primary << 8) | ALPHA_NAME_QUIET)
            };
            let full_name = self.name.to_string();
            div()
                .text_size(px(NAME_FONT_SIZE))
                .font_weight(name_weight)
                .overflow_hidden()
                .text_ellipsis()
                .id(ElementId::NamedInteger("list-name".into(), item_index as u64))
                .tooltip(move |window, cx| Tooltip::new(full_name.clone()).build(window, cx))
                .whitespace_nowrap()
                .line_height(px(NAME_LINE_HEIGHT))
                .text_color(name_color)
                .child(self.name)
        };

        item_content = item_content.child(name_element);

        // Description - progressive disclosure pattern (Spotlight/Raycast style)
        // Search mode keeps rows quieter by showing descriptions only when focused
        // or when the description itself contains a search match.
        if let Some(desc) = self.description {
            let has_description_match = self.description_highlight_indices.is_some();
            let show_description = if is_filtering {
                should_show_search_description(self.selected, self.hovered, has_description_match)
            } else {
                self.selected || self.hovered
            };

            if show_description {
                let desc_alpha = if self.selected {
                    ALPHA_DESC_SELECTED
                } else {
                    ALPHA_DESC_QUIET
                };
                let desc_color = rgba((colors.text_secondary << 8) | desc_alpha);
                let desc_element = if let Some(ref desc_indices) =
                    self.description_highlight_indices
                {
                    // Build StyledText with highlighted matched characters in description
                    let index_set: HashSet<usize> = desc_indices.iter().copied().collect();
                    let highlight_color = if self.selected {
                        rgba((colors.text_primary << 8) | ALPHA_MATCH_HIGHLIGHT)
                    } else {
                        rgba((colors.text_secondary << 8) | ALPHA_HINT)
                    };
                    let highlight_style = HighlightStyle {
                        color: Some(highlight_color.into()),
                        ..Default::default()
                    };

                    // Convert character indices to byte ranges for StyledText
                    let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
                    for (char_idx, (byte_offset, ch)) in desc.char_indices().enumerate() {
                        if index_set.contains(&char_idx) {
                            highlights
                                .push((byte_offset..byte_offset + ch.len_utf8(), highlight_style));
                        }
                    }

                    let base_alpha = if self.selected {
                        ALPHA_DESC_SELECTED
                    } else {
                        ALPHA_DESC_QUIET
                    };
                    let base_color = rgba((colors.text_secondary << 8) | base_alpha);
                    let full_desc = desc.clone();
                    let styled = StyledText::new(full_desc.clone()).with_highlights(highlights);

                    div()
                        .text_size(px(DESC_FONT_SIZE))
                        .line_height(px(DESC_LINE_HEIGHT))
                        .text_color(base_color)
                        .overflow_hidden()
                        .text_ellipsis()
                        .id(ElementId::NamedInteger("list-desc".into(), item_index as u64))
                        .tooltip(move |window, cx| Tooltip::new(full_desc.clone()).build(window, cx))
                        .whitespace_nowrap()
                        .child(styled)
                } else {
                    let full_desc = desc.clone();
                    div()
                        .text_size(px(DESC_FONT_SIZE))
                        .line_height(px(DESC_LINE_HEIGHT))
                        .text_color(desc_color)
                        .overflow_hidden()
                        .text_ellipsis()
                        .id(ElementId::NamedInteger("list-desc".into(), item_index as u64))
                        .tooltip(move |window, cx| Tooltip::new(full_desc.clone()).build(window, cx))
                        .whitespace_nowrap()
                        .child(desc)
                };
                item_content = item_content.child(desc_element);
            }
        }

        // Shortcut badge (if present) - right-aligned with kbd-style rendering
        // Uses macOS-native modifier symbols (⌘, ⇧, ⌥, ⌃) for a native feel
        let shortcut_element = if let Some(sc) = self.shortcut {
            let show_shortcut =
                should_show_search_shortcut(is_filtering, self.selected, self.hovered);
            if show_shortcut {
                let display_text = format_shortcut_display(&sc);
                if is_filtering {
                    div()
                        .text_size(px(SEARCH_SHORTCUT_FONT_SIZE))
                        .font_family(FONT_MONO)
                        .text_color(rgba((colors.text_dimmed << 8) | ALPHA_HINT))
                        .child(display_text)
                } else {
                    let badge_border = (colors.text_dimmed << 8) | ALPHA_BORDER;
                    div()
                        .text_size(px(BADGE_FONT_SIZE))
                        .font_family(FONT_MONO)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgba((colors.text_muted << 8) | ALPHA_READABLE))
                        .px(px(BADGE_PADDING_X))
                        .py(px(BADGE_PADDING_Y))
                        .rounded(px(BADGE_RADIUS))
                        .bg(rgba((colors.text_dimmed << 8) | ALPHA_TINT_LIGHT))
                        .border_1()
                        .border_color(rgba(badge_border))
                        .child(display_text)
                }
            } else {
                div()
            }
        } else {
            div()
        };

        // Determine background color based on selection/hover state
        // Priority: selected (full focus styling) > hovered (subtle feedback) > transparent
        // Note: For non-selected items, we ALSO apply GPUI's .hover() modifier for instant feedback
        let bg_color: Hsla = if self.selected {
            selected_bg // 15% opacity - subtle selection with vibrancy
        } else if self.hovered {
            hover_bg // 10% opacity - subtle hover feedback (state-based)
        } else {
            hsla(0.0, 0.0, 0.0, 0.0) // fully transparent
        };

        // Build the inner content div with all styling
        // Horizontal padding ITEM_PADDING_X and vertical padding ITEM_PADDING_Y
        //
        // HOVER TRANSITIONS: We use GPUI's built-in .hover() modifier for instant visual
        // feedback on non-selected items. This provides CSS-like instant hover effects
        // without waiting for state updates via cx.notify().
        //
        // For selected items, we don't apply hover styles (they already have full focus styling).
        let mut inner_content = div()
            .w_full()
            .h_full()
            .px(px(ITEM_PADDING_X))
            .py(px(ITEM_PADDING_Y))
            .bg(bg_color)
            .text_color(rgb(colors.text_primary))
            .font_family(FONT_SYSTEM_UI)
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(ITEM_ICON_TEXT_GAP))
            .child(icon_element)
            .child(item_content)
            .child({
                // Right-side accessories: [source hint] [type tag] [shortcut badge]
                let mut accessories = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .flex_shrink_0()
                    .gap(px(ITEM_ACCESSORIES_GAP));

                // Tool badge, source hint, and type tag use progressive disclosure.
                // Search mode intentionally strips noisy metadata to keep rows calm.
                let show_accessories = self.selected || self.hovered || is_filtering;

                // Tool/language badge for scriptlets (e.g., "ts", "bash")
                if show_accessories && !is_filtering {
                    if let Some(ref badge) = self.tool_badge {
                        let badge_bg = (colors.text_dimmed << 8) | ALPHA_TINT_MEDIUM;
                        accessories = accessories.child(
                            div()
                                .text_size(px(TOOL_BADGE_FONT_SIZE))
                                .font_family(FONT_MONO)
                                .text_color(rgba((colors.text_dimmed << 8) | ALPHA_READABLE))
                                .px(px(TOOL_BADGE_PADDING_X))
                                .py(px(TOOL_BADGE_PADDING_Y))
                                .rounded(px(TOOL_BADGE_RADIUS))
                                .bg(rgba(badge_bg))
                                .child(badge.clone()),
                        );
                    }
                }

                // Source/kit hint (e.g., "main", "cleanshot") - very subtle
                if show_accessories && !is_filtering {
                    if let Some(ref hint) = self.source_hint {
                        accessories = accessories.child(
                            div()
                                .text_size(px(SOURCE_HINT_FONT_SIZE))
                                .text_color(rgba((colors.text_dimmed << 8) | ALPHA_HINT))
                                .child(hint.clone()),
                        );
                    }
                }

                // Type tag stays visible during search, but as quiet text instead of a pill badge.
                if let Some(ref tag) = self.type_tag {
                    accessories = accessories.child(
                        div()
                            .text_size(px(TYPE_TAG_FONT_SIZE))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgba((tag.color << 8) | ALPHA_TYPE_LABEL))
                            .child(tag.label),
                    );
                }

                accessories = accessories.child(shortcut_element);
                accessories
            });

        // Apply instant hover effect for non-selected items when hover effects are enabled
        // This provides immediate visual feedback without state updates
        // Hover effects are disabled during keyboard navigation to prevent dual-highlight
        if !self.selected && self.enable_hover_effect {
            inner_content = inner_content.hover(move |s| s.bg(hover_bg));
        }

        // Use semantic_id for element ID if available, otherwise fall back to index
        // This allows AI agents to target elements by their semantic meaning
        let element_id = if let Some(ref sem_id) = semantic_id {
            // Use semantic ID as the element ID for better targeting
            ElementId::Name(sem_id.clone().into())
        } else {
            // Fall back to index-based ID
            ElementId::NamedInteger("list-item".into(), item_index as u64)
        };

        // Accent bar: Use LEFT BORDER instead of child div because:
        // 1. GPUI clamps corner radii to ≤ half the shortest side
        // 2. A 3px-wide child with 12px radius gets clamped to ~1.5px (invisible)
        // 3. A border on the container follows rounded corners naturally
        let accent_color = rgb(colors.accent_selected);

        // Base container with ID for stateful interactivity
        // Use left border for accent indicator - always reserve space, toggle color
        let mut container = div()
            .w_full()
            .h(px(LIST_ITEM_HEIGHT))
            .pr(px(ITEM_CONTAINER_PADDING_R))
            .flex()
            .flex_row()
            .items_center()
            .id(element_id);

        // Apply accent bar as left border (only when enabled)
        if self.show_accent_bar {
            container = container
                .border_l(px(ACCENT_BAR_WIDTH))
                .border_color(if self.selected {
                    accent_color
                } else {
                    rgba(0x00000000)
                });
        }

        // Add hover handler if we have both index and callback
        if let (Some(idx), Some(callback)) = (index, on_hover_callback) {
            // Use Rc to allow sharing the callback in the closure
            let callback = std::rc::Rc::new(callback);

            container = container.on_hover(move |hovered: &bool, _window, _cx| {
                // Log the mouse enter/leave event
                if *hovered {
                    logging::log_mouse_enter(idx, None);
                } else {
                    logging::log_mouse_leave(idx, None);
                }
                // Call the user-provided callback
                callback(idx, *hovered);
            });
        }

        // Add content (no separate accent bar child needed)
        container.child(inner_content)
    }
}
/// Decode PNG bytes to GPUI RenderImage
///
/// Decode PNG bytes to a GPUI RenderImage
///
/// Uses the `image` crate to decode PNG data and creates a GPUI-compatible
/// RenderImage for display. Returns an Arc<RenderImage> for caching.
///
/// **IMPORTANT**: Call this ONCE when loading icons, NOT during rendering.
/// Decoding PNGs on every render frame causes severe performance issues.
pub fn decode_png_to_render_image(png_data: &[u8]) -> Result<Arc<RenderImage>, image::ImageError> {
    decode_png_to_render_image_internal(png_data, false)
}
/// Decode PNG bytes to GPUI RenderImage with RGBA→BGRA conversion for Metal
///
/// GPUI/Metal expects BGRA pixel format. When creating RenderImage directly
/// from image::Frame (bypassing GPUI's internal loaders), we must do the
/// RGBA→BGRA conversion ourselves. This matches what GPUI does internally
/// in platform.rs for loaded images.
///
/// **IMPORTANT**: Call this ONCE when loading icons, NOT during rendering.
pub fn decode_png_to_render_image_with_bgra_conversion(
    png_data: &[u8],
) -> Result<Arc<RenderImage>, image::ImageError> {
    decode_png_to_render_image_internal(png_data, true)
}
// --- merged from part_003.rs ---
fn decode_png_to_render_image_internal(
    png_data: &[u8],
    convert_to_bgra: bool,
) -> Result<Arc<RenderImage>, image::ImageError> {
    use image::GenericImageView;
    use smallvec::SmallVec;

    // Decode PNG
    let img = image::load_from_memory(png_data)?;

    // Convert to RGBA8
    let mut rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    // Convert RGBA to BGRA for Metal/GPUI rendering
    // GPUI's internal image loading does this swap (see gpui/src/platform.rs)
    // We must do the same when creating RenderImage directly from image::Frame
    if convert_to_bgra {
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2); // Swap R and B: RGBA -> BGRA
        }
    }

    // Create Frame from buffer (now in BGRA order if converted)
    let buffer = image::RgbaImage::from_raw(width, height, rgba.into_raw())
        .ok_or_else(|| {
            image::ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to create image buffer",
            ))
        })?;
    let frame = image::Frame::new(buffer);

    // Create RenderImage
    let render_image = RenderImage::new(SmallVec::from_elem(frame, 1));

    Ok(Arc::new(render_image))
}
/// Create an IconKind from PNG bytes by pre-decoding them
///
/// Returns None if decoding fails. This should be called once when loading
/// icons, not during rendering.
pub fn icon_from_png(png_data: &[u8]) -> Option<IconKind> {
    decode_png_to_render_image(png_data)
        .ok()
        .map(IconKind::Image)
}
/// Render a section header for grouped lists (e.g., "Recent", "Main")
///
/// Visual design for section headers:
/// - Standard casing (not uppercase)
/// - 12px font (meets desktop minimum)
/// - Semi-bold weight (SEMIBOLD for subtlety)
/// - Dimmed color (subtle but readable)
/// - 32px height (8px grid aligned)
/// - Left-aligned with list item padding
/// - Subtle background tint for visual grouping
///
/// ## Technical Note: list() Height
/// Uses GPUI's `list()` component which supports variable-height items.
/// Section headers render at 32px, regular items at 40px.
///
/// # Arguments
/// * `label` - The section label (displayed as-is, standard casing)
/// * `icon` - Optional icon name (lucide icon, e.g., "settings")
/// * `colors` - ListItemColors for theme-aware styling
/// * `is_first` - Whether this is the first header in the list (suppresses top border)
///
pub fn render_section_header(
    label: &str,
    icon: Option<&str>,
    colors: ListItemColors,
    is_first: bool,
) -> impl IntoElement {
    // Section header at 32px (8px grid aligned, SECTION_HEADER_HEIGHT)
    // Used with GPUI's list() component which supports variable-height items.
    //
    // Layout: 32px total height
    // - pt(12px) top padding for visual separation from above item
    // - ~12px text height
    // - pb(4px) bottom padding for visual separation from below item

    // Parse label to separate name from count (e.g., "SUGGESTED · 5" → "SUGGESTED", "5")
    let (section_name, count_text) = if let Some(dot_pos) = label.find(" · ") {
        (&label[..dot_pos], Some(&label[dot_pos + " · ".len()..]))
    } else {
        (label, None)
    };

    // Build the inner content row: icon (optional) → section name → count (optional)
    // Headers should whisper — subtle orientation labels, not attention-grabbers
    let header_text_color = rgba((colors.text_secondary << 8) | ALPHA_ICON_QUIET);
    let mut content = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(SECTION_GAP))
        .text_size(px(SECTION_HEADER_FONT_SIZE))
        .font_weight(FontWeight::NORMAL) // Lightest weight — headers recede behind items
        .text_color(header_text_color);

    // Add icon before section name if provided — very quiet to avoid visual noise
    if let Some(name) = icon {
        if let Some(icon_name) = icon_name_from_str(name) {
            content = content.child(
                svg()
                    .external_path(icon_name.external_path())
                    .size(px(SECTION_HEADER_ICON_SIZE))
                    .text_color(rgba((colors.text_secondary << 8) | ALPHA_DESC_QUIET)),
            );
        }
    }

    content = content.child(section_name.to_string());

    // Add count badge if present - rendered as a very subtle separate element
    if let Some(count) = count_text {
        content = content.child(
            div()
                .text_xs()
                .font_weight(FontWeight::NORMAL)
                .text_color(rgba((colors.text_secondary << 8) | ALPHA_DESC_QUIET))
                .child(count.to_string()),
        );
    }

    // Clean section headers — no background tint for a calmer list appearance
    let header = div()
        .w_full()
        .h(px(SECTION_HEADER_HEIGHT))
        .px(px(SECTION_PADDING_X))
        .pt(px(SECTION_PADDING_TOP))
        .pb(px(SECTION_PADDING_BOTTOM))
        .flex()
        .flex_col()
        .justify_end(); // Align content to bottom for better visual anchoring

    // Only show top separator on non-first headers — very subtle
    let header = if is_first {
        header
    } else {
        header
            .border_t_1()
            .border_color(rgba((colors.text_secondary << 8) | ALPHA_SEPARATOR))
    };

    header.child(content)
}
// Note: GPUI rendering tests omitted due to GPUI macro recursion limit issues.
// The LIST_ITEM_HEIGHT constant is 40.0 and the component is integration-tested
// via the main application's script list and arg prompt rendering.
// Unit tests for format_shortcut_display are in src/list_item_tests.rs.
