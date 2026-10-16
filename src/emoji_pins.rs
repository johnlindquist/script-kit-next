//! Emoji picker persistence: pinned emoji (in the order you arranged them) in
//! `~/.kenv/emoji-pins.json` and the default skin tone in
//! `~/.kenv/emoji-settings.json`.

use anyhow::Context as _;
use std::path::{Path, PathBuf};

use crate::emoji::SkinTone;

fn kenv_file(name: &str) -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".kenv").join(name))
        .unwrap_or_else(|| PathBuf::from(name))
}

fn emoji_pins_path() -> PathBuf {
    kenv_file("emoji-pins.json")
}

fn emoji_settings_path() -> PathBuf {
    kenv_file("emoji-settings.json")
}

/// Pinned emoji in display order. A missing file means no pins.
pub fn load_pinned_emojis() -> anyhow::Result<Vec<String>> {
    load_pins_from(&emoji_pins_path())
}

fn load_pins_from(path: &Path) -> anyhow::Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let pins: Vec<String> = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let mut seen = std::collections::HashSet::new();
    Ok(pins
        .into_iter()
        .filter(|pin| seen.insert(pin.clone()))
        .collect())
}

/// Save the pinned emoji in display order.
pub fn save_pinned_emojis(pins: &[String]) -> anyhow::Result<()> {
    write_json(&emoji_pins_path(), &pins)?;
    tracing::info!(count = pins.len(), "saved pinned emojis");
    Ok(())
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(value).context("Failed to serialize")?;
    std::fs::write(path, json).with_context(|| format!("Failed to write {}", path.display()))
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct EmojiSettings {
    #[serde(default)]
    skin_tone: SkinTone,
}

/// The saved default skin tone, or none.
pub fn load_skin_tone() -> SkinTone {
    std::fs::read_to_string(emoji_settings_path())
        .ok()
        .and_then(|text| serde_json::from_str::<EmojiSettings>(&text).ok())
        .map(|settings| settings.skin_tone)
        .unwrap_or_default()
}

pub fn save_skin_tone(skin_tone: SkinTone) -> anyhow::Result<()> {
    write_json(&emoji_settings_path(), &EmojiSettings { skin_tone })
}

/// Where an emoji pin action leaves the pinned list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinEdit {
    Pin,
    Unpin,
    MoveUp,
    MoveDown,
}

/// Apply `edit` to `pins` for `emoji`. Returns whether anything changed.
pub fn apply_pin_edit(pins: &mut Vec<String>, emoji: &str, edit: PinEdit) -> bool {
    let position = pins.iter().position(|pin| pin == emoji);
    match (edit, position) {
        (PinEdit::Pin, None) => {
            pins.push(emoji.to_string());
            true
        }
        (PinEdit::Unpin, Some(index)) => {
            pins.remove(index);
            true
        }
        (PinEdit::MoveUp, Some(index)) if index > 0 => {
            pins.swap(index, index - 1);
            true
        }
        (PinEdit::MoveDown, Some(index)) if index + 1 < pins.len() => {
            pins.swap(index, index + 1);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_keep_their_order_and_old_files_still_load() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("emoji-pins.json");
        // Earlier versions wrote a sorted JSON array; it still reads.
        std::fs::write(&path, "[\"🎉\", \"😀\", \"🎉\"]").unwrap();
        assert_eq!(load_pins_from(&path).unwrap(), vec!["🎉", "😀"]);
        assert!(load_pins_from(&temp.path().join("missing.json"))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn pin_edits_reorder_and_toggle() {
        let mut pins = vec!["a".to_string(), "b".to_string()];
        assert!(apply_pin_edit(&mut pins, "c", PinEdit::Pin));
        assert!(!apply_pin_edit(&mut pins, "c", PinEdit::Pin));
        assert!(apply_pin_edit(&mut pins, "c", PinEdit::MoveUp));
        assert_eq!(pins, ["a", "c", "b"]);
        assert!(!apply_pin_edit(&mut pins, "a", PinEdit::MoveUp));
        assert!(!apply_pin_edit(&mut pins, "b", PinEdit::MoveDown));
        assert!(apply_pin_edit(&mut pins, "a", PinEdit::Unpin));
        assert_eq!(pins, ["c", "b"]);
    }

    #[test]
    fn skin_tone_settings_round_trip() {
        let json = serde_json::to_string(&EmojiSettings {
            skin_tone: SkinTone::MediumDark,
        })
        .unwrap();
        assert_eq!(json, "{\"skinTone\":\"mediumDark\"}");
        let back: EmojiSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.skin_tone, SkinTone::MediumDark);
        let empty: EmojiSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.skin_tone, SkinTone::None);
    }
}
