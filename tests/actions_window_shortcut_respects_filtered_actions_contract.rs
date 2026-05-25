//! Source-audit contract verifying that the detached actions window shortcut matching
//! respects filtered actions. This aligns detached window shortcut execution with
//! attached popup shortcut execution.

const ACTIONS_WINDOW: &str = include_str!("../src/actions/window.rs");

#[test]
fn actions_window_shortcut_respects_filtered_actions() {
    // Verify that the shortcut matching logic in window.rs uses
    // matching_filtered_action_id_for_keystroke instead of matching_action_id_for_keystroke.
    assert!(
        ACTIONS_WINDOW.contains("crate::actions::matching_filtered_action_id_for_keystroke("),
        "src/actions/window.rs MUST use matching_filtered_action_id_for_keystroke to match shortcuts"
    );

    assert!(
        !ACTIONS_WINDOW.contains("crate::actions::matching_action_id_for_keystroke("),
        "src/actions/window.rs MUST NOT use matching_action_id_for_keystroke (which matches filtered-out actions)"
    );
}
