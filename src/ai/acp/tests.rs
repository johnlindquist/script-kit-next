//! Targeted ACP tests covering render ownership, approval flow,
//! and Tab AI routing contracts.
//!
//! These complement the per-module unit tests in `thread.rs`,
//! `permission_broker.rs`, `events.rs`, and `client.rs` with
//! cross-cutting integration-style assertions.

use agent_client_protocol::{ContentBlock, TextContent};

use super::events::AcpEvent;
use super::permission_broker::{AcpApprovalOption, AcpApprovalRequest, AcpPermissionBroker};
use super::thread::{AcpThread, AcpThreadMessageRole, AcpThreadStatus};

// =========================================================================
// 1. First-turn staged context preservation
// =========================================================================

#[test]
fn staged_context_prepended_on_first_submit_only() {
    let mut thread = AcpThread::test_new(
        vec![ContentBlock::Text(TextContent::new("desktop-context"))],
        None,
    );

    let first_blocks = thread.prepare_turn_blocks("build a script");
    assert!(
        first_blocks.len() >= 2,
        "first turn should include staged context + input (got {})",
        first_blocks.len()
    );

    // Verify the context block is actually first
    if let ContentBlock::Text(ref t) = first_blocks[0] {
        assert_eq!(t.text, "desktop-context");
    } else {
        panic!("expected first block to be the staged context Text block");
    }

    let second_blocks = thread.prepare_turn_blocks("another request");
    assert_eq!(
        second_blocks.len(),
        1,
        "second turn should NOT re-include context"
    );
    // Second turn should be plain text without USER REQUEST marker
    if let ContentBlock::Text(ref t) = second_blocks[0] {
        assert_eq!(
            t.text, "another request",
            "second turn should be plain user text"
        );
    } else {
        panic!("expected second turn to be a Text block");
    }
}

#[test]
fn initial_input_populates_composer_for_auto_submit() {
    let thread = AcpThread::test_new(vec![], Some("build a clipboard manager".to_string()));

    assert_eq!(
        thread.input.text(),
        "build a clipboard manager",
        "initial_input should populate the composer"
    );
}

#[test]
fn empty_initial_input_leaves_composer_blank() {
    let thread = AcpThread::test_new(vec![], None);
    assert!(
        thread.input.text().is_empty(),
        "no initial_input should leave composer empty"
    );
}

// =========================================================================
// 2. Pending approval state and approval/cancel dispatch
// =========================================================================

#[test]
fn pending_permission_stored_and_clears_on_approve() {
    let mut thread = AcpThread::test_new(vec![], None);

    let (reply_tx, reply_rx) = async_channel::bounded(1);
    thread.pending_permission = Some(AcpApprovalRequest {
        id: 1,
        title: "Write to file".into(),
        body: "Agent wants to write to /tmp/test.txt".into(),
        preview: None,
        options: vec![
            AcpApprovalOption {
                option_id: "allow-once".into(),
                name: "Allow once".into(),
                kind: "AllowOnce".into(),
            },
            AcpApprovalOption {
                option_id: "deny".into(),
                name: "Deny".into(),
                kind: "RejectOnce".into(),
            },
        ],
        reply_tx,
    });
    thread.status = AcpThreadStatus::WaitingForPermission;

    assert!(thread.pending_permission.is_some());
    assert_eq!(thread.status, AcpThreadStatus::WaitingForPermission);

    // Simulate approve (same logic as approve_pending_permission without cx)
    if let Some(req) = thread.pending_permission.take() {
        let _ = req.reply_tx.send_blocking(Some("allow-once".to_string()));
    }
    thread.status = AcpThreadStatus::Idle;

    assert!(thread.pending_permission.is_none());
    assert_eq!(thread.status, AcpThreadStatus::Idle);

    let reply = reply_rx.recv_blocking().expect("should receive reply");
    assert_eq!(reply, Some("allow-once".to_string()));
}

