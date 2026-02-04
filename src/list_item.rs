//! Shared ListItem component for script list and arg prompt choice list
//!
//! This module provides a reusable, theme-aware list item component that can be
//! used in both the main script list and arg prompt choice lists.

#![allow(dead_code)]

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
pub const LIST_ITEM_HEIGHT: f32 = 36.0;

/// Fixed height for section headers (RECENT, MAIN, etc.)
/// Total height includes: pt(8px) + text (~8px via text_xs) + pb(4px) = ~20px content
/// Using 24px for comfortable spacing while maintaining visual compactness.
///
/// ## Performance Note (uniform_list vs list)
/// - Use `uniform_list` when every row has the same fixed height (fast O(1) scroll math).
/// - Use `list()` when you need variable heights (e.g., headers + items); it uses a SumTree
///   and scroll math is O(log n).
pub const SECTION_HEADER_HEIGHT: f32 = 24.0;

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
        let (selected_opacity, hover_opacity) = if is_dark {
            (0.12, 0.07) // Dark mode defaults
        } else {
            (0.18, 0.12) // Light mode: higher opacity for visibility
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
    /// Type tag shown as a subtle colored pill (e.g., "Script", "Snippet", "App")
    /// Only shown during search mode to help distinguish mixed result types
    type_tag: Option<TypeTag>,
    /// Source/kit name (e.g., "main", "cleanshot") shown as subtle text during search
    source_hint: Option<String>,
    /// Tool/language badge for scriptlets (e.g., "ts", "bash", "paste")
    /// Shown as a subtle monospace badge in the accessories area
    tool_badge: Option<String>,
}

/// Type tag displayed as a colored pill on list items during search
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

    /// Set a type tag to show as a colored pill (e.g., "Script", "Snippet")
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

impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let index = self.index;
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
        // Icon text color matches the item's text color (primary when selected, secondary otherwise)
        let icon_text_color = if self.selected {
            rgb(colors.text_primary)
        } else {
            rgb(colors.text_secondary)
        };
        let icon_element = match &self.icon {
            Some(IconKind::Emoji(emoji)) => div()
                .w(px(20.))
                .h(px(20.))
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
                    .w(px(20.))
                    .h(px(20.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .child(
                        img(move |_window: &mut Window, _cx: &mut App| Some(Ok(image.clone())))
                            .w(px(20.))
                            .h(px(20.))
                            .object_fit(ObjectFit::Contain),
                    )
            }
            Some(IconKind::Svg(name)) => {
                // Convert string to IconName and render SVG
                // Use external_path() for file system SVGs (not path() which is for embedded assets)
                if let Some(icon_name) = icon_name_from_str(name) {
                    let svg_path = icon_name.external_path();
                    div()
                        .w(px(20.))
                        .h(px(20.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            svg()
                                .external_path(svg_path)
                                .size(px(16.))
                                .text_color(icon_text_color),
                        )
                } else {
                    // Fallback to Code icon if name not recognized
                    let svg_path = IconName::Code.external_path();
                    div()
                        .w(px(20.))
                        .h(px(20.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            svg()
                                .external_path(svg_path)
                                .size(px(16.))
                                .text_color(icon_text_color),
                        )
                }
            }
            None => {
                div().w(px(0.)).h(px(0.)) // No space if no icon
            }
        };

        // Build content with name + description (tighter spacing)
        let mut item_content = div()
            .flex_1()
            .min_w(px(0.))
            .overflow_hidden()
            .flex()
            .flex_col()
            .justify_center();

        // Name rendering - 15px font size, medium weight
        // When highlight_indices are present, use StyledText to highlight matched characters
        // Otherwise, render as plain text
        let name_element = if let Some(ref indices) = self.highlight_indices {
            // Build StyledText with highlighted matched characters
            let index_set: HashSet<usize> = indices.iter().copied().collect();
            let highlight_color = if self.selected {
                rgb(colors.accent_selected)
            } else {
                rgb(colors.text_primary)
            };
            let highlight_style = HighlightStyle {
                color: Some(highlight_color.into()),
                font_weight: Some(FontWeight::SEMIBOLD),
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
                rgb(colors.text_secondary)
            } else {
                rgb(colors.text_muted)
            };

            let styled = StyledText::new(self.name.to_string()).with_highlights(highlights);

            div()
                .text_size(px(15.))
                .font_weight(FontWeight::MEDIUM)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .line_height(px(20.))
                .text_color(base_color)
                .child(styled)
        } else {
            // Plain text rendering (no search active)
            div()
                .text_size(px(15.))
                .font_weight(FontWeight::MEDIUM)
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .line_height(px(20.))
                .child(self.name)
        };

        item_content = item_content.child(name_element);

        // Description - text_xs (0.75rem ≈ 12px), muted color (never changes on selection - only bg shows selection)
        // Single-line with ellipsis truncation for long content
        // When description_highlight_indices are present, matched characters are rendered with accent color
        if let Some(desc) = self.description {
            let desc_color = rgb(colors.text_muted);
            let desc_element = if let Some(ref desc_indices) = self.description_highlight_indices {
                // Build StyledText with highlighted matched characters in description
                let index_set: HashSet<usize> = desc_indices.iter().copied().collect();
                let highlight_color = rgb(colors.accent_selected);
                let highlight_style = HighlightStyle {
                    color: Some(highlight_color.into()),
                    font_weight: Some(FontWeight::SEMIBOLD),
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

                // Base text is more muted when highlighting to create contrast
                let base_color = rgba((colors.text_dimmed << 8) | 0xCC);
                let styled = StyledText::new(desc.clone()).with_highlights(highlights);

                div()
                    .text_xs()
                    .line_height(px(14.))
                    .text_color(base_color)
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(styled)
            } else {
                div()
                    .text_xs()
                    .line_height(px(14.))
                    .text_color(desc_color)
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(desc)
            };
            item_content = item_content.child(desc_element);
        }

        // Shortcut badge (if present) - right-aligned with kbd-style rendering
        // Uses macOS-native modifier symbols (⌘, ⇧, ⌥, ⌃) for a native feel
        let shortcut_element = if let Some(sc) = self.shortcut {
            let display_text = format_shortcut_display(&sc);
            let badge_border = (colors.text_dimmed << 8) | 0x50; // 31% opacity border
            div()
                .text_xs()
                .font_family("SF Mono")
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(colors.text_muted))
                .px(px(7.))
                .py(px(3.))
                .rounded(px(4.))
                .bg(rgba((colors.text_dimmed << 8) | 0x10))
                .border_1()
                .border_color(rgba(badge_border))
                .child(display_text)
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
        // Horizontal padding px(12.) and vertical padding py(2.) for compact spacing
        //
        // HOVER TRANSITIONS: We use GPUI's built-in .hover() modifier for instant visual
        // feedback on non-selected items. This provides CSS-like instant hover effects
        // without waiting for state updates via cx.notify().
        //
        // For selected items, we don't apply hover styles (they already have full focus styling).
        // Subtle bottom separator for better scanability between items
        // Very faint 1px border visible only on non-selected items to avoid clutter
        let separator_color = if self.selected {
            rgba(0x00000000) // No separator on selected item
        } else {
            rgba((colors.text_muted << 8) | 0x14) // ~8% opacity - subtle but scannable
        };

        let mut inner_content = div()
            .w_full()
            .h_full()
            .px(px(12.))
            .py(px(2.))
            .bg(bg_color)
            .border_b_1()
            .border_color(separator_color)
            .text_color(if self.selected {
                rgb(colors.text_primary)
            } else {
                rgb(colors.text_secondary)
            })
            .font_family(".AppleSystemUIFont")
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .child(icon_element)
            .child(item_content)
            .child({
                // Right-side accessories: [source hint] [type tag] [shortcut badge]
                let mut accessories = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .flex_shrink_0()
                    .gap(px(6.));

                // Tool/language badge for scriptlets (e.g., "ts", "bash")
                if let Some(ref badge) = self.tool_badge {
                    let badge_bg = (colors.text_dimmed << 8) | 0x0F; // 6% opacity
                    accessories = accessories.child(
                        div()
                            .text_size(px(9.))
                            .font_family("SF Mono")
                            .text_color(rgba((colors.text_dimmed << 8) | 0x90)) // 56% opacity
                            .px(px(4.))
                            .py(px(1.))
                            .rounded(px(3.))
                            .bg(rgba(badge_bg))
                            .child(badge.clone()),
                    );
                }

                // Source/kit hint (e.g., "main", "cleanshot") - very subtle
                if let Some(ref hint) = self.source_hint {
                    accessories = accessories.child(
                        div()
                            .text_size(px(10.))
                            .text_color(rgba((colors.text_dimmed << 8) | 0x80)) // 50% opacity
                            .child(hint.clone()),
                    );
                }

                // Type tag pill (shown during search to distinguish result types)
                if let Some(ref tag) = self.type_tag {
                    let tag_bg = (tag.color << 8) | 0x1A; // 10% opacity background
                    accessories = accessories.child(
                        div()
                            .text_size(px(10.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(tag.color))
                            .px(px(5.))
                            .py(px(1.))
                            .rounded(px(3.))
                            .bg(rgba(tag_bg))
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
            let element_idx = index.unwrap_or(0);
            ElementId::NamedInteger("list-item".into(), element_idx as u64)
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
            .pr(px(4.)) // Right padding only
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
        .expect("Failed to create image buffer");
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
/// - Small font (~10-11px via text_xs)
/// - Semi-bold weight (SEMIBOLD for subtlety)
/// - Dimmed color (subtle but readable)
/// - Compact vertical footprint within the 48px uniform_list row
/// - Large top padding to create visual compression (appears ~24px tall)
/// - Left-aligned with list item padding
/// - No background, no border
///
/// ## Technical Note: uniform_list Height Constraint
/// GPUI's `uniform_list` requires fixed heights for O(1) scroll calculation.
/// We cannot use actual variable heights. Instead, we use a visual trick:
/// - Actual height: 48px (LIST_ITEM_HEIGHT, for uniform_list)
/// - Visual height: ~24px (via top padding compression)
/// - Content is pushed to the bottom 24px of the container
///
/// This gives the appearance of 50% height while maintaining uniform_list compatibility.
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
    // Compact section header with explicit height (SECTION_HEADER_HEIGHT = 24px)
    // Used with GPUI's list() component which supports variable-height items.
    //
    // Layout: 24px total height
    // - pt(8px) top padding for visual separation from above item
    // - ~8px text height (text_xs)
    // - pb(4px) bottom padding for visual separation from below item

    // Parse label to separate name from count (e.g., "SUGGESTED · 5" → "SUGGESTED", "5")
    let (section_name, count_text) = if let Some(dot_pos) = label.find(" · ") {
        (&label[..dot_pos], Some(&label[dot_pos + " · ".len()..]))
    } else {
        (label, None)
    };

    // Build the inner content row: icon (optional) → section name → count (optional)
    // Use text_muted (0x808080) instead of text_dimmed (0xaaaaaa in light mode)
    // for better contrast against vibrancy backgrounds
    let mut content = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .text_size(px(11.0)) // slightly bigger than text_xs for readability
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(colors.text_muted));

    // Add icon before section name if provided
    if let Some(name) = icon {
        if let Some(icon_name) = icon_name_from_str(name) {
            content = content.child(
                svg()
                    .external_path(icon_name.external_path())
                    .size(px(10.))
                    .text_color(rgba((colors.text_muted << 8) | 0xC0)), // 75% opacity of muted
            );
        }
    }

    content = content.child(section_name.to_string());

    // Add count badge if present - rendered as a subtle separate element
    if let Some(count) = count_text {
        content = content.child(
            div()
                .text_xs()
                .font_weight(FontWeight::NORMAL)
                .text_color(rgba((colors.text_muted << 8) | 0xB0)) // 69% opacity of muted
                .child(count.to_string()),
        );
    }

    // Subtle background tint for section headers to create visual grouping
    let header_bg = rgba((colors.text_muted << 8) | 0x0A); // ~4% opacity tint

    let header = div()
        .w_full()
        .h(px(28.0)) // Slightly taller than SECTION_HEADER_HEIGHT for breathing room
        .px(px(16.))
        .pt(px(10.)) // More top padding for visual separation
        .pb(px(4.)) // Bottom padding
        .bg(header_bg)
        .flex()
        .flex_col()
        .justify_center(); // Center content vertically

    // Only show top separator on non-first headers (first header has no item above it)
    let header = if is_first {
        header
    } else {
        header
            .border_t_1()
            .border_color(rgba((colors.text_muted << 8) | 0x30)) // ~19% opacity - visible separator
    };

    header.child(content)
}

// Note: GPUI rendering tests omitted due to GPUI macro recursion limit issues.
// The LIST_ITEM_HEIGHT constant is 48.0 and the component is integration-tested
// via the main application's script list and arg prompt rendering.
// Unit tests for format_shortcut_display are in src/list_item_tests.rs.
