//! Source-level contract for Notes-hosted Agent Chat Escape and Cmd+W behavior.

const KEYBOARD_SOURCE: &str = include_str!("../src/notes/window/keyboard.rs");
const AGENT_CHAT_HOST_SOURCE: &str = include_str!("../src/notes/window/agent_chat_host.rs");

fn block<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn notes_agent_chat_escape_dismisses_popup_then_cancels_stream_then_returns_to_notes() {
    let agent_chat_mode = block(
        KEYBOARD_SOURCE,
        "if self.surface_mode == NotesSurfaceMode::AgentChat",
        "// All other keys propagate to the Agent Chat chat view.",
    );
    let dismiss = agent_chat_mode
        .find("chat.dismiss_escape_popup(cx)")
        .unwrap();
    let cancel = agent_chat_mode
        .find("chat.cancel_streaming_from_escape(cx)")
        .unwrap();
    let switch = agent_chat_mode
        .find("self.switch_to_notes_surface(window, cx)")
        .unwrap();
    assert!(
        dismiss < cancel && cancel < switch,
        "Notes Agent Chat Escape must dismiss popup, cancel stream, then switch surfaces"
    );
}

#[test]
fn notes_agent_chat_cmd_w_prepares_agent_chat_and_closes_dialogs_before_remove_window() {
    let cmd_w = block(
        KEYBOARD_SOURCE,
        "if key.eq_ignore_ascii_case(\"w\") && !modifiers.shift",
        "// All other keys propagate to the Agent Chat chat view.",
    );
    let prepare = cmd_w
        .find("self.prepare_embedded_agent_chat_for_window_close(\"notes_agent_chat_cmd_w\", cx);")
        .unwrap();
    let save_bounds = cmd_w.find("WindowRole::Notes").unwrap();
    let close_dialogs = cmd_w.find("window.close_all_dialogs(cx);").unwrap();
    let remove = cmd_w.find("window.remove_window();").unwrap();
    assert!(prepare < save_bounds && save_bounds < close_dialogs && close_dialogs < remove);
    assert!(AGENT_CHAT_HOST_SOURCE
        .contains("pub(super) fn prepare_embedded_agent_chat_for_window_close"));
    assert!(AGENT_CHAT_HOST_SOURCE.contains("crate::actions::close_actions_window(cx);"));
}