#[test]
fn pending_permission_cancel_sends_none() {
    let mut thread = AcpThread::test_new(vec![], None);

    let (reply_tx, reply_rx) = async_channel::bounded(1);
    thread.pending_permission = Some(AcpApprovalRequest {
        id: 2,
        title: "Terminal access".into(),
        body: "Agent wants to run a command".into(),
        preview: None,
        options: vec![AcpApprovalOption {
            option_id: "allow".into(),
            name: "Allow".into(),
            kind: "AllowOnce".into(),
        }],
        reply_tx,
    });
    thread.status = AcpThreadStatus::WaitingForPermission;

    if let Some(req) = thread.pending_permission.take() {
        let _ = req.reply_tx.send_blocking(None);
    }
    thread.status = AcpThreadStatus::Idle;

    let reply = reply_rx.recv_blocking().expect("should receive reply");
    assert_eq!(reply, None, "cancel should send None");
}

#[test]
fn broker_full_roundtrip_with_three_options() {
    let (broker, rx) = AcpPermissionBroker::new();

    let handle = std::thread::spawn(move || {
        let request = rx.recv_blocking().expect("should receive request");
        assert_eq!(request.options.len(), 3, "all options should be forwarded");
        assert_eq!(request.options[0].option_id, "allow-once");
        assert_eq!(request.options[1].option_id, "allow-always");
        assert_eq!(request.options[2].option_id, "deny");
        request
            .reply_tx
            .send_blocking(Some("allow-always".to_string()))
            .expect("reply should send");
    });

    let result = broker
        .request(super::permission_broker::AcpApprovalRequestInput {
            title: "Read file".into(),
            body: "src/main.rs".into(),
            preview: None,
            options: vec![
                AcpApprovalOption {
                    option_id: "allow-once".into(),
                    name: "Allow once".into(),
                    kind: "AllowOnce".into(),
                },
                AcpApprovalOption {
                    option_id: "allow-always".into(),
                    name: "Allow always".into(),
                    kind: "AllowAlways".into(),
                },
                AcpApprovalOption {
                    option_id: "deny".into(),
                    name: "Deny".into(),
                    kind: "RejectOnce".into(),
                },
            ],
        })
        .expect("request should succeed");

    assert_eq!(result, Some("allow-always".to_string()));
    handle.join().expect("responder thread should finish");
}

// =========================================================================
// 3. Render ownership — ACP thread state contracts
// =========================================================================

#[test]
fn acp_thread_starts_idle_with_empty_state() {
    let thread = AcpThread::test_new(vec![], None);

    assert_eq!(thread.status, AcpThreadStatus::Idle);
    assert!(thread.messages.is_empty());
    assert!(thread.active_plan_entries().is_empty());
    assert!(thread.active_mode_id().is_none());
    assert!(thread.available_commands().is_empty());
    assert!(thread.active_tool_calls().is_empty());
}

#[test]
fn streaming_deltas_coalesce_for_view_render() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::AgentMessageDelta("## Plan\n".into()));
    thread.apply_event_test(AcpEvent::AgentMessageDelta("1. Read the file\n".into()));
    thread.apply_event_test(AcpEvent::AgentMessageDelta("2. Apply patch\n".into()));

    assert_eq!(thread.messages.len(), 1, "streaming chunks should coalesce");
    assert_eq!(thread.messages[0].role, AcpThreadMessageRole::Assistant);
    assert!(thread.messages[0].body.contains("## Plan"));
    assert!(thread.messages[0].body.contains("2. Apply patch"));
}

#[test]
fn thought_deltas_separate_from_assistant_deltas() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::AgentThoughtDelta("hmm...".into()));
    thread.apply_event_test(AcpEvent::AgentMessageDelta("Here's the plan".into()));

    assert_eq!(thread.messages.len(), 2);
    assert_eq!(thread.messages[0].role, AcpThreadMessageRole::Thought);
    assert_eq!(thread.messages[1].role, AcpThreadMessageRole::Assistant);
}

#[test]
fn plan_visible_to_view_without_message_creation() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::PlanUpdated {
        entries: vec!["Read file".into(), "Apply patch".into(), "Run tests".into()],
    });

    assert_eq!(thread.active_plan_entries().len(), 3);
    assert!(thread.messages.is_empty());
}

