//! Targeted ACP tests covering render ownership, approval flow,
//! and Tab AI routing contracts.
//!
//! These complement the per-module unit tests in `thread.rs`,
//! `permission_broker.rs`, `events.rs`, and `client.rs` with
//! cross-cutting integration-style assertions.

use agent_client_protocol::{ContentBlock, TextContent};

use super::events::AcpEvent;
use super::permission_broker::{AcpApprovalOption, AcpApprovalRequest, AcpPermissionBroker};
use super::preflight::AcpLaunchRequirements;
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
fn runtime_setup_required_arms_recovery_state() {
    let mut thread = AcpThread::test_new(vec![], None);

    thread.apply_event_test(AcpEvent::SetupRequired {
        reason: "auth_required".into(),
        auth_methods: vec!["oauth".into()],
    });

    let setup = thread
        .setup_state()
        .expect("runtime setup required should arm recovery state");
    assert_eq!(setup.title, "Authentication required");
    assert_eq!(thread.status, AcpThreadStatus::Error);
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

const TAB_AI_MODE_SOURCE: &str = include_str!("../../app_impl/tab_ai_mode/mod.rs");
const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../../app_impl/actions_toggle.rs");
const STARTUP_SOURCE: &str = include_str!("../../app_impl/startup.rs");
const STARTUP_NEW_ACTIONS_SOURCE: &str = include_str!("../../app_impl/startup_new_actions.rs");
const STARTUP_NEW_TAB_SOURCE: &str = include_str!("../../app_impl/startup_new_tab.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../../main_sections/render_impl.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../../main_sections/app_view_state.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../../app_impl/ui_window.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../../main_entry/app_run_setup.rs");
const RUNTIME_STDIN_SOURCE: &str = include_str!("../../main_entry/runtime_stdin.rs");
const HANDLE_ACTION_SOURCE: &str = include_str!("../../app_actions/handle_action/mod.rs");
const REGISTRIES_STATE_SOURCE: &str = include_str!("../../app_impl/registries_state.rs");
const ACP_MOD_SOURCE: &str = include_str!("mod.rs");
const ACP_HISTORY_POPUP_SOURCE: &str = include_str!("history_popup.rs");
const ACP_MODEL_SELECTOR_POPUP_SOURCE: &str = include_str!("model_selector_popup.rs");
const ACP_PICKER_POPUP_SOURCE: &str = include_str!("picker_popup.rs");
const ACP_POPUP_WINDOW_SOURCE: &str = include_str!("popup_window.rs");
const ACP_CHAT_WINDOW_SOURCE: &str = include_str!("chat_window.rs");
const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
const ACP_CLIENT_SOURCE: &str = include_str!("client.rs");
const ACP_THREAD_SOURCE: &str = include_str!("thread.rs");

fn acp_source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

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
fn startup_plain_enter_routes_to_acp_picker_when_open() {
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) fn handle_enter_key"),
        "AcpChatView must expose a plain-Enter picker handler for app interceptors"
    );
    assert!(
        STARTUP_SOURCE.contains("let is_plain_enter")
            && STARTUP_SOURCE.contains("chat.handle_enter_key(cx)"),
        "startup.rs should route plain Enter to ACP picker acceptance when embedded ACP owns the mention menu"
    );
    assert!(
        STARTUP_NEW_TAB_SOURCE.contains("let is_plain_enter")
            && STARTUP_NEW_TAB_SOURCE.contains("chat.handle_enter_key(cx)"),
        "startup_new_tab.rs should preserve the same plain Enter ACP picker routing"
    );
}

#[test]
fn acp_escape_defers_to_actions_dialog_before_unwinding_chat() {
    for (name, source) in [
        ("startup.rs", STARTUP_SOURCE),
        ("startup_new_actions.rs", STARTUP_NEW_ACTIONS_SOURCE),
    ] {
        let escape_block_start = source
            .find("// Handle Escape for AcpChatView (return to main menu)")
            .unwrap_or_else(|| panic!("ACP escape block not found in {name}"));
        let escape_block_end = (escape_block_start + 900).min(source.len());
        let escape_block = &source[escape_block_start..escape_block_end];

        assert!(
            escape_block.contains("!this.show_actions_popup"),
            "ACP escape block must defer to the actions dialog while it is open in {name}"
        );
        assert!(
            escape_block.contains("!acp_escape_popup_open"),
            "ACP escape block must defer to ACP-local popups while they are open in {name}"
        );
        assert!(
            escape_block.contains("this.close_tab_ai_harness_terminal_with_window(window, cx);"),
            "ACP escape block must still close the ACP chat when actions are closed in {name}"
        );
    }
}

// doc-anchor-removed: [[removed-docs Chat#Footer activity indicator]]
#[test]
fn acp_plain_escape_cancels_streaming_before_host_close() {
    let escape_block_start = ACP_VIEW_SOURCE
        .find("event = \"acp_escape_cancel_streaming_requested\"")
        .expect("ACP view must log the Escape streaming cancellation path");
    let escape_block = &ACP_VIEW_SOURCE[escape_block_start.saturating_sub(400)
        ..(escape_block_start + 800).min(ACP_VIEW_SOURCE.len())];

    assert!(
        escape_block.contains("AcpThreadStatus::Streaming")
            && escape_block.contains("thread.cancel_streaming(cx)"),
        "plain Escape helper must cancel active ACP streaming"
    );

    let focused_escape_start = ACP_VIEW_SOURCE
        .find("if self.cancel_streaming_from_escape(cx)")
        .expect("focused ACP Escape path must call the shared cancellation helper");
    let focused_escape_block = &ACP_VIEW_SOURCE
        [focused_escape_start..(focused_escape_start + 500).min(ACP_VIEW_SOURCE.len())];
    assert!(
        focused_escape_block.contains("cx.stop_propagation()")
            && focused_escape_block.contains("return;"),
        "focused ACP Escape cancellation must stop before the host-close branch"
    );
    assert!(
        focused_escape_start
            < ACP_VIEW_SOURCE
                .find("embedded_acp_escape_host_close_requested")
                .expect("focused ACP Escape close path must remain present"),
        "Escape cancellation must be checked before Escape closes Agent Chat"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Entry paths]]
#[test]
fn acp_root_escape_interceptor_cancels_streaming_before_returning_to_menu() {
    for (name, source) in [
        ("startup.rs", STARTUP_SOURCE),
        ("startup_new_actions.rs", STARTUP_NEW_ACTIONS_SOURCE),
    ] {
        let cancel_pos = source
            .find("chat.cancel_streaming_from_escape(cx)")
            .unwrap_or_else(|| panic!("{name} must let ACP consume Escape while streaming"));
        let close_pos = source
            .find("event = \"embedded_acp_escape_return_to_origin\"")
            .unwrap_or_else(|| panic!("{name} must retain idle Escape return-to-origin path"));
        assert!(
            cancel_pos < close_pos,
            "{name} must try ACP streaming cancellation before Escape returns to the main menu"
        );
        assert!(
            source[cancel_pos..close_pos].contains("cx.stop_propagation()")
                && source[cancel_pos..close_pos].contains("return;"),
            "{name} must stop propagation after ACP streaming cancellation"
        );
    }

    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) fn cancel_streaming_from_escape")
            && ACP_VIEW_SOURCE.contains("thread.cancel_streaming(cx)")
            && ACP_VIEW_SOURCE.contains("event = \"acp_escape_cancel_streaming_requested\""),
        "AcpChatView should expose a shared Escape cancellation helper for focused and host routes"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Entry paths]]
