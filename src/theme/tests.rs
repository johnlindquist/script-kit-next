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

// ============================================================================
// WCAG Contrast Ratio Audit
// ============================================================================

/// WCAG 2.1 contrast ratio between two colors.
/// Returns a ratio >= 1.0 (e.g. 4.5 means 4.5:1).
fn contrast_ratio(fg: u32, bg: u32) -> f32 {
    let l1 = super::types::relative_luminance_srgb(fg);
    let l2 = super::types::relative_luminance_srgb(bg);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Composite a semi-transparent foreground color over an opaque background.
/// Returns the resulting opaque RGB as a u32.
fn composite_alpha(fg: u32, alpha: f32, bg: u32) -> u32 {
    let blend = |shift: u32| {
        let f = ((fg >> shift) & 0xFF) as f32;
        let b = ((bg >> shift) & 0xFF) as f32;
        (f * alpha + b * (1.0 - alpha)).round() as u32
    };
    (blend(16) << 16) | (blend(8) << 8) | blend(0)
}

/// A single contrast check with context for error reporting
struct ContrastCheck {
    pair: &'static str,
    fg: u32,
    bg: u32,
    min_ratio: f32,
}

/// Audit all theme presets for WCAG contrast compliance.
///
/// Checks critical text-on-background pairs:
/// - primary text on main background (WCAG AA normal text: 4.5:1)
/// - secondary text on main background (WCAG AA normal text: 4.5:1)
/// - muted text on main background (WCAG AA large text: 3.0:1)
/// - accent on main background (WCAG AA large text: 3.0:1)
/// - on_accent text on accent background (WCAG AA large text: 3.0:1)
///
/// Themes that fail are collected and reported in a single assertion
/// with exact ratios so you can see what needs fixing.
#[test]
fn audit_theme_contrast_ratios() {
    let presets = presets::all_presets();
    let mut failures: Vec<String> = Vec::new();

    for preset in &presets {
        let theme = preset.create_theme();
        let bg = theme.colors.background.main;
        let opacity = theme.get_opacity();

        // Compute the effective selection background:
        // accent.selected_subtle at opacity.selected, composited over bg.main
        let sel_bg = composite_alpha(theme.colors.accent.selected_subtle, opacity.selected, bg);

        let checks = [
            ContrastCheck {
                pair: "primary/bg",
                fg: theme.colors.text.primary,
                bg,
                min_ratio: 4.5,
            },
            ContrastCheck {
                pair: "secondary/bg",
                fg: theme.colors.text.secondary,
                bg,
                min_ratio: 4.5,
            },
            ContrastCheck {
                pair: "muted/bg",
                fg: theme.colors.text.muted,
                bg,
                min_ratio: 3.0,
            },
            ContrastCheck {
                pair: "accent/bg",
                fg: theme.colors.accent.selected,
                bg,
                min_ratio: 3.0,
            },
            ContrastCheck {
                pair: "on_accent/accent",
                fg: theme.colors.text.on_accent,
                bg: theme.colors.accent.selected,
                min_ratio: 3.0,
            },
            // Selected item text contrast: primary text on the composited
            // selection highlight (accent.selected_subtle @ opacity.selected over bg.main)
            ContrastCheck {
                pair: "primary/sel_bg",
                fg: theme.colors.text.primary,
                bg: sel_bg,
                min_ratio: 4.5,
            },
            // Dimmed text (input placeholder) must be readable on main bg
            ContrastCheck {
                pair: "dimmed/bg",
                fg: theme.colors.text.dimmed,
                bg,
                min_ratio: 1.5, // Placeholders are intentionally quiet but must be visible
            },
        ];

        for check in &checks {
            let ratio = contrast_ratio(check.fg, check.bg);
            if ratio < check.min_ratio {
                failures.push(format!(
                    "  {:<25} {:<18} {:>5.2}:1  (need {:.1}:1)  fg=#{:06X} bg=#{:06X}",
                    preset.id, check.pair, ratio, check.min_ratio, check.fg, check.bg,
                ));
            }
        }
    }

    if !failures.is_empty() {
        let report = failures.join("\n");
        eprintln!(
            "\n╔══ Theme Contrast Audit ══════════════════════════════════════════════════════╗\n\
             ║ {} failure(s) across {} themes                                              \n\
             ╠════════════════════════════════════════════════════════════════════════════════╣\n\
             {}\n\
             ╚════════════════════════════════════════════════════════════════════════════════╝\n",
            failures.len(),
            presets.len(),
            report,
        );
        panic!(
            "{} contrast failure(s) found — see report above",
            failures.len()
        );
    }
}

/// Compute the optimal `selected_subtle` for a theme: the value closest to
/// `bg_main` that still passes `min_ratio` contrast for `text_primary` against
/// the composited selection background.
///
/// Strategy: binary search between bg_main and a target endpoint.
/// - Dark themes (light text): search toward white (brighter selection)
/// - Light themes (dark text): search toward black (darker selection)
fn compute_optimal_selected_subtle(
    bg_main: u32,
    text_primary: u32,
    opacity_selected: f32,
    min_ratio: f32,
) -> u32 {
    let is_dark = super::types::relative_luminance_srgb(bg_main) < 0.5;
    // Target: move selected_subtle away from bg toward the opposite extreme
    let target = if is_dark { 0xFFFFFF } else { 0x000000 };

    // Binary search: find the value closest to bg_main that passes
    let blend_channel = |bg_ch: u8, tgt_ch: u8, t: f32| -> u8 {
        (bg_ch as f32 + (tgt_ch as f32 - bg_ch as f32) * t).round() as u8
    };

    let make_color = |t: f32| -> u32 {
        let r = blend_channel((bg_main >> 16 & 0xFF) as u8, (target >> 16 & 0xFF) as u8, t);
        let g = blend_channel((bg_main >> 8 & 0xFF) as u8, (target >> 8 & 0xFF) as u8, t);
        let b = blend_channel((bg_main & 0xFF) as u8, (target & 0xFF) as u8, t);
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    };

    let check = |t: f32| -> bool {
        let subtle = make_color(t);
        let sel_bg = composite_alpha(subtle, opacity_selected, bg_main);
        contrast_ratio(text_primary, sel_bg) >= min_ratio
    };

    // Binary search for the minimum t that passes
    let mut lo: f32 = 0.0;
    let mut hi: f32 = 1.0;
    for _ in 0..32 {
        let mid = (lo + hi) / 2.0;
        if check(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    make_color(hi)
}

/// Report optimal selected_subtle values for all themes.
/// Prints themes where the current value differs significantly from optimal.
#[test]
fn report_optimal_selected_subtle() {
    let presets = presets::all_presets();
    let mut suggestions: Vec<String> = Vec::new();

    for preset in &presets {
        let theme = preset.create_theme();
        let opacity = theme.get_opacity();
        let optimal = compute_optimal_selected_subtle(
            theme.colors.background.main,
            theme.colors.text.primary,
            opacity.selected,
            4.5,
        );
        let current = theme.colors.accent.selected_subtle;

        // Check if current passes
        let sel_bg = composite_alpha(current, opacity.selected, theme.colors.background.main);
        let current_ratio = contrast_ratio(theme.colors.text.primary, sel_bg);

        // Check how far current is from optimal (by luminance distance from bg)
        let bg_lum = super::types::relative_luminance_srgb(theme.colors.background.main);
        let current_lum = super::types::relative_luminance_srgb(current);
        let optimal_lum = super::types::relative_luminance_srgb(optimal);
        let current_dist = (current_lum - bg_lum).abs();
        let optimal_dist = (optimal_lum - bg_lum).abs();

        // Flag if current is much farther from bg than needed (over-prominent selection)
        if current_dist > optimal_dist * 1.5 && current_ratio >= 4.5 {
            suggestions.push(format!(
                "  {:<25} current=0x{:06X} optimal=0x{:06X}  (ratio {:.1}:1, could be {:.1}:1 with optimal)",
                preset.id, current, optimal, current_ratio,
                {
                    let opt_sel_bg = composite_alpha(optimal, opacity.selected, theme.colors.background.main);
                    contrast_ratio(theme.colors.text.primary, opt_sel_bg)
                },
            ));
        }
    }

    if !suggestions.is_empty() {
        eprintln!(
            "\n╔══ Over-prominent selections ════════════════════════════════════════════════╗\n\
             ║ {} theme(s) have selected_subtle farther from bg than needed               \n\
             ╠════════════════════════════════════════════════════════════════════════════════╣\n\
             {}\n\
             ╚════════════════════════════════════════════════════════════════════════════════╝\n",
            suggestions.len(),
            suggestions.join("\n"),
        );
    }
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
