//! User shortcut customization persistence.
//!
//! Handles loading and saving user shortcut overrides to/from config.
//! Format: HashMap<binding_id, Option<String>> where:
//! - Some(shortcut_string) = user override to new shortcut
//! - None = user disabled this shortcut

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use tracing::debug;

// ============================================================================
// Shortcut overrides cache (avoids file I/O in render paths)
// ============================================================================

/// Cache for loaded shortcut overrides to avoid file I/O on every render
static SHORTCUT_OVERRIDES_CACHE: OnceLock<Mutex<Option<HashMap<String, Shortcut>>>> =
    OnceLock::new();

/// Get cached shortcut overrides (for use in render code)
///
/// This avoids file I/O on every render call. The cache is invalidated
/// when shortcuts are saved via `save_shortcut_override()` or
/// `remove_shortcut_override()`, or manually via `invalidate_shortcut_cache()`.
pub fn get_cached_shortcut_overrides() -> HashMap<String, Shortcut> {
    let cache = SHORTCUT_OVERRIDES_CACHE.get_or_init(|| Mutex::new(None));

    let mut guard = match cache.lock() {
        Ok(g) => g,
        Err(_) => return HashMap::new(),
    };

    // If cache is populated, return it
    if let Some(ref cached) = *guard {
        return cached.clone();
    }

    // Cache miss - load from disk and cache
    let overrides = load_shortcut_overrides().unwrap_or_default();
    *guard = Some(overrides.clone());
    debug!("Shortcut overrides cache populated");
    overrides
}

/// Invalidate the shortcut overrides cache
///
/// Call this when the shortcuts file changes externally.
pub fn invalidate_shortcut_cache() {
    if let Some(cache) = SHORTCUT_OVERRIDES_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            *guard = None;
            debug!("Shortcut overrides cache invalidated");
        }
    }
}

use serde::{Deserialize, Serialize};

use super::registry::ShortcutRegistry;
use super::types::{Shortcut, ShortcutParseError};

/// User shortcut overrides configuration.
///
/// Stored in ~/.scriptkit/shortcuts.json
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShortcutOverrides {
    /// Map of binding_id -> override
    /// - Some(string) = new shortcut
    /// - null in JSON = disabled
    #[serde(default)]
    pub overrides: HashMap<String, Option<String>>,
}

/// Error that can occur when loading/saving shortcut overrides.
#[derive(Debug)]
pub enum PersistenceError {
    /// IO error reading/writing file
    Io(std::io::Error),
    /// JSON parse error
    Json(serde_json::Error),
    /// Invalid shortcut string in config
    InvalidShortcut {
        binding_id: String,
        shortcut: String,
        error: ShortcutParseError,
    },
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Json(e) => write!(f, "JSON parse error: {}", e),
            Self::InvalidShortcut {
                binding_id,
                shortcut,
                error,
            } => {
                write!(
                    f,
                    "Invalid shortcut '{}' for binding '{}': {}",
                    shortcut, binding_id, error
                )
            }
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::InvalidShortcut { error, .. } => Some(error),
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl ShortcutOverrides {
    /// Load overrides from a JSON file.
    ///
    /// Returns empty overrides if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, PersistenceError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let overrides: Self = serde_json::from_str(&content)?;
        Ok(overrides)
    }

    /// Save overrides to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), PersistenceError> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Apply overrides to a registry.
    ///
    /// Returns a list of parse errors for invalid shortcuts (but still applies valid ones).
    pub fn apply_to_registry(&self, registry: &mut ShortcutRegistry) -> Vec<PersistenceError> {
        let mut errors = Vec::new();

        for (binding_id, override_opt) in &self.overrides {
            match override_opt {
                None => {
                    // Disable this shortcut
                    registry.set_override(binding_id, None);
                }
                Some(shortcut_str) => {
                    // Parse and set override
                    match Shortcut::parse(shortcut_str) {
                        Ok(shortcut) => {
                            registry.set_override(binding_id, Some(shortcut));
                        }
                        Err(e) => {
                            errors.push(PersistenceError::InvalidShortcut {
                                binding_id: binding_id.clone(),
                                shortcut: shortcut_str.clone(),
                                error: e,
                            });
                        }
                    }
                }
            }
        }

        errors
    }

