//! Source-level contract tests for ACP agent switching from the chat actions menu.

const ACTIONS_TOGGLE_SOURCE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const ACTION_HANDLER_SOURCE: &str = include_str!("../src/app_actions/handle_action/mod.rs");
const ACTION_BUILDER_SOURCE: &str = include_str!("../src/actions/builders/script_context.rs");
const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/acp/chat_window.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_WINDOW_SOURCE: &str = include_str!("../src/actions/window.rs");

use std::path::{Path, PathBuf};

fn required_pos(source: &str, needle: &str, reason: &str) -> usize {
    source
        .find(needle)
        .unwrap_or_else(|| panic!("{reason}: missing `{needle}`"))
}
fn required_pos_after(source: &str, start: usize, needle: &str, reason: &str) -> usize {
    source[start..]
        .find(needle)
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("{reason}: missing `{needle}` after byte {start}"))
}
fn rust_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let sig_pos = required_pos(source, signature, "function signature must exist");
    let body_start = source[sig_pos..]
        .find('{')
        .map(|offset| sig_pos + offset)
        .unwrap_or_else(|| panic!("function body must exist for `{signature}`"));
    let mut depth = 0usize;
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &source[body_start..=body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body for `{signature}`");
}

fn rust_sources_under(relative_dir: &str) -> Vec<(PathBuf, String)> {
    fn visit_dir(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|err| {
                panic!("failed to read entry under {}: {err}", dir.display())
            });
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                let contents = std::fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
                out.push((path, contents));
            }
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_dir);
    let mut sources = Vec::new();
    visit_dir(&root, &mut sources);
    sources
}

fn source_tree_contains(sources: &[(PathBuf, String)], needle: &str) -> bool {
    sources.iter().any(|(_, source)| source.contains(needle))
}

fn rust_function_body_owned(source: &str, fn_name: &str) -> Option<String> {
    let marker = format!("fn {fn_name}");
    let start = source.find(&marker)?;
    let body_start = source[start..].find('{')? + start;
    let mut depth = 0usize;
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = body_start + offset + 1;
                    return Some(source[body_start..end].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn source_tree_function_body_named(
    sources: &[(PathBuf, String)],
    fn_name: &str,
) -> Option<(PathBuf, String)> {
    sources.iter().find_map(|(path, source)| {
        rust_function_body_owned(source, fn_name).map(|body| (path.clone(), body))
    })
}

fn source_tree_function_body_containing(
    sources: &[(PathBuf, String)],
    needle: &str,
) -> Option<(PathBuf, String)> {
    for (path, source) in sources {
        let mut search_start = 0usize;
        while let Some(fn_offset) = source[search_start..].find("fn ") {
            let fn_start = search_start + fn_offset;
            let body_start = match source[fn_start..].find('{') {
                Some(offset) => fn_start + offset,
                None => break,
            };
            let mut depth = 0usize;
            for (offset, ch) in source[body_start..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            let end = body_start + offset + 1;
                            let body = &source[fn_start..end];
                            if body.contains(needle) {
                                return Some((path.clone(), body.to_string()));
                            }
                            search_start = end;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if search_start <= fn_start {
                break;
            }
        }
    }
    None
}

#[test]
fn acp_actions_popup_uses_dynamic_agent_actions() {
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("acp_actions_context_built"),
        "ACP actions popup must log when it builds ACP actions context from the active session"
    );
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("thread.available_agents().to_vec()"),
        "ACP actions popup must source available agents from the live ACP thread"
    );
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("thread.available_models().to_vec()"),
        "ACP actions popup must source available models from the live ACP thread"
    );
}

#[test]
fn acp_action_handler_switches_agents_by_persisting_and_reopening() {
    assert!(
        ACTION_HANDLER_SOURCE.contains("acp_switch_agent_id_from_action"),
        "ACP action handler must detect switch-agent action IDs"
    );
    assert!(
        ACTION_HANDLER_SOURCE.contains("persist_preferred_acp_agent_id"),
        "switch-agent action must persist the selected ACP agent"
    );
    assert!(
        ACTION_HANDLER_SOURCE.contains("self.open_tab_ai_acp_with_entry_intent(None, cx);"),
        "switch-agent action must reopen ACP chat after changing the agent"
    );
}

