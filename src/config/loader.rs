//! Configuration loading from file system
//!
//! Handles loading and parsing the config.ts file using bun.

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::{info, instrument, warn};

use super::types::{Config, ScriptKitUserPreferences};

const BUN_CONFIG_EXPORT_EXTRACT_SNIPPET: &str =
    "const modulePath = process.argv[1]; const loaded = require(modulePath); console.log(JSON.stringify(loaded.default));";

fn config_ts_path() -> PathBuf {
    crate::setup::get_kit_path().join("kit").join("config.ts")
}

fn settings_json_path() -> PathBuf {
    crate::setup::get_kit_path()
        .join("kit")
        .join("settings.json")
}

fn build_bun_extract_config_command(transpiled_js_path: &Path) -> Command {
    let mut command = Command::new("bun");
    command
        .arg("-e")
        .arg(BUN_CONFIG_EXPORT_EXTRACT_SNIPPET)
        .arg(transpiled_js_path);
    command
}

fn parse_optional_field<T>(
    object: &Map<String, Value>,
    field: &'static str,
    correlation_id: &str,
) -> Option<T>
where
    T: DeserializeOwned,
{
    let Some(raw) = object.get(field) else {
        return None;
    };

    match serde_json::from_value::<Option<T>>(raw.clone()) {
        Ok(parsed) => parsed,
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                field,
                error = %error,
                "Config field failed validation; using default for this field"
            );
            None
        }
    }
}

fn parse_required_field<T>(
    object: &Map<String, Value>,
    field: &'static str,
    fallback: T,
    correlation_id: &str,
) -> T
where
    T: DeserializeOwned,
{
    let Some(raw) = object.get(field) else {
        return fallback;
    };

    match serde_json::from_value::<T>(raw.clone()) {
        Ok(parsed) => parsed,
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                field,
                error = %error,
                "Required config field failed validation; using default"
            );
            fallback
        }
    }
}

fn recover_config_fields(value: Value, correlation_id: &str) -> Config {
    let Some(object) = value.as_object() else {
        warn!(
            correlation_id = %correlation_id,
            "Config root is not an object; using defaults"
        );
        return Config::default();
    };

    let defaults = Config::default();
    Config {
        hotkey: parse_required_field(object, "hotkey", defaults.hotkey.clone(), correlation_id),
        bun_path: parse_optional_field(object, "bun_path", correlation_id),
        editor: parse_optional_field(object, "editor", correlation_id),
        padding: parse_optional_field(object, "padding", correlation_id),
        editor_font_size: parse_optional_field(object, "editorFontSize", correlation_id),
        terminal_font_size: parse_optional_field(object, "terminalFontSize", correlation_id),
        ui_scale: parse_optional_field(object, "uiScale", correlation_id),
        built_ins: parse_optional_field(object, "builtIns", correlation_id),
        process_limits: parse_optional_field(object, "processLimits", correlation_id),
        clipboard_history_max_text_length: parse_optional_field(
            object,
            "clipboardHistoryMaxTextLength",
            correlation_id,
        ),
        suggested: parse_optional_field(object, "suggested", correlation_id),
        notes_hotkey: parse_optional_field(object, "notesHotkey", correlation_id),
        ai_hotkey: parse_optional_field(object, "aiHotkey", correlation_id),
        ai_hotkey_enabled: parse_optional_field(object, "aiHotkeyEnabled", correlation_id),
        logs_hotkey: parse_optional_field(object, "logsHotkey", correlation_id),
        logs_hotkey_enabled: parse_optional_field(object, "logsHotkeyEnabled", correlation_id),
        watcher: parse_optional_field(object, "watcher", correlation_id),
        layout: parse_optional_field(object, "layout", correlation_id),
        commands: parse_optional_field(object, "commands", correlation_id),
        claude_code: parse_optional_field(object, "claudeCode", correlation_id),
    }
}

fn parse_config_json(json_str: &str, correlation_id: &str) -> Config {
    let parsed_json = match serde_json::from_str::<Value>(json_str.trim()) {
        Ok(value) => value,
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                error = %error,
                "Config output was not valid JSON; using defaults"
            );
            return Config::default();
        }
    };

    match serde_json::from_value::<Config>(parsed_json.clone()) {
        Ok(config) => config,
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                error = %error,
                "Config parse failed; recovering valid fields"
            );
            recover_config_fields(parsed_json, correlation_id)
        }
    }
}

fn recover_user_preferences_fields(value: Value, correlation_id: &str) -> ScriptKitUserPreferences {
    let Some(object) = value.as_object() else {
        warn!(
            correlation_id = %correlation_id,
            "User preferences root is not an object; using defaults"
        );
        return ScriptKitUserPreferences::default();
    };

    let defaults = ScriptKitUserPreferences::default();
    ScriptKitUserPreferences {
        layout: parse_required_field(object, "layout", defaults.layout, correlation_id),
        theme: parse_required_field(object, "theme", defaults.theme, correlation_id),
    }
}

