//! Source-level contract for the native main-window footer surface owner.

const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const DICTATION_WINDOW_SOURCE: &str = include_str!("../src/dictation/window.rs");
const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");
const RENDER_PROMPTS_OTHER_SOURCE: &str = include_str!("../src/render_prompts/other.rs");
const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const RUNTIME_STDIN_MATCH_SIMULATE_KEY_SOURCE: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_start = &source[start..];
    let open = after_start
        .find('{')
        .unwrap_or_else(|| panic!("missing function body for: {signature}"));
    let mut depth = 0usize;
    for (offset, ch) in after_start[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after_start[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

// doc-anchor-removed: [[removed-docs contract]]
#[test]
fn app_view_owns_native_footer_surface_map() {
    let body = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    for expected in [
        "AppView::ScriptList => Some(\"script_list\")",
        "AppView::QuickTerminalView { .. } => Some(\"quick_terminal\")",
        "AppView::AcpChatView { .. } => Some(\"acp_chat\")",
        "AppView::ScriptIssuesView { .. } => Some(\"script_issues\")",
        "AppView::ConfirmPrompt { .. } => Some(\"confirm_prompt\")",
        "AppView::TermPrompt { .. }",
        "AppView::MicroPrompt { .. }",
    ] {
        assert!(
            body.contains(expected),
            "AppView::native_footer_surface must declare footer ownership for `{expected}`"
        );
    }
    assert!(
        !body.contains("_ => None"),
        "native footer ownership must remain explicit so new AppView variants cannot inherit footer behavior silently"
    );
}

#[test]
fn script_issues_view_keeps_native_footer_and_fix_in_agent_primary_action() {
    let footer_map = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    assert!(
        footer_map.contains("AppView::ScriptIssuesView { .. } => Some(\"script_issues\")"),
        "ScriptIssuesView must keep the main-window native footer active"
    );

    let footer_buttons = function_body(
        UI_WINDOW_SOURCE,
        "fn main_window_footer_buttons_for_current_view",
    );
    assert!(
        footer_buttons.contains("AppView::ScriptIssuesView { .. }")
            && footer_buttons
                .contains("FooterButtonConfig::new(FooterAction::Run, \"↵\", \"Fix in Agent\")")
            && footer_buttons
                .contains("FooterButtonConfig::new(FooterAction::Apply, \"⌘C\", \"Copy Issues\")"),
        "ScriptIssuesView footer must expose Fix in Agent and Copy Issues"
    );

    let footer_dispatch = function_body(
        UI_WINDOW_SOURCE,
        "pub(crate) fn dispatch_main_window_footer_action",
    );
    assert!(
        footer_dispatch.contains("AppView::ScriptIssuesView { report }")
            && footer_dispatch.contains("self.fix_script_issues_in_agent(&report, cx);"),
        "ScriptIssuesView footer Run must submit diagnostics to Agent Chat"
    );

    assert!(
        footer_dispatch.contains("AppView::ScriptIssuesView { report }")
            && footer_dispatch.contains("self.copy_script_issues_to_clipboard(&report, cx);"),
        "ScriptIssuesView footer Copy Issues must copy diagnostics"
    );
}

#[test]
fn script_issues_enter_routes_to_agent_chat_prompt_submission() {
    assert!(
        RENDER_PROMPTS_OTHER_SOURCE.contains("pub(crate) fn format_script_issues_agent_prompt")
            && RENDER_PROMPTS_OTHER_SOURCE.contains("Self::format_script_issues_diagnostics(report)")
            && RENDER_PROMPTS_OTHER_SOURCE.contains("open_tab_ai_acp_with_entry_intent_suppressing_focused_part(Some(prompt), cx)"),
        "Script issues Agent handoff must include the formatted diagnostics and suppress focused context"
    );

    assert!(
        STARTUP_SOURCE.contains("if let AppView::ScriptIssuesView { report } = &this.current_view")
            && STARTUP_SOURCE.contains("this.fix_script_issues_in_agent(&report, cx);")
            && STARTUP_SOURCE.contains("cx.stop_propagation();"),
        "physical Enter must route ScriptIssuesView to Fix in Agent before generic key handling"
    );

    assert!(
        RUNTIME_STDIN_MATCH_SIMULATE_KEY_SOURCE.contains("AppView::ScriptIssuesView { report }")
            && RUNTIME_STDIN_MATCH_SIMULATE_KEY_SOURCE
                .contains("SimulateKey: Enter - fix script issues in Agent Chat")
            && RUNTIME_STDIN_MATCH_SIMULATE_KEY_SOURCE
                .contains("view.fix_script_issues_in_agent(&report, ctx);"),
        "simulateKey Enter must keep parity with physical Enter for ScriptIssuesView"
    );

    let render_script_issues = function_body(
        RENDER_PROMPTS_OTHER_SOURCE,
        "pub(crate) fn render_script_issues_view",
    );
    assert!(
        render_script_issues.contains("has_cmd && key.eq_ignore_ascii_case(\"w\")")
            && render_script_issues.contains("this.go_back_or_close(window, cx);"),
        "ScriptIssuesView must handle Cmd+W as a close/back shortcut"
    );
}

// doc-anchor-removed: [[removed-docs contract]]
#[test]
fn ui_window_delegates_footer_surface_to_app_view_contract() {
    let body = function_body(UI_WINDOW_SOURCE, "fn main_window_footer_surface");
    assert!(
        body.contains("self.current_view.native_footer_surface()"),
        "ui_window must delegate footer surface identity to AppView::native_footer_surface"
    );
    assert!(
        !body.contains("match &self.current_view"),
        "ui_window must not duplicate the AppView footer surface map"
    );
}

#[test]
fn live_dictation_overlay_does_not_join_main_window_footer_ownership() {
    let footer_map = function_body(APP_VIEW_STATE_SOURCE, "pub(crate) fn native_footer_surface");
    assert!(
        footer_map.contains("AppView::DictationHistoryView { .. } => Some(\"dictation_history\")"),
        "DictationHistoryView is the main-window dictation history footer surface"
    );
    assert!(
        !footer_map.contains("dictation_overlay"),
        "Live DictationOverlay must not be owned by AppView::native_footer_surface"
    );
    assert!(
        DICTATION_WINDOW_SOURCE.contains("DICTATION_OVERLAY_FOOTER_SURFACE")
            && DICTATION_WINDOW_SOURCE.contains("\"dictation_overlay\""),
        "Dictation overlay may keep a local footer identity for tests/logging"
    );
    assert!(
        DICTATION_WINDOW_SOURCE.contains("dictation_footer_action_channel")
            && DICTATION_WINDOW_SOURCE.contains("MainWindowFooterConfig")
            && !DICTATION_WINDOW_SOURCE.contains("active_main_window_footer_surface")
            && DICTATION_WINDOW_SOURCE.contains("FooterAction::Stop")
            && DICTATION_WINDOW_SOURCE.contains("FooterAction::Close"),
        "Dictation overlay must reuse the native footer renderer without joining main-window surface ownership"
    );
}

#[test]
fn experimental_gpui_footer_overlay_keeps_native_material_as_background_only() {
    assert!(
        FOOTER_POPUP_SOURCE.contains("struct GpuiFooterOverlay")
            && FOOTER_POPUP_SOURCE.contains("render_footer_hint_content_constrained(")
            && FOOTER_POPUP_SOURCE.contains("left_pinned_buttons")
            && FOOTER_POPUP_SOURCE.contains("trailing_buttons")
            && FOOTER_POPUP_SOURCE.contains("render_left_info")
            && FOOTER_POPUP_SOURCE.contains("SCRIPT_KIT_GPUI_FOOTER_OVERLAY_SPIKE"),
        "footer overlay spike must be explicit, config-driven, and reuse the GPUI footer chrome renderer"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("WindowBackgroundAppearance::Transparent")
            && FOOTER_POPUP_SOURCE.contains("attach_inline_popup_to_parent_window")
            && FOOTER_POPUP_SOURCE.contains("setIgnoresMouseEvents: NO")
            && FOOTER_POPUP_SOURCE.contains("setBecomesKeyOnlyIfNeeded: YES")
            && FOOTER_POPUP_SOURCE.contains("send_footer_action_to_channel(action, false)"),
        "GPUI footer overlay spike must be a transparent child surface that owns hover/click without stealing key focus"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("layout_footer_hints(hints_view, text_color, &[], &theme)")
            && FOOTER_POPUP_SOURCE.contains("layout_footer_hints(hints_view, text_color, &config.buttons, &theme)"),
        "AppKit footer text must be bypassed only while the experimental GPUI overlay spike is enabled"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("layout_footer_left_info(left_info_view, None, text_color)")
            && FOOTER_POPUP_SOURCE.contains(
                "layout_footer_left_info(left_info_view, config.left_info.as_ref(), text_color)"
            ),
        "AppKit left-info text must also be bypassed only while GPUI overlay owns footer visuals"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("footer_overlay_button_width_px")
            && FOOTER_POPUP_SOURCE.contains("overlay_width_px")
            && FOOTER_POPUP_SOURCE.contains(".min_w(px(slot_width))")
            && FOOTER_POPUP_SOURCE.contains(".max_w(px(slot_width))")
            && FOOTER_POPUP_SOURCE.contains(".overflow_hidden()")
            && FOOTER_POPUP_SOURCE.contains(".flex_1()")
            && FOOTER_POPUP_SOURCE.contains(".flex_none()"),
        "GPUI footer overlay buttons must allocate fixed slots and truncate before neighboring chips overlap"
    );
}
