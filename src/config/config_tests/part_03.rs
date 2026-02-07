#[test]
fn test_config_deserialization_without_ui_settings() {
    // Existing configs without UI settings should still work
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    // All UI settings should be None
    assert!(config.padding.is_none());
    assert!(config.editor_font_size.is_none());
    assert!(config.terminal_font_size.is_none());
    assert!(config.ui_scale.is_none());

    // Getters should return defaults
    assert_eq!(config.get_padding().top, DEFAULT_PADDING_TOP);
    assert_eq!(config.get_editor_font_size(), DEFAULT_EDITOR_FONT_SIZE);
    assert_eq!(config.get_terminal_font_size(), DEFAULT_TERMINAL_FONT_SIZE);
    assert_eq!(config.get_ui_scale(), DEFAULT_UI_SCALE);
}

#[test]
fn test_config_serialization_skips_none_ui_settings() {
    let config = Config::default();
    let json = serde_json::to_string(&config).unwrap();

    // None values should not appear in JSON
    assert!(!json.contains("padding"));
    assert!(!json.contains("editorFontSize"));
    assert!(!json.contains("terminalFontSize"));
    assert!(!json.contains("uiScale"));
}

#[test]
fn test_config_serialization_includes_set_ui_settings() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: Some(ContentPadding::default()),
        editor_font_size: Some(16.0),
        terminal_font_size: Some(12.0),
        ui_scale: Some(1.5),
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
        commands: None,
        claude_code: None,
    };

    let json = serde_json::to_string(&config).unwrap();

    assert!(json.contains("padding"));
    assert!(json.contains("editorFontSize"));
    assert!(json.contains("terminalFontSize"));
    assert!(json.contains("uiScale"));
}

#[test]
fn test_config_constants() {
    // Verify constants match expected defaults from task
    assert_eq!(DEFAULT_PADDING_TOP, 8.0);
    assert_eq!(DEFAULT_PADDING_LEFT, 12.0);
    assert_eq!(DEFAULT_PADDING_RIGHT, 12.0);
    assert_eq!(DEFAULT_EDITOR_FONT_SIZE, 16.0);
    assert_eq!(DEFAULT_TERMINAL_FONT_SIZE, 14.0);
    assert_eq!(DEFAULT_UI_SCALE, 1.0);
}

// BuiltInConfig tests
#[test]
fn test_builtin_config_default() {
    let config = BuiltInConfig::default();
    assert!(config.clipboard_history);
    assert!(config.app_launcher);
    assert!(config.window_switcher);
}

#[test]
fn test_builtin_config_serialization_camel_case() {
    let config = BuiltInConfig {
        clipboard_history: true,
        app_launcher: false,
        window_switcher: true,
    };

    let json = serde_json::to_string(&config).unwrap();

    // Should use camelCase in JSON
    assert!(json.contains("clipboardHistory"));
    assert!(json.contains("appLauncher"));
    assert!(json.contains("windowSwitcher"));
    // Should NOT use snake_case
    assert!(!json.contains("clipboard_history"));
    assert!(!json.contains("app_launcher"));
    assert!(!json.contains("window_switcher"));
}

#[test]
fn test_builtin_config_deserialization_camel_case() {
    let json = r#"{
        "clipboardHistory": false,
        "appLauncher": true,
        "windowSwitcher": false
    }"#;

    let config: BuiltInConfig = serde_json::from_str(json).unwrap();

    assert!(!config.clipboard_history);
    assert!(config.app_launcher);
    assert!(!config.window_switcher);
}

#[test]
fn test_builtin_config_deserialization_with_defaults() {
    // Partial config - missing fields should use defaults
    let json = r#"{"clipboardHistory": false}"#;
    let config: BuiltInConfig = serde_json::from_str(json).unwrap();

    assert!(!config.clipboard_history);
    assert!(config.app_launcher); // Default true
    assert!(config.window_switcher); // Default true
}

#[test]
fn test_config_with_builtins() {
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
        built_ins: Some(BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: true,
        }),
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
        commands: None,
        claude_code: None,
    };

    let builtins = config.get_builtins();
    assert!(builtins.clipboard_history);
    assert!(!builtins.app_launcher);
    assert!(builtins.window_switcher);
}

#[test]
fn test_config_get_builtins_default() {
    let config = Config::default();
    let builtins = config.get_builtins();

    // Should return defaults when built_ins is None
    assert!(builtins.clipboard_history);
    assert!(builtins.app_launcher);
    assert!(builtins.window_switcher);
}

#[test]
fn test_config_deserialization_with_builtins() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        },
        "builtIns": {
            "clipboardHistory": true,
            "appLauncher": false,
            "windowSwitcher": true
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    assert!(config.built_ins.is_some());
    let builtins = config.get_builtins();
    assert!(builtins.clipboard_history);
    assert!(!builtins.app_launcher);
    assert!(builtins.window_switcher);
}

