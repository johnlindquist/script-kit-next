use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, instrument, warn};

/// Default padding values for content areas
pub const DEFAULT_PADDING_TOP: f32 = 8.0;
pub const DEFAULT_PADDING_LEFT: f32 = 12.0;
pub const DEFAULT_PADDING_RIGHT: f32 = 12.0;

/// Default font sizes
pub const DEFAULT_EDITOR_FONT_SIZE: f32 = 16.0;
pub const DEFAULT_TERMINAL_FONT_SIZE: f32 = 14.0;

/// Default UI scale
pub const DEFAULT_UI_SCALE: f32 = 1.0;

/// Default built-in feature flags
pub const DEFAULT_CLIPBOARD_HISTORY: bool = true;
pub const DEFAULT_APP_LAUNCHER: bool = true;
pub const DEFAULT_WINDOW_SWITCHER: bool = true;
/// Default max text length for clipboard history entries (bytes)
pub const DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH: usize = 100_000;

/// Default process limits
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 5000;

/// Default frecency settings
pub const DEFAULT_FRECENCY_HALF_LIFE_DAYS: f64 = 7.0;
pub const DEFAULT_FRECENCY_MAX_RECENT_ITEMS: usize = 10;
pub const DEFAULT_FRECENCY_ENABLED: bool = true;

/// Commands that require confirmation before execution by default.
/// Users can override this behavior per-command in config.ts using `confirmationRequired`.
pub const DEFAULT_CONFIRMATION_COMMANDS: &[&str] = &[
    "builtin-shut-down",
    "builtin-restart",
    "builtin-log-out",
    "builtin-empty-trash",
    "builtin-sleep",
    "builtin-quit-script-kit",
    "builtin-test-confirmation", // Dev test item
];

/// Configuration for built-in features (clipboard history, app launcher, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltInConfig {
    /// Enable clipboard history built-in (default: true)
    #[serde(default = "default_clipboard_history")]
    pub clipboard_history: bool,
    /// Enable app launcher built-in (default: true)
    #[serde(default = "default_app_launcher")]
    pub app_launcher: bool,
    /// Enable window switcher built-in (default: true)
    #[serde(default = "default_window_switcher")]
    pub window_switcher: bool,
}

fn default_clipboard_history() -> bool {
    DEFAULT_CLIPBOARD_HISTORY
}
fn default_app_launcher() -> bool {
    DEFAULT_APP_LAUNCHER
}
fn default_window_switcher() -> bool {
    DEFAULT_WINDOW_SWITCHER
}

impl Default for BuiltInConfig {
    fn default() -> Self {
        BuiltInConfig {
            clipboard_history: DEFAULT_CLIPBOARD_HISTORY,
            app_launcher: DEFAULT_APP_LAUNCHER,
            window_switcher: DEFAULT_WINDOW_SWITCHER,
        }
    }
}

/// Configuration for process resource limits and health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessLimits {
    /// Maximum memory usage in MB (None = no limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    /// Maximum runtime in seconds (None = no limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_runtime_seconds: Option<u64>,
    /// Health check interval in milliseconds (default: 5000)
    #[serde(default = "default_health_check_interval_ms")]
    pub health_check_interval_ms: u64,
}

fn default_health_check_interval_ms() -> u64 {
    DEFAULT_HEALTH_CHECK_INTERVAL_MS
}

impl Default for ProcessLimits {
    fn default() -> Self {
        ProcessLimits {
            max_memory_mb: None,
            max_runtime_seconds: None,
            health_check_interval_ms: DEFAULT_HEALTH_CHECK_INTERVAL_MS,
        }
    }
}

/// Configuration for frecency scoring (recent items ranking)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrecencyConfig {
    /// Whether frecency tracking is enabled (default: true)
    #[serde(default = "default_frecency_enabled")]
    pub enabled: bool,
    /// Half-life in days for frecency decay (default: 7.0)
    /// Lower values = more weight on recent items
    /// Higher values = more weight on frequently used items
    #[serde(default = "default_frecency_half_life_days")]
    pub half_life_days: f64,
    /// Maximum number of items to show in RECENT section (default: 10)
    #[serde(default = "default_frecency_max_recent_items")]
    pub max_recent_items: usize,
}

fn default_frecency_enabled() -> bool {
    DEFAULT_FRECENCY_ENABLED
}
fn default_frecency_half_life_days() -> f64 {
    DEFAULT_FRECENCY_HALF_LIFE_DAYS
}
fn default_frecency_max_recent_items() -> usize {
    DEFAULT_FRECENCY_MAX_RECENT_ITEMS
}

impl Default for FrecencyConfig {
    fn default() -> Self {
        FrecencyConfig {
            enabled: DEFAULT_FRECENCY_ENABLED,
            half_life_days: DEFAULT_FRECENCY_HALF_LIFE_DAYS,
            max_recent_items: DEFAULT_FRECENCY_MAX_RECENT_ITEMS,
        }
    }
}

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

