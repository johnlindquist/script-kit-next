//! Source-level contracts for the embedded ACP footer activity indicator.

const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");

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

// doc-anchor-removed: [[acp-chat#Footer activity indicator]]
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
        body.contains("FooterLeftInfo")
            && body.contains("dot_status,")
            && body.contains("model_name: model_name.clone()"),
        "native footer must still publish ACP dot/model info from cached values"
    );
    assert!(
        !body.contains("entity.read(") && !body.contains(".read(cx)"),
        "native footer sync must not read AcpChatView while child notifications may still be inside an AcpChatView update"
    );
}

#[test]
fn native_active_dot_pulse_uses_opacity_without_scaling() {
    assert!(
        FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MIN_OPACITY: f32 = 0.22"),
        "native active dot opacity must dip far enough below 50% to read as a color pulse"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS: f64 = 1.1"),
        "native active dot should use a slow breathing pulse, not a fast blinking cadence"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("ensure_active_dot_pulse_animation(layer);"),
        "native active dot must ensure the active pulse animation idempotently"
    );
    let body = fn_body(
        FOOTER_POPUP_SOURCE,
        "unsafe fn add_active_dot_pulse_animation(",
    );
    assert!(
        body.contains("ns_string(\"opacity\")") && body.contains("pulseOpacity"),
        "native active dot must pulse opacity/color"
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
fn native_footer_dot_is_reconciled_not_rebuilt_each_refresh() {
    let body = fn_body(FOOTER_POPUP_SOURCE, "unsafe fn layout_footer_left_info(");

    assert!(
        !body.contains("Remove all existing subviews")
            && !body.contains("for i in (0..count).rev()"),
        "left info layout must not blindly remove the animated dot every refresh"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("const FOOTER_STATUS_DOT_ID")
            && FOOTER_POPUP_SOURCE
                .contains("find_subview_by_identifier(left_info_view, FOOTER_STATUS_DOT_ID)"),
        "native active dot must be reused by identifier so its CALayer animation survives refreshes"
    );
    assert!(
        body.contains("ensure_footer_status_dot_view(left_info_view)")
            && body.contains("remove_identified_subview(left_info_view, FOOTER_STATUS_DOT_ID)"),
        "left info layout must reconcile the stable dot instead of recreating it"
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
            && body.contains("self.acp_footer_dot_status = Some(dot_status);")
            && body.contains("self.acp_footer_model_display = Some(model_display);"),
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
