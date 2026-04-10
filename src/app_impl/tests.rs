use super::{calculate_fallback_error_message, ScriptListApp};
use std::fs;

#[test]
fn test_sync_builtin_query_state_updates_query_and_selection_when_changed() {
    let mut query = String::from("old");
    let mut selected_index = 3;

    let changed = ScriptListApp::sync_builtin_query_state(&mut query, &mut selected_index, "new");

    assert!(changed);
    assert_eq!(query, "new");
    assert_eq!(selected_index, 0);
}

#[test]
fn test_sync_builtin_query_state_noop_when_query_is_unchanged() {
    let mut query = String::from("same");
    let mut selected_index = 4;

    let changed = ScriptListApp::sync_builtin_query_state(&mut query, &mut selected_index, "same");

    assert!(!changed);
    assert_eq!(query, "same");
    assert_eq!(selected_index, 4);
}

#[test]
fn test_clear_builtin_query_state_clears_text_and_resets_selection() {
    let mut query = String::from("abc");
    let mut selected_index = 2;

    ScriptListApp::clear_builtin_query_state(&mut query, &mut selected_index);

    assert!(query.is_empty());
    assert_eq!(selected_index, 0);
}

#[test]
fn test_calculate_fallback_error_message_includes_expression_and_recovery() {
    let message = calculate_fallback_error_message("2 + )");
    assert!(message.contains("2 + )"));
    assert!(message.contains("Could not evaluate expression"));
    assert!(message.contains("Check the syntax and try again"));
}

#[test]
fn test_shift_tab_no_longer_routes_through_harness_entry_intent() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        !startup_tab.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "Shift+Tab in ScriptList should no longer route through the quick-submit planner"
    );
}

#[test]
fn test_generate_script_builtin_routes_to_harness_terminal() {
    let builtin_execution = fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("Failed to read src/app_execute/builtin_execution.rs");

    assert!(
        builtin_execution.contains("open_tab_ai_chat_with_entry_intent(Some(trimmed), cx)"),
        "Generate Script built-in should route to harness terminal with entry intent"
    );
    assert!(
        builtin_execution.contains("ai_generate_script_routed_to_harness"),
        "Generate Script success label should indicate harness routing"
    );
}

#[test]
fn test_cmd_enter_routes_to_harness_terminal_in_startup_new_tab() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("try_route_global_cmd_enter_to_acp_context_capture"),
        "Cmd+Enter in startup_new_tab.rs should route through the global AI-entry helper"
    );
}

#[test]
fn test_tab_interceptor_matches_tab_key_case_insensitive() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("eq_ignore_ascii_case(\"tab\")"),
        "Tab interceptor should match tab key case-insensitively"
    );
}

#[test]
fn plain_tab_in_script_list_routes_to_acp_in_startup_new_tab() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("matches!(this.current_view, AppView::ScriptList)")
            && startup_tab.contains("this.open_tab_ai_acp_with_entry_intent(entry_intent, cx);"),
        "Plain Tab in ScriptList must route through the ACP entry path in startup_new_tab.rs"
    );
}

#[test]
fn test_generate_script_from_current_app_routes_to_harness() {
    let builtin_execution = fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("Failed to read src/app_execute/builtin_execution.rs");

    assert!(
        builtin_execution
            .contains("AiCommandType::GenerateScriptFromCurrentApp"),
        "GenerateScriptFromCurrentApp must be handled in builtin_execution.rs"
    );

    assert!(
        builtin_execution.contains("ai_{cmd_type:?}_routed_to_harness"),
        "GenerateScriptFromCurrentApp must route to harness terminal"
    );
}

#[test]
fn test_emoji_picker_arrow_interceptor_consumes_left_right_keys_before_input() {
    let startup = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");

    assert!(
        startup.contains("let is_left = crate::ui_foundation::is_key_left(key);")
            && startup.contains("let is_right = crate::ui_foundation::is_key_right(key);"),
        "arrow interceptor should normalize both left/right key variants"
    );

    assert!(
        startup.contains("if (is_left || is_right) && no_direction_modifiers"),
        "arrow interceptor should explicitly handle plain left/right arrows"
    );

    let horizontal_interceptor_start = startup
        .find("if (is_left || is_right) && no_direction_modifiers")
        .expect("left/right arrow interceptor block should exist in startup.rs");
    let horizontal_interceptor_end = (horizontal_interceptor_start + 2400).min(startup.len());
    let horizontal_interceptor = &startup[horizontal_interceptor_start..horizontal_interceptor_end];
    assert!(
        horizontal_interceptor.contains("if let AppView::EmojiPickerView {")
            && horizontal_interceptor.contains("*selected_index = if is_left")
            && horizontal_interceptor.contains("cx.stop_propagation();"),
        "EmojiPickerView left/right handling must navigate and stop propagation to Input"
    );
}

// ---------------------------------------------------------------------------
// Tab AI harness routing contract tests (startup.rs parity)
// ---------------------------------------------------------------------------

