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
fn test_shift_tab_routes_through_harness_entry_intent() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("if has_shift")
            && startup_tab
                .contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "Shift+Tab in ScriptList should route through the quick-submit planner. \
         Missing expected branch in startup_new_tab.rs"
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
fn test_tab_routes_to_harness_terminal_in_startup_new_tab() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("open_tab_ai_chat(cx)")
            || startup_tab.contains("open_tab_ai_chat_with_entry_intent("),
        "Tab in startup_new_tab.rs should route to the harness terminal"
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
fn script_list_shift_tab_routes_into_harness_entry_intent_in_standard_startup() {
    let source = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");

    assert!(
        source.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "Shift+Tab in ScriptList must route the filter text through the quick-submit planner"
    );
    assert!(
        !source.contains("dispatch_ai_script_generation_from_query(query, cx)"),
        "Standard startup must not keep the legacy Shift+Tab script-generation path"
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
fn acp_escape_only_unwinds_chat_when_actions_popup_is_closed() {
    for file in ["src/app_impl/startup.rs", "src/app_impl/startup_new_actions.rs"] {
        let source = fs::read_to_string(file)
            .unwrap_or_else(|_| panic!("Failed to read {file}"));
        let escape_block_start = source
            .find("// Handle Escape for AcpChatView (return to main menu)")
            .unwrap_or_else(|| panic!("ACP escape block not found in {file}"));
        let escape_block_end = (escape_block_start + 500).min(source.len());
        let escape_block = &source[escape_block_start..escape_block_end];

        assert!(
            escape_block.contains("!this.show_actions_popup"),
            "ACP escape block must defer to the actions dialog while it is open in {file}"
        );
        assert!(
            escape_block.contains("this.close_tab_ai_harness_terminal(cx);"),
            "ACP escape block must still close the ACP chat when actions are not open in {file}"
        );
    }
}

#[test]
fn simulated_acp_escape_closes_actions_before_unwinding_chat() {
    let source = fs::read_to_string("src/main_entry/app_run_setup.rs")
        .expect("Failed to read src/main_entry/app_run_setup.rs");
    let acp_block_start = source
        .find("AppView::AcpChatView { ref entity, .. } => {")
        .expect("ACP simulateKey branch not found in app_run_setup.rs");
    let acp_block_end = (acp_block_start + 2200).min(source.len());
    let acp_block = &source[acp_block_start..acp_block_end];

    let close_actions_pos = acp_block
        .find("view.close_actions_popup(ActionsDialogHost::AcpChat, window, ctx);")
        .expect("simulateKey ACP branch must close ACP actions popup");
    let close_chat_pos = acp_block
        .find("view.close_tab_ai_harness_terminal(ctx);")
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
