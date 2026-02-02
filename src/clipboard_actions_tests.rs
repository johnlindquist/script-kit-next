//! Regression tests for clipboard history action wiring.
//!
//! These tests assert that the clipboard actions are wired in the UI and
//! action handler layers, preventing accidental regressions.

use std::fs;

#[test]
fn test_app_actions_handles_clipboard_pin_unpin() {
    let content =
        fs::read_to_string("src/app_actions.rs").expect("Failed to read src/app_actions.rs");

    assert!(
        content.contains("\"clipboard_pin\""),
        "Expected app_actions.rs to handle \"clipboard_pin\" action id"
    );
    assert!(
        content.contains("clipboard_history::pin_entry"),
        "Expected app_actions.rs to call clipboard_history::pin_entry"
    );
    assert!(
        content.contains("\"clipboard_unpin\""),
        "Expected app_actions.rs to handle \"clipboard_unpin\" action id"
    );
    assert!(
        content.contains("clipboard_history::unpin_entry"),
        "Expected app_actions.rs to call clipboard_history::unpin_entry"
    );
}

#[test]
fn test_render_builtins_has_clipboard_pin_shortcut_and_actions_toggle() {
    let content = fs::read_to_string("src/render_builtins.rs")
        .expect("Failed to read src/render_builtins.rs");

    assert!(
        content.contains("toggle_clipboard_actions"),
        "Expected render_builtins.rs to define or call toggle_clipboard_actions"
    );
    assert!(
        content.contains("has_cmd && key_str == \"p\""),
        "Expected render_builtins.rs to handle Cmd+P for clipboard pin toggle"
    );
    assert!(
        content.contains("clipboard_pin"),
        "Expected render_builtins.rs to reference clipboard_pin action id"
    );
    assert!(
        content.contains("clipboard_unpin"),
        "Expected render_builtins.rs to reference clipboard_unpin action id"
    );
}

#[test]
fn test_app_actions_handles_clipboard_copy_and_paste_keep_open() {
    let content =
        fs::read_to_string("src/app_actions.rs").expect("Failed to read src/app_actions.rs");

    assert!(
        content.contains("\"clipboard_copy\""),
        "Expected app_actions.rs to handle \"clipboard_copy\" action id"
    );
    assert!(
        content.contains("\"clipboard_paste_keep_open\""),
        "Expected app_actions.rs to handle \"clipboard_paste_keep_open\" action id"
    );
    assert!(
        content.contains("clipboard_history::copy_entry_to_clipboard"),
        "Expected app_actions.rs to copy clipboard entries via clipboard_history::copy_entry_to_clipboard"
    );
    assert!(
        content.contains("selected_text::simulate_paste_with_cg"),
        "Expected app_actions.rs to simulate paste via selected_text::simulate_paste_with_cg"
    );
}

#[test]
fn test_app_actions_handles_clipboard_attach_to_ai() {
    let content =
        fs::read_to_string("src/app_actions.rs").expect("Failed to read src/app_actions.rs");

    // Just check that the action ID exists - the full implementation may come later
    assert!(
        content.contains("\"clipboard_attach_to_ai\""),
        "Expected app_actions.rs to handle \"clipboard_attach_to_ai\" action id"
    );
}

#[test]
fn test_app_actions_handles_clipboard_delete() {
    let content =
        fs::read_to_string("src/app_actions.rs").expect("Failed to read src/app_actions.rs");

    assert!(
        content.contains("\"clipboard_delete\""),
        "Expected app_actions.rs to handle \"clipboard_delete\" action id"
    );
    assert!(
        content.contains("clipboard_history::remove_entry"),
        "Expected app_actions.rs to call clipboard_history::remove_entry"
    );
}

#[test]
fn test_render_builtins_clipboard_footer_uses_selected_entry_for_actions() {
    let content = fs::read_to_string("src/render_builtins.rs")
        .expect("Failed to read src/render_builtins.rs");

    assert!(
        content.contains("show_secondary(has_entry)"),
        "Expected clipboard footer to show secondary actions only when an entry is selected"
    );
    assert!(
        content.contains("selected_entry_for_footer"),
        "Expected clipboard footer to capture selected_entry for actions callback"
    );
    assert!(
        content.contains("toggle_clipboard_actions(entry, window, cx)"),
        "Expected clipboard footer actions callback to call toggle_clipboard_actions with entry"
    );
}
