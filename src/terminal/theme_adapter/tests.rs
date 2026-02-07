use super::*;
use crate::theme::{ColorScheme, Theme};

#[test]
fn test_hex_to_rgb_white() {
    let rgb = hex_to_rgb(0xffffff);
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 255);
    assert_eq!(rgb.b, 255);
}

#[test]
fn test_hex_to_rgb_black() {
    let rgb = hex_to_rgb(0x000000);
    assert_eq!(rgb.r, 0);
    assert_eq!(rgb.g, 0);
    assert_eq!(rgb.b, 0);
}

#[test]
fn test_hex_to_rgb_red() {
    let rgb = hex_to_rgb(0xff0000);
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 0);
    assert_eq!(rgb.b, 0);
}

#[test]
fn test_hex_to_rgb_green() {
    let rgb = hex_to_rgb(0x00ff00);
    assert_eq!(rgb.r, 0);
    assert_eq!(rgb.g, 255);
    assert_eq!(rgb.b, 0);
}

#[test]
fn test_hex_to_rgb_blue() {
    let rgb = hex_to_rgb(0x0000ff);
    assert_eq!(rgb.r, 0);
    assert_eq!(rgb.g, 0);
    assert_eq!(rgb.b, 255);
}

#[test]
fn test_hex_to_rgb_vscode_dark_bg() {
    let rgb = hex_to_rgb(0x1e1e1e);
    assert_eq!(rgb.r, 0x1e);
    assert_eq!(rgb.g, 0x1e);
    assert_eq!(rgb.b, 0x1e);
}

#[test]
fn test_ansi_colors_default() {
    let ansi = AnsiColors::default();
    assert_eq!(ansi.black, hex_to_rgb(0x000000));
    assert_eq!(ansi.bright_white, hex_to_rgb(0xffffff));
}

#[test]
fn test_ansi_colors_get_normal_range() {
    let ansi = AnsiColors::default();
    assert_eq!(ansi.get(0), ansi.black);
    assert_eq!(ansi.get(1), ansi.red);
    assert_eq!(ansi.get(2), ansi.green);
    assert_eq!(ansi.get(3), ansi.yellow);
    assert_eq!(ansi.get(4), ansi.blue);
    assert_eq!(ansi.get(5), ansi.magenta);
    assert_eq!(ansi.get(6), ansi.cyan);
    assert_eq!(ansi.get(7), ansi.white);
}

#[test]
fn test_ansi_colors_get_bright_range() {
    let ansi = AnsiColors::default();
    assert_eq!(ansi.get(8), ansi.bright_black);
    assert_eq!(ansi.get(9), ansi.bright_red);
    assert_eq!(ansi.get(10), ansi.bright_green);
    assert_eq!(ansi.get(11), ansi.bright_yellow);
    assert_eq!(ansi.get(12), ansi.bright_blue);
    assert_eq!(ansi.get(13), ansi.bright_magenta);
    assert_eq!(ansi.get(14), ansi.bright_cyan);
    assert_eq!(ansi.get(15), ansi.bright_white);
}

#[test]
fn test_ansi_colors_get_out_of_range() {
    let ansi = AnsiColors::default();
    assert_eq!(ansi.get(16), ansi.black);
    assert_eq!(ansi.get(255), ansi.black);
}

#[test]
fn test_ansi_colors_dimmed() {
    let ansi = AnsiColors::default();
    let dimmed = ansi.dimmed(0.5);

    assert!(dimmed.bright_white.r < 255);
    assert!(dimmed.bright_white.r > 128);
}

#[test]
fn test_dark_default_colors() {
    let adapter = ThemeAdapter::dark_default();
    assert_eq!(adapter.background(), hex_to_rgb(0x1e1e1e));
    assert_eq!(adapter.foreground(), hex_to_rgb(0xd4d4d4));
    assert_eq!(adapter.cursor(), hex_to_rgb(0xffffff));
}

#[test]
fn test_dark_default_is_focused() {
    let adapter = ThemeAdapter::dark_default();
    assert!(adapter.is_focused());
}

#[test]
fn test_from_theme_maps_colors() {
    let theme = Theme::default();
    let adapter = ThemeAdapter::from_theme(&theme);

    assert_eq!(adapter.foreground(), hex_to_rgb(theme.colors.text.primary));
    assert_eq!(
        adapter.background(),
        hex_to_rgb(theme.colors.background.main)
    );
    assert_eq!(adapter.cursor(), hex_to_rgb(theme.colors.accent.selected));
    assert_eq!(
        adapter.selection_background(),
        hex_to_rgb(theme.colors.accent.selected_subtle)
    );
    assert_eq!(
        adapter.selection_foreground(),
        hex_to_rgb(theme.colors.text.secondary)
    );
}