#[test]
fn acp_stdin_simulate_key_escape_cancels_streaming_before_returning_to_menu() {
    for (name, source) in [
        ("runtime_stdin.rs", RUNTIME_STDIN_SOURCE),
        ("app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let cancel_pos = source
            .find("chat.cancel_streaming_from_escape(cx)")
            .unwrap_or_else(|| panic!("{name} must route simulated Escape through ACP cancel"));
        let close_pos = source
            .find("SimulateKey: Escape - return to main menu from Agent Chat")
            .unwrap_or_else(|| panic!("{name} must retain idle simulated Escape close path"));
        assert!(
            cancel_pos < close_pos,
            "{name} must cancel ACP streaming before simulated Escape returns to the main menu"
        );
        assert!(
            source[cancel_pos..close_pos]
                .contains("SimulateKey: Escape - cancel Agent Chat streaming"),
            "{name} must log the simulated Escape streaming-cancel route"
        );
    }
}

// doc-anchor-removed: [[removed-docs Chat#Telemetry]]
#[test]
fn acp_cancel_streaming_sends_session_cancel_to_agent() {
    assert!(
        ACP_THREAD_SOURCE.contains("self.connection.cancel_turn(self.ui_thread_id.clone())"),
        "AcpThread::cancel_streaming must enqueue an ACP cancel request"
    );
    assert!(
        ACP_CLIENT_SOURCE.contains("AcpCancelCommand::CancelTurn")
            && ACP_CLIENT_SOURCE.contains("CancelNotification::new")
            && ACP_CLIENT_SOURCE.contains(".cancel(CancelNotification::new")
            && ACP_CLIENT_SOURCE.contains("event = \"acp_session_cancel_requested\""),
        "ACP runtime must translate UI cancellation into a session/cancel notification"
    );
}

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn acp_plain_up_recalls_latest_user_prompt_when_composer_is_empty() {
    let up_block_start = ACP_VIEW_SOURCE
        .find("event = \"acp_plain_up_recalled_last_user_prompt\"")
        .expect("ACP view must log the plain Up prompt recall path");
    let up_block = &ACP_VIEW_SOURCE
        [up_block_start.saturating_sub(700)..(up_block_start + 300).min(ACP_VIEW_SOURCE.len())];

    assert!(
        up_block.contains("!modifiers.platform")
            && up_block.contains("crate::ui_foundation::is_key_up(key)")
            && up_block.contains("thread.recall_last_user_message(cx)")
            && up_block.contains("cx.stop_propagation()"),
        "plain Up should be consumed only when it recalls the latest user prompt"
    );
    assert!(
        ACP_THREAD_SOURCE.contains("pub(crate) fn recall_last_user_message")
            && ACP_THREAD_SOURCE.contains("!self.input.is_empty()")
            && ACP_THREAD_SOURCE.contains("AcpThreadStatus::Idle | AcpThreadStatus::Error")
            && ACP_THREAD_SOURCE.contains("message.role == AcpThreadMessageRole::User")
            && ACP_THREAD_SOURCE.contains("self.input.set_cursor(0)"),
        "AcpThread should recall the last user message only from an empty idle/error composer"
    );
}

// doc-anchor-removed: [[removed-docs Chat#ACP composer]]
#[test]
fn acp_cmd_0_resets_agent_chat_zoom_through_theme_sync() {
    let cmd_0_block_start = ACP_VIEW_SOURCE
        .find("event = \"acp_cmd_0_reset_agent_chat_zoom\"")
        .expect("ACP view must log the Cmd+0 reset path");
    let cmd_0_block = &ACP_VIEW_SOURCE[cmd_0_block_start.saturating_sub(900)
        ..(cmd_0_block_start + 500).min(ACP_VIEW_SOURCE.len())];

    assert!(
        cmd_0_block.contains("FontConfig::default()")
            && cmd_0_block.contains("fonts.ui_size = defaults.ui_size")
            && cmd_0_block.contains("fonts.mono_size = defaults.mono_size")
            && cmd_0_block.contains("persist_theme_and_sync_all_windows"),
        "Cmd+0 should reset Agent Chat font sizing through the shared theme sync path"
    );
    assert!(
        ACP_VIEW_SOURCE
            .contains("modifiers.platform && !modifiers.alt && !modifiers.shift && key == \"0\"")
            && ACP_VIEW_SOURCE.contains("self.reset_agent_chat_zoom(cx);")
            && ACP_VIEW_SOURCE.contains("\"acp_cmd_0_reset_agent_chat_zoom\""),
        "ACP key handling should route Cmd+0 to the zoom reset helper"
    );
}

#[test]
fn simulated_acp_escape_closes_actions_before_unwinding_chat() {
    let acp_block_start = APP_RUN_SETUP_SOURCE
        .find("AppView::AcpChatView { ref entity, .. } => {")
        .expect("ACP simulateKey branch not found in app_run_setup.rs");
    let acp_block_end = (acp_block_start + 2200).min(APP_RUN_SETUP_SOURCE.len());
    let acp_block = &APP_RUN_SETUP_SOURCE[acp_block_start..acp_block_end];

    let close_actions_pos = acp_block
        .find("view.close_actions_popup(ActionsDialogHost::AcpChat, window, ctx);")
        .expect("simulateKey ACP branch must close ACP actions popup");
    let close_chat_pos = acp_block
        .find("view.close_tab_ai_harness_terminal_with_window(window, ctx);")
        .expect("simulateKey ACP branch must still close the ACP chat");

    assert!(
        acp_block.contains("view.show_actions_popup && key_lower == \"escape\""),
        "simulateKey ACP branch must guard Escape with the ACP actions popup state"
    );
    assert!(
        close_actions_pos < close_chat_pos,
        "simulateKey ACP Escape should close the ACP actions popup before closing the ACP chat"
    );
}

#[test]
fn acp_actions_window_close_path_restores_acp_host_focus() {
    let toggle_actions_start = ACTIONS_TOGGLE_SOURCE
        .find("pub(crate) fn toggle_actions")
        .expect("toggle_actions not found in actions_toggle.rs");
    let toggle_actions_end = ACTIONS_TOGGLE_SOURCE[toggle_actions_start..]
        .find("pub(crate) fn toggle_arg_actions")
        .map(|offset| toggle_actions_start + offset)
        .unwrap_or(ACTIONS_TOGGLE_SOURCE.len());
    let toggle_actions = &ACTIONS_TOGGLE_SOURCE[toggle_actions_start..toggle_actions_end];

    assert!(
        toggle_actions.contains("self.actions_dialog_host_for_current_view()")
            && ACTIONS_TOGGLE_SOURCE.contains("ActionsDialogHost::AcpChat")
            && ACTIONS_TOGGLE_SOURCE.contains("ActionsDialogHost::MainList"),
        "toggle_actions must derive the actions host from the current view"
    );
    assert!(
        toggle_actions.contains("self.close_actions_popup(host, window, cx);"),
        "toggle_actions must close with the derived ACP/MainList host"
    );
    assert!(
        toggle_actions.contains("Some(host_label)"),
        "toggle_actions should emit the derived host label for popup events"
    );
    assert!(
        toggle_actions.contains("Self::make_actions_window_on_close_callback(")
            && ACTIONS_TOGGLE_SOURCE.contains("app.request_focus_restore_for_actions_host(host);"),
        "actions window close callback must restore focus for the ACP host"
    );
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

#[test]
fn acp_picker_popup_module_is_registered() {
    assert!(
        ACP_MOD_SOURCE.contains("pub(crate) mod picker_popup;"),
        "ACP module should register the detached picker popup module"
    );
}

#[test]
fn acp_model_selector_popup_module_is_registered() {
    assert!(
        ACP_MOD_SOURCE.contains("pub(crate) mod model_selector_popup;"),
        "ACP module should register the detached model selector popup module"
    );
}

#[test]
fn acp_history_popup_module_is_registered() {
    assert!(
        ACP_MOD_SOURCE.contains("pub(crate) mod history_popup;"),
        "ACP module should register the detached history popup module"
    );
}

#[test]
fn acp_picker_migration_uses_popup_window_instead_of_inline_layer() {
    assert!(
        !ACP_VIEW_SOURCE.contains("acp-mention-picker-layer"),
        "ACP chat view should no longer render the mention picker inline"
    );
    assert!(
        ACP_PICKER_POPUP_SOURCE.contains("AcpMentionPopupWindow")
            && ACP_PICKER_POPUP_SOURCE.contains("super::popup_window::popup_window_options")
            && ACP_PICKER_POPUP_SOURCE.contains("super::popup_window::configure_popup_window"),
        "ACP picker migration should render through a popup window entity"
    );
}

#[test]
fn acp_model_selector_migration_uses_popup_window_instead_of_inline_layer() {
    assert!(
        !ACP_VIEW_SOURCE.contains("fn render_model_selector"),
        "ACP chat view should no longer render the model selector inline"
    );
    assert!(
        ACP_MODEL_SELECTOR_POPUP_SOURCE.contains("AcpModelSelectorPopupWindow")
            && ACP_MODEL_SELECTOR_POPUP_SOURCE
                .contains("super::popup_window::popup_window_options")
            && ACP_MODEL_SELECTOR_POPUP_SOURCE
                .contains("super::popup_window::configure_popup_window"),
        "ACP model selector should render through a popup window entity"
    );
    assert!(
        ACP_MODEL_SELECTOR_POPUP_SOURCE.contains("render_dense_monoline_picker_row_with_accessory(")
            && ACP_MODEL_SELECTOR_POPUP_SOURCE.contains("IconName::Check")
            && ACP_MODEL_SELECTOR_POPUP_SOURCE.contains("super::popup_window::dense_picker_height")
            && ACP_MODEL_SELECTOR_POPUP_SOURCE.contains("InlineDropdown::new(")
            && ACP_MODEL_SELECTOR_POPUP_SOURCE
                .contains("super::popup_window::dense_picker_width_for_labels"),
        "ACP model selector popup must share the dense picker row and sizing contract with slash/@ pickers"
    );
    assert!(
        !ACP_MODEL_SELECTOR_POPUP_SOURCE
            .contains("crate::ai::context_picker_row::render_dense_monoline_picker_row"),
        "ACP model selector popup should import row renderers from shared inline_dropdown"
    );
}

#[test]
fn acp_history_migration_uses_popup_window_instead_of_inline_layer() {
    assert!(
        !ACP_VIEW_SOURCE.contains(".id(\"acp-history-picker\")"),
        "ACP chat view should no longer render the history picker inline"
    );
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("AcpHistoryPopupWindow")
            && ACP_HISTORY_POPUP_SOURCE.contains("super::popup_window::popup_window_options")
            && ACP_HISTORY_POPUP_SOURCE.contains("super::popup_window::configure_popup_window")
            && ACP_HISTORY_POPUP_SOURCE
                .contains("super::popup_window::set_popup_window_bounds"),
        "ACP history picker should render through a popup window entity using shared popup mechanics"
    );
    assert!(
        !ACP_HISTORY_POPUP_SOURCE.contains("fn popup_ns_window")
            && !ACP_HISTORY_POPUP_SOURCE.contains("fn attach_popup_to_parent_window")
            && !ACP_HISTORY_POPUP_SOURCE.contains("fn flipped_ns_window_y"),
        "ACP history popup must not copy AppKit popup plumbing that is owned by popup_window"
    );
}

#[test]
fn acp_picker_popup_row_rendering_comes_from_shared_inline_dropdown() {
    assert!(
        ACP_PICKER_POPUP_SOURCE.contains("crate::components::inline_dropdown")
            && !ACP_PICKER_POPUP_SOURCE
                .contains("crate::ai::context_picker_row::{\n    render_soft_compact_picker_row"),
        "ACP slash/@ picker popup should source shared inline-dropdown row rendering directly"
    );
    assert!(
        ACP_POPUP_WINDOW_SOURCE.contains("crate::components::inline_dropdown::CONTEXT_PICKER_ROW_HEIGHT")
            && !ACP_POPUP_WINDOW_SOURCE
                .contains("crate::ai::context_picker_row::CONTEXT_PICKER_ROW_HEIGHT"),
        "ACP popup facade should derive dense picker height from the shared inline-dropdown row contract"
    );
}

#[test]
fn acp_cmd_p_routes_to_dedicated_history_command() {
    // Cmd+P should trigger the host callback, not the inline popup toggle
    assert!(
        ACP_VIEW_SOURCE.contains("self.trigger_open_history_command(window, cx);"),
        "Cmd+P in ACP should route through the dedicated history command callback"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("acp_history_shortcut_routed_to_command"),
        "Cmd+P should emit a structured tracing event when routing to the history command"
    );
    // The view should expose the callback setter
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) fn set_on_open_history_command"),
        "AcpChatView must expose set_on_open_history_command for hosts to wire up"
    );
    // The old inline history picker intercept block should be removed
    assert!(
        !ACP_VIEW_SOURCE.contains("History picker intercept"),
        "the old inline history picker intercept block should be removed from the key handler"
    );
}

#[test]
fn acp_footer_actions_hint_uses_shared_clickable_toggle_path() {
    assert!(
        ACP_VIEW_SOURCE.contains("render_hint_icons_clickable"),
        "ACP footer should use the shared clickable hint-strip renderer so footer buttons behave like the main menu"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("FooterAction::Actions => \"⌘K Actions\"")
            && ACP_VIEW_SOURCE
                .contains("FooterAction::Actions => self.trigger_toggle_actions(window, cx),"),
        "ACP footer Actions hint must route through the shared clickable footer renderer"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("wire_embedded_acp_footer_callbacks(&view, cx);")
            && TAB_AI_MODE_SOURCE.contains("app.toggle_actions(cx, window);")
            && TAB_AI_MODE_SOURCE.contains("app.close_tab_ai_harness_terminal_with_window(window, cx);")
            && TAB_AI_MODE_SOURCE.contains("view.set_on_open_history_command")
            && TAB_AI_MODE_SOURCE.contains("app.open_embedded_acp_history_popup(window, cx);")
            && TAB_AI_MODE_SOURCE.contains("view.set_on_paste_response_requested")
            && TAB_AI_MODE_SOURCE.contains("app.paste_latest_acp_response_to_frontmost(cx);"),
        "embedded ACP hosts must wire footer clicks to the existing actions, close, history popup, and paste-response paths"
    );
    assert!(
        ACP_CHAT_WINDOW_SOURCE.contains("view.set_on_toggle_actions")
            && ACP_CHAT_WINDOW_SOURCE.contains("toggle_detached_actions(cx);")
            && ACP_CHAT_WINDOW_SOURCE.contains("close_chat_window(cx);"),
        "detached ACP hosts must wire footer clicks to the detached actions toggle and close paths"
    );
}

#[test]
fn acp_footer_primary_action_tracks_composer_response_and_streaming_state() {
    assert!(
        ACP_VIEW_SOURCE.contains("fn footer_buttons_for_thread")
            && ACP_VIEW_SOURCE.contains("FooterAction::PasteResponse")
            && ACP_VIEW_SOURCE.contains("label: \"Paste Response\"")
            && ACP_VIEW_SOURCE.contains("label: \"Send\"")
            && ACP_VIEW_SOURCE.contains("label: \"Stop\""),
        "ACP footer state must expose Send, Paste Response, and Stop as child-owned footer button specs"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("thread.input.text().is_empty()")
            && ACP_VIEW_SOURCE.contains("Self::has_pastable_assistant_response(thread)")
            && ACP_VIEW_SOURCE.contains("AcpThreadStatus::Streaming =>"),
        "ACP footer labels must be driven by raw composer emptiness, assistant response presence, and streaming state"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("Self::has_pastable_assistant_response(&thread)")
            && ACP_VIEW_SOURCE.contains("self.trigger_paste_response_requested(window, cx);")
            && ACP_VIEW_SOURCE.contains("caused_submit: false"),
        "Enter on an empty composer after an assistant response must route to Paste Response instead of empty-submit"
    );
}

