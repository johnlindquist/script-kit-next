//! Storybook variation system for the Notes window surface.
//!
//! This module follows the same adoption contract as `actions_dialog_variations`:
//! typed `VariationId` → `AdoptableSurface` → `resolve_surface_live` → live style.
//! Notes previews in Storybook are rendered via the shared runtime-fixture host.

use super::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
use super::runtime_fixture;
use super::StoryVariant;
use crate::notes::window::style::NotesWindowStyle;

/// Stable IDs for adoptable Notes window visual styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotesWindowVariationId {
    Current,
    Compact,
    Airy,
}

impl NotesWindowVariationId {
    pub const ALL: [Self; 3] = [Self::Current, Self::Compact, Self::Airy];
}

impl VariationId for NotesWindowVariationId {
    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Compact => "compact",
            Self::Airy => "airy",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Compact => "Compact",
            Self::Airy => "Airy",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Current => "Matches the current live Notes window defaults",
            Self::Compact => "Tighter spacing for smaller windows",
            Self::Airy => "More breathing room, relaxed layout",
        }
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current" => Some(Self::Current),
            "compact" => Some(Self::Compact),
            "airy" => Some(Self::Airy),
            _ => None,
        }
    }
}

/// Declarative registry entry for a Notes window style variation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotesWindowVariationSpec {
    pub id: NotesWindowVariationId,
    pub style: NotesWindowStyle,
}

pub const SPECS: [NotesWindowVariationSpec; 3] = [
    NotesWindowVariationSpec {
        id: NotesWindowVariationId::Current,
        style: NotesWindowStyle {
            titlebar_height: 36.0,
            footer_height: 28.0,
            editor_padding_x: 16.0,
            editor_padding_y: 12.0,
            chrome_opacity: 1.0,
        },
    },
    NotesWindowVariationSpec {
        id: NotesWindowVariationId::Compact,
        style: NotesWindowStyle {
            titlebar_height: 28.0,
            footer_height: 22.0,
            editor_padding_x: 8.0,
            editor_padding_y: 6.0,
            chrome_opacity: 1.0,
        },
    },
    NotesWindowVariationSpec {
        id: NotesWindowVariationId::Airy,
        style: NotesWindowStyle {
            titlebar_height: 44.0,
            footer_height: 32.0,
            editor_padding_x: 24.0,
            editor_padding_y: 16.0,
            chrome_opacity: 1.0,
        },
    },
];

pub struct NotesWindowSurface;

impl AdoptableSurface for NotesWindowSurface {
    type Id = NotesWindowVariationId;
    type Spec = NotesWindowVariationSpec;
    type Live = NotesWindowStyle;

    const STORY_ID: &'static str = "notes-window";
    const DEFAULT_ID: Self::Id = NotesWindowVariationId::Current;

    fn specs() -> &'static [Self::Spec] {
        &SPECS
    }

    fn spec_id(spec: &Self::Spec) -> Self::Id {
        spec.id
    }

    fn live_from_spec(spec: &Self::Spec) -> Self::Live {
        spec.style
    }
}

pub fn notes_window_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "notesWindow")
                .with_prop("representation", "runtimeFixture")
                .with_prop("variantId", spec.id.as_str())
                .with_prop(
                    "titlebarHeight",
                    format!("{:.0}", spec.style.titlebar_height),
                )
                .with_prop("footerHeight", format!("{:.0}", spec.style.footer_height))
                .with_prop(
                    "editorPaddingX",
                    format!("{:.0}", spec.style.editor_padding_x),
                )
        })
        .collect()
}

pub fn resolve_notes_window_style(
    selected: Option<&str>,
) -> (NotesWindowStyle, SurfaceSelectionResolution) {
    resolve_surface_live::<NotesWindowSurface>(selected)
}

pub fn adopted_notes_window_style() -> NotesWindowStyle {
    adopted_surface_live::<NotesWindowSurface>()
}

/// Render a Notes window storybook preview using the runtime-fixture host.
pub fn render_notes_window_story_preview(stable_id: &str) -> gpui::AnyElement {
    runtime_fixture::render_runtime_fixture("notes-window", stable_id, false)
}

/// Render a Notes window compare-mode thumbnail via the runtime-fixture host.
pub fn render_notes_window_compare_thumbnail(stable_id: &str) -> gpui::AnyElement {
    runtime_fixture::render_runtime_fixture("notes-window", stable_id, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variation_ids_have_stable_roundtrip() {
        for id in NotesWindowVariationId::ALL {
            let parsed = NotesWindowVariationId::from_stable_id(id.as_str());
            assert_eq!(parsed, Some(id), "roundtrip failed for {:?}", id);
        }
    }

    #[test]
    fn specs_match_variation_count() {
        assert_eq!(SPECS.len(), NotesWindowVariationId::ALL.len());
    }

    #[test]
    fn current_spec_matches_style_current() {
        let spec = &SPECS[0];
        assert_eq!(spec.id, NotesWindowVariationId::Current);
        assert_eq!(spec.style, NotesWindowStyle::current());
    }

    #[test]
    fn story_variants_generated_for_all_specs() {
        let variants = notes_window_story_variants();
        assert_eq!(variants.len(), SPECS.len());
        assert_eq!(variants[0].stable_id(), "current");
        assert_eq!(variants[1].stable_id(), "compact");
        assert_eq!(variants[2].stable_id(), "airy");
    }

    #[test]
    fn resolve_unknown_variant_falls_back_to_current() {
        let (style, resolution) = resolve_notes_window_style(Some("nonexistent"));
        assert_eq!(style, NotesWindowStyle::current());
        assert!(resolution.fallback_used);
    }

    #[test]
    fn resolve_none_returns_current() {
        let (style, resolution) = resolve_notes_window_style(None);
        assert_eq!(style, NotesWindowStyle::current());
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_compact_returns_compact_style() {
        let (style, resolution) = resolve_notes_window_style(Some("compact"));
        assert_eq!(style, NotesWindowStyle::compact());
        assert!(!resolution.fallback_used);
        assert_eq!(resolution.resolved_variant_id, "compact");
    }

    #[test]
    fn adoptable_surface_story_id_matches() {
        assert_eq!(NotesWindowSurface::STORY_ID, "notes-window");
    }
}
