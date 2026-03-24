//! Source audits for the Mini AI Window feature.
//!
//! These tests verify the mini AI shell IDs, keyboard ordering,
//! structured log contract, and layout constants without launching the UI.

use super::read_source as read;

#[test]
fn mini_shell_exposes_machine_addressable_ids() {
    let root = read("src/ai/window/render_root.rs");
    for id in [
        "ai-titlebar-mini",
        "ai-mini-recent",
        "ai-mini-new",
        "ai-mini-actions",
        "ai-mini-history-overlay",
        "ai-mini-model-name",
        "ai-mini-history-backdrop",
    ] {
        assert!(
            root.contains(id),
            "render_root.rs missing expected element ID: {id}"
        );
    }
}

#[test]
fn mini_main_panel_exposes_compact_composer_ids() {
    let panel = read("src/ai/window/render_main_panel.rs");
    for id in [
        "ai-main-panel-mini",
        "ai-mini-input-area",
        "ai-mini-composer",
        "ai-mini-submit-btn",
        "ai-mini-stop-btn",
    ] {
        assert!(
            panel.contains(id),
            "render_main_panel.rs missing expected element ID: {id}"
        );
    }
}

#[test]
fn mini_keydown_closes_overlay_before_window_close() {
    let source = read("src/ai/window/render_keydown.rs");
    let overlay_guard =
        "if is_key_escape(key) && self.window_mode.is_mini() && self.showing_mini_history_overlay";
    let final_close = "if is_key_escape(key) && self.window_mode.is_mini() {";
    assert!(
        source.contains(overlay_guard),
        "render_keydown.rs missing overlay-dismiss guard"
    );
    assert!(
        source.contains(final_close),
        "render_keydown.rs missing final mini Escape close"
    );
    assert!(
        source.find(overlay_guard) < source.find(final_close),
        "Overlay dismiss must come BEFORE final window close in keydown handler"
    );
}

#[test]
fn mini_entry_points_pass_explicit_source() {
    let root = read("src/ai/window/render_root.rs");
    assert!(
        root.contains("toggle_mini_history_overlay(\"header_recent_button\""),
        "Recent button click must pass source to toggle_mini_history_overlay"
    );

    let keydown = read("src/ai/window/render_keydown.rs");
    assert!(
        keydown.contains("toggle_mini_history_overlay(\"shortcut_cmd_j\""),
        "Cmd+J shortcut must pass source to toggle_mini_history_overlay"
    );
    assert!(
        keydown.contains("show_command_bar(\"shortcut_cmd_k\""),
        "Cmd+K shortcut must pass source to show_command_bar"
    );
    assert!(
        keydown.contains("show_new_chat_command_bar(\"shortcut_cmd_n\""),
        "Cmd+N shortcut must pass source to show_new_chat_command_bar"
    );

    let cmd_bar = read("src/ai/window/command_bar.rs");
    assert!(
        cmd_bar.contains("toggle_mini_history_overlay(\"simulated_cmd_j\""),
        "Simulated Cmd+J must pass source to toggle_mini_history_overlay"
    );
    assert!(
        cmd_bar.contains("show_command_bar(\"simulated_cmd_k\""),
        "Simulated Cmd+K must pass source to show_command_bar"
    );
}

#[test]
fn mini_ui_logs_use_structured_ai_ui_category() {
    let root = read("src/ai/window/render_root.rs");
    assert!(
        root.contains("category = \"AI_UI\""),
        "render_root.rs must use structured AI_UI category"
    );
    assert!(
        root.contains("\"mini_history_overlay_toggled\""),
        "render_root.rs must log mini_history_overlay_toggled event"
    );
    assert!(
        root.contains("\"mini_history_overlay_dismissed\""),
        "render_root.rs must log mini_history_overlay_dismissed event"
    );

    let cmd_bar = read("src/ai/window/command_bar.rs");
    assert!(
        cmd_bar.contains("category = \"AI_UI\""),
        "command_bar.rs must use structured AI_UI category"
    );
    assert!(
        cmd_bar.contains("event = \"command_bar_open\""),
        "command_bar.rs must log command_bar_open event"
    );
    assert!(
        cmd_bar.contains("event = \"command_bar_close\""),
        "command_bar.rs must log command_bar_close event"
    );
    assert!(
        cmd_bar.contains("event = \"new_chat_menu_open\""),
        "command_bar.rs must log new_chat_menu_open event"
    );
    assert!(
        cmd_bar.contains("event = \"new_chat_action_unresolved\""),
        "command_bar.rs must log new_chat_action_unresolved event"
    );

    let keydown = read("src/ai/window/render_keydown.rs");
    assert!(
        keydown.contains("event = \"mini_escape_close\""),
        "render_keydown.rs must log mini_escape_close event"
    );

    let panel = read("src/ai/window/render_main_panel.rs");
    assert!(
        panel.contains("event = \"mini_submit_click\""),
        "render_main_panel.rs must log mini_submit_click event"
    );
    assert!(
        panel.contains("event = \"mini_stop_click\""),
        "render_main_panel.rs must log mini_stop_click event"
    );
}

