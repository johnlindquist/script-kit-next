//! Contract tests for quick terminal mouse-wheel scrolling and modern interactions.
//!
//! These source-contract tests lock the presence of wheel scrolling, text selection,
//! copy/paste, and scroll indicator behavior in the quick terminal (TermPrompt).

const TERM_PROMPT_SOURCE: &str = include_str!("../src/term_prompt/mod.rs");
const TERMINAL_CREATION_SOURCE: &str = include_str!("../src/terminal/alacritty/handle_creation.rs");
const QUICK_TERMINAL_WARM_SOURCE: &str = include_str!("../src/app_impl/quick_terminal_warm.rs");
const THEME_FOCUS_SOURCE: &str = include_str!("../src/app_impl/theme_focus.rs");
const THEME_CHOOSER_SOURCE: &str = include_str!("../src/render_builtins/theme_chooser.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const RENDER_TERM_PROMPT_SOURCE: &str = include_str!("../src/render_prompts/term.rs");
const PROMPT_HANDLER_SOURCE: &str = include_str!("../src/prompt_handler/mod.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/agent_handoff/mod.rs");
const SIMULATE_KEY_DISPATCH_SOURCE: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");
const UTILITY_VIEWS_SOURCE: &str = include_str!("../src/app_execute/utility_views.rs");

fn source_block_after<'a>(source: &'a str, needle: &str, len: usize) -> &'a str {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find `{needle}`"));
    &source[start..source.len().min(start + len)]
}

#[test]
fn quick_terminal_mouse_wheel_and_modern_interaction_contract() {
    // --- Mouse wheel scrolling ---
    assert!(
        TERM_PROMPT_SOURCE.contains(".on_scroll_wheel("),
        "TermPrompt must register a mouse-wheel handler"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.scroll(whole_lines);"),
        "mouse-wheel handler must forward wheel movement into terminal scrollback"
    );
    // The scroll handler must trigger a view refresh after scrolling
    assert!(
        TERM_PROMPT_SOURCE.contains("cx.notify();"),
        "mouse-wheel handler must notify the view after scrolling"
    );

    // --- Copy / paste ---
    assert!(
        TERM_PROMPT_SOURCE.contains("if has_meta && key_str == \"c\""),
        "Cmd+C copy/SIGINT behavior must remain implemented"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("if has_meta && key_str == \"v\""),
        "Cmd+V paste behavior must remain implemented"
    );

    // --- Mouse text selection ---
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.start_selection(col, row);"),
        "single-click drag selection must remain implemented"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.start_semantic_selection(col, row);"),
        "double-click word selection must remain implemented"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("this.terminal.start_line_selection(col, row);"),
        "triple-click line selection must remain implemented"
    );

    // --- Scroll position indicator ---
    assert!(
        TERM_PROMPT_SOURCE.contains("let scroll_offset = self.terminal.display_offset();"),
        "scrollback indicator must remain implemented"
    );
}

#[test]
fn quick_terminal_theme_respects_light_dark_contract() {
    assert!(
        TERMINAL_CREATION_SOURCE.contains("pub fn new_with_theme("),
        "TerminalHandle must expose themed shell creation"
    );
    assert!(
        TERMINAL_CREATION_SOURCE.contains("pub fn with_command_and_theme("),
        "TerminalHandle must expose themed command creation"
    );
    assert!(
        TERMINAL_CREATION_SOURCE.contains(".map(ThemeAdapter::from_theme)"),
        "terminal creation must build its adapter from the active Script Kit theme"
    );

    assert!(
        TERM_PROMPT_SOURCE.contains("TerminalHandle::new_with_theme("),
        "TermPrompt must pass its theme into new terminal creation"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("TerminalHandle::with_command_and_theme("),
        "TermPrompt must pass its theme into command terminal creation"
    );
    assert!(
        TERM_PROMPT_SOURCE.contains("terminal.update_theme(&theme);"),
        "warm PTYs attached to TermPrompt must be rethemed before render"
    );
    assert!(
        QUICK_TERMINAL_WARM_SOURCE.contains("TerminalHandle::new_with_theme("),
        "Quick Terminal warm PTYs must be prewarmed with the current theme"
    );
    assert!(
        QUICK_TERMINAL_WARM_SOURCE.contains("SCRIPT_KIT_DISABLE_QUICK_TERMINAL_WARM_PTY")
            && QUICK_TERMINAL_WARM_SOURCE.contains("reason = \"disabled_by_env\""),
        "Quick Terminal warm PTY startup must have an env opt-out before spawning a shell"
    );
    assert!(
        THEME_FOCUS_SOURCE.contains("pub(crate) fn sync_open_terminal_theme("),
        "theme changes must have a terminal propagation helper"
    );
    assert!(
        THEME_CHOOSER_SOURCE.contains("self.sync_open_terminal_theme(cx);"),
        "theme chooser previews and restores must propagate to an open terminal"
    );
}