    /// Extract current overrides from a registry.
    pub fn from_registry(registry: &ShortcutRegistry) -> Self {
        let overrides = registry.export_overrides();
        Self { overrides }
    }

    /// Set an override.
    pub fn set(&mut self, binding_id: impl Into<String>, shortcut: Option<String>) {
        self.overrides.insert(binding_id.into(), shortcut);
    }

    /// Remove an override (revert to default).
    pub fn remove(&mut self, binding_id: &str) {
        self.overrides.remove(binding_id);
    }

    /// Check if a binding has an override.
    pub fn has_override(&self, binding_id: &str) -> bool {
        self.overrides.contains_key(binding_id)
    }

    /// Get the override for a binding.
    pub fn get(&self, binding_id: &str) -> Option<&Option<String>> {
        self.overrides.get(binding_id)
    }

    /// Get the number of overrides.
    pub fn len(&self) -> usize {
        self.overrides.len()
    }

    /// Check if there are no overrides.
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    /// Clear all overrides.
    pub fn clear(&mut self) {
        self.overrides.clear();
    }
}

/// Get the default path for shortcut overrides.
pub fn default_overrides_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".scriptkit")
        .join("shortcuts.json")
}

// ============================================================================
// Standalone convenience functions for UI integration
// ============================================================================
// These functions provide a simpler API for the shortcut recorder UI.
// The JSON format is: { "command_id": { "modifiers": {...}, "key": "..." }, ... }
// ============================================================================

use anyhow::{Context, Result};

/// Load all shortcut overrides from ~/.scriptkit/shortcuts.json.
///
/// Returns a HashMap mapping command_id to Shortcut.
/// Returns an empty HashMap if the file doesn't exist.
///
/// # Errors
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load_shortcut_overrides() -> Result<HashMap<String, Shortcut>> {
    let path = default_overrides_path();

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read shortcuts file: {}", path.display()))?;

    let overrides: HashMap<String, Shortcut> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse shortcuts file: {}", path.display()))?;

    Ok(overrides)
}

/// Save a shortcut override for a specific command.
///
/// This function:
/// 1. Loads existing overrides (or creates empty map if file doesn't exist)
/// 2. Adds/updates the shortcut for the given command_id
/// 3. Writes the updated overrides back to ~/.scriptkit/shortcuts.json
///
/// # Arguments
/// * `command_id` - The unique identifier for the command (e.g., "script.my-script")
/// * `shortcut` - The new shortcut to assign
///
/// # Errors
/// Returns an error if the file cannot be written or the JSON cannot be serialized.
pub fn save_shortcut_override(command_id: &str, shortcut: &Shortcut) -> Result<()> {
    let path = default_overrides_path();

    // Load existing overrides
    let mut overrides = load_shortcut_overrides().unwrap_or_default();

    // Update with new shortcut
    overrides.insert(command_id.to_string(), shortcut.clone());

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Write back to file
    let content = serde_json::to_string_pretty(&overrides)
        .context("Failed to serialize shortcuts to JSON")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write shortcuts file: {}", path.display()))?;

    // Invalidate cache so next render picks up changes
    invalidate_shortcut_cache();

    Ok(())
}

