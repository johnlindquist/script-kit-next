//! Targeted regression tests for destructive clipboard action safeguards.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"))
}

#[test]
fn clipboard_delete_multiple_requires_confirmation_before_delete() {
    let content = read("src/app_actions.rs");

    assert!(
        content.contains("\"clipboard_delete_multiple\""),
        "Expected app_actions.rs to handle clipboard_delete_multiple"
    );
    assert!(
        content.contains("Are you sure you want to delete these")
            && content.contains("matching clipboard entries"),
        "Expected clipboard_delete_multiple to show a count-aware confirmation message"
    );
    assert!(
        content.contains("open_confirm_window("),
        "Expected clipboard_delete_multiple to use confirmation modal via open_confirm_window"
    );
}

#[test]
fn clipboard_delete_all_requires_confirmation_before_delete() {
    let content = read("src/app_actions.rs");

    assert!(
        content.contains("\"clipboard_delete_all\""),
        "Expected app_actions.rs to handle clipboard_delete_all"
    );
    assert!(
        content.contains("Are you sure you want to delete all")
            && content.contains("unpinned clipboard entries"),
        "Expected clipboard_delete_all to show a confirmation warning before deleting all unpinned entries"
    );
    assert!(
        content.contains("open_confirm_window("),
        "Expected clipboard_delete_all to use confirmation modal via open_confirm_window"
    );
}

#[test]
fn builtin_confirmation_modal_failure_does_not_auto_confirm() {
    let content = read("src/app_execute.rs");

    assert!(
        !content.contains("confirm_sender.try_send((entry_id.clone(), true))"),
        "Expected execute_builtin confirmation modal failure path to NOT auto-confirm destructive action"
    );
}

#[test]
fn clipboard_save_snippet_rejects_non_text_entries() {
    let content = read("src/app_actions.rs");

    assert!(
        content.contains("\"clipboard_save_snippet\""),
        "Expected app_actions.rs to handle clipboard_save_snippet"
    );
    assert!(
        content.contains("entry.content_type != clipboard_history::ContentType::Text"),
        "Expected clipboard_save_snippet to explicitly guard non-text clipboard entries"
    );
    assert!(
        content.contains("Only text can be saved as snippet"),
        "Expected clipboard_save_snippet to show a clear user-facing error for non-text entries"
    );
}