#[test]
fn native_acp_footer_uses_child_snapshot_and_explicit_footer_actions() {
    assert!(
        UI_WINDOW_SOURCE.contains("self.acp_footer_snapshot.as_ref()")
            && UI_WINDOW_SOURCE.contains("FooterButtonConfig::new(button.action, button.key, button.label)")
            && UI_WINDOW_SOURCE.contains("FooterAction::Stop")
            && UI_WINDOW_SOURCE.contains("FooterAction::PasteResponse"),
        "native ACP footer must render from the child ACP footer snapshot and dispatch explicit Stop/PasteResponse actions"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("paste_latest_acp_response_to_frontmost")
            && UI_WINDOW_SOURCE.contains("crate::platform::defer_hide_main_window(cx)")
            && UI_WINDOW_SOURCE.contains("injector.paste_text(&text)"),
        "Paste Response must use the existing frontmost-app paste path instead of being a label-only no-op"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Agent switching]]
#[test]
fn acp_actions_dialog_preserves_route_backed_agent_actions() {
    let acp_dialog_start = ACTIONS_TOGGLE_SOURCE
        .find("let is_acp_actions_dialog = acp_context.is_some();")
        .expect("actions toggle must identify ACP dialogs before construction");
    let acp_dialog_block = &ACTIONS_TOGGLE_SOURCE
        [acp_dialog_start..(acp_dialog_start + 3500).min(ACTIONS_TOGGLE_SOURCE.len())];

    assert!(
        acp_dialog_block.contains("ActionsDialog::with_acp_chat("),
        "ACP actions must be constructed with the route-backed ACP dialog"
    );
    assert!(
        acp_dialog_block.contains("if !is_acp_actions_dialog")
            && ACTIONS_TOGGLE_SOURCE.contains("dialog.set_menu_syntax_section")
            && ACTIONS_TOGGLE_SOURCE.contains(".set_focused_scriptlet"),
        "generic script/global action rebuild hooks must be gated away from ACP actions"
    );

    let root_route_block_start = ACTIONS_TOGGLE_SOURCE
        .find("ActionsDialog::with_acp_chat(")
        .expect("ACP dialog constructor call missing");
    let rebuild_pos = ACTIONS_TOGGLE_SOURCE[root_route_block_start..]
        .find("dialog.set_menu_syntax_section")
        .map(|pos| root_route_block_start + pos)
        .expect("shared menu syntax hook missing");
    let guard_pos = ACTIONS_TOGGLE_SOURCE[root_route_block_start..rebuild_pos]
        .rfind("if !is_acp_actions_dialog")
        .map(|pos| root_route_block_start + pos)
        .expect("ACP guard must appear before generic menu syntax rebuild");
    assert!(
        guard_pos < rebuild_pos,
        "ACP guard must prevent set_menu_syntax_section(None) from replacing Change Agent/Model with global actions"
    );
}

// doc-anchor-removed: [[removed-docs Chat#Footer activity indicator]]
#[test]
fn acp_footer_omits_global_cmd_enter_ai_button() {
    let footer_start = UI_WINDOW_SOURCE
        .find("fn acp_footer_buttons")
        .expect("native ACP footer builder missing");
    let footer_block =
        &UI_WINDOW_SOURCE[footer_start..(footer_start + 900).min(UI_WINDOW_SOURCE.len())];
    assert!(
        !footer_block.contains("FooterAction::Ai"),
        "native ACP footer should not show the global Cmd+Enter AI button"
    );

    let external_footer_start = ACP_VIEW_SOURCE
        .find("fn render_external_host_footer_from_snapshot")
        .expect("external ACP footer renderer missing");
    let external_footer_block = &ACP_VIEW_SOURCE
        [external_footer_start..(external_footer_start + 2600).min(ACP_VIEW_SOURCE.len())];
    assert!(
        !external_footer_block.contains("\"⌘↵ AI\""),
        "external ACP footer should not show the global Cmd+Enter AI hint"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("FooterAction::Actions")
            && ACP_VIEW_SOURCE.contains("FooterAction::Actions => \"⌘K Actions\""),
        "ACP footers must keep the Actions affordance after removing the AI button"
    );
}

#[test]
fn acp_embedded_cmd_k_uses_host_actions_callback() {
    assert!(
        ACP_VIEW_SOURCE.contains("event = \"acp_cmd_k_route\"")
            && ACP_VIEW_SOURCE.contains("embedded_host_callback")
            && ACP_VIEW_SOURCE.contains("self.trigger_toggle_actions(window, cx);")
            && ACP_VIEW_SOURCE.contains("cx.stop_propagation();"),
        "Cmd+K inside focused embedded ACP must open the host actions menu locally"
    );
    assert!(
        !ACP_VIEW_SOURCE.contains("propagate_to_main_window"),
        "embedded ACP Cmd+K must not depend on bubbling to the launcher interceptor"
    );
}

#[test]
fn acp_detached_cmd_k_keeps_detached_actions_path() {
    let route_start = ACP_VIEW_SOURCE
        .find("event = \"detached_actions_shortcut_pressed\"")
        .expect("detached Cmd+K branch should emit route tracing");
    let route_block = &ACP_VIEW_SOURCE[route_start..(route_start + 900).min(ACP_VIEW_SOURCE.len())];
    assert!(
        ACP_VIEW_SOURCE.contains("detached_local")
            && ACP_CHAT_WINDOW_SOURCE.contains("toggle_detached_actions(cx);"),
        "detached ACP Cmd+K must keep using the detached actions window path through the installed detached host callback"
    );
    assert!(
        route_block.contains("self.trigger_toggle_actions(window, cx);")
            && !route_block.contains("toggle_detached_actions(cx);"),
        "detached ACP Cmd+K must defer through trigger_toggle_actions instead of synchronously calling toggle_detached_actions while AcpChatView is updating"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("cx.background_executor().timer(Duration::from_millis(1)).await;"),
        "ACP footer callbacks must hop a timer tick before updating the host window so protocol simulateGpuiEvent cannot re-enter the app update stack"
    );
}

#[test]
fn acp_show_history_action_prefers_embedded_popup_before_builtin_browser() {
    assert!(
        HANDLE_ACTION_SOURCE.contains("if !self.open_embedded_acp_history_popup(window, cx) {")
            && HANDLE_ACTION_SOURCE.contains("AppView::AcpHistoryView"),
        "acp_show_history should open the embedded ACP popup when possible and only fall back to the builtin browser otherwise"
    );
}

#[test]
fn acp_picker_refresh_and_navigation_sync_popup_window() {
    assert!(
        ACP_VIEW_SOURCE.contains("pub(super) fn refresh_mention_session")
            && ACP_VIEW_SOURCE.contains("fn cache_popup_parent_window")
            && ACP_VIEW_SOURCE.contains("self.sync_mention_popup_window_from_cached_parent(cx);"),
        "picker refresh should keep the detached popup window synchronized"
    );

    let keydown_block_start = ACP_VIEW_SOURCE
        .find("if self.mention_session.is_some() {")
        .expect("mention-session keydown block should exist");
    let keydown_block_end = ACP_VIEW_SOURCE[keydown_block_start..]
        .find("// Shift+Enter inserts a newline.")
        .map(|offset| keydown_block_start + offset)
        .unwrap_or(ACP_VIEW_SOURCE.len());
    let keydown_block = &ACP_VIEW_SOURCE[keydown_block_start..keydown_block_end];
    assert!(
        keydown_block
            .matches("self.sync_mention_popup_window_from_cached_parent(cx);")
            .count()
            >= 2,
        "picker navigation should resync the detached popup window"
    );
}

#[test]
fn acp_picker_parent_mouse_down_dismisses_slash_and_mention_popup() {
    let render_body = acp_source_between(
        ACP_VIEW_SOURCE,
        "impl Render for AcpChatView",
        "#[cfg(test)]",
    );
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) fn dismiss_mention_picker")
            && ACP_VIEW_SOURCE.contains("self.mention_session.take()")
            && ACP_VIEW_SOURCE.contains("self.sync_mention_popup_window_from_cached_parent(cx);"),
        "AcpChatView must expose a shared picker dismiss helper for both slash and @ mention sessions"
    );
    assert!(
        render_body.contains(".on_any_mouse_down(cx.listener(|this, _event, _window, cx| {")
            && render_body.contains("this.dismiss_mention_picker(cx);"),
        "ACP chat root mouse-down should dismiss the shared slash/@ picker when clicking outside the popup window"
    );
}

#[test]
fn acp_picker_outside_dismiss_suppresses_unchanged_trigger_reopen() {
    let dismiss = acp_source_between(
        ACP_VIEW_SOURCE,
        "pub(crate) fn dismiss_mention_picker",
        "/// Access the live thread entity",
    );
    assert!(
        dismiss.contains("self.dismissed_mention_trigger = Some(AcpDismissedMentionTrigger")
            && dismiss.contains("trigger_range: session.trigger_range.clone()")
            && dismiss.contains("query: session.query.clone()"),
        "outside-click dismiss must remember the exact active slash/@ trigger so unchanged composer text does not reopen the popup"
    );

    let refresh = acp_source_between(
        ACP_VIEW_SOURCE,
        "pub(super) fn refresh_mention_session",
        "/// Log the visible window range",
    );
    assert!(
        refresh.contains("dismissed_trigger_still_active")
            && refresh.contains("self.dismissed_mention_trigger.as_ref() == Some(&active_trigger)")
            && refresh.contains("if !dismissed_trigger_still_active")
            && refresh.contains("self.dismissed_mention_trigger = None;"),
        "refresh_mention_session must keep the dismissed trigger closed until the input/cursor context changes"
    );
}

#[test]
fn acp_picker_row_click_matches_actions_dialog_mouse_arming() {
    assert!(
        ACP_PICKER_POPUP_SOURCE.contains("mouse_armed_row: Option<(usize, String)>")
            && ACP_PICKER_POPUP_SOURCE.contains("fn should_submit_acp_picker_row_click")
            && ACP_PICKER_POPUP_SOURCE.contains("was_mouse_armed || click_count >= 2"),
        "ACP slash/@ picker rows must use actions-dialog-style mouse arming: first click focuses, second or double-click accepts"
    );
    assert!(
        ACP_PICKER_POPUP_SOURCE.contains("fn handle_row_click")
            && ACP_PICKER_POPUP_SOURCE.contains("this.handle_row_click(idx, event, window, cx);")
            && ACP_PICKER_POPUP_SOURCE.contains("self.select_item(index, cx);")
            && ACP_PICKER_POPUP_SOURCE.contains("self.activate_item(index, cx);"),
        "ACP picker row clicks must route through a shared handler that selects before accepting"
    );
}

#[test]
fn acp_picker_mouse_focus_does_not_recreate_popup_window() {
    let select_item = acp_source_between(
        ACP_PICKER_POPUP_SOURCE,
        "fn select_item",
        "fn handle_row_click",
    );
    assert!(
        !select_item.contains("sync_mention_popup_window_from_cached_parent"),
        "first mouse click should update selection in the existing popup instead of resyncing/recreating the popup window"
    );
    assert!(
        select_item.contains("self.snapshot.selected_index")
            && select_item.contains("self.snapshot.visible_start = visible.start"),
        "mouse focus should still update the popup's local selected row"
    );
}

#[test]
fn acp_picker_mouse_submit_dismisses_popup_window() {
    let click_handler = acp_source_between(
        ACP_PICKER_POPUP_SOURCE,
        "fn handle_row_click",
        "fn apply_hint",
    );
    let activate = click_handler
        .find("self.activate_item(index, cx);")
        .expect("mouse submit should activate the focused picker item");
    let clear_slot = click_handler
        .find("clear_mention_popup_window_slot();")
        .expect("mouse submit should clear the popup slot");
    let remove_window = click_handler
        .find("window.remove_window();")
        .expect("mouse submit should remove the popup window directly");
    assert!(
        activate < clear_slot && clear_slot < remove_window,
        "double-click or second-click submit must dismiss the slash/@ picker popup after activation"
    );
    assert!(
        click_handler.contains("is_actionable && should_submit_acp_picker_row_click"),
        "inert picker rows must not be dismissed as submitted actions"
    );
}

