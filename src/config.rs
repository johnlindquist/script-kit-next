use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;
use tracing::{info, warn, instrument};

/// Default padding values for content areas
pub const DEFAULT_PADDING_TOP: f32 = 8.0;
pub const DEFAULT_PADDING_LEFT: f32 = 12.0;
pub const DEFAULT_PADDING_RIGHT: f32 = 12.0;

/// Default font sizes
pub const DEFAULT_EDITOR_FONT_SIZE: f32 = 14.0;
pub const DEFAULT_TERMINAL_FONT_SIZE: f32 = 14.0;

/// Default UI scale
pub const DEFAULT_UI_SCALE: f32 = 1.0;

/// Content padding configuration for prompts (terminal, editor, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPadding {
    #[serde(default = "default_padding_top")]
    pub top: f32,
    #[serde(default = "default_padding_left")]
    pub left: f32,
    #[serde(default = "default_padding_right")]
    pub right: f32,
}

fn default_padding_top() -> f32 { DEFAULT_PADDING_TOP }
fn default_padding_left() -> f32 { DEFAULT_PADDING_LEFT }
fn default_padding_right() -> f32 { DEFAULT_PADDING_RIGHT }

impl Default for ContentPadding {
    fn default() -> Self {
        ContentPadding {
            top: DEFAULT_PADDING_TOP,
            left: DEFAULT_PADDING_LEFT,
            right: DEFAULT_PADDING_RIGHT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: HotkeyConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bun_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    /// Padding for content areas (terminal, editor, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<ContentPadding>,
    /// Font size for the editor prompt (in pixels)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "editorFontSize")]
    pub editor_font_size: Option<f32>,
    /// Font size for the terminal prompt (in pixels)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "terminalFontSize")]
    pub terminal_font_size: Option<f32>,
    /// UI scale factor (1.0 = 100%)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "uiScale")]
    pub ui_scale: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),  // Cmd+; matches main.rs default
            },
            bun_path: None,  // Will use system PATH if not specified
            editor: None,    // Will use $EDITOR or fallback to "code"
            padding: None,   // Will use ContentPadding::default() via getter
            editor_font_size: None,    // Will use DEFAULT_EDITOR_FONT_SIZE via getter
            terminal_font_size: None,  // Will use DEFAULT_TERMINAL_FONT_SIZE via getter
            ui_scale: None,  // Will use DEFAULT_UI_SCALE via getter
        }
    }
}

impl Config {
    /// Returns the configured editor, falling back to $EDITOR env var or "code" (VS Code)
    /// Used by ActionsDialog "Open in Editor" action
    #[allow(dead_code)]  // Will be used by ActionsDialog worker
    pub fn get_editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "code".to_string())
    }

    /// Returns the content padding, or defaults if not configured
    #[allow(dead_code)]  // Will be used by TermPrompt/EditorPrompt workers
    pub fn get_padding(&self) -> ContentPadding {
        self.padding.clone().unwrap_or_default()
    }

    /// Returns the editor font size, or DEFAULT_EDITOR_FONT_SIZE if not configured
    #[allow(dead_code)]  // Will be used by EditorPrompt worker
    pub fn get_editor_font_size(&self) -> f32 {
        self.editor_font_size.unwrap_or(DEFAULT_EDITOR_FONT_SIZE)
    }

    /// Returns the terminal font size, or DEFAULT_TERMINAL_FONT_SIZE if not configured
    #[allow(dead_code)]  // Will be used by TermPrompt worker
    pub fn get_terminal_font_size(&self) -> f32 {
        self.terminal_font_size.unwrap_or(DEFAULT_TERMINAL_FONT_SIZE)
    }

    /// Returns the UI scale factor, or DEFAULT_UI_SCALE if not configured
    #[allow(dead_code)]  // Will be used for UI scaling
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scale.unwrap_or(DEFAULT_UI_SCALE)
    }
}

#[instrument(name = "load_config")]
pub fn load_config() -> Config {
    let config_path = PathBuf::from(shellexpand::tilde("~/.kenv/config.ts").as_ref());

    // Check if config file exists
    if !config_path.exists() {
        info!(path = %config_path.display(), "Config file not found, using defaults");
        return Config::default();
    }

    // Step 1: Transpile TypeScript to JavaScript using bun build
    let tmp_js_path = "/tmp/kit-config.js";
    let build_output = Command::new("bun")
        .arg("build")
        .arg("--target=bun")
        .arg(config_path.to_string_lossy().to_string())
        .arg(format!("--outfile={}", tmp_js_path))
        .output();

    match build_output {
        Err(e) => {
            warn!(error = %e, "Failed to transpile config with bun, using defaults");
            return Config::default();
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun build failed, using defaults"
                );
                return Config::default();
            }
        }
    }

    // Step 2: Execute the transpiled JS and extract the default export as JSON
    let json_output = Command::new("bun")
        .arg("-e")
        .arg(format!(
            "console.log(JSON.stringify(require('{}').default))",
            tmp_js_path
        ))
        .output();

    match json_output {
        Err(e) => {
            warn!(error = %e, "Failed to execute bun to extract JSON, using defaults");
            Config::default()
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun execution failed, using defaults"
                );
                Config::default()
            } else {
                // Step 3: Parse the JSON output into Config struct
                let json_str = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<Config>(json_str.trim()) {
                    Ok(config) => {
                        info!(path = %config_path.display(), "Successfully loaded config");
                        config
                    }
                    Err(e) => {
                        // Provide helpful error message for common config mistakes
                        let error_hint = if e.to_string().contains("missing field `hotkey`") {
                            "\n\nHint: Your config.ts must include a 'hotkey' field. Example:\n\
                            import type { Config } from \"@johnlindquist/kit\";\n\n\
                            export default {\n\
                              hotkey: {\n\
                                modifiers: [\"meta\"],\n\
                                key: \"Semicolon\"\n\
                              }\n\
                            } satisfies Config;"
                        } else if e.to_string().contains("missing field `modifiers`") || e.to_string().contains("missing field `key`") {
                            "\n\nHint: The 'hotkey' field requires 'modifiers' (array) and 'key' (string). Example:\n\
                            hotkey: {\n\
                              modifiers: [\"meta\"],  // \"meta\", \"ctrl\", \"alt\", \"shift\"\n\
                              key: \"Digit0\"         // e.g., \"Semicolon\", \"KeyK\", \"Digit0\"\n\
                            }"
                        } else {
                            ""
                        };
                        
                        warn!(
                            error = %e,
                            json_output = %json_str,
                            hint = %error_hint,
                            "Failed to parse config JSON, using defaults"
                        );
                        Config::default()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.hotkey.modifiers, vec!["meta"]);
        assert_eq!(config.hotkey.key, "Semicolon");
        assert_eq!(config.bun_path, None);
        assert_eq!(config.editor, None);
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
        };

        let json = serde_json::to_string(&config).unwrap();
        // Editor should not appear in JSON when None (skip_serializing_if)
        assert!(!json.contains("editor"));
        
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.editor, None);
    }

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
        assert_eq!(DEFAULT_EDITOR_FONT_SIZE, 14.0);
        assert_eq!(DEFAULT_TERMINAL_FONT_SIZE, 14.0);
        assert_eq!(DEFAULT_UI_SCALE, 1.0);
    }
}
