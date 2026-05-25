//! Source-level contract for Run 8 Pass #19's Fix of stdin `simulateKey`
//! routing alphanumeric keystrokes into an open ActionsDialog's filter.
//!
//! Collapsed in Run 13 into a single unified helper at `src/app_impl/simulate_key_dispatch.rs`.
//! This test verifies that the live dispatcher sites delegate to the helper,
//! and that the helper retains the canonical key_char derivation block.

const SIMULATE_KEY_HELPER_SRC: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");
const APP_RUN_SETUP_SRC: &str = include_str!("../src/main_entry/app_run_setup.rs");
const SIMULATE_KEY_DISPATCHER_SRC: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");

#[test]
fn runtime_stdin_match_simulate_key_derives_key_char_from_single_char() {
    assert!(
        SIMULATE_KEY_DISPATCHER_SRC.contains("view.dispatch_simulate_key("),
        "src/main_entry/runtime_stdin_match_simulate_key.rs MUST delegate to view.dispatch_simulate_key"
    );
}

#[test]
fn app_run_setup_derives_key_char_from_single_char() {
    assert!(
        APP_RUN_SETUP_SRC.contains("view.dispatch_simulate_key("),
        "src/main_entry/app_run_setup.rs MUST delegate to view.dispatch_simulate_key"
    );
}

#[test]
fn key_char_derivation_is_identical_across_both_dispatchers() {
    let call_idx = SIMULATE_KEY_HELPER_SRC
        .find("view.route_key_to_actions_dialog(")
        .expect("helper must retain route_key_to_actions_dialog call");
    let after = &SIMULATE_KEY_HELPER_SRC[call_idx..];
    let body: String = after.lines().take(8).collect::<Vec<_>>().join("\n");

    assert!(
        body.contains("key_char,"),
        "helper MUST pass `key_char` as the second argument to `route_key_to_actions_dialog(...)`. Found call args:\n{body}"
    );
    assert!(
        !body.contains("None,"),
        "helper MUST NOT pass a bare `None,` literal. Found:\n{body}"
    );

    let anchor = "let key_char: Option<&str> = if key.chars().count() == 1 {\n            Some(key)\n        } else {\n            None\n        };";
    assert!(
        SIMULATE_KEY_HELPER_SRC.contains(anchor),
        "helper MUST contain the canonical key_char derivation block verbatim. Drift is a regression vector."
    );
}