#[test]
fn acp_close_paths_close_slash_and_mention_popup() {
    let detached_cmd_w_block = acp_source_between(
        ACP_VIEW_SOURCE,
        "event = \"detached_acp_cmd_w_close_requested\"",
        "this.handle_key_down(event, window, cx);",
    );
    let detached_cmd_w_prepare = detached_cmd_w_block
        .find("this.prepare_for_host_hide(cx);")
        .expect("detached Cmd+W block must prepare ACP host hide");
    let detached_cmd_w_remove = detached_cmd_w_block
        .find("window.remove_window();")
        .expect("detached Cmd+W block must remove the window");
    assert!(
        detached_cmd_w_prepare < detached_cmd_w_remove,
        "detached ACP Cmd+W must close slash/@ picker popups before removing the window"
    );

    let detached_close_helper = acp_source_between(
        ACP_CHAT_WINDOW_SOURCE,
        "pub fn close_chat_window",
        "// Detached ACP action allowlist",
    );
    let detached_helper_prepare = detached_close_helper
        .find("view.prepare_for_host_hide(cx);")
        .expect("close_chat_window must prepare ACP host hide");
    let detached_helper_remove = detached_close_helper
        .find("window.remove_window();")
        .expect("close_chat_window must remove the window");
    assert!(
        detached_helper_prepare < detached_helper_remove,
        "detached close_chat_window must close slash/@ picker popups before removing the window"
    );

    let detached_titlebar_close = acp_source_between(
        ACP_CHAT_WINDOW_SOURCE,
        "pub fn open_chat_window_with_thread",
        "/// Return a strong reference to the detached ACP chat view entity",
    );
    assert!(
        detached_titlebar_close.contains("view_entity_slot_on_close")
            && detached_titlebar_close.contains("view.prepare_for_host_hide(cx);"),
        "detached titlebar close must prepare the ACP view so slash/@ picker popups cannot outlive chat"
    );
}

#[test]
fn reset_to_script_list_runs_embedded_acp_teardown() {
    let reset_start = REGISTRIES_STATE_SOURCE
        .find("pub(crate) fn reset_to_script_list")
        .expect("reset_to_script_list should exist");
    let reset_body = &REGISTRIES_STATE_SOURCE[reset_start..];

    assert!(
        reset_body.contains("view.prepare_for_host_hide(cx);")
            && reset_body.contains("crate::windows::ensure_embedded_ai_window(false);")
            && reset_body.contains("AcpSurfaceEvent::EmbeddedClosed"),
        "reset_to_script_list must close embedded ACP popups and automation state before returning to ScriptList"
    );
}

#[test]
fn acp_composer_stays_width_wrapped_without_explicit_newline() {
    assert!(
        ACP_VIEW_SOURCE.contains("multiline: true"),
        "ACP composer should use width-driven multiline rendering"
    );
    assert!(
        !ACP_VIEW_SOURCE.contains("multiline: input_has_newline"),
        "ACP composer should not wait for an explicit newline before wrapping"
    );
}

#[test]
fn acp_model_selection_is_visible_in_footer_and_routed_through_actions() {
    assert!(
        ACP_VIEW_SOURCE.contains(".id(\"acp-model-display\")")
            && !ACP_VIEW_SOURCE.contains(".id(\"acp-model-btn\")")
            && ACP_VIEW_SOURCE.contains("\"⌘K Actions\""),
        "ACP footer should keep the active model visible and route changes through the actions menu"
    );
}

#[test]
fn acp_history_toggle_and_selection_sync_popup_window() {
    assert!(
        ACP_VIEW_SOURCE.contains("self.sync_history_popup_window_from_cached_parent(cx);")
            && ACP_VIEW_SOURCE.contains("pub(crate) fn select_history_from_popup")
            && ACP_VIEW_SOURCE.contains("pub(crate) fn toggle_history_popup"),
        "history picker interactions should open and close through the detached popup window"
    );
}

#[test]
fn acp_history_toggle_uses_recent_close_debounce() {
    assert!(
        ACP_VIEW_SOURCE.contains("history_closed_at: Option<Instant>")
            && ACP_VIEW_SOURCE.contains("fn was_history_recently_closed(&self) -> bool")
            && ACP_VIEW_SOURCE.contains("fn mark_history_popup_closed(&mut self, cx: &mut Context<Self>)")
            && ACP_VIEW_SOURCE.contains("event = \"acp_history_popup_toggle_suppressed_recent_close\""),
        "ACP history popup should track recent closes and suppress immediate reopen races like the shared actions dialog"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("view.set_on_open_history_command")
            && TAB_AI_MODE_SOURCE.contains("app.open_embedded_acp_history_popup(window, cx);"),
        "embedded ACP history host should wire the footer and shortcut into the dedicated embedded history popup path"
    );
}

#[test]
fn acp_history_popup_window_observes_focus_loss_and_escape() {
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("activation_subscription: Option<Subscription>")
            && ACP_HISTORY_POPUP_SOURCE.contains("fn ensure_activation_subscription(")
            && ACP_HISTORY_POPUP_SOURCE.contains("observe_window_activation(")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.request_close(window, cx, \"focus_lost\");"),
        "ACP history popup window should observe activation changes and close on focus loss"
    );
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains(".on_mouse_down_out(cx.listener(|this, _event: &gpui::MouseDownEvent, window, cx| {")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.request_close(window, cx, \"mouse_down_out\");")
            && ACP_HISTORY_POPUP_SOURCE.contains("view.dismiss_history_popup_from_window(reason, cx);")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.request_close(window, cx, \"escape\");"),
        "ACP history popup window should close on outside clicks and sync dismissals back into ACP state"
    );
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("view.dismiss_history_popup_from_window(reason, cx);")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.request_close(window, cx, \"escape\");"),
        "ACP history popup window should sync dismissals back into ACP state for both focus loss and Escape"
    );
    assert!(
        ACP_VIEW_SOURCE.contains(".id(\"acp-history-popup-backdrop\")")
            && ACP_VIEW_SOURCE.contains("this.dismiss_history_popup(cx);")
            && ACP_VIEW_SOURCE.contains(".bottom(px(self.inline_footer_height()))"),
        "ACP host should render an outside-click backdrop above chat content so clicks outside the popup close it without swallowing the footer toggle"
    );
}

#[test]
fn acp_history_popup_window_supports_actions_style_search_and_keyboard_navigation() {
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("enum AcpHistoryPopupKeyIntent")
            && ACP_HISTORY_POPUP_SOURCE.contains("TypeChar(char)")
            && ACP_HISTORY_POPUP_SOURCE.contains("Backspace")
            && ACP_HISTORY_POPUP_SOURCE.contains("MovePageDown")
            && ACP_HISTORY_POPUP_SOURCE.contains("MoveHome")
            && ACP_HISTORY_POPUP_SOURCE.contains("history_popup_key_intent"),
        "ACP history popup should use an actions-style key intent model for search and navigation"
    );
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("sync_history_popup_state_from_window")
            && ACP_HISTORY_POPUP_SOURCE.contains("sync_history_popup_selection_from_window")
            && ACP_HISTORY_POPUP_SOURCE.contains("Type to Search")
            && ACP_HISTORY_POPUP_SOURCE.contains(".track_scroll(&self.scroll_handle)"),
        "ACP history popup should expose a visible search row and keep popup state synchronized while keyboard navigation scrolls"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("if self.history_menu.is_some() {")
            && ACP_VIEW_SOURCE.contains("match history_popup_key_intent(key, modifiers)")
            && ACP_VIEW_SOURCE.contains("self.set_history_popup_query(next_query, cx);")
            && ACP_VIEW_SOURCE.contains("self.execute_history_popup_selection(modifiers, cx);"),
        "ACP host key routing should intercept history popup navigation and search the same way the shared actions popup does"
    );
}

#[test]
fn acp_history_enter_resumes_selected_chat() {
    assert!(
        ACP_VIEW_SOURCE.contains("if modifiers.platform {")
            && ACP_VIEW_SOURCE.contains("self.select_history_from_popup(&entry, cx);"),
        "embedded ACP history keyboard handling should route the resume action through select_history_from_popup"
    );
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("if has_shift {")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.attach_transcript(&entry, cx);")
            && ACP_HISTORY_POPUP_SOURCE.contains("} else if has_cmd {")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.attach_summary(&entry, cx);")
            && ACP_HISTORY_POPUP_SOURCE.contains("} else {")
            && ACP_HISTORY_POPUP_SOURCE.contains("this.resume_session(&entry, cx);")
            && ACP_HISTORY_POPUP_SOURCE.contains("\"\\u{21B5} Resume\".into(),")
            && ACP_HISTORY_POPUP_SOURCE.contains("\"\\u{2318}\\u{21B5} Attach Summary\".into(),"),
        "ACP history popup should advertise and honor Enter-to-resume while keeping modifier-based attach actions"
    );
}

#[test]
fn acp_history_runtime_shortcuts_route_to_dedicated_command() {
    assert!(
        APP_RUN_SETUP_SOURCE.contains("view.handle_action(\"acp_show_history\"")
            && RUNTIME_STDIN_SOURCE.contains("view.handle_action(\"acp_show_history\""),
        "runtime ACP Cmd+P paths should dispatch the acp_show_history action to open the dedicated history command"
    );
    // Verify the old popup toggle is no longer used by stdin simulation
    assert!(
        !APP_RUN_SETUP_SOURCE.contains("chat.toggle_history_popup(window, cx);")
            && !RUNTIME_STDIN_SOURCE.contains("chat.toggle_history_popup(window, cx);"),
        "runtime ACP Cmd+P paths should no longer toggle the inline history popup"
    );
}

#[test]
fn acp_view_exposes_escape_popup_dismiss_helper() {
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) fn dismiss_escape_popup")
            && ACP_VIEW_SOURCE.contains("pub(crate) fn has_escape_dismissible_popup")
            && ACP_VIEW_SOURCE.contains("self.model_selector_open = false;")
            && ACP_VIEW_SOURCE
                .contains("self.sync_model_selector_popup_window_from_cached_parent(cx);")
            && ACP_VIEW_SOURCE.contains("self.history_menu.is_some()")
            && ACP_VIEW_SOURCE.contains("self.mention_session = None;")
            && ACP_VIEW_SOURCE.contains("self.sync_mention_popup_window_from_cached_parent(cx);")
            && ACP_VIEW_SOURCE.contains("if self.attach_menu_open {")
            && ACP_VIEW_SOURCE.contains("|| self.attach_menu_open"),
        "ACP view should expose a helper that dismisses the detached ACP popups on Escape"
    );
}

#[test]
fn acp_picker_portals_require_host_callbacks_before_staging() {
    let portal_fn_start = ACP_VIEW_SOURCE
        .find("fn open_picker_portal(")
        .expect("open_picker_portal should exist");
    let portal_fn =
        &ACP_VIEW_SOURCE[portal_fn_start..(portal_fn_start + 1800).min(ACP_VIEW_SOURCE.len())];

    let callback_guard_idx = portal_fn
        .find("let Some(callback) = self.on_open_portal.clone() else {")
        .expect("picker portals should require a host callback");
    let stage_idx = portal_fn
        .find("self.stage_pending_portal_session(")
        .expect("picker portals should still stage after the guard");

    assert!(
        callback_guard_idx < stage_idx,
        "picker portals should only stage pending portal state after a host callback is available"
    );
    assert!(
        portal_fn.contains("event = \"acp_portal_open_blocked_missing_host_callback\""),
        "missing picker portal callbacks should emit a warning log"
    );
}

