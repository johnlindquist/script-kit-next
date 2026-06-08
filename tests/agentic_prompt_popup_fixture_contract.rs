//! Source-level contract for the promoted Prompt Popup screenshot fixture.
//!
//! Prompt Popup proves a protocol-only Agent Chat host setup before active
//! attached-popup screenshot capture.

const NAVIGATOR: &str = include_str!("../scripts/agentic/surface-navigator.ts");
const MATRIX: &str = include_str!("../scripts/agentic/attached-popup-surface-matrix.ts");
const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");
const AGENT_CHAT_PICKER_POPUP: &str = include_str!("../src/ai/agent_chat/ui/picker_popup.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_TAIL: &str = include_str!("../src/main_entry/runtime_stdin_match_tail.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

#[test]
fn prompt_popup_fixture_is_active_attached_popup_case() {
    assert!(
        MATRIX.contains("export const ATTACHED_POPUP_SURFACE_MATRIX")
            && MATRIX.contains("id: \"prompt-popup-on-agent_chat-chat-slash\"")
            && MATRIX.contains("viewName: \"prompt-popup-on-agent_chat-chat-slash\"")
            && MATRIX.contains("imageLibraryName: \"prompt-popup-on-agent_chat-chat-slash.png\"")
            && MATRIX.contains("windowKind: \"PromptPopup\"")
            && MATRIX.contains("targetKind: \"promptPopup\"")
            && MATRIX.contains("kind: \"promptPopup\"")
            && MATRIX.contains("expectedAutomationWindowId: \"agent_chat-mention-popup\""),
        "Prompt Popup must be included in the active attached-popup matrix with durable naming"
    );
    assert!(
        !MATRIX.contains("prompt-popup-on-agent_chat-chat-slash-candidate")
            && !MATRIX.contains("PROMPT_POPUP_FIXTURE_MATRIX"),
        "promoted Prompt Popup must not keep candidate-only matrix artifacts"
    );
}

#[test]
fn prompt_popup_fixture_uses_protocol_agent_chat_input_not_native_input() {
    assert!(
        MATRIX.contains("hostFixture: { kind: \"agent_chat-chat\", trigger: \"slash\" }"),
        "Prompt Popup must declare Agent Chat chat as the real host fixture"
    );
    assert!(
        NAVIGATOR.contains("type: \"triggerBuiltin\", name: \"tab-ai\"")
            && NAVIGATOR.contains("type: \"setAgentChatInput\"")
            && NAVIGATOR.contains("text: \"/\"")
            && NAVIGATOR.contains("submit: false"),
        "Prompt Popup setup must open Agent Chat and trigger slash popup through protocol commands"
    );
    assert!(
        !NAVIGATOR.contains("macos-input.ts"),
        "Prompt Popup must not depend on native input helpers"
    );
}

#[test]
fn protocol_agent_chat_input_refreshes_prompt_popup_session() {
    let set_input = AGENT_CHAT_VIEW
        .find("pub(crate) fn set_input(&mut self, value: String")
        .expect("AgentChatView::set_input must exist for protocol setAgentChatInput");
    let refresh = AGENT_CHAT_VIEW[set_input..]
        .find("self.refresh_mention_session(cx);")
        .expect("protocol Agent Chat input must refresh the slash/mention popup session");
    assert!(
        refresh < 220,
        "AgentChatView::set_input must immediately refresh the Prompt Popup session"
    );
    assert!(
        AGENT_CHAT_VIEW.contains("pub(crate) fn set_input_in_window")
            && AGENT_CHAT_VIEW.contains("self.cache_popup_parent_window(window, cx);")
            && AGENT_CHAT_VIEW.contains("self.set_input(value, cx);"),
        "protocol Agent Chat input must have a window-aware setter for attached Prompt Popup geometry"
    );
    for (path, source) in [
        ("runtime_stdin.rs", RUNTIME_STDIN),
        ("runtime_stdin_match_tail.rs", RUNTIME_STDIN_TAIL),
        ("app_run_setup.rs", APP_RUN_SETUP),
    ] {
        assert!(
            source.contains("chat.set_input_in_window(text.clone(), window, cx);"),
            "{path} setAgentChatInput arm must cache parent window geometry before refreshing the picker"
        );
    }
}

#[test]
fn prompt_popup_target_is_promoted_to_exact_id_with_crop_bounds() {
    assert!(
        AGENT_CHAT_PICKER_POPUP.contains("AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID")
            && AGENT_CHAT_PICKER_POPUP.contains("register_mention_popup_automation_window")
            && AGENT_CHAT_PICKER_POPUP.contains("register_attached_popup(")
            && AGENT_CHAT_PICKER_POPUP.contains("AutomationWindowKind::PromptPopup")
            && AGENT_CHAT_PICKER_POPUP.contains("remove_automation_window(AGENT_CHAT_MENTION_POPUP_AUTOMATION_ID)"),
        "Agent Chat mention popup must register and unregister as a PromptPopup attached automation window"
    );
    assert!(
        NAVIGATOR.contains("promoteAttachedPopupTarget")
            && NAVIGATOR.contains("response.windowKind === entry.windowKind")
            && NAVIGATOR.contains("entry.expectedAutomationWindowId")
            && NAVIGATOR.contains("automationWindowId !== entry.expectedAutomationWindowId")
            && NAVIGATOR.contains("targetBoundsInScreenshot")
            && NAVIGATOR.contains("osWindowId"),
        "Prompt Popup must promote kind target to exact automation id only after PromptPopup crop bounds exist"
    );
    assert!(
        NAVIGATOR.contains("expectedPopupCaptureStrategy")
            && NAVIGATOR.contains("parent_capture_with_crop")
            && NAVIGATOR.contains("popupCapture.targetBounds"),
        "Prompt Popup capture must require parent_capture_with_crop and target bounds"
    );
}

#[test]
fn prompt_popup_collects_semantic_receipts_before_capture() {
    assert!(
        NAVIGATOR.contains("getElementsForTarget")
            && NAVIGATOR.contains("pre-capture-elements")
            && NAVIGATOR.contains("preCaptureElements")
            && NAVIGATOR.contains("preCaptureInspection"),
        "Prompt Popup must preserve exact-target semantic receipts before screenshot capture"
    );
}
