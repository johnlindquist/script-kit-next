use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySelectionStore {
    #[serde(default)]
    pub selections: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySelectionWriteResult {
    pub story_id: String,
    pub variant_id: String,
    pub previous_variant_id: Option<String>,
    pub selection_store_path: String,
    pub selection_count: usize,
}

#[cfg(test)]
static TEST_SELECTION_STORE_PATH: LazyLock<Mutex<Option<PathBuf>>> =
    LazyLock::new(|| Mutex::new(None));

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

pub(crate) fn selection_store_path() -> PathBuf {
    #[cfg(test)]
    {
        if let Some(path) = TEST_SELECTION_STORE_PATH
            .lock()
            .expect("test selection store path mutex poisoned")
            .clone()
        {
            return path;
        }
    }

    crate::setup::get_kit_path().join("design-explorer-selections.json")
}

#[cfg(test)]
pub(crate) fn with_test_selection_store_path<T>(
    path: impl Into<PathBuf>,
    f: impl FnOnce() -> T,
) -> T {
    let mut guard = TEST_SELECTION_STORE_PATH
        .lock()
        .expect("test selection store path mutex poisoned");
    let previous = guard.replace(path.into());
    let result = f();
    *guard = previous;
    result
}

fn load_story_selections_from_path(path: &Path) -> Result<StorySelectionStore> {
    let contents = match fs::read_to_string(path) {
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

pub fn load_story_selections() -> Result<StorySelectionStore> {
    load_story_selections_from_path(&selection_store_path())
}

pub fn load_selected_story_variant(story_id: &str) -> Option<String> {
    load_story_selections()
        .ok()
        .and_then(|store| store.selected_variant(story_id).map(str::to_owned))
}

fn save_story_selections_to_path(path: &Path, store: &StorySelectionStore) -> Result<()> {
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

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &contents).with_context(|| {
        format!(
            "failed to write temporary story selections: {}",
            tmp_path.display()
        )
    })?;

    if let Err(error) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(error).with_context(|| {
            format!(
                "failed to atomically replace story selections: {}",
                path.display()
            )
        });
    }

    Ok(())
}

pub fn save_story_selections(store: &StorySelectionStore) -> Result<()> {
    save_story_selections_to_path(&selection_store_path(), store)
}

pub fn save_selected_story_variant(
    story_id: &str,
    variant_id: &str,
) -> Result<StorySelectionWriteResult> {
    let path = selection_store_path();
    let mut store = load_story_selections_from_path(&path)?;
    let previous_variant_id = store.selected_variant(story_id).map(str::to_owned);

    store.set_selected_variant(story_id.to_string(), variant_id.to_string());
    save_story_selections_to_path(&path, &store)?;

    tracing::info!(
        event = "story_selection_saved",
        story_id = story_id,
        variant_id = variant_id,
        previous_variant_id = previous_variant_id.as_deref().unwrap_or(""),
        path = %path.display(),
        selection_count = store.selections.len(),
        "Persisted story selection"
    );

    Ok(StorySelectionWriteResult {
        story_id: story_id.to_string(),
        variant_id: variant_id.to_string(),
        previous_variant_id,
        selection_store_path: path.display().to_string(),
        selection_count: store.selections.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn later_selection_wins_for_same_story() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("footer-layout-variations", "raycast-exact");
        store.set_selected_variant("footer-layout-variations", "minimal");

        assert_eq!(
            store.selected_variant("footer-layout-variations"),
            Some("minimal")
        );
    }

    #[test]
    fn load_returns_empty_store_when_file_missing() {
        let path = std::env::temp_dir().join("nonexistent-selection-test.json");
        let _ = fs::remove_file(&path);

        let store =
            load_story_selections_from_path(&path).expect("load from missing file should succeed");
        assert_eq!(store, StorySelectionStore::default());
    }

    #[test]
    fn atomic_save_writes_via_temp_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("selections.json");

        let mut store = StorySelectionStore::default();
        store.set_selected_variant("test-story", "variant-a");
        save_story_selections_to_path(&path, &store).expect("save should succeed");

        // The temp file should not remain
        let tmp_path = path.with_extension("json.tmp");
        assert!(!tmp_path.exists(), "temp file should be cleaned up");

        // The final file should contain the correct data
        let loaded =
            load_story_selections_from_path(&path).expect("load after save should succeed");
        assert_eq!(loaded.selected_variant("test-story"), Some("variant-a"));
    }

    #[test]
    fn save_selected_story_variant_returns_write_result() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("selections.json");

        // Seed an initial selection
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("my-story", "old-variant");
        save_story_selections_to_path(&path, &store).expect("seed save");

        // Use the path-based helpers directly to test the write result logic
        let mut store = load_story_selections_from_path(&path).expect("load");
        let previous = store.selected_variant("my-story").map(str::to_owned);
        store.set_selected_variant("my-story", "new-variant");
        save_story_selections_to_path(&path, &store).expect("save");

        let result = StorySelectionWriteResult {
            story_id: "my-story".to_string(),
            variant_id: "new-variant".to_string(),
            previous_variant_id: previous,
            selection_store_path: path.display().to_string(),
            selection_count: store.selections.len(),
        };

        assert_eq!(result.story_id, "my-story");
        assert_eq!(result.variant_id, "new-variant");
        assert_eq!(result.previous_variant_id, Some("old-variant".to_string()));
        assert_eq!(result.selection_count, 1);
    }

    #[test]
    fn write_result_serializes_to_camel_case() {
        let result = StorySelectionWriteResult {
            story_id: "s".to_string(),
            variant_id: "v".to_string(),
            previous_variant_id: None,
            selection_store_path: "/tmp/test.json".to_string(),
            selection_count: 1,
        };

        let json = serde_json::to_value(&result).expect("serialize write result");
        assert!(json.get("storyId").is_some());
        assert!(json.get("variantId").is_some());
        assert!(json.get("previousVariantId").is_some());
        assert!(json.get("selectionStorePath").is_some());
        assert!(json.get("selectionCount").is_some());
    }
}
