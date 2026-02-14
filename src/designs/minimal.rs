//! Minimal Design Renderer
//!
//! Ultra minimalist design with maximum whitespace and NO visual noise.
//!
//! Design principles:
//! - Maximum whitespace with generous padding (80px horizontal, 24px vertical)
//! - Thin sans-serif typography (.AppleSystemUIFont)
//! - NO borders anywhere
//! - Subtle hover states (slight opacity change only)
//! - Monochrome palette with single accent color
//! - Full-width list (no preview panel)
//! - Search bar is just cursor + typed text, no box
//! - Items show name only (no description)
//! - Taller items (64px instead of 52px)

use gpui::*;

use crate::scripts::SearchResult;

/// Height for minimal design items (taller than default 52px)
pub const MINIMAL_ITEM_HEIGHT: f32 = 64.0;

/// Horizontal padding for list items
pub const HORIZONTAL_PADDING: f32 = 80.0;

/// Vertical padding for list items
pub const VERTICAL_PADDING: f32 = 24.0;

/// Pre-computed colors for minimal list item rendering
#[derive(Clone, Copy)]
pub struct MinimalColors {
    pub text_primary: u32,
    pub accent_selected: u32,
}

/// Minimal design renderer
///
/// Provides an ultra-clean, whitespace-focused UI with:
/// - No borders or dividers
/// - Simple text-only list items
/// - Accent color for selected items
/// - Generous padding throughout
pub struct MinimalRenderer;

impl MinimalRenderer {
    /// Create a new minimal renderer
    pub fn new() -> Self {
        Self
    }

    /// Render a single list item in minimal style
    pub fn render_item(
        &self,
        result: &SearchResult,
        index: usize,
        is_selected: bool,
        colors: MinimalColors,
    ) -> impl IntoElement {
        // Get name only (no description in minimal design)
        let name = result.name().to_string();

        // Text color: accent when selected, primary otherwise
        let text_color = if is_selected {
            rgb(colors.accent_selected)
        } else {
            rgb(colors.text_primary)
        };

        // Font weight: normal when selected, thin otherwise
        let font_weight = if is_selected {
            FontWeight::NORMAL
        } else {
            FontWeight::THIN
        };

        div()
            .id(ElementId::NamedInteger("minimal-item".into(), index as u64))
            .w_full()
            .h(px(MINIMAL_ITEM_HEIGHT))
            .px(px(HORIZONTAL_PADDING))
            .py(px(VERTICAL_PADDING / 2.0))
            .flex()
            .items_center()
            .font_family(".AppleSystemUIFont")
            .font_weight(font_weight)
            .text_base()
            .text_color(text_color)
            .cursor_pointer()
            // Subtle hover: just slightly brighter
            .hover(|s| s.opacity(0.8))
            .child(name)
    }
}

impl Default for MinimalRenderer {
    fn default() -> Self {
        Self::new()
    }
}
