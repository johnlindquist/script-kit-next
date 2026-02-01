//! Alias persistence for commands.
//!
//! Handles loading and saving user alias overrides to/from `~/.scriptkit/aliases.json`.
//! Format: HashMap<command_id, alias_string>
//!
//! Command ID formats:
//! - `builtin/{id}` - Built-in Script Kit features
//! - `app/{bundle_id}` - macOS applications (by bundle identifier)
//! - `script/{name}` - User scripts (by filename)
//! - `scriptlet/{name}` - Inline scriptlets (by name)

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use tracing::debug;

// ============================================================================
// Alias overrides cache (avoids file I/O in render paths)
// ============================================================================

/// Cache for loaded alias overrides to avoid file I/O on every render
static ALIAS_OVERRIDES_CACHE: OnceLock<Mutex<Option<HashMap<String, String>>>> = OnceLock::new();

/// Get cached alias overrides (for use in render code)
///
/// This avoids file I/O on every render call. The cache is invalidated
/// when aliases are saved via `save_alias_override()` or
/// `remove_alias_override()`, or manually via `invalidate_alias_cache()`.
pub fn get_cached_alias_overrides() -> HashMap<String, String> {
    let cache = ALIAS_OVERRIDES_CACHE.get_or_init(|| Mutex::new(None));

    let mut guard = match cache.lock() {
        Ok(g) => g,
        Err(_) => return HashMap::new(),
    };

    // If cache is populated, return it
    if let Some(ref cached) = *guard {
        return cached.clone();
    }

    // Cache miss - load from disk and cache
    let overrides = load_alias_overrides().unwrap_or_default();
    *guard = Some(overrides.clone());
    debug!("Alias overrides cache populated");
    overrides
}

/// Invalidate the alias overrides cache
///
/// Call this when the aliases file changes externally.
pub fn invalidate_alias_cache() {
    if let Some(cache) = ALIAS_OVERRIDES_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            *guard = None;
            debug!("Alias overrides cache invalidated");
        }
    }
}

/// Get the default path for alias overrides.
///
/// Returns `~/.scriptkit/aliases.json`
pub fn default_aliases_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".scriptkit")
        .join("aliases.json")
}

/// Load all alias overrides from ~/.scriptkit/aliases.json.
///
/// Returns a HashMap mapping command_id to alias string.
/// Returns an empty HashMap if the file doesn't exist.
///
/// # Errors
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load_alias_overrides() -> Result<HashMap<String, String>> {
    let path = default_aliases_path();

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read aliases file: {}", path.display()))?;

    let overrides: HashMap<String, String> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse aliases file: {}", path.display()))?;

    Ok(overrides)
}

/// Save an alias override for a specific command.
///
/// This function:
/// 1. Loads existing overrides (or creates empty map if file doesn't exist)
/// 2. Adds/updates the alias for the given command_id
/// 3. Writes the updated overrides back to ~/.scriptkit/aliases.json
///
/// # Arguments
/// * `command_id` - The unique identifier for the command (e.g., "builtin/clipboard-history")
/// * `alias` - The alias string to assign (e.g., "ch")
///
/// # Errors
/// Returns an error if the file cannot be written or the JSON cannot be serialized.
pub fn save_alias_override(command_id: &str, alias: &str) -> Result<()> {
    let path = default_aliases_path();

    // Load existing overrides
    let mut overrides = load_alias_overrides().unwrap_or_default();

    // Update with new alias
    overrides.insert(command_id.to_string(), alias.to_string());

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Write back to file
    let content =
        serde_json::to_string_pretty(&overrides).context("Failed to serialize aliases to JSON")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write aliases file: {}", path.display()))?;

    // Invalidate cache so next render picks up changes
    invalidate_alias_cache();

    Ok(())
}

/// Remove an alias override for a specific command.
///
/// This reverts the command to having no alias.
/// If the command_id doesn't exist in overrides, this is a no-op.
///
/// # Arguments
/// * `command_id` - The unique identifier for the command to remove
///
/// # Errors
/// Returns an error if the file cannot be read or written.
pub fn remove_alias_override(command_id: &str) -> Result<()> {
    let path = default_aliases_path();

    // If file doesn't exist, nothing to remove
    if !path.exists() {
        return Ok(());
    }

    // Load existing overrides
    let mut overrides = load_alias_overrides()?;

    // Remove the override
    overrides.remove(command_id);

    // Write back to file (even if empty, to reflect the removal)
    let content =
        serde_json::to_string_pretty(&overrides).context("Failed to serialize aliases to JSON")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write aliases file: {}", path.display()))?;

    // Invalidate cache so next render picks up changes
    invalidate_alias_cache();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_empty_when_no_file() {
        // Create a temp directory and test there
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases.json");

        // Directly test the logic: reading non-existent file
        assert!(!path.exists());
        let content = std::fs::read_to_string(&path);
        assert!(content.is_err());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases.json");

        // Create initial overrides
        let mut overrides: HashMap<String, String> = HashMap::new();
        overrides.insert("builtin/clipboard-history".to_string(), "ch".to_string());
        overrides.insert("builtin/app-launcher".to_string(), "apps".to_string());

        // Save to temp path
        let content = serde_json::to_string_pretty(&overrides).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Load back
        let loaded_content = std::fs::read_to_string(&path).unwrap();
        let loaded: HashMap<String, String> = serde_json::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(
            loaded.get("builtin/clipboard-history"),
            Some(&"ch".to_string())
        );
        assert_eq!(
            loaded.get("builtin/app-launcher"),
            Some(&"apps".to_string())
        );
    }

    #[test]
    fn test_remove_alias_from_map() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases.json");

        // Create initial overrides
        let mut overrides: HashMap<String, String> = HashMap::new();
        overrides.insert("builtin/clipboard-history".to_string(), "ch".to_string());
        overrides.insert("builtin/app-launcher".to_string(), "apps".to_string());

        // Save
        let content = serde_json::to_string_pretty(&overrides).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Remove one
        overrides.remove("builtin/clipboard-history");

        // Save again
        let content = serde_json::to_string_pretty(&overrides).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Verify
        let loaded_content = std::fs::read_to_string(&path).unwrap();
        let loaded: HashMap<String, String> = serde_json::from_str(&loaded_content).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(!loaded.contains_key("builtin/clipboard-history"));
        assert!(loaded.contains_key("builtin/app-launcher"));
    }

    #[test]
    fn test_alias_json_format() {
        // Verify the JSON format is human-readable
        let mut overrides: HashMap<String, String> = HashMap::new();
        overrides.insert("builtin/clipboard-history".to_string(), "ch".to_string());
        overrides.insert("app/com.apple.Safari".to_string(), "safari".to_string());

        let json = serde_json::to_string_pretty(&overrides).unwrap();

        // Verify structure
        assert!(json.contains("builtin/clipboard-history"));
        assert!(json.contains("ch"));
        assert!(json.contains("app/com.apple.Safari"));
        assert!(json.contains("safari"));
    }

    #[test]
    fn test_alias_validation() {
        // Aliases should be simple strings without special characters
        let valid_aliases = ["ch", "apps", "clip", "notes", "ai"];

        for alias in valid_aliases {
            // Just verify they're valid strings
            assert!(!alias.is_empty());
            assert!(alias
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
        }
    }
}
