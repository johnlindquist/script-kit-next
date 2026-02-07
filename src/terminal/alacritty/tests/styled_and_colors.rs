use super::*;
use vte::ansi::{Color, NamedColor, Rgb};

#[test]
fn test_terminal_cell_default() {
    let cell = TerminalCell::default();
    assert_eq!(cell.c, ' ');
    assert_eq!(cell.attrs, CellAttributes::empty());
}

#[test]
fn test_cell_attributes_bitflags() {
    let mut attrs = CellAttributes::empty();
    assert!(!attrs.contains(CellAttributes::BOLD));
    assert!(!attrs.contains(CellAttributes::ITALIC));
    assert!(!attrs.contains(CellAttributes::UNDERLINE));

    attrs.insert(CellAttributes::BOLD);
    assert!(attrs.contains(CellAttributes::BOLD));

    attrs.insert(CellAttributes::ITALIC);
    assert!(attrs.contains(CellAttributes::BOLD | CellAttributes::ITALIC));
}

#[test]
fn test_terminal_content_styled_lines() {
    let content = TerminalContent {
        lines: vec!["hello".to_string()],
        styled_lines: vec![vec![
            TerminalCell {
                c: 'h',
                fg: Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                bg: Rgb { r: 0, g: 0, b: 0 },
                attrs: CellAttributes::empty(),
            },
            TerminalCell {
                c: 'e',
                fg: Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                bg: Rgb { r: 0, g: 0, b: 0 },
                attrs: CellAttributes::empty(),
            },
            TerminalCell {
                c: 'l',
                fg: Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                bg: Rgb { r: 0, g: 0, b: 0 },
                attrs: CellAttributes::empty(),
            },
            TerminalCell {
                c: 'l',
                fg: Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                bg: Rgb { r: 0, g: 0, b: 0 },
                attrs: CellAttributes::empty(),
            },
            TerminalCell {
                c: 'o',
                fg: Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                bg: Rgb { r: 0, g: 0, b: 0 },
                attrs: CellAttributes::empty(),
            },
        ]],
        cursor_line: 0,
        cursor_col: 5,
        selected_cells: vec![],
    };
    assert_eq!(content.styled_lines.len(), 1);
    assert_eq!(content.styled_lines[0].len(), 5);
    assert_eq!(content.styled_lines[0][0].c, 'h');
}

#[test]
fn test_terminal_content_lines_plain_backward_compat() {
    let content = TerminalContent {
        lines: vec!["hello".to_string(), "world".to_string()],
        styled_lines: vec![],
        cursor_line: 0,
        cursor_col: 0,
        selected_cells: vec![],
    };
    let plain = content.lines_plain();
    assert_eq!(plain.len(), 2);
    assert_eq!(plain[0], "hello");
    assert_eq!(plain[1], "world");
}

#[test]
fn test_resolve_color_named_foreground() {
    let theme = ThemeAdapter::dark_default();
    let color = Color::Named(NamedColor::Foreground);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(resolved, theme.foreground());
}

#[test]
fn test_resolve_color_named_background() {
    let theme = ThemeAdapter::dark_default();
    let color = Color::Named(NamedColor::Background);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(resolved, theme.background());
}

#[test]
fn test_resolve_color_named_ansi_red() {
    let theme = ThemeAdapter::dark_default();
    let color = Color::Named(NamedColor::Red);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(resolved, theme.ansi_color(1));
}

#[test]
fn test_resolve_color_indexed() {
    let theme = ThemeAdapter::dark_default();
    let color = Color::Indexed(4);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(resolved, theme.ansi_color(4));
}

#[test]
fn test_resolve_color_indexed_216_cube() {
    let theme = ThemeAdapter::dark_default();

    let color = Color::Indexed(16);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(resolved, Rgb { r: 0, g: 0, b: 0 });

    let color = Color::Indexed(231);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(
        resolved,
        Rgb {
            r: 255,
            g: 255,
            b: 255
        }
    );
}

#[test]
fn test_resolve_color_indexed_grayscale() {
    let theme = ThemeAdapter::dark_default();

    let color = Color::Indexed(232);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(resolved, Rgb { r: 8, g: 8, b: 8 });

    let color = Color::Indexed(255);
    let resolved = resolve_color(&color, &theme);
    assert_eq!(
        resolved,
        Rgb {
            r: 238,
            g: 238,
            b: 238
        }
    );
}

#[test]
fn test_resolve_color_spec_direct() {
    let theme = ThemeAdapter::dark_default();

    let color = Color::Spec(Rgb {
        r: 128,
        g: 64,
        b: 32,
    });
    let resolved = resolve_color(&color, &theme);
    assert_eq!(
        resolved,
        Rgb {
            r: 128,
            g: 64,
            b: 32
        }
    );
}
