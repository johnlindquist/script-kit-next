use super::super::*;
use serde::{Deserialize, Serialize};

#[test]
fn test_hex_color_parse_hash_prefix() {
    let result = hex_color_serde::parse_color_string("#FBBF24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_lowercase() {
    let result = hex_color_serde::parse_color_string("#fbbf24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_0x_prefix() {
    let result = hex_color_serde::parse_color_string("0xFBBF24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_bare_hex() {
    let result = hex_color_serde::parse_color_string("FBBF24");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_rgb() {
    let result = hex_color_serde::parse_color_string("rgb(251, 191, 36)");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_rgba() {
    let result = hex_color_serde::parse_color_string("rgba(251, 191, 36, 1.0)");
    assert_eq!(result.unwrap(), 0xFBBF24);
}

#[test]
fn test_hex_color_parse_black() {
    assert_eq!(
        hex_color_serde::parse_color_string("#000000").unwrap(),
        0x000000
    );
    assert_eq!(
        hex_color_serde::parse_color_string("rgb(0, 0, 0)").unwrap(),
        0x000000
    );
}

#[test]
fn test_hex_color_parse_white() {
    assert_eq!(
        hex_color_serde::parse_color_string("#FFFFFF").unwrap(),
        0xFFFFFF
    );
    assert_eq!(
        hex_color_serde::parse_color_string("rgb(255, 255, 255)").unwrap(),
        0xFFFFFF
    );
}

#[test]
fn test_hex_color_parse_shorthand_rgb() {
    assert_eq!(
        hex_color_serde::parse_color_string("#FFF").expect("expected #FFF to parse"),
        0xFFFFFF
    );
    assert_eq!(
        hex_color_serde::parse_color_string("FFF").expect("expected FFF to parse"),
        0xFFFFFF
    );
    assert_eq!(
        hex_color_serde::parse_color_string("#1e1").expect("expected #1e1 to parse"),
        0x11EE11
    );
    assert_eq!(
        hex_color_serde::parse_color_string("#000").expect("expected #000 to parse"),
        0x000000
    );
}

#[test]
fn test_hex_color_parse_shorthand_rgba_ignores_alpha() {
    assert_eq!(
        hex_color_serde::parse_color_string("#FFFF").expect("expected #FFFF to parse"),
        0xFFFFFF
    );
    assert_eq!(
        hex_color_serde::parse_color_string("#FFFA").expect("expected #FFFA to parse"),
        0xFFFFFF
    );
    assert_eq!(
        hex_color_serde::parse_color_string("#0000").expect("expected #0000 to parse"),
        0x000000
    );
}

#[test]
fn test_hex_color_parse_rrggbbaa_ignores_alpha() {
    assert_eq!(
        hex_color_serde::parse_color_string("#1E1E1EFF")
            .expect("expected #1E1E1EFF to parse"),
        0x1E1E1E
    );
    assert_eq!(
        hex_color_serde::parse_color_string("1E1E1EFF").expect("expected 1E1E1EFF to parse"),
        0x1E1E1E
    );
    assert_eq!(
        hex_color_serde::parse_color_string("0x1E1E1EFF")
            .expect("expected 0x1E1E1EFF to parse"),
        0x1E1E1E
    );
    assert_eq!(
        hex_color_serde::parse_color_string("#FFFFFF00")
            .expect("expected #FFFFFF00 to parse"),
        0xFFFFFF
    );
}

#[test]
fn test_hex_color_parse_invalid_lengths() {
    assert!(hex_color_serde::parse_color_string("#12").is_err());
    assert!(hex_color_serde::parse_color_string("#12345").is_err());
    assert!(hex_color_serde::parse_color_string("#1234567").is_err());
    assert!(hex_color_serde::parse_color_string("#123456789").is_err());
}

#[test]
fn test_hex_color_parse_invalid() {
    assert!(hex_color_serde::parse_color_string("invalid").is_err());
    assert!(hex_color_serde::parse_color_string("#GGG").is_err());
    assert!(hex_color_serde::parse_color_string("rgb(300, 0, 0)").is_err());
    // 300 > 255
}

#[test]
fn test_hex_color_json_deserialize_string() {
    let json = r##"{"main": "#1E1E1E"}"##;
    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(with = "hex_color_serde")]
        main: HexColor,
    }
    let parsed: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.main, 0x1E1E1E);
}

#[test]
fn test_hex_color_json_deserialize_number() {
    let json = r##"{"main": 1973790}"##; // 0x1E1E1E = 1973790
    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(with = "hex_color_serde")]
        main: HexColor,
    }
    let parsed: TestStruct = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.main, 0x1E1E1E);
}

#[test]
fn test_hex_color_json_serialize() {
    #[derive(Serialize)]
    struct TestStruct {
        #[serde(with = "hex_color_serde")]
        main: HexColor,
    }
    let data = TestStruct { main: 0xFBBF24 };
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r##"{"main":"#FBBF24"}"##);
}

#[test]
fn test_theme_deserialize_hex_strings() {
    let json = r##"{
        "colors": {
            "background": {
                "main": "#1E1E1E",
                "title_bar": "#2D2D30",
                "search_box": "#3C3C3C",
                "log_panel": "#0D0D0D"
            },
            "text": {
                "primary": "#FFFFFF",
                "secondary": "#CCCCCC",
                "tertiary": "#999999",
                "muted": "#808080",
                "dimmed": "#666666"
            },
            "accent": {
                "selected": "#FBBF24"
            },
            "ui": {
                "border": "#464647",
                "success": "#00FF00"
            }
        }
    }"##;

    let theme: Theme = serde_json::from_str(json).unwrap();
    assert_eq!(theme.colors.background.main, 0x1E1E1E);
    assert_eq!(theme.colors.accent.selected, 0xFBBF24);
    assert_eq!(theme.colors.text.secondary, 0xCCCCCC);
}

#[test]
fn test_theme_deserialize_mixed_formats() {
    // Mix of hex strings and numbers should work
    let json = r##"{
        "colors": {
            "background": {
                "main": "#1E1E1E",
                "title_bar": 2960688,
                "search_box": "rgb(60, 60, 60)",
                "log_panel": "0x0D0D0D"
            },
            "text": {
                "primary": "#FFFFFF",
                "secondary": "#CCCCCC",
                "tertiary": "#999999",
                "muted": "#808080",
                "dimmed": "#666666"
            },
            "accent": {
                "selected": "rgba(251, 191, 36, 1.0)"
            },
            "ui": {
                "border": "#464647",
                "success": "#00FF00"
            }
        }
    }"##;

    let theme: Theme = serde_json::from_str(json).unwrap();
    assert_eq!(theme.colors.background.main, 0x1E1E1E);
    assert_eq!(theme.colors.background.title_bar, 2960688);
    assert_eq!(theme.colors.background.search_box, 0x3C3C3C);
    assert_eq!(theme.colors.accent.selected, 0xFBBF24);
}

#[test]
fn test_theme_prelude_exports_core_theme_types() {
    let theme = crate::theme::prelude::Theme::default();
    let colors = crate::theme::prelude::ColorScheme::default();

    assert_eq!(
        theme.colors.background.main,
        Theme::default().colors.background.main
    );
    assert_eq!(colors.ui.border, ColorScheme::default().ui.border);
}
