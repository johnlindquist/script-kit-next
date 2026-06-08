//! Source-level contracts for Agent Chat shortcut wiring across Notes and detached windows.

#[test]
fn detached_agent_chat_window_focuses_view_and_wires_history_callback() {
    let source = include_str!("../src/ai/agent_chat/ui/chat_window.rs");

    assert!(
        source.contains("let focus_handle = entity.read(cx).focus_handle(cx);")
            && source.contains("window.focus(&focus_handle, cx);"),
        "Detached Agent Chat activation should restore focus to the AgentChatView so window shortcuts keep working"
    );
    assert!(
        source.contains("if existing.is_some() {")
            && source.contains("activate_chat_window(cx);"),
        "Reusing an open detached Agent Chat window must restore the chat view focus, not just bring the shell forward"
    );

    assert!(
        source.contains("view.set_on_open_history_command")
            && source.contains("let _ = open_detached_history_actions(cx);"),
        "Detached Agent Chat windows should wire Cmd+P through the host-owned ActionsDialog history route"
    );
}

#[test]
fn notes_cmd_shift_a_routes_through_existing_embedded_agent_chat_path() {
    let source = include_str!("../src/notes/window/keyboard.rs");

    assert!(
        source.contains("self.request_focus_surface(")
            && source.contains("focus::NotesFocusSurface::AgentChat")
            && source.contains("open_selected_note_cart_in_embedded_agent_chat(\"NotesWindowCmdShiftA\"")
            && source.contains("\"NotesWindowCmdShiftA\""),
        "Notes Cmd+Shift+A should reuse the embedded Agent Chat cart handoff path instead of duplicating AI routing"
    );
}

#[test]
fn notes_agent_chat_history_uses_actions_route() {
    let source = include_str!("../src/notes/window/agent_chat_host.rs");

    assert!(
        source.contains("chat.set_on_open_history_command")
            && source.contains("let _ = app.open_agent_chat_history_actions(window, cx);"),
        "Notes-hosted Agent Chat Cmd+P should open the Notes-anchored ActionsDialog history route"
    );
    assert!(
        source.contains("action_id.strip_prefix(crate::actions::AGENT_CHAT_HISTORY_SELECT_ACTION_PREFIX)")
            && source.contains("chat.select_history_session_by_id(session_id, cx)"),
        "Notes-hosted Agent Chat history rows should dispatch back into the embedded Agent Chat view by session id"
    );
}

#[test]
fn global_cmd_enter_uses_return_preserving_agent_chat_entry_helper() {
    let source = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
    let fn_start = source
        .find("pub(crate) fn try_route_global_cmd_enter_to_agent_chat_context_capture(")
        .expect("global Cmd+Enter helper must exist");
    let fn_body = &source[fn_start..];

    assert!(
        fn_body
            .contains("self.open_tab_ai_agent_chat_with_entry_intent_preserving_return(None, cx);"),
        "global Cmd+Enter must route through the return-preserving Agent Chat helper"
    );
}

#[test]
fn entry_intent_return_helper_restores_previous_state_on_short_circuit() {
    let source = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
    let fn_start = source
        .find("fn open_tab_ai_agent_chat_with_entry_intent_preserving_return_and_options(")
        .expect("entry-intent preserving helper must exist");
    let fn_body = &source[fn_start..];

    assert!(
        fn_body.contains("tab_ai_entry_intent_return_seeded"),
        "entry-intent helper must log seeded return origin"
    );
    assert!(
        fn_body.contains("tab_ai_entry_intent_return_restored_without_launch"),
        "entry-intent helper must restore prior return origin when Agent Chat launch short-circuits"
    );
}