#[test]
fn quick_terminal_native_footer_does_not_capture_sdk_term_prompt_footer() {
    assert!(
        APP_VIEW_STATE_SOURCE
            .contains("AppView::QuickTerminalView { .. } => Some(\"quick_terminal\")"),
        "QuickTerminalView must register the native footer surface"
    );
    assert!(
        !APP_VIEW_STATE_SOURCE.contains("AppView::TermPrompt { .. } => Some(\"term_prompt\")"),
        "SDK TermPrompt must not register a native footer surface; it keeps the GPUI terminal hint strip"
    );
    assert!(
        RENDER_TERM_PROMPT_SOURCE.contains("render_terminal_prompt_hint_strip(None, None)"),
        "non-quick terminal prompts must keep the route-aware GPUI hint strip"
    );
    assert!(
        RENDER_TERM_PROMPT_SOURCE.contains("\"native_footer_spacer\"")
            && UI_WINDOW_SOURCE.contains("render_native_main_window_footer_spacer()"),
        "Quick Terminal must reserve space for the native AppKit footer through the shared footer slot"
    );
}

#[test]
fn quick_terminal_apply_keyboard_and_footer_share_visibility_predicate() {
    assert!(
        UI_WINDOW_SOURCE.contains("pub(crate) fn quick_terminal_can_apply_back(&self) -> bool"),
        "Quick Terminal must expose one apply-back predicate"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("let can_apply = self.quick_terminal_can_apply_back();"),
        "Quick Terminal footer buttons must use the shared apply-back predicate"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("pub(crate) fn dispatch_quick_terminal_cmd_enter")
            && TAB_AI_MODE_SOURCE.contains("self.quick_terminal_can_apply_back()"),
        "Quick Terminal Cmd+Enter must guard apply-back through the shared dispatcher"
    );
}

#[test]
fn quick_terminal_cmd_enter_uses_single_dispatch_helper() {
    assert!(
        RENDER_TERM_PROMPT_SOURCE.contains("dispatch_quick_terminal_cmd_enter("),
        "Quick Terminal Cmd+Enter must delegate to the shared app dispatcher"
    );
    assert!(
        SIMULATE_KEY_DISPATCH_SOURCE.contains("dispatch_quick_terminal_cmd_enter("),
        "simulateKey QuickTerminal Cmd+Enter must use the same dispatcher"
    );
}

#[test]
fn quick_terminal_cmd_enter_preserves_apply_back_when_available() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("pub(crate) fn dispatch_quick_terminal_cmd_enter")
            && TAB_AI_MODE_SOURCE.contains("self.attach_quick_terminal_output_to_portal_origin")
            && TAB_AI_MODE_SOURCE.contains("self.quick_terminal_can_apply_back()")
            && TAB_AI_MODE_SOURCE.contains("self.apply_tab_ai_result_from_terminal(entity")
            && TAB_AI_MODE_SOURCE.contains("self.open_agent_chat_with_quick_terminal_output(entity"),
        "Quick Terminal Cmd+Enter must attach portal output first, Apply when Apply is available, otherwise attach output to Agent Chat"
    );
}

#[test]
fn quick_terminal_portal_handoff_returns_output_to_attachment_origin() {
    let handoff = source_block_after(
        TAB_AI_MODE_SOURCE,
        "pub(crate) fn attach_quick_terminal_output_to_portal_origin",
        1600,
    );
    assert!(
        handoff.contains("PortalKind::Terminal")
            && handoff.contains("quick_terminal_context_part_from_entity")
            && handoff.contains("term.terminate_session()")
            && handoff.contains("self.close_attachment_portal_with_part(part, cx);"),
        "@Terminal Quick Terminal Cmd+Enter must capture output, tear down the PTY, and attach back through the portal origin"
    );
}

#[test]
fn quick_terminal_cwd_sync_updates_shared_spine_cwd() {
    assert!(
        TERM_PROMPT_SOURCE.contains("SCRIPT_KIT_CWD_TITLE_PREFIX")
            && TERM_PROMPT_SOURCE.contains("pub fn synced_cwd_path(&self)"),
        "TermPrompt must expose the Quick Terminal cwd title signal"
    );
    assert!(
        UTILITY_VIEWS_SOURCE.contains("quick_terminal_cwd_sync_hook")
            && UTILITY_VIEWS_SOURCE.contains("add-zsh-hook precmd __skgpui_cwd_sync")
            && UTILITY_VIEWS_SOURCE.contains("PROMPT_COMMAND")
            && UTILITY_VIEWS_SOURCE.contains("sync_spine_cwd_from_quick_terminal"),
        "Quick Terminal must install shell cwd hooks and sync them into the shared Agent Chat cwd"
    );
}

