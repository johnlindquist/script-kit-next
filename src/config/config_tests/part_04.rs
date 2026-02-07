#[test]
fn test_config_with_process_limits() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: None,
        terminal_font_size: None,
        ui_scale: None,
        built_ins: None,
        process_limits: Some(ProcessLimits {
            max_memory_mb: Some(512),
            max_runtime_seconds: Some(300),
            health_check_interval_ms: 3000,
        }),
        clipboard_history_max_text_length: None,
        suggested: None,
        notes_hotkey: None,
        ai_hotkey: None,
        logs_hotkey: None,
        ai_hotkey_enabled: None,
        logs_hotkey_enabled: None,
        watcher: None,
        layout: None,
        commands: None,
        claude_code: None,
    };

    let limits = config.get_process_limits();
    assert_eq!(limits.max_memory_mb, Some(512));
    assert_eq!(limits.max_runtime_seconds, Some(300));
    assert_eq!(limits.health_check_interval_ms, 3000);
}

#[test]
fn test_config_get_process_limits_default() {
    let config = Config::default();
    let limits = config.get_process_limits();

    // Should return defaults when process_limits is None
    assert_eq!(limits.max_memory_mb, None);
    assert_eq!(limits.max_runtime_seconds, None);
    assert_eq!(
        limits.health_check_interval_ms,
        DEFAULT_HEALTH_CHECK_INTERVAL_MS
    );
}

#[test]
fn test_config_get_process_limits_uses_default_interval_when_zero() {
    let config = Config {
        process_limits: Some(ProcessLimits {
            max_memory_mb: Some(1024),
            max_runtime_seconds: Some(60),
            health_check_interval_ms: 0,
        }),
        ..Config::default()
    };

    let limits = config.get_process_limits();

    assert_eq!(limits.max_memory_mb, Some(1024));
    assert_eq!(limits.max_runtime_seconds, Some(60));
    assert_eq!(
        limits.health_check_interval_ms,
        DEFAULT_HEALTH_CHECK_INTERVAL_MS
    );
}

#[test]
fn test_config_deserialization_with_process_limits() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        },
        "processLimits": {
            "maxMemoryMb": 1024,
            "maxRuntimeSeconds": 600,
            "healthCheckIntervalMs": 2000
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    assert!(config.process_limits.is_some());
    let limits = config.get_process_limits();
    assert_eq!(limits.max_memory_mb, Some(1024));
    assert_eq!(limits.max_runtime_seconds, Some(600));
    assert_eq!(limits.health_check_interval_ms, 2000);
}

#[test]
fn test_config_deserialization_without_process_limits() {
    // Existing configs without processLimits should still work
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    assert!(config.process_limits.is_none());

    // Getter should return defaults
    let limits = config.get_process_limits();
    assert_eq!(limits.max_memory_mb, None);
    assert_eq!(limits.max_runtime_seconds, None);
    assert_eq!(
        limits.health_check_interval_ms,
        DEFAULT_HEALTH_CHECK_INTERVAL_MS
    );
}

#[test]
fn test_config_serialization_skips_none_process_limits() {
    let config = Config::default();
    let json = serde_json::to_string(&config).unwrap();

    // None values should not appear in JSON
    assert!(!json.contains("processLimits"));
}

#[test]
fn test_config_serialization_includes_set_process_limits() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: None,
        terminal_font_size: None,
        ui_scale: None,
        built_ins: None,
        process_limits: Some(ProcessLimits::default()),
        clipboard_history_max_text_length: None,
        suggested: None,
        notes_hotkey: None,
        ai_hotkey: None,
        logs_hotkey: None,
        ai_hotkey_enabled: None,
        logs_hotkey_enabled: None,
        watcher: None,
        layout: None,
        commands: None,
        claude_code: None,
    };

    let json = serde_json::to_string(&config).unwrap();

    assert!(json.contains("processLimits"));
    assert!(json.contains("healthCheckIntervalMs"));
}

