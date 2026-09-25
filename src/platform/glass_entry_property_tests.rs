//! Rules for window entry and exit motion.
//!
//! Every window keeps its settled frame; entry is one grouped ease-out fade
//! from each preset's start alphas to 1.0 and exit is one alpha fade to 0.
//! With no frame travel left, the motion rules are rules about the values:
//! where the fade may start, how long it may take, and whether a window is
//! removed before its exit finishes. Stored preset ids are generated input
//! from config.ts, so every string must still load as some preset.
//!
//! These tests read the calibrated values; they never change them.

use gpui::proptest::prelude::*;

use crate::test_utils::property::assert_rule;
use crate::theme::animation::MainWindowAnimationPreset;

/// NSWindow alpha below this exposes desktop pixels through the glass.
const ALPHA_FLOOR: f64 = 0.85;
/// The longest a window may take to arrive or leave and still read as a
/// quick fade.
const FAST_MS: f64 = 150.0;

#[test]
fn every_preset_fades_in_fast_from_above_the_alpha_floor() {
    let mut broken = Vec::new();
    for preset in MainWindowAnimationPreset::ALL {
        let spec = preset.spec();
        let id = preset.id();
        if !(ALPHA_FLOOR..=1.0).contains(&spec.window_start_alpha) {
            broken.push(format!(
                "{id}: window starts at alpha {}, outside {ALPHA_FLOOR}..=1",
                spec.window_start_alpha
            ));
        }
        for (channel, start) in [
            ("material", spec.material_start_alpha),
            ("content", spec.content_start_alpha),
        ] {
            if !(start > 0.0 && start <= 1.0) {
                broken.push(format!("{id}: {channel} starts at alpha {start}"));
            }
        }
        if spec.duration_ms == 0 || f64::from(spec.duration_ms) > FAST_MS {
            broken.push(format!(
                "{id}: entry takes {}ms (fast is 1..={FAST_MS}ms)",
                spec.duration_ms
            ));
        }
        if spec.blur_radius < 0.0 || spec.foreground_blur_radius < 0.0 {
            broken.push(format!("{id}: negative defocus radius"));
        }
    }
    assert!(
        broken.is_empty(),
        "entry rules broken:\n{}",
        broken.join("\n")
    );
}

#[test]
fn the_exit_fade_finishes_before_the_window_is_removed() {
    let exit_ms = super::GLASS_EXIT_DURATION * 1000.0;
    let remove_ms = super::glass_exit_remove_delay().as_secs_f64() * 1000.0;
    assert!(
        exit_ms > 0.0 && exit_ms <= FAST_MS,
        "exit fade takes {exit_ms}ms (fast is up to {FAST_MS}ms)"
    );
    assert!(
        remove_ms >= exit_ms,
        "window removed at {remove_ms}ms, before its {exit_ms}ms exit fade finishes"
    );
}

fn stored_preset_id() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(vec![
            "fixed_frame",
            "sharp_presence",
            "previous_calibration",
            "coherent",
            "grow_in",
            "",
            "SHARP_PRESENCE",
        ])
        .prop_map(str::to_string),
        any::<String>(),
    ]
}

#[test]
fn any_stored_preset_id_loads_and_round_trips() {
    assert_rule(
        "Any stored preset id loads: known ids keep their preset, everything else is the default, and the result round-trips",
        512,
        stored_preset_id(),
        |stored| {
            let loaded: MainWindowAnimationPreset =
                serde_json::from_value(serde_json::Value::String(stored.clone()))
                    .map_err(|error| TestCaseError::fail(format!("{stored:?} rejected: {error}")))?;
            let expected = MainWindowAnimationPreset::ALL
                .into_iter()
                .find(|preset| preset.id() == stored)
                .unwrap_or_default();
            prop_assert_eq!(loaded, expected, "{:?} loaded as {:?}", stored, loaded);
            let saved = serde_json::to_value(loaded).expect("serialize preset");
            let reloaded: MainWindowAnimationPreset =
                serde_json::from_value(saved).expect("reload saved preset");
            prop_assert_eq!(reloaded, loaded);
            Ok(())
        },
    );
}
