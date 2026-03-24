//! Tests for action_helpers module.
//!
//! Covers all SearchResult variants (Script, Scriptlet, BuiltIn, App, Agent, Window, Fallback)
//! for extract_path_for_reveal, extract_path_for_copy, and extract_path_for_edit.

use super::*;
use crate::agents::Agent;
use crate::app_launcher;
use crate::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};
use crate::fallbacks::{BuiltinFallback, FallbackAction, FallbackCondition, FallbackItem};
use crate::scripts::{
    AgentMatch, AppMatch, BuiltInMatch, FallbackMatch, MatchIndices, Script, ScriptMatch,
    Scriptlet, ScriptletMatch, WindowMatch,
};
use crate::window_control;
use std::path::PathBuf;
use std::sync::Arc;

fn make_script(name: &str, path: &str) -> Arc<Script> {
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(path),
        extension: "ts".to_string(),
        description: None,
        icon: None,
        alias: None,
        shortcut: None,
        typed_metadata: None,
        schema: None,
        kit_name: None,
    })
}

fn make_script_match(name: &str, path: &str) -> ScriptMatch {
    ScriptMatch {
        script: make_script(name, path),
        score: 100,
        filename: format!("{}.ts", name),
        match_indices: MatchIndices::default(),
    }
}

fn make_scriptlet_match() -> ScriptletMatch {
    ScriptletMatch {
        scriptlet: Arc::new(Scriptlet {
            name: "test-scriptlet".to_string(),
            description: None,
            code: "console.log('test')".to_string(),
            tool: "ts".to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            file_path: None,
            command: None,
            alias: None,
        }),
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    }
}

fn make_builtin_match() -> BuiltInMatch {
    BuiltInMatch {
        entry: BuiltInEntry {
            id: "clipboard_history".to_string(),
            name: "Clipboard History".to_string(),
            description: "View clipboard history".to_string(),
            keywords: vec!["clipboard".to_string()],
            feature: BuiltInFeature::ClipboardHistory,
            icon: None,
            group: BuiltInGroup::Core,
        },
        score: 100,
    }
}

fn make_app_match(name: &str, path: &str) -> AppMatch {
    AppMatch {
        app: app_launcher::AppInfo {
            name: name.to_string(),
            path: PathBuf::from(path),
            bundle_id: Some("com.example.app".to_string()),
            icon: None,
        },
        score: 100,
    }
}

fn make_agent_match(name: &str, path: &str) -> AgentMatch {
    AgentMatch {
        agent: Arc::new(Agent {
            name: name.to_string(),
            path: PathBuf::from(path),
            ..Agent::default()
        }),
        score: 100,
        display_name: name.to_string(),
        match_indices: MatchIndices::default(),
    }
}

fn make_window_match() -> WindowMatch {
    WindowMatch {
        window: window_control::WindowInfo::for_test(
            1,
            "Test App".to_string(),
            "Test Window".to_string(),
            window_control::Bounds::new(0, 0, 800, 600),
            1234,
        ),
        score: 100,
    }
}

fn make_fallback_match() -> FallbackMatch {
    FallbackMatch {
        fallback: FallbackItem::Builtin(BuiltinFallback {
            id: "search_google",
            name: "Search Google",
            description: "Search Google for query",
            icon: "search",
            action: FallbackAction::CopyToClipboard,
            condition: FallbackCondition::Always,
            enabled: true,
            priority: 0,
        }),
        score: 0,
    }
}

// Tests for extract_path_for_reveal

#[test]
fn test_extract_path_for_reveal_none() {
    let result = extract_path_for_reveal(None);
    assert!(matches!(result, Err(PathExtractionError::NoSelection)));
    assert_eq!(result.unwrap_err().message().as_ref(), "No item selected");
}

#[test]
fn test_extract_path_for_reveal_script() {
    let script_match = make_script_match("test", "/path/to/test.ts");
    let result = extract_path_for_reveal(Some(&SearchResult::Script(script_match)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/path/to/test.ts"));
}

#[test]
fn test_extract_path_for_reveal_scriptlet() {
    let scriptlet_match = make_scriptlet_match();
    let result = extract_path_for_reveal(Some(&SearchResult::Scriptlet(scriptlet_match)));
    assert!(matches!(
        result,
        Err(PathExtractionError::UnsupportedType(_))
    ));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot reveal scriptlets in Finder"
    );
}

#[test]
fn test_extract_path_for_reveal_builtin() {
    let builtin_match = make_builtin_match();
    let result = extract_path_for_reveal(Some(&SearchResult::BuiltIn(builtin_match)));
    assert!(matches!(
        result,
        Err(PathExtractionError::UnsupportedType(_))
    ));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot reveal built-in features"
    );
}

