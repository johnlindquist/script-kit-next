use super::types::AiWindowMode;

/// Emit a structured lifecycle event for AI window open/close/mode transitions.
///
/// All lifecycle events share one schema so agents and log parsers can query
/// the full open→mode→close arc with a single filter.
pub(super) fn log_ai_lifecycle(
    event: &'static str,
    window_mode: AiWindowMode,
    source: &'static str,
    status: &'static str,
) {
    tracing::info!(
        target: "ai",
        category = "AI",
        event,
        window_mode = ?window_mode,
        source,
        status,
        "ai_lifecycle"
    );
}

/// Emit a structured UI interaction event (button clicks, overlay toggles, etc.).
pub(super) fn log_ai_ui(event: &'static str, window_mode: AiWindowMode, source: &'static str) {
    tracing::info!(
        target: "ai",
        category = "AI_UI",
        event,
        window_mode = ?window_mode,
        source,
        "ai_ui"
    );
}

/// Emit a structured post-transition state snapshot.
///
/// Call this after any state mutation that changes a dismissible overlay,
/// streaming state, or search query. The resulting log line is a complete
/// picture of the AI window's observable state — agents and tests can
/// parse a single event to verify the Esc-chain or any other transition.
pub(super) fn log_ai_state(
    event: &'static str,
    source: &'static str,
    snapshot: &super::state::AiMiniDebugSnapshot,
) {
    tracing::info!(
        target: "ai",
        category = "AI_STATE",
        event,
        source,
        window_mode = %snapshot.window_mode,
        history_overlay_visible = snapshot.history_overlay_visible,
        command_bar_open = snapshot.command_bar_open,
        new_chat_menu_open = snapshot.new_chat_menu_open,
        presets_dropdown_open = snapshot.presets_dropdown_open,
        api_key_input_visible = snapshot.api_key_input_visible,
        context_picker_open = snapshot.context_picker_open,
        shortcuts_overlay_visible = snapshot.shortcuts_overlay_visible,
        search_query_len = snapshot.search_query.len(),
        search_query_present = !snapshot.search_query.is_empty(),
        is_streaming = snapshot.is_streaming,
        streaming_error_present = snapshot.streaming_error_present,
        editing_message_present = snapshot.editing_message_present,
        renaming_chat_present = snapshot.renaming_chat_present,
        "ai_state"
    );
}

/// Emit a structured shortcut-decision event for AI keyboard routing.
///
/// Logs which branch a keyboard shortcut took, including overlay and search
/// state at decision time. Agents and log parsers can filter on
/// `category = "AI_SHORTCUT"` to reconstruct the full shortcut decision tree.
#[allow(clippy::too_many_arguments)]
pub(super) fn log_ai_shortcut_decision(
    event: &'static str,
    window_mode: AiWindowMode,
    source: &'static str,
    key: &str,
    branch: &'static str,
    handled: bool,
    overlay_visible: bool,
    search_active: bool,
) {
    tracing::info!(
        target: "ai",
        category = "AI_SHORTCUT",
        event,
        window_mode = ?window_mode,
        source,
        key,
        branch,
        handled,
        overlay_visible,
        search_active,
        "ai_shortcut_decision"
    );
}

/// Emit a structured focus-request event for AI window focus transitions.
///
/// Logs when focus moves between input, search, or other targets. Agents and
/// log parsers can filter on `category = "AI_FOCUS"` to trace focus flow.
pub(super) fn log_ai_focus_request(
    event: &'static str,
    window_mode: AiWindowMode,
    source: &'static str,
    target: &'static str,
    overlay_visible: bool,
) {
    tracing::info!(
        target: "ai",
        category = "AI_FOCUS",
        event,
        window_mode = ?window_mode,
        source,
        target,
        overlay_visible,
        "ai_focus_request"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_ai_lifecycle_does_not_panic_for_both_modes() {
        log_ai_lifecycle("test_event", AiWindowMode::Full, "test", "ok");
        log_ai_lifecycle("test_event", AiWindowMode::Mini, "test", "ok");
    }

    #[test]
    fn log_ai_ui_does_not_panic_for_both_modes() {
        log_ai_ui("test_event", AiWindowMode::Full, "test");
        log_ai_ui("test_event", AiWindowMode::Mini, "test");
    }

    #[test]
    fn log_ai_state_does_not_panic_with_default_snapshot() {
        let snapshot = super::super::state::AiMiniDebugSnapshot {
            window_mode: "mini".to_string(),
            history_overlay_visible: false,
            command_bar_open: false,
            new_chat_menu_open: false,
            presets_dropdown_open: false,
            api_key_input_visible: false,
            context_picker_open: false,
            selected_model: None,
            selected_chat_id: None,
            pending_context_parts: 0,
            has_pending_image: false,
            is_streaming: false,
            streaming_error_present: false,
            pending_delete_chat_present: false,
            chat_count: 0,
            current_message_count: 0,
            sidebar_collapsed: false,
            show_context_inspector: false,
            show_context_drawer: false,
            search_query: String::new(),
            shortcuts_overlay_visible: false,
            editing_message_present: false,
            renaming_chat_present: false,
        };
        log_ai_state("test_event", "test", &snapshot);
    }

    #[test]
    fn log_ai_shortcut_decision_does_not_panic() {
        log_ai_shortcut_decision(
            "test_shortcut",
            AiWindowMode::Mini,
            "test",
            "n",
            "cmd_n_new_conversation",
            true,
            false,
            false,
        );
    }

    #[test]
    fn log_ai_focus_request_does_not_panic() {
        log_ai_focus_request("test_focus", AiWindowMode::Mini, "test", "search", true);
    }
}
