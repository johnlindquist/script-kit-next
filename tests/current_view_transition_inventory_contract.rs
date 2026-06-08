//! Source-level contract for the generated current-view transition inventory.
//!
//! The inventory is not a replacement for named transition owners. It is the
//! agent-readable map of remaining direct mutation sites while those owners are
//! introduced in smaller behavior slices.

use std::process::Command;

use serde_json::json;

const GENERATOR: &str = include_str!("../scripts/generate-current-view-transitions.ts");
const INVENTORY_JSON: &str = include_str!("../docs/ai/contracts/current-view-transitions.json");
const AUTOMATION_SURFACE: &str = include_str!("../src/app_impl/automation_surface.rs");
const AGENT_CHAT_SURFACE_TRANSITIONS: &str =
    include_str!("../src/app_impl/agent_chat_surface_transitions.rs");

fn inventory_entries() -> Vec<serde_json::Value> {
    let parsed: serde_json::Value = serde_json::from_str(INVENTORY_JSON)
        .expect("current-view transition inventory must be valid JSON");
    assert_eq!(parsed["schemaVersion"], 1);
    assert_eq!(
        parsed["inventory"],
        "ScriptListApp/AppView current_view transition sites"
    );
    parsed["entries"]
        .as_array()
        .expect("inventory entries must be an array")
        .clone()
}

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let open_rel = source[start..]
        .find('{')
        .unwrap_or_else(|| panic!("missing function body open: {signature}"));
    let open = start + open_rel;
    let mut depth = 0usize;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("missing function body close: {signature}");
}

fn assert_before(source: &str, before: &str, after: &str) {
    let before_index = source
        .find(before)
        .unwrap_or_else(|| panic!("missing ordered source marker: {before}"));
    let after_index = source
        .find(after)
        .unwrap_or_else(|| panic!("missing ordered source marker: {after}"));
    assert!(
        before_index < after_index,
        "`{before}` must appear before `{after}`"
    );
}

