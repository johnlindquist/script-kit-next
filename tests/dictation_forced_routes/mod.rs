//! Contract tests for forced-route dictation builtins and overlay route badges.

const BUILTIN_EXECUTION_SOURCE: &str = include_str!("../../src/app_execute/builtin_execution.rs");
const BUILTINS_SOURCE: &str = include_str!("../../src/builtins/mod.rs");
const OVERLAY_SOURCE: &str = include_str!("../../src/dictation/window.rs");
const TYPES_SOURCE: &str = include_str!("../../src/dictation/types.rs");
const EXECUTION_SCRIPTS_SOURCE: &str = include_str!("../../src/app_impl/execution_scripts.rs");

fn production_source(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

#[test]
fn dictation_forced_route_variants_remain_internal() {
    let builtins_source = production_source(BUILTINS_SOURCE);

    assert!(builtins_source.contains("BuiltInFeature::DictationToFrontmostApp"));
    assert!(builtins_source.contains("BuiltInFeature::DictationToNotes"));
    assert!(
        !builtins_source.contains("builtin/dictation-to-app"),
        "forced app dictation should no longer be advertised as a top-level launcher entry"
    );
    assert!(
        !builtins_source.contains("builtin/dictation-to-notes"),
        "forced notes dictation should no longer be advertised as a top-level launcher entry"
    );
}

#[test]
fn forced_route_builtins_are_no_main_window() {
    assert!(EXECUTION_SCRIPTS_SOURCE.contains("builtin/dictation-to-app"));
    assert!(EXECUTION_SCRIPTS_SOURCE.contains("builtin/dictation-to-notes"));
}

#[test]
fn overlay_state_carries_target() {
    assert!(
        OVERLAY_SOURCE.contains("pub target: crate::dictation::DictationTarget")
            || OVERLAY_SOURCE.contains("pub target: DictationTarget")
    );
}

#[test]
fn dictation_target_exposes_overlay_labels() {
    assert!(TYPES_SOURCE.contains("pub fn overlay_label(self) -> &'static str"));
    assert!(TYPES_SOURCE.contains(r#""Prompt""#));
    assert!(TYPES_SOURCE.contains(r#""Notes""#));
    assert!(TYPES_SOURCE.contains(r#""Tab AI""#));
    assert!(TYPES_SOURCE.contains(r#""App""#));
}

#[test]
fn overlay_renders_target_badge() {
    assert!(OVERLAY_SOURCE.contains("render_target_badge"));
    assert!(OVERLAY_SOURCE.contains("TARGET_BADGE_SLOT_WIDTH_PX"));
}

#[test]
fn dictation_to_app_handler_forces_external_app_target() {
    let handler_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::DictationToFrontmostApp")
        .expect("DictationToFrontmostApp match arm must exist");
    let handler_body = &BUILTIN_EXECUTION_SOURCE[handler_start..];
    let next_arm = handler_body[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_arm];
    assert!(handler_body.contains("DictationTarget::ExternalApp"));
    assert!(handler_body.contains("dictation_to_frontmost_app_toggle"));
    assert!(handler_body.contains("Starting forced-route dictation"));
}

#[test]
fn dictation_to_notes_handler_forces_notes_target() {
    let handler_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::DictationToNotes")
        .expect("DictationToNotes match arm must exist");
    let handler_body = &BUILTIN_EXECUTION_SOURCE[handler_start..];
    let next_arm = handler_body[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_arm];
    assert!(handler_body.contains("DictationTarget::NotesEditor"));
    assert!(handler_body.contains("dictation_to_notes_toggle"));
    assert!(handler_body.contains("Starting forced-route dictation"));
}