#[test]
fn test_process_limits_roundtrip() {
    // Test full roundtrip serialization/deserialization
    let original = ProcessLimits {
        max_memory_mb: Some(256),
        max_runtime_seconds: Some(120),
        health_check_interval_ms: 10000,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: ProcessLimits = serde_json::from_str(&json).unwrap();

    assert_eq!(original.max_memory_mb, restored.max_memory_mb);
    assert_eq!(original.max_runtime_seconds, restored.max_runtime_seconds);
    assert_eq!(
        original.health_check_interval_ms,
        restored.health_check_interval_ms
    );
}

#[test]
fn test_process_limits_clone() {
    let original = ProcessLimits {
        max_memory_mb: Some(512),
        max_runtime_seconds: Some(300),
        health_check_interval_ms: 5000,
    };
    let cloned = original.clone();

    assert_eq!(original.max_memory_mb, cloned.max_memory_mb);
    assert_eq!(original.max_runtime_seconds, cloned.max_runtime_seconds);
    assert_eq!(
        original.health_check_interval_ms,
        cloned.health_check_interval_ms
    );
}

// Confirmation required tests
#[test]
fn test_default_confirmation_commands_constant() {
    // Verify the constant contains expected dangerous commands
    assert!(DEFAULT_CONFIRMATION_COMMANDS.contains(&"builtin-shut-down"));
    assert!(DEFAULT_CONFIRMATION_COMMANDS.contains(&"builtin-restart"));
    assert!(DEFAULT_CONFIRMATION_COMMANDS.contains(&"builtin-log-out"));
    assert!(DEFAULT_CONFIRMATION_COMMANDS.contains(&"builtin-empty-trash"));
    assert!(DEFAULT_CONFIRMATION_COMMANDS.contains(&"builtin-sleep"));
    assert!(DEFAULT_CONFIRMATION_COMMANDS.contains(&"builtin-test-confirmation"));
}

#[test]
fn test_requires_confirmation_default_commands() {
    // Default commands should require confirmation
    let config = Config::default();

    assert!(config.requires_confirmation("builtin-shut-down"));
    assert!(config.requires_confirmation("builtin-restart"));
    assert!(config.requires_confirmation("builtin-log-out"));
    assert!(config.requires_confirmation("builtin-empty-trash"));
    assert!(config.requires_confirmation("builtin-sleep"));
    assert!(config.requires_confirmation("builtin-test-confirmation"));
}

#[test]
fn test_requires_confirmation_non_dangerous_commands() {
    // Non-dangerous commands should NOT require confirmation
    let config = Config::default();

    assert!(!config.requires_confirmation("builtin-clipboard-history"));
    assert!(!config.requires_confirmation("builtin-app-launcher"));
    assert!(!config.requires_confirmation("script/hello-world"));
    assert!(!config.requires_confirmation("app/com.apple.Safari"));
}

#[test]
fn test_requires_confirmation_user_override_disable() {
    // User can disable confirmation for a default dangerous command
    let mut commands = HashMap::new();
    commands.insert(
        "builtin-shut-down".to_string(),
        CommandConfig {
            shortcut: None,
            hidden: None,
            confirmation_required: Some(false), // User explicitly disables
        },
    );

    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: None,
        terminal_font_size: None,
        ui_scale: None,
        built_ins: None,
        process_limits: None,
        clipboard_history_max_text_length: None,
        suggested: None,
        notes_hotkey: None,
        ai_hotkey: None,
        logs_hotkey: None,
        ai_hotkey_enabled: None,
        logs_hotkey_enabled: None,
        watcher: None,
        layout: None,
        commands: Some(commands),
        claude_code: None,
    };

    // Should NOT require confirmation because user disabled it
    assert!(!config.requires_confirmation("builtin-shut-down"));
    // Other default commands still require it
    assert!(config.requires_confirmation("builtin-restart"));
}

#[test]
fn test_requires_confirmation_user_override_enable() {
    // User can enable confirmation for a non-default command
    let mut commands = HashMap::new();
    commands.insert(
        "script/dangerous-script".to_string(),
        CommandConfig {
            shortcut: None,
            hidden: None,
            confirmation_required: Some(true), // User explicitly enables
        },
    );

    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: None,
        terminal_font_size: None,
        ui_scale: None,
        built_ins: None,
        process_limits: None,
        clipboard_history_max_text_length: None,
        suggested: None,
        notes_hotkey: None,
        ai_hotkey: None,
        logs_hotkey: None,
        ai_hotkey_enabled: None,
        logs_hotkey_enabled: None,
        watcher: None,
        layout: None,
        commands: Some(commands),
        claude_code: None,
    };

    // Should require confirmation because user enabled it
    assert!(config.requires_confirmation("script/dangerous-script"));
    // Non-configured commands still use defaults
    assert!(!config.requires_confirmation("script/safe-script"));
}