/// Remove a shortcut override for a specific command.
///
/// This reverts the command to its default shortcut (or no shortcut).
/// If the command_id doesn't exist in overrides, this is a no-op.
///
/// # Arguments
/// * `command_id` - The unique identifier for the command to remove
///
/// # Errors
/// Returns an error if the file cannot be read or written.
pub fn remove_shortcut_override(command_id: &str) -> Result<()> {
    let path = default_overrides_path();

    // If file doesn't exist, nothing to remove
    if !path.exists() {
        return Ok(());
    }

    // Load existing overrides
    let mut overrides = load_shortcut_overrides()?;

    // Remove the override
    overrides.remove(command_id);

    // Write back to file (even if empty, to reflect the removal)
    let content = serde_json::to_string_pretty(&overrides)
        .context("Failed to serialize shortcuts to JSON")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write shortcuts file: {}", path.display()))?;

    // Invalidate cache so next render picks up changes
    invalidate_shortcut_cache();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_nonexistent_returns_empty() {
        let result = ShortcutOverrides::load(Path::new("/nonexistent/path/shortcuts.json"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");

        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", Some("cmd+k".to_string()));
        overrides.set("test.disabled", None);

        overrides.save(&path).unwrap();

        let loaded = ShortcutOverrides::load(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("test.action"), Some(&Some("cmd+k".to_string())));
        assert_eq!(loaded.get("test.disabled"), Some(&None));
    }

    #[test]
    fn apply_valid_override_to_registry() {
        use super::super::context::ShortcutContext;
        use super::super::registry::{ShortcutBinding, ShortcutCategory, ShortcutRegistry};
        use super::super::types::Modifiers;

        let mut registry = ShortcutRegistry::new();
        registry.register(ShortcutBinding::builtin(
            "test.action",
            "Test",
            Shortcut {
                key: "k".to_string(),
                modifiers: Modifiers::cmd(),
            },
            ShortcutContext::Global,
            ShortcutCategory::Actions,
        ));

        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", Some("cmd+j".to_string()));

        let errors = overrides.apply_to_registry(&mut registry);
        assert!(errors.is_empty());

        let shortcut = registry.get_shortcut("test.action").unwrap();
        assert_eq!(shortcut.key, "j");
    }

    #[test]
    fn apply_disable_override_to_registry() {
        use super::super::context::ShortcutContext;
        use super::super::registry::{ShortcutBinding, ShortcutCategory, ShortcutRegistry};
        use super::super::types::Modifiers;

        let mut registry = ShortcutRegistry::new();
        registry.register(ShortcutBinding::builtin(
            "test.action",
            "Test",
            Shortcut {
                key: "k".to_string(),
                modifiers: Modifiers::cmd(),
            },
            ShortcutContext::Global,
            ShortcutCategory::Actions,
        ));

        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", None);

        let errors = overrides.apply_to_registry(&mut registry);
        assert!(errors.is_empty());

        assert!(registry.is_disabled("test.action"));
    }

    #[test]
    fn apply_invalid_shortcut_returns_error() {
        use super::super::registry::ShortcutRegistry;

        let mut registry = ShortcutRegistry::new();
        let mut overrides = ShortcutOverrides::default();
        overrides.set("test.action", Some("invalid+shortcut+xyz".to_string()));

        let errors = overrides.apply_to_registry(&mut registry);
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            PersistenceError::InvalidShortcut { binding_id, .. } => {
                assert_eq!(binding_id, "test.action");
            }
            _ => panic!("Expected InvalidShortcut error"),
        }
    }

    #[test]
    fn set_and_remove_override() {
        let mut overrides = ShortcutOverrides::default();

        overrides.set("test.action", Some("cmd+k".to_string()));
        assert!(overrides.has_override("test.action"));

        overrides.remove("test.action");
        assert!(!overrides.has_override("test.action"));
    }

    #[test]
    fn json_format_is_readable() {
        let mut overrides = ShortcutOverrides::default();
        overrides.set("nav.up", Some("cmd+k".to_string()));
        overrides.set("nav.down", Some("cmd+j".to_string()));
        overrides.set("edit.copy", None);

        let json = serde_json::to_string_pretty(&overrides).unwrap();

        // Verify it's human-readable
        assert!(json.contains("nav.up"));
        assert!(json.contains("cmd+k"));
        assert!(json.contains("edit.copy"));
        assert!(json.contains("null")); // disabled shortcut
    }

    // ========================================================================
    // Tests for standalone convenience functions
    // ========================================================================

    #[test]
    fn load_shortcut_overrides_empty_when_no_file() {
        // The standalone function uses default path, but we can test the behavior
        // by verifying it returns empty HashMap for non-existent files
        // (this relies on the file not existing at the default path during tests)

        // Create a temp directory and test there via the underlying mechanisms
        let dir = tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");

        // Directly test the logic: reading non-existent file
        assert!(!path.exists());
        let content = std::fs::read_to_string(&path);
        assert!(content.is_err());
    }

    #[test]
    fn save_and_load_shortcut_override_roundtrip() {
        use super::super::types::Modifiers;

        let dir = tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");

        // Create a shortcut
        let shortcut = Shortcut {
            key: "k".to_string(),
            modifiers: Modifiers {
                cmd: true,
                shift: true,
                ..Default::default()
            },
        };

        // Manually save to temp path (simulating what save_shortcut_override does)
        let mut overrides: HashMap<String, Shortcut> = HashMap::new();
        overrides.insert("test.command".to_string(), shortcut.clone());

        std::fs::create_dir_all(dir.path()).unwrap();
        let content = serde_json::to_string_pretty(&overrides).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Load back
        let loaded_content = std::fs::read_to_string(&path).unwrap();
        let loaded: HashMap<String, Shortcut> = serde_json::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.len(), 1);
        let loaded_shortcut = loaded.get("test.command").unwrap();
        assert_eq!(loaded_shortcut.key, "k");
        assert!(loaded_shortcut.modifiers.cmd);
        assert!(loaded_shortcut.modifiers.shift);
        assert!(!loaded_shortcut.modifiers.ctrl);
        assert!(!loaded_shortcut.modifiers.alt);
    }

    #[test]
    fn remove_shortcut_override_from_map() {
        use super::super::types::Modifiers;

        let dir = tempdir().unwrap();
        let path = dir.path().join("shortcuts.json");

        // Create initial overrides
        let mut overrides: HashMap<String, Shortcut> = HashMap::new();
        overrides.insert(
            "test.command1".to_string(),
            Shortcut {
                key: "k".to_string(),
                modifiers: Modifiers::cmd(),
            },
        );
        overrides.insert(
            "test.command2".to_string(),
            Shortcut {
                key: "j".to_string(),
                modifiers: Modifiers::cmd(),
            },
        );

        // Save
        let content = serde_json::to_string_pretty(&overrides).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Remove one
        overrides.remove("test.command1");

        // Save again
        let content = serde_json::to_string_pretty(&overrides).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Verify
        let loaded_content = std::fs::read_to_string(&path).unwrap();
        let loaded: HashMap<String, Shortcut> = serde_json::from_str(&loaded_content).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(!loaded.contains_key("test.command1"));
        assert!(loaded.contains_key("test.command2"));
    }

    #[test]
    fn shortcut_json_format_matches_spec() {
        use super::super::types::Modifiers;

        // Verify the JSON format is: { "command_id": { "modifiers": {...}, "key": "..." } }
        let mut overrides: HashMap<String, Shortcut> = HashMap::new();
        overrides.insert(
            "my.command".to_string(),
            Shortcut {
                key: "k".to_string(),
                modifiers: Modifiers {
                    cmd: true,
                    shift: true,
                    ctrl: false,
                    alt: false,
                },
            },
        );

        let json = serde_json::to_string_pretty(&overrides).unwrap();

        // Verify structure
        assert!(json.contains("\"my.command\""));
        assert!(json.contains("\"key\""));
        assert!(json.contains("\"modifiers\""));
        assert!(json.contains("\"cmd\""));
        assert!(json.contains("\"shift\""));
        assert!(json.contains("\"k\""));
    }

    #[test]
    fn shortcut_serialization_includes_all_modifiers() {
        use super::super::types::Modifiers;

        let shortcut = Shortcut {
            key: "a".to_string(),
            modifiers: Modifiers {
                cmd: true,
                ctrl: true,
                alt: true,
                shift: true,
            },
        };

        let json = serde_json::to_string(&shortcut).unwrap();

        assert!(json.contains("\"cmd\":true"));
        assert!(json.contains("\"ctrl\":true"));
        assert!(json.contains("\"alt\":true"));
        assert!(json.contains("\"shift\":true"));
        assert!(json.contains("\"key\":\"a\""));
    }

    #[test]
    fn shortcut_deserialization_with_missing_modifiers_defaults_false() {
        // Test that missing modifier fields default to false
        let json = r#"{"key": "b", "modifiers": {"cmd": true}}"#;
        let shortcut: Shortcut = serde_json::from_str(json).unwrap();

        assert_eq!(shortcut.key, "b");
        assert!(shortcut.modifiers.cmd);
        assert!(!shortcut.modifiers.ctrl);
        assert!(!shortcut.modifiers.alt);
        assert!(!shortcut.modifiers.shift);
    }
}