#[test]
fn test_from_theme_uses_terminal_colors_for_ansi() {
    let theme = Theme::default();
    let adapter = ThemeAdapter::from_theme(&theme);

    assert_eq!(adapter.ansi_color(1), hex_to_rgb(theme.colors.terminal.red));
    assert_eq!(
        adapter.ansi_color(2),
        hex_to_rgb(theme.colors.terminal.green)
    );
    assert_eq!(
        adapter.ansi_color(3),
        hex_to_rgb(theme.colors.terminal.yellow)
    );
    assert_eq!(
        adapter.ansi_color(4),
        hex_to_rgb(theme.colors.terminal.blue)
    );
}

#[test]
fn test_ansi_color_returns_correct_colors() {
    let adapter = ThemeAdapter::dark_default();
    for i in 0..16 {
        let _color = adapter.ansi_color(i);
    }
}

#[test]
fn test_update_for_focus_dims_colors() {
    let mut adapter = ThemeAdapter::dark_default();
    let original_fg = adapter.foreground();

    adapter.update_for_focus(false);

    let dimmed_fg = adapter.foreground();
    assert_ne!(original_fg, dimmed_fg);
    assert!(!adapter.is_focused());
}

#[test]
fn test_update_for_focus_restores_colors() {
    let mut adapter = ThemeAdapter::dark_default();
    let original_fg = adapter.foreground();
    let original_bg = adapter.background();

    adapter.update_for_focus(false);
    adapter.update_for_focus(true);

    assert_eq!(adapter.foreground(), original_fg);
    assert_eq!(adapter.background(), original_bg);
    assert!(adapter.is_focused());
}

#[test]
fn test_update_for_focus_noop_when_unchanged() {
    let mut adapter = ThemeAdapter::dark_default();
    let original_fg = adapter.foreground();

    adapter.update_for_focus(true);

    assert_eq!(adapter.foreground(), original_fg);
}

#[test]
fn test_update_for_focus_dims_ansi_colors() {
    let mut adapter = ThemeAdapter::dark_default();
    let original_red = adapter.ansi_color(1);

    adapter.update_for_focus(false);

    let dimmed_red = adapter.ansi_color(1);
    assert_ne!(original_red, dimmed_red);
}

#[test]
fn test_default_is_dark_default() {
    let default_adapter = ThemeAdapter::default();
    let dark_adapter = ThemeAdapter::dark_default();

    assert_eq!(default_adapter.foreground(), dark_adapter.foreground());
    assert_eq!(default_adapter.background(), dark_adapter.background());
    assert_eq!(default_adapter.cursor(), dark_adapter.cursor());
}

#[test]
fn test_dim_color_full_gray() {
    let white = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    let dimmed = dim_color(white, 0.0);
    assert_eq!(dimmed.r, 0x80);
    assert_eq!(dimmed.g, 0x80);
    assert_eq!(dimmed.b, 0x80);
}

#[test]
fn test_dim_color_no_change() {
    let color = Rgb {
        r: 100,
        g: 150,
        b: 200,
    };
    let dimmed = dim_color(color, 1.0);
    assert_eq!(dimmed.r, 100);
    assert_eq!(dimmed.g, 150);
    assert_eq!(dimmed.b, 200);
}

#[test]
fn test_dim_color_half_blend() {
    let white = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    let dimmed = dim_color(white, 0.5);
    assert!((dimmed.r as i32 - 191).abs() <= 1);
}

#[test]
fn test_light_theme_adapter() {
    let theme = Theme {
        colors: ColorScheme::light_default(),
        ..Default::default()
    };
    let adapter = ThemeAdapter::from_theme(&theme);

    assert_eq!(
        adapter.background(),
        hex_to_rgb(theme.colors.background.main)
    );
    assert_eq!(adapter.foreground(), hex_to_rgb(theme.colors.text.primary));
}

#[test]
fn test_focus_cycle() {
    let mut adapter = ThemeAdapter::dark_default();

    assert!(adapter.is_focused());

    adapter.update_for_focus(false);
    assert!(!adapter.is_focused());

    adapter.update_for_focus(true);
    assert!(adapter.is_focused());

    adapter.update_for_focus(false);
    assert!(!adapter.is_focused());

    adapter.update_for_focus(true);
    assert!(adapter.is_focused());
}

#[test]
fn test_light_default_colors() {
    let adapter = ThemeAdapter::light_default();
    assert_eq!(adapter.foreground(), hex_to_rgb(0x000000));
    assert_eq!(adapter.background(), hex_to_rgb(0xf5f5f5));
    assert_eq!(adapter.cursor(), hex_to_rgb(0x000000));
}

#[test]
fn test_light_default_is_focused() {
    let adapter = ThemeAdapter::light_default();
    assert!(adapter.is_focused());
}

#[test]
fn test_ansi_colors_light_default() {
    let ansi = AnsiColors::light_default();
    assert_eq!(ansi.black, hex_to_rgb(0x000000));
    assert_eq!(ansi.white, hex_to_rgb(0x555555));
}

#[test]
fn test_light_default_selection_contrast() {
    let adapter = ThemeAdapter::light_default();
    assert_eq!(adapter.selection_background(), hex_to_rgb(0x0078d4));
    assert_eq!(adapter.selection_foreground(), hex_to_rgb(0xffffff));
}