#[test]
fn test_command_config_confirmation_required_serialization() {
    let cmd_config = CommandConfig {
        shortcut: None,
        hidden: None,
        confirmation_required: Some(true),
    };

    let json = serde_json::to_string(&cmd_config).unwrap();

    // Should use camelCase in JSON
    assert!(json.contains("confirmationRequired"));
    assert!(!json.contains("confirmation_required"));

    let deserialized: CommandConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.confirmation_required, Some(true));
}

#[test]
fn test_command_config_confirmation_required_deserialization() {
    let json = r#"{"confirmationRequired": false}"#;
    let cmd_config: CommandConfig = serde_json::from_str(json).unwrap();

    assert_eq!(cmd_config.confirmation_required, Some(false));
    assert!(cmd_config.shortcut.is_none());
    assert!(cmd_config.hidden.is_none());
}

#[test]
fn test_command_config_confirmation_required_skips_none() {
    let cmd_config = CommandConfig {
        shortcut: None,
        hidden: None,
        confirmation_required: None,
    };

    let json = serde_json::to_string(&cmd_config).unwrap();

    // None values should not appear in JSON
    assert!(!json.contains("confirmationRequired"));
}

#[test]
fn test_config_deserialization_with_confirmation_required() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        },
        "commands": {
            "builtin-shut-down": {
                "confirmationRequired": false
            },
            "script/my-script": {
                "confirmationRequired": true
            }
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    // User disabled confirmation for shut-down
    assert!(!config.requires_confirmation("builtin-shut-down"));
    // User enabled confirmation for custom script
    assert!(config.requires_confirmation("script/my-script"));
    // Other default commands still require it
    assert!(config.requires_confirmation("builtin-restart"));
}

// ============================================
// DEEPLINK URL TESTS
// ============================================

#[test]
fn test_command_id_to_deeplink_uses_scriptkit_scheme() {
    use crate::config::types::command_id_to_deeplink;

    // The app registers 'scriptkit' URL scheme, so deeplinks must use scriptkit://
    let deeplink = command_id_to_deeplink("builtin/clipboard-history");
    assert_eq!(deeplink, "scriptkit://commands/builtin/clipboard-history");

    let deeplink = command_id_to_deeplink("script/hello-world");
    assert_eq!(deeplink, "scriptkit://commands/script/hello-world");

    let deeplink = command_id_to_deeplink("app/com.apple.Safari");
    assert_eq!(deeplink, "scriptkit://commands/app/com.apple.Safari");

    let deeplink = command_id_to_deeplink("scriptlet/my-snippet");
    assert_eq!(deeplink, "scriptkit://commands/scriptlet/my-snippet");
}

#[test]
fn test_command_id_to_deeplink_not_kit_scheme() {
    use crate::config::types::command_id_to_deeplink;

    // Verify we're NOT using the old incorrect 'kit://' scheme
    let deeplink = command_id_to_deeplink("builtin/test");
    assert!(
        !deeplink.starts_with("kit://"),
        "Deeplink should NOT use kit:// scheme, got: {}",
        deeplink
    );
    assert!(
        deeplink.starts_with("scriptkit://"),
        "Deeplink should use scriptkit:// scheme, got: {}",
        deeplink
    );
}

#[test]
fn test_requires_confirmation_with_partial_command_config() {
    // Command config exists but doesn't specify confirmation_required
    // Should fall back to defaults
    let mut commands = HashMap::new();
    commands.insert(
        "builtin-shut-down".to_string(),
        CommandConfig {
            shortcut: Some(HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "KeyX".to_string(),
            }),
            hidden: None,
            confirmation_required: None, // Not specified
        },
    );

    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: None,
        terminal_font_size: None,
        ui_scale: None,
        built_ins: None,
        process_limits: None,
        clipboard_history_max_text_length: None,
        suggested: None,
        notes_hotkey: None,
        ai_hotkey: None,
        logs_hotkey: None,
        ai_hotkey_enabled: None,
        logs_hotkey_enabled: None,
        watcher: None,
        layout: None,
        commands: Some(commands),
        claude_code: None,
    };

    // Should still require confirmation (falls back to default)
    assert!(config.requires_confirmation("builtin-shut-down"));
}
