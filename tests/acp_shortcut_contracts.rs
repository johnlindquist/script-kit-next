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
            && source.contains("self.open_selected_note_in_embedded_acp(")
            && source.contains("\"NotesWindowCmdShiftA\""),
        "Notes Cmd+Shift+A should reuse the embedded ACP open/focus path instead of duplicating AI routing"
    );
}
