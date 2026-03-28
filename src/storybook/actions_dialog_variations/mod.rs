use super::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
use super::StoryVariant;

// Seeded from src/actions/constants.rs. Keep these in sync with the live dialog defaults.
const CURRENT_ROW_HEIGHT: f32 = 30.0;
const CURRENT_SEARCH_INPUT_HEIGHT: f32 = 36.0;
const CURRENT_ROW_RADIUS: f32 = 6.0;

/// Stable IDs for adoptable Actions dialog visual styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionsDialogVariationId {
    Current,
    Whisper,
    GhostPills,
    Typewriter,
    SingleColumn,
    InlineKeys,
    SearchFocused,
    DotAccent,
}

impl ActionsDialogVariationId {
    pub const ALL: [Self; 8] = [
        Self::Current,
        Self::Whisper,
        Self::GhostPills,
        Self::Typewriter,
        Self::SingleColumn,
        Self::InlineKeys,
        Self::SearchFocused,
        Self::DotAccent,
    ];
}

impl VariationId for ActionsDialogVariationId {
    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Whisper => "whisper",
            Self::GhostPills => "ghost-pills",
            Self::Typewriter => "typewriter",
            Self::SingleColumn => "single-column",
            Self::InlineKeys => "inline-keys",
            Self::SearchFocused => "search-focused",
            Self::DotAccent => "dot-accent",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Whisper => "Whisper",
            Self::GhostPills => "Ghost Pills",
            Self::Typewriter => "Typewriter",
            Self::SingleColumn => "Single Column",
            Self::InlineKeys => "Inline Keys",
            Self::SearchFocused => "Search Focused",
            Self::DotAccent => "Dot Accent",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Current => "Matches the current live actions dialog defaults",
            Self::Whisper => "Ultra-quiet chrome with barely-there selection and no header",
            Self::GhostPills => "Rounded pill rows with stronger selection chrome",
            Self::Typewriter => "Monospace prompt treatment with a terminal-style prefix",
            Self::SingleColumn => "Label-only rows with icons and shortcuts removed",
            Self::InlineKeys => "Shortcut hints rendered inline instead of keycaps",
            Self::SearchFocused => "Search divider emphasized to anchor the input row",
            Self::DotAccent => "Selection background removed in favor of a small leading dot",
        }
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current" => Some(Self::Current),
            "whisper" => Some(Self::Whisper),
            "ghost-pills" => Some(Self::GhostPills),
            "typewriter" => Some(Self::Typewriter),
            "single-column" => Some(Self::SingleColumn),
            "inline-keys" => Some(Self::InlineKeys),
            "search-focused" => Some(Self::SearchFocused),
            "dot-accent" => Some(Self::DotAccent),
            _ => None,
        }
    }
}

/// Typed live style consumed by both storybook previews and the real dialog renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsDialogStyle {
    pub show_container_border: bool,
    pub show_header: bool,
    pub show_search_divider: bool,
    pub show_icons: bool,
    pub selection_opacity: f32,
    pub hover_opacity: f32,
    pub row_height: f32,
    pub row_radius: f32,
    pub shortcut_visible: bool,
    pub mono_font: bool,
    pub prefix_marker: Option<&'static str>,
}

/// Declarative registry entry for an actions dialog style variation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsDialogVariationSpec {
    pub id: ActionsDialogVariationId,
    pub style: ActionsDialogStyle,
}

pub const SPECS: [ActionsDialogVariationSpec; 8] = [
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::Current,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: false,
            show_icons: false,
            selection_opacity: 1.0,
            hover_opacity: 1.0,
            row_height: CURRENT_ROW_HEIGHT,
            row_radius: CURRENT_ROW_RADIUS,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::Whisper,
        style: ActionsDialogStyle {
            show_container_border: false,
            show_header: false,
            show_search_divider: false,
            show_icons: false,
            selection_opacity: 0.04,
            hover_opacity: 0.03,
            row_height: CURRENT_ROW_HEIGHT,
            row_radius: 6.0,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::GhostPills,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: false,
            show_icons: true,
            selection_opacity: 0.12,
            hover_opacity: 0.08,
            row_height: CURRENT_ROW_HEIGHT + 2.0,
            row_radius: 16.0,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::Typewriter,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: true,
            show_icons: false,
            selection_opacity: 0.06,
            hover_opacity: 0.04,
            row_height: CURRENT_ROW_HEIGHT - 2.0,
            row_radius: 0.0,
            shortcut_visible: true,
            mono_font: true,
            prefix_marker: Some(">"),
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::SingleColumn,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: false,
            show_icons: false,
            selection_opacity: 0.10,
            hover_opacity: 0.06,
            row_height: CURRENT_ROW_HEIGHT,
            row_radius: 8.0,
            shortcut_visible: false,
            mono_font: false,
            prefix_marker: None,
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::InlineKeys,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: false,
            show_icons: false,
            selection_opacity: 0.08,
            hover_opacity: 0.05,
            row_height: CURRENT_ROW_HEIGHT,
            row_radius: CURRENT_ROW_RADIUS,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::SearchFocused,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: true,
            show_icons: true,
            selection_opacity: 0.08,
            hover_opacity: 0.05,
            row_height: CURRENT_ROW_HEIGHT - 2.0,
            row_radius: CURRENT_ROW_RADIUS,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        },
    },
    ActionsDialogVariationSpec {
        id: ActionsDialogVariationId::DotAccent,
        style: ActionsDialogStyle {
            show_container_border: true,
            show_header: true,
            show_search_divider: false,
            show_icons: false,
            selection_opacity: 0.0,
            hover_opacity: 0.03,
            row_height: CURRENT_ROW_HEIGHT,
            row_radius: CURRENT_ROW_RADIUS,
            shortcut_visible: true,
            mono_font: false,
            prefix_marker: None,
        },
    },
];

pub struct ActionsDialogSurface;

impl AdoptableSurface for ActionsDialogSurface {
    type Id = ActionsDialogVariationId;
    type Spec = ActionsDialogVariationSpec;
    type Live = ActionsDialogStyle;

    const STORY_ID: &'static str = "actions-mini-variations";
    const DEFAULT_ID: Self::Id = ActionsDialogVariationId::Current;

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

pub fn actions_dialog_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "actionsDialog")
                .with_prop("variantId", spec.id.as_str())
                .with_prop(
                    "showContainerBorder",
                    if spec.style.show_container_border {
                        "true"
                    } else {
                        "false"
                    },
                )
                .with_prop(
                    "showHeader",
                    if spec.style.show_header {
                        "true"
                    } else {
                        "false"
                    },
                )
                .with_prop(
                    "showSearchDivider",
                    if spec.style.show_search_divider {
                        "true"
                    } else {
                        "false"
                    },
                )
                .with_prop(
                    "rowHeight",
                    format!(
                        "{:.0}",
                        spec.style.row_height.max(CURRENT_SEARCH_INPUT_HEIGHT - 6.0)
                    ),
                )
        })
        .collect()
}

pub fn resolve_actions_dialog_style(
    selected: Option<&str>,
) -> (ActionsDialogStyle, SurfaceSelectionResolution) {
    resolve_surface_live::<ActionsDialogSurface>(selected)
}

pub fn adopted_actions_dialog_style() -> ActionsDialogStyle {
    adopted_surface_live::<ActionsDialogSurface>()
}

pub fn actions_dialog_style_uses_inline_shortcuts(style: &ActionsDialogStyle) -> bool {
    *style == SPECS[5].style
}

#[cfg(test)]
mod tests;
