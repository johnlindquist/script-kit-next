use gpui::*;

use crate::scripts::SearchResult;

use super::colors::TerminalColors;
use super::constants::{
    ASCII_FOOTER_BORDER_BOTTOM, ASCII_HEADER_BORDER_TOP, ASCII_HEADER_TITLE_LINE,
    ASCII_SECTION_SEPARATOR, EMPTY_FILTER_MESSAGE, TERMINAL_CURSOR_HIDDEN, TERMINAL_CURSOR_VISIBLE,
    TERMINAL_FONT_FAMILY, TERMINAL_ITEM_HEIGHT, TERMINAL_PROMPT,
};

/// Retro Terminal design renderer
///
/// Implements a classic CRT terminal aesthetic with green phosphor text,
/// scanline effects, and ASCII box drawing characters.
pub struct RetroTerminalRenderer {
    colors: TerminalColors,
}

impl RetroTerminalRenderer {
    /// Create a new retro terminal renderer with default colors
    pub fn new() -> Self {
        Self {
            colors: TerminalColors::default(),
        }
    }

    /// Render a single terminal list item
    pub fn render_item(
        &self,
        result: &SearchResult,
        index: usize,
        is_selected: bool,
    ) -> impl IntoElement {
        let colors = self.colors;

        // Get name and convert to UPPERCASE for terminal aesthetic
        let name = result.name().to_uppercase();

        // Terminal-style item prefix
        let prefix = if is_selected { "> " } else { "  " };

        // Build the display text
        let display_text = format!("{}{}", prefix, name);

        // Determine colors based on selection (inverted when selected)
        let (text_color, bg_color) = if is_selected {
            (rgb(colors.background), rgb(colors.phosphor)) // Inverted: black on green
        } else {
            (rgb(colors.phosphor), rgb(colors.background)) // Normal: green on black
        };

        // Scanline effect: slightly darker background on odd rows
        let row_bg = if !is_selected && index % 2 == 1 {
            rgba((colors.scanline << 8) | 0x40) // Very subtle darker stripe
        } else {
            bg_color
        };

        // Create glow shadow for selected items
        let shadows = if is_selected {
            vec![BoxShadow {
                color: hsla(120.0 / 360.0, 1.0, 0.5, 0.6), // Green glow
                offset: point(px(0.), px(0.)),
                blur_radius: px(8.),
                spread_radius: px(0.),
            }]
        } else {
            vec![]
        };

        div()
            .id(ElementId::NamedInteger(
                "terminal-item".into(),
                index as u64,
            ))
            .w_full()
            .h(px(TERMINAL_ITEM_HEIGHT))
            .px(px(8.))
            .flex()
            .items_center()
            .bg(row_bg)
            .font_family(TERMINAL_FONT_FAMILY)
            .text_sm()
            .text_color(text_color)
            .shadow(shadows)
            .child(display_text)
    }

    /// Render the search input with terminal prompt style
    pub fn render_search_input(&self, filter_text: &str, cursor_visible: bool) -> impl IntoElement {
        let colors = self.colors;

        let cursor = if cursor_visible {
            TERMINAL_CURSOR_VISIBLE
        } else {
            TERMINAL_CURSOR_HIDDEN
        };
        let display_text = format!(
            "{}{}{}",
            TERMINAL_PROMPT,
            filter_text.to_uppercase(),
            cursor
        );

        div()
            .w_full()
            .px(px(8.))
            .py(px(8.))
            .bg(rgb(colors.background))
            .border_b_1()
            .border_color(rgb(colors.dim))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_sm()
            .text_color(rgb(colors.phosphor))
            .shadow(vec![BoxShadow {
                color: hsla(120.0 / 360.0, 1.0, 0.5, 0.3), // Subtle green glow
                offset: point(px(0.), px(0.)),
                blur_radius: px(4.),
                spread_radius: px(0.),
            }])
            .child(display_text)
    }

    /// Render the terminal header with ASCII box characters
    pub fn render_header(&self) -> impl IntoElement {
        let colors = self.colors;

        div()
            .w_full()
            .flex()
            .flex_col()
            .bg(rgb(colors.background))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_xs()
            .text_color(rgb(colors.dim))
            .child(div().px(px(8.)).child(ASCII_HEADER_BORDER_TOP))
            .child(
                div()
                    .px(px(8.))
                    .text_color(rgb(colors.phosphor))
                    .child(ASCII_HEADER_TITLE_LINE),
            )
            .child(div().px(px(8.)).child(ASCII_SECTION_SEPARATOR))
    }

    /// Render the terminal footer with ASCII box characters
    pub fn render_footer(&self, item_count: usize) -> impl IntoElement {
        let colors = self.colors;

        let status = format!("│ {} ITEMS LOADED                         │", item_count);

        div()
            .w_full()
            .flex()
            .flex_col()
            .bg(rgb(colors.background))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_xs()
            .text_color(rgb(colors.dim))
            .child(div().px(px(8.)).child(ASCII_SECTION_SEPARATOR))
            .child(div().px(px(8.)).child(status))
            .child(div().px(px(8.)).child(ASCII_FOOTER_BORDER_BOTTOM))
    }

    /// Render empty state message
    pub fn render_empty_state(&self, filter_text: &str) -> impl IntoElement {
        let colors = self.colors;

        let message = if filter_text.is_empty() {
            EMPTY_FILTER_MESSAGE.to_string()
        } else {
            format!("NO MATCH FOR '{}'", filter_text.to_uppercase())
        };

        div()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(colors.background))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_sm()
            .text_color(rgb(colors.dim))
            .child(message)
    }
}

impl Default for RetroTerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a retro terminal renderer instance
pub fn create_renderer() -> RetroTerminalRenderer {
    RetroTerminalRenderer::new()
}
