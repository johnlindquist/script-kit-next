//! Source-level contract checks for the guided dictation setup NUX.

const BUILTINS: &str = include_str!("../src/builtins/mod.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");
const ARG_HELPERS: &str = include_str!("../src/render_prompts/arg/helpers.rs");
const SETTINGS: &str = include_str!("../src/render_builtins/settings.rs");
const SETUP_MODEL: &str = include_str!("../src/dictation/setup.rs");
const DEVICE: &str = include_str!("../src/dictation/device.rs");
const STDIN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const STDIN_MATCH: &str = include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Settings entry opens setup]]
fn settings_exposes_first_class_dictation_setup_entry() {
    assert!(
        BUILTINS.contains("SettingsCommandType::DictationSetup"),
        "settings command enum must include DictationSetup"
    );
    assert!(
        BUILTINS.contains("\"builtin/dictation-setup\""),
        "builtins registry must expose builtin/dictation-setup"
    );
    assert!(
        SETTINGS.contains("name: \"Dictation Setup\"")
            && SETTINGS.contains("SettingsAction::DictationSetup")
            && SETTINGS.contains("SettingsCommandType::DictationSetup"),
        "settings hub must show Dictation Setup and route it through the builtin"
    );
}

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Pure readiness model]]
fn setup_state_model_keeps_hotkey_optional_but_microphone_required() {
    assert!(
        SETUP_MODEL.contains("pub struct DictationSetupState")
            && SETUP_MODEL.contains("pub ready: bool")
            && SETUP_MODEL.contains("matches!(microphone_status, DictationMicrophoneStatus::Ready")
            && SETUP_MODEL.contains("DictationHotkeyStatus::NotConfigured"),
        "setup model must summarize readiness without making hotkey presence a recording prerequisite"
    );
}

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Non-prompting microphone preflight]]
fn microphone_permission_preflight_is_read_only() {
    let permission_body = DEVICE
        .split("pub fn microphone_permission_status()")
        .nth(1)
        .expect("microphone_permission_status must exist");
    assert!(
        permission_body.contains("authorizationStatusForMediaType"),
        "microphone preflight must use AVFoundation authorization status"
    );
    assert!(
        !permission_body.contains("requestAccessForMediaType"),
        "opening setup must not request microphone permission as a hidden side effect"
    );
}

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Missing readiness opens setup]]
fn dictation_start_paths_open_setup_when_microphone_is_not_ready() {
    for marker in [
        "BuiltInFeature::Dictation =>",
        "BuiltInFeature::DictationToAiHarness =>",
        "BuiltInFeature::DictationToFrontmostApp =>",
        "BuiltInFeature::DictationToNotes =>",
    ] {
        let start = BUILTIN_EXECUTION
            .find(marker)
            .unwrap_or_else(|| panic!("{} arm must exist", marker));
        let body = &BUILTIN_EXECUTION[start..start + 2600.min(BUILTIN_EXECUTION.len() - start)];
        assert!(
            body.contains("open_dictation_setup_if_microphone_not_ready(cx)"),
            "{} must open setup before starting capture when microphone readiness fails",
            marker
        );
    }
}

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Safe download prompt actions]]
fn dictation_setup_prompt_preserves_safe_download_actions() {
    assert!(
        BUILTIN_EXECUTION.contains("transcription is local after Parakeet installs"),
        "setup copy must explain the local transcription boundary after install"
    );
    assert!(
        BUILTIN_EXECUTION.contains("DictationModelStatus::Downloading { .. } => 1"),
        "downloading state must default to Hide, not Cancel download"
    );
    assert!(
        ARG_HELPERS.contains("BUILTIN_DICTATION_MODEL_HIDE"),
        "submit validation must allow Hide so repeated Enter cannot strand the prompt"
    );
    assert!(
        BUILTIN_EXECUTION.contains("Download continues in background"),
        "Hide action must tell the user the download continues"
    );
}

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Protocol Enter submits setup prompt]]
fn protocol_enter_submits_mini_setup_prompt() {
    for (label, source) in [
        ("app_run_setup", STDIN_SETUP),
        ("runtime_stdin_match", STDIN_MATCH),
    ] {
        assert!(
            source.contains("AppView::MiniPrompt { id, .. }"),
            "{} must route MiniPrompt through simulateKey",
            label
        );
        assert!(
            source.contains("SimulateKey: Enter - submit mini prompt selection")
                && source.contains("view.submit_arg_prompt_from_current_state("),
            "{} must submit the current MiniPrompt selection on protocol Enter",
            label
        );
    }
}

#[test]
// @lat: [[tests/dictation-setup-nux#Dictation Setup NUX#Hotkey guidance does not invent default]]
fn ready_copy_reports_real_hotkey_or_config_guidance() {
    assert!(
        BUILTIN_EXECUTION.contains("no default is assumed"),
        "hotkey guidance must not invent a default shortcut"
    );
    assert!(
        BUILTIN_EXECUTION.contains("Start dictation from the launcher or configured hotkey"),
        "ready copy must tell the user how to start dictation"
    );
}
