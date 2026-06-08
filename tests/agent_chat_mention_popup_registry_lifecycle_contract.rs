//! Source-level contract for Agent Chat mention popup automation registry lifecycle.

const PICKER: &str = include_str!("../src/ai/agent_chat/ui/picker_popup.rs");

fn function_body<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source.find(name).expect("function must exist");
    let tail = &source[start..];
    let end = tail.find("\n}\n\n").expect("function body terminator");
    &tail[..end]
}

#[test]
fn mention_popup_has_single_owner_unregister_helper() {
    assert!(PICKER.contains("fn unregister_mention_popup_automation_window()"));
    assert!(
        PICKER.contains("unregister_agent_chat_prompt_popup_automation_window")
            && PICKER.contains("AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID")
    );
}

#[test]
fn close_helper_unregisters_before_removing_popup_window() {
    let body = function_body(PICKER, "pub(crate) fn close_mention_popup_window");
    assert!(body.contains("unregister_mention_popup_automation_window();"));
    assert!(body.contains("window.remove_window();"));
}

#[test]
fn direct_accept_close_path_unregisters_registry_entry() {
    let click_start = PICKER
        .find("fn handle_row_click")
        .expect("mention popup click handler must exist");
    let click_body = &PICKER[click_start..];
    let accept_start = click_body
        .find("if should_accept")
        .expect("accept branch must exist");
    let accept_body = &click_body[accept_start..];
    assert!(
        accept_body.contains("unregister_mention_popup_automation_window();")
            && accept_body.contains("window.remove_window();"),
        "direct row-click close path must unregister before removing the window"
    );
}
