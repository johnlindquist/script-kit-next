use super::*;
use std::collections::HashMap;

fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.hotkey.modifiers, vec!["meta"]);
    assert_eq!(config.hotkey.key, "Semicolon");
    assert_eq!(config.bun_path, None);
    assert_eq!(config.editor, None);
}

#[test]
fn test_clipboard_history_max_text_length_default() {
    let config = Config::default();
    assert_eq!(
        config.get_clipboard_history_max_text_length(),
        DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH
    );
}

#[test]
fn test_config_serialization() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["ctrl".to_string(), "alt".to_string()],
            key: "KeyA".to_string(),
        },
        bun_path: Some("/usr/local/bin/bun".to_string()),
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

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.hotkey.modifiers, config.hotkey.modifiers);
    assert_eq!(deserialized.hotkey.key, config.hotkey.key);
    assert_eq!(deserialized.bun_path, config.bun_path);
    assert_eq!(deserialized.editor, config.editor);
}

#[test]
fn test_hotkey_config_default_values() {
    let hotkey = HotkeyConfig {
        modifiers: vec!["meta".to_string(), "shift".to_string()],
        key: "KeyK".to_string(),
    };
    assert_eq!(hotkey.modifiers.len(), 2);
    assert!(hotkey.modifiers.contains(&"meta".to_string()));
    assert!(hotkey.modifiers.contains(&"shift".to_string()));
}

#[test]
fn test_config_with_bun_path() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        },
        bun_path: Some("/custom/path/bun".to_string()),
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
    assert_eq!(config.bun_path, Some("/custom/path/bun".to_string()));
}

#[test]
fn test_config_without_bun_path() {
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
    assert_eq!(config.bun_path, None);
}

#[test]
fn test_config_serialization_skip_none_bun_path() {
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

    let json = serde_json::to_string(&config).unwrap();
    // Verify that bun_path is not included when None
    assert!(!json.contains("null"));
    // Should contain hotkey config
    assert!(json.contains("meta"));
}

#[test]
fn test_config_serialization_preserves_multiple_modifiers() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec!["ctrl".to_string(), "shift".to_string(), "alt".to_string()],
            key: "KeyP".to_string(),
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

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.hotkey.modifiers.len(), 3);
    assert_eq!(deserialized.hotkey.key, "KeyP");
}

#[test]
fn test_config_deserialization_with_custom_values() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["shift", "alt"],
            "key": "KeyX"
        },
        "bun_path": "/usr/bin/bun"
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.hotkey.modifiers, vec!["shift", "alt"]);
    assert_eq!(config.hotkey.key, "KeyX");
    assert_eq!(config.bun_path, Some("/usr/bin/bun".to_string()));
}

#[test]
fn test_config_deserialization_minimal() {
    let json = r#"{
        "hotkey": {
            "modifiers": ["meta"],
            "key": "Semicolon"
        }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.hotkey.modifiers, vec!["meta"]);
    assert_eq!(config.hotkey.key, "Semicolon");
    assert_eq!(config.bun_path, None);
}

#[test]
fn test_load_config_returns_config_struct() {
    // Test that load_config returns a valid Config struct
    // It may load from actual config or return defaults
    let config = load_config();
    // Verify it has required fields
    assert!(!config.hotkey.modifiers.is_empty());
    assert!(!config.hotkey.key.is_empty());
    // Either has bun_path set or it's None - both valid
    let _ = config.bun_path;
}

#[test]
fn test_config_clone_independence() {
    let config1 = Config::default();
    let config2 = config1.clone();

    // Verify they are equal but independent
    assert_eq!(config1.hotkey.modifiers, config2.hotkey.modifiers);
    assert_eq!(config1.hotkey.key, config2.hotkey.key);
    assert_eq!(config1.bun_path, config2.bun_path);
    assert_eq!(config1.editor, config2.editor);
}

#[test]
fn test_hotkey_config_clone() {
    let hotkey = HotkeyConfig {
        modifiers: vec!["meta".to_string(), "alt".to_string()],
        key: "KeyK".to_string(),
    };
    let cloned = hotkey.clone();

    assert_eq!(hotkey.modifiers, cloned.modifiers);
    assert_eq!(hotkey.key, cloned.key);
}

#[test]
fn test_config_with_empty_modifiers_list() {
    let config = Config {
        hotkey: HotkeyConfig {
            modifiers: vec![],
            key: "KeyA".to_string(),
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

    assert_eq!(config.hotkey.modifiers.len(), 0);
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.hotkey.modifiers.len(), 0);
}

#[test]
fn test_config_key_preservation() {
    let keys = vec!["Semicolon", "KeyK", "KeyP", "Space", "Enter"];
    for key in keys {
        let config = Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: key.to_string(),
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

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hotkey.key, key);
    }
}

// Editor config tests
#[test]
fn test_config_with_editor() {
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

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("vim"));

    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.editor, Some("vim".to_string()));
}

#[test]
fn test_config_without_editor() {
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

    let json = serde_json::to_string(&config).unwrap();
    // Editor should not appear in JSON when None (skip_serializing_if)
    assert!(!json.contains("editor"));

    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.editor, None);
}