fn parse_user_preferences_json(json_str: &str, correlation_id: &str) -> ScriptKitUserPreferences {
    let parsed_json = match serde_json::from_str::<Value>(json_str.trim()) {
        Ok(value) => value,
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                error = %error,
                "User preferences JSON was invalid; using defaults"
            );
            return ScriptKitUserPreferences::default();
        }
    };

    match serde_json::from_value::<ScriptKitUserPreferences>(parsed_json.clone()) {
        Ok(preferences) => preferences,
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                error = %error,
                "User preferences parse failed; recovering valid fields"
            );
            recover_user_preferences_fields(parsed_json, correlation_id)
        }
    }
}

/// Load user preferences from `<SK_PATH>/kit/settings.json` (or `~/.scriptkit/kit/settings.json`)
///
/// This file is intentionally JSON (not TypeScript) so runtime readers can parse
/// lightweight preferences (layout/theme) without invoking Bun.
pub fn load_user_preferences() -> ScriptKitUserPreferences {
    let correlation_id = format!("settings_load:{}", uuid::Uuid::new_v4());
    let settings_path = settings_json_path();

    if !settings_path.exists() {
        info!(
            correlation_id = %correlation_id,
            path = %settings_path.display(),
            "Settings file not found, using defaults"
        );
        return ScriptKitUserPreferences::default();
    }

    match std::fs::read_to_string(&settings_path) {
        Ok(contents) => {
            let preferences = parse_user_preferences_json(&contents, &correlation_id);
            info!(
                correlation_id = %correlation_id,
                path = %settings_path.display(),
                "Successfully loaded user preferences"
            );
            preferences
        }
        Err(error) => {
            warn!(
                correlation_id = %correlation_id,
                path = %settings_path.display(),
                error = %error,
                "Failed to read settings file, using defaults"
            );
            ScriptKitUserPreferences::default()
        }
    }
}

