//! Machine-readable story catalog snapshot for the design explorer.
//!
//! Provides a serializable view of every registered story, its surface,
//! compare-readiness, persisted selection, and per-variant metadata.
//! Agents can query this without opening a GPUI window.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;

use super::{all_stories, load_story_selections, selection_store_path, StorySelectionStore};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoryVariantSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub props: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoryCatalogEntry {
    pub story_id: String,
    pub name: String,
    pub category: String,
    pub surface: String,
    pub comparable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_variant_id: Option<String>,
    pub variants: Vec<StoryVariantSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorySurfaceSummary {
    pub surface: String,
    pub story_count: usize,
    pub comparable_story_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoryCatalogSnapshot {
    pub selection_store_path: String,
    pub story_count: usize,
    pub comparable_story_count: usize,
    pub surfaces: Vec<StorySurfaceSummary>,
    pub stories: Vec<StoryCatalogEntry>,
}

pub fn build_story_catalog_snapshot(selection_store: &StorySelectionStore) -> StoryCatalogSnapshot {
    let mut surface_counts: BTreeMap<String, (usize, usize)> = BTreeMap::new();

    let mut stories: Vec<StoryCatalogEntry> = all_stories()
        .map(|entry| {
            let story = &entry.story;
            let surface = story.surface().label().to_string();
            let variants = story.variants();
            let comparable = variants.len() > 1;

            let counter = surface_counts.entry(surface.clone()).or_insert((0, 0));
            counter.0 += 1;
            if comparable {
                counter.1 += 1;
            }

            StoryCatalogEntry {
                story_id: story.id().to_string(),
                name: story.name().to_string(),
                category: story.category().to_string(),
                surface,
                comparable,
                selected_variant_id: selection_store
                    .selected_variant(story.id())
                    .map(str::to_owned),
                variants: variants
                    .into_iter()
                    .map(|variant| StoryVariantSummary {
                        id: variant.stable_id(),
                        name: variant.name,
                        description: variant.description,
                        props: variant.props.into_iter().collect(),
                    })
                    .collect(),
            }
        })
        .collect();

    stories.sort_by(|left, right| left.story_id.cmp(&right.story_id));

    let comparable_story_count = stories.iter().filter(|story| story.comparable).count();
    let surfaces = surface_counts
        .into_iter()
        .map(
            |(surface, (story_count, comparable_story_count))| StorySurfaceSummary {
                surface,
                story_count,
                comparable_story_count,
            },
        )
        .collect();

    tracing::info!(
        event = "story_catalog_snapshot_built",
        story_count = stories.len(),
        comparable_story_count,
        "Built story catalog snapshot"
    );

    StoryCatalogSnapshot {
        selection_store_path: selection_store_path().display().to_string(),
        story_count: stories.len(),
        comparable_story_count,
        surfaces,
        stories,
    }
}

pub fn load_story_catalog_snapshot() -> Result<StoryCatalogSnapshot> {
    let selection_store = load_story_selections()
        .context("failed to load design explorer selections while building story catalog")?;
    Ok(build_story_catalog_snapshot(&selection_store))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storybook::StorySelectionStore;

    #[test]
    fn catalog_snapshot_includes_main_menu_story_metadata() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());
        let main_menu_story = snapshot
            .stories
            .iter()
            .find(|story| story.story_id == "main-menu")
            .expect("main-menu story should be registered");

        assert_eq!(main_menu_story.surface, "Main Menu");
        assert!(!main_menu_story.comparable);
        assert!(main_menu_story
            .variants
            .iter()
            .any(|variant| variant.id == "current-main-menu"));
    }

    #[test]
    fn catalog_snapshot_marks_persisted_selection() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("main-menu", "current-main-menu");

        let snapshot = build_story_catalog_snapshot(&store);
        let main_menu_story = snapshot
            .stories
            .iter()
            .find(|story| story.story_id == "main-menu")
            .expect("main-menu story should be registered");

        assert_eq!(
            main_menu_story.selected_variant_id.as_deref(),
            Some("current-main-menu")
        );
    }

    #[test]
    fn catalog_snapshot_serializes_to_camel_case_json() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("main-menu", "current-main-menu");
        let snapshot = build_story_catalog_snapshot(&store);
        let json = serde_json::to_string(&snapshot).expect("serialize catalog snapshot");

        assert!(json.contains("\"storyCount\""));
        assert!(json.contains("\"comparableStoryCount\""));
        assert!(json.contains("\"selectionStorePath\""));
        assert!(json.contains("\"storyId\""));
        assert!(json.contains("\"selectedVariantId\""));
    }

    #[test]
    fn catalog_snapshot_surfaces_only_include_main_menu() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());

        let main_menu_surface = snapshot
            .surfaces
            .iter()
            .find(|s| s.surface == "Main Menu")
            .expect("Main Menu surface should be present");
        assert_eq!(main_menu_surface.story_count, 1);
        assert_eq!(main_menu_surface.comparable_story_count, 0);
        assert_eq!(snapshot.surfaces.len(), 1);
    }

    #[test]
    fn catalog_snapshot_marks_persisted_main_menu_selection() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("main-menu", "current-main-menu");

        let snapshot = build_story_catalog_snapshot(&store);

        let main_menu_story = snapshot
            .stories
            .iter()
            .find(|story| story.story_id == "main-menu")
            .expect("main-menu story should be registered");
        assert_eq!(
            main_menu_story.selected_variant_id.as_deref(),
            Some("current-main-menu"),
            "Main menu story should reflect the persisted selection"
        );
    }

    #[test]
    fn catalog_snapshot_unset_selection_is_none() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());

        let main_menu_story = snapshot
            .stories
            .iter()
            .find(|story| story.story_id == "main-menu")
            .expect("main-menu story should be registered");
        assert_eq!(
            main_menu_story.selected_variant_id, None,
            "Main menu story should have no selection when store is empty"
        );
    }

    #[test]
    fn catalog_snapshot_includes_variant_props() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());
        let main_menu_story = snapshot
            .stories
            .iter()
            .find(|story| story.story_id == "main-menu")
            .expect("main-menu story should be registered");

        let current = main_menu_story
            .variants
            .iter()
            .find(|v| v.id == "current-main-menu")
            .expect("current-main-menu variant should exist");

        assert!(
            !current.props.is_empty(),
            "current main-menu variant should have props"
        );
    }
}
