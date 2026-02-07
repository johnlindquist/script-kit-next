#[test]
fn test_get_editor_from_config() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: Some("nvim".to_string()),
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
        commands: None,
        claude_code: None,
    };

    // Config editor takes precedence
    assert_eq!(config.get_editor(), "nvim");
}

#[test]
fn test_get_editor_from_env() {
    // Save current EDITOR value
    let original_editor = std::env::var("EDITOR").ok();

    // Set EDITOR env var
    std::env::set_var("EDITOR", "emacs");

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
        commands: None,
        claude_code: None,
    };

    // Should fall back to EDITOR env var
    assert_eq!(config.get_editor(), "emacs");

    // Restore original EDITOR value
    match original_editor {
        Some(val) => std::env::set_var("EDITOR", val),
        None => std::env::remove_var("EDITOR"),
    }
}

#[test]
fn test_get_editor_default() {
    // Save current EDITOR value
    let original_editor = std::env::var("EDITOR").ok();

    // Remove EDITOR env var
    std::env::remove_var("EDITOR");

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
        commands: None,
        claude_code: None,
    };

    // Should fall back to "code" default
    assert_eq!(config.get_editor(), "code");

    // Restore original EDITOR value
    if let Some(val) = original_editor {
        std::env::set_var("EDITOR", val);
    }
}

#[test]
fn test_config_editor_priority() {
    // Save current EDITOR value
    let original_editor = std::env::var("EDITOR").ok();

    // Set EDITOR env var
    std::env::set_var("EDITOR", "emacs");

    // Config with editor set should take precedence over env var
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: Some("vim".to_string()),
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
        commands: None,
        claude_code: None,
    };

    // Config editor should win
    assert_eq!(config.get_editor(), "vim");

    // Restore original EDITOR value
    match original_editor {
        Some(val) => std::env::set_var("EDITOR", val),
        None => std::env::remove_var("EDITOR"),
    }
}

#[test]
fn test_config_deserialization_with_editor() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        },
        "editor": "subl"
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.editor, Some("subl".to_string()));
    assert_eq!(config.get_editor(), "subl");
}

// ContentPadding tests
#[test]
fn test_content_padding_default() {
    let padding = ContentPadding::default();
    assert_eq!(padding.top, DEFAULT_PADDING_TOP);
    assert_eq!(padding.left, DEFAULT_PADDING_LEFT);
    assert_eq!(padding.right, DEFAULT_PADDING_RIGHT);
}

#[test]
fn test_content_padding_serialization() {
    let padding = ContentPadding {
        top: 10.0,
        left: 16.0,
        right: 16.0,
    };

    let json = serde_json::to_string(&padding).unwrap();
    let deserialized: ContentPadding = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.top, 10.0);
    assert_eq!(deserialized.left, 16.0);
    assert_eq!(deserialized.right, 16.0);
}

#[test]
fn test_content_padding_partial_deserialization() {
    // If only some fields are present, defaults should fill in
    let json = r#"{"top": 20.0}"#;
    let padding: ContentPadding = serde_json::from_str(json).unwrap();

    assert_eq!(padding.top, 20.0);
    assert_eq!(padding.left, DEFAULT_PADDING_LEFT);
    assert_eq!(padding.right, DEFAULT_PADDING_RIGHT);
}

// UI settings tests
#[test]
fn test_config_default_has_none_ui_settings() {
    let config = Config::default();
    assert!(config.padding.is_none());
    assert!(config.editor_font_size.is_none());
    assert!(config.terminal_font_size.is_none());
    assert!(config.ui_scale.is_none());
}

#[test]
fn test_config_get_padding_default() {
    let config = Config::default();
    let padding = config.get_padding();

    assert_eq!(padding.top, DEFAULT_PADDING_TOP);
    assert_eq!(padding.left, DEFAULT_PADDING_LEFT);
    assert_eq!(padding.right, DEFAULT_PADDING_RIGHT);
}

#[test]
fn test_config_get_padding_custom() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: Some(ContentPadding {
            top: 10.0,
            left: 20.0,
            right: 20.0,
        }),
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
        commands: None,
        claude_code: None,
    };

    let padding = config.get_padding();
    assert_eq!(padding.top, 10.0);
    assert_eq!(padding.left, 20.0);
    assert_eq!(padding.right, 20.0);
}

#[test]
fn test_config_get_editor_font_size_default() {
    let config = Config::default();
    assert_eq!(config.get_editor_font_size(), DEFAULT_EDITOR_FONT_SIZE);
}

#[test]
fn test_config_get_editor_font_size_custom() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: Some(16.0),
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
        commands: None,
        claude_code: None,
    };

    assert_eq!(config.get_editor_font_size(), 16.0);
}

#[test]
fn test_config_get_terminal_font_size_default() {
    let config = Config::default();
    assert_eq!(config.get_terminal_font_size(), DEFAULT_TERMINAL_FONT_SIZE);
}

#[test]
fn test_config_get_terminal_font_size_custom() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: None,
        editor: None,
        padding: None,
        editor_font_size: None,
        terminal_font_size: Some(12.0),
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
        commands: None,
        claude_code: None,
    };

    assert_eq!(config.get_terminal_font_size(), 12.0);
}

#[test]
fn test_config_get_ui_scale_default() {
    let config = Config::default();
    assert_eq!(config.get_ui_scale(), DEFAULT_UI_SCALE);
}

#[test]
fn test_config_get_ui_scale_custom() {
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

    assert_eq!(config.get_ui_scale(), 1.5);
}

#[test]
fn test_config_deserialization_with_ui_settings() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        },
        "padding": {
            "top": 10,
            "left": 16,
            "right": 16
        },
        "editorFontSize": 16,
        "terminalFontSize": 14,
        "uiScale": 1.2
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    assert!(config.padding.is_some());
    let padding = config.get_padding();
    assert_eq!(padding.top, 10.0);
    assert_eq!(padding.left, 16.0);
    assert_eq!(padding.right, 16.0);

    assert_eq!(config.get_editor_font_size(), 16.0);
    assert_eq!(config.get_terminal_font_size(), 14.0);
    assert_eq!(config.get_ui_scale(), 1.2);
}
