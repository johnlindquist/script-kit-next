//! Tests for action_helpers module.
//!
//! Covers all SearchResult variants (Script, Scriptlet, BuiltIn, App, Agent, Window, Fallback)
//! for extract_path_for_reveal, extract_path_for_copy, and extract_path_for_edit.

use script_kit_gpui::action_helpers::{
    extract_path_for_copy, extract_path_for_edit, extract_path_for_reveal, find_sdk_action,
    is_reserved_action_id, trigger_sdk_action, ActionOutcomeStatus, DispatchContext,
    DispatchOutcome, DispatchSurface, PathExtractionError, SdkActionResult, ERROR_CANCELLED,
    ERROR_CHANNEL_DISCONNECTED, ERROR_CHANNEL_FULL, ERROR_NO_SENDER,
};
use script_kit_gpui::agents::Agent;
use script_kit_gpui::app_launcher;
use script_kit_gpui::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};
use script_kit_gpui::fallbacks::{
    BuiltinFallback, FallbackAction, FallbackCondition, FallbackItem,
};
use script_kit_gpui::protocol::{self, ProtocolAction};
use script_kit_gpui::scripts::{
    AgentMatch, AppMatch, BuiltInMatch, FallbackMatch, MatchIndices, Script, ScriptMatch,
    Scriptlet, ScriptletMatch, SearchResult, WindowMatch,
};
use script_kit_gpui::window_control;
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

// Tests for pbcopy (macOS only)

// REMOVED: test_pbcopy_basic, test_pbcopy_empty_string, test_pbcopy_unicode
// — write to system clipboard via pbcopy, interferes with user workflow

// ============================================================================
// Error code and Cancelled variant tests
// ============================================================================

#[test]
fn error_code_constants_are_stable_strings() {
    assert_eq!(ERROR_NO_SENDER, "no_sender");
    assert_eq!(ERROR_CHANNEL_FULL, "channel_full");
    assert_eq!(ERROR_CHANNEL_DISCONNECTED, "channel_disconnected");
    assert_eq!(ERROR_CANCELLED, "cancelled");
}

#[test]
fn cancelled_variant_produces_code_but_no_error_message() {
    let result = SdkActionResult::Cancelled;

    assert_eq!(result.error_code(), Some(ERROR_CANCELLED));
    assert!(
        result.error_message("delete_all").is_none(),
        "Cancelled must NOT produce a user-facing error message"
    );
    assert!(!result.is_sent());
}

#[test]
fn all_sdk_action_result_variants_have_consistent_error_code_mapping() {
    // Success variants: no code
    assert!(SdkActionResult::Sent.error_code().is_none());
    assert!(SdkActionResult::NoEffect.error_code().is_none());

    // Error variants: stable codes
    assert_eq!(SdkActionResult::NoSender.error_code(), Some("no_sender"));
    assert_eq!(
        SdkActionResult::ChannelFull.error_code(),
        Some("channel_full")
    );
    assert_eq!(
        SdkActionResult::ChannelDisconnected.error_code(),
        Some("channel_disconnected")
    );

    // Cancellation: code but no error message
    assert_eq!(SdkActionResult::Cancelled.error_code(), Some("cancelled"));
}

#[test]
fn error_messages_never_expose_transport_enum_names() {
    let forbidden = [
        "NoSender",
        "ChannelFull",
        "ChannelDisconnected",
        "Cancelled",
    ];

    for variant in &[
        SdkActionResult::NoSender,
        SdkActionResult::ChannelFull,
        SdkActionResult::ChannelDisconnected,
        SdkActionResult::Cancelled,
    ] {
        if let Some(msg) = variant.error_message("test") {
            for name in &forbidden {
                assert!(
                    !msg.contains(name),
                    "error_message leaked transport name '{name}': {msg}"
                );
            }
        }
    }
}

// ============================================================================
// DispatchOutcome — constructors and from_sdk conversion
// ============================================================================

#[test]
fn dispatch_outcome_success_is_handled() {
    let outcome = DispatchOutcome::success();
    assert_eq!(outcome.status, ActionOutcomeStatus::Success);
    assert!(outcome.was_handled());
    assert!(outcome.error_code.is_none());
    assert!(outcome.user_message.is_none());
}

#[test]
fn dispatch_outcome_not_handled_is_not_handled() {
    let outcome = DispatchOutcome::not_handled();
    assert_eq!(outcome.status, ActionOutcomeStatus::NoEffect);
    assert!(!outcome.was_handled());
}

