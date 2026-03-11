//! Tests for action dispatch patterns, SDK action routing, and error feedback.
//!
//! These tests exercise the actual `action_helpers` API rather than scanning source
//! code for string patterns — ensuring behavioral correctness, not just code presence.

use script_kit_gpui::action_helpers::{
    extract_path_for_copy, extract_path_for_edit, extract_path_for_reveal, find_sdk_action,
    is_reserved_action_id, trigger_sdk_action, PathExtractionError, SdkActionResult,
};
use script_kit_gpui::protocol::{self, ProtocolAction};
use std::sync::mpsc;

// ---------------------------------------------------------------------------
// Helper to build a ProtocolAction
// ---------------------------------------------------------------------------

fn make_action(name: &str, has_action: bool, value: Option<&str>) -> ProtocolAction {
    ProtocolAction {
        name: name.to_string(),
        description: None,
        shortcut: None,
        value: value.map(|v| v.to_string()),
        has_action,
        visible: None,
        close: None,
    }
}

// ---------------------------------------------------------------------------
// SDK action dispatch — successful action trigger
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_with_handler_sends_action_triggered_message() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("refresh", true, Some("all"));

    let result = trigger_sdk_action("refresh", &action, "query text", Some(&tx));
    assert_eq!(result, SdkActionResult::Sent);

    match rx.try_recv().expect("expected a message") {
        protocol::Message::ActionTriggered {
            action,
            value,
            input,
        } => {
            assert_eq!(action, "refresh");
            assert_eq!(value, Some("all".to_string()));
            assert_eq!(input, "query text");
        }
        other => panic!("Expected ActionTriggered, got {:?}", other),
    }
}

