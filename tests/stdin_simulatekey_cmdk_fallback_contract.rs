//! Source-level contract for the `actions-cmdk-dispatcher-fallback-contract`
//! user story (Run 8 Pass #5). Defends the Pass #4 generic Cmd+K fallback
//! in the outer `_ =>` arm of both stdin `simulateKey` dispatchers against
//! a concrete refactor threat.
//!
//! Collapsed in Run 13 into a single unified helper at `src/app_impl/simulate_key_dispatch.rs`.
//! This test verifies that the live dispatcher sites delegate to the helper,
//! and that the helper retains the generic Cmd+K fallback.

const CANONICAL_SIMULATEKEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const SIMULATE_KEY_HELPER: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");

const DISPATCHERS: &[(&str, &str)] = &[
    (
        "src/main_entry/runtime_stdin_match_simulate_key.rs",
        CANONICAL_SIMULATEKEY,
    ),
    ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
];

const FALLBACK_LOG_LINE: &str = "SimulateKey: Cmd+K - generic actions toggle (fallback for view=";
const UNHANDLED_VIEW_EVENT: &str = "event = \"simulateKey_unhandled_view\"";
const FALLBACK_HELPER_CALL: &str = "view.simulate_key_requests_generic_actions_toggle(";

fn actions_toggle_helper_body() -> &'static str {
    let start = ACTIONS_TOGGLE
        .find("pub(crate) fn simulate_key_requests_generic_actions_toggle(")
        .expect("simulate_key_requests_generic_actions_toggle helper must exist");
    let section = &ACTIONS_TOGGLE[start..];
    let end = section
        .find("/// Single per-view actions-toggle dispatcher.")
        .unwrap_or(section.len());
    &section[..end]
}

#[test]
fn both_dispatchers_contain_generic_cmdk_fallback_log_line() {
    // Both dispatchers must delegate to dispatch_simulate_key helper
    for (name, source) in DISPATCHERS {
        assert!(
            source.contains("view.dispatch_simulate_key("),
            "{name} MUST delegate to view.dispatch_simulate_key"
        );
    }

    // Helper must contain the fallback log line
    assert!(
        SIMULATE_KEY_HELPER.contains(FALLBACK_LOG_LINE),
        "simulate_key_dispatch.rs is missing the generic Cmd+K fallback log line `{FALLBACK_LOG_LINE}`"
    );
}

#[test]
fn both_dispatchers_use_named_generic_cmdk_predicate() {
    assert!(
        SIMULATE_KEY_HELPER.contains(FALLBACK_HELPER_CALL),
        "simulate_key_dispatch.rs fallback block must call the named `simulate_key_requests_generic_actions_toggle` predicate"
    );
}

#[test]
fn both_dispatchers_call_toggle_actions_after_fallback_log() {
    let anchor_idx = SIMULATE_KEY_HELPER
        .find(FALLBACK_LOG_LINE)
        .unwrap_or_else(|| panic!("simulate_key_dispatch.rs lost the fallback log anchor."));
    let tail = &SIMULATE_KEY_HELPER[anchor_idx
        ..anchor_idx
            .saturating_add(400)
            .min(SIMULATE_KEY_HELPER.len())];
    assert!(
        tail.contains("view.toggle_actions(ctx, window);"),
        "simulate_key_dispatch.rs fallback block must call `view.toggle_actions(ctx, window)`"
    );
}

#[test]
fn named_generic_cmdk_predicate_gates_on_modifiers_and_host() {
    let helper_body = actions_toggle_helper_body();
    for token in [
        "has_cmd",
        "!has_shift",
        "!has_alt",
        "!has_ctrl",
        "key_lower == \"k\"",
        "self.current_actions_host().is_some()",
    ] {
        assert!(
            helper_body.contains(token),
            "simulate_key_requests_generic_actions_toggle missing `{token}`."
        );
    }
}

#[test]
fn fallback_block_precedes_unhandled_view_warn() {
    let fallback_idx = SIMULATE_KEY_HELPER
        .find(FALLBACK_LOG_LINE)
        .unwrap_or_else(|| panic!("simulate_key_dispatch.rs lost the fallback log anchor."));
    let warn_idx = SIMULATE_KEY_HELPER
        .find(UNHANDLED_VIEW_EVENT)
        .unwrap_or_else(|| {
            panic!(
                "simulate_key_dispatch.rs is missing the `{UNHANDLED_VIEW_EVENT}` tracing event."
            )
        });
    assert!(
        fallback_idx < warn_idx,
        "simulate_key_dispatch.rs fallback block must appear BEFORE the `{UNHANDLED_VIEW_EVENT}` warn"
    );
}

#[test]
fn current_actions_host_api_still_defined_on_app_impl_actions_dialog() {
    assert!(
        ACTIONS_DIALOG
            .contains("pub(crate) fn current_actions_host(&self) -> Option<ActionsDialogHost>"),
        "`src/app_impl/actions_dialog.rs` must define `pub(crate) fn current_actions_host(&self) -> Option<ActionsDialogHost>`"
    );
}
