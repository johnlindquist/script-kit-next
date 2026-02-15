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
fn test_shift_tab_routes_to_shared_ai_script_generation_handler() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("if has_shift")
            && startup_tab.contains("this.dispatch_ai_script_generation_from_query(query, cx);"),
        "Shift+Tab in ScriptList should route to dispatch_ai_script_generation_from_query. \
         Missing expected branch in startup_new_tab.rs"
    );
}

#[test]
fn test_generate_script_builtin_routes_to_shared_ai_script_generation_handler() {
    let builtin_execution = fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("Failed to read src/app_execute/builtin_execution.rs");

    assert!(
        builtin_execution.contains("self.dispatch_ai_script_generation_from_query(query, cx);"),
        "Generate Script built-in should route to dispatch_ai_script_generation_from_query"
    );
}

#[test]
fn test_tab_still_routes_to_inline_ai_chat_in_script_list_tab_interceptor() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("this.show_inline_ai_chat(Some(query), cx);"),
        "Tab in ScriptList should continue routing to show_inline_ai_chat"
    );
}

#[test]
fn test_tab_interceptor_matches_both_tab_key_variants() {
    let startup_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    assert!(
        startup_tab.contains("matches!(event.keystroke.key.as_str(), \"tab\" | \"Tab\")"),
        "Tab interceptor should match both tab key variants: \"tab\" and \"Tab\""
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
