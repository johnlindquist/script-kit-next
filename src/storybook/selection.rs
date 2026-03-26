use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySelectionStore {
    #[serde(default)]
    pub selections: HashMap<String, String>,
}

impl StorySelectionStore {
    pub fn selected_variant(&self, story_id: &str) -> Option<&str> {
        self.selections.get(story_id).map(String::as_str)
    }

    pub fn set_selected_variant(
        &mut self,
        story_id: impl Into<String>,
        variant_id: impl Into<String>,
    ) {
        self.selections.insert(story_id.into(), variant_id.into());
    }
}

fn selection_store_path() -> PathBuf {
    crate::setup::get_kit_path().join("design-explorer-selections.json")
}

pub fn load_story_selections() -> Result<StorySelectionStore> {
    let path = selection_store_path();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(StorySelectionStore::default());
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read story selections: {}", path.display()));
        }
    };

    serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse story selections: {}", path.display()))
}

pub fn save_story_selections(store: &StorySelectionStore) -> Result<()> {
    let path = selection_store_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create story selection directory: {}",
                parent.display()
            )
        })?;
    }

    let contents = serde_json::to_string_pretty(store)
        .context("failed to serialize story selections for persistence")?;
    fs::write(&path, contents)
        .with_context(|| format!("failed to write story selections: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::StorySelectionStore;

    #[test]
    fn story_selection_store_roundtrips() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("button-story", "ghost");

        let serialized = serde_json::to_string(&store).expect("serialize story selection store");
        let deserialized: StorySelectionStore =
            serde_json::from_str(&serialized).expect("deserialize story selection store");

        assert_eq!(deserialized.selected_variant("button-story"), Some("ghost"));
        assert_eq!(deserialized, store);
    }
}
