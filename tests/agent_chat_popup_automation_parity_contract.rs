//! Source-level contract for Agent Chat PromptPopup automation parity.

const PICKER: &str = include_str!("../src/ai/agent_chat/ui/picker_popup.rs");
const HISTORY: &str = include_str!("../src/ai/agent_chat/ui/history_popup.rs");
const COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn all_agent_chat_popups_have_exact_prompt_popup_automation_ids() {
    assert!(PICKER.contains("AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID"));
    assert!(
        HISTORY.contains("AGENT_CHAT_HISTORY_POPUP_AUTOMATION_ID")
            && HISTORY.contains("\"agent_chat-history-popup\"")
    );
}

#[test]
fn history_registers_as_attached_prompt_popup() {
    for (name, source) in [("history", HISTORY)] {
        assert!(
            source.contains("register_agent_chat_prompt_popup_automation_window"),
            "{name} popup must register with the shared attached PromptPopup helper"
        );
        assert!(
            source.contains("AgentChatPopupRegistration::register"),
            "{name} popup must publish an exact runtime window handle through the facade"
        );
    }
}

#[test]
fn automation_collector_and_read_target_include_history_without_model_selector() {
    assert!(COLLECTOR.contains("collect_history_popup_snapshot(cx)"));
    assert!(PROMPT_HANDLER.contains("is_history_popup_window_open()"));
    assert!(!COLLECTOR.contains("collect_model_selector_snapshot"));
    assert!(!PROMPT_HANDLER.contains("is_model_selector_popup_window_open"));
}