#[test]
fn test_config_deserialization_without_builtins() {
    // Existing configs without builtIns should still work
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    assert!(config.built_ins.is_none());

    // Getter should return defaults
    let builtins = config.get_builtins();
    assert!(builtins.clipboard_history);
    assert!(builtins.app_launcher);
    assert!(builtins.window_switcher);
}

#[test]
fn test_config_serialization_skips_none_builtins() {
    let config = Config::default();
    let json = serde_json::to_string(&config).unwrap();

    // None values should not appear in JSON
    assert!(!json.contains("builtIns"));
}

#[test]
fn test_config_serialization_includes_set_builtins() {
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
        built_ins: Some(BuiltInConfig::default()),
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
        commands: None,
        claude_code: None,
    };

    let json = serde_json::to_string(&config).unwrap();

    assert!(json.contains("builtIns"));
    assert!(json.contains("clipboardHistory"));
    assert!(json.contains("appLauncher"));
    assert!(json.contains("windowSwitcher"));
}

#[test]
fn test_builtin_config_roundtrip() {
    // Test full roundtrip serialization/deserialization
    let original = BuiltInConfig {
        clipboard_history: false,
        app_launcher: true,
        window_switcher: true,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: BuiltInConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original.clipboard_history, restored.clipboard_history);
    assert_eq!(original.app_launcher, restored.app_launcher);
    assert_eq!(original.window_switcher, restored.window_switcher);
}

// ProcessLimits tests
#[test]
fn test_process_limits_default() {
    let limits = ProcessLimits::default();
    assert_eq!(limits.max_memory_mb, None);
    assert_eq!(limits.max_runtime_seconds, None);
    assert_eq!(
        limits.health_check_interval_ms,
        DEFAULT_HEALTH_CHECK_INTERVAL_MS
    );
}

#[test]
fn test_process_limits_default_constant() {
    assert_eq!(DEFAULT_HEALTH_CHECK_INTERVAL_MS, 5000);
}

#[test]
fn test_process_limits_serialization_camel_case() {
    let limits = ProcessLimits {
        max_memory_mb: Some(512),
        max_runtime_seconds: Some(300),
        health_check_interval_ms: 3000,
    };

    let json = serde_json::to_string(&limits).unwrap();

    // Should use camelCase in JSON
    assert!(json.contains("maxMemoryMb"));
    assert!(json.contains("maxRuntimeSeconds"));
    assert!(json.contains("healthCheckIntervalMs"));
    // Should NOT use snake_case
    assert!(!json.contains("max_memory_mb"));
    assert!(!json.contains("max_runtime_seconds"));
    assert!(!json.contains("health_check_interval_ms"));
}

#[test]
fn test_process_limits_deserialization_camel_case() {
    let json = r#"{
        "maxMemoryMb": 1024,
        "maxRuntimeSeconds": 600,
        "healthCheckIntervalMs": 2000
    }"#;

    let limits: ProcessLimits = serde_json::from_str(json).unwrap();

    assert_eq!(limits.max_memory_mb, Some(1024));
    assert_eq!(limits.max_runtime_seconds, Some(600));
    assert_eq!(limits.health_check_interval_ms, 2000);
}

#[test]
fn test_process_limits_deserialization_with_defaults() {
    // Partial config - missing fields should use defaults
    let json = r#"{"maxMemoryMb": 256}"#;
    let limits: ProcessLimits = serde_json::from_str(json).unwrap();

    assert_eq!(limits.max_memory_mb, Some(256));
    assert_eq!(limits.max_runtime_seconds, None); // Default
    assert_eq!(
        limits.health_check_interval_ms,
        DEFAULT_HEALTH_CHECK_INTERVAL_MS
    ); // Default
}

#[test]
fn test_process_limits_deserialization_empty() {
    // Empty object should use all defaults
    let json = r#"{}"#;
    let limits: ProcessLimits = serde_json::from_str(json).unwrap();

    assert_eq!(limits.max_memory_mb, None);
    assert_eq!(limits.max_runtime_seconds, None);
    assert_eq!(
        limits.health_check_interval_ms,
        DEFAULT_HEALTH_CHECK_INTERVAL_MS
    );
}

#[test]
fn test_process_limits_serialization_skips_none() {
    let limits = ProcessLimits::default();
    let json = serde_json::to_string(&limits).unwrap();

    // None values should not appear in JSON
    assert!(!json.contains("maxMemoryMb"));
    assert!(!json.contains("maxRuntimeSeconds"));
    // But healthCheckIntervalMs always appears (has value)
    assert!(json.contains("healthCheckIntervalMs"));
}