#[test]
fn script_list_shift_tab_no_longer_routes_into_harness_entry_intent_in_standard_startup() {
    let source = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");

    assert!(
        !source.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "Shift+Tab in ScriptList must no longer route the filter text through the quick-submit planner"
    );
}

#[test]
fn quick_terminal_tab_is_written_directly_to_pty_in_standard_startup() {
    let source = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");

    assert!(
        source.contains("b\"\\t\""),
        "QuickTerminal must forward Tab directly to the PTY"
    );
    assert!(
        source.contains("b\"\\x1b[Z\""),
        "QuickTerminal must forward Shift+Tab/backtab directly to the PTY"
    );
    assert!(
        source.contains("term.terminal.input(bytes)"),
        "QuickTerminal Tab handling must write raw bytes to the PTY"
    );
}

#[test]
fn plain_tab_in_script_list_routes_to_acp_in_standard_startup() {
    let source = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");

    assert!(
        source.contains("matches!(this.current_view, AppView::ScriptList)")
            && source.contains("this.try_route_plain_tab_to_acp_context_capture(cx)"),
        "Plain Tab in ScriptList must route through the ACP handoff helper in startup.rs"
    );
}

#[test]
fn plain_tab_routes_raw_launcher_text_and_submits_to_detached_acp() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("self.filter_text.trim()"),
        "Plain Tab ACP helper must derive the entry intent from raw ScriptList filter text"
    );
    assert!(
        source.contains("self.open_tab_ai_acp_with_options(entry_intent, true, cx);"),
        "Plain Tab ACP helper must suppress focused-choice staging on non-detached ACP launches"
    );
    assert!(
        source.contains("get_detached_acp_view_entity()")
            && source.contains("thread.submit_input(cx)"),
        "Plain Tab ACP helper must submit launcher text to an existing detached ACP chat"
    );
}

#[test]
fn plain_tab_suppresses_focused_part_staging_but_cmd_enter_does_not() {
    let source = fs::read_to_string("src/app_impl/tab_ai_mode.rs")
        .expect("Failed to read src/app_impl/tab_ai_mode.rs");

    assert!(
        source.contains("suppress_focused_part: bool"),
        "Tab AI launch requests must carry a focused-part suppression flag"
    );
    assert!(
        source.contains("if suppress_focused_part {")
            && source.contains("request.suppress_focused_part"),
        "Focused-part routing must honor the suppression flag during ACP launch"
    );
    assert!(
        source.contains("self.open_tab_ai_acp_with_options(entry_intent, true, cx);")
            && source.contains("self.open_tab_ai_acp_with_options(entry_intent, false, cx);"),
        "Plain Tab should suppress focused-choice staging while the shared ACP entry path should not"
    );
}

#[test]
fn legacy_simulate_key_tab_uses_plain_tab_acp_helper() {
    let runtime_match = fs::read_to_string("src/main_entry/runtime_stdin_match_simulate_key.rs")
        .expect("Failed to read src/main_entry/runtime_stdin_match_simulate_key.rs");
    let app_run_setup = fs::read_to_string("src/main_entry/app_run_setup.rs")
        .expect("Failed to read src/main_entry/app_run_setup.rs");

    assert!(
        runtime_match.contains("try_route_plain_tab_to_acp_context_capture")
            && app_run_setup.contains("try_route_plain_tab_to_acp_context_capture"),
        "Legacy stdin simulateKey Tab path must reuse the plain Tab ACP helper in both include sources"
    );
}

#[test]
fn confirm_popup_guards_global_launcher_interceptors() {
    let startup = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");
    let startup_new_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");
    let startup_new_arrow = fs::read_to_string("src/app_impl/startup_new_arrow.rs")
        .expect("Failed to read src/app_impl/startup_new_arrow.rs");
    let startup_new_actions = fs::read_to_string("src/app_impl/startup_new_actions.rs")
        .expect("Failed to read src/app_impl/startup_new_actions.rs");

    for (label, source) in [
        ("startup.rs", startup.as_str()),
        ("startup_new_tab.rs", startup_new_tab.as_str()),
        ("startup_new_arrow.rs", startup_new_arrow.as_str()),
        ("startup_new_actions.rs", startup_new_actions.as_str()),
    ] {
        assert!(
            source.contains("confirm::consume_main_window_key_while_confirm_open("),
            "{label} must guard global key interceptors while a confirm popup is open"
        );
    }
}

#[test]
fn render_impl_routes_modifier_aware_keys_into_confirm_popup_guard() {
    let source = fs::read_to_string("src/main_sections/render_impl.rs")
        .expect("Failed to read src/main_sections/render_impl.rs");

    assert!(
        source.contains("confirm::consume_main_window_key_while_confirm_open("),
        "render_impl.rs must route root key capture through the confirm popup guard"
    );
    assert!(
        source.contains("&event.keystroke.modifiers"),
        "render_impl.rs must pass real modifiers so Shift+Tab stays inside the confirm popup"
    );
}
