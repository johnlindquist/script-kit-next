//! Theme adapter for converting Script Kit themes to Alacritty colors.
//!
//! This module bridges Script Kit's theme system with Alacritty's color
//! configuration, ensuring the embedded terminal matches the application's
//! visual style.
//!
//! # Color Mapping
//!
//! Script Kit themes define colors for UI elements, which are mapped to
//! terminal ANSI colors:
//!
//! | Script Kit                    | Terminal Use              |
//! |-------------------------------|---------------------------|
//! | `background.main`             | Terminal background       |
//! | `text.primary`                | Default foreground        |
//! | `accent.selected`             | Cursor                    |
//! | `accent.selected_subtle`      | Selection background      |
//! | `text.secondary`              | Selection foreground      |
//!
//! # Focus-Aware Colors
//!
//! When the window is unfocused, colors are dimmed by blending toward gray
//! to provide visual feedback that the terminal is not active.

use vte::ansi::Rgb;

use crate::theme::Theme;

mod color_utils;
mod impls;
#[cfg(test)]
mod tests;

use color_utils::dim_color;
pub use color_utils::hex_to_rgb;

/// Standard ANSI colors - used as fallback/base for the 16-color palette.
///
/// These colors follow the standard ANSI color naming convention:
/// - Colors 0-7: Normal (black, red, green, yellow, blue, magenta, cyan, white)
/// - Colors 8-15: Bright variants of the above
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiColors {
    /// ANSI 0: Black
    pub black: Rgb,
    /// ANSI 1: Red
    pub red: Rgb,
    /// ANSI 2: Green
    pub green: Rgb,
    /// ANSI 3: Yellow
    pub yellow: Rgb,
    /// ANSI 4: Blue
    pub blue: Rgb,
    /// ANSI 5: Magenta
    pub magenta: Rgb,
    /// ANSI 6: Cyan
    pub cyan: Rgb,
    /// ANSI 7: White
    pub white: Rgb,
    /// ANSI 8: Bright Black (Gray)
    pub bright_black: Rgb,
    /// ANSI 9: Bright Red
    pub bright_red: Rgb,
    /// ANSI 10: Bright Green
    pub bright_green: Rgb,
    /// ANSI 11: Bright Yellow
    pub bright_yellow: Rgb,
    /// ANSI 12: Bright Blue
    pub bright_blue: Rgb,
    /// ANSI 13: Bright Magenta
    pub bright_magenta: Rgb,
    /// ANSI 14: Bright Cyan
    pub bright_cyan: Rgb,
    /// ANSI 15: Bright White
    pub bright_white: Rgb,
}

impl Default for AnsiColors {
    fn default() -> Self {
        Self {
            black: hex_to_rgb(0x000000),
            red: hex_to_rgb(0xcd3131),
            green: hex_to_rgb(0x0dbc79),
            yellow: hex_to_rgb(0xe5e510),
            blue: hex_to_rgb(0x2472c8),
            magenta: hex_to_rgb(0xbc3fbc),
            cyan: hex_to_rgb(0x11a8cd),
            white: hex_to_rgb(0xe5e5e5),
            bright_black: hex_to_rgb(0x666666),
            bright_red: hex_to_rgb(0xf14c4c),
            bright_green: hex_to_rgb(0x23d18b),
            bright_yellow: hex_to_rgb(0xf5f543),
            bright_blue: hex_to_rgb(0x3b8eea),
            bright_magenta: hex_to_rgb(0xd670d6),
            bright_cyan: hex_to_rgb(0x29b8db),
            bright_white: hex_to_rgb(0xffffff),
        }
    }
}

impl AnsiColors {
    /// Light mode ANSI colors matching VS Code light theme.
    ///
    /// These colors are designed for readability on light backgrounds.
    pub fn light_default() -> Self {
        Self {
            black: hex_to_rgb(0x000000),
            red: hex_to_rgb(0xcd3131),
            green: hex_to_rgb(0x00bc00),
            yellow: hex_to_rgb(0x949800),
            blue: hex_to_rgb(0x0451a5),
            magenta: hex_to_rgb(0xbc05bc),
            cyan: hex_to_rgb(0x0598bc),
            white: hex_to_rgb(0x555555),
            bright_black: hex_to_rgb(0x666666),
            bright_red: hex_to_rgb(0xcd3131),
            bright_green: hex_to_rgb(0x14ce14),
            bright_yellow: hex_to_rgb(0xb5ba00),
            bright_blue: hex_to_rgb(0x0451a5),
            bright_magenta: hex_to_rgb(0xbc05bc),
            bright_cyan: hex_to_rgb(0x0598bc),
            bright_white: hex_to_rgb(0xa5a5a5),
        }
    }

    /// Get an ANSI color by index (0-15).
    ///
    /// # Arguments
    ///
    /// * `index` - ANSI color index (0-15)
    ///
    /// # Returns
    ///
    /// The corresponding RGB color, or black if index is out of range.
    pub fn get(&self, index: u8) -> Rgb {
        match index {
            0 => self.black,
            1 => self.red,
            2 => self.green,
            3 => self.yellow,
            4 => self.blue,
            5 => self.magenta,
            6 => self.cyan,
            7 => self.white,
            8 => self.bright_black,
            9 => self.bright_red,
            10 => self.bright_green,
            11 => self.bright_yellow,
            12 => self.bright_blue,
            13 => self.bright_magenta,
            14 => self.bright_cyan,
            15 => self.bright_white,
            _ => self.black,
        }
    }

    /// Apply dimming factor to all colors for unfocused state.
    fn dimmed(&self, factor: f32) -> Self {
        Self {
            black: dim_color(self.black, factor),
            red: dim_color(self.red, factor),
            green: dim_color(self.green, factor),
            yellow: dim_color(self.yellow, factor),
            blue: dim_color(self.blue, factor),
            magenta: dim_color(self.magenta, factor),
            cyan: dim_color(self.cyan, factor),
            white: dim_color(self.white, factor),
            bright_black: dim_color(self.bright_black, factor),
            bright_red: dim_color(self.bright_red, factor),
            bright_green: dim_color(self.bright_green, factor),
            bright_yellow: dim_color(self.bright_yellow, factor),
            bright_blue: dim_color(self.bright_blue, factor),
            bright_magenta: dim_color(self.bright_magenta, factor),
            bright_cyan: dim_color(self.bright_cyan, factor),
            bright_white: dim_color(self.bright_white, factor),
        }
    }
}

/// Adapts Script Kit themes to terminal color schemes.
#[derive(Debug, Clone)]
pub struct ThemeAdapter {
    /// Foreground text color
    foreground: Rgb,
    /// Background color
    background: Rgb,
    /// Cursor color
    cursor: Rgb,
    /// Selection background color
    selection_background: Rgb,
    /// Selection foreground color
    selection_foreground: Rgb,
    /// The 16 ANSI colors
    ansi: AnsiColors,
    /// Whether the window is currently focused
    is_focused: bool,
    /// Original colors before focus dimming (for restoration)
    original_foreground: Rgb,
    original_background: Rgb,
    original_cursor: Rgb,
    original_selection_background: Rgb,
    original_selection_foreground: Rgb,
    original_ansi: AnsiColors,
}
