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

// =========================================================================
// Mention picker windowing — selected item always visible
// =========================================================================

/// Helper: call the private `mention_visible_range_for` and assert the
/// selected index falls within the returned range.
fn assert_selected_visible(selected: usize, item_count: usize) {
    let range = super::view::AcpChatView::mention_visible_range_for(selected, item_count);
    assert!(
        range.contains(&selected),
        "selected_index={selected} must be inside visible range {range:?} (item_count={item_count})",
    );
    assert!(
        range.len() <= super::view::AcpChatView::MENTION_PICKER_MAX_VISIBLE,
        "visible range len {} exceeds max {}",
        range.len(),
        super::view::AcpChatView::MENTION_PICKER_MAX_VISIBLE,
    );
}

#[test]
fn mention_picker_windowing_small_list() {
    // Fewer items than max_visible → range is 0..item_count
    for selected in 0..5 {
        let range = super::view::AcpChatView::mention_visible_range_for(selected, 5);
        assert_eq!(range, 0..5);
    }
}

#[test]
fn mention_picker_windowing_selected_always_visible() {
    let item_count = 20;
    for selected in 0..item_count {
        assert_selected_visible(selected, item_count);
    }
}

#[test]
fn mention_picker_windowing_wrap_to_last() {
    // Simulate pressing Up from index 0 → wrap to last
    let item_count = 15;
    let selected = item_count - 1;
    assert_selected_visible(selected, item_count);
}

#[test]
fn mention_picker_windowing_wrap_to_first() {
    // Simulate pressing Down from last → wrap to 0
    assert_selected_visible(0, 15);
}

// =========================================================================
// 5. ACP preflight and setup mode — source code contracts
// =========================================================================

#[test]
fn tab_ai_mode_uses_catalog_loader_not_claude_only_loader() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("load_acp_agent_catalog_entries"),
        "tab_ai_mode must use the catalog loader, not Claude-only config"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("resolve_default_acp_launch"),
        "tab_ai_mode must use preflight resolution"
    );
}

#[test]
fn tab_ai_mode_routes_to_setup_mode_when_blocked() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("AcpChatView::new_setup"),
        "tab_ai_mode must create setup-mode view when agent is blocked"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("acp_launch_resolution"),
        "tab_ai_mode must log launch resolution event"
    );
}

#[test]
fn acp_view_supports_setup_constructor() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    assert!(
        ACP_VIEW_SOURCE.contains("fn new_setup"),
        "AcpChatView must have a new_setup constructor"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("AcpChatSession::Setup"),
        "AcpChatView must support Setup session state"
    );
}

#[test]
fn acp_view_thread_accessor_returns_option() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    assert!(
        ACP_VIEW_SOURCE.contains("fn thread(&self) -> Option<Entity<AcpThread>>"),
        "AcpChatView must have a thread() method returning Option"
    );
}

#[test]
fn setup_state_from_resolution_covers_all_blockers() {
    use super::preflight::{AcpLaunchBlocker, AcpLaunchResolution};
    use super::setup_state::{AcpInlineSetupState, AcpSetupAction};

    let blockers = [
        AcpLaunchBlocker::NoAgentsAvailable,
        AcpLaunchBlocker::AgentNotInstalled,
        AcpLaunchBlocker::AuthenticationRequired,
        AcpLaunchBlocker::AgentMisconfigured,
        AcpLaunchBlocker::UnsupportedAgent,
    ];

    for blocker in &blockers {
        let resolution = AcpLaunchResolution {
            selected_agent: None,
            blocker: Some(blocker.clone()),
        };
        let state = AcpInlineSetupState::from_resolution(&resolution);
        assert!(
            !state.title.is_empty(),
            "setup state title must be non-empty for {:?}",
            blocker
        );
        assert!(
            !state.body.is_empty(),
            "setup state body must be non-empty for {:?}",
            blocker
        );
    }
}

#[test]
fn events_have_setup_required_variant() {
    const EVENTS_SOURCE: &str = include_str!("events.rs");
    assert!(
        EVENTS_SOURCE.contains("SetupRequired"),
        "AcpEvent must have a SetupRequired variant"
    );
}

#[test]
fn ai_setup_surface_no_longer_mentions_claude_only_copy() {
    const SETUP_RENDER_SOURCE: &str = include_str!("../../ai/window/render_setup.rs");
    assert!(
        SETUP_RENDER_SOURCE.contains("ACP Agent Required"),
        "setup card title must say ACP Agent Required"
    );
    assert!(
        SETUP_RENDER_SOURCE.contains("Open ACP Agent Catalog"),
        "setup card must offer Open ACP Agent Catalog"
    );
    assert!(
        !SETUP_RENDER_SOURCE.contains("Connect to Claude Code"),
        "setup card must NOT mention Claude Code specifically"
    );
}