#[test]
fn tool_call_lifecycle_tracks_state_for_view() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::ToolCallStarted {
        tool_call_id: "tc-abc".into(),
        title: "Read file".into(),
        status: "running".into(),
    });

    assert_eq!(thread.active_tool_calls().len(), 1);
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].role, AcpThreadMessageRole::Tool);

    thread.apply_event_test(AcpEvent::ToolCallUpdated {
        tool_call_id: "tc-abc".into(),
        title: None,
        status: Some("completed".into()),
        body: Some("file contents...".into()),
    });

    assert_eq!(thread.messages.len(), 1, "should update in-place");
    assert!(thread.messages[0].body.contains("completed"));
    assert_eq!(thread.active_tool_calls()[0].status, "completed");
}

#[test]
fn error_event_creates_error_message_and_sets_status() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::Failed {
        error: "ACP connection lost".into(),
    });

    assert_eq!(thread.status, AcpThreadStatus::Error);
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].role, AcpThreadMessageRole::Error);
}

#[test]
fn turn_finished_returns_to_idle_from_streaming() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::AgentMessageDelta("hello".into()));
    assert_eq!(thread.status, AcpThreadStatus::Streaming);

    thread.apply_event_test(AcpEvent::TurnFinished {
        stop_reason: "end_turn".into(),
    });
    assert_eq!(thread.status, AcpThreadStatus::Idle);
}

// =========================================================================
// 4. Tab AI routing — source code contracts
// =========================================================================

const TAB_AI_MODE_SOURCE: &str = include_str!("../../app_impl/tab_ai_mode.rs");
const STARTUP_SOURCE: &str = include_str!("../../app_impl/startup.rs");
const STARTUP_NEW_TAB_SOURCE: &str = include_str!("../../app_impl/startup_new_tab.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../../main_sections/render_impl.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../../main_sections/app_view_state.rs");

#[test]
fn app_view_has_acp_chat_view_variant() {
    assert!(
        APP_VIEW_STATE_SOURCE.contains("AcpChatView"),
        "AppView enum must have an AcpChatView variant"
    );
}

#[test]
fn tab_ai_mode_creates_acp_chat_view_for_tab() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("AcpChatView::new"),
        "tab_ai_mode must create an AcpChatView"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("AppView::AcpChatView"),
        "tab_ai_mode must set current_view to AcpChatView"
    );
}

#[test]
fn tab_ai_mode_creates_acp_thread_with_connection() {
    assert!(TAB_AI_MODE_SOURCE.contains("AcpThread::new"));
    assert!(TAB_AI_MODE_SOURCE.contains("AcpConnection::spawn_with_approval"));
    assert!(TAB_AI_MODE_SOURCE.contains("AcpPermissionBroker::new"));
}

#[test]
fn tab_ai_mode_stages_context_on_acp_thread() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("stage_context"),
        "tab_ai_mode must stage context on the AcpThread"
    );
}

#[test]
fn tab_ai_mode_supports_auto_submit_with_initial_input() {
    assert!(TAB_AI_MODE_SOURCE.contains("initial_input"));
    assert!(TAB_AI_MODE_SOURCE.contains("AcpThreadInit"));
}

#[test]
fn startup_tab_guard_checks_acp_chat_view() {
    assert!(STARTUP_SOURCE.contains("AppView::AcpChatView"));
    assert!(STARTUP_SOURCE.contains("handle_tab_key"));
}

#[test]
fn startup_new_tab_guard_checks_acp_chat_view() {
    assert!(STARTUP_NEW_TAB_SOURCE.contains("AppView::AcpChatView"));
    assert!(STARTUP_NEW_TAB_SOURCE.contains("handle_tab_key"));
}

#[test]
fn render_impl_dispatches_acp_chat_view() {
    assert!(RENDER_IMPL_SOURCE.contains("AppView::AcpChatView"));
}

#[test]
fn script_triggered_terminals_still_use_pty() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn open_tab_ai_harness_terminal_from_request"),
        "PTY path must still exist for script-triggered terminals"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("AppView::QuickTerminalView"),
        "PTY path must still set QuickTerminalView"
    );
}

#[test]
fn acp_and_pty_views_coexist_in_app_view() {
    assert!(APP_VIEW_STATE_SOURCE.contains("AcpChatView"));
    assert!(APP_VIEW_STATE_SOURCE.contains("QuickTerminalView"));
}
