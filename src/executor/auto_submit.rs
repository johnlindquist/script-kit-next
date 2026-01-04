//! AUTO_SUBMIT Mode for Autonomous Testing
//!
//! These functions are used by the UI layer (main.rs) to enable autonomous
//! testing of prompts. The #[allow(dead_code)] is temporary until integration
//! is complete.

use std::time::Duration;

/// Check if AUTO_SUBMIT mode is enabled via environment variable.
///
/// When AUTO_SUBMIT=true or AUTO_SUBMIT=1, prompts will be automatically
/// submitted after a configurable delay for autonomous testing.
///
/// # Environment Variables
/// - `AUTO_SUBMIT` - Set to "true" or "1" to enable auto-submit mode
///
/// # Example
/// ```bash
/// AUTO_SUBMIT=true ./target/debug/script-kit-gpui tests/sdk/test-arg.ts
/// ```
#[allow(dead_code)]
pub fn is_auto_submit_enabled() -> bool {
    std::env::var("AUTO_SUBMIT")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Get the delay before auto-submitting a prompt.
///
/// This allows the UI to render before automatically submitting,
/// useful for visual testing or debugging.
///
/// # Environment Variables
/// - `AUTO_SUBMIT_DELAY_MS` - Delay in milliseconds (default: 100)
///
/// # Returns
/// Duration for the delay, defaults to 100ms if not specified or invalid.
#[allow(dead_code)]
pub fn get_auto_submit_delay() -> Duration {
    std::env::var("AUTO_SUBMIT_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(100))
}

/// Get the value to auto-submit for prompts.
///
/// If set, this value will be submitted instead of selecting from choices.
/// Useful for testing specific input scenarios.
///
/// # Environment Variables
/// - `AUTO_SUBMIT_VALUE` - The value to submit (optional)
///
/// # Returns
/// Some(value) if AUTO_SUBMIT_VALUE is set, None otherwise.
#[allow(dead_code)]
pub fn get_auto_submit_value() -> Option<String> {
    std::env::var("AUTO_SUBMIT_VALUE").ok()
}

/// Get the index of the choice to auto-select.
///
/// If set, this index will be used to select from the choices array.
/// If the index is out of bounds, defaults to 0.
///
/// # Environment Variables
/// - `AUTO_SUBMIT_INDEX` - The 0-based index to select (default: 0)
///
/// # Returns
/// The index to select, defaults to 0 if not specified or invalid.
#[allow(dead_code)]
pub fn get_auto_submit_index() -> usize {
    std::env::var("AUTO_SUBMIT_INDEX")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Configuration for AUTO_SUBMIT mode.
///
/// This struct captures all AUTO_SUBMIT environment variables at initialization time,
/// providing a consistent snapshot for the duration of the session.
///
/// # Example
/// ```bash
/// AUTO_SUBMIT=true AUTO_SUBMIT_DELAY_MS=200 ./target/debug/script-kit-gpui
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AutoSubmitConfig {
    /// Whether auto-submit mode is enabled
    pub enabled: bool,
    /// Delay before auto-submitting (in milliseconds)
    pub delay: Duration,
    /// Override value to submit (if set)
    pub value_override: Option<String>,
    /// Index of choice to select (0-based)
    pub index: usize,
}

impl Default for AutoSubmitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            delay: Duration::from_millis(100),
            value_override: None,
            index: 0,
        }
    }
}

#[allow(dead_code)]
impl AutoSubmitConfig {
    /// Create a new AutoSubmitConfig by reading environment variables.
    ///
    /// This captures the current state of all AUTO_SUBMIT env vars.
    pub fn from_env() -> Self {
        Self {
            enabled: is_auto_submit_enabled(),
            delay: get_auto_submit_delay(),
            value_override: get_auto_submit_value(),
            index: get_auto_submit_index(),
        }
    }

    /// Get the default value for an arg prompt with choices.
    ///
    /// Priority:
    /// 1. If `value_override` is set, use it
    /// 2. Otherwise, use `choices[index].value` (clamped to valid range)
    /// 3. If no choices, return None
    pub fn get_arg_value(&self, choices: &[crate::protocol::Choice]) -> Option<String> {
        // Check for value override first
        if let Some(ref override_value) = self.value_override {
            return Some(override_value.clone());
        }

        // Get choice by index (clamped to valid range)
        if choices.is_empty() {
            return None;
        }

        let idx = self.index.min(choices.len() - 1);
        Some(choices[idx].value.clone())
    }

    /// Get the default value for a div prompt.
    ///
    /// Div prompts just need dismissal, so we return None (no value needed).
    pub fn get_div_value(&self) -> Option<String> {
        None
    }

    /// Get the default value for an editor prompt.
    ///
    /// Returns the original content unchanged if no override is set.
    pub fn get_editor_value(&self, original_content: &str) -> Option<String> {
        if let Some(ref override_value) = self.value_override {
            Some(override_value.clone())
        } else {
            Some(original_content.to_string())
        }
    }

    /// Get the default value for a terminal prompt.
    ///
    /// Terminal prompts return the exit code (0 for success).
    pub fn get_term_value(&self) -> Option<String> {
        Some("0".to_string())
    }

    /// Get the default value for a form prompt.
    ///
    /// Forms return an empty JSON object by default.
    pub fn get_form_value(&self) -> Option<String> {
        Some("{}".to_string())
    }

    /// Get the default value for a select prompt (multi-select).
    ///
    /// Returns a JSON array with the first choice selected.
    pub fn get_select_value(&self, choices: &[crate::protocol::Choice]) -> Option<String> {
        if choices.is_empty() {
            return Some("[]".to_string());
        }

        let idx = self.index.min(choices.len() - 1);
        let value = &choices[idx].value;
        Some(format!("[\"{}\"]", value))
    }

    /// Get the default value for a fields prompt.
    ///
    /// Returns a JSON array of empty strings matching the number of fields.
    pub fn get_fields_value(&self, field_count: usize) -> Option<String> {
        let empty_strings: Vec<&str> = vec![""; field_count];
        Some(serde_json::to_string(&empty_strings).unwrap_or_else(|_| "[]".to_string()))
    }

    /// Get the default value for a path prompt.
    ///
    /// Returns "/tmp/test-path" as the default path.
    pub fn get_path_value(&self) -> Option<String> {
        Some("/tmp/test-path".to_string())
    }

    /// Get the default value for a hotkey prompt.
    ///
    /// Returns a JSON object representing Cmd+A.
    pub fn get_hotkey_value(&self) -> Option<String> {
        Some(r#"{"key":"a","command":true}"#.to_string())
    }

    /// Get the default value for a drop prompt.
    ///
    /// Returns a JSON array with a test file path.
    pub fn get_drop_value(&self) -> Option<String> {
        Some(r#"[{"path":"/tmp/test.txt"}]"#.to_string())
    }
}

/// Get a snapshot of the current AUTO_SUBMIT configuration.
///
/// This is the main entry point for checking auto-submit settings.
/// Call this once at startup or when needed, rather than repeatedly
/// reading env vars.
///
#[allow(dead_code)]
pub fn get_auto_submit_config() -> AutoSubmitConfig {
    AutoSubmitConfig::from_env()
}