#[test]
fn acp_action_handler_routes_agent_switch_through_retry_wrapper_before_reopen() {
    let switch_pos = required_pos(
        ACTION_HANDLER_SOURCE,
        "acp_switch_agent_id_from_action",
        "ACP action handler must detect switch-agent action IDs",
    );
    let wrapper_pos = required_pos_after(
        ACTION_HANDLER_SOURCE,
        switch_pos,
        "relaunch_for_agent_switch_preserving_draft",
        "switch-agent action must use the retry-staging relaunch wrapper",
    );
    let event_pos = required_pos_after(
        ACTION_HANDLER_SOURCE,
        wrapper_pos,
        "acp_switch_agent_relaunch_requested",
        "switch-agent action must emit acp_switch_agent_relaunch_requested tracing event",
    );
    let close_pos = required_pos_after(
        ACTION_HANDLER_SOURCE,
        wrapper_pos,
        "close_tab_ai_harness_terminal",
        "switch-agent action must close the harness terminal after staging retry",
    );
    let reopen_pos = required_pos_after(
        ACTION_HANDLER_SOURCE,
        close_pos,
        "open_tab_ai_acp_with_entry_intent(None, cx)",
        "switch-agent action must reopen ACP after closing the harness terminal",
    );
    assert!(
        wrapper_pos < close_pos && close_pos < reopen_pos,
        "switch-agent action must request retry staging before closing/reopening ACP"
    );
    assert!(
        wrapper_pos < event_pos && event_pos < close_pos,
        "switch-agent relaunch tracing should be emitted after retry staging is requested and before terminal close"
    );
}

#[test]
fn acp_agent_switch_relaunch_wrapper_stages_retry_payload() {
    let wrapper = rust_function_body(
        ACP_VIEW_SOURCE,
        "pub(crate) fn relaunch_for_agent_switch_preserving_draft",
    );
    let revalidate_pos = required_pos(
        wrapper,
        "revalidate_skill_context_for_agent",
        "agent-switch relaunch wrapper must revalidate skill context",
    );
    let stage_pos = required_pos(
        wrapper,
        "self.stage_agent_switch_retry(next_agent_id, cx);",
        "agent-switch relaunch wrapper must stage the retry payload",
    );
    assert!(
        revalidate_pos < stage_pos,
        "agent-switch relaunch wrapper should revalidate skill context before staging retry"
    );
    if let Some(close_pos) = wrapper.find("close_tab_ai_harness_terminal") {
        assert!(
            stage_pos < close_pos,
            "if teardown moves into the wrapper, retry payload staging must still happen before close"
        );
    }
}

#[test]
fn acp_agent_switch_retry_payload_preserves_requirements_and_draft_state() {
    let stage = rust_function_body(ACP_VIEW_SOURCE, "pub(crate) fn stage_agent_switch_retry");
    assert!(
        stage.contains("let launch_requirements = self.current_retry_launch_requirements(cx);"),
        "agent-switch retry payload must preserve launch capability requirements"
    );
    assert!(
        stage.contains("let draft_state = self.current_retry_draft_state(cx);"),
        "agent-switch retry payload must capture the current draft state"
    );
    assert!(
        stage.contains("self.pending_retry_request = Some(AcpRetryRequest"),
        "agent-switch retry payload must be queued before relaunch"
    );
    assert!(
        stage.contains("preferred_agent_id: Some(next_agent_id.clone())"),
        "agent-switch retry payload must target the selected replacement agent"
    );
    assert!(
        stage.contains("launch_requirements,") && stage.contains("draft_state,"),
        "agent-switch retry payload must carry both requirements and draft state"
    );
}

#[test]
fn acp_agent_switch_relaunch_preserves_existing_draft_state() {
    assert!(
        ACP_VIEW_SOURCE.contains("pub(crate) struct AcpRetryDraftState"),
        "ACP retry payload must define AcpRetryDraftState for live draft restoration"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("let draft_state = self.current_retry_draft_state(cx);"),
        "ACP agent switch retry staging must capture the current draft state"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("event = \"acp_switch_agent_retry_draft_restored\""),
        "ACP view must emit tracing when a switch-agent relaunch restores draft state"
    );
    let tab_ai_sources = rust_sources_under("src/app_impl/tab_ai_mode");
    assert!(
        source_tree_contains(
            &tab_ai_sources,
            "view.restore_retry_draft_state(draft_state, cx);"
        ),
        "ACP relaunch must restore the captured retry draft state onto the new view, wherever tab_ai_mode staging code lives"
    );
    let (_, guard_body) = source_tree_function_body_named(
        &tab_ai_sources,
        "should_stage_focused_part_for_retry_draft_restore",
    )
    .expect(
        "tab_ai_mode staging must expose a named guard for retry-draft focused-part suppression",
    );
    assert!(
        guard_body.contains("!has_retry_draft_state")
            || guard_body.contains("retry_draft_state.is_none()"),
        "retry-draft focused-part guard must return false when retry draft state is present"
    );
    let (restore_path, restore_body) = source_tree_function_body_containing(
        &tab_ai_sources,
        "view.restore_retry_draft_state(draft_state, cx);",
    )
    .expect("restore_retry_draft_state call must live inside a tab_ai_mode function body");
    assert!(
        restore_body.contains("should_stage_focused_part_for_retry_draft_restore"),
        "the relaunch function that restores retry drafts must also apply the focused-part suppression guard; found restore in {}",
        restore_path.display()
    );
    assert!(
        restore_body.contains("retry_draft_state.is_some()")
            || restore_body.contains("retry_draft_state.is_none()"),
        "the retry-draft restore path must branch on retry_draft_state when deciding whether to stage focused context; found restore in {}",
        restore_path.display()
    );
}