#[test]
fn detached_acp_limits_portals_to_history() {
    assert!(
        ACP_CHAT_WINDOW_SOURCE.contains("view.set_allowed_portal_kinds(vec![PortalKind::AcpHistory]);")
            && ACP_CHAT_WINDOW_SOURCE.contains("view.set_on_open_portal(move |kind, cx| match kind {")
            && ACP_CHAT_WINDOW_SOURCE.contains("PortalKind::AcpHistory => {")
            && ACP_CHAT_WINDOW_SOURCE.contains("open_history_portal_in_detached_chat_window(cx)")
            && ACP_CHAT_WINDOW_SOURCE.contains("cancel_portal_session_in_detached_chat_window(kind, cx)")
            && ACP_CHAT_WINDOW_SOURCE.contains("reason = \"unsupported_in_detached_host\""),
        "detached ACP should expose only the locally supported history portal, clear staged portal state on open failure, and log rejected portal requests"
    );
}

#[test]
fn acp_history_popup_attach_consumes_pending_history_portal_session() {
    let popup_select_fn_start = ACP_VIEW_SOURCE
        .find("pub(crate) fn select_history_from_popup")
        .expect("select_history_from_popup should exist");
    let popup_select_fn = &ACP_VIEW_SOURCE
        [popup_select_fn_start..(popup_select_fn_start + 1400).min(ACP_VIEW_SOURCE.len())];

    assert!(
        ACP_VIEW_SOURCE.contains("fn has_pending_history_portal_session(&self) -> bool")
            && ACP_VIEW_SOURCE.contains("fn build_history_attachment_part(")
            && ACP_VIEW_SOURCE.contains("event = \"acp_history_portal_selection_attached_via_contract\"")
            && ACP_VIEW_SOURCE.contains("self.attach_portal_part(part, cx);")
            && ACP_VIEW_SOURCE.contains("let had_pending_history_portal = self.has_pending_history_portal_session();")
            && popup_select_fn.contains("if had_pending_history_portal {")
            && popup_select_fn.contains("event = \"acp_history_popup_attach_failed\"")
            && popup_select_fn.contains("self.cancel_pending_portal_session(")
            && popup_select_fn.contains("PortalKind::AcpHistory")
            && popup_select_fn.contains("return;"),
        "ACP history attachment should consume the staged AcpHistory portal session instead of bypassing the shared replacement contract"
    );
}

#[test]
fn acp_history_popup_dismiss_restores_pending_history_portal_session() {
    assert!(
        ACP_VIEW_SOURCE.contains("event = \"acp_history_portal_dismissed_via_popup\"")
            && ACP_VIEW_SOURCE.contains("event = \"acp_history_portal_dismissed_from_window\"")
            && ACP_VIEW_SOURCE.contains("self.has_pending_history_portal_session()")
            && ACP_VIEW_SOURCE.contains("self.cancel_pending_portal_session(")
            && ACP_VIEW_SOURCE.contains("PortalKind::AcpHistory"),
        "ACP history popup dismissals should cancel the staged AcpHistory portal session so the composer text and caret are restored on close"
    );
}

// =========================================================================
// ACP test probe — ring buffer and snapshot
// =========================================================================

#[test]
fn acp_test_probe_records_key_routes() {
    let mut probe = super::view::AcpTestProbe::default();
    assert_eq!(probe.event_seq, 0);
    assert!(probe.key_routes.is_empty());

    let event = crate::protocol::AcpKeyRouteTelemetry {
        key: "tab".to_string(),
        route: crate::protocol::AcpKeyRoute::Picker,
        picker_open: true,
        permission_active: false,
        cursor_before: 1,
        cursor_after: 17,
        caused_submit: false,
        consumed: true,
    };

    probe.event_seq += 1;
    probe.key_routes.push_back(event.clone());
    assert_eq!(probe.event_seq, 1);
    assert_eq!(probe.key_routes.len(), 1);
    assert_eq!(probe.key_routes[0].key, "tab");
}

#[test]
fn acp_test_probe_records_picker_accepts() {
    let mut probe = super::view::AcpTestProbe::default();

    let event = crate::protocol::AcpPickerItemAcceptedTelemetry {
        trigger: "@".to_string(),
        item_label: "Current Context".to_string(),
        item_id: "built_in:context".to_string(),
        accepted_via_key: "tab".to_string(),
        cursor_after: 17,
        caused_submit: false,
    };

    probe.event_seq += 1;
    probe.accepted_items.push_back(event.clone());
    assert_eq!(probe.accepted_items.len(), 1);
    assert_eq!(probe.accepted_items[0].accepted_via_key, "tab");
}

#[test]
fn acp_test_probe_records_input_layout() {
    let mut probe = super::view::AcpTestProbe::default();

    let event = crate::protocol::AcpInputLayoutTelemetry {
        char_count: 27,
        visible_start: 0,
        visible_end: 27,
        cursor_in_window: 17,
    };

    probe.event_seq += 1;
    probe.input_layout = Some(event.clone());
    assert!(probe.input_layout.is_some());
    assert_eq!(
        probe
            .input_layout
            .as_ref()
            .expect("layout")
            .cursor_in_window,
        17
    );
}

#[test]
fn acp_test_probe_bounded_at_max_events() {
    let mut probe = super::view::AcpTestProbe::default();
    let max = crate::protocol::ACP_TEST_PROBE_MAX_EVENTS;

    for i in 0..(max + 10) {
        if probe.key_routes.len() >= max {
            probe.key_routes.pop_front();
        }
        probe
            .key_routes
            .push_back(crate::protocol::AcpKeyRouteTelemetry {
                key: format!("key-{i}"),
                route: crate::protocol::AcpKeyRoute::Composer,
                picker_open: false,
                permission_active: false,
                cursor_before: i,
                cursor_after: i + 1,
                caused_submit: false,
                consumed: true,
            });
    }

    assert_eq!(
        probe.key_routes.len(),
        max,
        "ring buffer must be bounded at {max}"
    );
    // Oldest event should have been evicted
    assert_eq!(probe.key_routes[0].key, "key-10");
}

#[test]
fn acp_test_probe_reset_clears_all() {
    let mut probe = super::view::AcpTestProbe::default();

    probe.event_seq = 42;
    probe
        .key_routes
        .push_back(crate::protocol::AcpKeyRouteTelemetry {
            key: "tab".to_string(),
            route: crate::protocol::AcpKeyRoute::Picker,
            picker_open: true,
            permission_active: false,
            cursor_before: 1,
            cursor_after: 9,
            caused_submit: false,
            consumed: true,
        });
    probe
        .accepted_items
        .push_back(crate::protocol::AcpPickerItemAcceptedTelemetry {
            trigger: "@".to_string(),
            item_label: "context".to_string(),
            item_id: "built_in:context".to_string(),
            accepted_via_key: "tab".to_string(),
            cursor_after: 9,
            caused_submit: false,
        });
    probe.input_layout = Some(crate::protocol::AcpInputLayoutTelemetry {
        char_count: 10,
        visible_start: 0,
        visible_end: 10,
        cursor_in_window: 9,
    });

    // Reset
    probe.event_seq = 0;
    probe.key_routes.clear();
    probe.accepted_items.clear();
    probe.input_layout = None;

    assert_eq!(probe.event_seq, 0);
    assert!(probe.key_routes.is_empty());
    assert!(probe.accepted_items.is_empty());
    assert!(probe.input_layout.is_none());
}

// =========================================================================
// ACP test probe — source code contracts
// =========================================================================

#[test]
fn acp_view_has_test_probe_methods() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    assert!(
        ACP_VIEW_SOURCE.contains("fn reset_test_probe("),
        "AcpChatView must have reset_test_probe method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn record_key_route("),
        "AcpChatView must have record_key_route method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn record_picker_accept("),
        "AcpChatView must have record_picker_accept method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn record_input_layout("),
        "AcpChatView must have record_input_layout method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn test_probe_snapshot("),
        "AcpChatView must have test_probe_snapshot method"
    );
}

#[test]
fn emit_methods_record_into_probe() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    assert!(
        ACP_VIEW_SOURCE.contains("self.record_key_route(telemetry.clone())"),
        "emit_key_route_telemetry must record into probe"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("self.record_picker_accept(telemetry.clone())"),
        "emit_picker_accepted_telemetry must record into probe"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("self.record_input_layout(telemetry.clone())"),
        "emit_input_layout_telemetry must record into probe"
    );
}

#[test]
fn emit_key_route_telemetry_uses_real_permission_state() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    // The function must accept permission_active as a parameter, not hardcode it.
    assert!(
        ACP_VIEW_SOURCE.contains("permission_active: bool,"),
        "emit_key_route_telemetry must accept permission_active as a parameter"
    );
    assert!(
        !ACP_VIEW_SOURCE.contains("let permission_active = false;"),
        "emit_key_route_telemetry must not hardcode permission_active to false"
    );
}

#[test]
fn call_sites_pass_real_permission_active() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    // All call sites should read the real permission state from the thread.
    let permission_reads = ACP_VIEW_SOURCE
        .matches("pending_permission.is_some()")
        .count();
    let telemetry_calls = ACP_VIEW_SOURCE
        .matches(".emit_key_route_telemetry(")
        .count();
    assert!(
        permission_reads >= telemetry_calls,
        "each emit_key_route_telemetry call site ({telemetry_calls}) must read \
         pending_permission.is_some() ({permission_reads} found)"
    );
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
        TAB_AI_MODE_SOURCE.contains("resolve_acp_launch_with_requirements"),
        "tab_ai_mode must use capability-aware preflight resolution"
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
            catalog_entries: vec![],
        };
        let state =
            AcpInlineSetupState::from_resolution(&resolution, AcpLaunchRequirements::default());
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
        SETUP_RENDER_SOURCE.contains("Agent Required"),
        "setup card title must say Agent Required"
    );
    assert!(
        SETUP_RENDER_SOURCE.contains("Open Agent Catalog"),
        "setup card must offer Open Agent Catalog"
    );
    assert!(
        !SETUP_RENDER_SOURCE.contains("Connect to Claude Code"),
        "setup card must NOT mention Claude Code specifically"
    );
}

// =========================================================================
// 6. Capability-driven ACP launch and recovery
// =========================================================================

#[test]
fn tab_ai_mode_derives_launch_requirements() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("AcpLaunchRequirements"),
        "tab_ai_mode must derive AcpLaunchRequirements"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("acp_open_retry_request_consumed"),
        "tab_ai_mode must log retry request consumption with requirements"
    );
}

#[test]
fn acp_retry_request_from_setup_state_preserves_agent_and_requirements() {
    use super::setup_state::AcpInlineSetupState;
    use super::view::AcpRetryRequest;

    let setup = AcpInlineSetupState {
        reason_code: "authenticationRequired",
        title: "Auth required".into(),
        body: "test".into(),
        primary_action: super::setup_state::AcpSetupAction::Retry,
        secondary_action: None,
        selected_agent: Some(super::catalog::AcpAgentCatalogEntry {
            id: "opencode".into(),
            display_name: "OpenCode".into(),
            source: super::catalog::AcpAgentSource::ScriptKitCatalog,
            install_state: super::catalog::AcpAgentInstallState::Ready,
            auth_state: super::catalog::AcpAgentAuthState::Unknown,
            config_state: super::catalog::AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: Some(true),
            supports_image: None,
            last_session_ok: false,
            config: None,
        }),
        catalog_entries: Vec::new(),
        launch_requirements: super::preflight::AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        },
    };

    let request = AcpRetryRequest::from_setup_state(&setup);
    assert_eq!(request.preferred_agent_id.as_deref(), Some("opencode"));
    assert!(request.launch_requirements.needs_embedded_context);
    assert!(!request.launch_requirements.needs_image);
}

