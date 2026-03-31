/// Drift-prevention tests: ensure the TypeScript config surface and skill docs
/// stay aligned with the Rust runtime contract.

#[test]
fn config_skill_mentions_root_skills_and_dictation_split() {
    let content = include_str!("../kit-init/skills/config/SKILL.md");
    assert!(
        content.contains("~/.scriptkit/skills/"),
        "SKILL.md must reference root skills path"
    );
    assert!(
        content.contains("~/.scriptkit/kit/config.ts"),
        "SKILL.md must reference kit/config.ts"
    );
    assert!(
        content.contains("~/.scriptkit/kit/settings.json"),
        "SKILL.md must reference kit/settings.json"
    );
    assert!(
        content.contains("dictationHotkey"),
        "SKILL.md must mention dictationHotkey"
    );
    assert!(
        content.contains("selectedDeviceId"),
        "SKILL.md must mention selectedDeviceId"
    );
}

#[test]
fn kit_sdk_mentions_current_public_config_fields() {
    let sdk = include_str!("../scripts/kit-sdk.ts");
    assert!(
        sdk.contains("~/.scriptkit/kit/config.ts"),
        "kit-sdk.ts must reference the authoritative config path"
    );
    assert!(
        !sdk.contains("~/.scriptkit/config.ts"),
        "kit-sdk.ts must not reference the legacy config path"
    );
    for field in [
        "suggested",
        "watcher",
        "layout",
        "notesHotkey",
        "aiHotkey",
        "aiHotkeyEnabled",
        "logsHotkey",
        "logsHotkeyEnabled",
        "dictationHotkey",
        "dictationHotkeyEnabled",
        "commands",
        "claudeCode",
    ] {
        assert!(sdk.contains(field), "kit-sdk.ts missing {field}");
    }
}

#[test]
fn config_cli_known_top_level_is_current() {
    let cli = include_str!("../scripts/config-cli.ts");
    for field in [
        "clipboardHistoryMaxTextLength",
        "suggested",
        "notesHotkey",
        "aiHotkey",
        "logsHotkey",
        "dictationHotkey",
        "watcher",
        "layout",
        "commands",
        "claudeCode",
    ] {
        assert!(cli.contains(field), "config-cli.ts missing {field}");
    }
}

#[test]
fn kit_sdk_exports_canonical_config_types() {
    let sdk = include_str!("../scripts/kit-sdk.ts");
    for type_name in [
        "export interface SuggestedConfig",
        "export interface WatcherConfig",
        "export interface LayoutConfig",
        "export interface HotkeyConfig",
        "export interface CommandConfig",
        "export interface ClaudeCodeConfig",
        "export interface Config",
    ] {
        assert!(sdk.contains(type_name), "kit-sdk.ts missing {type_name}");
    }
}

#[test]
fn kit_sdk_config_imports_from_canonical_source() {
    let config_sdk = include_str!("../scripts/kit-sdk-config.ts");
    assert!(
        config_sdk.contains("from \"./kit-sdk\""),
        "kit-sdk-config.ts must import types from ./kit-sdk"
    );
    // Must NOT contain its own interface definitions
    assert!(
        !config_sdk.contains("interface HotkeyConfig"),
        "kit-sdk-config.ts must not define its own HotkeyConfig"
    );
}

#[test]
fn config_template_uses_satisfies_config() {
    let template = include_str!("../kit-init/config-template.ts");
    assert!(
        template.contains("satisfies Config"),
        "config-template.ts must use satisfies Config"
    );
    assert!(
        template.contains("import type { Config }"),
        "config-template.ts must import Config"
    );
}