// Tests for extract_path_for_reveal — App, Agent, Window, Fallback variants

#[test]
fn test_extract_path_for_reveal_app() {
    let app_match = make_app_match("Safari", "/Applications/Safari.app");
    let result = extract_path_for_reveal(Some(&SearchResult::App(app_match)));
    assert_eq!(
        result.expect("App should be revealable"),
        PathBuf::from("/Applications/Safari.app")
    );
}

#[test]
fn test_extract_path_for_reveal_agent() {
    let agent_match = make_agent_match(
        "my-agent",
        "/Users/test/.scriptkit/agents/my-agent.claude.md",
    );
    let result = extract_path_for_reveal(Some(&SearchResult::Agent(agent_match)));
    assert_eq!(
        result.expect("Agent should be revealable"),
        PathBuf::from("/Users/test/.scriptkit/agents/my-agent.claude.md")
    );
}

#[test]
fn test_extract_path_for_reveal_window() {
    let window_match = make_window_match();
    let result = extract_path_for_reveal(Some(&SearchResult::Window(window_match)));
    assert!(
        matches!(result, Err(PathExtractionError::UnsupportedType(_))),
        "Window should not be revealable"
    );
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot reveal windows in Finder"
    );
}

#[test]
fn test_extract_path_for_reveal_fallback() {
    let fallback_match = make_fallback_match();
    let result = extract_path_for_reveal(Some(&SearchResult::Fallback(fallback_match)));
    assert!(
        matches!(result, Err(PathExtractionError::UnsupportedType(_))),
        "Fallback should not be revealable"
    );
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot reveal fallback commands in Finder"
    );
}

// Tests for extract_path_for_copy

#[test]
fn test_extract_path_for_copy_none() {
    let result = extract_path_for_copy(None);
    assert!(matches!(result, Err(PathExtractionError::NoSelection)));
}

