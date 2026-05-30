//! Integration tests for theme contrast auditing.
//!
//! Validates that:
//! 1. `audit_theme_contrast` returns the expected sample count
//! 2. The default light theme passes all audited samples
//! 3. The worst sample still meets the minimum ratio
//! 4. `theme_contrast_score` agrees with individual sample passes

use script_kit_gpui::theme::{
    audit_theme_contrast, theme_contrast_score, worst_theme_contrast, Theme, ThemeContrastSample,
    LIGHT_ROW_HOVER_OPACITY, LIGHT_ROW_SELECTED_OPACITY,
};
use serde_json::json;

#[test]
fn default_light_theme_passes_all_contrast_checks() {
    let theme = Theme::light_default();
    let samples = audit_theme_contrast(&theme);
    let failing: Vec<_> = samples.iter().filter(|sample| !sample.passes()).collect();
    assert!(
        failing.is_empty(),
        "default light theme must pass all contrast checks, failing samples: {:?}",
        failing
    );
}

#[test]
fn default_light_theme_uses_light_ordered_row_state_opacity() {
    let theme_json = serde_json::to_value(Theme::light_default())
        .expect("light theme should serialize with opacity tokens");
    let selected = theme_json["opacity"]["selected"]
        .as_f64()
        .expect("selected opacity should be numeric");
    let hover = theme_json["opacity"]["hover"]
        .as_f64()
        .expect("hover opacity should be numeric");

    assert!((selected - f64::from(LIGHT_ROW_SELECTED_OPACITY)).abs() < 1e-6);
    assert!((hover - f64::from(LIGHT_ROW_HOVER_OPACITY)).abs() < 1e-6);
    assert!(
        hover < selected,
        "light theme hover should remain quieter than focused selection"
    );
}

#[test]
fn default_light_worst_sample_still_meets_minimum() {
    let theme = Theme::light_default();
    let worst = worst_theme_contrast(&theme);
    assert!(
        worst.passes(),
        "worst sample '{}' has ratio {:.2}:1 which is below minimum {:.1}:1",
        worst.label,
        worst.ratio,
        worst.minimum,
    );
}

#[test]
fn audit_returns_expected_sample_count() {
    let theme = Theme::dark_default();
    let samples = audit_theme_contrast(&theme);
    assert_eq!(
        samples.len(),
        25,
        "audit should return 25 surface pairing samples"
    );
}

#[test]
fn score_matches_individual_sample_passes() {
    let theme = Theme::dark_default();
    let samples = audit_theme_contrast(&theme);
    let manual_passing = samples.iter().filter(|s| s.passes()).count();
    let (score_passing, score_total) = theme_contrast_score(&theme);

    assert_eq!(score_total, samples.len());
    assert_eq!(score_passing, manual_passing);
}

#[test]
fn worst_contrast_is_minimum_ratio_sample() {
    let theme = Theme::dark_default();
    let samples = audit_theme_contrast(&theme);
    let worst = worst_theme_contrast(&theme);

    let manual_worst = samples
        .iter()
        .min_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap())
        .expect("samples should not be empty");

    assert_eq!(worst.label, manual_worst.label);
    assert!((worst.ratio - manual_worst.ratio).abs() < f32::EPSILON);
}

#[test]
fn sample_passes_boundary() {
    let at_boundary = ThemeContrastSample {
        label: "boundary",
        foreground_hex: 0x000000,
        background_hex: 0x000000,
        ratio: 4.5,
        minimum: 4.5,
    };
    assert!(at_boundary.passes(), "ratio == minimum should pass");

    let just_below = ThemeContrastSample {
        label: "below",
        foreground_hex: 0x000000,
        background_hex: 0x000000,
        ratio: 4.49,
        minimum: 4.5,
    };
    assert!(!just_below.passes(), "ratio < minimum should fail");
}

#[test]
fn all_samples_have_positive_ratios() {
    for (name, theme) in [
        ("dark_default", Theme::dark_default()),
        ("light_default", Theme::light_default()),
    ] {
        let samples = audit_theme_contrast(&theme);
        for sample in &samples {
            assert!(
                sample.ratio > 0.0,
                "{name}: sample '{}' has non-positive ratio {:.2}",
                sample.label,
                sample.ratio,
            );
        }
    }
}

#[test]
fn agentic_theme_contrast_receipt() {
    if std::env::var("AGENTIC_THEME_CONTRAST_RECEIPT")
        .ok()
        .as_deref()
        != Some("1")
    {
        return;
    }

    let themes = [
        ("dark", Theme::dark_default()),
        ("light", Theme::light_default()),
    ];
    let theme_samples = themes
        .into_iter()
        .map(|(theme_id, theme)| {
            let samples = audit_theme_contrast(&theme);
            let passing = samples.iter().filter(|sample| sample.passes()).count();
            let worst = worst_theme_contrast(&theme);
            json!({
                "themeId": theme_id,
                "passing": passing,
                "total": samples.len(),
                "worst": {
                    "label": worst.label,
                    "ratio": worst.ratio,
                    "minimum": worst.minimum,
                    "foregroundColor": format!("#{:06X}", worst.foreground_hex),
                    "backgroundColor": format!("#{:06X}", worst.background_hex),
                    "contrastPass": worst.passes(),
                },
                "samples": samples
                    .into_iter()
                    .map(|sample| json!({
                        "label": sample.label,
                        "foregroundColor": format!("#{:06X}", sample.foreground_hex),
                        "backgroundColor": format!("#{:06X}", sample.background_hex),
                        "contrastRatio": sample.ratio,
                        "minimumContrastRatio": sample.minimum,
                        "contrastPass": sample.passes(),
                        "readabilityPass": sample.passes(),
                    }))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();

    let failing = theme_samples
        .iter()
        .filter(|theme| theme["passing"] != theme["total"])
        .count();
    let receipt = json!({
        "schemaVersion": 1,
        "receiptKind": "visual.contrastReadableState",
        "source": "script_kit_gpui::theme::audit_theme_contrast",
        "themeCount": theme_samples.len(),
        "failingThemeCount": failing,
        "themes": theme_samples,
    });

    println!("AGENTIC_THEME_CONTRAST_RECEIPT={receipt}");
    assert_eq!(
        failing, 0,
        "agentic contrast receipt must have no failing themes"
    );
}
