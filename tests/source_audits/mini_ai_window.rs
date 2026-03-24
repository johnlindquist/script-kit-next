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
