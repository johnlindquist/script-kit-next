//! Source-level contract for routing the ACP model selector through Cmd+K actions.

const ACP_VIEW: &str = include_str!("../src/ai/acp/view.rs");
const ACP_MOD: &str = include_str!("../src/ai/acp/mod.rs");
const SCRIPT_CONTEXT_ACTIONS: &str = include_str!("../src/actions/builders/script_context.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");
const LIFECYCLE_RESET: &str = include_str!("../src/app_impl/lifecycle_reset.rs");
const TRANSACTION_PROVIDER: &str =
    include_str!("../src/windows/automation_transaction_provider.rs");

#[test]
fn acp_model_toolbar_opens_actions_instead_of_detached_selector_list() {
    assert!(
        ACP_VIEW.contains("fn trigger_toggle_actions_from_parent")
            && ACP_VIEW.contains("parent.handle.update(cx, |_root, window, cx|")
            && ACP_VIEW.contains("callback(window, cx);"),
        "ACP model toolbar needs a parent-window bridge into the host actions callback"
    );

    let handler = ACP_VIEW
        .split("AcpToolbarEvent::ToggleModelSelector(parent) =>")
        .nth(1)
        .and_then(|tail| tail.split("AcpToolbarEvent::ExportThread").next())
        .expect("model selector toolbar handler");

    assert!(
        handler.contains("this.sync_acp_popup_windows_from_cached_parent(cx);")
            && handler.contains("this.trigger_toggle_actions_from_parent(*parent, cx);")
            && !handler.contains("model_selector_open"),
        "toolbar model selection should open Cmd+K actions rather than toggling the detached selector"
    );
}

#[test]
fn acp_model_picker_actions_are_stable_model_rows() {
    assert!(
        SCRIPT_CONTEXT_ACTIONS.contains("pub(crate) fn get_acp_model_picker_route")
            && SCRIPT_CONTEXT_ACTIONS.contains("id: ACP_MODEL_PICKER_ROUTE_ID.to_string()")
            && SCRIPT_CONTEXT_ACTIONS.contains("get_acp_model_picker_actions")
            && SCRIPT_CONTEXT_ACTIONS.contains("initial_selected_action_id: selected_model_id.map(acp_switch_model_action_id)"),
        "ACP model picker must be an actions route with a selected model action id"
    );
    assert!(
        SCRIPT_CONTEXT_ACTIONS.contains("acp_switch_model_action_id(&entry.id)")
            && SCRIPT_CONTEXT_ACTIONS.contains("AcpModelSelectionActionPlan::from_is_selected(is_selected)")
            && SCRIPT_CONTEXT_ACTIONS.contains("selection_plan.picker_title(&display_name)")
            && SCRIPT_CONTEXT_ACTIONS.contains("ActionCategory::ScriptContext"),
        "available models should remain stable directly selectable action rows"
    );
}

#[test]
fn acp_model_selector_no_longer_registers_prompt_popup_route() {
    assert!(!ACP_MOD.contains("mod model_selector_popup"));
    assert!(!PROMPT_HANDLER.contains("is_model_selector_popup_window_open"));
    assert!(!PROMPT_HANDLER.contains("batch_select_model_by_value"));
    assert!(!PROMPT_HANDLER.contains("batch_select_model_by_semantic_id"));
    assert!(!COLLECTOR.contains("collect_model_selector_snapshot"));
    assert!(!LIFECYCLE_RESET.contains("model_selector_popup"));
    assert!(!TRANSACTION_PROVIDER.contains("model_selector_popup"));
}