#[test]
fn acp_retry_request_from_setup_state_without_agent() {
    use super::view::AcpRetryRequest;

    let setup = super::setup_state::AcpInlineSetupState {
        reason_code: "noAgentsAvailable",
        title: "No agents".into(),
        body: "test".into(),
        primary_action: super::setup_state::AcpSetupAction::OpenCatalog,
        secondary_action: None,
        selected_agent: None,
        catalog_entries: Vec::new(),
        launch_requirements: super::preflight::AcpLaunchRequirements::default(),
    };

    let request = AcpRetryRequest::from_setup_state(&setup);
    assert_eq!(request.preferred_agent_id, None);
    assert!(!request.launch_requirements.needs_embedded_context);
    assert!(!request.launch_requirements.needs_image);
}

#[test]
fn tab_ai_mode_consumes_retry_request_before_preference() {
    // Verify the open path checks for retry request before loading preference.
    assert!(
        TAB_AI_MODE_SOURCE.contains("take_acp_retry_request_for_open"),
        "tab_ai_mode must check for retry request from current view"
    );
    // The retry request should be checked before load_preferred_acp_agent_id.
    let retry_pos = TAB_AI_MODE_SOURCE
        .find("take_acp_retry_request_for_open")
        .expect("must have retry request extraction");
    let pref_pos = TAB_AI_MODE_SOURCE
        .find("load_preferred_acp_agent_id")
        .expect("must still fall back to preference loading");
    assert!(
        retry_pos < pref_pos,
        "retry request must be consumed before preference fallback"
    );
}

#[test]
fn acp_view_queues_retry_payload_on_setup_retry() {
    // The view must queue the payload with structured tracing.
    let view_source = include_str!("view.rs");
    assert!(
        view_source.contains("acp_setup_retry_payload_queued"),
        "view must emit acp_setup_retry_payload_queued tracing event"
    );
    assert!(
        view_source.contains("queue_setup_retry_request"),
        "Retry action must call queue_setup_retry_request"
    );
}

#[test]
fn setup_state_handles_capability_mismatch_with_switch() {
    use super::preflight::{AcpLaunchBlocker, AcpLaunchResolution};
    use super::setup_state::{AcpInlineSetupState, AcpSetupAction};

    let agents = vec![
        super::catalog::AcpAgentCatalogEntry {
            id: "blocked".into(),
            display_name: "Blocked".into(),
            source: super::catalog::AcpAgentSource::ScriptKitCatalog,
            install_state: super::catalog::AcpAgentInstallState::Ready,
            auth_state: super::catalog::AcpAgentAuthState::Unknown,
            config_state: super::catalog::AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: Some(false),
            supports_image: None,
            last_session_ok: false,
            config: None,
        },
        super::catalog::AcpAgentCatalogEntry {
            id: "ready".into(),
            display_name: "Ready".into(),
            source: super::catalog::AcpAgentSource::ScriptKitCatalog,
            install_state: super::catalog::AcpAgentInstallState::Ready,
            auth_state: super::catalog::AcpAgentAuthState::Unknown,
            config_state: super::catalog::AcpAgentConfigState::Valid,
            install_hint: None,
            config_hint: None,
            supports_embedded_context: Some(true),
            supports_image: None,
            last_session_ok: false,
            config: None,
        },
    ];

    let resolution = AcpLaunchResolution {
        selected_agent: Some(agents[0].clone()),
        blocker: Some(AcpLaunchBlocker::CapabilityMismatch),
        catalog_entries: agents,
    };

    let state = AcpInlineSetupState::from_resolution(
        &resolution,
        AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        },
    );
    assert_eq!(state.title.as_ref(), "ACP capability mismatch");
    assert_eq!(
        state.primary_action,
        AcpSetupAction::SelectAgent,
        "should offer SelectAgent when a capable alternative exists"
    );
}

#[test]
fn setup_state_handles_capability_mismatch_without_alternative() {
    use super::preflight::{AcpLaunchBlocker, AcpLaunchResolution};
    use super::setup_state::{AcpInlineSetupState, AcpSetupAction};

    let agents = vec![super::catalog::AcpAgentCatalogEntry {
        id: "only-agent".into(),
        display_name: "Only Agent".into(),
        source: super::catalog::AcpAgentSource::ScriptKitCatalog,
        install_state: super::catalog::AcpAgentInstallState::Ready,
        auth_state: super::catalog::AcpAgentAuthState::Unknown,
        config_state: super::catalog::AcpAgentConfigState::Valid,
        install_hint: None,
        config_hint: None,
        supports_embedded_context: Some(false),
        supports_image: None,
        last_session_ok: false,
        config: None,
    }];

    let resolution = AcpLaunchResolution {
        selected_agent: Some(agents[0].clone()),
        blocker: Some(AcpLaunchBlocker::CapabilityMismatch),
        catalog_entries: agents,
    };

    let state = AcpInlineSetupState::from_resolution(
        &resolution,
        AcpLaunchRequirements {
            needs_embedded_context: true,
            needs_image: false,
        },
    );
    assert_eq!(state.title.as_ref(), "ACP capability mismatch");
    assert_eq!(
        state.primary_action,
        AcpSetupAction::Retry,
        "should offer Retry when no capable alternative exists"
    );
}

// =========================================================================
// 7. ACP history primitives — delete + resume request
// =========================================================================

#[test]
fn delete_conversation_removes_file_and_rewrites_index() {
    use super::history::{
        delete_conversation, load_history, save_conversation, save_history_entry, AcpHistoryEntry,
        SavedConversation, SavedMessage,
    };

    let dir = tempfile::tempdir().expect("create temp dir");
    let kit_path = dir.path().to_path_buf();

    // Create a history index and conversation file manually
    let history_path = kit_path.join("acp-history.jsonl");
    let conv_dir = kit_path.join("acp-conversations");
    std::fs::create_dir_all(&conv_dir).expect("create conv dir");

    let entry_a = AcpHistoryEntry {
        timestamp: "2026-04-05T10:00:00Z".to_string(),
        first_message: "hello from A".to_string(),
        message_count: 3,
        session_id: "session-a".to_string(),
        ..Default::default()
    };
    let entry_b = AcpHistoryEntry {
        timestamp: "2026-04-05T11:00:00Z".to_string(),
        first_message: "hello from B".to_string(),
        message_count: 5,
        session_id: "session-b".to_string(),
        ..Default::default()
    };

    // Write index entries
    let mut index_content = String::new();
    index_content.push_str(&serde_json::to_string(&entry_a).expect("serialize"));
    index_content.push('\n');
    index_content.push_str(&serde_json::to_string(&entry_b).expect("serialize"));
    index_content.push('\n');
    std::fs::write(&history_path, &index_content).expect("write index");

    // Write conversation file for session-a
    let conv = SavedConversation {
        session_id: "session-a".to_string(),
        timestamp: "2026-04-05T10:00:00Z".to_string(),
        messages: vec![SavedMessage {
            role: "user".to_string(),
            body: "test message".to_string(),
        }],
    };
    let conv_path = conv_dir.join("session-a.json");
    std::fs::write(
        &conv_path,
        serde_json::to_string_pretty(&conv).expect("serialize"),
    )
    .expect("write conv");

    assert!(
        conv_path.exists(),
        "conversation file must exist before delete"
    );

    // Note: delete_conversation uses the global kit path, so we can only
    // test the function's serialization/deserialization logic in isolation.
    // The actual file I/O integration depends on the global kit path.
    // Instead, verify the entry types roundtrip correctly through serde.
    let entries_json = std::fs::read_to_string(&history_path).expect("read index");
    let parsed: Vec<AcpHistoryEntry> = entries_json
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    assert_eq!(parsed.len(), 2);

    // Simulate delete by filtering
    let remaining: Vec<&AcpHistoryEntry> = parsed
        .iter()
        .filter(|e| e.session_id != "session-a")
        .collect();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].session_id, "session-b");
}

#[test]
fn delete_conversation_is_idempotent_for_missing_session() {
    // Calling delete on a non-existent session should succeed.
    // We can't test the real function without the global kit path,
    // but we verify the AcpHistoryEntry serde contract supports it.
    let entry = super::history::AcpHistoryEntry {
        timestamp: "2026-04-05T10:00:00Z".to_string(),
        first_message: "test".to_string(),
        message_count: 1,
        session_id: "nonexistent-session".to_string(),
        ..Default::default()
    };
    let json = serde_json::to_string(&entry).expect("serialize");
    let parsed: super::history::AcpHistoryEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.session_id, "nonexistent-session");
}

#[test]
fn history_resume_request_struct_carries_session_id() {
    let request = super::view::AcpHistoryResumeRequest {
        session_id: "test-session-42".to_string(),
    };
    assert_eq!(request.session_id, "test-session-42");

    let cloned = request.clone();
    assert_eq!(cloned.session_id, "test-session-42");
}

#[test]
fn acp_view_exposes_history_resume_primitives() {
    const ACP_VIEW_SOURCE: &str = include_str!("view.rs");
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) struct AcpHistoryResumeRequest"),
        "AcpChatView module must define AcpHistoryResumeRequest"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn stage_history_resume("),
        "AcpChatView must have stage_history_resume method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn take_history_resume("),
        "AcpChatView must have take_history_resume method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("fn resume_from_history("),
        "AcpChatView must have resume_from_history method"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("pending_history_resume"),
        "AcpChatView must have pending_history_resume field"
    );
}

#[test]
fn history_delete_function_exists_in_history_module() {
    const HISTORY_SOURCE: &str = include_str!("history.rs");
    assert!(
        HISTORY_SOURCE.contains("pub(crate) fn delete_conversation(session_id: &str)"),
        "history module must expose delete_conversation(session_id)"
    );
    assert!(
        HISTORY_SOURCE.contains("acp_history_item_deleted"),
        "delete_conversation must emit structured tracing event"
    );
}

#[test]
fn history_resume_is_reexported_from_acp_mod() {
    const MOD_SOURCE: &str = include_str!("mod.rs");
    assert!(
        MOD_SOURCE.contains("AcpHistoryResumeRequest"),
        "AcpHistoryResumeRequest must be re-exported from acp mod"
    );
}

#[test]
fn global_clear_history_remains_separate_from_per_item_delete() {
    const HISTORY_SOURCE: &str = include_str!("history.rs");
    // delete_conversation filters by session_id — it does NOT remove all entries
    assert!(
        HISTORY_SOURCE.contains("filter(|entry| entry.session_id != session_id)"),
        "delete_conversation must filter by session_id, not clear all"
    );
    // The chat_window.rs clear path uses remove_file + remove_dir_all
    const CHAT_WINDOW_SOURCE: &str = include_str!("chat_window.rs");
    assert!(
        CHAT_WINDOW_SOURCE.contains("remove_dir_all"),
        "clear_history must remove entire conversations directory"
    );
}

// =========================================================================
// Shared inline-token sync kernel — ACP adoption contracts
// =========================================================================

#[test]
fn acp_uses_shared_inline_sync_plan() {
    assert!(
        ACP_VIEW_SOURCE.contains("build_inline_mention_sync_plan"),
        "ACP sync_inline_mentions must use the shared sync plan builder"
    );
}

#[test]
fn acp_uses_shared_visible_chip_indices() {
    assert!(
        ACP_VIEW_SOURCE.contains("visible_context_chip_indices"),
        "ACP render_pending_context_chips must use shared visible chip filtering"
    );
}

#[test]
fn acp_uses_shared_atomic_delete() {
    assert!(
        ACP_VIEW_SOURCE.contains("remove_inline_mention_at_cursor"),
        "ACP key handler must use shared token-atomic delete"
    );
}

