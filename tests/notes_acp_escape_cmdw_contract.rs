//! Source-level contract for Notes-hosted ACP Escape and Cmd+W behavior.

const KEYBOARD_SOURCE: &str = include_str!("../src/notes/window/keyboard.rs");
const ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");

fn block<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

// doc-anchor-removed: [[tests/notes-acp#Notes ACP Escape streaming cancellation#Escape cancels streaming before returning to editor]]
#[test]
fn notes_acp_escape_dismisses_popup_then_cancels_stream_then_returns_to_notes() {
    let acp_mode = block(
        KEYBOARD_SOURCE,
        "if self.surface_mode == NotesSurfaceMode::Acp",
        "// All other keys propagate to the ACP chat view.",
    );
    let dismiss = acp_mode.find("chat.dismiss_escape_popup(cx)").unwrap();
    let cancel = acp_mode
        .find("chat.cancel_streaming_from_escape(cx)")
        .unwrap();
    let switch = acp_mode
        .find("self.switch_to_notes_surface(window, cx)")
        .unwrap();
    assert!(
        dismiss < cancel && cancel < switch,
        "Notes ACP Escape must dismiss popup, cancel stream, then switch surfaces"
    );
}

// doc-anchor-removed: [[tests/notes-acp#Notes ACP CmdW window close cleanup#CmdW prepares ACP and closes dialogs before removing Notes]]
#[test]
fn notes_acp_cmd_w_prepares_acp_and_closes_dialogs_before_remove_window() {
    let cmd_w = block(
        KEYBOARD_SOURCE,
        "if key.eq_ignore_ascii_case(\"w\") && !modifiers.shift",
        "// All other keys propagate to the ACP chat view.",
    );
    let prepare = cmd_w
        .find("self.prepare_embedded_acp_for_window_close(\"notes_acp_cmd_w\", cx);")
        .unwrap();
    let save_bounds = cmd_w.find("WindowRole::Notes").unwrap();
    let close_dialogs = cmd_w.find("window.close_all_dialogs(cx);").unwrap();
    let remove = cmd_w.find("window.remove_window();").unwrap();
    assert!(prepare < save_bounds && save_bounds < close_dialogs && close_dialogs < remove);
    assert!(ACP_HOST_SOURCE.contains("pub(super) fn prepare_embedded_acp_for_window_close"));
    assert!(ACP_HOST_SOURCE.contains("crate::actions::close_actions_window(cx);"));
}