#[test]
fn mini_layout_uses_named_constants() {
    let root = read("src/ai/window/render_root.rs");
    assert!(
        root.contains("MINI_TITLEBAR_H"),
        "render_root.rs must use MINI_TITLEBAR_H constant"
    );
    assert!(
        root.contains("MINI_HISTORY_OVERLAY_W"),
        "render_root.rs must use MINI_HISTORY_OVERLAY_W constant"
    );
    assert!(
        root.contains("MINI_HISTORY_OVERLAY_MAX_H"),
        "render_root.rs must use MINI_HISTORY_OVERLAY_MAX_H constant"
    );
    assert!(
        root.contains("MINI_HISTORY_OVERLAY_TOP"),
        "render_root.rs must use MINI_HISTORY_OVERLAY_TOP constant"
    );

    let panel = read("src/ai/window/render_main_panel.rs");
    assert!(
        panel.contains("MINI_CONTENT_MAX_W"),
        "render_main_panel.rs must use MINI_CONTENT_MAX_W constant"
    );
    assert!(
        panel.contains("MINI_BTN_SIZE"),
        "render_main_panel.rs must use MINI_BTN_SIZE constant"
    );
}

#[test]
fn builtin_execution_routes_mini_ai_to_deferred_handoff() {
    let source = read("src/app_execute/builtin_execution.rs");
    assert!(
        source.contains("AiCommandType::MiniAi"),
        "builtin_execution.rs must handle AiCommandType::MiniAi"
    );
    assert!(
        source.contains("open_mini_ai_window"),
        "builtin_execution.rs must call open_mini_ai_window"
    );
}

#[test]
fn mini_overlay_dismiss_restores_focus_and_clears_search() {
    // All three dismiss paths must call focus_input AND clear_search_state:
    // 1. Toggle dismiss (toggle_mini_history_overlay when closing)
    let root = read("src/ai/window/render_root.rs");
    let dismiss_marker = "\"mini_history_overlay_dismissed\"";
    let toggle_fn_start = root
        .find("fn toggle_mini_history_overlay")
        .expect("toggle_mini_history_overlay must exist");
    let toggle_section = &root[toggle_fn_start..toggle_fn_start + 800];
    assert!(
        toggle_section.contains("focus_input(window, cx)"),
        "toggle_mini_history_overlay dismiss path must call focus_input"
    );
    assert!(
        toggle_section.contains("clear_search_state(window, cx)"),
        "toggle_mini_history_overlay dismiss path must clear search"
    );

    // 2. Backdrop click dismiss
    let backdrop_start = root
        .find("ai-mini-history-backdrop")
        .expect("backdrop element must exist");
    let backdrop_section = &root[backdrop_start..backdrop_start + 600];
    assert!(
        backdrop_section.contains("focus_input(window, cx)"),
        "Backdrop click dismiss must call focus_input"
    );
    assert!(
        backdrop_section.contains("clear_search_state(window, cx)"),
        "Backdrop click dismiss must clear search"
    );

    // 3. Escape key dismiss
    let keydown = read("src/ai/window/render_keydown.rs");
    let escape_overlay = keydown
        .find(dismiss_marker)
        .expect("Escape overlay dismiss must log the dismiss event");
    let escape_start = escape_overlay.saturating_sub(200);
    let escape_section = &keydown[escape_start..escape_overlay + 100];
    assert!(
        escape_section.contains("focus_input(window, cx)"),
        "Escape overlay dismiss must call focus_input"
    );
    assert!(
        escape_section.contains("clear_search_state(window, cx)"),
        "Escape overlay dismiss must clear search"
    );
}