#[test]
fn acp_action_builder_exposes_agent_section_entries() {
    assert!(
        ACTION_BUILDER_SOURCE.contains(".with_section(\"Agent\")"),
        "ACP action builder must place switch actions in an Agent section"
    );
}

// ── Route / back-stack contract tests ────────────────────────────────────────

#[test]
fn acp_root_route_uses_change_agent_entry() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("ACP_CHANGE_AGENT_ACTION_ID"),
        "ACP actions must define ACP_CHANGE_AGENT_ACTION_ID"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_chat_root_route"),
        "ACP root route builder must exist"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_agent_picker_route"),
        "ACP agent picker route builder must exist"
    );
}

#[test]
fn acp_actions_dialog_registers_change_agent_drill_down() {
    assert!(
        DIALOG_SOURCE.contains("with_acp_chat"),
        "ActionsDialog must expose with_acp_chat"
    );
    assert!(
        DIALOG_SOURCE.contains("ACP_CHANGE_AGENT_ACTION_ID"),
        "with_acp_chat must register ACP_CHANGE_AGENT_ACTION_ID"
    );
    assert!(
        DIALOG_SOURCE.contains("register_drill_down_route"),
        "with_acp_chat must register the ACP drill-down route"
    );
}

#[test]
fn acp_picker_preserves_existing_switch_action_ids() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("acp_switch_agent_action_id(entry.id.as_ref())"),
        "Second-level ACP picker must preserve acp_switch_agent:* IDs"
    );
}

#[test]
fn toggle_actions_uses_route_based_acp_dialog() {
    assert!(
        ACTIONS_TOGGLE_SOURCE.contains("ActionsDialog::with_acp_chat"),
        "ACP actions open path must build a route-based dialog"
    );
}

#[test]
fn dialog_exposes_route_stack_public_api() {
    // Verify the core route/back-stack types and methods exist
    assert!(
        DIALOG_SOURCE.contains("pub struct ActionsDialogRoute"),
        "ActionsDialogRoute must be a public struct"
    );
    assert!(
        DIALOG_SOURCE.contains("pub enum ActionsDialogActivation"),
        "ActionsDialogActivation must be a public enum"
    );
    assert!(
        DIALOG_SOURCE.contains("pub enum ActionsDialogEscapeOutcome"),
        "ActionsDialogEscapeOutcome must be a public enum"
    );
    assert!(
        DIALOG_SOURCE.contains("pub fn activate_selected"),
        "activate_selected must be a public method"
    );
    assert!(
        DIALOG_SOURCE.contains("pub fn handle_escape"),
        "handle_escape must be a public method"
    );
    assert!(
        DIALOG_SOURCE.contains("pub fn route_hint_label"),
        "route_hint_label must be a public method"
    );
}

#[test]
fn dialog_has_structured_route_tracing() {
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_activation"),
        "activate_selected must emit actions_dialog_activation tracing"
    );
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_route_push"),
        "push_route must emit actions_dialog_route_push tracing"
    );
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_route_pop"),
        "pop_route must emit actions_dialog_route_pop tracing"
    );
    assert!(
        DIALOG_SOURCE.contains("actions_dialog_escape"),
        "handle_escape must emit actions_dialog_escape tracing"
    );
}

// ── Notes-hosted ACP agent switching contract tests ─────────────────────────

const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");

#[test]
fn notes_acp_dispatch_handles_switch_agent_actions() {
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("acp_switch_agent_id_from_action"),
        "Notes-hosted ACP must detect switch-agent action IDs"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("persist_preferred_acp_agent_id_sync"),
        "Notes-hosted ACP switch-agent flow must persist the selected agent synchronously"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("notes_acp_switch_agent_relaunched"),
        "Notes-hosted ACP switch-agent flow must emit relaunch tracing"
    );
}

#[test]
fn notes_acp_switch_agent_preserves_draft_input() {
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("view.capture_draft_snapshot(cx)"),
        "Notes-hosted ACP switch-agent flow must capture a full draft snapshot before relaunch"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("has_draft_input"),
        "Notes-hosted ACP switch-agent tracing must include draft input status"
    );
}

