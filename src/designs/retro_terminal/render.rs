use gpui::*;

use crate::scripts::SearchResult;

use super::colors::TerminalColors;
use super::constants::{
    ASCII_EMPTY_BORDER_BOTTOM, ASCII_EMPTY_BORDER_TOP, ASCII_LOG_HEADER, ASCII_PREVIEW_FOOTER,
    ASCII_PREVIEW_HEADER, TERMINAL_CURSOR_HIDDEN, TERMINAL_CURSOR_VISIBLE, TERMINAL_FONT_FAMILY,
    TERMINAL_PROMPT,
};
use super::renderer::RetroTerminalRenderer;

/// Render the terminal header/search bar with command prompt style
///
/// Displays a classic terminal prompt with `>_` prefix.
/// Shows filter text in UPPERCASE with optional blinking block cursor.
///
/// # Arguments
///
/// * `filter_text` - Current search/filter text
/// * `cursor_visible` - Whether the blinking cursor should be visible
/// * `colors` - Terminal color scheme
///
/// # Returns
///
/// A styled div element representing the terminal command prompt
pub fn render_terminal_header(
    filter_text: &str,
    cursor_visible: bool,
    colors: TerminalColors,
) -> impl IntoElement {
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

    // Create green glow shadow for the header
    let glow_shadows = vec![BoxShadow {
        color: hsla(120.0 / 360.0, 1.0, 0.5, 0.3), // Subtle green glow
        offset: point(px(0.), px(0.)),
        blur_radius: px(4.),
        spread_radius: px(0.),
    }];

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
        .shadow(glow_shadows)
        .child(display_text)
}

/// Render the terminal preview panel for code/content display
///
/// Displays content with classic terminal aesthetics:
/// - Green phosphor text on black background
/// - Monospace font
/// - Optional line numbers
/// - CRT-style glow effect
///
/// # Arguments
///
/// * `content` - The text content to display (can be code, text, etc.)
/// * `colors` - Terminal color scheme
///
/// # Returns
///
/// A styled div element representing the preview panel
pub fn render_terminal_preview_panel(content: &str, colors: TerminalColors) -> impl IntoElement {
    // Split content into lines for rendering
    let lines: Vec<&str> = content.lines().collect();

    // Create glow effect for the panel
    let panel_glow = vec![BoxShadow {
        color: hsla(120.0 / 360.0, 1.0, 0.5, 0.1), // Very subtle green glow
        offset: point(px(0.), px(0.)),
        blur_radius: px(12.),
        spread_radius: px(0.),
    }];

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(colors.background))
        .border_l_1()
        .border_color(rgb(colors.scanline))
        .font_family(TERMINAL_FONT_FAMILY)
        .text_xs()
        .shadow(panel_glow)
        .child(
            // Header bar
            div()
                .w_full()
                .px(px(8.))
                .py(px(4.))
                .border_b_1()
                .border_color(rgb(colors.scanline))
                .text_color(rgb(colors.dim))
                .child(ASCII_PREVIEW_HEADER),
        )
        .child(
            // Content area with line numbers
            div()
                .flex_1()
                .w_full()
                .overflow_hidden()
                .px(px(8.))
                .py(px(4.))
                .children(lines.into_iter().enumerate().map(|(line_num, line)| {
                    // Line number + content
                    let line_prefix = format!("{:4} │ ", line_num + 1);
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .child(
                            // Line number (dim)
                            div().text_color(rgb(colors.scanline)).child(line_prefix),
                        )
                        .child(
                            // Line content (bright green)
                            div()
                                .text_color(rgb(colors.phosphor))
                                .child(line.to_string()),
                        )
                })),
        )
        .child(
            // Footer bar
            div()
                .w_full()
                .px(px(8.))
                .py(px(4.))
                .border_t_1()
                .border_color(rgb(colors.scanline))
                .text_color(rgb(colors.dim))
                .child(ASCII_PREVIEW_FOOTER),
        )
}

