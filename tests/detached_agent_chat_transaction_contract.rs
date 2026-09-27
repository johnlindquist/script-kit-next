//! Contract tests for the ActionsDialog batch selection API. These are
//! source-level audits that confirm the wiring exists without requiring a live
//! GPUI window.

// ============================================================
// ActionsDialog direct mutation methods exist
// ============================================================

#[test]
fn actions_dialog_has_select_action_by_id_method() {
    let source = include_str!("../src/actions/dialog.rs");
    assert!(
        source.contains("pub fn select_action_by_id"),
        "ActionsDialog must have a pub select_action_by_id method for batch selectByValue"
    );
}

#[test]
fn actions_dialog_has_select_action_by_semantic_id_method() {
    let source = include_str!("../src/actions/dialog.rs");
    assert!(
        source.contains("pub fn select_action_by_semantic_id"),
        "ActionsDialog must have a pub select_action_by_semantic_id method for batch selectBySemanticId"
    );
}
