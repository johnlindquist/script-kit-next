use std::{fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
// @lat: [[lat.md/tests/acp-portal-contract#Cmd+Enter origin parity#Label and HUD parity]]
fn agent_chat_labels_have_a_single_source() {
    let labels = read("src/ai/acp/labels.rs");
    for needle in [
        "AGENT_CHAT_LABEL",
        "AGENT_CHAT_CMD_ENTER_HINT",
        "AGENT_CHAT_OPEN_ACTION",
        "AGENT_CHAT_CHANGE_AGENT",
        "AGENT_CHAT_CHANGE_MODEL",
        "acp_entry_hint",
    ] {
        assert!(labels.contains(needle), "missing {needle}");
    }
}

#[test]
fn main_footer_and_hints_use_agent_chat_label_constants() {
    let ui = read("src/app_impl/ui_window.rs");
    assert!(ui.contains("crate::ai::acp::labels::AGENT_CHAT_LABEL"));

    let hints = read("src/components/prompt_layout_shell.rs");
    assert!(hints.contains("crate::ai::acp::labels::AGENT_CHAT_CMD_ENTER_HINT"));
}

#[test]
fn acp_footers_omit_global_cmd_enter_ai_button() {
    let acp_tests = read("src/ai/acp/tests.rs");
    assert!(acp_tests.contains("acp_footer_omits_global_cmd_enter_ai_button"));

    let acp_chat = read("lat.md/acp-chat.md");
    assert!(acp_chat.contains("Agent Chat footers expose ACP actions"));
}
