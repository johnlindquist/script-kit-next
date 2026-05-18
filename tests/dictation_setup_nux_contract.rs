//! Source-level contract checks for the guided dictation setup NUX.

const BUILTINS: &str = include_str!("../src/builtins/mod.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");
const ARG_HELPERS: &str = include_str!("../src/render_prompts/arg/helpers.rs");
const SETTINGS: &str = include_str!("../src/render_builtins/settings.rs");
const SETUP_MODEL: &str = include_str!("../src/dictation/setup.rs");
const CONFIG_TYPES: &str = include_str!("../src/config/types.rs");
const DICTATION_WINDOW: &str = include_str!("../src/dictation/window.rs");
const DEVICE: &str = include_str!("../src/dictation/device.rs");
const INFO_PLIST_EXT: &str = include_str!("../assets/Info.plist.ext");
const STDIN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const STDIN_MATCH: &str = include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const CONFIG_TEMPLATE: &str = include_str!("../kit-init/config-template.ts");
const GUIDE: &str = include_str!("../kit-init/GUIDE.md");
const UPDATE_CONFIG_EXAMPLES: &str =
    include_str!("../kit-init/skills/update-config/references/config-examples.md");
const KIT_SDK: &str = include_str!("../scripts/kit-sdk.ts");

#[test]
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Settings entry opens setup]]
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
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Pure readiness model]]
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
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Non-prompting microphone preflight]]
fn microphone_permission_preflight_is_read_only() {
    let permission_body = DEVICE
        .split("pub fn microphone_permission_status()")
        .nth(1)
        .expect("microphone_permission_status must exist")
        .split("pub fn request_microphone_permission()")
        .next()
        .expect("microphone_permission_status body must be delimited before active request");
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
fn released_bundle_declares_microphone_usage_reason() {
    assert!(
        INFO_PLIST_EXT.contains("NSMicrophoneUsageDescription")
            && INFO_PLIST_EXT.contains("local dictation audio"),
        "release bundle Info.plist extension must include microphone purpose copy for macOS TCC prompts"
    );
}

#[test]
fn explicit_dictation_start_requests_undetermined_microphone_permission() {
    assert!(
        DEVICE.contains("pub fn request_microphone_permission()")
            && DEVICE.contains("requestAccessForMediaType"),
        "explicit dictation start must have an active microphone request path"
    );
    assert!(
        BUILTIN_EXECUTION.contains("DictationMicrophonePermissionStatus::NotDetermined")
            && BUILTIN_EXECUTION.contains("request_microphone_permission()")
            && BUILTIN_EXECUTION.contains("System Settings → Privacy & Security → Microphone"),
        "dictation start preflight must prompt on first use and provide denied-state guidance"
    );
}

#[test]
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Missing readiness opens setup]]
fn dictation_start_paths_open_setup_when_microphone_is_not_ready() {
    assert!(
        BUILTIN_EXECUTION.contains("fn prepare_dictation_builtin_start(")
            && BUILTIN_EXECUTION.contains("open_dictation_setup_if_microphone_not_ready(cx)"),
        "shared dictation start preflight must open setup before starting capture when microphone readiness fails"
    );

    for (marker, action) in [
        (
            "BuiltInFeature::Dictation =>",
            "DictationBuiltinAction::CurrentSurface",
        ),
        (
            "BuiltInFeature::DictationToAiHarness =>",
            "DictationBuiltinAction::AgentChat",
        ),
        (
            "BuiltInFeature::DictationToFrontmostApp =>",
            "DictationBuiltinAction::FrontmostApp",
        ),
        (
            "BuiltInFeature::DictationToNotes =>",
            "DictationBuiltinAction::Notes",
        ),
    ] {
        let start = BUILTIN_EXECUTION
            .find(marker)
            .unwrap_or_else(|| panic!("{} arm must exist", marker));
        let body = &BUILTIN_EXECUTION[start..start + 300.min(BUILTIN_EXECUTION.len() - start)];
        assert!(
            body.contains(action),
            "{} must route through {} so shared setup preflight applies",
            marker,
            action
        );
    }
}

#[test]
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Safe download prompt actions]]
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
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Protocol Enter submits setup prompt]]
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
// doc-anchor-removed: [[tests/dictation-setup-nux#Dictation Setup NUX#Hotkey guidance reports config-owned default]]
fn ready_copy_reports_config_owned_default_hotkey() {
    assert!(
        BUILTIN_EXECUTION.contains("configured hotkey (default: ⌘⇧;)"),
        "ready copy must report the config-owned default shortcut"
    );
    assert!(
        BUILTIN_EXECUTION.contains("Start dictation from the launcher or the configured hotkey"),
        "ready copy must tell the user how to start dictation"
    );
}

#[test]
fn dictation_hotkey_default_is_owned_by_config_not_overlay_ui() {
    assert!(
        CONFIG_TYPES.contains("pub fn default_dictation_hotkey() -> Self")
            && CONFIG_TYPES.contains("key: \"Semicolon\".to_string()")
            && CONFIG_TYPES.contains("unwrap_or_else(HotkeyConfig::default_dictation_hotkey)"),
        "Config::get_dictation_hotkey must supply the Cmd+Shift+; default"
    );
    assert!(
        CONFIG_TEMPLATE
            .contains("dictationHotkey: { modifiers: [\"meta\", \"shift\"], key: \"Semicolon\" }"),
        "fresh ~/.scriptkit/config.ts must define the default dictation shortcut"
    );
    assert!(
        GUIDE.contains("Defaults to Cmd+Shift+; when enabled and unset.")
            && GUIDE.contains(
                "dictationHotkey: { modifiers: [\"meta\", \"shift\"], key: \"Semicolon\" }"
            )
            && !GUIDE.contains("No default dictation shortcut")
            && !GUIDE.contains("key: \"KeyD\""),
        "generated user guide must describe the config-owned dictation shortcut default"
    );
    assert!(
        UPDATE_CONFIG_EXAMPLES.contains("key: \"Semicolon\"")
            && !UPDATE_CONFIG_EXAMPLES.contains("key: \"KeyD\""),
        "update-config skill examples must use the config-owned dictation shortcut default"
    );
    assert!(
        KIT_SDK.contains("@default { modifiers: [\"meta\", \"shift\"], key: \"Semicolon\" }"),
        "SDK Config docs must expose the dictationHotkey default"
    );
    assert!(
        !DICTATION_WINDOW.contains("key: \"Semicolon\".to_string()"),
        "overlay UI must read the dictation keycap from config instead of hard-coding the default"
    );
}