fn default_padding_top() -> f32 {
    DEFAULT_PADDING_TOP
}
fn default_padding_left() -> f32 {
    DEFAULT_PADDING_LEFT
}
fn default_padding_right() -> f32 {
    DEFAULT_PADDING_RIGHT
}

impl Default for ContentPadding {
    fn default() -> Self {
        ContentPadding {
            top: DEFAULT_PADDING_TOP,
            left: DEFAULT_PADDING_LEFT,
            right: DEFAULT_PADDING_RIGHT,
        }
    }
}

/// Configuration for a specific command (script, built-in, or app).
///
/// Used to set per-command shortcuts and visibility options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandConfig {
    /// Optional keyboard shortcut to invoke this command directly
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<HotkeyConfig>,
    /// Whether this command should be hidden from the main menu
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Whether this command requires confirmation before execution.
    /// Overrides the default behavior from DEFAULT_CONFIRMATION_COMMANDS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_required: Option<bool>,
}

/// Check if a string is a valid command ID format.
///
/// Valid command IDs start with one of:
/// - `builtin/` - Built-in Script Kit features
/// - `app/` - macOS applications (by bundle identifier)
/// - `script/` - User scripts (by filename)
/// - `scriptlet/` - Inline scriptlets (by UUID or name)
#[allow(dead_code)]
pub fn is_valid_command_id(id: &str) -> bool {
    id.starts_with("builtin/")
        || id.starts_with("app/")
        || id.starts_with("script/")
        || id.starts_with("scriptlet/")
}

/// Convert a command ID to its deeplink URL.
///
/// The deeplink format is: `kit://commands/{commandId}`
#[allow(dead_code)]
pub fn command_id_to_deeplink(command_id: &str) -> String {
    format!("kit://commands/{}", command_id)
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "editorFontSize"
    )]
    pub editor_font_size: Option<f32>,
    /// Font size for the terminal prompt (in pixels)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "terminalFontSize"
    )]
    pub terminal_font_size: Option<f32>,
    /// UI scale factor (1.0 = 100%)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "uiScale")]
    pub ui_scale: Option<f32>,
    /// Built-in features configuration (clipboard history, app launcher, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "builtIns")]
    pub built_ins: Option<BuiltInConfig>,
    /// Process resource limits and health monitoring configuration
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "processLimits"
    )]
    pub process_limits: Option<ProcessLimits>,
    /// Maximum text length for clipboard history entries (bytes). 0 = no limit.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "clipboardHistoryMaxTextLength"
    )]
    pub clipboard_history_max_text_length: Option<usize>,
    /// Frecency configuration for recent items ranking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frecency: Option<FrecencyConfig>,
    /// Hotkey for opening Notes window (default: Cmd+Shift+N)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "notesHotkey"
    )]
    pub notes_hotkey: Option<HotkeyConfig>,
    /// Hotkey for opening AI Chat window (default: Cmd+Shift+Space)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "aiHotkey")]
    pub ai_hotkey: Option<HotkeyConfig>,
    /// Per-command configuration overrides (shortcuts, visibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<HashMap<String, CommandConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl HotkeyConfig {
    /// Create a default notes hotkey (Cmd+Shift+N)
    pub fn default_notes_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyN".to_string(),
        }
    }

    /// Create a default AI hotkey (Cmd+Shift+Space)
    pub fn default_ai_hotkey() -> Self {
        HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Space".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(), // Cmd+; matches main.rs default
            },
            bun_path: None,           // Will use system PATH if not specified
            editor: None,             // Will use $EDITOR or fallback to "code"
            padding: None,            // Will use ContentPadding::default() via getter
            editor_font_size: None,   // Will use DEFAULT_EDITOR_FONT_SIZE via getter
            terminal_font_size: None, // Will use DEFAULT_TERMINAL_FONT_SIZE via getter
            ui_scale: None,           // Will use DEFAULT_UI_SCALE via getter
            built_ins: None,          // Will use BuiltInConfig::default() via getter
            process_limits: None,     // Will use ProcessLimits::default() via getter
            clipboard_history_max_text_length: None, // Will use default via getter
            frecency: None,           // Will use FrecencyConfig::default() via getter
            notes_hotkey: None,       // Will use HotkeyConfig::default_notes_hotkey() via getter
            ai_hotkey: None,          // Will use HotkeyConfig::default_ai_hotkey() via getter
            commands: None,           // No per-command overrides by default
        }
    }
}

