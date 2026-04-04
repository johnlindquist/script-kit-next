//! Integration tests for theme contrast auditing.
//!
//! Validates that:
//! 1. `audit_theme_contrast` returns the expected sample count
//! 2. The default light theme passes all audited samples
//! 3. The worst sample still meets the minimum ratio
//! 4. `theme_contrast_score` agrees with individual sample passes

use script_kit_gpui::theme::{
    audit_theme_contrast, theme_contrast_score, worst_theme_contrast, Theme, ThemeContrastSample,
};

#[test]
fn default_light_theme_passes_all_contrast_checks() {
    let theme = Theme::light_default();
    let (passing, total) = theme_contrast_score(&theme);
    assert_eq!(
        passing, total,
        "default light theme must pass all {total} contrast checks, but only {passing} passed"
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
        14,
        "audit should return 14 surface pairing samples"
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
