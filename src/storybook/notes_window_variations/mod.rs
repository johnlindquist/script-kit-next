//! Storybook variation system for the Notes window surface.
//!
//! This module follows the same adoption contract as `actions_dialog_variations`:
//! typed `VariationId` → `AdoptableSurface` → `resolve_surface_live` → live style.
//! Registered visual coverage lives in `notes-window-states`, which uses a
//! deterministic non-PNG presenter for the Notes window's canonical states.

use super::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
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

pub fn resolve_notes_window_style(
    selected: Option<&str>,
) -> (NotesWindowStyle, SurfaceSelectionResolution) {
    resolve_surface_live::<NotesWindowSurface>(selected)
}

pub fn adopted_notes_window_style() -> NotesWindowStyle {
    adopted_surface_live::<NotesWindowSurface>()
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
    fn specs_preserve_expected_order() {
        let ids: Vec<_> = SPECS.iter().map(|spec| spec.id.as_str()).collect();
        assert_eq!(ids, vec!["current", "compact", "airy"]);
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
