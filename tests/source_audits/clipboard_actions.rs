//! Targeted regression tests for destructive clipboard action safeguards.

use super::read_source as read;

#[test]
fn clipboard_delete_multiple_requires_confirmation_before_delete() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"clipboard_delete_multiple\""),
        "Expected handle_action/ to handle clipboard_delete_multiple"
    );
    assert!(
        actions.contains("Are you sure you want to delete these")
            && actions.contains("matching clipboard entries"),
        "Expected clipboard_delete_multiple to show a count-aware confirmation message"
    );
    assert!(
        actions.contains("confirm_with_modal("),
        "Expected clipboard_delete_multiple to use the shared confirm_with_modal helper"
    );
}

#[test]
fn clipboard_delete_all_requires_confirmation_before_delete() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"clipboard_delete_all\""),
        "Expected handle_action/ to handle clipboard_delete_all"
    );
    assert!(
        actions.contains("Are you sure you want to delete all")
            && actions.contains("unpinned clipboard entries"),
        "Expected clipboard_delete_all to show a confirmation warning before deleting all unpinned entries"
    );
    assert!(
        actions.contains("confirm_with_modal("),
        "Expected clipboard_delete_all to use the shared confirm_with_modal helper"
    );
}

#[test]
fn confirm_with_modal_helper_calls_open_confirm_window() {
    let helpers = read("src/app_actions/helpers.rs");

    assert!(
        helpers.contains("async fn confirm_with_modal("),
        "Expected helpers.rs to define confirm_with_modal async helper"
    );
    assert!(
        helpers.contains("confirm::open_confirm_window("),
        "Expected confirm_with_modal to delegate to open_confirm_window"
    );
    assert!(
        helpers.contains("async_channel::bounded::<bool>(1)"),
        "Expected confirm_with_modal to use a bounded channel for the confirmation result"
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
    let actions = super::read_all_handle_action_sources();

    assert!(
        actions.contains("\"clipboard_save_snippet\""),
        "Expected handle_action/ to handle clipboard_save_snippet"
    );
    assert!(
        actions.contains("entry.content_type != clipboard_history::ContentType::Text"),
        "Expected clipboard_save_snippet to explicitly guard non-text clipboard entries"
    );
    assert!(
        actions.contains("Only text can be saved as snippet"),
        "Expected clipboard_save_snippet to show a clear user-facing error for non-text entries"
    );
}
