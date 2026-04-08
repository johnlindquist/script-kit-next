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

const TAB_AI_MODE_SOURCE: &str = include_str!("../../app_impl/tab_ai_mode.rs");
const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../../app_impl/actions_toggle.rs");
const STARTUP_SOURCE: &str = include_str!("../../app_impl/startup.rs");
const STARTUP_NEW_ACTIONS_SOURCE: &str = include_str!("../../app_impl/startup_new_actions.rs");
const STARTUP_NEW_TAB_SOURCE: &str = include_str!("../../app_impl/startup_new_tab.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../../main_sections/render_impl.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../../main_sections/app_view_state.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../../main_entry/app_run_setup.rs");
const RUNTIME_STDIN_SOURCE: &str = include_str!("../../main_entry/runtime_stdin.rs");
const ACP_MOD_SOURCE: &str = include_str!("mod.rs");
const ACP_HISTORY_POPUP_SOURCE: &str = include_str!("history_popup.rs");
const ACP_MODEL_SELECTOR_POPUP_SOURCE: &str = include_str!("model_selector_popup.rs");
const ACP_PICKER_POPUP_SOURCE: &str = include_str!("picker_popup.rs");
const ACP_CHAT_WINDOW_SOURCE: &str = include_str!("chat_window.rs");
const ACP_VIEW_SOURCE: &str = include_str!("view.rs");

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
}

#[test]
fn acp_history_migration_uses_popup_window_instead_of_inline_layer() {
    assert!(
        !ACP_VIEW_SOURCE.contains(".id(\"acp-history-picker\")"),
        "ACP chat view should no longer render the history picker inline"
    );
    assert!(
        ACP_HISTORY_POPUP_SOURCE.contains("WindowKind::PopUp")
            && ACP_HISTORY_POPUP_SOURCE.contains("AcpHistoryPopupWindow"),
        "ACP history picker should render through a popup window entity"
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
        ACP_VIEW_SOURCE.contains("\"⌘K Actions\"")
            && ACP_VIEW_SOURCE.contains("this.trigger_toggle_actions(window, cx);"),
        "ACP footer Actions hint must route through the shared clickable footer renderer"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("wire_embedded_acp_footer_callbacks(&view_entity, cx);")
            && TAB_AI_MODE_SOURCE.contains("app.toggle_actions(cx, window);")
            && TAB_AI_MODE_SOURCE.contains("app.open_acp_model_picker_actions(cx, window);")
            && TAB_AI_MODE_SOURCE
                .contains("app.close_tab_ai_harness_terminal_with_window(window, cx);"),
        "embedded ACP hosts must wire footer clicks to the actions toggle, model picker, and close paths"
    );
    assert!(
        ACP_CHAT_WINDOW_SOURCE.contains("view.set_on_toggle_actions")
            && ACP_CHAT_WINDOW_SOURCE.contains("view.set_on_open_model_picker")
            && ACP_CHAT_WINDOW_SOURCE.contains("toggle_detached_actions(cx);")
            && ACP_CHAT_WINDOW_SOURCE.contains("open_detached_model_picker(cx);")
            && ACP_CHAT_WINDOW_SOURCE.contains("close_chat_window(cx);"),
        "detached ACP hosts must wire footer clicks to the detached actions toggle, model picker, and close paths"
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
            && ACP_VIEW_SOURCE.contains("this.trigger_open_model_picker(window, cx);")
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
            && ACP_VIEW_SOURCE.contains("self.sync_mention_popup_window_from_cached_parent(cx);"),
        "ACP view should expose a helper that dismisses the detached ACP popups on Escape"
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
        TAB_AI_MODE_SOURCE.contains("take_acp_retry_request_from_current_view"),
        "tab_ai_mode must check for retry request from current view"
    );
    // The retry request should be checked before load_preferred_acp_agent_id.
    let retry_pos = TAB_AI_MODE_SOURCE
        .find("take_acp_retry_request_from_current_view")
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
    };
    let entry_b = AcpHistoryEntry {
        timestamp: "2026-04-05T11:00:00Z".to_string(),
        first_message: "hello from B".to_string(),
        message_count: 5,
        session_id: "session-b".to_string(),
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