#[test]
fn test_extract_path_for_copy_script() {
    let script_match = make_script_match("test", "/path/to/test.ts");
    let result = extract_path_for_copy(Some(&SearchResult::Script(script_match)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/path/to/test.ts"));
}

#[test]
fn test_extract_path_for_copy_scriptlet() {
    let scriptlet_match = make_scriptlet_match();
    let result = extract_path_for_copy(Some(&SearchResult::Scriptlet(scriptlet_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot copy scriptlet path"
    );
}

// Tests for extract_path_for_copy — App, Agent, Window, Fallback variants

#[test]
fn test_extract_path_for_copy_app() {
    let app_match = make_app_match("Safari", "/Applications/Safari.app");
    let result = extract_path_for_copy(Some(&SearchResult::App(app_match)));
    assert_eq!(
        result.expect("App path should be copyable"),
        PathBuf::from("/Applications/Safari.app")
    );
}

#[test]
fn test_extract_path_for_copy_agent() {
    let agent_match = make_agent_match("my-agent", "/tmp/agents/my-agent.md");
    let result = extract_path_for_copy(Some(&SearchResult::Agent(agent_match)));
    assert_eq!(
        result.expect("Agent path should be copyable"),
        PathBuf::from("/tmp/agents/my-agent.md")
    );
}

#[test]
fn test_extract_path_for_copy_window() {
    let window_match = make_window_match();
    let result = extract_path_for_copy(Some(&SearchResult::Window(window_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot copy window path"
    );
}

#[test]
fn test_extract_path_for_copy_fallback() {
    let fallback_match = make_fallback_match();
    let result = extract_path_for_copy(Some(&SearchResult::Fallback(fallback_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot copy fallback command path"
    );
}

#[test]
fn test_extract_path_for_copy_builtin() {
    let builtin_match = make_builtin_match();
    let result = extract_path_for_copy(Some(&SearchResult::BuiltIn(builtin_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot copy built-in path"
    );
}

// Tests for extract_path_for_edit

#[test]
fn test_extract_path_for_edit_none() {
    let result = extract_path_for_edit(None);
    assert!(matches!(result, Err(PathExtractionError::NoSelection)));
}

#[test]
fn test_extract_path_for_edit_script() {
    let script_match = make_script_match("test", "/path/to/test.ts");
    let result = extract_path_for_edit(Some(&SearchResult::Script(script_match)));
    assert!(result.is_ok());
}

#[test]
fn test_extract_path_for_edit_scriptlet() {
    let scriptlet_match = make_scriptlet_match();
    let result = extract_path_for_edit(Some(&SearchResult::Scriptlet(scriptlet_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot edit scriptlets"
    );
}

// Tests for extract_path_for_edit — App, Agent, Window, Fallback, BuiltIn variants

#[test]
fn test_extract_path_for_edit_agent() {
    let agent_match = make_agent_match("my-agent", "/tmp/agents/my-agent.claude.md");
    let result = extract_path_for_edit(Some(&SearchResult::Agent(agent_match)));
    assert_eq!(
        result.expect("Agent should be editable"),
        PathBuf::from("/tmp/agents/my-agent.claude.md")
    );
}

#[test]
fn test_extract_path_for_edit_app() {
    let app_match = make_app_match("Safari", "/Applications/Safari.app");
    let result = extract_path_for_edit(Some(&SearchResult::App(app_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot edit applications"
    );
}

#[test]
fn test_extract_path_for_edit_window() {
    let window_match = make_window_match();
    let result = extract_path_for_edit(Some(&SearchResult::Window(window_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot edit windows"
    );
}

#[test]
fn test_extract_path_for_edit_fallback() {
    let fallback_match = make_fallback_match();
    let result = extract_path_for_edit(Some(&SearchResult::Fallback(fallback_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot edit fallback commands"
    );
}

#[test]
fn test_extract_path_for_edit_builtin() {
    let builtin_match = make_builtin_match();
    let result = extract_path_for_edit(Some(&SearchResult::BuiltIn(builtin_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot edit built-in features"
    );
}

// Tests for reserved action IDs

#[test]
fn reserved_action_ids_include_visible() {
    use std::collections::BTreeSet;

    let expected: BTreeSet<&str> = [
        "__cancel__",
        "add_alias",
        "add_shortcut",
        "attach_to_ai",
        "configure_shortcut",
        "copy_deeplink",
        "copy_path",
        "copy_scriptlet_path",
        "delete_script",
        "edit_script",
        "edit_scriptlet",
        "open_directory",
        "open_file",
        "open_with",
        "quick_look",
        "reload_scripts",
        "remove_alias",
        "remove_script",
        "reveal_in_finder",
        "reveal_scriptlet_in_finder",
        "show_info",
        "update_alias",
        "update_shortcut",
        "view_logs",
    ]
    .into_iter()
    .collect();

    let actual: BTreeSet<&str> = RESERVED_ACTION_IDS.iter().copied().collect();
    let missing: Vec<&str> = expected.difference(&actual).copied().collect();

    assert!(
        missing.is_empty(),
        "Missing reserved local action ids: {missing:?}"
    );
}

#[test]
fn reserved_action_ids_are_unique() {
    use std::collections::BTreeSet;

    let unique: BTreeSet<&str> = RESERVED_ACTION_IDS.iter().copied().collect();

    assert_eq!(
        unique.len(),
        RESERVED_ACTION_IDS.len(),
        "Duplicate ids in RESERVED_ACTION_IDS"
    );
}

#[test]
fn test_is_reserved_action_id() {
    assert!(is_reserved_action_id("copy_path"));
    assert!(is_reserved_action_id("edit_script"));
    assert!(is_reserved_action_id("copy_deeplink"));
    assert!(is_reserved_action_id("__cancel__"));

    assert!(!is_reserved_action_id("custom_action"));
    assert!(!is_reserved_action_id("quit")); // quit is no longer reserved (main menu only)
    assert!(!is_reserved_action_id(""));
}

#[test]
fn test_find_sdk_action_none() {
    let result = find_sdk_action(None, "test", false);
    assert!(result.is_none());
}

#[test]
fn test_find_sdk_action_found() {
    let actions = vec![
        ProtocolAction {
            name: "test_action".to_string(),
            description: Some("Test".to_string()),
            shortcut: None,
            value: Some("value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        },
        ProtocolAction {
            name: "other_action".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: false,
            visible: None,
            close: None,
        },
    ];

    let result = find_sdk_action(Some(&actions), "test_action", false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "test_action");

    let result = find_sdk_action(Some(&actions), "not_found", false);
    assert!(result.is_none());
}

// Tests for trigger_sdk_action

#[test]
fn trigger_sdk_action_returns_no_sender_when_sender_is_none() {
    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: Some("value".to_string()),
        has_action: true,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "", None, "test-trace");
    assert_eq!(result, SdkActionResult::NoSender);
    assert!(!result.is_sent());
    assert!(result.error_message("test").is_some());
}

#[test]
fn trigger_sdk_action_sends_action_triggered_when_has_action() {
    use std::sync::mpsc;

    let (sender, receiver) = mpsc::sync_channel::<protocol::Message>(10);

    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: Some("value".to_string()),
        has_action: true,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action(
        "test",
        &action,
        "current input",
        Some(&sender),
        "test-trace",
    );
    assert_eq!(result, SdkActionResult::Sent);
    assert!(result.is_sent());
    assert!(result.error_message("test").is_none());

    let msg = receiver.try_recv().unwrap();
    match msg {
        protocol::Message::ActionTriggered {
            action,
            value,
            input,
        } => {
            assert_eq!(action, "test");
            assert_eq!(value, Some("value".to_string()));
            assert_eq!(input, "current input");
        }
        _ => panic!("Expected ActionTriggered message, got {:?}", msg),
    }
}

#[test]
fn trigger_sdk_action_sends_submit_when_no_handler_but_has_value() {
    use std::sync::mpsc;

    let (sender, receiver) = mpsc::sync_channel::<protocol::Message>(10);

    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: Some("submit_value".to_string()),
        has_action: false,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "", Some(&sender), "test-trace");
    assert_eq!(result, SdkActionResult::Sent);

    let msg = receiver.try_recv().unwrap();
    match msg {
        protocol::Message::Submit { id, value } => {
            assert_eq!(id, "action");
            assert_eq!(value, Some("submit_value".to_string()));
        }
        _ => panic!("Expected Submit message, got {:?}", msg),
    }
}

#[test]
fn trigger_sdk_action_returns_no_effect_when_no_handler_no_value() {
    use std::sync::mpsc;

    let (sender, _receiver) = mpsc::sync_channel::<protocol::Message>(10);

    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "", Some(&sender), "test-trace");
    assert_eq!(result, SdkActionResult::NoEffect);
    assert!(!result.is_sent());
    // NoEffect is not an error — no Toast needed
    assert!(result.error_message("test").is_none());
}

// Tests for Cancelled variant behavior — modal dismissal is not an error

#[test]
fn cancelled_is_distinct_from_error_variants() {
    // Cancelled has error_code (for machine consumption) but no error_message (no toast)
    let cancelled = SdkActionResult::Cancelled;
    let no_sender = SdkActionResult::NoSender;

    // Both have error codes
    assert!(cancelled.error_code().is_some());
    assert!(no_sender.error_code().is_some());

    // But only the real error has an error message
    assert!(cancelled.error_message("test").is_none());
    assert!(no_sender.error_message("test").is_some());

    // And their codes are different
    assert_ne!(cancelled.error_code(), no_sender.error_code());
}

#[test]
fn trigger_sdk_action_returns_channel_full_when_buffer_exhausted() {
    use std::sync::mpsc;

    // Buffer size 0 means try_send always fails with Full
    let (sender, _receiver) = mpsc::sync_channel::<protocol::Message>(0);

    let action = ProtocolAction {
        name: "busy_action".to_string(),
        description: None,
        shortcut: None,
        value: None,
        has_action: true,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("busy_action", &action, "", Some(&sender), "test-trace");
    assert_eq!(result, SdkActionResult::ChannelFull);
    assert!(!result.is_sent());
    let err = result.error_message("busy_action").unwrap();
    assert!(
        err.contains("channel is busy"),
        "Expected 'channel is busy' in error: {err}"
    );
}

#[test]
fn trigger_sdk_action_returns_channel_disconnected_when_receiver_dropped() {
    use std::sync::mpsc;

    let (sender, receiver) = mpsc::sync_channel::<protocol::Message>(10);
    drop(receiver); // Simulate script exit

    let action = ProtocolAction {
        name: "late_action".to_string(),
        description: None,
        shortcut: None,
        value: Some("val".to_string()),
        has_action: true,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("late_action", &action, "", Some(&sender), "test-trace");
    assert_eq!(result, SdkActionResult::ChannelDisconnected);
    let err = result.error_message("late_action").unwrap();
    assert!(
        err.contains("script has exited"),
        "Expected 'script has exited' in error: {err}"
    );
}

// REMOVED: test_pbcopy_basic, test_pbcopy_empty_string, test_pbcopy_unicode
// — write to system clipboard via pbcopy, interferes with user workflow

// ============================================================================
// ActionOutcomeStatus tests
// ============================================================================

#[test]
fn status_maps_sent_to_success() {
    assert_eq!(SdkActionResult::Sent.status(), ActionOutcomeStatus::Success);
}

#[test]
fn status_maps_no_effect_to_no_effect() {
    assert_eq!(
        SdkActionResult::NoEffect.status(),
        ActionOutcomeStatus::NoEffect
    );
}

#[test]
fn status_maps_cancelled_to_cancelled() {
    assert_eq!(
        SdkActionResult::Cancelled.status(),
        ActionOutcomeStatus::Cancelled
    );
}

#[test]
fn status_maps_error_variants_to_error() {
    assert_eq!(
        SdkActionResult::NoSender.status(),
        ActionOutcomeStatus::Error
    );
    assert_eq!(
        SdkActionResult::ChannelFull.status(),
        ActionOutcomeStatus::Error
    );
    assert_eq!(
        SdkActionResult::ChannelDisconnected.status(),
        ActionOutcomeStatus::Error
    );
}

#[test]
fn action_outcome_status_is_copy() {
    // Verify Copy semantics work (important for pattern matching / logging)
    let s = ActionOutcomeStatus::Success;
    let s2 = s;
    assert_eq!(s, s2);
}

// ============================================================================
// user_message tests
// ============================================================================

#[test]
fn user_message_none_for_success_variants() {
    assert!(SdkActionResult::Sent.user_message().is_none());
    assert!(SdkActionResult::NoEffect.user_message().is_none());
    assert!(SdkActionResult::Cancelled.user_message().is_none());
}

#[test]
fn user_message_present_for_error_variants() {
    assert!(SdkActionResult::NoSender.user_message().is_some());
    assert!(SdkActionResult::ChannelFull.user_message().is_some());
    assert!(SdkActionResult::ChannelDisconnected
        .user_message()
        .is_some());
}

#[test]
fn user_message_never_contains_variant_names() {
    for variant in &[
        SdkActionResult::NoSender,
        SdkActionResult::ChannelFull,
        SdkActionResult::ChannelDisconnected,
    ] {
        let msg = variant.user_message().unwrap();
        assert!(!msg.contains("NoSender"), "leaked variant name in: {msg}");
        assert!(
            !msg.contains("ChannelFull"),
            "leaked variant name in: {msg}"
        );
        assert!(
            !msg.contains("ChannelDisconnected"),
            "leaked variant name in: {msg}"
        );
    }
}

// ============================================================================
// Error code tests
// ============================================================================

#[test]
fn error_code_constants_are_stable_strings() {
    // Verify error codes are stable string constants (not enum variant names).
    assert_eq!(ERROR_CHANNEL_FULL, "channel_full");
    assert_eq!(ERROR_CHANNEL_DISCONNECTED, "channel_disconnected");
    assert_eq!(ERROR_UNSUPPORTED_PLATFORM, "unsupported_platform");
    assert_eq!(ERROR_LAUNCH_FAILED, "launch_failed");
    assert_eq!(ERROR_REVEAL_FAILED, "reveal_failed");
    assert_eq!(ERROR_TRASH_FAILED, "trash_failed");
    assert_eq!(ERROR_MODAL_FAILED, "modal_failed");
    assert_eq!(ERROR_CANCELLED, "cancelled");
    assert_eq!(ERROR_NO_SENDER, "no_sender");
}

#[test]
fn sdk_action_result_error_code_for_success_variants() {
    assert_eq!(SdkActionResult::Sent.error_code(), None);
    assert_eq!(SdkActionResult::NoEffect.error_code(), None);
}

#[test]
fn sdk_action_result_error_code_for_failure_variants() {
    assert_eq!(
        SdkActionResult::NoSender.error_code(),
        Some(ERROR_NO_SENDER)
    );
    assert_eq!(
        SdkActionResult::ChannelFull.error_code(),
        Some(ERROR_CHANNEL_FULL)
    );
    assert_eq!(
        SdkActionResult::ChannelDisconnected.error_code(),
        Some(ERROR_CHANNEL_DISCONNECTED)
    );
}

#[test]
fn sdk_action_result_error_message_never_contains_variant_names() {
    // Ensure raw enum variant names like "ChannelFull" or "ChannelDisconnected"
    // never leak into user-facing messages.
    let variants = [
        SdkActionResult::NoSender,
        SdkActionResult::ChannelFull,
        SdkActionResult::ChannelDisconnected,
        SdkActionResult::Cancelled,
    ];

    for variant in &variants {
        if let Some(msg) = variant.error_message("test") {
            assert!(
                !msg.contains("ChannelFull"),
                "error_message leaked variant name 'ChannelFull': {msg}"
            );
            assert!(
                !msg.contains("ChannelDisconnected"),
                "error_message leaked variant name 'ChannelDisconnected': {msg}"
            );
            assert!(
                !msg.contains("NoSender"),
                "error_message leaked variant name 'NoSender': {msg}"
            );
            assert!(
                !msg.contains("Cancelled"),
                "error_message leaked variant name 'Cancelled': {msg}"
            );
        }
    }
}

// ============================================================================
// Modal cancellation tests — Cancelled is NOT an error
// ============================================================================

#[test]
fn cancelled_variant_has_error_code_but_no_error_message() {
    // Cancellation is machine-readable (error_code = "cancelled") but is NOT
    // an error from the user's perspective — no toast should be shown.
    let result = SdkActionResult::Cancelled;
    assert_eq!(
        result.error_code(),
        Some(ERROR_CANCELLED),
        "Cancelled should have a machine-readable error code"
    );
    assert!(
        result.error_message("any_action").is_none(),
        "Cancelled should NOT produce an error message (no toast)"
    );
}

#[test]
fn cancelled_variant_is_not_sent() {
    assert!(
        !SdkActionResult::Cancelled.is_sent(),
        "Cancelled should not be is_sent()"
    );
}

#[test]
fn cancelled_error_code_matches_stable_constant() {
    assert_eq!(
        SdkActionResult::Cancelled
            .error_code()
            .expect("should have code"),
        "cancelled",
        "Cancelled error_code must be the stable string 'cancelled'"
    );
}

// ============================================================================
// DispatchOutcome tests
// ============================================================================

#[test]
fn dispatch_outcome_success_has_no_error() {
    let outcome = DispatchOutcome::success();
    assert_eq!(outcome.status, ActionOutcomeStatus::Success);
    assert!(outcome.error_code.is_none());
    assert!(outcome.user_message.is_none());
    assert!(outcome.was_handled());
}

#[test]
fn dispatch_outcome_not_handled_is_no_effect() {
    let outcome = DispatchOutcome::not_handled();
    assert_eq!(outcome.status, ActionOutcomeStatus::NoEffect);
    assert!(!outcome.was_handled());
}

#[test]
fn dispatch_outcome_error_carries_code_and_message() {
    let outcome = DispatchOutcome::error(ERROR_LAUNCH_FAILED, "Editor not found");
    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert_eq!(outcome.error_code, Some(ERROR_LAUNCH_FAILED));
    assert_eq!(outcome.user_message.as_deref(), Some("Editor not found"));
    assert!(outcome.was_handled());
}

#[test]
fn dispatch_outcome_cancelled_has_code_but_no_message() {
    let outcome = DispatchOutcome::cancelled();
    assert_eq!(outcome.status, ActionOutcomeStatus::Cancelled);
    assert_eq!(outcome.error_code, Some(ERROR_CANCELLED));
    assert!(outcome.user_message.is_none());
    assert!(outcome.was_handled());
}

#[test]
fn dispatch_outcome_from_sdk_sent() {
    let outcome = DispatchOutcome::from_sdk(&SdkActionResult::Sent, "test");
    assert_eq!(outcome.status, ActionOutcomeStatus::Success);
    assert!(outcome.error_code.is_none());
    assert!(outcome.user_message.is_none());
}

#[test]
fn dispatch_outcome_from_sdk_error() {
    let outcome = DispatchOutcome::from_sdk(&SdkActionResult::NoSender, "test");
    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert_eq!(outcome.error_code, Some(ERROR_NO_SENDER));
    assert!(outcome.user_message.is_some());
}

#[test]
fn dispatch_outcome_with_detail() {
    let outcome = DispatchOutcome::success().with_detail("extra context");
    assert_eq!(outcome.detail.as_deref(), Some("extra context"));
    assert_eq!(outcome.status, ActionOutcomeStatus::Success);
}

#[test]
fn dispatch_outcome_from_sdk_cancelled() {
    let outcome = DispatchOutcome::from_sdk(&SdkActionResult::Cancelled, "delete_all");
    assert_eq!(outcome.status, ActionOutcomeStatus::Cancelled);
    assert_eq!(outcome.error_code, Some(ERROR_CANCELLED));
    assert!(
        outcome.user_message.is_none(),
        "Cancelled should not produce a user message"
    );
}

#[test]
fn dispatch_outcome_from_sdk_no_effect() {
    let outcome = DispatchOutcome::from_sdk(&SdkActionResult::NoEffect, "noop");
    assert_eq!(outcome.status, ActionOutcomeStatus::NoEffect);
    assert!(outcome.error_code.is_none());
    assert!(outcome.user_message.is_none());
    assert!(!outcome.was_handled());
}

#[test]
fn dispatch_outcome_from_sdk_channel_full() {
    let outcome = DispatchOutcome::from_sdk(&SdkActionResult::ChannelFull, "busy_action");
    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert_eq!(outcome.error_code, Some(ERROR_CHANNEL_FULL));
    assert!(outcome.user_message.is_some());
}

#[test]
fn dispatch_outcome_from_sdk_channel_disconnected() {
    let outcome = DispatchOutcome::from_sdk(&SdkActionResult::ChannelDisconnected, "late_action");
    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert_eq!(outcome.error_code, Some(ERROR_CHANNEL_DISCONNECTED));
    assert!(outcome.user_message.is_some());
}

// ============================================================================
// DispatchContext tests
// ============================================================================

#[test]
fn dispatch_context_for_action_sets_action_surface() {
    let ctx = DispatchContext::for_action("copy_path");
    assert_eq!(ctx.surface, DispatchSurface::Action);
    assert_eq!(ctx.action_id, "copy_path");
    assert!(!ctx.trace_id.is_empty(), "trace_id must be non-empty");
}

#[test]
fn dispatch_context_for_builtin_sets_builtin_surface() {
    let ctx = DispatchContext::for_builtin("clipboard_history");
    assert_eq!(ctx.surface, DispatchSurface::Builtin);
    assert_eq!(ctx.action_id, "clipboard_history");
    assert!(!ctx.trace_id.is_empty(), "trace_id must be non-empty");
}

#[test]
fn dispatch_context_trace_ids_are_unique() {
    let ctx1 = DispatchContext::for_action("a");
    let ctx2 = DispatchContext::for_action("a");
    assert_ne!(
        ctx1.trace_id, ctx2.trace_id,
        "Each dispatch context must get a unique trace_id"
    );
}

#[test]
fn dispatch_context_accepts_string_and_str() {
    // Verify Into<String> works for both &str and String
    let ctx_str = DispatchContext::for_action("literal");
    let ctx_string = DispatchContext::for_action(String::from("owned"));
    assert_eq!(ctx_str.action_id, "literal");
    assert_eq!(ctx_string.action_id, "owned");
}

#[test]
fn dispatch_surface_display_action() {
    assert_eq!(DispatchSurface::Action.to_string(), "action");
}

#[test]
fn dispatch_surface_display_builtin() {
    assert_eq!(DispatchSurface::Builtin.to_string(), "builtin");
}
