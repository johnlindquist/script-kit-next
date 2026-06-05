//! Source-level contracts for the embedded ACP footer activity indicator.

const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const PROTOCOL_SURFACE_SOURCE: &str = include_str!("../src/protocol/types/automation_surface.rs");

fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature must exist");
    let rest = &source[start..];
    let body_start = rest.find('{').expect("function body must start");
    let mut depth = 0usize;
    for (idx, ch) in rest[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[..body_start + idx + 1];
                }
            }
            _ => {}
        }
    }
    panic!("function body must close");
}

#[test]
fn acp_view_exposes_one_footer_dot_status_mapper_for_active_states() {
    let body = fn_body(ACP_VIEW_SOURCE, "pub(crate) fn footer_dot_status(");

    assert!(
        body.contains("if self.context_capture_pending")
            && body.contains("return FooterDotStatus::Streaming;"),
        "context capture must activate the footer dot before the thread itself is streaming"
    );
    assert!(
        body.contains("AcpThreadStatus::Streaming => FooterDotStatus::Streaming"),
        "streaming turns, including tool-only activity, must use the pulsing active dot"
    );
    assert!(
        body.contains(
            "AcpThreadStatus::WaitingForPermission => FooterDotStatus::WaitingForPermission"
        ),
        "permission waits must keep the active pulsing dot treatment"
    );
}

#[test]
fn native_footer_uses_cached_acp_status_without_child_entity_reads() {
    let body = fn_body(
        UI_WINDOW_SOURCE,
        "pub(crate) fn enrich_footer_config_with_acp_info(",
    );

    assert!(
        body.contains("self.acp_footer_dot_status")
            && body.contains("self.acp_footer_model_display.as_ref()"),
        "native footer must use the deferred parent cache populated from AcpChatView notifications"
    );
    assert!(
        body.contains("profile_left_info()")
            || (body.contains(
                "icon_token: Some(crate::components::footer_chrome::FOOTER_PROFILE_ICON_TOKEN)"
            ) && body.contains("action: Some(crate::footer_popup::FooterAction::Ai)")),
        "native footer must expose profile selector through the merged left status marker"
    );
    assert!(
        !body.contains("entity.read(") && !body.contains(".read(cx)"),
        "native footer sync must not read AcpChatView while child notifications may still be inside an AcpChatView update"
    );
}

#[test]
fn native_profile_icon_pulse_uses_opacity_without_scaling() {
    assert!(
        FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MIN_OPACITY: f32 = 0.22"),
        "native active profile icon opacity must dip far enough below 50% to read as a pulse"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS: f64 = 1.1"),
        "native active dot should use a slow breathing pulse, not a fast blinking cadence"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("update_footer_icon_layer(icon_layer, info);"),
        "native footer must pulse the profile icon for active ACP states"
    );
    let body = fn_body(
        FOOTER_POPUP_SOURCE,
        "unsafe fn add_active_dot_pulse_animation(",
    );
    assert!(
        body.contains("ns_string(\"opacity\")") && body.contains("pulseOpacity"),
        "native active profile icon must pulse opacity"
    );
    assert!(
        !body.contains("transform.scale")
            && !body.contains("pulseScale")
            && !FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MAX_SCALE")
            && !FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MIN_SCALE"),
        "native active dot must not scale while pulsing"
    );
}

#[test]
fn native_footer_profile_icon_replaces_active_dot_for_acp_marker() {
    let body = fn_body(FOOTER_POPUP_SOURCE, "unsafe fn layout_footer_left_info(");

    assert!(
        body.contains("let show_dot = info.icon_token.is_none()"),
        "ACP profile markers must not render a separate status dot"
    );
    assert!(
        body.contains("ensure_footer_left_profile_icon_view(left_info_view)")
            && body.contains("update_footer_icon_layer(icon_layer, info);"),
        "left info layout must reconcile and pulse the profile icon"
    );
    assert!(
        !body.contains("info.icon_token.is_some() && !matches!(info.dot_status"),
        "active state should never add a second dot beside a profile icon"
    );
}

#[test]
fn native_dot_animation_uses_core_animation_duration_abi_safely() {
    let body = fn_body(
        FOOTER_POPUP_SOURCE,
        "unsafe fn add_active_dot_pulse_animation(",
    );

    assert!(
        body.contains("let duration: f64")
            && body.contains("setDuration: duration")
            && !body.contains("setDuration: FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS"),
        "CABasicAnimation setDuration: must receive f64/CFTimeInterval, not an inferred numeric type"
    );
}

#[test]
fn active_dot_is_not_gated_by_model_label() {
    let body = fn_body(FOOTER_POPUP_SOURCE, "unsafe fn layout_footer_left_info(");

    assert!(
        !body.contains("if info.model_name.is_empty() {\n        return;"),
        "active footer dot must still show while model label is empty"
    );
    assert!(
        body.contains("if info.model_name.is_empty()")
            && body.contains("remove_identified_subview(left_info_view, FOOTER_MODEL_LABEL_ID)")
            && body.find("let show_dot").expect("show dot calculation")
                < body
                    .find("if info.model_name.is_empty()")
                    .expect("model label branch"),
        "model label absence should only remove the label after dot reconciliation"
    );
}