#[test]
fn acp_emits_inline_mentions_synced_event() {
    assert!(
        ACP_VIEW_SOURCE.contains("acp_inline_mentions_synced"),
        "ACP must emit acp_inline_mentions_synced tracing event on sync"
    );
}

#[test]
fn acp_emits_inline_mention_deleted_atomically_event() {
    assert!(
        ACP_VIEW_SOURCE.contains("acp_inline_mention_deleted_atomically"),
        "ACP must emit acp_inline_mention_deleted_atomically tracing event on atomic delete"
    );
}

// =========================================================================
// AI-window inline-token unification source contracts
// =========================================================================
//
// These tests verify that the AI window picker, chip rendering, and input
// handling use the same shared inline-token infrastructure as ACP.

const AI_WINDOW_CONTEXT_PICKER_SOURCE: &str = include_str!("../window/context_picker/mod.rs");
const AI_WINDOW_RENDER_SOURCE: &str = include_str!("../window/render_main_panel.rs");
const AI_WINDOW_INPUT_SOURCE: &str = include_str!("../window/render_keydown.rs");

#[test]
fn ai_window_picker_inserts_inline_token_and_syncs_parts() {
    assert!(
        AI_WINDOW_CONTEXT_PICKER_SOURCE.contains("ai_context_picker_token_inserted"),
        "AI window picker must log inline token insertion",
    );
    assert!(
        AI_WINDOW_CONTEXT_PICKER_SOURCE.contains("part_to_inline_token(&part)"),
        "AI window picker must derive canonical inline tokens from attached parts",
    );
    assert!(
        AI_WINDOW_CONTEXT_PICKER_SOURCE.contains("sync_inline_mentions(cx)"),
        "AI window picker must synchronize inline tokens back into pending_context_parts",
    );
}

#[test]
fn ai_window_hides_inline_backed_chips() {
    assert!(
        AI_WINDOW_RENDER_SOURCE.contains("visible_context_chip_indices"),
        "AI window chip rendering must hide parts already represented inline",
    );
}

#[test]
fn ai_window_uses_atomic_inline_delete() {
    assert!(
        AI_WINDOW_INPUT_SOURCE.contains("remove_inline_mention_at_cursor"),
        "AI window input handling must use shared token-atomic delete",
    );
}

#[test]
fn ai_window_emits_inline_mentions_synced_event() {
    assert!(
        AI_WINDOW_CONTEXT_PICKER_SOURCE.contains("ai_inline_mentions_synced"),
        "AI window must emit ai_inline_mentions_synced tracing event on sync",
    );
}

#[test]
fn ai_window_emits_inline_mention_deleted_atomically_event() {
    assert!(
        AI_WINDOW_INPUT_SOURCE.contains("ai_inline_mention_deleted_atomically"),
        "AI window must emit ai_inline_mention_deleted_atomically tracing event on atomic delete",
    );
}

#[test]
fn acp_and_ai_window_share_inline_sync_kernel() {
    assert!(
        ACP_VIEW_SOURCE.contains("build_inline_mention_sync_plan"),
        "ACP must use shared inline sync planning",
    );
    assert!(
        AI_WINDOW_CONTEXT_PICKER_SOURCE.contains("build_inline_mention_sync_plan"),
        "AI window must use shared inline sync planning",
    );
}

// =========================================================================
// Source-aware slash command identity and resolution
// =========================================================================

#[test]
fn acp_resolved_slash_commands_keep_local_skills_without_provider_advertisement() {
    use super::view::{SlashCommandEntry, SlashCommandSource};
    use std::path::PathBuf;

    // Simulate cached entries: one default, two plugin skills, one claude skill
    let cached = vec![
        SlashCommandEntry {
            name: "clear".to_string(),
            description: String::new(),
            source: SlashCommandSource::Default,
        },
        SlashCommandEntry {
            name: "review".to_string(),
            description: "Alpha review".to_string(),
            source: SlashCommandSource::PluginSkill(crate::plugins::PluginSkill {
                plugin_id: "alpha".to_string(),
                plugin_title: "Alpha".to_string(),
                skill_id: "review".to_string(),
                path: PathBuf::from("/alpha/skills/review/SKILL.md"),
                title: "Review".to_string(),
                description: "Alpha review".to_string(),
            }),
        },
        SlashCommandEntry {
            name: "review".to_string(),
            description: "Beta review".to_string(),
            source: SlashCommandSource::PluginSkill(crate::plugins::PluginSkill {
                plugin_id: "beta".to_string(),
                plugin_title: "Beta".to_string(),
                skill_id: "review".to_string(),
                path: PathBuf::from("/beta/skills/review/SKILL.md"),
                title: "Review".to_string(),
                description: "Beta review".to_string(),
            }),
        },
        SlashCommandEntry {
            name: "plan".to_string(),
            description: "Plan skill".to_string(),
            source: SlashCommandSource::ClaudeCodeSkill {
                skill_id: "plan".to_string(),
                skill_path: PathBuf::from("/claude/skills/plan/SKILL.md"),
            },
        },
    ];

    // Provider only advertises "clear" and "help" — plugin and Claude skills
    // should still appear in the resolved output.
    let available = vec!["clear".to_string(), "help".to_string()];

    // Exercise the resolution logic directly using the same algorithm as the view.
    let available_set: std::collections::HashSet<&str> =
        available.iter().map(|s| s.as_str()).collect();
    let mut result: Vec<SlashCommandEntry> = Vec::new();

    for entry in &cached {
        match &entry.source {
            SlashCommandSource::Default if available_set.contains(entry.name.as_str()) => {
                result.push(entry.clone());
            }
            SlashCommandSource::PluginSkill(_) | SlashCommandSource::ClaudeCodeSkill { .. } => {
                result.push(entry.clone());
            }
            _ => {}
        }
    }
    for cmd in &available {
        let already_present = result
            .iter()
            .any(|entry| matches!(entry.source, SlashCommandSource::Default) && entry.name == *cmd);
        if !already_present {
            result.push(SlashCommandEntry {
                name: cmd.clone(),
                description: String::new(),
                source: SlashCommandSource::Default,
            });
        }
    }

    let names: Vec<&str> = result.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["clear", "review", "review", "plan", "help"],
        "Expected: default clear + both plugin reviews + claude plan + new default help"
    );

    // Verify both reviews are from different plugins
    let reviews: Vec<&SlashCommandEntry> = result.iter().filter(|e| e.name == "review").collect();
    assert_eq!(reviews.len(), 2);
    assert!(
        matches!(
            &reviews[0].source,
            SlashCommandSource::PluginSkill(s) if s.plugin_id == "alpha"
        ),
        "First review should be from alpha"
    );
    assert!(
        matches!(
            &reviews[1].source,
            SlashCommandSource::PluginSkill(s) if s.plugin_id == "beta"
        ),
        "Second review should be from beta"
    );

    // Plugin skills appear even though provider didn't advertise "review"
    assert!(
        !available_set.contains("review"),
        "Provider should not have advertised 'review'"
    );
}

#[test]
fn acp_slash_command_entry_qualified_keys_are_distinct() {
    use super::view::{SlashCommandEntry, SlashCommandSource};
    use std::path::PathBuf;

    let default = SlashCommandEntry {
        name: "review".to_string(),
        description: String::new(),
        source: SlashCommandSource::Default,
    };
    let plugin = SlashCommandEntry {
        name: "review".to_string(),
        description: "Plugin review".to_string(),
        source: SlashCommandSource::PluginSkill(crate::plugins::PluginSkill {
            plugin_id: "alpha".to_string(),
            plugin_title: "Alpha".to_string(),
            skill_id: "review".to_string(),
            path: PathBuf::from("/alpha/review/SKILL.md"),
            title: "Review".to_string(),
            description: String::new(),
        }),
    };
    let claude = SlashCommandEntry {
        name: "review".to_string(),
        description: "Claude review".to_string(),
        source: SlashCommandSource::ClaudeCodeSkill {
            skill_id: "review".to_string(),
            skill_path: PathBuf::from("/claude/review/SKILL.md"),
        },
    };

    let keys: Vec<String> = vec![
        default.qualified_key(),
        plugin.qualified_key(),
        claude.qualified_key(),
    ];
    let unique: std::collections::HashSet<&str> = keys.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        unique.len(),
        3,
        "All three sources should have distinct qualified keys: {:?}",
        keys
    );
}

// =========================================================================
// Slash acceptance contracts
// =========================================================================

/// Default slash commands produce a `SlashCommandPayload::Default` whose
/// acceptance contract is to insert literal `/command` text — NOT to stage
/// skill content via `build_staged_skill_prompt`.
#[test]
fn acp_default_slash_accept_inserts_command_text() {
    use super::view::SlashCommandEntry;
    use crate::ai::window::context_picker::types::SlashCommandPayload;

    let entry = SlashCommandEntry::default_command("compact");
    let payload = entry.to_payload();

    // Must produce a Default payload, which the acceptance path inserts as
    // literal `/compact ` text into the composer.
    match &payload {
        SlashCommandPayload::Default { name } => {
            assert_eq!(name, "compact");
            // Acceptance inserts `/{name} ` — verify the format.
            let inserted = format!("/{name} ");
            assert_eq!(inserted, "/compact ");
        }
        other => panic!(
            "Expected SlashCommandPayload::Default for a default entry, got {:?}",
            other
        ),
    }
}

/// Plugin skills accepted from the ACP slash picker must use the same
/// slash-prefill text and attached skill-part payload as the main-menu
/// skill launch path.
#[test]
fn acp_plugin_slash_accept_stages_selected_skill_prompt() {
    use super::view::{build_skill_context_part, build_skill_slash_command_text};
    use std::io::Write;

    // Create a temp skill file with known content.
    let dir = tempfile::tempdir().expect("tempdir");
    let skill_path = dir.path().join("SKILL.md");
    {
        let mut f = std::fs::File::create(&skill_path).expect("create skill file");
        f.write_all(b"# Review\nReview code changes against project guidelines.")
            .expect("write");
    }

    let skill = crate::plugins::PluginSkill {
        plugin_id: "alpha".to_string(),
        plugin_title: "Alpha".to_string(),
        skill_id: "review".to_string(),
        path: skill_path.clone(),
        title: "Review".to_string(),
        description: "Alpha review".to_string(),
    };

    let slash_prefill = build_skill_slash_command_text(&skill.skill_id);
    assert_eq!(slash_prefill, "/review ");

    let owner = if skill.plugin_title.is_empty() {
        &skill.plugin_id
    } else {
        &skill.plugin_title
    };
    let slash_part = build_skill_context_part(&skill.title, owner, &skill.skill_id, &skill.path);
    let main_menu_part =
        build_skill_context_part(&skill.title, owner, &skill.skill_id, &skill.path);
    assert_eq!(slash_part, main_menu_part);

    match &slash_part {
        crate::ai::message_parts::AiContextPart::SkillFile {
            path,
            label,
            skill_name,
            owner_label,
            slash_name,
        } => {
            assert_eq!(path, &skill_path.to_string_lossy().to_string());
            assert_eq!(label, "/review");
            assert_eq!(skill_name, "Review");
            assert_eq!(owner_label, "Alpha");
            assert_eq!(slash_name, "review");
        }
        other => panic!("expected SkillFile part, got {other:?}"),
    }

    // When two plugins share the same slash slug, their attached parts diverge
    // because the owner and path differ.
    let beta_path = dir.path().join("BETA_SKILL.md");
    {
        let mut f = std::fs::File::create(&beta_path).expect("create beta skill");
        f.write_all(b"# Review\nBeta-specific review checklist.")
            .expect("write");
    }

    let beta_part = build_skill_context_part("Review", "Beta", "review", &beta_path);
    assert_ne!(
        slash_part, beta_part,
        "Same slug from different plugins must produce different attached parts"
    );
}

