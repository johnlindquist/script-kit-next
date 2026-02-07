use vte::ansi::{Color, NamedColor, Rgb};

use crate::terminal::theme_adapter::ThemeAdapter;

/// Resolve a terminal Color to an actual Rgb value using the theme adapter.
///
/// Terminal colors can be:
/// - Named colors (Foreground, Background, Red, Green, etc.)
/// - Indexed colors (0-255 palette)
/// - Spec colors (direct RGB values)
///
/// This function converts all of these to actual Rgb values using the
/// theme adapter for consistent theming.
pub fn resolve_color(color: &Color, theme: &ThemeAdapter) -> Rgb {
    match color {
        Color::Named(named) => resolve_named_color(*named, theme),
        Color::Indexed(index) => resolve_indexed_color(*index, theme),
        Color::Spec(rgb) => *rgb,
    }
}

/// Resolve foreground color, applying "bold as bright" behavior.
///
/// Many traditional terminals brighten normal ANSI colors (0-7) to their bright
/// variants (8-15) when the BOLD attribute is set. This improves visibility and
/// is the expected behavior for tools like `ls --color`, `git diff`, etc.
///
/// Only applies to:
/// - Named colors (Black, Red, Green, Yellow, Blue, Magenta, Cyan, White)
/// - Indexed colors 0-7
///
/// Does NOT apply to:
/// - Already-bright colors (indices 8-15)
/// - 216 color cube (indices 16-231)
/// - Grayscale (indices 232-255)
/// - Direct RGB (Spec colors)
/// - Background colors (use resolve_color for those)
pub fn resolve_fg_color_with_bold(color: &Color, is_bold: bool, theme: &ThemeAdapter) -> Rgb {
    if !is_bold {
        return resolve_color(color, theme);
    }

    match color {
        Color::Named(named) => resolve_named_color_brightened(*named, theme),
        Color::Indexed(index) if *index < 8 => theme.ansi_color(index + 8),
        _ => resolve_color(color, theme),
    }
}

/// Resolve a named color to Rgb, using bright variant for normal ANSI colors.
fn resolve_named_color_brightened(named: NamedColor, theme: &ThemeAdapter) -> Rgb {
    match named {
        NamedColor::Black => theme.ansi_color(8),
        NamedColor::Red => theme.ansi_color(9),
        NamedColor::Green => theme.ansi_color(10),
        NamedColor::Yellow => theme.ansi_color(11),
        NamedColor::Blue => theme.ansi_color(12),
        NamedColor::Magenta => theme.ansi_color(13),
        NamedColor::Cyan => theme.ansi_color(14),
        NamedColor::White => theme.ansi_color(15),
        other => resolve_named_color(other, theme),
    }
}

/// Resolve a named color to Rgb.
fn resolve_named_color(named: NamedColor, theme: &ThemeAdapter) -> Rgb {
    match named {
        NamedColor::Foreground | NamedColor::BrightForeground => theme.foreground(),
        NamedColor::Background => theme.background(),
        NamedColor::Cursor => theme.cursor(),

        NamedColor::Black => theme.ansi_color(0),
        NamedColor::Red => theme.ansi_color(1),
        NamedColor::Green => theme.ansi_color(2),
        NamedColor::Yellow => theme.ansi_color(3),
        NamedColor::Blue => theme.ansi_color(4),
        NamedColor::Magenta => theme.ansi_color(5),
        NamedColor::Cyan => theme.ansi_color(6),
        NamedColor::White => theme.ansi_color(7),

        NamedColor::BrightBlack => theme.ansi_color(8),
        NamedColor::BrightRed => theme.ansi_color(9),
        NamedColor::BrightGreen => theme.ansi_color(10),
        NamedColor::BrightYellow => theme.ansi_color(11),
        NamedColor::BrightBlue => theme.ansi_color(12),
        NamedColor::BrightMagenta => theme.ansi_color(13),
        NamedColor::BrightCyan => theme.ansi_color(14),
        NamedColor::BrightWhite => theme.ansi_color(15),

        NamedColor::DimBlack => dim_rgb(theme.ansi_color(0)),
        NamedColor::DimRed => dim_rgb(theme.ansi_color(1)),
        NamedColor::DimGreen => dim_rgb(theme.ansi_color(2)),
        NamedColor::DimYellow => dim_rgb(theme.ansi_color(3)),
        NamedColor::DimBlue => dim_rgb(theme.ansi_color(4)),
        NamedColor::DimMagenta => dim_rgb(theme.ansi_color(5)),
        NamedColor::DimCyan => dim_rgb(theme.ansi_color(6)),
        NamedColor::DimWhite => dim_rgb(theme.ansi_color(7)),
        NamedColor::DimForeground => dim_rgb(theme.foreground()),
    }
}

/// Resolve an indexed color (0-255) to Rgb.
///
/// The 256-color palette is organized as:
/// - 0-15: Standard ANSI colors
/// - 16-231: 6x6x6 color cube
/// - 232-255: 24 grayscale shades
fn resolve_indexed_color(index: u8, theme: &ThemeAdapter) -> Rgb {
    match index {
        0..=15 => theme.ansi_color(index),
        16..=231 => {
            let index = index - 16;
            let r = (index / 36) % 6;
            let g = (index / 6) % 6;
            let b = index % 6;

            let to_component = |v: u8| -> u8 {
                if v == 0 {
                    0
                } else {
                    55 + v * 40
                }
            };

            Rgb {
                r: to_component(r),
                g: to_component(g),
                b: to_component(b),
            }
        }
        232..=255 => {
            let shade = index - 232;
            let gray = 8 + shade * 10;
            Rgb {
                r: gray,
                g: gray,
                b: gray,
            }
        }
    }
}

/// Dim an RGB color (reduce intensity by ~30%).
fn dim_rgb(color: Rgb) -> Rgb {
    Rgb {
        r: (color.r as f32 * 0.7) as u8,
        g: (color.g as f32 * 0.7) as u8,
        b: (color.b as f32 * 0.7) as u8,
    }
}