#[test]
fn embedded_acp_observer_repaints_parent_for_visible_footer_status_transitions() {
    let body = fn_body(TAB_AI_MODE_SOURCE, "fn sync_embedded_acp_observed_state(");

    assert!(
        body.contains("let ready_script_path_changed = self.acp_ready_script_path != new_path;"),
        "deferred observer sync must still cache ready-script changes for footer button resolution"
    );
    assert!(
        body.contains("let visible_acp_view_changed = matches!(")
            && body.contains("AppView::AcpChatView { entity } if entity == view_entity"),
        "deferred observer sync must detect notifications from the currently visible embedded ACP view"
    );
    assert!(
        body.contains("let footer_status_changed = if visible_acp_view_changed")
            && body.contains("let snapshot = view.footer_snapshot(cx);")
            && body.contains("self.acp_footer_snapshot = Some(snapshot);"),
        "observer must cache visible ACP footer state so token-by-token child updates do not restart the pulse animation"
    );
    assert!(
        body.contains("if ready_script_path_changed || footer_status_changed")
            && body.contains("cx.notify();"),
        "visible ACP footer status transitions must repaint ScriptListApp so the native footer dot can pulse during active turns"
    );
}

#[test]
fn embedded_acp_observer_defers_child_entity_reads() {
    let observer_body = fn_body(TAB_AI_MODE_SOURCE, "fn wire_embedded_acp_footer_callbacks(");
    assert!(
        observer_body.contains("this.schedule_embedded_acp_observed_state_sync(view_entity, cx);")
            && !observer_body.contains("let view = view_entity.read(cx);"),
        "observer must not synchronously read AcpChatView while the child notify may still be inside an AcpChatView update"
    );

    let schedule_body = fn_body(
        TAB_AI_MODE_SOURCE,
        "fn schedule_embedded_acp_observed_state_sync(",
    );
    assert!(
        schedule_body.contains("ACP_OBSERVED_STATE_SYNC_GENERATION.fetch_add")
            && schedule_body.contains("timer(std::time::Duration::from_millis(50))")
            && schedule_body.contains("ACP_OBSERVED_STATE_SYNC_GENERATION.load")
            && schedule_body.contains("this.sync_embedded_acp_observed_state(&view_entity, cx);"),
        "observer must debounce child-state reads until the AcpChatView notification burst settles"
    );
}

#[test]
fn get_state_active_footer_exposes_acp_model_status_text() {
    assert!(
        PROTOCOL_SURFACE_SOURCE.contains("pub left_info: Option<ActiveFooterLeftInfoSnapshot>")
            && PROTOCOL_SURFACE_SOURCE.contains("pub struct ActiveFooterLeftInfoSnapshot")
            && PROTOCOL_SURFACE_SOURCE.contains("pub dot_status: String")
            && PROTOCOL_SURFACE_SOURCE.contains("pub model_name: String")
            && PROTOCOL_SURFACE_SOURCE.contains("pub profile_name: Option<String>")
            && PROTOCOL_SURFACE_SOURCE.contains("pub action: Option<String>"),
        "getState.activeFooter must expose the footer model/status label for ACP proof"
    );

    let body = fn_body(
        PROMPT_HANDLER_SOURCE,
        "pub(crate) fn active_footer_snapshot(",
    );
    assert!(
        body.contains("self.enrich_footer_config_with_acp_info(cfg);")
            && body.contains("left_info = config.as_ref().and_then")
            && body.contains("dot_status: Self::active_footer_dot_status_name")
            && body.contains("model_name: info.model_name.clone()")
            && body.contains("profile_name: info.profile_name.clone()")
            && body.contains("action: info.action.map(Self::footer_action_name)"),
        "active footer snapshots must include ACP-enriched left info, including status text"
    );
}

#[test]
fn acp_agent_model_chip_remains_context_slot_with_active_dot() {
    let chip_body = fn_body(UI_WINDOW_SOURCE, "fn global_main_window_left_chip_buttons(");
    assert!(
        chip_body.contains("FooterAction::AgentModel")
            && chip_body.contains("agent_model_dot_status")
            && chip_body.contains("button.leading_dot(dot_status)")
            && chip_body.contains("buttons.push(button);"),
        "ACP footer enrichment must keep the Agent/Model entry as the active status chip"
    );

    let role_body = fn_body(FOOTER_POPUP_SOURCE, "pub(crate) fn footer_button_slot_role");
    assert!(
        role_body.contains("FooterAction::Cwd | FooterAction::AgentModel")
            && role_body.contains("FooterSlotRole::ContextChip"),
        "AgentModel must be a context chip so its active dot does not inflate footer action slots"
    );

    assert!(
        PROTOCOL_SURFACE_SOURCE.contains("pub action_slot_count: usize")
            && PROTOCOL_SURFACE_SOURCE.contains("pub context_chip_count: usize")
            && PROMPT_HANDLER_SOURCE.contains("let slot_model = config.as_ref().map(|cfg| cfg.slot_model());")
            && PROMPT_HANDLER_SOURCE.contains("model.action_slot_count")
            && PROMPT_HANDLER_SOURCE.contains("model.context_chip_count"),
        "getState.activeFooter must expose action and context slot counts derived from the footer slot model"
    );
}
