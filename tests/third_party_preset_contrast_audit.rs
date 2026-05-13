//! Verifies the third-party preset contrast goals defined in
//! `.goals/third-party-preset-contrast.md`.
//!
//! Replicates the math from `src/theme/tests.rs::audit_selection_visibility_across_presets`
//! and the `primary/selection_surface` row of
//! `src/theme/tests.rs::audit_app_chrome_surface_contrast`, but as an
//! integration test so it can run independently of the lib-test target
//! (which currently fails to compile in this worktree due to unrelated WIP).

use script_kit_gpui::theme::{contrast_ratio, presets};

const PRESET_PREVIEW_OPACITY_SELECTED_DARK: f32 = 0.23;
const PRESET_PREVIEW_OPACITY_HOVER_DARK: f32 = 0.06;
const PRESET_PREVIEW_OPACITY_SELECTED_LIGHT: f32 = 0.08;
const PRESET_PREVIEW_OPACITY_HOVER_LIGHT: f32 = 0.04;

fn composite_alpha(fg: u32, alpha: f32, bg: u32) -> u32 {
    let blend = |shift: u32| {
        let f = ((fg >> shift) & 0xFF) as f32;
        let b = ((bg >> shift) & 0xFF) as f32;
        (f * alpha + b * (1.0 - alpha)).round() as u32
    };
    (blend(16) << 16) | (blend(8) << 8) | blend(0)
}

fn hard_readable_text_hex(bg: u32) -> u32 {
    // Mirrors super::helpers::hard_readable_text_hex: pick black or white
    // by which yields better contrast.
    let white_ratio = contrast_ratio(0xFFFFFF, bg);
    let black_ratio = contrast_ratio(0x000000, bg);
    if white_ratio >= black_ratio {
        0xFFFFFF
    } else {
        0x000000
    }
}

#[test]
fn third_party_presets_clear_selection_and_hover_visibility() {
    let mut failures: Vec<String> = Vec::new();

    for preset in &presets::all_presets() {
        let theme = preset.create_theme();
        let opacity = theme.get_opacity();
        let bg = theme.colors.background.main;
        let subtle = theme.colors.accent.selected_subtle;

        let sel_bg = composite_alpha(subtle, opacity.selected, bg);
        let hov_bg = composite_alpha(subtle, opacity.hover, bg);
        let sel_vis = contrast_ratio(sel_bg, bg);
        let hov_vis = contrast_ratio(hov_bg, bg);

        if sel_vis < 1.10 {
            failures.push(format!(
                "{:<25} selection {:>5.3}:1 (need 1.10:1) subtle=#{:06X} bg=#{:06X}",
                preset.id, sel_vis, subtle, bg
            ));
        }
        if hov_vis < 1.03 {
            failures.push(format!(
                "{:<25} hover     {:>5.3}:1 (need 1.03:1) subtle=#{:06X} bg=#{:06X}",
                preset.id, hov_vis, subtle, bg
            ));
        }

        // Sanity: opacity defaults match what the audit assumes per appearance.
        if preset.is_dark {
            assert!(
                (opacity.selected - PRESET_PREVIEW_OPACITY_SELECTED_DARK).abs() < 1e-3
                    && (opacity.hover - PRESET_PREVIEW_OPACITY_HOVER_DARK).abs() < 1e-3,
                "{} unexpected dark row opacity: selected={} hover={}",
                preset.id,
                opacity.selected,
                opacity.hover
            );
        } else {
            assert!(
                (opacity.selected - PRESET_PREVIEW_OPACITY_SELECTED_LIGHT).abs() < 1e-3
                    && (opacity.hover - PRESET_PREVIEW_OPACITY_HOVER_LIGHT).abs() < 1e-3,
                "{} unexpected light row opacity: selected={} hover={}",
                preset.id,
                opacity.selected,
                opacity.hover
            );
        }
    }

    assert!(
        failures.is_empty(),
        "\n{} selection-visibility failure(s):\n  {}\n",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
fn primary_selection_surface_passes_for_all_presets_except_documented_exemption() {
    // Replicates the `primary/selection_surface` row of
    // `audit_app_chrome_surface_contrast`.  The chrome contract composites
    // `text.primary` over `bg.main` at `opacity.selected` and compares the
    // result against `text_primary_hex` (which `normalize_*_interactive_tokens`
    // remaps to a hard black/white decision via `hard_readable_text_hex`).
    let mut failures: Vec<String> = Vec::new();

    for preset in &presets::all_presets() {
        let theme = preset.create_theme();
        let opacity = theme.get_opacity();
        let bg = theme.colors.background.main;

        let text_primary_hex = hard_readable_text_hex(bg);
        // Sanity-check the normalization pass: theme.colors.text.primary
        // should match the hard-readable decision (this is the post-pass
        // guarantee that the test audit relies on).
        assert_eq!(
            theme.colors.text.primary, text_primary_hex,
            "{} text.primary should be hard-readable for bg=#{:06X}",
            preset.id, bg
        );

        let selection_surface = composite_alpha(text_primary_hex, opacity.selected, bg);
        let ratio = contrast_ratio(text_primary_hex, selection_surface);

        let is_exempt = preset.id == "fairy-floss";

        if ratio < 4.5 && !is_exempt {
            failures.push(format!(
                "{:<25} primary/selection_surface {:>5.2}:1 (need 4.5:1) fg=#{:06X} bg=#{:06X}",
                preset.id, ratio, text_primary_hex, selection_surface
            ));
        } else if ratio >= 4.5 && is_exempt {
            panic!(
                "{} no longer needs the chrome-contrast exemption ({:.2}:1 >= 4.5:1) — \
                 remove it from src/theme/tests.rs::is_chrome_contrast_exempt and from \
                 lat.md/theme.md#Preset contrast guardrail",
                preset.id, ratio
            );
        }
    }

    assert!(
        failures.is_empty(),
        "\n{} chrome-selection failure(s):\n  {}\n",
        failures.len(),
        failures.join("\n  ")
    );
}
