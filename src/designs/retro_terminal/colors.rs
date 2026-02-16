use gpui::{rgba, Hsla, Rgba};

use crate::theme::{get_cached_theme, Theme};
use crate::ui_foundation::{hex_to_hsla_with_alpha, hex_to_rgba_with_opacity};

use super::constants::TERMINAL_ITEM_HEIGHT;

/// Pre-computed colors for terminal rendering.
///
/// All values are derived from semantic theme tokens.
#[derive(Clone, Copy, Debug)]
pub struct TerminalColors {
    pub phosphor: u32,
    pub background: u32,
    pub dim: u32,
    pub scanline: u32,
    pub error: u32,
    pub warning: u32,
    pub glow: u32,
}

impl TerminalColors {
    /// Build terminal colors from semantic theme tokens.
    pub fn from_theme(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let terminal = &colors.terminal;

        let phosphor = terminal.foreground.unwrap_or(terminal.green);
        let background = terminal.background.unwrap_or(colors.background.log_panel);

        // Derive dim/scanline shades from the active phosphor + background pair,
        // so retro rendering adapts across light/dark themes.
        let dim = blend_hex(phosphor, background, 0.45);
        let scanline = blend_hex(phosphor, background, 0.80);

        Self {
            phosphor,
            background,
            dim,
            scanline,
            error: colors.ui.error,
            warning: colors.ui.warning,
            glow: terminal.bright_green,
        }
    }
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self::from_theme(&get_cached_theme())
    }
}

/// Apply theme-driven opacity to a hex color and return GPUI `Rgba`.
#[inline]
pub(crate) fn color_rgba(hex: u32, opacity: f32) -> Rgba {
    rgba(hex_to_rgba_with_opacity(hex, opacity))
}

/// Apply theme-driven opacity to a hex color and return GPUI `Hsla`.
#[inline]
pub(crate) fn color_hsla(hex: u32, opacity: f32) -> Hsla {
    hex_to_hsla_with_alpha(hex, opacity)
}

/// Get terminal design constants for external use.
pub struct TerminalConstants;

impl TerminalConstants {
    /// Item height for terminal list (dense: 28px)
    pub const fn item_height() -> f32 {
        TERMINAL_ITEM_HEIGHT
    }

    /// Phosphor color from the active theme.
    pub fn phosphor_green() -> u32 {
        TerminalColors::default().phosphor
    }

    /// Terminal background from the active theme.
    pub fn crt_black() -> u32 {
        TerminalColors::default().background
    }

    /// Dim terminal foreground from the active theme.
    pub fn dim_green() -> u32 {
        TerminalColors::default().dim
    }

    /// Glow color from the active theme.
    pub fn glow_green() -> u32 {
        TerminalColors::default().glow
    }
}

fn blend_hex(foreground: u32, background: u32, mix_to_background: f32) -> u32 {
    let mix_channel = |fg: u32, bg: u32| -> u32 {
        ((fg as f32 * (1.0 - mix_to_background)) + (bg as f32 * mix_to_background)).round() as u32
    };

    let fg_r = (foreground >> 16) & 255;
    let fg_g = (foreground >> 8) & 255;
    let fg_b = foreground & 255;

    let bg_r = (background >> 16) & 255;
    let bg_g = (background >> 8) & 255;
    let bg_b = background & 255;

    let out_r = mix_channel(fg_r, bg_r);
    let out_g = mix_channel(fg_g, bg_g);
    let out_b = mix_channel(fg_b, bg_b);

    (out_r << 16) | (out_g << 8) | out_b
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    fn assert_no_banned_color_literals(file_name: &str, source: &str) {
        let hsla_literal = Regex::new(r"hsla\(\s*[0-9]").expect("valid hsla literal regex");
        let rgba_literal = Regex::new(r"rgba\(\s*0x").expect("valid rgba literal regex");
        let rgb_literal = Regex::new(r"rgb\(\s*0x").expect("valid rgb literal regex");
        let hex_literal = Regex::new(r"0x[0-9a-fA-F]{6,8}").expect("valid hex literal regex");
        let css_hex = Regex::new(r"#[0-9a-fA-F]{3,8}").expect("valid css hex regex");

        assert!(
            !hsla_literal.is_match(source),
            "{file_name} contains hardcoded hsla(...) color literals"
        );
        assert!(
            !rgba_literal.is_match(source),
            "{file_name} contains hardcoded rgba(0x...) color literals"
        );
        assert!(
            !rgb_literal.is_match(source),
            "{file_name} contains hardcoded rgb(0x...) color literals"
        );
        assert!(
            !hex_literal.is_match(source),
            "{file_name} contains hardcoded 0xRRGGBB color literals"
        );
        assert!(
            !css_hex.is_match(source),
            "{file_name} contains hardcoded #RRGGBB color literals"
        );
    }

    #[test]
    fn test_retro_terminal_sources_do_not_use_hardcoded_color_literals() {
        let sources = [
            ("window.rs", include_str!("window.rs")),
            ("render.rs", include_str!("render.rs")),
            ("renderer.rs", include_str!("renderer.rs")),
        ];

        for (file_name, source) in sources {
            assert_no_banned_color_literals(file_name, source);
        }
    }
}
