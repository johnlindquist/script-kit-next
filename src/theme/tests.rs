use super::{hex_color::hex_color_serde, presets, validation, Theme};
use serde::Serialize;
use serde_json::json;

const THEME_DARK_DEFAULT_GOLDEN: &str =
    include_str!("../../tests/theme/snapshots/theme_dark_default.json");
const THEME_LIGHT_DEFAULT_GOLDEN: &str =
    include_str!("../../tests/theme/snapshots/theme_light_default.json");
const PRESET_PREVIEW_COLORS_GOLDEN: &str =
    include_str!("../../tests/theme/snapshots/preset_preview_colors.json");
const COLOR_STRING_PARSE_MATRIX_GOLDEN: &str =
    include_str!("../../tests/theme/snapshots/color_string_parse_matrix.json");

#[derive(Debug, Serialize)]
struct PresetPreviewSnapshot<'a> {
    id: &'a str,
    name: &'a str,
    is_dark: bool,
    bg: String,
    accent: String,
    text: String,
    secondary: String,
    border: String,
}

#[derive(Debug, Serialize)]
struct ColorParseMatrixSnapshot<'a> {
    input: &'a str,
    parsed: Option<String>,
    error: Option<String>,
}

#[test]
fn snapshot_theme_dark_default_json() {
    let actual =
        serde_json::to_string_pretty(&Theme::dark_default()).expect("serialize dark default theme");
    assert_snapshot_matches_golden("theme_dark_default", &actual, THEME_DARK_DEFAULT_GOLDEN);
}

#[test]
fn snapshot_theme_light_default_json() {
    let actual = serde_json::to_string_pretty(&Theme::light_default())
        .expect("serialize light default theme");
    assert_snapshot_matches_golden("theme_light_default", &actual, THEME_LIGHT_DEFAULT_GOLDEN);
}

#[test]
fn snapshot_preset_preview_colors() {
    let snapshots: Vec<PresetPreviewSnapshot<'_>> = presets::all_presets()
        .iter()
        .map(|preset| {
            let theme = preset.create_theme();
            PresetPreviewSnapshot {
                id: preset.id,
                name: preset.name,
                is_dark: preset.is_dark,
                bg: to_hex_rgb(theme.colors.background.main),
                accent: to_hex_rgb(theme.colors.accent.selected),
                text: to_hex_rgb(theme.colors.text.primary),
                secondary: to_hex_rgb(theme.colors.text.secondary),
                border: to_hex_rgb(theme.colors.ui.border),
            }
        })
        .collect();

    let actual = serde_json::to_string_pretty(&snapshots).expect("serialize preset preview colors");
    assert_snapshot_matches_golden(
        "preset_preview_colors",
        &actual,
        PRESET_PREVIEW_COLORS_GOLDEN,
    );
}

#[test]
fn snapshot_color_string_parse_matrix() {
    let inputs = [
        "#1E1E1E",
        "1e1e1e",
        "0xFBBF24",
        "0X464647",
        "rgb(30, 30, 30)",
        "rgba(251, 191, 36, 0.75)",
        " rgba(0, 120, 212, 1.0) ",
        "#fff",
        "#1E1E1G",
        "rgb(256, 0, 0)",
        "rgb(30, 30)",
        "rgba(30, 30, 30)",
        "rgb(10, green, 30)",
        "rgba(10, 20, blue, 0.5)",
        "0x12345",
        "1234567",
        "",
        "hello",
    ];

    let snapshots: Vec<ColorParseMatrixSnapshot<'_>> = inputs
        .iter()
        .map(|input| match hex_color_serde::parse_color_string(input) {
            Ok(parsed) => ColorParseMatrixSnapshot {
                input,
                parsed: Some(to_hex_rgb(parsed)),
                error: None,
            },
            Err(error) => ColorParseMatrixSnapshot {
                input,
                parsed: None,
                error: Some(error),
            },
        })
        .collect();

    let actual = serde_json::to_string_pretty(&snapshots).expect("serialize color parse matrix");
    assert_snapshot_matches_golden(
        "color_string_parse_matrix",
        &actual,
        COLOR_STRING_PARSE_MATRIX_GOLDEN,
    );
}

#[test]
fn test_validate_theme_json_accepts_numeric_rgb_color_value() {
    let theme_json = json!({
        "colors": {
            "background": {
                "main": 0x001E_1E1E
            }
        }
    });

    let diagnostics = validation::validate_theme_json(&theme_json);

    assert_eq!(
        diagnostics.error_count(),
        0,
        "RGB numeric values should remain valid"
    );
    assert_eq!(
        diagnostics.warning_count(),
        0,
        "RGB numeric values should not emit warnings"
    );
}

#[test]
fn test_validate_theme_json_errors_when_numeric_color_is_float() {
    let theme_json = json!({
        "colors": {
            "background": {
                "main": 1.5
            }
        }
    });

    let diagnostics = validation::validate_theme_json(&theme_json);

    assert_eq!(
        diagnostics.error_count(),
        1,
        "floating-point numeric color values should fail validation"
    );
    assert_eq!(
        diagnostics.warning_count(),
        0,
        "floating-point numeric color values should be hard errors"
    );

    let error = diagnostics
        .diagnostics
        .first()
        .expect("expected error diagnostic");
    assert_eq!(error.path, "/colors/background/main");
    assert_eq!(
        error.message,
        "Color value must be an integer — channel values must be 0-255"
    );
}

#[test]
fn test_validate_theme_json_warns_when_numeric_color_includes_alpha_channel() {
    let theme_json = json!({
        "colors": {
            "background": {
                "main": 0x1E1E_1E80
            }
        }
    });

    let diagnostics = validation::validate_theme_json(&theme_json);

    assert_eq!(
        diagnostics.error_count(),
        0,
        "RGBA numeric values within u32 range should not be hard errors"
    );
    assert_eq!(
        diagnostics.warning_count(),
        1,
        "RGBA numeric values should emit an alpha-stripping warning"
    );

    let warning = diagnostics
        .diagnostics
        .first()
        .expect("expected warning diagnostic");
    assert_eq!(warning.path, "/colors/background/main");
    assert_eq!(
        warning.message,
        "Numeric color includes alpha channel (0xRRGGBBAA) — alpha will be stripped"
    );
}

#[test]
fn test_validate_theme_json_errors_when_numeric_color_exceeds_u32_range() {
    let theme_json = json!({
        "colors": {
            "background": {
                "main": 0x0001_0000_0000u64
            }
        }
    });

    let diagnostics = validation::validate_theme_json(&theme_json);

    assert_eq!(
        diagnostics.error_count(),
        1,
        "values above 0xFFFFFFFF should produce an error"
    );
    assert_eq!(
        diagnostics.warning_count(),
        0,
        "out-of-range values should fail validation instead of warning"
    );

    let error = diagnostics
        .diagnostics
        .first()
        .expect("expected error diagnostic");
    assert_eq!(error.path, "/colors/background/main");
    assert_eq!(
        error.message,
        "Color value exceeds 0xFFFFFFFF — expected RGB (0xRRGGBB) or RGBA (0xRRGGBBAA)"
    );
}

fn assert_snapshot_matches_golden(name: &str, actual: &str, expected: &str) {
    assert_eq!(
        actual,
        expected.trim_end(),
        "snapshot mismatch for {name}. If this is intentional, update tests/theme/snapshots/{name}.json",
    );
}

fn to_hex_rgb(color: u32) -> String {
    format!("#{:06X}", color & 0x00FF_FFFF)
}
