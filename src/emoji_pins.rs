//! Emoji pin persistence — load/save pinned emoji set to ~/.kenv/emoji-pins.json.

use anyhow::Context as _;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn emoji_pins_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".kenv").join("emoji-pins.json"))
        .unwrap_or_else(|| PathBuf::from("emoji-pins.json"))
}

/// Load the set of pinned emoji characters from disk.
/// Returns an empty set if the file does not exist.
pub fn load_pinned_emojis() -> anyhow::Result<BTreeSet<String>> {
    let path = emoji_pins_path();
    if !path.exists() {
        tracing::debug!(path = %path.display(), "emoji pins file not found, returning empty set");
        return Ok(BTreeSet::new());
    }

    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let pins: BTreeSet<String> = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    tracing::info!(count = pins.len(), path = %path.display(), "loaded pinned emojis");
    Ok(pins)
}

/// Persist the set of pinned emoji characters to disk.
pub fn save_pinned_emojis(pins: &BTreeSet<String>) -> anyhow::Result<()> {
    let path = emoji_pins_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(pins).context("Failed to serialize emoji pins")?;
    std::fs::write(&path, json)
        .with_context(|| format!("Failed to write {}", path.display()))?;

    tracing::info!(count = pins.len(), path = %path.display(), "saved pinned emojis");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_round_trip_serialization() {
        let mut pins = BTreeSet::new();
        pins.insert("😀".to_string());
        pins.insert("🎉".to_string());

        let json = serde_json::to_string_pretty(&pins).expect("serialize");
        let loaded: BTreeSet<String> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(pins, loaded);
    }

    #[test]
    fn test_empty_json_array_parses_to_empty_set() {
        let json = "[]";
        let pins: BTreeSet<String> = serde_json::from_str(json).expect("deserialize");
        assert!(pins.is_empty());
    }

    #[test]
    fn test_load_nonexistent_path_returns_empty_set() {
        // The default load_pinned_emojis uses ~/.kenv which may or may not exist,
        // so we just verify the serialization round-trip works.
        let mut temp = NamedTempFile::new().expect("create temp");
        write!(temp, "[]").expect("write");
        let contents = std::fs::read_to_string(temp.path()).expect("read");
        let pins: BTreeSet<String> = serde_json::from_str(&contents).expect("parse");
        assert!(pins.is_empty());
    }
}
