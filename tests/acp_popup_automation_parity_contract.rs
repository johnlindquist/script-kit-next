//! Source-level contract for ACP PromptPopup automation parity.

const PICKER: &str = include_str!("../src/ai/acp/picker_popup.rs");
const HISTORY: &str = include_str!("../src/ai/acp/history_popup.rs");
const COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn all_acp_popups_have_exact_prompt_popup_automation_ids() {
    assert!(PICKER.contains("ACP_MENTION_POPUP_AUTOMATION_ID"));
    assert!(
        HISTORY.contains("ACP_HISTORY_POPUP_AUTOMATION_ID")
            && HISTORY.contains("\"acp-history-popup\"")
    );
}

#[test]
fn history_registers_as_attached_prompt_popup() {
    for (name, source) in [("history", HISTORY)] {
        assert!(
            source.contains("register_acp_prompt_popup_automation_window"),
            "{name} popup must register with the shared attached PromptPopup helper"
        );
        assert!(
            source.contains("AcpPopupRegistration::register"),
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