#[test]
fn dispatch_outcome_error_is_handled_with_code_and_message() {
    let outcome = DispatchOutcome::error(ERROR_NO_SENDER, "No active script");
    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert!(outcome.was_handled());
    assert_eq!(outcome.error_code, Some(ERROR_NO_SENDER));
    assert_eq!(outcome.user_message.as_deref(), Some("No active script"));
}

#[test]
fn dispatch_outcome_cancelled_is_handled_with_code_but_no_message() {
    let outcome = DispatchOutcome::cancelled();
    assert_eq!(outcome.status, ActionOutcomeStatus::Cancelled);
    assert!(outcome.was_handled());
    assert_eq!(outcome.error_code, Some(ERROR_CANCELLED));
    assert!(outcome.user_message.is_none());
}

#[test]
fn dispatch_outcome_from_sdk_all_variants() {
    // Sent → Success
    let o = DispatchOutcome::from_sdk(&SdkActionResult::Sent, "x");
    assert_eq!(o.status, ActionOutcomeStatus::Success);
    assert!(o.error_code.is_none());

    // NoEffect → NoEffect (not handled)
    let o = DispatchOutcome::from_sdk(&SdkActionResult::NoEffect, "x");
    assert_eq!(o.status, ActionOutcomeStatus::NoEffect);
    assert!(!o.was_handled());

    // Cancelled → Cancelled
    let o = DispatchOutcome::from_sdk(&SdkActionResult::Cancelled, "x");
    assert_eq!(o.status, ActionOutcomeStatus::Cancelled);
    assert_eq!(o.error_code, Some(ERROR_CANCELLED));
    assert!(o.user_message.is_none());

    // NoSender → Error
    let o = DispatchOutcome::from_sdk(&SdkActionResult::NoSender, "my_action");
    assert_eq!(o.status, ActionOutcomeStatus::Error);
    assert_eq!(o.error_code, Some(ERROR_NO_SENDER));
    assert!(o.user_message.is_some());

    // ChannelFull → Error
    let o = DispatchOutcome::from_sdk(&SdkActionResult::ChannelFull, "my_action");
    assert_eq!(o.status, ActionOutcomeStatus::Error);
    assert_eq!(o.error_code, Some(ERROR_CHANNEL_FULL));

    // ChannelDisconnected → Error
    let o = DispatchOutcome::from_sdk(&SdkActionResult::ChannelDisconnected, "my_action");
    assert_eq!(o.status, ActionOutcomeStatus::Error);
    assert_eq!(o.error_code, Some(ERROR_CHANNEL_DISCONNECTED));
}

#[test]
fn dispatch_outcome_with_detail_preserves_status() {
    let outcome = DispatchOutcome::error(ERROR_NO_SENDER, "msg").with_detail("extra");
    assert_eq!(outcome.status, ActionOutcomeStatus::Error);
    assert_eq!(outcome.detail.as_deref(), Some("extra"));
}

// ============================================================================
// DispatchContext — creation and trace_id propagation
// ============================================================================

#[test]
fn dispatch_context_for_action_has_action_surface() {
    let ctx = DispatchContext::for_action("copy_path");
    assert_eq!(ctx.surface, DispatchSurface::Action);
    assert_eq!(ctx.action_id, "copy_path");
    assert!(!ctx.trace_id.is_empty());
}

#[test]
fn dispatch_context_for_builtin_has_builtin_surface() {
    let ctx = DispatchContext::for_builtin("clipboard_history");
    assert_eq!(ctx.surface, DispatchSurface::Builtin);
    assert_eq!(ctx.action_id, "clipboard_history");
    assert!(!ctx.trace_id.is_empty());
}

#[test]
fn dispatch_context_trace_ids_are_unique_across_calls() {
    let a = DispatchContext::for_action("x");
    let b = DispatchContext::for_action("x");
    assert_ne!(a.trace_id, b.trace_id);
}

#[test]
fn dispatch_context_accepts_owned_string() {
    let ctx = DispatchContext::for_action(String::from("owned_id"));
    assert_eq!(ctx.action_id, "owned_id");
}

#[test]
fn dispatch_surface_display_values() {
    assert_eq!(format!("{}", DispatchSurface::Action), "action");
    assert_eq!(format!("{}", DispatchSurface::Builtin), "builtin");
}