#[test]
fn sdk_action_without_handler_submits_value() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("select_item", false, Some("item_42"));

    let result = trigger_sdk_action("select_item", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::Sent);

    match rx.try_recv().expect("expected a message") {
        protocol::Message::Submit { id, value } => {
            assert_eq!(id, "action");
            assert_eq!(value, Some("item_42".to_string()));
        }
        other => panic!("Expected Submit, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// SDK action dispatch — channel error feedback
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_on_full_channel_returns_error_with_message() {
    // Capacity 0: try_send always fails with Full
    let (tx, _rx) = mpsc::sync_channel::<protocol::Message>(0);
    let action = make_action("slow_action", true, None);

    let result = trigger_sdk_action("slow_action", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::ChannelFull);

    let msg = result.error_message("slow_action").expect("should have error message");
    assert!(
        msg.contains("channel busy"),
        "Error should mention channel busy: {msg}"
    );
}

#[test]
fn sdk_action_on_disconnected_channel_returns_error_with_message() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    drop(rx); // Simulate script exit

    let action = make_action("post_exit", true, Some("val"));

    let result = trigger_sdk_action("post_exit", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::ChannelDisconnected);

    let msg = result.error_message("post_exit").expect("should have error message");
    assert!(
        msg.contains("script has exited"),
        "Error should mention script exited: {msg}"
    );
}

#[test]
fn sdk_action_without_sender_returns_no_sender_error() {
    let action = make_action("orphan", true, Some("val"));

    let result = trigger_sdk_action("orphan", &action, "", None);
    assert_eq!(result, SdkActionResult::NoSender);
    assert!(result.error_message("orphan").is_some());
}

// ---------------------------------------------------------------------------
// SDK action dispatch — unknown / no-effect actions
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_with_no_handler_and_no_value_returns_no_effect() {
    let (tx, _rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("noop", false, None);

    let result = trigger_sdk_action("noop", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::NoEffect);
    // NoEffect is not an error — no toast should be shown
    assert!(result.error_message("noop").is_none());
}

#[test]
fn find_sdk_action_returns_none_for_missing_action() {
    let actions = vec![make_action("exists", true, None)];
    assert!(find_sdk_action(Some(&actions), "missing", false).is_none());
}

#[test]
fn find_sdk_action_returns_none_when_actions_list_is_none() {
    assert!(find_sdk_action(None, "anything", false).is_none());
}

// ---------------------------------------------------------------------------
// Reserved action IDs — built-in actions cannot be shadowed
// ---------------------------------------------------------------------------

#[test]
fn reserved_action_ids_include_core_script_actions() {
    for id in &["run_script", "edit_script", "copy_path", "reveal_in_finder"] {
        assert!(
            is_reserved_action_id(id),
            "'{id}' should be a reserved action ID"
        );
    }
}

#[test]
fn custom_action_ids_are_not_reserved() {
    assert!(!is_reserved_action_id("my_custom_action"));
    assert!(!is_reserved_action_id(""));
    assert!(!is_reserved_action_id("foobar"));
}

// ---------------------------------------------------------------------------
// Path extraction — guards for unsupported types
// ---------------------------------------------------------------------------

#[test]
fn path_extraction_returns_no_selection_for_none() {
    assert_eq!(
        extract_path_for_reveal(None).unwrap_err(),
        PathExtractionError::NoSelection
    );
    assert_eq!(
        extract_path_for_copy(None).unwrap_err(),
        PathExtractionError::NoSelection
    );
    assert_eq!(
        extract_path_for_edit(None).unwrap_err(),
        PathExtractionError::NoSelection
    );
}

// ---------------------------------------------------------------------------
// SdkActionResult — error message formatting
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_result_sent_has_no_error_message() {
    assert!(SdkActionResult::Sent.error_message("x").is_none());
}

#[test]
fn sdk_action_result_error_messages_include_action_name() {
    let msg = SdkActionResult::ChannelFull
        .error_message("delete_item")
        .unwrap();
    assert!(msg.contains("delete_item"));

    let msg = SdkActionResult::NoSender
        .error_message("save_draft")
        .unwrap();
    assert!(msg.contains("save_draft"));
}

// ---------------------------------------------------------------------------
// SdkActionResult — is_sent consistency across all variants
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_result_is_sent_only_true_for_sent_variant() {
    assert!(SdkActionResult::Sent.is_sent(), "Sent should be is_sent()");
    assert!(
        !SdkActionResult::NoSender.is_sent(),
        "NoSender should not be is_sent()"
    );
    assert!(
        !SdkActionResult::NoEffect.is_sent(),
        "NoEffect should not be is_sent()"
    );
    assert!(
        !SdkActionResult::ChannelFull.is_sent(),
        "ChannelFull should not be is_sent()"
    );
    assert!(
        !SdkActionResult::ChannelDisconnected.is_sent(),
        "ChannelDisconnected should not be is_sent()"
    );
}

// ---------------------------------------------------------------------------
// SdkActionResult — NoEffect is never an error (no toast)
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_result_no_effect_has_no_error_message() {
    assert!(
        SdkActionResult::NoEffect.error_message("any").is_none(),
        "NoEffect should not produce an error message"
    );
}

// ---------------------------------------------------------------------------
// Reserved action IDs — coverage for all categories
// ---------------------------------------------------------------------------

#[test]
fn reserved_action_ids_include_file_search_actions() {
    for id in &["open_file", "open_directory", "quick_look", "open_with", "show_info", "copy_filename"] {
        assert!(
            is_reserved_action_id(id),
            "'{id}' should be a reserved file search action ID"
        );
    }
}

#[test]
fn reserved_action_ids_include_path_prompt_actions() {
    for id in &["select_file", "open_in_finder", "open_in_editor", "open_in_terminal", "move_to_trash"] {
        assert!(
            is_reserved_action_id(id),
            "'{id}' should be a reserved path prompt action ID"
        );
    }
}

#[test]
fn reserved_action_ids_include_shortcut_actions() {
    for id in &["add_shortcut", "update_shortcut", "remove_shortcut"] {
        assert!(
            is_reserved_action_id(id),
            "'{id}' should be a reserved shortcut action ID"
        );
    }
}

#[test]
fn reserved_action_ids_include_internal_cancel() {
    assert!(
        is_reserved_action_id("__cancel__"),
        "__cancel__ should be reserved"
    );
}

// ---------------------------------------------------------------------------
// find_sdk_action — multi-action list, first match wins
// ---------------------------------------------------------------------------

#[test]
fn find_sdk_action_returns_first_matching_action() {
    let actions = vec![
        make_action("duplicate", true, Some("first")),
        make_action("duplicate", false, Some("second")),
        make_action("other", true, None),
    ];

    let found = find_sdk_action(Some(&actions), "duplicate", false)
        .expect("should find 'duplicate' action");
    assert_eq!(found.value, Some("first".to_string()), "should return the first match");
}

#[test]
fn find_sdk_action_empty_list_returns_none() {
    let actions: Vec<ProtocolAction> = vec![];
    assert!(find_sdk_action(Some(&actions), "anything", false).is_none());
}

// ---------------------------------------------------------------------------
// PathExtractionError — message accessors
// ---------------------------------------------------------------------------

#[test]
fn path_extraction_error_no_selection_message_is_stable() {
    let err = PathExtractionError::NoSelection;
    assert_eq!(err.message().as_ref(), "No item selected");
}

#[test]
fn path_extraction_error_unsupported_type_preserves_custom_message() {
    let custom = "Cannot frobnicate this item type";
    let err = PathExtractionError::UnsupportedType(custom.into());
    assert_eq!(err.message().as_ref(), custom);
}

// ---------------------------------------------------------------------------
// SDK action dispatch — action_triggered carries current_input through
// ---------------------------------------------------------------------------

#[test]
fn sdk_action_triggered_includes_current_input_text() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("search", true, None);

    let result = trigger_sdk_action("search", &action, "hello world", Some(&tx));
    assert_eq!(result, SdkActionResult::Sent);

    match rx.try_recv().expect("expected a message") {
        protocol::Message::ActionTriggered { input, .. } => {
            assert_eq!(input, "hello world", "current input should be passed through");
        }
        other => panic!("Expected ActionTriggered, got {:?}", other),
    }
}

#[test]
fn sdk_action_triggered_with_empty_input_and_no_value() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("bare", true, None);

    let result = trigger_sdk_action("bare", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::Sent);

    match rx.try_recv().expect("expected a message") {
        protocol::Message::ActionTriggered {
            action,
            value,
            input,
        } => {
            assert_eq!(action, "bare");
            assert_eq!(value, None, "no value should be None");
            assert_eq!(input, "", "empty input should be empty string");
        }
        other => panic!("Expected ActionTriggered, got {:?}", other),
    }
}
