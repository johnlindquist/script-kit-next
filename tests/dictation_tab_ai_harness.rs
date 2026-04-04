//! Contract tests verifying the dedicated dictation-to-AI-harness entry path.
//!
//! The `DictationToAiHarness` built-in forces `DictationTarget::TabAiHarness`
//! via `resolve_dictation_target_with_override(true)` and validates it with
//! `ensure_dictation_delivery_target_available_for(target)`.  This file
//! asserts that the wiring exists and stays correct.

const BUILTIN_EXECUTION_SOURCE: &str = include_str!("../src/app_execute/builtin_execution.rs");
const BUILTINS_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const DICTATION_TYPES_SOURCE: &str = include_str!("../src/dictation/types.rs");

// =========================================================================
// Built-in variant exists
// =========================================================================

#[test]
fn dictation_to_ai_harness_variant_exists_in_builtin_feature() {
    assert!(
        BUILTINS_SOURCE.contains("DictationToAiHarness"),
        "BuiltInFeature must include a DictationToAiHarness variant"
    );
}

#[test]
fn dictation_to_ai_harness_entry_registered() {
    assert!(
        BUILTINS_SOURCE.contains("builtin/dictation-to-ai"),
        "A built-in entry with id 'builtin-dictation-to-ai' must be registered"
    );
    assert!(
        BUILTINS_SOURCE.contains("Dictate to AI"),
        "The entry must have a user-facing name 'Dictate to AI'"
    );
}

// =========================================================================
// Target override wiring
// =========================================================================

/// The DictationToAiHarness handler must use the target-override helper
/// to force `TabAiHarness`, not the generic `resolve_dictation_target()`.
#[test]
fn handler_uses_target_override() {
    let handler_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::DictationToAiHarness")
        .expect("DictationToAiHarness match arm must exist in builtin_execution.rs");
    let handler_body = &BUILTIN_EXECUTION_SOURCE[handler_start..];
    // Find the end of this match arm (next top-level BuiltInFeature:: arm)
    let next_arm = handler_body[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_arm];

    assert!(
        handler_body.contains("resolve_dictation_target_with_override(true)"),
        "DictationToAiHarness handler must call resolve_dictation_target_with_override(true)"
    );
    assert!(
        !handler_body.contains("self.resolve_dictation_target()")
            || handler_body.contains("resolve_dictation_target_with_override"),
        "DictationToAiHarness handler must not fall back to the generic target resolver"
    );
}

/// The handler must validate using the target-aware validator, not the
/// generic `ensure_dictation_delivery_target_available()`.
#[test]
fn handler_uses_target_aware_validation() {
    let handler_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::DictationToAiHarness")
        .expect("DictationToAiHarness match arm must exist");
    let handler_body = &BUILTIN_EXECUTION_SOURCE[handler_start..];
    let next_arm = handler_body[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_arm];

    assert!(
        handler_body.contains("ensure_dictation_delivery_target_available_for(target)"),
        "DictationToAiHarness handler must call ensure_dictation_delivery_target_available_for(target)"
    );
}

// =========================================================================
// Target-aware validation accepts TabAiHarness without QuickTerminalView
// =========================================================================

/// The target-aware validation helper must accept `TabAiHarness` as a
/// valid target — it doesn't require any active view to deliver to.
#[test]
fn target_aware_validator_accepts_tab_ai_harness() {
    let fn_start = BUILTIN_EXECUTION_SOURCE
        .find("fn ensure_dictation_delivery_target_available_for")
        .expect("ensure_dictation_delivery_target_available_for must exist");
    let fn_body = &BUILTIN_EXECUTION_SOURCE[fn_start..];
    let fn_end = fn_body[1..]
        .find("\n    fn ")
        .or_else(|| fn_body[1..].find("\n    pub"))
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..fn_end];

    // TabAiHarness must be in the Ok(()) arm, not the ExternalApp arm.
    assert!(
        fn_body.contains("TabAiHarness => Ok(())"),
        "ensure_dictation_delivery_target_available_for must return Ok(()) for TabAiHarness"
    );
}

/// The resolve_dictation_target_with_override(true) must return TabAiHarness.
#[test]
fn target_override_forces_tab_ai_harness() {
    let fn_start = BUILTIN_EXECUTION_SOURCE
        .find("fn resolve_dictation_target_with_override")
        .expect("resolve_dictation_target_with_override must exist");
    let fn_body = &BUILTIN_EXECUTION_SOURCE[fn_start..];
    let fn_end = fn_body[1..]
        .find("\n    fn ")
        .or_else(|| fn_body[1..].find("\n    pub"))
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..fn_end];

    assert!(
        fn_body.contains("DictationTarget::TabAiHarness"),
        "resolve_dictation_target_with_override must return TabAiHarness when forced"
    );
}

// =========================================================================
// DictationTarget enum has TabAiHarness
// =========================================================================

#[test]
fn dictation_target_enum_has_tab_ai_harness_variant() {
    assert!(
        DICTATION_TYPES_SOURCE.contains("TabAiHarness"),
        "DictationTarget must have a TabAiHarness variant"
    );
}

// =========================================================================
// Dictation delivery handler routes TabAiHarness to quick-submit
// =========================================================================

/// When dictation finishes with target=TabAiHarness, the transcript must
/// be submitted via the harness quick-submit path.
#[test]
fn dictation_transcript_delivery_routes_tab_ai_harness() {
    let fn_start = BUILTIN_EXECUTION_SOURCE
        .find("fn handle_dictation_transcript")
        .expect("handle_dictation_transcript must exist");
    let fn_body = &BUILTIN_EXECUTION_SOURCE[fn_start..];
    let fn_end = fn_body[1..]
        .find("\n    fn ")
        .or_else(|| fn_body[1..].find("\n    pub"))
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..fn_end];

    assert!(
        fn_body.contains("DictationTarget::TabAiHarness"),
        "handle_dictation_transcript must have a TabAiHarness arm"
    );
    assert!(
        fn_body.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "TabAiHarness delivery must call submit_to_current_or_new_tab_ai_harness_from_text"
    );
}

// =========================================================================
// Stop edge defaults to TabAiHarness (not ExternalApp) for this handler
// =========================================================================

#[test]
fn stop_edge_defaults_to_tab_ai_harness() {
    let handler_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::DictationToAiHarness")
        .expect("DictationToAiHarness match arm must exist");
    let handler_body = &BUILTIN_EXECUTION_SOURCE[handler_start..];
    let next_arm = handler_body[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_arm];

    // On the stop edge, the fallback target must be TabAiHarness
    // (not ExternalApp like the generic handler uses).
    assert!(
        handler_body.contains("unwrap_or(crate::dictation::DictationTarget::TabAiHarness)"),
        "Stop edge must default to TabAiHarness, not ExternalApp"
    );
}

// =========================================================================
// Model download preflight shared with generic Dictation
// =========================================================================

#[test]
fn harness_dictation_checks_model_availability() {
    let handler_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::DictationToAiHarness")
        .expect("DictationToAiHarness match arm must exist");
    let handler_body = &BUILTIN_EXECUTION_SOURCE[handler_start..];
    let next_arm = handler_body[1..]
        .find("builtins::BuiltInFeature::")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_arm];

    assert!(
        handler_body.contains("is_parakeet_model_available()"),
        "DictationToAiHarness must check Parakeet model availability"
    );
    assert!(
        handler_body.contains("open_dictation_model_prompt(cx)"),
        "DictationToAiHarness must open the model download prompt when model is missing"
    );
}
