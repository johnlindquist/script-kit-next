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
        "ai-mini-expand",
        "ai-mini-streaming-dot",
        "ai-mini-recent",
        "ai-mini-new",
        "ai-mini-actions",
        "ai-mini-history-overlay",
        "ai-mini-history-backdrop",
    ] {
        assert!(
            root.contains(id),
            "render_root.rs missing expected element ID: {id}"
        );
    }

    let input = read("src/ai/window/render_input.rs");
    assert!(
        input.contains("ai-mini-model-chip") || input.contains("ai-mini-model-setup"),
        "render_input.rs must expose the mini model chip or setup fallback ID"
    );
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
    // The mini history overlay key guard intercepts Up/Down/Enter/Esc before other handlers
    let overlay_guard = "if self.window_mode.is_mini() && self.showing_mini_history_overlay";
    let final_close = "if is_key_escape(key) && self.window_mode.is_mini() {";
    assert!(
        source.contains(overlay_guard),
        "render_keydown.rs missing overlay key routing guard"
    );
    assert!(
        source.contains(final_close),
        "render_keydown.rs missing final mini Escape close"
    );
    assert!(
        source.find(overlay_guard) < source.find(final_close),
        "Overlay key routing must come BEFORE final window close in keydown handler"
    );
}

#[test]
fn mini_entry_points_pass_explicit_source() {
    let root = read("src/ai/window/render_root.rs");
    assert!(
        root.contains("toggle_mini_history_overlay(") && root.contains("\"header_recent_button\""),
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
        keydown.contains("self.new_conversation(window, cx)"),
        "Cmd+N shortcut must call new_conversation"
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
fn mini_ui_logs_use_telemetry_helpers() {
    // All mini UI events now route through the telemetry helper module.
    let telemetry = read("src/ai/window/telemetry.rs");
    assert!(
        telemetry.contains("category = \"AI_UI\""),
        "telemetry.rs must define the AI_UI category"
    );
    assert!(
        telemetry.contains("category = \"AI\""),
        "telemetry.rs must define the AI lifecycle category"
    );

    // Render paths pass event names to the telemetry helpers
    let root = read("src/ai/window/render_root.rs");
    assert!(
        root.contains("telemetry::log_ai_ui("),
        "render_root.rs must call log_ai_ui helper"
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
        keydown.contains("telemetry::log_ai_lifecycle("),
        "render_keydown.rs must call log_ai_lifecycle for window close events"
    );

    let panel = read("src/ai/window/render_main_panel.rs");
    assert!(
        panel.contains("\"mini_submit_click\""),
        "render_main_panel.rs must log mini_submit_click event"
    );
    assert!(
        panel.contains("\"mini_stop_click\""),
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
    // The canonical dismiss_mini_history_overlay helper must call focus_input AND
    // clear_search_state. All dismiss paths (toggle, backdrop, Esc) delegate to it.
    let root = read("src/ai/window/render_root.rs");

    // 1. dismiss_mini_history_overlay must contain both calls
    let dismiss_fn_start = root
        .find("fn dismiss_mini_history_overlay")
        .expect("dismiss_mini_history_overlay must exist");
    let dismiss_section = &root[dismiss_fn_start..(dismiss_fn_start + 800).min(root.len())];
    assert!(
        dismiss_section.contains("focus_input(window, cx)"),
        "dismiss_mini_history_overlay must call focus_input"
    );
    assert!(
        dismiss_section.contains("clear_search_state(window, cx)"),
        "dismiss_mini_history_overlay must clear search"
    );

    // 2. Toggle delegates to dismiss helper
    let toggle_fn_start = root
        .find("fn toggle_mini_history_overlay")
        .expect("toggle_mini_history_overlay must exist");
    let toggle_section = &root[toggle_fn_start..(toggle_fn_start + 600).min(root.len())];
    assert!(
        toggle_section.contains("self.dismiss_mini_history_overlay(source, window, cx)"),
        "toggle must delegate to dismiss_mini_history_overlay"
    );

    // 3. Backdrop click delegates to dismiss helper
    let backdrop_start = root
        .find("ai-mini-history-backdrop")
        .expect("backdrop element must exist");
    let backdrop_section = &root[backdrop_start..(backdrop_start + 600).min(root.len())];
    assert!(
        backdrop_section.contains("dismiss_mini_history_overlay(\"backdrop_click\""),
        "Backdrop click must call dismiss_mini_history_overlay"
    );

    // 4. Escape key delegates to dismiss helper
    let keydown = read("src/ai/window/render_keydown.rs");
    assert!(
        keydown.contains("dismiss_mini_history_overlay(\"escape_key\""),
        "Escape key must call dismiss_mini_history_overlay"
    );
}

#[test]
fn mode_switch_focuses_input_after_switch() {
    // The canonical set_window_mode helper must focus input
    let interactions = read("src/ai/window/interactions.rs");
    let set_fn = interactions
        .find("fn set_window_mode")
        .expect("set_window_mode must exist");
    let fn_end = (set_fn + 3000).min(interactions.len());
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
    let f_section = &keydown[f_section_start..(f_section_start + 2000).min(keydown.len())];
    assert!(
        f_section.contains("self.window_mode.is_mini()"),
        "Cmd+Shift+F handler must check for mini mode"
    );
    assert!(
        f_section.contains("show_mini_history_overlay"),
        "Cmd+Shift+F in mini mode must idempotently open the history overlay"
    );
}

#[test]
fn mini_sidebar_shortcuts_are_guarded() {
    let keydown = read("src/ai/window/render_keydown.rs");
    // Cmd+\ and Cmd+B should check for mini mode before toggling sidebar
    let backslash_start = keydown
        .find("\"\\\\\" | \"backslash\"")
        .expect("Cmd+\\ handler must exist");
    let backslash_section = &keydown[backslash_start..(backslash_start + 200).min(keydown.len())];
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
    let titlebar_section = &root[mini_titlebar_start..(mini_titlebar_start + 2000).min(root.len())];
    assert!(
        titlebar_section.contains("title_text"),
        "Mini titlebar should display chat title when messages exist"
    );
}

#[test]
fn close_paths_clear_global_handle() {
    // Both Cmd+W and Esc-mini close paths must call cleanup_ai_window_globals()
    // to prevent stale AI_WINDOW handles from breaking reopen.
    let keydown = read("src/ai/window/render_keydown.rs");

    // Cmd+W handler
    let cmd_w_start = keydown
        .find("// Cmd+W closes the AI window")
        .expect("Cmd+W close comment must exist");
    let cmd_w_section = &keydown[cmd_w_start..(cmd_w_start + 800).min(keydown.len())];
    assert!(
        cmd_w_section.contains("cleanup_ai_window_globals()"),
        "Cmd+W handler must call cleanup_ai_window_globals before remove_window"
    );

    // Esc-mini handler
    let esc_mini_start = keydown
        .find("ai_window_close")
        .expect("ai_window_close lifecycle event must exist");
    let esc_mini_section = &keydown[esc_mini_start..(esc_mini_start + 400).min(keydown.len())];
    assert!(
        esc_mini_section.contains("cleanup_ai_window_globals()"),
        "Esc-mini handler must call cleanup_ai_window_globals before remove_window"
    );

    // close_ai_window() must also call cleanup
    let api = read("src/ai/window/window_api.rs");
    assert!(
        api.contains("fn cleanup_ai_window_globals()"),
        "cleanup_ai_window_globals helper must exist in window_api.rs"
    );
    let close_fn_start = api
        .find("pub fn close_ai_window")
        .expect("close_ai_window must exist");
    let close_fn_section = &api[close_fn_start..(close_fn_start + 1500).min(api.len())];
    assert!(
        close_fn_section.contains("cleanup_ai_window_globals()"),
        "close_ai_window must delegate to cleanup_ai_window_globals"
    );
}

#[test]
fn both_close_paths_log_lifecycle_telemetry() {
    // Both Cmd+W and Esc-mini close paths must log ai_window_close via
    // telemetry::log_ai_lifecycle so that close events are always observable.
    let keydown = read("src/ai/window/render_keydown.rs");

    // Cmd+W handler must log lifecycle
    let cmd_w_start = keydown
        .find("// Cmd+W closes the AI window")
        .expect("Cmd+W close comment must exist");
    let cmd_w_section = &keydown[cmd_w_start..(cmd_w_start + 600).min(keydown.len())];
    assert!(
        cmd_w_section.contains("log_ai_lifecycle("),
        "Cmd+W handler must call log_ai_lifecycle before closing"
    );
    assert!(
        cmd_w_section.contains("\"ai_window_close\""),
        "Cmd+W handler must log the ai_window_close event"
    );
    assert!(
        cmd_w_section.contains("\"cmd_w\""),
        "Cmd+W handler must tag source as cmd_w"
    );

    // Esc-mini handler must log lifecycle
    let esc_close_start = keydown
        .find("// Mini mode: final Esc closes the window")
        .expect("final mini Esc close comment must exist");
    let esc_section = &keydown[esc_close_start..(esc_close_start + 600).min(keydown.len())];
    assert!(
        esc_section.contains("log_ai_lifecycle("),
        "Esc-mini handler must call log_ai_lifecycle before closing"
    );
    assert!(
        esc_section.contains("\"ai_window_close\""),
        "Esc-mini handler must log the ai_window_close event"
    );
    assert!(
        esc_section.contains("\"escape_key\""),
        "Esc-mini handler must tag source as escape_key"
    );
}

#[test]
fn close_path_uses_atomic_mode_not_title_string() {
    // close_ai_window must derive role from AI_CURRENT_WINDOW_MODE atomic,
    // not from window title strings. This ensures persistence correctness
    // even if the title is changed or localized.
    let api = read("src/ai/window/window_api.rs");
    let close_fn_start = api
        .find("pub fn close_ai_window")
        .expect("close_ai_window must exist");
    let close_fn_section = &api[close_fn_start..(close_fn_start + 1000).min(api.len())];
    assert!(
        close_fn_section.contains("AI_CURRENT_WINDOW_MODE"),
        "close_ai_window must read mode from AI_CURRENT_WINDOW_MODE atomic"
    );
    assert!(
        !close_fn_section.contains("window_title()"),
        "close_ai_window must NOT use window_title() to determine mode"
    );
}

#[test]
fn maybe_persist_bounds_uses_state_derived_mode() {
    // Render-frame bounds persistence must use self.window_mode via
    // window_role_for_mode, not title strings or hardcoded roles.
    let interactions = read("src/ai/window/interactions.rs");
    let persist_fn_start = interactions
        .find("fn maybe_persist_bounds")
        .expect("maybe_persist_bounds must exist");
    let fn_end = (persist_fn_start + 1000).min(interactions.len());
    let fn_body = &interactions[persist_fn_start..fn_end];
    assert!(
        fn_body.contains("window_role_for_mode(self.window_mode)"),
        "maybe_persist_bounds must derive role from self.window_mode"
    );
}

#[test]
fn mode_toggle_restores_full_bounds_via_platform_layer() {
    // set_window_mode must attempt full position+size restore via
    // move_window_by_view, not just resize. This is critical for
    // mini↔full transitions to restore the correct window position.
    let interactions = read("src/ai/window/interactions.rs");
    let set_fn = interactions
        .find("fn set_window_mode")
        .expect("set_window_mode must exist");
    let fn_end = (set_fn + 3000).min(interactions.len());
    let fn_body = &interactions[set_fn..fn_end];
    assert!(
        fn_body.contains("move_window_by_view"),
        "set_window_mode must use move_window_by_view for full bounds restore"
    );
    assert!(
        fn_body.contains("window_handle"),
        "set_window_mode must access raw window handle for position restore"
    );
}

#[test]
fn mini_main_panel_excludes_full_mode_chrome() {
    // The mini main panel must NOT contain inspector or drawer chrome.
    // These are full-mode-only features behind Cmd+K in mini mode.
    let panel = read("src/ai/window/render_main_panel.rs");
    let mini_fn_start = panel
        .find("fn render_mini_main_panel")
        .expect("render_mini_main_panel must exist");
    let full_fn_start = panel
        .find("fn render_full_main_panel")
        .expect("render_full_main_panel must exist");
    // Mini function body is between its declaration and the full panel declaration
    let mini_body = &panel[mini_fn_start..full_fn_start];
    assert!(
        !mini_body.contains("render_model_picker"),
        "Mini panel must not render inline model picker (it's in the header)"
    );
    assert!(
        !mini_body.contains("word_count"),
        "Mini panel must not show word count chrome"
    );
}

#[test]
fn telemetry_module_provides_both_event_helpers() {
    let telemetry = read("src/ai/window/telemetry.rs");
    assert!(
        telemetry.contains("pub(super) fn log_ai_lifecycle("),
        "telemetry.rs must export log_ai_lifecycle helper"
    );
    assert!(
        telemetry.contains("pub(super) fn log_ai_ui("),
        "telemetry.rs must export log_ai_ui helper"
    );
}

#[test]
fn builtin_handoff_uses_source_traced_entry_point() {
    let api = read("src/ai/window/window_api.rs");
    assert!(
        api.contains("pub fn open_mini_ai_window_from("),
        "window_api.rs must export open_mini_ai_window_from for source tracing"
    );
    assert!(
        api.contains("fn open_ai_window_with_mode_from("),
        "window_api.rs must have the internal source-traced open path"
    );

    let execution = read("src/app_execute/builtin_execution.rs");
    assert!(
        execution.contains("open_mini_ai_window_from(\"builtin_mini_ai\""),
        "builtin execution must call open_mini_ai_window_from with 'builtin_mini_ai' source"
    );
}

#[test]
fn debug_snapshot_is_serializable_and_complete() {
    let state = read("src/ai/window/state.rs");
    assert!(
        state.contains("pub struct AiMiniDebugSnapshot"),
        "state.rs must define AiMiniDebugSnapshot"
    );
    assert!(
        state.contains("serde::Serialize"),
        "AiMiniDebugSnapshot must derive Serialize"
    );
    assert!(
        state.contains("fn debug_snapshot(&self) -> AiMiniDebugSnapshot"),
        "AiApp must implement debug_snapshot()"
    );

    // All critical state fields should be in the snapshot
    for field in [
        "window_mode",
        "history_overlay_visible",
        "command_bar_open",
        "new_chat_menu_open",
        "presets_dropdown_open",
        "api_key_input_visible",
        "context_picker_open",
        "selected_model",
        "selected_chat_id",
        "pending_context_parts",
        "has_pending_image",
        "is_streaming",
        "streaming_error_present",
        "pending_delete_chat_present",
        "chat_count",
        "current_message_count",
        "show_context_inspector",
        "show_context_drawer",
        "search_query",
        "shortcuts_overlay_visible",
        "editing_message_present",
        "renaming_chat_present",
    ] {
        assert!(
            state.contains(&format!("{field}:")),
            "AiMiniDebugSnapshot must include {field} field"
        );
    }
}

#[test]
fn window_api_has_no_logging_log_calls() {
    // All freeform logging::log calls should be replaced with structured tracing
    let api = read("src/ai/window/window_api.rs");
    assert!(
        !api.contains("logging::log("),
        "window_api.rs should use structured tracing, not logging::log()"
    );
}

#[test]
fn mini_context_contract_keeps_essential_surfaces_reachable() {
    // Mini mode must render context bar, picker, chips, and image preview.
    // Context recommendations are hidden in mini (too much vertical space
    // in a 440px window — users can still attach via Cmd+Shift+A).
    // Inspector/drawer are full-mode-only.
    let panel = read("src/ai/window/render_main_panel.rs");
    let mini_fn_start = panel
        .find("fn render_mini_main_panel")
        .expect("render_mini_main_panel must exist");
    let full_fn_start = panel
        .find("fn render_full_main_panel")
        .expect("render_full_main_panel must exist");
    let mini_body = &panel[mini_fn_start..full_fn_start];

    // Surfaces that MUST be in mini mode
    for surface in [
        "render_context_bar",
        "render_context_picker",
        "render_pending_context_chips",
        "render_pending_image_preview",
    ] {
        assert!(
            mini_body.contains(surface),
            "Mini main panel must render {surface}"
        );
    }

    // Context recommendations are now shown in mini mode (parity with full mode)
    assert!(
        mini_body.contains("render_context_recommendations"),
        "Mini main panel must render context recommendations (parity with full mode)"
    );
}

#[test]
fn new_conversation_clears_mini_overlay() {
    // new_conversation and select_chat must both clear the mini history
    // overlay to prevent stale overlay state after navigation.
    let chat = read("src/ai/window/chat.rs");

    let new_conv_start = chat
        .find("fn new_conversation")
        .expect("new_conversation must exist");
    let new_conv_end = (new_conv_start + 2000).min(chat.len());
    let new_conv_body = &chat[new_conv_start..new_conv_end];
    assert!(
        new_conv_body.contains("showing_mini_history_overlay = false"),
        "new_conversation must clear mini history overlay"
    );

    let select_chat_start = chat.find("fn select_chat").expect("select_chat must exist");
    let select_chat_end = (select_chat_start + 4000).min(chat.len());
    let select_chat_body = &chat[select_chat_start..select_chat_end];
    assert!(
        select_chat_body.contains("showing_mini_history_overlay = false"),
        "select_chat must clear mini history overlay"
    );
}

#[test]
fn escape_chain_guards_every_intermediate_state_before_final_close() {
    // The Escape key handler must dismiss intermediate states in priority order
    // before reaching the final mini-close. Each guard must appear earlier in
    // the source than the final close to prevent skipping states.
    let keydown = read("src/ai/window/render_keydown.rs");
    let final_close = keydown
        .find("// Mini mode: final Esc closes the window")
        .expect("final mini Esc close must exist");

    let guards = [
        ("showing_mini_history_overlay", "mini history overlay"),
        ("showing_shortcuts_overlay", "shortcuts overlay"),
        ("search_query.is_empty()", "search query clear"),
        ("editing_message_id.is_some()", "edit cancel"),
        ("renaming_chat_id.is_some()", "rename cancel"),
        ("self.is_streaming", "stop streaming"),
        ("showing_api_key_input", "api key input"),
        ("command_bar.is_open()", "command bar close"),
        (
            "new_chat_command_bar.is_open()",
            "new chat command bar close",
        ),
    ];

    for (pattern, label) in guards {
        let guard_pos = keydown
            .find(pattern)
            .unwrap_or_else(|| panic!("Escape guard for {label} must exist"));
        assert!(
            guard_pos < final_close,
            "Escape guard for {label} must come BEFORE final mini close"
        );
    }
}

#[test]
fn snapshot_exposes_interaction_state_fields() {
    // The debug snapshot must include fields that allow agentic tests to
    // verify interaction state without reaching into AiApp internals.
    let state = read("src/ai/window/state.rs");
    let snapshot_start = state
        .find("pub struct AiMiniDebugSnapshot")
        .expect("AiMiniDebugSnapshot must exist");
    let builder_start = state
        .find("fn debug_snapshot(&self)")
        .expect("debug_snapshot builder must exist");
    let snapshot_def = &state[snapshot_start..builder_start];
    let builder_body = &state[builder_start..(builder_start + 2000).min(state.len())];

    // Each interaction field must appear in both the struct and the builder
    for (field, source_expr) in [
        ("search_query", "self.search_query.clone()"),
        (
            "shortcuts_overlay_visible",
            "self.showing_shortcuts_overlay",
        ),
        (
            "editing_message_present",
            "self.editing_message_id.is_some()",
        ),
        ("renaming_chat_present", "self.renaming_chat_id.is_some()"),
        ("presets_dropdown_open", "self.showing_presets_dropdown"),
        ("api_key_input_visible", "self.showing_api_key_input"),
        ("context_picker_open", "self.is_context_picker_open()"),
        ("streaming_error_present", "self.streaming_error.is_some()"),
        (
            "pending_delete_chat_present",
            "self.pending_delete_chat_id.is_some()",
        ),
    ] {
        assert!(
            snapshot_def.contains(field),
            "AiMiniDebugSnapshot must declare {field}"
        );
        assert!(
            builder_body.contains(source_expr),
            "debug_snapshot() must populate {field} from {source_expr}"
        );
    }
}

#[test]
fn new_chat_command_bar_blocks_shortcut_fallthrough() {
    // When new_chat_command_bar is open, the handler must `return` after the
    // match block to prevent Cmd+J/K/N shortcuts from leaking through.
    // We verify that a bare `return;` exists between the bar's open-guard and
    // the next top-level `if` (the welcome suggestions block), proving the
    // handler exits before any Cmd shortcut can fire.
    let keydown = read("src/ai/window/render_keydown.rs");
    let bar_block_start = keydown
        .find("if self.new_chat_command_bar.is_open() {")
        .expect("new_chat_command_bar.is_open() guard must exist");
    // Scan from the bar guard to the next top-level block (welcome suggestions)
    let next_block = keydown[bar_block_start..]
        .find("// Cmd+1-4:")
        .expect("welcome suggestions block must follow the new_chat_command_bar block");
    let bar_block_section = &keydown[bar_block_start..bar_block_start + next_block];
    assert!(
        bar_block_section.contains("return;"),
        "new_chat_command_bar block must have a return guard to prevent shortcut leakage"
    );
}

#[test]
fn cmd_n_calls_new_conversation_in_all_modes() {
    // Cmd+N must call new_conversation in both mini and full mode (unified behavior).
    // There should be no mini-specific branching for Cmd+N.
    let keydown = read("src/ai/window/render_keydown.rs");
    let n_arm = keydown
        .find("\"n\" => {")
        .expect("Cmd+N handler must exist in render_keydown.rs");
    let f_arm = keydown[n_arm..]
        .find("// Cmd+Shift+F to focus search")
        .expect("Cmd+Shift+F comment must bound Cmd+N handler");
    let n_section = &keydown[n_arm..n_arm + f_arm];
    assert!(
        n_section.contains("self.new_conversation(window, cx)"),
        "Cmd+N must call new_conversation"
    );
    assert!(
        !n_section.contains("show_new_chat_command_bar"),
        "Cmd+N must not branch to show_new_chat_command_bar"
    );
}

#[test]
fn hide_all_dropdowns_closes_new_chat_command_bar() {
    // Verify that hide_all_dropdowns explicitly closes the new_chat_command_bar.
    // Scan from the function signature to the next `pub` function boundary.
    let dropdowns = read("src/ai/window/dropdowns.rs");
    let hide_fn_start = dropdowns
        .find("fn hide_all_dropdowns")
        .expect("hide_all_dropdowns must exist");
    let fn_body_end = dropdowns[hide_fn_start..]
        .find("cx.notify()")
        .expect("hide_all_dropdowns must call cx.notify()");
    let fn_body = &dropdowns[hide_fn_start..hide_fn_start + fn_body_end];
    assert!(
        fn_body.contains("new_chat_command_bar.close(cx)"),
        "hide_all_dropdowns must close new_chat_command_bar"
    );
}

#[test]
fn mini_welcome_is_compact_with_fewer_suggestions() {
    let source = read("src/ai/window/render_welcome.rs");
    // Mini mode must have a dedicated renderer
    assert!(
        source.contains("fn render_mini_welcome("),
        "Mini welcome must have a dedicated render_mini_welcome method"
    );
    // Mini renderer must limit suggestion count
    assert!(
        source.contains("MINI_SUGGESTION_COUNT"),
        "Mini welcome must use MINI_SUGGESTION_COUNT constant for suggestion limit"
    );
    // Mini renderer must use compact heading (not the full-mode "Ask Anything")
    assert!(
        source.contains("Try a suggestion"),
        "Mini welcome must use a compact heading"
    );
    // Mini renderer must push content toward composer (not center vertically)
    assert!(
        source.contains("justify_end()"),
        "Mini welcome must use justify_end to push content near composer"
    );
    // Mini panel must dispatch to the dedicated mini renderer
    let panel = read("src/ai/window/render_main_panel.rs");
    assert!(
        panel.contains("render_mini_welcome("),
        "Mini panel must call render_mini_welcome instead of render_welcome"
    );
}

#[test]
fn mini_welcome_shortcuts_match_visible_suggestions() {
    let keydown = read("src/ai/window/render_keydown.rs");
    assert!(
        keydown.contains("MINI_SUGGESTION_COUNT") && keydown.contains("FULL_SUGGESTION_COUNT"),
        "Welcome shortcuts must use named constants for suggestion count in both modes"
    );
    assert!(
        keydown.contains("idx.filter(|i| *i < max_visible_suggestions)"),
        "Welcome shortcuts must ignore hidden suggestion slots in mini mode"
    );
    // Keyboard shortcuts must use the same data source as the rendered cards
    assert!(
        keydown.contains("script_kit_welcome_suggestions()"),
        "Keyboard shortcuts must use script_kit_welcome_suggestions() from render_welcome — not a duplicate constant"
    );
    // The old WELCOME_SUGGESTIONS constant must NOT be referenced
    assert!(
        !keydown.contains("WELCOME_SUGGESTIONS"),
        "render_keydown must NOT reference the removed WELCOME_SUGGESTIONS constant"
    );
}

#[test]
fn mini_content_area_is_flex_column_for_welcome_layout() {
    let panel = read("src/ai/window/render_main_panel.rs");
    let mini_fn_start = panel
        .find("fn render_mini_main_panel")
        .expect("render_mini_main_panel must exist");
    let full_fn_start = panel
        .find("fn render_full_main_panel")
        .expect("render_full_main_panel must exist");
    let mini_body = &panel[mini_fn_start..full_fn_start];
    // The max-width container must be a flex column so welcome's
    // .flex_1() + .justify_end() can push content toward the composer
    assert!(
        mini_body.contains(".flex()\n                        .flex_col()")
            || mini_body.contains(".flex().flex_col()"),
        "Mini content wrapper must be a flex column for welcome layout"
    );
}

#[test]
fn mini_header_has_expand_button_and_streaming_indicator() {
    let root = read("src/ai/window/render_root.rs");
    // Expand button must exist and trigger toggle_window_mode
    assert!(
        root.contains("ai-mini-expand"),
        "Mini header must have an expand button with id ai-mini-expand"
    );
    assert!(
        root.contains("toggle_window_mode(window, cx)"),
        "Expand button must call toggle_window_mode"
    );
    // Streaming dot indicator
    assert!(
        root.contains("ai-mini-streaming-dot"),
        "Mini header must show a streaming indicator dot"
    );
    // History overlay anchored to the right
    assert!(
        root.contains(".right(S3)"),
        "History overlay must be right-anchored (not left)"
    );
}

#[test]
fn mini_ai_is_backed_by_ai_app_not_a_separate_type() {
    // The architecture must use AiApp + AiWindowMode::Mini, not a forked MiniAiApp.
    // Scan all AI window source files for any "MiniAiApp" declaration.
    let files = [
        "src/ai/window/state.rs",
        "src/ai/window/types.rs",
        "src/ai/window/window_api.rs",
        "src/ai/window/render_root.rs",
        "src/ai/window/render_main_panel.rs",
        "src/ai/window/render_keydown.rs",
        "src/ai/mod.rs",
        "src/ai/window.rs",
    ];
    for path in files {
        let source = read(path);
        assert!(
            !source.contains("MiniAiApp"),
            "{path} must NOT define or reference a separate MiniAiApp type"
        );
    }

    // Verify AiWindowMode::Mini is defined on the existing AiApp
    let types = read("src/ai/window/types.rs");
    assert!(
        types.contains("Mini,") || types.contains("Mini\n"),
        "AiWindowMode must have a Mini variant"
    );
    let state = read("src/ai/window/state.rs");
    assert!(
        state.contains("window_mode: AiWindowMode"),
        "AiApp must carry window_mode: AiWindowMode field"
    );
}

#[test]
fn mini_history_overlay_uses_dedicated_panel() {
    // The mini history overlay must use a dedicated panel wrapper that
    // composes the shared chat list body with mini-specific chrome.
    let sidebar = read("src/ai/window/render_sidebar.rs");
    assert!(
        sidebar.contains("fn render_chat_list_body("),
        "render_sidebar.rs must define render_chat_list_body"
    );
    assert!(
        sidebar.contains("fn render_mini_history_panel("),
        "render_sidebar.rs must define render_mini_history_panel"
    );
    assert!(
        sidebar.contains("fn render_sidebar_body("),
        "render_sidebar.rs must still define render_sidebar_body for the full sidebar"
    );
    assert!(
        sidebar.contains("Used by both the full sidebar and the mini history overlay"),
        "render_sidebar_body must document its dual-use role"
    );

    // render_mini_history_panel must compose the shared chat list body
    let panel_section = &sidebar[sidebar
        .find("fn render_mini_history_panel(")
        .expect("render_mini_history_panel must exist")..];
    assert!(
        panel_section.contains("render_chat_list_body(cx)"),
        "render_mini_history_panel must compose render_chat_list_body"
    );
    assert!(
        panel_section.contains("ai-mini-history-header"),
        "render_mini_history_panel must have a compact header"
    );

    // render_root.rs must call render_mini_history_panel for the overlay
    let root = read("src/ai/window/render_root.rs");
    assert!(
        root.contains("self.render_mini_history_panel(cx)"),
        "render_root.rs must call render_mini_history_panel for the mini history overlay"
    );
}

#[test]
fn mini_mode_hides_docked_sidebar_in_root() {
    // In mini mode, the root layout must NOT render the docked sidebar.
    // The full sidebar is only shown in the else branch of a window_mode.is_mini() check.
    let root = read("src/ai/window/render_root.rs");

    let sidebar_render = root
        .find(".child(self.render_sidebar(cx))")
        .expect("Full sidebar render must exist in render_root.rs");
    // The is_mini guard may be up to ~500 chars before the sidebar call (covers the
    // entire conditional block). The mini branch renders main_panel only; the else
    // branch adds the sidebar.
    let search_start = sidebar_render.saturating_sub(700);
    let preceding = &root[search_start..sidebar_render];
    assert!(
        preceding.contains("self.window_mode.is_mini()"),
        "Docked sidebar must be inside a window_mode.is_mini() conditional \
         (mini branch omits sidebar, else branch includes it)"
    );
}

#[test]
fn mini_ai_has_stdin_protocol_commands() {
    // openMiniAi and openMiniAiWithMockData must exist as stdin commands
    // for DX/testing — opens the mini AI window via the JSON protocol.
    let stdin = read("src/stdin_commands/mod.rs");
    assert!(
        stdin.contains("OpenMiniAi"),
        "stdin_commands must define OpenMiniAi variant"
    );
    assert!(
        stdin.contains("OpenMiniAiWithMockData"),
        "stdin_commands must define OpenMiniAiWithMockData variant"
    );
    assert!(
        stdin.contains("\"openMiniAi\""),
        "OpenMiniAi must serialize as \"openMiniAi\""
    );
    assert!(
        stdin.contains("\"openMiniAiWithMockData\""),
        "OpenMiniAiWithMockData must serialize as \"openMiniAiWithMockData\""
    );

    // All three dispatch sites must handle both commands
    for dispatch_file in [
        "src/main_entry/runtime_stdin.rs",
        "src/main_entry/runtime_stdin_match_tail.rs",
        "src/main_entry/app_run_setup.rs",
    ] {
        let source = read(dispatch_file);
        assert!(
            source.contains("ExternalCommand::OpenMiniAi"),
            "{dispatch_file} must dispatch ExternalCommand::OpenMiniAi"
        );
        assert!(
            source.contains("ExternalCommand::OpenMiniAiWithMockData"),
            "{dispatch_file} must dispatch ExternalCommand::OpenMiniAiWithMockData"
        );
        assert!(
            source.contains("open_mini_ai_window"),
            "{dispatch_file} must call open_mini_ai_window"
        );
    }
}

#[test]
fn esc_chain_emits_log_ai_state_at_each_dismissal() {
    // Every Escape dismissal branch in render_keydown must call log_ai_state
    // so the Esc-chain is machine-verifiable from logs alone.
    let keydown = read("src/ai/window/render_keydown.rs");

    // Each Esc event name must appear (proves the telemetry call exists)
    for event_name in [
        "esc_dismiss_history_overlay",
        "esc_dismiss_shortcuts_overlay",
        "esc_clear_search",
        "esc_cancel_editing",
        "esc_cancel_rename",
        "esc_stop_streaming",
        "esc_dismiss_api_key_input",
        "esc_dismiss_dropdown",
        "esc_close_mini_window",
    ] {
        assert!(
            keydown.contains(event_name),
            "render_keydown must emit log_ai_state(\"{event_name}\", ...) on Esc"
        );
    }

    // The helper itself must be called (not just the event string)
    assert!(
        keydown.contains("telemetry::log_ai_state("),
        "render_keydown must call telemetry::log_ai_state()"
    );
}

#[test]
fn telemetry_log_ai_state_helper_exists() {
    let telemetry = read("src/ai/window/telemetry.rs");
    assert!(
        telemetry.contains("pub(super) fn log_ai_state("),
        "telemetry.rs must define log_ai_state helper"
    );
    assert!(
        telemetry.contains("AiMiniDebugSnapshot"),
        "log_ai_state must accept AiMiniDebugSnapshot"
    );
    assert!(
        telemetry.contains("category = \"AI_STATE\""),
        "log_ai_state must use AI_STATE category"
    );
}

/// `set_window_mode` must clear search state when the history overlay was open,
/// so switching from mini (with search typed) to full doesn't leak stale queries.
#[test]
fn mode_switch_clears_search_when_overlay_was_open() {
    let source = read("src/ai/window/interactions.rs");
    let set_fn = source
        .find("fn set_window_mode")
        .expect("set_window_mode must exist");
    let after = &source[set_fn..];
    // Must clear search state when overlay was showing
    assert!(
        after.contains("clear_search_state"),
        "set_window_mode must call clear_search_state when overlay was open"
    );
    // Must emit state telemetry after mode switch
    assert!(
        after.contains("log_ai_state"),
        "set_window_mode must emit log_ai_state after switching"
    );
}

/// `set_ai_pending_chat` must emit a structured error event on lock failure
/// so agents can detect and report the failure programmatically.
#[test]
fn pending_chat_lock_failure_emits_structured_error() {
    let source = read("src/ai/window/window_api.rs");
    let fn_pos = source
        .find("fn set_ai_pending_chat")
        .expect("set_ai_pending_chat must exist");
    let after = &source[fn_pos..];
    assert!(
        after.contains("ai_pending_chat_failed"),
        "set_ai_pending_chat must log ai_pending_chat_failed on lock failure"
    );
    assert!(
        after.contains("reason = \"lock_poisoned\""),
        "error event must include a stable reason field"
    );
}

// === Esc-chain ordering audit ===

/// The Esc-chain in render_keydown.rs must dismiss overlays in the correct
/// priority order. Each layer must appear *before* the next one so that
/// higher-priority dismissals are checked first.
#[test]
fn esc_chain_layers_are_ordered_correctly() {
    let source = read("src/ai/window/render_keydown.rs");

    // These are the Esc guards in priority order (highest first).
    // Each must appear before the next in the source.
    let layers = [
        // Layer 1: Mini history overlay (broader key guard, not just Esc)
        "if self.window_mode.is_mini() && self.showing_mini_history_overlay",
        // Layer 2: Shortcuts overlay
        "is_key_escape(key) && self.showing_shortcuts_overlay",
        // Layer 3: Active search
        "is_key_escape(key) && !self.search_query.is_empty()",
        // Layer 4: Editing message
        "is_key_escape(key) && self.editing_message_id.is_some()",
        // Layer 5: Renaming chat
        "is_key_escape(key) && self.renaming_chat_id.is_some()",
        // Layer 6: Streaming
        "is_key_escape(key) && self.is_streaming",
        // Layer 7: API key input
        "is_key_escape(key) && self.showing_api_key_input",
    ];

    let mut prev_pos = 0;
    let mut prev_label = "";
    for &layer in &layers {
        let pos = source.find(layer).unwrap_or_else(|| {
            panic!("Esc-chain missing guard: {layer}");
        });
        assert!(
            pos > prev_pos || prev_label.is_empty(),
            "Esc-chain order violation: '{layer}' must come after '{prev_label}'"
        );
        prev_pos = pos;
        prev_label = layer;
    }
}

/// Each Esc-chain layer must emit a structured state snapshot via `log_ai_state`
/// so agents can verify transitions without screenshots.
#[test]
fn esc_chain_layers_emit_state_snapshots() {
    let source = read("src/ai/window/render_keydown.rs");
    let expected_events = [
        "esc_dismiss_history_overlay",
        "esc_dismiss_shortcuts_overlay",
        "esc_clear_search",
        "esc_cancel_editing",
        "esc_cancel_rename",
        "esc_stop_streaming",
        "esc_dismiss_api_key_input",
        "esc_close_mini_window",
    ];
    for event in expected_events {
        assert!(
            source.contains(event),
            "render_keydown.rs missing Esc telemetry event: {event}"
        );
    }
}

// === getAiWindowState command wiring audit ===

/// The getAiWindowState external command must be defined and wired in all stdin dispatch sites.
#[test]
fn get_ai_window_state_command_wired_in_all_dispatch_sites() {
    let stdin_mod = read("src/stdin_commands/mod.rs");
    assert!(
        stdin_mod.contains("GetAiWindowState"),
        "stdin_commands/mod.rs must define GetAiWindowState variant"
    );
    assert!(
        stdin_mod.contains("\"getAiWindowState\""),
        "stdin_commands/mod.rs must map GetAiWindowState to command type string"
    );

    // All 3 dispatch sites must handle the command
    for path in [
        "src/main_entry/runtime_stdin.rs",
        "src/main_entry/runtime_stdin_match_tail.rs",
        "src/main_entry/app_run_setup.rs",
    ] {
        let source = read(path);
        assert!(
            source.contains("ExternalCommand::GetAiWindowState"),
            "{path} must handle ExternalCommand::GetAiWindowState"
        );
        assert!(
            source.contains("ai_window_state_result"),
            "{path} must emit ai_window_state_result event"
        );
    }
}

/// The get_ai_window_state public function must exist in window_api.rs
/// and be exported through the module facade.
#[test]
fn get_ai_window_state_is_exported() {
    let api = read("src/ai/window/window_api.rs");
    assert!(
        api.contains("pub fn get_ai_window_state("),
        "window_api.rs must export get_ai_window_state"
    );

    let window_mod = read("src/ai/window.rs");
    assert!(
        window_mod.contains("get_ai_window_state"),
        "window.rs must re-export get_ai_window_state"
    );

    let ai_mod = read("src/ai/mod.rs");
    assert!(
        ai_mod.contains("get_ai_window_state"),
        "ai/mod.rs must re-export get_ai_window_state"
    );
}

#[test]
fn get_ai_window_state_redacts_search_text_before_external_use() {
    let api = read("src/ai/window/window_api.rs");
    assert!(
        api.contains("redact_for_external_use()"),
        "get_ai_window_state must redact user-entered search text before export"
    );

    let state = read("src/ai/window/state.rs");
    assert!(
        state.contains("fn redact_for_external_use"),
        "AiMiniDebugSnapshot must provide an external redaction helper"
    );
}

// === AiMiniDebugSnapshot contract audit ===

/// AiMiniDebugSnapshot must derive both Serialize and Deserialize for roundtrip.
#[test]
fn debug_snapshot_derives_serde_roundtrip() {
    let source = read("src/ai/window/state.rs");
    let struct_pos = source
        .find("struct AiMiniDebugSnapshot")
        .expect("AiMiniDebugSnapshot must exist");
    let before = &source[..struct_pos];
    assert!(
        before.contains("serde::Serialize") && before.contains("serde::Deserialize"),
        "AiMiniDebugSnapshot must derive both Serialize and Deserialize"
    );
    assert!(
        before.contains("rename_all = \"camelCase\""),
        "AiMiniDebugSnapshot must use camelCase serde renaming for wire compatibility"
    );
}

/// AiMiniDebugSnapshot must be pub (not pub(crate)) for integration test access.
#[test]
fn debug_snapshot_is_public() {
    let source = read("src/ai/window/state.rs");
    assert!(
        source.contains("pub struct AiMiniDebugSnapshot"),
        "AiMiniDebugSnapshot must be pub for cross-crate test access"
    );
}

// === Legacy bridge removal audit ===

/// The legacy per-display AI window bridge must be removed.
#[test]
fn legacy_per_display_bridge_removed() {
    let source = read("src/ai/window/window_api.rs");
    assert!(
        !source.contains("ai_window_reference_legacy_per_display_apis"),
        "Legacy bridge function must be removed from window_api.rs"
    );
}

// === Mini welcome compact layout audit ===

/// The mini welcome renderer must use dedicated mini layout constants,
/// not the full-mode suggestion constants.
#[test]
fn mini_welcome_uses_dedicated_layout_constants() {
    let source = read("src/ai/window/render_welcome.rs");
    let mini_fn_start = source
        .find("fn render_mini_welcome(")
        .expect("render_mini_welcome must exist");
    // Find the next fn boundary to scope assertions
    let next_fn = source[mini_fn_start + 10..]
        .find("\n    pub(super) fn ")
        .unwrap_or(source.len() - mini_fn_start - 10);
    let mini_body = &source[mini_fn_start..mini_fn_start + 10 + next_fn];

    assert!(
        mini_body.contains("MINI_WELCOME_MAX_W"),
        "Mini welcome must use MINI_WELCOME_MAX_W for suggestion list width"
    );
    assert!(
        mini_body.contains("MINI_WELCOME_ICON_CONTAINER"),
        "Mini welcome must use MINI_WELCOME_ICON_CONTAINER for icon hit area"
    );
    assert!(
        mini_body.contains("MINI_WELCOME_ICON_SIZE"),
        "Mini welcome must use MINI_WELCOME_ICON_SIZE for icon dimensions"
    );
    // Must NOT reference the full-mode constants
    assert!(
        !mini_body.contains("SUGGESTION_ICON_CONTAINER"),
        "Mini welcome must NOT use full-mode SUGGESTION_ICON_CONTAINER"
    );
    assert!(
        !mini_body.contains("SUGGESTION_MAX_W"),
        "Mini welcome must NOT use full-mode SUGGESTION_MAX_W"
    );
}

/// The mini welcome renderer must emit structured observability logs.
#[test]
fn mini_welcome_emits_structured_logs() {
    let source = read("src/ai/window/render_welcome.rs");
    let mini_fn_start = source
        .find("fn render_mini_welcome(")
        .expect("render_mini_welcome must exist");
    let next_fn = source[mini_fn_start + 10..]
        .find("\n    pub(super) fn ")
        .unwrap_or(source.len() - mini_fn_start - 10);
    let mini_body = &source[mini_fn_start..mini_fn_start + 10 + next_fn];

    assert!(
        mini_body.contains("category = \"mini_welcome\""),
        "Mini welcome must log with category = mini_welcome"
    );
    assert!(
        mini_body.contains("event = \"render\""),
        "Mini welcome must emit a render event"
    );
    assert!(
        mini_body.contains("event = \"suggestion_clicked\""),
        "Mini welcome must emit suggestion_clicked event on click"
    );
    assert!(
        mini_body.contains("event = \"setup_card_shown\""),
        "Mini welcome must emit setup_card_shown when no models configured"
    );
}

// === Mini composer disabled send button audit ===

/// When input is empty in mini mode, a disabled send affordance must be shown
/// instead of a blank spacer. The element must have a distinct ID.
#[test]
fn mini_composer_has_disabled_send_affordance() {
    let panel = read("src/ai/window/render_main_panel.rs");
    let mini_fn_start = panel
        .find("fn render_mini_main_panel")
        .expect("render_mini_main_panel must exist");
    let full_fn_start = panel
        .find("fn render_full_main_panel")
        .expect("render_full_main_panel must exist");
    let mini_body = &panel[mini_fn_start..full_fn_start];

    assert!(
        mini_body.contains("ai-mini-submit-btn-disabled"),
        "Mini composer must show a disabled send button (ai-mini-submit-btn-disabled) when input is empty"
    );
    // The disabled affordance must still show the arrow icon
    assert!(
        mini_body.contains("ArrowUp"),
        "Disabled send button must show the ArrowUp icon for visual consistency"
    );
    // Must use MINI_BTN_SIZE for consistent sizing
    let disabled_section_start = mini_body
        .find("ai-mini-submit-btn-disabled")
        .expect("disabled send button must exist");
    let disabled_section = &mini_body[disabled_section_start..];
    assert!(
        disabled_section.contains("MINI_BTN_SIZE"),
        "Disabled send button must use MINI_BTN_SIZE constant"
    );
}

// === Mini history panel header audit ===

/// The mini history panel must expose machine-addressable element IDs
/// for the header, new-chat button, and close button.
#[test]
fn mini_history_panel_exposes_header_element_ids() {
    let sidebar = read("src/ai/window/render_sidebar.rs");
    let panel_start = sidebar
        .find("fn render_mini_history_panel(")
        .expect("render_mini_history_panel must exist");
    let panel_body = &sidebar[panel_start..];

    for id in [
        "ai-mini-history-header",
        "ai-mini-history-new",
        "ai-mini-history-close",
    ] {
        assert!(
            panel_body.contains(id),
            "render_mini_history_panel missing expected element ID: {id}"
        );
    }
}

/// The mini history panel header must use named layout constants.
#[test]
fn mini_history_panel_uses_named_constants() {
    let sidebar = read("src/ai/window/render_sidebar.rs");
    let panel_start = sidebar
        .find("fn render_mini_history_panel(")
        .expect("render_mini_history_panel must exist");
    let panel_body = &sidebar[panel_start..];

    assert!(
        panel_body.contains("MINI_HISTORY_HEADER_H"),
        "Mini history panel header must use MINI_HISTORY_HEADER_H constant"
    );
    assert!(
        panel_body.contains("MINI_BTN_SIZE"),
        "Mini history panel buttons must use MINI_BTN_SIZE constant"
    );
}

/// The mini history panel must emit structured observability logs.
#[test]
fn mini_history_panel_emits_structured_logs() {
    let sidebar = read("src/ai/window/render_sidebar.rs");
    let panel_start = sidebar
        .find("fn render_mini_history_panel(")
        .expect("render_mini_history_panel must exist");
    let panel_body = &sidebar[panel_start..];

    assert!(
        panel_body.contains("category = \"mini_history_panel\""),
        "Mini history panel must log with category = mini_history_panel"
    );
    assert!(
        panel_body.contains("event = \"render\""),
        "Mini history panel must emit a render event"
    );
    assert!(
        panel_body.contains("chat_count"),
        "Mini history panel log must include chat_count"
    );
    assert!(
        panel_body.contains("search_active"),
        "Mini history panel log must include search_active"
    );
}

// === Mini overlay height overflow prevention ===

/// MINI_HISTORY_OVERLAY_MAX_H must be at most MINI_WINDOW_DEFAULT_H - 44
/// (the titlebar height in raw pixels) to prevent the overlay from
/// overflowing the mini window bounds.
#[test]
fn mini_overlay_max_height_does_not_overflow_window() {
    let types = read("src/ai/window/types.rs");

    // Extract the raw pixel values from the constants.
    // MINI_HISTORY_OVERLAY_MAX_H and MINI_TITLEBAR_H use px(), but
    // MINI_WINDOW_DEFAULT_H is a bare f32 (not wrapped in px()).
    let parse_px = |line: &str| -> f32 {
        line.split("px(")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.trim_end_matches('.').parse().ok())
            .unwrap_or_else(|| panic!("Could not parse px() value from: {line}"))
    };
    let parse_f32 = |line: &str| -> f32 {
        line.split('=')
            .nth(1)
            .and_then(|s| {
                s.trim()
                    .trim_end_matches(';')
                    .trim()
                    .parse::<f32>()
                    .ok()
            })
            .unwrap_or_else(|| panic!("Could not parse f32 value from: {line}"))
    };

    let overlay_h_line = types
        .lines()
        .find(|l| l.contains("MINI_HISTORY_OVERLAY_MAX_H") && l.contains("px("))
        .expect("MINI_HISTORY_OVERLAY_MAX_H must be defined with px()");
    let titlebar_h_line = types
        .lines()
        .find(|l| l.contains("MINI_TITLEBAR_H") && l.contains("px(") && l.contains("const"))
        .expect("MINI_TITLEBAR_H must be defined with px()");
    let window_h_line = types
        .lines()
        .find(|l| l.contains("MINI_WINDOW_DEFAULT_H") && l.contains("const"))
        .expect("MINI_WINDOW_DEFAULT_H must be defined");

    let overlay_h = parse_px(overlay_h_line);
    let titlebar_h = parse_px(titlebar_h_line);
    let window_h = parse_f32(window_h_line);

    let max_content_h = window_h - titlebar_h;
    assert!(
        overlay_h <= max_content_h,
        "MINI_HISTORY_OVERLAY_MAX_H ({overlay_h}) must be <= \
         MINI_WINDOW_DEFAULT_H - MINI_TITLEBAR_H ({max_content_h}) to prevent overflow"
    );
}

// === Mini history panel new-chat and close behavior ===

/// The new-chat button in the mini history panel must call new_conversation.
/// The close button must call dismiss_mini_history_overlay.
#[test]
fn mini_history_panel_buttons_delegate_correctly() {
    let sidebar = read("src/ai/window/render_sidebar.rs");
    let panel_start = sidebar
        .find("fn render_mini_history_panel(")
        .expect("render_mini_history_panel must exist");
    let panel_body = &sidebar[panel_start..];

    // New chat button delegates to new_conversation
    let new_btn_start = panel_body
        .find("ai-mini-history-new")
        .expect("new-chat button must exist");
    let new_btn_section = &panel_body[new_btn_start..(new_btn_start + 1500).min(panel_body.len())];
    assert!(
        new_btn_section.contains("new_conversation(window, cx)"),
        "Mini history new-chat button must call new_conversation"
    );

    // Close button delegates to dismiss_mini_history_overlay
    let close_btn_start = panel_body
        .find("ai-mini-history-close")
        .expect("close button must exist");
    let close_btn_section =
        &panel_body[close_btn_start..(close_btn_start + 1500).min(panel_body.len())];
    assert!(
        close_btn_section.contains("dismiss_mini_history_overlay("),
        "Mini history close button must call dismiss_mini_history_overlay"
    );
}
