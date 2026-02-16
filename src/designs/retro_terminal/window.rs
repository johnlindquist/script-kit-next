use gpui::Hsla;

use super::colors::{color_hsla, TerminalColors};
use super::constants::TERMINAL_FONT_FAMILY;

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
        let colors = TerminalColors::default();

        Self {
            background: colors.background,
            border: colors.dim,
            border_width: 1.0,
            font_family: TERMINAL_FONT_FAMILY,
            glow_enabled: true,
            glow_color: color_hsla(colors.glow, 0.15),
            glow_blur: 20.0,
        }
    }
}

/// Returns terminal window container configuration with CRT styling
///
/// Use this to wrap your main terminal UI with consistent styling:
/// - Theme terminal background
/// - Theme-derived dim border
/// - Monospace font (Menlo/SF Mono)
/// - Optional CRT glow effect
pub fn render_terminal_window_container() -> TerminalWindowConfig {
    TerminalWindowConfig::default()
}