#[test]
fn quick_terminal_output_capture_records_terminal_history() {
    let capture = source_block_after(
        TAB_AI_MODE_SOURCE,
        "fn quick_terminal_context_part_from_entity",
        1300,
    );
    assert!(
        capture.contains("crate::terminal_history::record")
            && capture.contains("TerminalHistoryEntry")
            && capture.contains("AiContextPart::TextBlock"),
        "Quick Terminal transcript capture must both attach a TextBlock and retain terminal history"
    );
}

#[test]
fn quick_terminal_context_capture_pumps_pending_pty_output_before_snapshot() {
    let capture = source_block_after(
        TERM_PROMPT_SOURCE,
        "pub fn terminal_context_capture(&mut self)",
        1300,
    );
    assert!(
        capture.contains("self.terminal.process()")
            && capture.contains("self.has_received_output = true")
            && capture.contains("TerminalEvent::Title(title)")
            && capture.contains("TerminalEvent::Exit(code)"),
        "Quick Terminal context capture must process pending PTY output before snapshotting text"
    );
}

#[test]
fn quick_terminal_footer_offers_agent_when_apply_unavailable() {
    assert!(
        UI_WINDOW_SOURCE.contains("quick_terminal_can_attach_to_agent_chat")
            && UI_WINDOW_SOURCE.contains("FooterAction::Ai")
            && UI_WINDOW_SOURCE.contains("\"⌘↩\"")
            && UI_WINDOW_SOURCE.contains("\"Agent\""),
        "Standalone Quick Terminal footer must expose Cmd+Enter Agent handoff when Apply is unavailable"
    );
}

#[test]
fn quick_terminal_agent_footer_is_semantically_activatable() {
    assert!(
        PROMPT_HANDLER_SOURCE.contains("footer:quick_terminal:ai")
            && PROMPT_HANDLER_SOURCE.contains("footer:prompt:ai")
            && PROMPT_HANDLER_SOURCE.contains("open_agent_chat_with_quick_terminal_output"),
        "Quick Terminal Agent footer must be activatable through selectBySemanticId for runtime proof"
    );
}

#[test]
fn quick_terminal_set_input_writes_to_pty_for_runtime_proof() {
    assert!(
        PROMPT_HANDLER_SOURCE.contains("AppView::QuickTerminalView { entity }")
            && PROMPT_HANDLER_SOURCE.contains("term.send_raw_input(&payload)")
            && TERM_PROMPT_SOURCE.contains("pub fn send_raw_input"),
        "batch.setInput must write to the Quick Terminal PTY so runtime tests can drive terminal output"
    );
}

#[test]
fn quick_terminal_agent_handoff_does_not_use_clipboard_priming() {
    let handoff = source_block_after(
        TAB_AI_MODE_SOURCE,
        "pub(crate) fn open_agent_chat_with_quick_terminal_output",
        1200,
    );
    assert!(
        !handoff.contains("prime_apply_clipboard")
            && !handoff.contains("read_tab_ai_apply_back_clipboard_text")
            && !handoff.contains("Clipboard::new"),
        "Terminal output Agent handoff must use direct terminal capture, not clipboard"
    );
}

#[test]
fn quick_terminal_keyboard_and_footer_close_share_state_first_close() {
    assert!(
        RENDER_TERM_PROMPT_SOURCE
            .contains("this.close_quick_terminal_main_window_state_first(cx);"),
        "Quick Terminal Cmd+W must use the state-first close helper"
    );
    assert!(
        UI_WINDOW_SOURCE.contains("self.close_quick_terminal_main_window_state_first(cx);"),
        "Quick Terminal footer Close must share the state-first close helper advertised by Cmd+W"
    );
    assert!(
        !UI_WINDOW_SOURCE.contains("event = \"quick_terminal_footer_close\",\n                        \"Closing quick terminal from native footer\"\n                    );\n                    self.close_tab_ai_harness_terminal_with_window"),
        "Quick Terminal footer Close must not accidentally diverge back to restore-origin close"
    );
}

#[test]
fn quick_terminal_native_footer_close_is_semantically_selectable_for_agentic_proof() {
    assert!(
        PROMPT_HANDLER_SOURCE.contains("semantic_id == \"footer:native:close\"")
            && PROMPT_HANDLER_SOURCE.contains("AppView::QuickTerminalView { .. }")
            && PROMPT_HANDLER_SOURCE.contains("self.close_quick_terminal_main_window_state_first(cx);"),
        "Agentic selectBySemanticId proof must be able to activate Quick Terminal native footer Close"
    );
}
