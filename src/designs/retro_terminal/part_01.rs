use gpui::*;

use super::{DesignRenderer, DesignVariant};
use crate::scripts::SearchResult;

/// Fixed height for terminal list items (dense terminal feel)
pub const TERMINAL_ITEM_HEIGHT: f32 = 28.0;

/// Phosphor green color (classic CRT green)
const PHOSPHOR_GREEN: u32 = 0x00ff00;

/// CRT black background
const CRT_BLACK: u32 = 0x000000;

/// Dimmed green for less prominent elements
const DIM_GREEN: u32 = 0x00aa00;

/// Very dim green for scanlines/borders
const SCANLINE_GREEN: u32 = 0x003300;

/// Pre-computed colors for terminal rendering
#[derive(Clone, Copy)]
pub struct TerminalColors {
    pub phosphor: u32,
    pub background: u32,
    pub dim: u32,
    pub scanline: u32,
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self {
            phosphor: PHOSPHOR_GREEN,
            background: CRT_BLACK,
            dim: DIM_GREEN,
            scanline: SCANLINE_GREEN,
        }
    }
}

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
            .font_family("Menlo")
            .text_sm()
            .text_color(text_color)
            .shadow(shadows)
            .child(display_text)
    }

    /// Render the search input with terminal prompt style
    pub fn render_search_input(&self, filter_text: &str, cursor_visible: bool) -> impl IntoElement {
        let colors = self.colors;

        // Terminal prompt: >_
        let prompt = ">_ ";

        // Build input display with blinking cursor
        let cursor = if cursor_visible { "█" } else { " " };
        let display_text = format!("{}{}{}", prompt, filter_text.to_uppercase(), cursor);

        div()
            .w_full()
            .px(px(8.))
            .py(px(8.))
            .bg(rgb(colors.background))
            .border_b_1()
            .border_color(rgb(colors.dim))
            .font_family("Menlo")
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

        // ASCII box top border: ┌────────────────────────────┐
        let border_line = "┌────────────────────────────────────────┐";
        let title_line = "│           SCRIPT-KIT TERMINAL          │";

        div()
            .w_full()
            .flex()
            .flex_col()
            .bg(rgb(colors.background))
            .font_family("Menlo")
            .text_xs()
            .text_color(rgb(colors.dim))
            .child(div().px(px(8.)).child(border_line))
            .child(
                div()
                    .px(px(8.))
                    .text_color(rgb(colors.phosphor))
                    .child(title_line),
            )
            .child(
                div()
                    .px(px(8.))
                    .child("├────────────────────────────────────────┤"),
            )
    }

    /// Render the terminal footer with ASCII box characters
    pub fn render_footer(&self, item_count: usize) -> impl IntoElement {
        let colors = self.colors;

        let status = format!("│ {} ITEMS LOADED                         │", item_count);
        let border_bottom = "└────────────────────────────────────────┘";

        div()
            .w_full()
            .flex()
            .flex_col()
            .bg(rgb(colors.background))
            .font_family("Menlo")
            .text_xs()
            .text_color(rgb(colors.dim))
            .child(
                div()
                    .px(px(8.))
                    .child("├────────────────────────────────────────┤"),
            )
            .child(div().px(px(8.)).child(status))
            .child(div().px(px(8.)).child(border_bottom))
    }

    /// Render empty state message
    pub fn render_empty_state(&self, filter_text: &str) -> impl IntoElement {
        let colors = self.colors;

        let message = if filter_text.is_empty() {
            "NO SCRIPTS FOUND".to_string()
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
            .font_family("Menlo")
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

impl<App> DesignRenderer<App> for RetroTerminalRenderer
where
    App: 'static,
{
    fn render_script_list(&self, _app: &App, _cx: &mut Context<App>) -> AnyElement {
        // Note: This is a placeholder implementation.
        // The actual integration requires access to app state (scripts, filter, selected_index).
        // For now, we return an empty terminal container.
        // The real implementation will be wired up when the main app integrates custom renderers.

        let colors = self.colors;

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(colors.background))
            .font_family("Menlo")
            .child(self.render_header())
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(colors.dim))
                    .child("INITIALIZING..."),
            )
            .child(self.render_footer(0))
            .into_any_element()
    }

    fn variant(&self) -> DesignVariant {
        DesignVariant::RetroTerminal
    }
}

/// Create a retro terminal renderer instance
pub fn create_renderer() -> RetroTerminalRenderer {
    RetroTerminalRenderer::new()
}

// ============================================================================
// Standalone Render Helper Functions
// ============================================================================

/// Terminal window container configuration
///
/// Returns styling properties for the terminal window wrapper.
/// Use this to apply consistent terminal aesthetic to the main container.
#[derive(Debug, Clone, Copy)]
pub struct TerminalWindowConfig {
    /// Background color (CRT black)
    pub background: u32,
    /// Border color (dim green)
    pub border: u32,
    /// Border width in pixels
    pub border_width: f32,
    /// Font family for all terminal text
    pub font_family: &'static str,
    /// Whether to show the CRT glow effect
    pub glow_enabled: bool,
    /// Glow color (phosphor green with alpha)
    pub glow_color: Hsla,
    /// Glow blur radius
    pub glow_blur: f32,
}

impl Default for TerminalWindowConfig {
    fn default() -> Self {
        Self {
            background: 0x0a0a0a, // Slightly off-black for CRT feel
            border: DIM_GREEN,
            border_width: 1.0,
            font_family: "Menlo",
            glow_enabled: true,
            glow_color: hsla(120.0 / 360.0, 1.0, 0.5, 0.15), // Subtle green glow
            glow_blur: 20.0,
        }
    }
}

/// Returns terminal window container configuration with CRT styling
///
/// Use this to wrap your main terminal UI with consistent styling:
/// - Black background (0x0a0a0a)
/// - Dim green border
/// - Monospace font (Menlo/SF Mono)
/// - Optional CRT glow effect
///
pub fn render_terminal_window_container() -> TerminalWindowConfig {
    TerminalWindowConfig::default()
}

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
    // Terminal prompt: >_
    let prompt = ">_ ";

    // Build input display with blinking block cursor
    let cursor = if cursor_visible { "█" } else { " " };
    let display_text = format!("{}{}{}", prompt, filter_text.to_uppercase(), cursor);

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
        .font_family("Menlo")
        .text_sm()
        .text_color(rgb(colors.phosphor))
        .shadow(glow_shadows)
        .child(display_text)
}
