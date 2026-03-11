//! Tests for SDK action routing and feedback patterns used by builtin execution.
//!
//! These tests exercise the action_helpers API behaviorally rather than scanning
//! source code for string patterns.

use script_kit_gpui::action_helpers::{
    find_sdk_action, trigger_sdk_action, SdkActionResult, RESERVED_ACTION_IDS, ERROR_CANCELLED,
    ERROR_CHANNEL_DISCONNECTED, ERROR_CHANNEL_FULL, ERROR_NO_SENDER,
};
use script_kit_gpui::protocol::{self, ProtocolAction};
use std::sync::mpsc;

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
// Reserved action IDs — prevent SDK from overriding builtins
// ---------------------------------------------------------------------------

#[test]
fn reserved_action_list_contains_system_actions() {
    let expected = ["empty_trash", "lock_screen", "sleep", "restart"];
    // System actions are NOT in the reserved list — they go through dispatch_system_action,
    // not SDK actions. Verify that file/script context actions ARE reserved.
    for id in &[
        "open_file",
        "open_directory",
        "quick_look",
        "show_info",
        "copy_filename",
    ] {
        assert!(
            RESERVED_ACTION_IDS.contains(id),
            "File search action '{id}' should be reserved"
        );
    }
    // System actions are NOT reserved (handled via separate dispatch path)
    for id in &expected {
        assert!(
            !RESERVED_ACTION_IDS.contains(id),
            "System action '{id}' should NOT be in RESERVED_ACTION_IDS (separate dispatch)"
        );
    }
}

#[test]
fn reserved_actions_include_path_prompt_actions() {
    for id in &[
        "select_file",
        "open_in_finder",
        "open_in_editor",
        "open_in_terminal",
        "move_to_trash",
    ] {
        assert!(
            RESERVED_ACTION_IDS.contains(id),
            "Path prompt action '{id}' should be reserved"
        );
    }
}

// ---------------------------------------------------------------------------
// SDK action lookup with shadowing detection
// ---------------------------------------------------------------------------

#[test]
fn find_sdk_action_finds_matching_action_by_name() {
    let actions = vec![
        make_action("alpha", true, None),
        make_action("beta", false, Some("val")),
    ];

    let found = find_sdk_action(Some(&actions), "beta", false);
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "beta");
    assert_eq!(found.unwrap().value, Some("val".to_string()));
}

#[test]
fn find_sdk_action_returns_none_for_undefined_action() {
    let actions = vec![make_action("alpha", true, None)];
    assert!(find_sdk_action(Some(&actions), "gamma", false).is_none());
}

#[test]
fn find_sdk_action_returns_none_when_no_actions_defined() {
    assert!(find_sdk_action(None, "anything", false).is_none());
}

// ---------------------------------------------------------------------------
// Action dispatch — message routing correctness
// ---------------------------------------------------------------------------

#[test]
fn action_with_handler_sends_action_triggered_with_correct_fields() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("confirm_delete", true, Some("item_99"));

    let result = trigger_sdk_action("confirm_delete", &action, "search query", Some(&tx));
    assert_eq!(result, SdkActionResult::Sent);

    match rx.try_recv().expect("expected message") {
        protocol::Message::ActionTriggered {
            action,
            value,
            input,
        } => {
            assert_eq!(action, "confirm_delete");
            assert_eq!(value, Some("item_99".to_string()));
            assert_eq!(input, "search query");
        }
        other => panic!("Expected ActionTriggered, got {:?}", other),
    }
}

#[test]
fn action_without_handler_submits_value_as_submit_message() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    let action = make_action("pick_option", false, Some("option_a"));

    let result = trigger_sdk_action("pick_option", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::Sent);

    match rx.try_recv().expect("expected message") {
        protocol::Message::Submit { id, value } => {
            assert_eq!(id, "action");
            assert_eq!(value, Some("option_a".to_string()));
        }
        other => panic!("Expected Submit, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Error feedback — all error paths produce user-visible messages
// ---------------------------------------------------------------------------

#[test]
fn channel_full_error_includes_action_name_in_message() {
    let result = SdkActionResult::ChannelFull;
    let msg = result
        .error_message("run_build")
        .expect("ChannelFull should produce error message");
    assert!(
        msg.contains("run_build"),
        "Error should include action name"
    );
    assert!(
        msg.contains("channel is busy"),
        "Error should explain the failure"
    );
}

#[test]
fn channel_disconnected_error_mentions_script_exit() {
    let result = SdkActionResult::ChannelDisconnected;
    let msg = result
        .error_message("late_action")
        .expect("Disconnected should produce error message");
    assert!(msg.contains("script has exited"));
}

#[test]
fn no_sender_error_mentions_no_active_script() {
    let result = SdkActionResult::NoSender;
    let msg = result
        .error_message("orphan")
        .expect("NoSender should produce error message");
    assert!(msg.contains("no active script"));
}

#[test]
fn sent_and_no_effect_produce_no_error_message() {
    assert!(SdkActionResult::Sent.error_message("x").is_none());
    assert!(SdkActionResult::NoEffect.error_message("x").is_none());
}

// ---------------------------------------------------------------------------
// End-to-end: channel error detection
// ---------------------------------------------------------------------------

#[test]
fn full_channel_is_detected_and_reported() {
    let (tx, _rx) = mpsc::sync_channel::<protocol::Message>(0);
    let action = make_action("trigger", true, None);

    let result = trigger_sdk_action("trigger", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::ChannelFull);
    assert!(!result.is_sent());
}

#[test]
fn disconnected_channel_is_detected_and_reported() {
    let (tx, rx) = mpsc::sync_channel::<protocol::Message>(10);
    drop(rx);
    let action = make_action("trigger", true, None);

    let result = trigger_sdk_action("trigger", &action, "", Some(&tx));
    assert_eq!(result, SdkActionResult::ChannelDisconnected);
    assert!(!result.is_sent());
}

// ---------------------------------------------------------------------------
// Error code stability — builtin execution relies on stable codes
// ---------------------------------------------------------------------------

#[test]
fn error_codes_are_stable_lowercase_snake_case_strings() {
    assert_eq!(ERROR_NO_SENDER, "no_sender");
    assert_eq!(ERROR_CHANNEL_FULL, "channel_full");
    assert_eq!(ERROR_CHANNEL_DISCONNECTED, "channel_disconnected");
    assert_eq!(ERROR_CANCELLED, "cancelled");
}

#[test]
fn cancelled_result_has_code_but_no_user_error() {
    let result = SdkActionResult::Cancelled;
    // Machine-readable: has a code
    assert_eq!(result.error_code(), Some(ERROR_CANCELLED));
    // Not a user error: no toast message
    assert!(result.error_message("force_quit").is_none());
    // Not sent
    assert!(!result.is_sent());
}

#[test]
fn all_error_variants_produce_distinct_error_codes() {
    let codes: Vec<Option<&str>> = vec![
        SdkActionResult::NoSender.error_code(),
        SdkActionResult::ChannelFull.error_code(),
        SdkActionResult::ChannelDisconnected.error_code(),
        SdkActionResult::Cancelled.error_code(),
    ];

    // All should be Some
    for code in &codes {
        assert!(code.is_some(), "All non-success variants must have error codes");
    }

    // All should be distinct
    let mut unique: Vec<&str> = codes.iter().map(|c| c.expect("checked above")).collect();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        codes.len(),
        "All error codes must be distinct"
    );
}
