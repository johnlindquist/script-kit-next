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

/// Typed live-spec describing how the launcher should render for a given Main Menu variant.
///
/// These fields are consumed at render time via read-only local overrides — they must
/// never cause state mutation inside `render_script_list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainMenuLiveSpec {
    /// When `true`, the list renders as empty regardless of actual script inventory.
    pub force_empty_results: bool,
    /// When `true`, the first real item (not a section header) gets keyboard focus.
    pub prefer_first_result_selected: bool,
    /// When set, overrides the filter text displayed in the empty-state body.
    pub filter_text_override: Option<&'static str>,
}

/// A Main Menu variation paired with its live-spec for adoption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainMenuVariationSpec {
    pub id: MainMenuVariationId,
    pub live: MainMenuLiveSpec,
}

pub const SPECS: [MainMenuVariationSpec; 3] = [
    MainMenuVariationSpec {
        id: MainMenuVariationId::CurrentMainMenu,
        live: MainMenuLiveSpec {
            force_empty_results: false,
            prefer_first_result_selected: false,
            filter_text_override: None,
        },
    },
    MainMenuVariationSpec {
        id: MainMenuVariationId::EmptyState,
        live: MainMenuLiveSpec {
            force_empty_results: true,
            prefer_first_result_selected: false,
            filter_text_override: Some("storybook-empty"),
        },
    },
    MainMenuVariationSpec {
        id: MainMenuVariationId::SelectedResult,
        live: MainMenuLiveSpec {
            force_empty_results: false,
            prefer_first_result_selected: true,
            filter_text_override: None,
        },
    },
];

pub struct MainMenuSurface;

impl AdoptableSurface for MainMenuSurface {
    type Id = MainMenuVariationId;
    type Spec = MainMenuVariationSpec;
    type Live = MainMenuLiveSpec;

    const STORY_ID: &'static str = "main-menu";
    const DEFAULT_ID: Self::Id = MainMenuVariationId::CurrentMainMenu;

    fn specs() -> &'static [Self::Spec] {
        &SPECS
    }

    fn spec_id(spec: &Self::Spec) -> Self::Id {
        spec.id
    }

    fn live_from_spec(spec: &Self::Spec) -> Self::Live {
        spec.live
    }
}

pub fn main_menu_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "mainMenu")
                .with_prop("representation", "runtimeFixture")
                .with_prop("variantId", spec.id.as_str())
                .with_prop(
                    "forceEmptyResults",
                    spec.live.force_empty_results.to_string(),
                )
                .with_prop(
                    "preferFirstResultSelected",
                    spec.live.prefer_first_result_selected.to_string(),
                )
        })
        .collect()
}

pub fn resolve_main_menu_variant(
    selected: Option<&str>,
) -> (MainMenuLiveSpec, SurfaceSelectionResolution) {
    resolve_surface_live::<MainMenuSurface>(selected)
}

pub fn adopted_main_menu_variant() -> MainMenuVariationId {
    let selected = super::load_selected_story_variant(MainMenuSurface::STORY_ID);
    let (_, resolution) = resolve_surface_live::<MainMenuSurface>(selected.as_deref());
    MainMenuVariationId::from_stable_id(&resolution.resolved_variant_id)
        .unwrap_or(MainMenuSurface::DEFAULT_ID)
}

/// Resolve the current on-disk storybook selection into a typed `MainMenuLiveSpec`.
pub fn adopted_main_menu_live_spec() -> MainMenuLiveSpec {
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
        let (live, resolution) = resolve_main_menu_variant(Some("nonexistent"));
        // Default (current-main-menu) has no overrides
        assert!(!live.force_empty_results);
        assert!(!live.prefer_first_result_selected);
        assert!(resolution.fallback_used);
    }

    #[test]
    fn resolve_none_returns_current() {
        let (live, resolution) = resolve_main_menu_variant(None);
        assert!(!live.force_empty_results);
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_empty_state_returns_force_empty() {
        let (live, resolution) = resolve_main_menu_variant(Some("empty-state"));
        assert!(live.force_empty_results);
        assert!(!live.prefer_first_result_selected);
        assert_eq!(live.filter_text_override, Some("storybook-empty"));
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_selected_result_returns_prefer_first() {
        let (live, resolution) = resolve_main_menu_variant(Some("selected-result"));
        assert!(!live.force_empty_results);
        assert!(live.prefer_first_result_selected);
        assert_eq!(live.filter_text_override, None);
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn adoptable_surface_story_id_matches() {
        assert_eq!(MainMenuSurface::STORY_ID, "main-menu");
    }

    #[test]
    fn specs_have_correct_live_values() {
        for spec in &SPECS {
            let live = MainMenuSurface::live_from_spec(spec);
            assert_eq!(live, spec.live, "live_from_spec mismatch for {:?}", spec.id);
        }
    }
}