/// Render the terminal log panel
///
/// Displays log entries with classic terminal aesthetics:
/// - Green text on black background
/// - Monospace font throughout
/// - Alternating row colors for scanline effect
/// - Log level indicators (INFO, WARN, ERR)
///
/// # Arguments
///
/// * `logs` - Vector of log entry strings
/// * `colors` - Terminal color scheme
///
/// # Returns
///
/// A styled div element representing the log panel
pub fn render_terminal_log_panel(logs: &[String], colors: TerminalColors) -> impl IntoElement {
    // Create glow effect for the panel
    let panel_glow = vec![BoxShadow {
        color: hsla(120.0 / 360.0, 1.0, 0.5, 0.08), // Very subtle green glow
        offset: point(px(0.), px(0.)),
        blur_radius: px(8.),
        spread_radius: px(0.),
    }];

    div()
        .w_full()
        .flex()
        .flex_col()
        .bg(rgb(colors.background))
        .border_t_1()
        .border_color(rgb(colors.dim))
        .font_family(TERMINAL_FONT_FAMILY)
        .text_xs()
        .shadow(panel_glow)
        .child(
            // Header bar
            div()
                .w_full()
                .px(px(8.))
                .py(px(2.))
                .border_b_1()
                .border_color(rgb(colors.scanline))
                .text_color(rgb(colors.dim))
                .child(ASCII_LOG_HEADER),
        )
        .child(
            // Log entries
            div()
                .w_full()
                .overflow_hidden()
                .max_h(px(150.))
                .children(logs.iter().enumerate().map(|(index, log_entry)| {
                    // Determine log level and color from content
                    let (level_indicator, text_color) = if log_entry.contains("[ERR]")
                        || log_entry.contains("ERROR")
                        || log_entry.contains("error")
                    {
                        ("█", rgb(0xff4444)) // Red for errors
                    } else if log_entry.contains("[WARN]")
                        || log_entry.contains("WARNING")
                        || log_entry.contains("warn")
                    {
                        ("▒", rgb(0xffff00)) // Yellow for warnings
                    } else {
                        ("░", rgb(colors.phosphor)) // Green for info
                    };

                    // Scanline effect: slightly darker on odd rows
                    let row_bg = if index % 2 == 1 {
                        rgba((colors.scanline << 8) | 0x20)
                    } else {
                        rgb(colors.background)
                    };

                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(1.))
                        .bg(row_bg)
                        .flex()
                        .flex_row()
                        .gap(px(4.))
                        .child(
                            // Level indicator
                            div().text_color(text_color).child(level_indicator),
                        )
                        .child(
                            // Log content
                            div()
                                .flex_1()
                                .text_color(text_color)
                                .overflow_hidden()
                                .child(log_entry.clone()),
                        )
                })),
        )
}

/// Render an empty terminal state with retro messaging
pub fn render_terminal_empty_state(message: &str, colors: TerminalColors) -> impl IntoElement {
    let display_message = message.to_uppercase();

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .bg(rgb(colors.background))
        .font_family(TERMINAL_FONT_FAMILY)
        .text_sm()
        .gap(px(8.))
        .child(
            div()
                .text_color(rgb(colors.dim))
                .child(ASCII_EMPTY_BORDER_TOP),
        )
        .child(
            div()
                .text_color(rgb(colors.phosphor))
                .child(format!("│  {}  │", display_message)),
        )
        .child(
            div()
                .text_color(rgb(colors.dim))
                .child(ASCII_EMPTY_BORDER_BOTTOM),
        )
}

/// Terminal list rendering helper
///
/// Renders a list of search results in full terminal style.
/// Use this with uniform_list for virtualized rendering.
pub fn render_terminal_list(
    results: &[SearchResult],
    selected_index: usize,
    colors: TerminalColors,
) -> impl IntoElement {
    let renderer = RetroTerminalRenderer::new();

    div()
        .w_full()
        .h_full()
        .bg(rgb(colors.background))
        .flex()
        .flex_col()
        .font_family(TERMINAL_FONT_FAMILY)
        .children(results.iter().enumerate().map(|(index, result)| {
            let is_selected = index == selected_index;
            renderer.render_item(result, index, is_selected)
        }))
}
