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
        actions.contains("crate::confirm::confirm_with_parent_dialog("),
        "Expected clipboard_delete_multiple to call confirm_with_parent_dialog directly"
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
        actions.contains("crate::confirm::confirm_with_parent_dialog("),
        "Expected clipboard_delete_all to call confirm_with_parent_dialog directly"
    );
}

#[test]
fn clipboard_actions_do_not_use_confirm_with_modal() {
    let actions = super::read_all_handle_action_sources();

    assert!(
        !actions.contains("confirm_with_modal("),
        "Clipboard actions should call confirm_with_parent_dialog directly, not confirm_with_modal"
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

#[test]
fn clipboard_builder_only_advertises_save_snippet_for_text_entries() {
    let builder = read("src/actions/builders/clipboard.rs");
    let save_snippet = builder
        .find("\"clip:clipboard_save_snippet\"")
        .expect("Expected clipboard builder to define clipboard_save_snippet");
    let before_save_snippet = &builder[..save_snippet];
    let text_guard = before_save_snippet
        .rfind("if entry_plan.is_text()")
        .expect("Expected clipboard_save_snippet to be guarded by text entry plan");
    let image_guard = before_save_snippet.rfind("if entry_plan.is_image()");

    assert!(
        image_guard.map_or(true, |index| index < text_guard),
        "clipboard_save_snippet must live inside the text-entry plan guard, not the image-entry guard"
    );
}