/// Load configuration from `<SK_PATH>/kit/config.ts` (or `~/.scriptkit/kit/config.ts`)
///
/// This function:
/// 1. Checks if the config file exists
/// 2. Transpiles TypeScript to JavaScript using bun build
/// 3. Executes the JS to extract the default export as JSON
/// 4. Parses the JSON into a Config struct
///
/// Returns Config::default() if any step fails.
#[instrument(name = "load_config")]
pub fn load_config() -> Config {
    let correlation_id = format!("config_load:{}", uuid::Uuid::new_v4());
    let config_path = config_ts_path();

    // Check if config file exists
    if !config_path.exists() {
        info!(
            correlation_id = %correlation_id,
            path = %config_path.display(),
            "Config file not found, using defaults"
        );
        return Config::default();
    }

    // Step 1: Transpile TypeScript to JavaScript using bun build
    // Use secure temporary file creation to avoid predictable paths and TOCTOU attacks
    let tmp_js = match NamedTempFile::new() {
        Ok(file) => file,
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "Failed to create temporary file, using defaults"
            );
            return Config::default();
        }
    };
    let tmp_js_path = tmp_js.path();

    let build_output = Command::new("bun")
        .arg("build")
        .arg("--target=bun")
        .arg(config_path.to_string_lossy().to_string())
        .arg(format!("--outfile={}", tmp_js_path.display()))
        .output();

    match build_output {
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "Failed to transpile config with bun, using defaults"
            );
            return Config::default();
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    correlation_id = %correlation_id,
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun build failed, using defaults"
                );
                return Config::default();
            }
        }
    }

    // Step 2: Execute the transpiled JS and extract the default export as JSON
    let json_output = build_bun_extract_config_command(tmp_js_path).output();

    match json_output {
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "Failed to execute bun to extract JSON, using defaults"
            );
            Config::default()
        }
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    correlation_id = %correlation_id,
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "bun execution failed, using defaults"
                );
                Config::default()
            } else {
                let json_str = String::from_utf8_lossy(&output.stdout);
                let config = parse_config_json(&json_str, &correlation_id);
                info!(
                    correlation_id = %correlation_id,
                    path = %config_path.display(),
                    "Successfully loaded config"
                );
                config
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_bun_extract_config_command, parse_config_json, parse_user_preferences_json,
        BUN_CONFIG_EXPORT_EXTRACT_SNIPPET,
    };
    use crate::config::HotkeyConfig;
    use std::fs;
    use std::path::Path;

    /// Code audit test: Verify all config.ts path references use the authoritative path.
    ///
    /// AUTHORITATIVE PATH: ~/.scriptkit/kit/config.ts
    ///
    /// This test ensures no code accidentally references the wrong path like:
    /// - ~/.scriptkit/config.ts (missing /kit/)
    ///
    /// The correct pattern is always: ~/.scriptkit/kit/config.ts
    #[test]
    fn test_config_path_consistency() {
        // Files to audit for config path references
        // Note: We exclude this file (loader.rs) since it contains the test pattern strings
        let files_to_check = [
            "src/main.rs",
            "src/app_impl/mod.rs",
            "src/config/mod.rs",
            "src/config/types.rs",
            "src/watcher.rs",
            "src/setup.rs",
            "src/lib.rs",
        ];

        // The wrong pattern we're looking for (split to avoid self-detection)
        let wrong_pattern = format!(".scriptkit{}config.ts", "/");

        let mut violations = Vec::new();

        for file in files_to_check {
            let content = match fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue, // Skip files that don't exist
            };

            for (line_num, line) in content.lines().enumerate() {
                // Skip comments (documentation)
                if line.trim_start().starts_with("//") || line.trim_start().starts_with("///") {
                    continue;
                }

                // Detect WRONG pattern: .scriptkit/config.ts (missing /kit/)
                // This catches: "~/.scriptkit/config.ts" or ".scriptkit/config.ts"
                // But NOT: "~/.scriptkit/kit/config.ts" (the correct path)
                if line.contains(&wrong_pattern) {
                    violations.push(format!(
                        "{}:{}: {}\n  Found: {} (missing /kit/)\n  Expected: .scriptkit/kit/config.ts",
                        file,
                        line_num + 1,
                        line.trim(),
                        wrong_pattern
                    ));
                }
            }
        }

        if !violations.is_empty() {
            panic!(
                "Found {} inconsistent config path reference(s):\n\n{}\n\n\
                AUTHORITATIVE PATH: ~/.scriptkit/kit/config.ts",
                violations.len(),
                violations.join("\n\n")
            );
        }
    }

    #[test]
    fn test_config_loader_preserves_valid_fields_when_one_field_invalid() {
        let json = r#"{
            "hotkey": { "modifiers": ["meta"], "key": "Semicolon" },
            "editor": "nvim",
            "watcher": { "debounceMs": "bad-type", "stormThreshold": 321 }
        }"#;

        let config = parse_config_json(json, "test-correlation-id");

        // Valid fields remain intact
        assert_eq!(config.editor.as_deref(), Some("nvim"));
        assert_eq!(config.hotkey.key, "Semicolon");
        // Invalid watcher field should fall back per-field while preserving valid watcher fields
        let watcher = config.get_watcher();
        assert_eq!(watcher.storm_threshold, 321);
        assert_eq!(
            watcher.debounce_ms,
            super::super::defaults::DEFAULT_WATCHER_DEBOUNCE_MS
        );
    }

    #[test]
    fn test_config_loader_uses_default_hotkey_when_hotkey_missing_or_invalid() {
        let missing_hotkey = r#"{
            "editor": "vim"
        }"#;
        let missing_config = parse_config_json(missing_hotkey, "test-correlation-id");
        assert_eq!(
            missing_config.hotkey,
            HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            }
        );

        let invalid_hotkey = r#"{
            "hotkey": { "modifiers": "meta", "key": 7 },
            "editor": "hx"
        }"#;
        let invalid_config = parse_config_json(invalid_hotkey, "test-correlation-id");
        assert_eq!(
            invalid_config.hotkey,
            HotkeyConfig {
                modifiers: vec!["meta".to_string()],
                key: "Semicolon".to_string(),
            }
        );
        assert_eq!(invalid_config.editor.as_deref(), Some("hx"));
    }

    #[test]
    fn test_user_preferences_loader_parses_layout_and_theme_preset() {
        let json = r#"{
            "layout": { "standardHeight": 640, "maxHeight": 920 },
            "theme": { "presetId": "catppuccin-mocha" }
        }"#;

        let preferences = parse_user_preferences_json(json, "test-correlation-id");

        assert_eq!(preferences.layout.standard_height, 640.0);
        assert_eq!(preferences.layout.max_height, 920.0);
        assert_eq!(
            preferences.theme.preset_id.as_deref(),
            Some("catppuccin-mocha")
        );
    }

    #[test]
    fn test_user_preferences_loader_recovers_from_invalid_layout_field() {
        let json = r#"{
            "layout": { "standardHeight": "bad", "maxHeight": 920 },
            "theme": { "presetId": "nord" }
        }"#;

        let preferences = parse_user_preferences_json(json, "test-correlation-id");

        assert_eq!(
            preferences.layout.standard_height,
            super::super::defaults::DEFAULT_LAYOUT_STANDARD_HEIGHT
        );
        assert_eq!(
            preferences.layout.max_height,
            super::super::defaults::DEFAULT_LAYOUT_MAX_HEIGHT
        );
        assert_eq!(preferences.theme.preset_id.as_deref(), Some("nord"));
    }

    #[test]
    fn test_build_bun_extract_config_command_passes_module_path_as_argument() {
        let module_path = Path::new("/tmp/config-with-'quote'.js");
        let command = build_bun_extract_config_command(module_path);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();

        assert_eq!(args[0], "-e");
        assert_eq!(args[1], BUN_CONFIG_EXPORT_EXTRACT_SNIPPET);
        assert_eq!(args[2], module_path.to_string_lossy());
        assert!(
            !args[1].contains("require('"),
            "module path should not be string-interpolated into the eval script"
        );
    }
}
