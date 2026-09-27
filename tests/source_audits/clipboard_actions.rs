//! Targeted regression tests for destructive clipboard action safeguards.

use super::read_source as read;

#[test]
fn builtin_confirmation_modal_failure_does_not_auto_confirm() {
    let content = read("src/app_execute.rs");

    assert!(
        !content.contains("confirm_sender.try_send((entry_id.clone(), true))"),
        "Expected execute_builtin confirmation modal failure path to NOT auto-confirm destructive action"
    );
}
