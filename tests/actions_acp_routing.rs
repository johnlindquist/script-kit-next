//! Source-contract tests for detached actions window → ACP Chat handoff.
//!
//! Locks the invariant that the detached actions window uses the shared
//! action target builder and the shared secondary-window handoff helper.

use std::fs;

#[test]
fn detached_actions_window_uses_shared_action_target_builder_and_handoff() {
    let source =
        fs::read_to_string("src/actions/window.rs").expect("Failed to read src/actions/window.rs");

    assert!(
        source.contains("build_action_target_for_ai"),
        "Detached actions window must use the shared action target builder"
    );

    assert!(
        source.contains("request_explicit_acp_handoff_from_secondary_window"),
        "Detached actions window must use the shared secondary-window ACP handoff helper"
    );
}
