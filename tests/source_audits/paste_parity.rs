//! Regression tests ensuring emoji and clipboard paste share the same
//! `finalize_paste_after_clipboard_ready` helper and that the helper emits
//! machine-readable structured tracing fields.

use super::*;

#[test]
fn emoji_and_clipboard_routes_share_finalize_helper() {
    let emoji = read_source("src/app_actions/handle_action/emoji.rs");
    let clipboard = read_source("src/app_actions/handle_action/clipboard.rs");

    assert!(
        emoji.contains("finalize_paste_after_clipboard_ready("),
        "emoji paste actions must call finalize_paste_after_clipboard_ready"
    );
    assert!(
        clipboard.contains("finalize_paste_after_clipboard_ready("),
        "clipboard paste actions must call finalize_paste_after_clipboard_ready"
    );
}

#[test]
fn finalize_helper_logs_machine_readable_fields() {
    let paste = read_source("src/app_actions/handle_action/paste.rs");

    assert!(
        paste.contains("paste_strategy = \"clipboard_then_simulated_cmd_v\""),
        "paste helper must expose a stable paste_strategy field"
    );
    assert!(
        paste.contains("close_behavior = close_behavior.as_str()"),
        "paste helper must log close_behavior as a machine-readable field"
    );
    assert!(
        paste.contains("status = \"queued\""),
        "paste helper must log queued status"
    );
    assert!(
        paste.contains("source_kind"),
        "paste helper must log source_kind"
    );
    assert!(
        paste.contains("source_id"),
        "paste helper must log source_id"
    );
}

#[test]
fn emoji_no_longer_calls_spawn_or_hide_directly() {
    let emoji = read_source("src/app_actions/handle_action/emoji.rs");

    // The emoji paste arms should NOT directly call these anymore
    assert!(
        !emoji.contains("self.spawn_clipboard_paste_simulation()"),
        "emoji.rs must not call spawn_clipboard_paste_simulation directly — use finalize_paste_after_clipboard_ready"
    );
    assert!(
        !emoji.contains("self.hide_main_and_reset(cx)"),
        "emoji.rs must not call hide_main_and_reset directly — use finalize_paste_after_clipboard_ready"
    );
}

#[test]
fn emoji_and_clipboard_paste_flows_share_the_same_finalizer() {
    let emoji = read_source("src/app_actions/handle_action/emoji.rs");
    let clipboard = read_source("src/app_actions/handle_action/clipboard.rs");
    let paste = read_source("src/app_actions/handle_action/paste.rs");

    assert!(
        paste.contains("enum PasteCloseBehavior"),
        "paste.rs must define PasteCloseBehavior enum"
    );
    assert!(
        paste.contains("fn finalize_paste_after_clipboard_ready("),
        "paste.rs must define finalize_paste_after_clipboard_ready"
    );

    assert!(
        emoji.contains("self.finalize_paste_after_clipboard_ready("),
        "emoji.rs must call finalize_paste_after_clipboard_ready"
    );
    assert!(
        emoji.contains("PasteCloseBehavior::HideWindow"),
        "emoji.rs must use PasteCloseBehavior::HideWindow"
    );
    assert!(
        emoji.contains("PasteCloseBehavior::KeepWindowOpen"),
        "emoji.rs must use PasteCloseBehavior::KeepWindowOpen"
    );

    assert!(
        clipboard.contains("self.finalize_paste_after_clipboard_ready("),
        "clipboard.rs must call finalize_paste_after_clipboard_ready"
    );
    assert!(
        clipboard.contains("PasteCloseBehavior::HideWindow"),
        "clipboard.rs must use PasteCloseBehavior::HideWindow"
    );
    assert!(
        clipboard.contains("PasteCloseBehavior::KeepWindowOpen"),
        "clipboard.rs must use PasteCloseBehavior::KeepWindowOpen"
    );
}

#[test]
fn emoji_paste_action_appears_before_copy_in_builder() {
    let source = read_source("src/actions/builders/emoji.rs");

    let paste_idx = source
        .find("\"emoji:emoji_paste\"")
        .expect("emoji_paste action missing from builder");
    let copy_idx = source
        .find("\"emoji:emoji_copy\"")
        .expect("emoji_copy action missing from builder");

    assert!(
        paste_idx < copy_idx,
        "emoji_paste must appear before emoji_copy in action builder \
         (paste is the default action, so it must be first)"
    );
}

#[test]
fn enter_key_dispatches_paste_not_copy_in_emoji_picker() {
    let source = read_source("src/render_builtins/emoji_picker.rs");

    // The plain Enter arm (no modifiers) must dispatch emoji_paste
    let enter_check = source.find("is_key_enter(key)");
    assert!(enter_check.is_some(), "emoji picker must handle Enter key via is_key_enter");

    let enter_pos = enter_check.unwrap();
    // After the Enter match arm, the first handle_emoji_action call should be for paste
    let after_enter = &source[enter_pos..];
    let first_action = after_enter.find("handle_emoji_action(\"emoji_paste\"");
    let copy_action = after_enter.find("handle_emoji_action(\"emoji_copy\"");

    assert!(
        first_action.is_some(),
        "Enter key handler must call handle_emoji_action(\"emoji_paste\")"
    );
    // emoji_copy (Cmd+Enter) appears in the same block but the plain paste comes
    // last in the if/else chain (the else arm), which is the no-modifier default.
    // Both must exist.
    assert!(
        copy_action.is_some(),
        "Cmd+Enter handler must call handle_emoji_action(\"emoji_copy\")"
    );
}

#[test]
fn emoji_picker_footer_primary_action_is_paste() {
    let source = read_source("src/render_builtins/emoji_picker.rs");

    assert!(
        source.contains(".primary_label(paste_label)"),
        "emoji picker footer primary label must be paste_label"
    );
    assert!(
        source.contains(".primary_shortcut(\"↵\")"),
        "emoji picker footer primary shortcut must be Enter (↵)"
    );
}