impl Config {
    /// Returns the configured editor, falling back to $EDITOR env var or "code" (VS Code)
    /// Used by ActionsDialog "Open in Editor" action
    #[allow(dead_code)] // Will be used by ActionsDialog worker
    pub fn get_editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "code".to_string())
    }

    /// Returns the content padding, or defaults if not configured
    #[allow(dead_code)] // Will be used by TermPrompt/EditorPrompt workers
    pub fn get_padding(&self) -> ContentPadding {
        self.padding.clone().unwrap_or_default()
    }

    /// Returns the editor font size, or DEFAULT_EDITOR_FONT_SIZE if not configured
    #[allow(dead_code)] // Will be used by EditorPrompt worker
    pub fn get_editor_font_size(&self) -> f32 {
        self.editor_font_size.unwrap_or(DEFAULT_EDITOR_FONT_SIZE)
    }

    /// Returns the terminal font size, or DEFAULT_TERMINAL_FONT_SIZE if not configured
    #[allow(dead_code)] // Will be used by TermPrompt worker
    pub fn get_terminal_font_size(&self) -> f32 {
        self.terminal_font_size
            .unwrap_or(DEFAULT_TERMINAL_FONT_SIZE)
    }

    /// Returns the UI scale factor, or DEFAULT_UI_SCALE if not configured
    #[allow(dead_code)] // Will be used for UI scaling
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scale.unwrap_or(DEFAULT_UI_SCALE)
    }

    /// Returns the built-in features configuration, or defaults if not configured
    #[allow(dead_code)] // Will be used by builtins module
    pub fn get_builtins(&self) -> BuiltInConfig {
        self.built_ins.clone().unwrap_or_default()
    }

    /// Returns max clipboard history text length (bytes), or default if not configured
    #[allow(dead_code)] // Used for clipboard history limits
    pub fn get_clipboard_history_max_text_length(&self) -> usize {
        self.clipboard_history_max_text_length
            .unwrap_or(DEFAULT_CLIPBOARD_HISTORY_MAX_TEXT_LENGTH)
    }

    /// Returns the process limits configuration, or defaults if not configured
    #[allow(dead_code)] // Will be used by process_manager module
    pub fn get_process_limits(&self) -> ProcessLimits {
        self.process_limits.clone().unwrap_or_default()
    }

    /// Returns the frecency configuration, or defaults if not configured
    pub fn get_frecency(&self) -> FrecencyConfig {
        self.frecency.clone().unwrap_or_default()
    }

    /// Returns the notes hotkey configuration, or default (Cmd+Shift+N) if not configured
    #[allow(dead_code)]
    pub fn get_notes_hotkey(&self) -> HotkeyConfig {
        self.notes_hotkey
            .clone()
            .unwrap_or_else(HotkeyConfig::default_notes_hotkey)
    }

    /// Returns the AI hotkey configuration, or default (Cmd+Shift+Space) if not configured
    #[allow(dead_code)]
    pub fn get_ai_hotkey(&self) -> HotkeyConfig {
        self.ai_hotkey
            .clone()
            .unwrap_or_else(HotkeyConfig::default_ai_hotkey)
    }

    /// Returns command configuration for a specific command ID, or None if not configured.
    #[allow(dead_code)]
    pub fn get_command_config(&self, command_id: &str) -> Option<&CommandConfig> {
        self.commands.as_ref().and_then(|cmds| cmds.get(command_id))
    }

    /// Check if a command should be hidden from the main menu.
    #[allow(dead_code)]
    pub fn is_command_hidden(&self, command_id: &str) -> bool {
        self.get_command_config(command_id)
            .and_then(|c| c.hidden)
            .unwrap_or(false)
    }

    /// Get the shortcut for a command, if configured.
    #[allow(dead_code)]
    pub fn get_command_shortcut(&self, command_id: &str) -> Option<&HotkeyConfig> {
        self.get_command_config(command_id)
            .and_then(|c| c.shortcut.as_ref())
    }

    /// Check if a command requires confirmation before execution.
    ///
    /// Returns true if:
    /// - Command is in DEFAULT_CONFIRMATION_COMMANDS AND not explicitly disabled in config
    /// - OR command has confirmationRequired: true in config
    #[allow(dead_code)]
    pub fn requires_confirmation(&self, command_id: &str) -> bool {
        // Check if user has explicitly configured this command
        if let Some(cmd_config) = self.get_command_config(command_id) {
            if let Some(requires) = cmd_config.confirmation_required {
                return requires;
            }
        }
        // Fall back to defaults
        DEFAULT_CONFIRMATION_COMMANDS.contains(&command_id)
    }
}

#[instrument(name = "load_config")]
pub fn load_config() -> Config {
    let config_path = PathBuf::from(shellexpand::tilde("~/.sk/kit/config.ts").as_ref());

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
                            import type { Config } from \"@scriptkit/sdk\";\n\n\
                            export default {\n\
                              hotkey: {\n\
                                modifiers: [\"meta\"],\n\
                                key: \"Semicolon\"\n\
                              }\n\
                            } satisfies Config;"
                        } else if e.to_string().contains("missing field `modifiers`")
                            || e.to_string().contains("missing field `key`")
                        {
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
                frecency: None,
                notes_hotkey: None,
                ai_hotkey: None,
                commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            built_ins: None,
            process_limits: None,
            clipboard_history_max_text_length: None,
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            built_ins: None,
            process_limits: None,
            clipboard_history_max_text_length: None,
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: None,
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: Some(commands),
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: Some(commands),
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
            frecency: None,
            notes_hotkey: None,
            ai_hotkey: None,
            commands: Some(commands),
        };

        // Should still require confirmation (falls back to default)
        assert!(config.requires_confirmation("builtin-shut-down"));
    }
}