#[test]
fn notes_acp_switch_agent_tears_down_before_relaunch() {
    let hide_pos = NOTES_ACP_HOST_SOURCE
        .find("notes_acp_switch_agent_requested")
        .and_then(|start| {
            NOTES_ACP_HOST_SOURCE[start..]
                .find("prepare_for_host_hide")
                .map(|offset| start + offset)
        })
        .expect("prepare_for_host_hide must appear after switch-agent-requested");
    let drop_pos = NOTES_ACP_HOST_SOURCE[hide_pos..]
        .find("embedded_acp_chat = None")
        .map(|offset| hide_pos + offset)
        .expect("embedded_acp_chat = None must appear after prepare_for_host_hide");
    let relaunch_pos = NOTES_ACP_HOST_SOURCE[drop_pos..]
        .find("open_or_focus_embedded_acp")
        .map(|offset| drop_pos + offset)
        .expect("open_or_focus_embedded_acp must appear after dropping cached view");
    assert!(
        hide_pos < drop_pos && drop_pos < relaunch_pos,
        "Agent switch must: hide popups -> drop cached view -> relaunch"
    );
}

// ── Host-aware ACP unification contract tests ───────────────────────────────

#[test]
fn acp_builder_exposes_host_aware_route_api() {
    assert!(
        ACTION_BUILDER_SOURCE.contains("enum AcpActionsDialogHost"),
        "ACP builder must define AcpActionsDialogHost enum"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("AcpActionsDialogHost::Shared"),
        "AcpActionsDialogHost must have a Shared variant"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("AcpActionsDialogHost::Detached"),
        "AcpActionsDialogHost must have a Detached variant"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_chat_root_route_for_host"),
        "Host-aware ACP root route builder must exist"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("get_acp_agent_picker_route_for_host"),
        "Host-aware ACP agent picker route builder must exist"
    );
}

#[test]
fn actions_window_routes_focused_popup_shortcuts_through_shared_matcher() {
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("matching_action_id_for_keystroke"),
        "ActionsWindow must reuse the shared dialog shortcut matcher for focused popup fallback"
    );
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("activate_action_id"),
        "Shared actions dialog routing must expose activation by explicit action id"
    );
}

#[test]
fn actions_window_defers_activation_to_host_callback_when_present() {
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("on_activation_callback"),
        "ActionsWindow must read the dialog's activation callback"
    );
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("callback(activation, window, cx);"),
        "ActionsWindow must defer focused-popup activation back to the host callback"
    );
}

#[test]
fn detached_acp_uses_host_aware_route_builder() {
    // Detached ACP must NOT use the old flat DETACHED_SUPPORTED_ACTIONS constant
    assert!(
        !CHAT_WINDOW_SOURCE.contains("const DETACHED_SUPPORTED_ACTIONS"),
        "Detached ACP must not define a local DETACHED_SUPPORTED_ACTIONS allowlist"
    );
    // Detached ACP must use the host-aware dialog constructor
    assert!(
        CHAT_WINDOW_SOURCE.contains("with_acp_chat_for_host"),
        "Detached ACP must use with_acp_chat_for_host"
    );
    assert!(
        CHAT_WINDOW_SOURCE.contains("AcpActionsDialogHost::Detached")
            || CHAT_WINDOW_SOURCE.contains("builders::AcpActionsDialogHost::Detached"),
        "Detached ACP must specify Detached host"
    );
}

#[test]
fn dialog_exposes_host_aware_constructor() {
    assert!(
        DIALOG_SOURCE.contains("with_acp_chat_for_host"),
        "ActionsDialog must expose with_acp_chat_for_host"
    );
}

#[test]
fn detached_host_excludes_unsupported_actions() {
    // The detached host filter must reject panel-only actions
    assert!(
        ACTION_BUILDER_SOURCE.contains("acp_action_supported_in_host"),
        "ACP builder must have a host action filter function"
    );
    assert!(
        ACTION_BUILDER_SOURCE.contains("filter_acp_actions_for_host"),
        "ACP builder must have a host action filter"
    );
}

#[test]
fn route_visibility_logs_include_depth_and_escape_hint() {
    // Both shared and detached log sites must include route_depth and escape_hint
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("route_depth"),
        "Shared actions dialog route logs must include route_depth"
    );
    assert!(
        ACTIONS_DIALOG_SOURCE.contains("escape_hint"),
        "Shared actions dialog route logs must include escape_hint"
    );
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("route_depth"),
        "Detached actions window route logs must include route_depth"
    );
    assert!(
        ACTIONS_WINDOW_SOURCE.contains("escape_hint"),
        "Detached actions window route logs must include escape_hint"
    );
}
