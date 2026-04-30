//! Source-level contracts for ACP shortcut wiring across Notes and detached windows.

#[test]
fn detached_acp_window_focuses_view_and_wires_history_callback() {
    let source = include_str!("../src/ai/acp/chat_window.rs");

    assert!(
        source.contains("let focus_handle = entity.read(cx).focus_handle(cx);")
            && source.contains("window.focus(&focus_handle, cx);"),
        "Detached ACP activation should restore focus to the AcpChatView so window shortcuts keep working"
    );
    assert!(
        source.contains("if existing.is_some() {")
            && source.contains("activate_chat_window(cx);"),
        "Reusing an open detached ACP window must restore the chat view focus, not just bring the shell forward"
    );

    assert!(
        source.contains("view.set_on_open_history_command")
            && source.contains(
                "view.open_history_popup_from_host(parent_handle, parent_bounds, display_id, cx);"
            ),
        "Detached ACP windows should wire Cmd+P through the host-owned history popup callback"
    );
}

#[test]
fn notes_cmd_shift_a_routes_through_existing_embedded_acp_path() {
    let source = include_str!("../src/notes/window/keyboard.rs");

    assert!(
        source.contains("self.request_focus_surface(")
            && source.contains("focus::NotesFocusSurface::AcpChat")
            && source.contains("open_selected_note_cart_in_embedded_acp(\"NotesWindowCmdShiftA\"")
            && source.contains("\"NotesWindowCmdShiftA\""),
        "Notes Cmd+Shift+A should reuse the embedded ACP cart handoff path instead of duplicating AI routing"
    );
}

#[test]
fn global_cmd_enter_uses_return_preserving_acp_entry_helper() {
    let source = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
    let fn_start = source
        .find("pub(crate) fn try_route_global_cmd_enter_to_acp_context_capture(")
        .expect("global Cmd+Enter helper must exist");
    let fn_body = &source[fn_start..];

    assert!(
        fn_body.contains("self.open_tab_ai_acp_with_entry_intent_preserving_return(None, cx);"),
        "global Cmd+Enter must route through the return-preserving ACP helper"
    );
}

#[test]
fn entry_intent_return_helper_restores_previous_state_on_short_circuit() {
    let source = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
    let fn_start = source
        .find("fn open_tab_ai_acp_with_entry_intent_preserving_return_and_options(")
        .expect("entry-intent preserving helper must exist");
    let fn_body = &source[fn_start..];

    assert!(
        fn_body.contains("tab_ai_entry_intent_return_seeded"),
        "entry-intent helper must log seeded return origin"
    );
    assert!(
        fn_body.contains("tab_ai_entry_intent_return_restored_without_launch"),
        "entry-intent helper must restore prior return origin when ACP launch short-circuits"
    );
}
