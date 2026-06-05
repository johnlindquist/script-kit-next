//! Source-level contract for the dictation overlay microphone picker.
//!
//! The overlay mic button must use an attached PromptPopup-style window, expose
//! safe automation rows, and persist through the shared dictation device path.

const POPUP: &str = include_str!("../src/dictation/microphone_popup_window.rs");
const WINDOW: &str = include_str!("../src/dictation/window.rs");
const DEVICE: &str = include_str!("../src/dictation/device.rs");
const COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn overlay_mic_button_opens_attached_prompt_popup_instead_of_cycling() {
    assert!(
        WINDOW.contains("fn open_microphone_picker(&mut self, window: &mut Window")
            && WINDOW.contains("sync_dictation_microphone_popup_window(cx, request)")
            && WINDOW.contains("this.open_microphone_picker(window, cx)")
            && !WINDOW.contains("fn cycle_microphone(")
            && !WINDOW.contains("this.cycle_microphone(cx)"),
        "dictation overlay mic button must open the popup selector instead of cycling microphones"
    );
}

#[test]
fn popup_uses_trigger_popup_window_primitives_and_prompt_popup_registration() {
    assert!(
        POPUP.contains("configure_inline_popup_window")
            && POPUP.contains("inline_popup_window_options")
            && POPUP.contains("set_inline_popup_window_bounds")
            && POPUP.contains("dictation_microphone_popup_bounds_above")
            && POPUP.contains("INLINE_POPUP_EDGE_GUTTER")
            && POPUP.contains("parent_bounds.origin.y.as_f32() - height")
            && POPUP.contains("register_attached_popup")
            && POPUP.contains("AutomationWindowKind::PromptPopup")
            && POPUP.contains("dictationMicrophonePopup"),
        "dictation mic selector must reuse attached trigger-popup window primitives and appear above the overlay"
    );
}

#[test]
fn popup_selection_persists_through_shared_device_helper() {
    assert!(
        POPUP.contains("apply_device_selection(&row.action)")
            && DEVICE.contains("pub fn apply_device_selection")
            && DEVICE.contains("save_dictation_device_id(None)")
            && DEVICE.contains("save_dictation_device_id(Some(device_id.0.as_str()))")
            && DEVICE.contains("notify_dictation_device_preference_changed()"),
        "dictation popup selections must update the same persisted microphone preference as settings"
    );
}

#[test]
fn popup_automation_rows_are_safe_and_batch_selectable() {
    assert!(
        POPUP.contains("DICTATION_MICROPHONE_POPUP_AUTOMATION_ID")
            && POPUP.contains("dictation-mic-row-{idx}")
            && POPUP.contains("choice:{idx}:{row_id}")
            && COLLECTOR.contains("\"panel:dictation-microphone-popup\"")
            && COLLECTOR.contains("\"list:dictation-microphones\"")
            && COLLECTOR.contains("Some(row.row_id.clone())")
            && PROMPT_HANDLER.contains("batch_select_dictation_microphone_popup_row_by_value")
            && PROMPT_HANDLER
                .contains("batch_select_dictation_microphone_popup_row_by_semantic_id"),
        "dictation popup must expose safe row ids and route PromptPopup batch selection"
    );
}

#[test]
fn overlay_mic_control_uses_select_label_and_icon_not_keycap_value() {
    assert!(
        WINDOW.contains("const ACTION_MIC_LABEL: &str = \"Select Mic\";")
            && WINDOW.contains(
                "const MIC_KEYCAP: &str = crate::components::footer_chrome::FOOTER_MIC_ICON_TOKEN;"
            )
            && WINDOW.contains("fn render_mic_action_chip_content(")
            && WINDOW.contains(
                ".external_path(crate::components::footer_chrome::FOOTER_MIC_ICON_PATH)"
            )
            && WINDOW.contains("FooterButtonConfig::new(")
            && WINDOW.contains("FooterAction::Ai,")
            && WINDOW.contains("MIC_KEYCAP,")
            && WINDOW.contains("active_microphone_footer_label(),")
            && WINDOW.contains("crate::dictation::get_active_dictation_device()")
            && !WINDOW.contains("fn current_microphone_label()")
            && !WINDOW.contains("FooterHintKeyMode::TextValue")
            && !WINDOW.contains("fn current_microphone_keycap()")
            && !WINDOW.contains("const MAX_CHARS: usize = 8"),
        "overlay mic control must render as a left-side mic glyph action, not a text/keycap value chip"
    );
}
