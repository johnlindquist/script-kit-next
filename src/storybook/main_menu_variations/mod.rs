//! Main menu variation system for storybook.
//!
//! This module follows the same adoption contract as `notes_window_variations`:
//! typed `VariationId` → `AdoptableSurface` → `resolve_surface_live` → live id.
//! Main Menu previews in Storybook are rendered via the shared runtime-fixture host.

use super::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
use super::runtime_fixture;
use super::StoryVariant;

/// Stable IDs for adoptable Main Menu visual states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainMenuVariationId {
    CurrentMainMenu,
    EmptyState,
    SelectedResult,
}

impl MainMenuVariationId {
    pub const ALL: [Self; 3] = [
        Self::CurrentMainMenu,
        Self::EmptyState,
        Self::SelectedResult,
    ];
}

impl VariationId for MainMenuVariationId {
    fn as_str(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "current-main-menu",
            Self::EmptyState => "empty-state",
            Self::SelectedResult => "selected-result",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "Current Main Menu",
            Self::EmptyState => "Empty State",
            Self::SelectedResult => "Selected Result",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "Real launcher with populated search results",
            Self::EmptyState => "Real launcher chrome with no matching results",
            Self::SelectedResult => "Real launcher with a keyboard-focused result row",
        }
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current-main-menu" => Some(Self::CurrentMainMenu),
            "empty-state" => Some(Self::EmptyState),
            "selected-result" => Some(Self::SelectedResult),
            _ => None,
        }
    }
}

pub const SPECS: [MainMenuVariationId; 3] = MainMenuVariationId::ALL;

pub struct MainMenuSurface;

impl AdoptableSurface for MainMenuSurface {
    type Id = MainMenuVariationId;
    type Spec = MainMenuVariationId;
    type Live = MainMenuVariationId;

    const STORY_ID: &'static str = "main-menu";
    const DEFAULT_ID: Self::Id = MainMenuVariationId::CurrentMainMenu;

    fn specs() -> &'static [Self::Spec] {
        &SPECS
    }

    fn spec_id(spec: &Self::Spec) -> Self::Id {
        *spec
    }

    fn live_from_spec(spec: &Self::Spec) -> Self::Live {
        *spec
    }
}

pub fn main_menu_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .copied()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "mainMenu")
                .with_prop("representation", "runtimeFixture")
                .with_prop("variantId", id.as_str())
        })
        .collect()
}

pub fn resolve_main_menu_variant(
    selected: Option<&str>,
) -> (MainMenuVariationId, SurfaceSelectionResolution) {
    resolve_surface_live::<MainMenuSurface>(selected)
}

pub fn adopted_main_menu_variant() -> MainMenuVariationId {
    adopted_surface_live::<MainMenuSurface>()
}

/// Render a Main Menu storybook preview using the runtime-fixture host.
pub fn render_main_menu_story_preview(stable_id: &str) -> gpui::AnyElement {
    runtime_fixture::render_runtime_fixture("main-menu", stable_id, false)
}

/// Render a Main Menu compare-mode thumbnail via the runtime-fixture host.
pub fn render_main_menu_compare_thumbnail(stable_id: &str) -> gpui::AnyElement {
    runtime_fixture::render_runtime_fixture("main-menu", stable_id, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variation_ids_have_stable_roundtrip() {
        for id in MainMenuVariationId::ALL {
            let parsed = MainMenuVariationId::from_stable_id(id.as_str());
            assert_eq!(parsed, Some(id), "roundtrip failed for {:?}", id);
        }
    }

    #[test]
    fn specs_match_variation_count() {
        assert_eq!(SPECS.len(), MainMenuVariationId::ALL.len());
    }

    #[test]
    fn story_variants_generated_for_all_specs() {
        let variants = main_menu_story_variants();
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0].stable_id(), "current-main-menu");
        assert_eq!(variants[1].stable_id(), "empty-state");
        assert_eq!(variants[2].stable_id(), "selected-result");
    }

    #[test]
    fn resolve_unknown_variant_falls_back_to_current() {
        let (id, resolution) = resolve_main_menu_variant(Some("nonexistent"));
        assert_eq!(id, MainMenuVariationId::CurrentMainMenu);
        assert!(resolution.fallback_used);
    }

    #[test]
    fn resolve_none_returns_current() {
        let (id, resolution) = resolve_main_menu_variant(None);
        assert_eq!(id, MainMenuVariationId::CurrentMainMenu);
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn adoptable_surface_story_id_matches() {
        assert_eq!(MainMenuSurface::STORY_ID, "main-menu");
    }
}
