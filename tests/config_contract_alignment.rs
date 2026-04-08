/// Drift-prevention tests: ensure the TypeScript config surface and skill docs
/// stay aligned with the Rust runtime contract, and that canonical command IDs
/// round-trip correctly between config keys, deeplinks, and runtime resolution.

#[test]
fn config_skill_mentions_authoring_skills_and_dictation_split() {
    let content = include_str!("../kit-init/skills/config/SKILL.md");
    assert!(
        content.contains("~/.scriptkit/kit/authoring/skills/"),
        "SKILL.md must reference authoring plugin skills path"
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
fn kit_sdk_yields_focus_before_set_selected_text() {
    let sdk = include_str!("../scripts/kit-sdk.ts");
    let set_fn_start = sdk
        .find("globalThis.setSelectedText")
        .expect("kit-sdk.ts must define setSelectedText");
    let get_fn_start = sdk[set_fn_start..]
        .find("globalThis.getSelectedText")
        .map(|offset| set_fn_start + offset)
        .expect("setSelectedText block must end before getSelectedText");
    let set_block = &sdk[set_fn_start..get_fn_start];

    let hide_pos = set_block
        .find("await hide();")
        .expect("setSelectedText must hide before pasting into the frontmost app");
    let delay_pos = set_block
        .find("await new Promise(r => setTimeout(r, 20));")
        .expect("setSelectedText must briefly yield focus before sending the paste request");
    let send_pos = set_block
        .find("const message: SetSelectedTextMessage")
        .expect("setSelectedText must send a protocol request");

    assert!(
        hide_pos < delay_pos,
        "setSelectedText must wait only after it has hidden the main window"
    );
    assert!(
        delay_pos < send_pos,
        "setSelectedText must hide and yield focus before sending the paste request"
    );
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

// ============================================================================
// Command ID round-trip contract tests
// ============================================================================

#[test]
fn builtin_registry_ids_are_canonical_slash_style() {
    let entries = script_kit_gpui::builtins::get_builtin_entries(
        &script_kit_gpui::config::BuiltInConfig::default(),
    );
    for entry in &entries {
        assert!(
            entry.id.starts_with("builtin/"),
            "builtin id is not canonical: {}",
            entry.id
        );
        // Must not have double prefix like "builtin/builtin-..."
        let identifier = entry.id.strip_prefix("builtin/").unwrap();
        assert!(
            !identifier.starts_with("builtin-") && !identifier.starts_with("builtin/"),
            "builtin id has double prefix: {}",
            entry.id
        );
    }
}

#[test]
fn builtin_registry_roundtrips_with_deeplinks() {
    let entries = script_kit_gpui::builtins::get_builtin_entries(
        &script_kit_gpui::config::BuiltInConfig::default(),
    );
    for entry in &entries {
        let deeplink = script_kit_gpui::config::command_id_to_deeplink(&entry.id)
            .unwrap_or_else(|e| panic!("valid deeplink for {}: {}", entry.id, e));
        let parsed = script_kit_gpui::config::command_id_from_deeplink(&deeplink)
            .unwrap_or_else(|e| panic!("valid command id from {}: {}", deeplink, e));
        assert_eq!(parsed, entry.id, "round-trip failed for {}", entry.id);
    }
}

#[test]
fn confirmation_defaults_use_canonical_builtin_ids() {
    for command_id in script_kit_gpui::config::defaults::DEFAULT_CONFIRMATION_COMMANDS {
        assert!(
            command_id.starts_with("builtin/"),
            "default uses non-canonical builtin id: {}",
            command_id
        );
        // Verify each default parses as a valid command ID
        assert!(
            script_kit_gpui::config::is_valid_command_id(command_id),
            "default is not a valid command id: {}",
            command_id
        );
    }
}

#[test]
fn frecency_excluded_defaults_use_canonical_builtin_ids() {
    for command_id in script_kit_gpui::config::defaults::DEFAULT_FRECENCY_EXCLUDED_COMMANDS {
        assert!(
            command_id.starts_with("builtin/"),
            "frecency excluded uses non-canonical builtin id: {}",
            command_id
        );
    }
}

#[test]
fn shortcut_handlers_do_not_double_prefix_builtin_ids() {
    let source = include_str!("../src/app_actions/handle_action/shortcuts.rs");
    assert!(
        !source.contains("format!(\"builtin/{}\", m.entry.id)"),
        "shortcuts.rs still double-prefixes builtin ids"
    );
}

#[test]
fn execution_scripts_has_explicit_script_command_branch() {
    let source = include_str!("../src/app_impl/execution_scripts.rs");
    assert!(
        source.contains("CommandCategory::Script"),
        "script/{{name}} ids must resolve before legacy path fallback"
    );
}

#[test]
fn deeplink_parser_validates_through_command_id_helpers() {
    let source = include_str!("../src/main_sections/deeplink.rs");
    assert!(
        source.contains("command_id_from_deeplink"),
        "deeplink parser must use command_id_from_deeplink for validation"
    );
}

#[test]
fn script_command_id_deeplink_roundtrip() {
    let command_id = "script/hello-world";
    let deeplink = script_kit_gpui::config::command_id_to_deeplink(command_id)
        .expect("script command id should produce valid deeplink");
    assert_eq!(deeplink, "scriptkit://commands/script/hello-world");
    let parsed = script_kit_gpui::config::command_id_from_deeplink(&deeplink)
        .expect("should parse back to command id");
    assert_eq!(parsed, command_id);
}

#[test]
fn config_surfaces_document_canonical_excluded_command_ids() {
    let cli = include_str!("../scripts/config-cli.ts");
    let sdk = include_str!("../scripts/kit-sdk.ts");

    assert!(
        cli.contains("\"builtin/quit-script-kit\""),
        "config-cli.ts must use canonical builtin/quit-script-kit default"
    );
    assert!(
        !cli.contains("\"builtin-quit-script-kit\""),
        "config-cli.ts still contains dash-style builtin-quit-script-kit"
    );
    assert!(
        sdk.contains("excludedCommands?: CommandId[];"),
        "kit-sdk.ts must type SuggestedConfig.excludedCommands as CommandId[]"
    );
    assert!(
        sdk.contains("\"builtin/quit-script-kit\""),
        "kit-sdk.ts must document canonical builtin/quit-script-kit default"
    );
    assert!(
        !sdk.contains("\"builtin-quit-script-kit\""),
        "kit-sdk.ts still contains dash-style builtin-quit-script-kit"
    );
}

#[test]
fn canonical_builtin_command_id_handles_all_input_forms() {
    // Legacy dash-style
    assert_eq!(
        script_kit_gpui::config::canonical_builtin_command_id("builtin-clipboard-history"),
        "builtin/clipboard-history"
    );
    // Bare identifier
    assert_eq!(
        script_kit_gpui::config::canonical_builtin_command_id("clipboard-history"),
        "builtin/clipboard-history"
    );
    // Already canonical
    assert_eq!(
        script_kit_gpui::config::canonical_builtin_command_id("builtin/clipboard-history"),
        "builtin/clipboard-history"
    );
}