fn helper_contract(entries: &[serde_json::Value], helper: &str) -> serde_json::Value {
    entries
        .iter()
        .find_map(|entry| {
            if entry["operation"] == "transitionHelper" && entry["helper"] == helper {
                entry["transitionContract"]
                    .as_object()
                    .map(|_| entry["transitionContract"].clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("missing transitionContract for helper {helper}"))
}

fn expected_transition_contract(helper: &str) -> serde_json::Value {
    match helper {
        "transition_current_view_and_rekey_main_automation_surface" => json!({
            "oldView": "runtimeCurrentView",
            "newView": "dynamicArgument",
            "surfaceKind": "derivedFromNewView",
            "semanticSurface": "rekeyMainAutomationSurfaceFromCurrentView",
            "mainAutomationRekey": true,
            "focusTarget": "callerOwned",
            "focusedInput": "callerOwned",
            "resize": "callerOwned",
            "activePopupContract": "stateReceiptOnly",
            "stateSnapshot": "getState.surfaceContract"
        }),
        "restore_current_view_with_focus" => json!({
            "oldView": "runtimeCurrentView",
            "newView": "dynamicArgument",
            "mainAutomationRekey": false,
            "focusTarget": "dynamicArgument",
            "focusedInputMap": {
                "MainFilter": "MainFilter",
                "ActionsDialog": "ActionsSearch",
                "default": "None"
            },
            "resize": "callerOwned",
            "stateSnapshot": "getState.surfaceContract"
        }),
        "show_script_list_with_main_filter_focus" => json!({
            "oldView": "runtimeCurrentView",
            "newView": "AppView::ScriptList",
            "delegatesTo": "restore_current_view_with_focus",
            "focusTarget": "MainFilter",
            "focusedInput": "MainFilter",
            "mainAutomationRekey": true,
            "semanticSurface": "rekeyMainAutomationSurfaceFromCurrentView",
            "resize": "callerOwned",
            "stateSnapshot": "getState.surfaceContract"
        }),
        "enter_embedded_agent_chat_surface" => json!({
            "oldView": "runtimeCurrentView",
            "newView": "AppView::AgentChatView",
            "surfaceKind": "AgentChat",
            "embeddedAiWindowUpsert": true,
            "mainAutomationRekey": true,
            "agent_chatSurfaceEvent": "EmbeddedOpened",
            "actionsCleanup": "clearActionsPopupState",
            "focusTarget": "AgentChat",
            "focusCoordinatorRequest": "FocusRequest::agent_chat",
            "focusedInput": "None",
            "resize": "callerOwned",
            "stateSnapshot": "getState.surfaceContract"
        }),
        _ => panic!("unexpected transition helper {helper}"),
    }
}

#[test]
fn generated_current_view_transition_inventory_is_not_stale() {
    let output = Command::new("bun")
        .arg("scripts/generate-current-view-transitions.ts")
        .arg("--check")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("bun must run current-view transition generator");
    assert!(
        output.status.success(),
        "current-view transition inventory is stale:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn inventory_entries_expose_agent_needed_transition_fields() {
    let entries = inventory_entries();
    assert!(
        entries.len() >= 20,
        "inventory should expose the broad remaining current_view transition surface"
    );
    for entry in entries {
        for key in [
            "file",
            "owner",
            "receiver",
            "operation",
            "expression",
            "inferredTarget",
        ] {
            assert!(
                entry[key].as_str().is_some(),
                "inventory entry must expose `{key}` as a string: {entry:?}"
            );
        }
        assert!(
            entry["line"].as_u64().is_some(),
            "inventory entry must expose line as an integer: {entry:?}"
        );
        assert!(
            entry["requiresManualReview"].as_bool().is_some(),
            "inventory entry must expose requiresManualReview as a boolean: {entry:?}"
        );
        if entry["operation"] == "transitionHelper" {
            assert!(
                entry["helper"].as_str().is_some(),
                "transitionHelper entries must expose the helper name: {entry:?}"
            );
            assert!(
                entry["transitionContract"].as_object().is_some(),
                "transitionHelper entries must expose transitionContract metadata: {entry:?}"
            );
        }
    }
}

#[test]
fn inventory_excludes_source_audit_fixture_strings() {
    let entries = inventory_entries();
    assert!(
        entries.iter().all(|entry| {
            entry["expression"]
                .as_str()
                .map(|expression| !expression.contains("\","))
                .unwrap_or(true)
        }),
        "inventory expressions must come from Rust transition code, not quoted source-audit fixtures"
    );
    assert!(
        !entries.iter().any(|entry| {
            entry["file"] == "src/app_impl/tab_ai_mode/mod.rs"
                && entry["expression"] == "AppView::QuickTerminalView\","
        }),
        "tab_ai source-audit fixture string must not appear as a current_view transition"
    );
}

#[test]
fn inventory_captures_known_transition_classes() {
    let entries = inventory_entries();
    for (file, owner, target) in [
        (
            "src/app_impl/about_route.rs",
            "open_about_surface",
            "AppView::About",
        ),
        (
            "src/app_impl/trigger_builtin_dispatch.rs",
            "apply_filterable_route_plan",
            "dynamic",
        ),
        (
            "src/app_impl/agent_chat_surface_transitions.rs",
            "enter_embedded_agent_chat_surface",
            "AppView::AgentChatView",
        ),
        (
            "src/app_impl/registries_state.rs",
            "reset_to_script_list",
            "AppView::ScriptList",
        ),
        (
            "src/prompt_handler/mod.rs",
            "handle_prompt_message",
            "AppView::ArgPrompt",
        ),
        (
            "src/app_execute/builtin_execution.rs",
            "open_main_window",
            "AppView::ScriptList",
        ),
        (
            "src/app_actions/handle_action/mod.rs",
            "transition_to_script_list_after_action",
            "AppView::ScriptList",
        ),
    ] {
        assert!(
            entries.iter().any(|entry| {
                entry["file"] == file
                    && entry["owner"] == owner
                    && entry["inferredTarget"] == target
            }),
            "inventory must include {file}::{owner} -> {target}"
        );
    }
}

#[test]
fn inventory_captures_named_transition_helper_call_sites() {
    let entries = inventory_entries();
    for (file, owner, helper, target) in [
        (
            "src/app_impl/tab_ai_mode/agent_chat_setup.rs",
            "show_embedded_agent_chat_setup_view",
            "enter_embedded_agent_chat_surface",
            "AppView::AgentChatView",
        ),
        (
            "src/app_impl/tab_ai_mode/agent_chat_launch.rs",
            "open_standard_agent_chat_mock_fixture",
            "enter_embedded_agent_chat_surface",
            "AppView::AgentChatView",
        ),
        (
            "src/app_impl/tab_ai_mode/mod.rs",
            "try_reuse_embedded_agent_chat_view",
            "enter_embedded_agent_chat_surface",
            "AppView::AgentChatView",
        ),
        (
            "src/app_execute/builtin_execution.rs",
            "open_main_window",
            "show_script_list_with_main_filter_focus",
            "AppView::ScriptList",
        ),
        (
            "src/app_impl/attachment_portal.rs",
            "open_script_list_attachment_portal",
            "show_script_list_with_main_filter_focus",
            "AppView::ScriptList",
        ),
        (
            "src/app_impl/filter_input_change.rs",
            "handle_filter_input_change",
            "show_script_list_with_main_filter_focus",
            "AppView::ScriptList",
        ),
        (
            "src/app_impl/about_route.rs",
            "open_about_surface",
            "transition_current_view_and_rekey_main_automation_surface",
            "AppView::About",
        ),
    ] {
        assert!(
            entries.iter().any(|entry| {
                entry["file"] == file
                    && entry["owner"] == owner
                    && entry["helper"] == helper
                    && entry["inferredTarget"] == target
            }),
            "inventory must include helper call site {file}::{owner} via {helper} -> {target}"
        );
    }
}

#[test]
fn inventory_transition_helper_entries_expose_checked_transition_contracts() {
    let entries = inventory_entries();
    for entry in entries
        .iter()
        .filter(|entry| entry["operation"] == "transitionHelper")
    {
        let helper = entry["helper"]
            .as_str()
            .expect("transitionHelper entry must expose helper");
        assert_eq!(
            entry["transitionContract"],
            expected_transition_contract(helper),
            "transition helper entry must expose exact transitionContract metadata: {entry:?}"
        );
    }

    let rekey = helper_contract(
        &entries,
        "transition_current_view_and_rekey_main_automation_surface",
    );
    assert_eq!(rekey["oldView"], "runtimeCurrentView");
    assert_eq!(rekey["newView"], "dynamicArgument");
    assert_eq!(rekey["surfaceKind"], "derivedFromNewView");
    assert_eq!(
        rekey["semanticSurface"],
        "rekeyMainAutomationSurfaceFromCurrentView"
    );
    assert_eq!(rekey["mainAutomationRekey"], true);
    assert_eq!(rekey["focusTarget"], "callerOwned");
    assert_eq!(rekey["focusedInput"], "callerOwned");
    assert_eq!(rekey["resize"], "callerOwned");
    assert_eq!(rekey["activePopupContract"], "stateReceiptOnly");
    assert_eq!(rekey["stateSnapshot"], "getState.surfaceContract");

    let restore = helper_contract(&entries, "restore_current_view_with_focus");
    assert_eq!(restore["oldView"], "runtimeCurrentView");
    assert_eq!(restore["newView"], "dynamicArgument");
    assert_eq!(restore["mainAutomationRekey"], false);
    assert_eq!(restore["focusTarget"], "dynamicArgument");
    assert_eq!(restore["focusedInputMap"]["MainFilter"], "MainFilter");
    assert_eq!(restore["focusedInputMap"]["ActionsDialog"], "ActionsSearch");
    assert_eq!(restore["focusedInputMap"]["default"], "None");
    assert_eq!(restore["resize"], "callerOwned");
    assert_eq!(restore["stateSnapshot"], "getState.surfaceContract");

    let script_list = helper_contract(&entries, "show_script_list_with_main_filter_focus");
    assert_eq!(script_list["newView"], "AppView::ScriptList");
    assert_eq!(
        script_list["delegatesTo"],
        "restore_current_view_with_focus"
    );
    assert_eq!(script_list["focusTarget"], "MainFilter");
    assert_eq!(script_list["focusedInput"], "MainFilter");
    assert_eq!(script_list["mainAutomationRekey"], true);
    assert_eq!(
        script_list["semanticSurface"],
        "rekeyMainAutomationSurfaceFromCurrentView"
    );
    assert_eq!(script_list["resize"], "callerOwned");
    assert_eq!(script_list["stateSnapshot"], "getState.surfaceContract");

    let agent_chat = helper_contract(&entries, "enter_embedded_agent_chat_surface");
    assert_eq!(agent_chat["newView"], "AppView::AgentChatView");
    assert_eq!(agent_chat["surfaceKind"], "AgentChat");
    assert_eq!(agent_chat["embeddedAiWindowUpsert"], true);
    assert_eq!(agent_chat["mainAutomationRekey"], true);
    assert_eq!(agent_chat["agent_chatSurfaceEvent"], "EmbeddedOpened");
    assert_eq!(agent_chat["actionsCleanup"], "clearActionsPopupState");
    assert_eq!(agent_chat["focusTarget"], "AgentChat");
    assert_eq!(
        agent_chat["focusCoordinatorRequest"],
        "FocusRequest::agent_chat"
    );
    assert_eq!(agent_chat["focusedInput"], "None");
    assert_eq!(agent_chat["resize"], "callerOwned");
    assert_eq!(agent_chat["stateSnapshot"], "getState.surfaceContract");
}

#[test]
fn transition_helper_bodies_match_declared_transition_contracts() {
    let rekey = function_body(
        AUTOMATION_SURFACE,
        "pub(crate) fn transition_current_view_and_rekey_main_automation_surface(",
    );
    assert_before(
        rekey,
        "self.current_view = next_view;",
        "self.rekey_main_automation_surface_from_current_view()",
    );

    let restore = function_body(
        AUTOMATION_SURFACE,
        "pub(crate) fn restore_current_view_with_focus(",
    );
    assert!(restore.contains("self.current_view = next_view;"));
    assert!(restore.contains("self.pending_focus = Some(focus_target);"));
    assert!(restore.contains("FocusTarget::MainFilter => FocusedInput::MainFilter"));
    assert!(restore.contains("FocusTarget::ActionsDialog => FocusedInput::ActionsSearch"));
    assert!(restore.contains("_ => FocusedInput::None"));
    assert_before(
        restore,
        "self.current_view = next_view;",
        "self.pending_focus = Some(focus_target);",
    );
    assert_before(
        restore,
        "self.pending_focus = Some(focus_target);",
        "self.focused_input = match focus_target",
    );

    let script_list = function_body(
        AUTOMATION_SURFACE,
        "pub(crate) fn show_script_list_with_main_filter_focus(",
    );
    assert_before(
        script_list,
        "self.restore_current_view_with_focus(AppView::ScriptList, FocusTarget::MainFilter);",
        "self.rekey_main_automation_surface_from_current_view()",
    );

    let agent_chat = function_body(
        AGENT_CHAT_SURFACE_TRANSITIONS,
        "pub(crate) fn enter_embedded_agent_chat_surface(",
    );
    assert_before(
        agent_chat,
        "self.current_view = AppView::AgentChatView",
        "crate::windows::ensure_embedded_ai_window(true);",
    );
    assert_before(
        agent_chat,
        "crate::windows::ensure_embedded_ai_window(true);",
        "self.rekey_main_automation_surface_from_current_view();",
    );
    assert_before(
        agent_chat,
        "self.rekey_main_automation_surface_from_current_view();",
        "self.transition_agent_chat_surface(AgentChatSurfaceEvent::EmbeddedOpened);",
    );
    assert_before(
        agent_chat,
        "self.clear_actions_popup_state();",
        "self.focus_coordinator",
    );
    assert_before(
        agent_chat,
        "self.transition_agent_chat_surface(AgentChatSurfaceEvent::EmbeddedOpened);",
        "self.clear_actions_popup_state();",
    );
    assert_before(
        agent_chat,
        "self.focused_input = FocusedInput::None;",
        "self.focus_coordinator",
    );
    assert_before(
        agent_chat,
        ".request(crate::focus_coordinator::FocusRequest::agent_chat());",
        "self.sync_coordinator_to_legacy();",
    );
}

#[test]
fn generator_scans_source_instead_of_hardcoding_transition_entries() {
    for expected in [
        "SOURCE_DIRS",
        "src/app_actions",
        "src/app_execute",
        "src/app_impl",
        "src/prompt_handler",
        "src/main_sections",
        "src/main_entry",
        "assignmentRegex",
        "std::mem::replace",
        "TRANSITION_HELPERS",
        "enter_embedded_agent_chat_surface",
        "restore_current_view_with_focus",
        "show_script_list_with_main_filter_focus",
        "--check",
        "--write",
    ] {
        assert!(
            GENERATOR.contains(expected),
            "generator must include source-backed inventory marker `{expected}`"
        );
    }
}
