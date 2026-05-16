const KIT_SDK: &str = include_str!("../scripts/kit-sdk.ts");
const EXECUTE_SCRIPT: &str = include_str!("../src/execute_script/mod.rs");
const PROMPT_MESSAGES: &str = include_str!("../src/main_sections/prompt_messages.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_SIMULATE_KEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const SHORTCUT_RECORDER: &str = include_str!("../src/app_impl/shortcut_recorder.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

#[test]
fn sdk_hotkey_routes_to_real_host_prompt() {
    // doc-anchor-removed: [[shortcuts#Transient SDK hotkey capture]]
    assert!(KIT_SDK.contains("globalThis.hotkey = async function hotkey("));
    assert!(!KIT_SDK.contains("hotkey() is not yet implemented"));
    assert!(KIT_SDK.contains("const message: HotkeyMessage = {"));
    assert!(KIT_SDK.contains("type: 'hotkey'"));
    assert!(EXECUTE_SCRIPT.contains("Some(PromptMessage::ShowHotkey { id, placeholder })"));
    assert!(PROMPT_MESSAGES.contains("ShowHotkey"));
    assert!(PROMPT_HANDLER.contains("PromptMessage::ShowHotkey { id, placeholder }"));
    assert!(PROMPT_HANDLER.contains("AppView::HotkeyPrompt { id, entity }"));
}

#[test]
fn hotkey_prompt_is_transient_and_does_not_use_persistent_shortcut_save() {
    // doc-anchor-removed: [[shortcuts#Transient SDK hotkey capture#Does not mutate shortcut config]]
    let show_hotkey = PROMPT_HANDLER
        .split("PromptMessage::ShowHotkey { id, placeholder } => {")
        .nth(1)
        .expect("ShowHotkey arm missing")
        .split("PromptMessage::WidgetComingSoon")
        .next()
        .expect("ShowHotkey arm should precede widget arm");

    assert!(show_hotkey.contains("shortcut_recorder::ShortcutRecorder::new"));
    assert!(show_hotkey.contains("Transient capture for SDK hotkey()"));
    assert!(!show_hotkey.contains("show_shortcut_recorder"));
    assert!(!show_hotkey.contains("handle_shortcut_save"));
    assert!(!show_hotkey.contains("write_config_command_shortcut"));
    assert!(!show_hotkey.contains("update_script_hotkey"));

    assert!(SHORTCUT_RECORDER.contains("write_config_command_shortcut"));
    assert!(SHORTCUT_RECORDER.contains("update_script_hotkey"));
}

#[test]
fn hotkey_prompt_has_state_first_capture_and_cancel_receipts() {
    // doc-anchor-removed: [[shortcuts#Transient SDK hotkey capture#Automation receipts]]
    for source in [
        RUNTIME_STDIN,
        RUNTIME_STDIN_MATCH_SIMULATE_KEY,
        APP_RUN_SETUP,
    ] {
        let arm = source
            .split("AppView::HotkeyPrompt { entity, id, .. }")
            .nth(1)
            .expect("HotkeyPrompt simulateKey arm missing")
            .split("AppView::ChatPrompt")
            .next()
            .expect("HotkeyPrompt arm should precede ChatPrompt");
        assert!(arm.contains("SimulateKey: cancel HotkeyPrompt"));
        assert!(arm.contains("has_cmd && key_lower == \"w\""));
        assert!(arm.contains("submit_prompt_response(prompt_id_clone, None"));
        assert!(arm.contains("cancel_script_execution"));
        assert!(arm.contains("prompt.handle_key_down(&key_lower, modifiers, cx)"));
        assert!(arm.contains("prompt.shortcut.to_hotkey_info_json()"));
        assert!(arm.contains("SimulateKey: captured HotkeyPrompt shortcut"));
    }

    assert!(COLLECT_ELEMENTS.contains("collect_hotkey_prompt_elements"));
    assert!(COLLECT_ELEMENTS.contains("\"hotkey-capture\""));
    assert!(COLLECT_ELEMENTS.contains("\"hotkey-shortcut\""));
}