#[test]
fn mode_switch_focuses_input_after_switch() {
    // The canonical set_window_mode helper must focus input
    let interactions = read("src/ai/window/interactions.rs");
    let set_fn = interactions
        .find("fn set_window_mode")
        .expect("set_window_mode must exist");
    let fn_end = (set_fn + 2000).min(interactions.len());
    let fn_body = &interactions[set_fn..fn_end];
    assert!(
        fn_body.contains("focus_input(window, cx)"),
        "set_window_mode must focus input after switching modes"
    );

    // toggle_window_mode must delegate to set_window_mode (not duplicate logic)
    let toggle_fn = interactions
        .find("fn toggle_window_mode")
        .expect("toggle_window_mode must exist");
    let toggle_end = (toggle_fn + 500).min(interactions.len());
    let toggle_body = &interactions[toggle_fn..toggle_end];
    assert!(
        toggle_body.contains("self.set_window_mode("),
        "toggle_window_mode must delegate to set_window_mode"
    );

    // SetWindowMode command handler must delegate to set_window_mode (not duplicate logic)
    let root = read("src/ai/window/render_root.rs");
    let set_mode = root
        .find("AiCommand::SetWindowMode(window_mode)")
        .expect("SetWindowMode command handler must exist");
    let handler_end = (set_mode + 200).min(root.len());
    let handler_body = &root[set_mode..handler_end];
    assert!(
        handler_body.contains("self.set_window_mode("),
        "SetWindowMode command handler must delegate to set_window_mode"
    );
}

#[test]
fn mini_cmd_shift_f_opens_history_overlay() {
    let keydown = read("src/ai/window/render_keydown.rs");
    let f_section_start = keydown
        .find("\"f\" => {")
        .expect("Cmd+F handler must exist in render_keydown.rs");
    let f_section = &keydown[f_section_start..(f_section_start + 600).min(keydown.len())];
    assert!(
        f_section.contains("self.window_mode.is_mini()"),
        "Cmd+Shift+F handler must check for mini mode"
    );
    assert!(
        f_section.contains("toggle_mini_history_overlay"),
        "Cmd+Shift+F in mini mode must open the history overlay"
    );
}

#[test]
fn mini_sidebar_shortcuts_are_guarded() {
    let keydown = read("src/ai/window/render_keydown.rs");
    // Cmd+\ and Cmd+B should check for mini mode before toggling sidebar
    let backslash_start = keydown
        .find("\"\\\\\" | \"backslash\"")
        .expect("Cmd+\\ handler must exist");
    let backslash_section =
        &keydown[backslash_start..(backslash_start + 200).min(keydown.len())];
    assert!(
        backslash_section.contains("!self.window_mode.is_mini()"),
        "Cmd+\\ must guard against mini mode before toggling sidebar"
    );

    let b_start = keydown
        .find("// Cmd+B also toggles sidebar")
        .expect("Cmd+B comment must exist");
    let b_section = &keydown[b_start..(b_start + 200).min(keydown.len())];
    assert!(
        b_section.contains("!self.window_mode.is_mini()"),
        "Cmd+B must guard against mini mode before toggling sidebar"
    );
}

#[test]
fn mini_header_shows_chat_title() {
    let root = read("src/ai/window/render_root.rs");
    // Mini titlebar should show chat title when there are messages
    assert!(
        root.contains("ai-mini-title-label"),
        "render_root.rs must have a mini title label element"
    );
    // Should conditionally show title_text vs "AI"
    let mini_titlebar_start = root
        .find("ai-titlebar-mini")
        .expect("mini titlebar must exist");
    let titlebar_section =
        &root[mini_titlebar_start..(mini_titlebar_start + 2000).min(root.len())];
    assert!(
        titlebar_section.contains("title_text"),
        "Mini titlebar should display chat title when messages exist"
    );
}
