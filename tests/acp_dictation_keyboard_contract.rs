//! Source-level contracts for embedded ACP keyboard/window close behavior
//! after dictation opens ACP with an initial submitted prompt.

const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_ACTIONS_SOURCE: &str = include_str!("../src/app_impl/startup_new_actions.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../src/main_entry/app_run_setup.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker must exist");
    let tail = &source[start_idx..];
    let end_idx = tail.find(end).expect("end marker must exist");
    &tail[..end_idx]
}

fn assert_ordered(section: &str, before: &str, after: &str) {
    let before_idx = section.find(before).expect("before marker must exist");
    let after_idx = section.find(after).expect("after marker must exist");
    assert!(
        before_idx < after_idx,
        "`{before}` must appear before `{after}` in:\n{section}"
    );
}

#[test]
fn embedded_acp_escape_routes_to_lifecycle_close_without_hiding_main() {
    for (name, source, end_marker) in [
        (
            "startup.rs",
            STARTUP_SOURCE,
            "// Handle Cmd+Shift+K for add_shortcut",
        ),
        (
            "startup_new_actions.rs",
            STARTUP_NEW_ACTIONS_SOURCE,
            "// Handle Cmd+I to toggle info panel",
        ),
    ] {
        let section = section_between(source, "let acp_escape_popup_open", end_marker);
        assert!(
            section.contains("has_escape_dismissible_popup")
                && section.contains("!this.show_actions_popup")
                && section.contains("embedded_acp_escape_return_to_origin")
                && section.contains("close_tab_ai_harness_terminal_with_window(window, cx)")
                && section.contains("cx.stop_propagation()"),
            "{name} must intercept Escape for embedded ACP only when no ACP/actions popup owns Escape"
        );
        assert!(
            !section.contains("close_and_reset_window(cx)"),
            "{name} Escape from embedded ACP must return to origin, not hide the main window"
        );
    }
}

#[test]
fn embedded_acp_cmd_w_closes_lifecycle_before_hiding_main() {
    for (name, source) in [
        ("startup.rs", STARTUP_SOURCE),
        ("startup_new_actions.rs", STARTUP_NEW_ACTIONS_SOURCE),
    ] {
        let section = section_between(
            source,
            "// Handle Cmd+W for AcpChatView",
            "let acp_escape_popup_open",
        );
        assert!(
            section.contains("AppView::AcpChatView")
                && section.contains("embedded_acp_cmd_w_close_window"),
            "{name} must identify embedded ACP Cmd+W distinctly"
        );
        assert_ordered(
            section,
            "close_tab_ai_harness_terminal_with_window(window, cx)",
            "close_and_reset_window(cx)",
        );
    }
}

#[test]
fn focused_acp_view_handles_escape_and_cmd_w_without_root_bubbling() {
    let escape_section = section_between(
        ACP_VIEW_SOURCE,
        "Escape with no open dialogs: close via the host callback",
        "Enter submits.",
    );
    assert!(
        escape_section.contains("embedded_acp_escape_host_close_requested")
            && escape_section.contains("self.trigger_close_requested(window, cx)")
            && escape_section.contains("cx.stop_propagation()")
            && !escape_section.contains("cx.propagate()"),
        "focused embedded ACP Escape must close through the host callback instead of relying on root propagation"
    );

    let cmd_w_section = section_between(
        ACP_VIEW_SOURCE,
        "if modifiers.platform && key.eq_ignore_ascii_case(\"w\")",
        "Cmd+. / Cmd+Shift+O",
    );
    assert!(
        cmd_w_section.contains("!is_detached_host")
            && cmd_w_section.contains("self.trigger_close_window_requested(window, cx)")
            && cmd_w_section.contains("cx.stop_propagation()"),
        "focused embedded ACP Cmd+W must call the host window-close callback directly"
    );
}

#[test]
fn embedded_acp_host_wires_close_window_shortcut_callback() {
    let section = section_between(
        TAB_AI_MODE_SOURCE,
        "fn wire_embedded_acp_footer_callbacks",
        "let history_app = app_entity.clone();",
    );
    assert!(
        section.contains("set_on_close_requested")
            && section.contains("set_on_close_window_requested")
            && section.contains("close_tab_ai_harness_terminal_with_window(window, cx)")
            && section.contains("close_and_reset_window(cx)"),
        "embedded ACP host must distinguish Escape/close from Cmd+W host-window close"
    );
    assert_ordered(
        section,
        "close_tab_ai_harness_terminal_with_window(window, cx)",
        "close_and_reset_window(cx)",
    );
}

#[test]
fn main_native_close_routes_acp_through_lifecycle_close() {
    let section = section_between(
        APP_RUN_SETUP_SOURCE,
        "window.on_window_should_close",
        "// Store the entity for external access",
    );
    assert!(
        section.contains("AppView::AcpChatView")
            && section.contains("embedded_acp_native_close_window")
            && section.contains("SurfaceClosedBySystem")
            && section.contains("SurfaceId::Main")
            && section.contains("false"),
        "main native close must hide the singleton main window while syncing orchestrator state"
    );
    assert_ordered(
        section,
        "close_tab_ai_harness_terminal_with_window(window, cx)",
        "close_and_reset_window(cx)",
    );
}

#[test]
fn embedded_acp_close_helper_tears_down_surface_and_registry() {
    let section = section_between(
        TAB_AI_MODE_SOURCE,
        "fn close_tab_ai_harness_terminal_impl",
        "pub(crate) fn close_tab_ai_harness_terminal_with_window",
    );
    assert!(
        section.contains("closing_acp_chat")
            && section.contains("prepare_for_host_hide")
            && section.contains("embedded_acp_return_origin_self_guarded")
            && section.contains("self.acp_ready_script_path = None")
            && section.contains("rekey_main_automation_surface_from_current_view")
            && section.contains("ensure_embedded_ai_window(false)")
            && section.contains("AcpSurfaceEvent::EmbeddedClosed"),
        "embedded ACP close helper must preserve the view, restore origin, and tear down ACP surface bookkeeping"
    );
    assert_ordered(
        section,
        "prepare_for_host_hide",
        "self.restore_current_view_with_focus(return_view, return_focus_target)",
    );
    assert_ordered(
        section,
        "self.restore_current_view_with_focus(return_view, return_focus_target)",
        "rekey_main_automation_surface_from_current_view",
    );
}

#[test]
fn detached_acp_cmd_w_stays_on_detached_window_path() {
    let section = section_between(
        ACP_VIEW_SOURCE,
        "let is_detached_host = crate::ai::acp::chat_window::is_chat_window(window)",
        "this.handle_key_down(event, window, cx)",
    );
    assert!(
        section.contains("is_chat_window(window)") && section.contains("window.remove_window()"),
        "detached ACP Cmd+W must keep using the detached window close path"
    );
}