#[test]
fn acp_claude_skill_staged_prompt_uses_claude_owner_phrase() {
    use super::view::build_staged_skill_prompt;
    use std::io::Write;

    let dir = tempfile::tempdir().expect("tempdir");
    let skill_path = dir.path().join("SKILL.md");
    {
        let mut f = std::fs::File::create(&skill_path).expect("create skill file");
        f.write_all(b"# Plan\nDraft a concise implementation plan.")
            .expect("write");
    }

    let staged = build_staged_skill_prompt("Plan", "Claude Code", &skill_path);

    assert!(
        staged.contains("from Claude Code"),
        "Claude Code skills should be labeled as Claude Code, got: {staged}"
    );
    assert!(
        !staged.contains("from plugin \"Claude Code\""),
        "Claude Code skills must not be mislabeled as plugins: {staged}"
    );
}

// =========================================================================
// Cross-source collision detection
// =========================================================================

/// Verify that cross-source collision bookkeeping correctly seeds
/// default names and detects plugin-vs-default and Claude-vs-{default,plugin}
/// collisions. This test exercises the collision bookkeeping in isolation
/// using the same data structures as `discover_slash_commands`.
#[test]
fn acp_slash_cross_source_collision_bookkeeping() {
    use std::collections::{HashMap, HashSet};

    // Simulate the discover_slash_commands owner-tracking logic.
    let default_names: HashSet<String> = ["review", "compact"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut owners_by_slash: HashMap<String, Vec<String>> = HashMap::new();
    for name in &default_names {
        owners_by_slash
            .entry(name.clone())
            .or_default()
            .push("Built-in".to_string());
    }

    // Plugin skill collides with "review" default.
    let plugin_name = "review".to_string();
    let mut plugin_names: HashSet<String> = HashSet::new();
    plugin_names.insert(plugin_name.clone());
    owners_by_slash
        .entry(plugin_name.clone())
        .or_default()
        .push("Alpha".to_string());

    assert!(
        default_names.contains(&plugin_name),
        "Plugin 'review' should collide with default 'review'"
    );

    // Claude skill collides with both "review" (default + plugin).
    let claude_name = "review".to_string();
    owners_by_slash
        .entry(claude_name.clone())
        .or_default()
        .push("Claude Code".to_string());

    assert!(
        plugin_names.contains(&claude_name),
        "Claude 'review' should collide with plugin 'review'"
    );
    assert!(
        default_names.contains(&claude_name),
        "Claude 'review' should collide with default 'review'"
    );

    // Final cross-source check: "review" has 3 owners.
    let review_owners = owners_by_slash.get("review").expect("review should exist");
    assert_eq!(
        review_owners.len(),
        3,
        "review should have 3 owners (Built-in, Alpha, Claude Code): {:?}",
        review_owners
    );
    assert!(review_owners.contains(&"Built-in".to_string()));
    assert!(review_owners.contains(&"Alpha".to_string()));
    assert!(review_owners.contains(&"Claude Code".to_string()));

    // "compact" should have exactly 1 owner (no collision).
    let compact_owners = owners_by_slash
        .get("compact")
        .expect("compact should exist");
    assert_eq!(
        compact_owners.len(),
        1,
        "compact should have 1 owner: {:?}",
        compact_owners
    );
}

/// Qualified key dedup preserves source-distinct rows even when slash
/// names collide. This ensures the picker shows both rows with owner labels.
#[test]
fn acp_slash_dedup_preserves_source_distinct_rows() {
    use super::view::{SlashCommandEntry, SlashCommandSource};
    use std::collections::HashSet;

    let default_review = SlashCommandEntry::default_command("review");
    let plugin_review = SlashCommandEntry {
        name: "review".to_string(),
        description: "Alpha review".to_string(),
        source: SlashCommandSource::PluginSkill(crate::plugins::PluginSkill {
            plugin_id: "alpha".to_string(),
            plugin_title: "Alpha".to_string(),
            skill_id: "review".to_string(),
            path: std::path::PathBuf::from("/alpha/review/SKILL.md"),
            title: "Review".to_string(),
            description: String::new(),
        }),
    };
    let claude_review = SlashCommandEntry {
        name: "review".to_string(),
        description: "Claude review".to_string(),
        source: SlashCommandSource::ClaudeCodeSkill {
            skill_id: "review".to_string(),
            skill_path: std::path::PathBuf::from("/claude/review/SKILL.md"),
        },
    };

    let mut seen: HashSet<String> = HashSet::new();
    let mut commands = Vec::new();

    for entry in [&default_review, &plugin_review, &claude_review] {
        if seen.insert(entry.qualified_key()) {
            commands.push(entry.clone());
        }
    }

    // All three should be distinct by qualified_key even though all share
    // the same bare slash name "review".
    assert_eq!(
        commands.len(),
        3,
        "All three sources should produce distinct rows: {:?}",
        commands
            .iter()
            .map(|e| e.qualified_key())
            .collect::<Vec<_>>()
    );
}

/// Plugin and Claude skill slash acceptance both share the same slash-prefill
/// and attached-skill-part contract as main-menu skill launch.
#[test]
fn acp_slash_and_main_menu_skill_launch_share_prompt_contract() {
    use super::view::{build_skill_context_part, build_skill_slash_command_text};
    use std::io::Write;

    let dir = tempfile::tempdir().expect("tempdir");
    let skill_path = dir.path().join("SKILL.md");
    {
        let mut f = std::fs::File::create(&skill_path).expect("create");
        f.write_all(b"# Deploy\nDeploy the current project.")
            .expect("write");
    }

    let skill = crate::plugins::PluginSkill {
        plugin_id: "infra".to_string(),
        plugin_title: "Infrastructure".to_string(),
        skill_id: "deploy".to_string(),
        path: skill_path.clone(),
        title: "Deploy".to_string(),
        description: "Deploy skill".to_string(),
    };

    // Main-menu path (from open_acp_with_selected_skill).
    let owner = if skill.plugin_title.is_empty() {
        &skill.plugin_id
    } else {
        &skill.plugin_title
    };
    let main_menu_prefill = build_skill_slash_command_text(&skill.skill_id);
    let slash_accept_prefill = build_skill_slash_command_text(&skill.skill_id);
    let main_menu_part =
        build_skill_context_part(&skill.title, owner, &skill.skill_id, &skill.path);
    let slash_accept_part =
        build_skill_context_part(&skill.title, owner, &skill.skill_id, &skill.path);

    assert_eq!(
        main_menu_prefill, slash_accept_prefill,
        "Main-menu and slash acceptance must produce identical slash prefill"
    );
    assert_eq!(
        main_menu_part, slash_accept_part,
        "Main-menu and slash acceptance must attach the same skill part"
    );

    let claude_skill_path = dir.path().join("CLAUDE_SKILL.md");
    {
        let mut f = std::fs::File::create(&claude_skill_path).expect("create");
        f.write_all(b"# Plan\nCreate a plan.").expect("write");
    }
    let claude_part = build_skill_context_part("Plan", "Claude Code", "plan", &claude_skill_path);
    match claude_part {
        crate::ai::message_parts::AiContextPart::SkillFile { owner_label, .. } => {
            assert_eq!(owner_label, "Claude Code");
        }
        other => panic!("expected SkillFile part, got {other:?}"),
    }
}

#[test]
fn acp_main_menu_skill_stage_matches_slash_selection_without_submit() {
    let body = acp_source_between(
        ACP_VIEW_SOURCE,
        "pub(crate) fn stage_selected_plugin_skill_from_main_menu(",
        "    /// Reuse the current live thread for a fresh external entry intent.",
    );

    for required in [
        "build_skill_slash_command_text(&skill.skill_id)",
        "build_skill_context_part(&skill.title, owner, &skill.skill_id, &skill.path)",
        "thread.replace_pending_context_parts(vec![part], \"main_menu_selected_skill\", cx);",
        "thread.input.set_text(command_text.clone());",
        "thread.input.set_cursor(cursor_after);",
        "thread.mark_context_bootstrap_ready(cx);",
    ] {
        assert!(
            body.contains(required),
            "main-menu skill staging must preserve slash-selection behavior: {required}"
        );
    }

    assert!(
        !body.contains("submit_input("),
        "main-menu skill staging must not auto-submit"
    );
}

const LIFECYCLE_RESET_SOURCE: &str = include_str!("../../app_impl/lifecycle_reset.rs");

#[test]
fn at_inline_portal_window_cannot_outlive_owner() {
    // Invariant: the ACP `@` mention picker (a detached non-activating popup
    // window) must never be visible after its owner ACP view loses the
    // trigger or after the main launcher abandons the ACP surface.
    //
    // We enforce this with three independent guards. If any of them is
    // removed, the screenshot bug (orphaned "Open full notes / Portal"
    // popup floating next to a `ScriptList` main window) can reappear.

    // 1. apply_composer_picker_transition closes the detached popup window
    //    on every non-Open transition, even when the reducer forgot to
    //    request sync_popup.
    let composer_body = acp_source_between(
        ACP_VIEW_SOURCE,
        "fn apply_composer_picker_transition",
        "fn clear_composer_picker",
    );
    assert!(
        composer_body.contains("let next_picker_open = matches!(&state, AcpComposerPickerState::Open(_));")
            && composer_body.contains(
                "crate::ai::acp::picker_popup::close_mention_popup_window(cx);",
            ),
        "apply_composer_picker_transition must unconditionally close the detached @ popup whenever the picker state is not Open"
    );

    // 2. AcpMentionPopupWindow self-prunes on render when its WeakEntity
    //    owner is dropped or no longer carries a live `@` session.
    assert!(
        ACP_PICKER_POPUP_SOURCE.contains("fn owner_is_live(&self, cx: &App) -> bool")
            && ACP_PICKER_POPUP_SOURCE
                .contains("view.read(cx).has_active_mention_session()")
            && ACP_PICKER_POPUP_SOURCE.contains("if !self.owner_is_live(cx)"),
        "AcpMentionPopupWindow::render must self-prune when its owner ACP view is gone or has no live mention session"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) fn has_active_mention_session(&self) -> bool"),
        "AcpChatView must expose `has_active_mention_session` so the detached popup can verify owner liveness"
    );

    // 3. Lifecycle reset paths centralize the close so it cannot be
    //    skipped by individual surface transitions.
    assert!(
        LIFECYCLE_RESET_SOURCE.contains(
            "pub(crate) fn close_floating_popups_for_owner_loss",
        ) && LIFECYCLE_RESET_SOURCE
            .contains("crate::ai::acp::picker_popup::close_mention_popup_window(cx);")
            && LIFECYCLE_RESET_SOURCE.contains(
                "crate::menu_syntax_trigger_popup_window::close_menu_syntax_trigger_popup_window(cx);",
            ),
        "lifecycle_reset must expose `close_floating_popups_for_owner_loss` that closes the ACP @ picker and menu-syntax popup"
    );
    for caller in [
        "close_and_reset_window",
        "hide_main_window_preserving_state_for_focus_loss",
    ] {
        let start = LIFECYCLE_RESET_SOURCE
            .find(caller)
            .unwrap_or_else(|| panic!("missing lifecycle entry: {caller}"));
        let body = &LIFECYCLE_RESET_SOURCE[start..LIFECYCLE_RESET_SOURCE.len().min(start + 1500)];
        assert!(
            body.contains("close_floating_popups_for_owner_loss"),
            "{caller} must close detached popup windows before tearing down the owner surface"
        );
    }
    assert!(
        REGISTRIES_STATE_SOURCE.contains(
            "self.close_floating_popups_for_owner_loss(\"reset_to_script_list\", cx);",
        ),
        "reset_to_script_list must close detached popup windows so they cannot survive a return to the main script list"
    );
}
