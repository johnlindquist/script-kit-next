//! Machine-readable story catalog snapshot for the design explorer.
//!
//! Provides a serializable view of every registered story, its surface,
//! compare-readiness, persisted selection, and per-variant metadata.
//! Agents can query this without opening a GPUI window.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;

use super::adoption::SurfaceSelectionResolution;
use super::{
    all_stories, load_story_selections, resolve_main_menu_variant, resolve_mini_ai_chat_style,
    resolve_notes_window_style, selection_store_path, StorySelectionStore,
};

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

            let story_id_str = story.id().to_string();
            StoryCatalogEntry {
                name: story.name().to_string(),
                category: story.category().to_string(),
                surface,
                comparable,
                selected_variant_id: selection_store
                    .selected_variant(story.id())
                    .map(str::to_owned),
                variants: variants
                    .into_iter()
                    .map(|variant| {
                        let variant_id = variant.stable_id();
                        let mut props: BTreeMap<String, String> =
                            variant.props.into_iter().collect();
                        augment_runtime_fixture_props(&story_id_str, &variant_id, &mut props);
                        StoryVariantSummary {
                            id: variant_id,
                            name: variant.name,
                            description: variant.description,
                            props,
                        }
                    })
                    .collect(),
                story_id: story_id_str,
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

    let runtime_fixture_variant_count = stories
        .iter()
        .flat_map(|story| story.variants.iter())
        .filter(|variant| is_runtime_fixture_variant(variant))
        .count();
    let missing_runtime_fixture_variant_count = stories
        .iter()
        .flat_map(|story| story.variants.iter())
        .filter(|variant| is_runtime_fixture_variant(variant))
        .filter(|variant| runtime_fixture_is_incomplete(variant))
        .count();

    tracing::info!(
        event = "story_catalog_snapshot_built",
        story_count = stories.len(),
        comparable_story_count,
        runtime_fixture_variant_count,
        missing_runtime_fixture_variant_count,
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

// ─── Adopted Surface Resolution Snapshot ───────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptedSurfaceResolutionEntry {
    pub story_id: String,
    pub surface: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_variant_id: Option<String>,
    pub resolved_variant_id: String,
    pub fallback_used: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdoptedSurfaceResolutionSnapshot {
    pub selection_store_path: String,
    pub surface_count: usize,
    pub surfaces: Vec<AdoptedSurfaceResolutionEntry>,
}

pub fn build_adopted_surface_resolution_snapshot(
    selection_store: &StorySelectionStore,
) -> AdoptedSurfaceResolutionSnapshot {
    let (_, main_menu_resolution) =
        resolve_main_menu_variant(selection_store.selected_variant("main-menu"));
    let (_, mini_ai_chat_resolution) =
        resolve_mini_ai_chat_style(selection_store.selected_variant("mini-ai-chat-variations"));
    let (_, notes_window_resolution) =
        resolve_notes_window_style(selection_store.selected_variant("notes-window"));

    let mut surfaces = vec![
        surface_resolution_entry("Main Menu", main_menu_resolution),
        surface_resolution_entry("Mini ACP Chat", mini_ai_chat_resolution),
        surface_resolution_entry("Notes Window", notes_window_resolution),
    ];
    surfaces.sort_by(|left, right| left.story_id.cmp(&right.story_id));

    tracing::info!(
        event = "adopted_surface_resolution_snapshot_built",
        surface_count = surfaces.len(),
        "Built adopted surface resolution snapshot"
    );

    AdoptedSurfaceResolutionSnapshot {
        selection_store_path: selection_store_path().display().to_string(),
        surface_count: surfaces.len(),
        surfaces,
    }
}

pub fn load_adopted_surface_resolution_snapshot() -> Result<AdoptedSurfaceResolutionSnapshot> {
    let selection_store = load_story_selections().context(
        "failed to load design explorer selections while building adopted surface resolutions",
    )?;
    Ok(build_adopted_surface_resolution_snapshot(&selection_store))
}

fn surface_resolution_entry(
    surface: &str,
    resolution: SurfaceSelectionResolution,
) -> AdoptedSurfaceResolutionEntry {
    AdoptedSurfaceResolutionEntry {
        story_id: resolution.story_id,
        surface: surface.to_string(),
        requested_variant_id: resolution.requested_variant_id,
        resolved_variant_id: resolution.resolved_variant_id,
        fallback_used: resolution.fallback_used,
    }
}

// ─── Runtime Fixture Completeness Helpers ──────────────────────────────

fn runtime_fixture_surface_for_story(story_id: &str) -> Option<&'static str> {
    match story_id {
        "main-menu" => Some("main-menu"),
        "mini-ai-chat-variations" => Some("mini-ai-chat"),
        "notes-window" => Some("notes-window"),
        _ => None,
    }
}

fn augment_runtime_fixture_props(
    story_id: &str,
    variant_id: &str,
    props: &mut BTreeMap<String, String>,
) {
    if props.get("representation").map(String::as_str) != Some("runtimeFixture") {
        return;
    }
    let Some(surface) = runtime_fixture_surface_for_story(story_id) else {
        return;
    };
    let presence = super::runtime_fixture::describe_runtime_fixture(surface, variant_id);
    props.insert(
        "fixtureImagePresent".to_string(),
        presence.image_present.to_string(),
    );
    props.insert(
        "fixtureManifestPresent".to_string(),
        presence.manifest_present.to_string(),
    );
}

fn is_runtime_fixture_variant(variant: &StoryVariantSummary) -> bool {
    variant.props.get("representation").map(String::as_str) == Some("runtimeFixture")
}

fn runtime_fixture_is_incomplete(variant: &StoryVariantSummary) -> bool {
    variant.props.get("fixtureImagePresent").map(String::as_str) != Some("true")
        || variant
            .props
            .get("fixtureManifestPresent")
            .map(String::as_str)
            != Some("true")
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
    fn catalog_snapshot_surfaces_include_design_lab_surfaces() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());

        let main_menu_surface = snapshot
            .surfaces
            .iter()
            .find(|s| s.surface == "Main Menu")
            .expect("Main Menu surface should be present");
        assert_eq!(main_menu_surface.story_count, 1);
        assert_eq!(main_menu_surface.comparable_story_count, 0);

        assert!(
            snapshot.surfaces.len() >= 4,
            "design lab should expose at least 4 surfaces, got {}",
            snapshot.surfaces.len()
        );

        assert!(
            snapshot.surfaces.iter().any(|s| s.surface == "Footer"),
            "Footer surface should be present"
        );
        assert!(
            snapshot
                .surfaces
                .iter()
                .any(|s| s.surface == "Action Dialog"),
            "Action Dialog surface should be present"
        );
        assert!(
            snapshot.surfaces.iter().any(|s| s.surface == "Input"),
            "Input surface should be present"
        );
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

    // ─── Surface Resolution Snapshot Tests ──────────────────────────────

    #[test]
    fn resolution_snapshot_includes_all_surfaces() {
        let snapshot = build_adopted_surface_resolution_snapshot(&StorySelectionStore::default());
        assert_eq!(snapshot.surface_count, 3);
        assert_eq!(snapshot.surfaces.len(), 3);

        let story_ids: Vec<&str> = snapshot
            .surfaces
            .iter()
            .map(|s| s.story_id.as_str())
            .collect();
        assert!(story_ids.contains(&"main-menu"));
        assert!(story_ids.contains(&"mini-ai-chat-variations"));
        assert!(story_ids.contains(&"notes-window"));
    }

    #[test]
    fn resolution_snapshot_no_fallback_when_empty_store() {
        let snapshot = build_adopted_surface_resolution_snapshot(&StorySelectionStore::default());
        for entry in &snapshot.surfaces {
            assert!(
                !entry.fallback_used,
                "fallback should not be used for {} with empty store",
                entry.story_id
            );
            assert!(
                entry.requested_variant_id.is_none(),
                "requested_variant_id should be None for {} with empty store",
                entry.story_id
            );
        }
    }

    #[test]
    fn resolution_snapshot_tracks_persisted_selection() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("main-menu", "empty-state");

        let snapshot = build_adopted_surface_resolution_snapshot(&store);
        let main_menu = snapshot
            .surfaces
            .iter()
            .find(|s| s.story_id == "main-menu")
            .expect("main-menu surface should be present");

        assert_eq!(
            main_menu.requested_variant_id.as_deref(),
            Some("empty-state")
        );
        assert_eq!(main_menu.resolved_variant_id, "empty-state");
        assert!(!main_menu.fallback_used);
    }

    #[test]
    fn resolution_snapshot_detects_fallback_on_unknown_variant() {
        let mut store = StorySelectionStore::default();
        store.set_selected_variant("main-menu", "nonexistent-variant");

        let snapshot = build_adopted_surface_resolution_snapshot(&store);
        let main_menu = snapshot
            .surfaces
            .iter()
            .find(|s| s.story_id == "main-menu")
            .expect("main-menu surface should be present");

        assert_eq!(
            main_menu.requested_variant_id.as_deref(),
            Some("nonexistent-variant")
        );
        assert_eq!(main_menu.resolved_variant_id, "current-main-menu");
        assert!(main_menu.fallback_used);
    }

    #[test]
    fn resolution_snapshot_serializes_to_camel_case_json() {
        let snapshot = build_adopted_surface_resolution_snapshot(&StorySelectionStore::default());
        let json = serde_json::to_string(&snapshot).expect("serialize resolution snapshot");

        assert!(json.contains("\"selectionStorePath\""));
        assert!(json.contains("\"surfaceCount\""));
        assert!(json.contains("\"storyId\""));
        assert!(json.contains("\"resolvedVariantId\""));
        assert!(json.contains("\"fallbackUsed\""));
    }

    // ─── Runtime Fixture Completeness Tests ─────────────────────────────

    #[test]
    fn catalog_main_menu_variants_report_live_surface_representation() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());
        let main_menu = snapshot
            .stories
            .iter()
            .find(|s| s.story_id == "main-menu")
            .expect("main-menu story should exist");

        for variant in &main_menu.variants {
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("liveSurface"),
                "main-menu variant {} should have representation=liveSurface",
                variant.id
            );
            assert!(
                !variant.props.contains_key("fixtureImagePresent"),
                "main-menu variant {} should not have fixtureImagePresent prop",
                variant.id
            );
            assert!(
                !variant.props.contains_key("fixtureManifestPresent"),
                "main-menu variant {} should not have fixtureManifestPresent prop",
                variant.id
            );
        }
    }

    #[test]
    fn catalog_notes_window_variants_have_fixture_presence_props() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());
        let notes = snapshot
            .stories
            .iter()
            .find(|s| s.story_id == "notes-window")
            .expect("notes-window story should exist");

        for variant in &notes.variants {
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("runtimeFixture"),
                "notes-window variant {} should have representation=runtimeFixture",
                variant.id
            );
            assert!(
                variant.props.contains_key("fixtureImagePresent"),
                "notes-window variant {} should have fixtureImagePresent prop",
                variant.id
            );
            assert!(
                variant.props.contains_key("fixtureManifestPresent"),
                "notes-window variant {} should have fixtureManifestPresent prop",
                variant.id
            );
        }
    }

    #[test]
    fn catalog_mini_ai_chat_variants_have_fixture_presence_props() {
        let snapshot = build_story_catalog_snapshot(&StorySelectionStore::default());
        let mini_ai_chat = snapshot
            .stories
            .iter()
            .find(|s| s.story_id == "mini-ai-chat-variations")
            .expect("mini-ai-chat-variations story should exist");

        for variant in &mini_ai_chat.variants {
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("runtimeFixture"),
                "mini-ai-chat variant {} should have representation=runtimeFixture",
                variant.id
            );
            assert!(
                variant.props.contains_key("fixtureImagePresent"),
                "mini-ai-chat variant {} should have fixtureImagePresent prop",
                variant.id
            );
            assert!(
                variant.props.contains_key("fixtureManifestPresent"),
                "mini-ai-chat variant {} should have fixtureManifestPresent prop",
                variant.id
            );
        }
    }
}
