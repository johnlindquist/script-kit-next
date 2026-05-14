//! Context-picker popup redesign gallery for `@` mentions and slash commands.
//!
//! The real runtime dropdown already shares the dense monoline row and
//! `InlineDropdown` shell. This playground explores seven focused design
//! directions per trigger while staying inside the existing typography,
//! opacity, and whisper-chrome rules from `.impeccable.md`.

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::scroll::ScrollableElement as _;

use crate::ai::acp::picker_popup::acp_context_picker_item_to_inline_picker_row;
use crate::ai::window::context_picker::types::{
    ContextPickerItem, ContextPickerItemKind, PortalKind, SlashCommandPayload,
};
use crate::components::inline_dropdown::{
    render_soft_compact_picker_row, InlineDropdown, InlineDropdownColors, InlineDropdownEmptyState,
    InlineDropdownSynopsis, GHOST, HINT, MUTED_OP, SOFT_COMPACT_PICKER_ROW_HEIGHT,
};
use crate::components::inline_picker::{inline_picker_normalize_selected_index, InlinePickerRow};
use crate::components::prompt_footer::{PromptFooter, PromptFooterColors};
use crate::list_item::FONT_MONO;
use crate::storybook::{
    config_from_storybook_footer_selection_value,
    playground_overlay_metrics::context_picker_playground_overlay_metrics, story_container,
    FooterVariationId, IntegratedSurfaceShell, IntegratedSurfaceShellConfig, StoryVariant,
};
use crate::theme::{get_cached_theme, AppChromeColors};
use crate::ui_foundation::HexColorExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextPickerPopupPlaygroundId {
    MentionWhisperDense,
    MentionGroupedCatalog,
    MentionLeadingVisuals,
    MentionAccessoryBadges,
    MentionFlatCompact,
    MentionSynopsisRail,
    MentionEmptyState,
    SlashWhisperDense,
    SlashGroupedCatalog,
    SlashLeadingVisuals,
    SlashAccessoryBadges,
    SlashFlatCompact,
    SlashSynopsisRail,
    SlashEmptyState,
}

impl ContextPickerPopupPlaygroundId {
    pub const ALL: [Self; 14] = [
        Self::MentionWhisperDense,
        Self::MentionGroupedCatalog,
        Self::MentionLeadingVisuals,
        Self::MentionAccessoryBadges,
        Self::MentionFlatCompact,
        Self::MentionSynopsisRail,
        Self::MentionEmptyState,
        Self::SlashWhisperDense,
        Self::SlashGroupedCatalog,
        Self::SlashLeadingVisuals,
        Self::SlashAccessoryBadges,
        Self::SlashFlatCompact,
        Self::SlashSynopsisRail,
        Self::SlashEmptyState,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MentionWhisperDense => "mention-whisper-dense",
            Self::MentionGroupedCatalog => "mention-grouped-catalog",
            Self::MentionLeadingVisuals => "mention-leading-visuals",
            Self::MentionAccessoryBadges => "mention-accessory-badges",
            Self::MentionFlatCompact => "mention-flat-compact",
            Self::MentionSynopsisRail => "mention-synopsis-rail",
            Self::MentionEmptyState => "mention-empty-state",
            Self::SlashWhisperDense => "slash-whisper-dense",
            Self::SlashGroupedCatalog => "slash-grouped-catalog",
            Self::SlashLeadingVisuals => "slash-leading-visuals",
            Self::SlashAccessoryBadges => "slash-accessory-badges",
            Self::SlashFlatCompact => "slash-flat-compact",
            Self::SlashSynopsisRail => "slash-synopsis-rail",
            Self::SlashEmptyState => "slash-empty-state",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MentionWhisperDense => "Whisper Dense",
            Self::MentionGroupedCatalog => "Grouped Catalog",
            Self::MentionLeadingVisuals => "Leading Visuals",
            Self::MentionAccessoryBadges => "Accessory Badges",
            Self::MentionFlatCompact => "Flat Compact",
            Self::MentionSynopsisRail => "Synopsis Rail",
            Self::MentionEmptyState => "Empty State",
            Self::SlashWhisperDense => "Whisper Dense",
            Self::SlashGroupedCatalog => "Grouped Catalog",
            Self::SlashLeadingVisuals => "Leading Visuals",
            Self::SlashAccessoryBadges => "Accessory Badges",
            Self::SlashFlatCompact => "Flat Compact",
            Self::SlashSynopsisRail => "Synopsis Rail",
            Self::SlashEmptyState => "Empty State",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::MentionWhisperDense => {
                "Baseline whisper-chrome mention list with synopsis and highlight-first search."
            }
            Self::MentionGroupedCatalog => {
                "Grouped mention catalog with uppercase sections and quieter synopsis."
            }
            Self::MentionLeadingVisuals => {
                "Mention list with restrained leading markers for faster scanning."
            }
            Self::MentionAccessoryBadges => {
                "Mention rows with subtle source badges instead of repeated right-side tokens."
            }
            Self::MentionFlatCompact => {
                "Flatter, tighter mention popup that leans into footer-aligned density."
            }
            Self::MentionSynopsisRail => {
                "Mention picker with a stronger bottom synopsis rail for focused context."
            }
            Self::MentionEmptyState => {
                "No-match mention state with hint chips that keep discovery lightweight."
            }
            Self::SlashWhisperDense => {
                "Baseline slash-command list with mono command text and synopsis."
            }
            Self::SlashGroupedCatalog => {
                "Grouped slash picker for built-ins, Claude commands, and plugin skills."
            }
            Self::SlashLeadingVisuals => {
                "Slash rows with leading source markers that stay inside the existing type scale."
            }
            Self::SlashAccessoryBadges => {
                "Slash rows with quiet owner badges for Core, Claude, and plugin discovery."
            }
            Self::SlashFlatCompact => {
                "Tighter slash popup with flatter chrome and reduced synopsis weight."
            }
            Self::SlashSynopsisRail => {
                "Slash picker with a stronger synopsis rail for command learning."
            }
            Self::SlashEmptyState => {
                "No-match slash state with hint chips for built-in command discovery."
            }
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "mention-whisper-dense" => Some(Self::MentionWhisperDense),
            "mention-grouped-catalog" => Some(Self::MentionGroupedCatalog),
            "mention-leading-visuals" => Some(Self::MentionLeadingVisuals),
            "mention-accessory-badges" => Some(Self::MentionAccessoryBadges),
            "mention-flat-compact" => Some(Self::MentionFlatCompact),
            "mention-synopsis-rail" => Some(Self::MentionSynopsisRail),
            "mention-empty-state" => Some(Self::MentionEmptyState),
            "slash-whisper-dense" => Some(Self::SlashWhisperDense),
            "slash-grouped-catalog" => Some(Self::SlashGroupedCatalog),
            "slash-leading-visuals" => Some(Self::SlashLeadingVisuals),
            "slash-accessory-badges" => Some(Self::SlashAccessoryBadges),
            "slash-flat-compact" => Some(Self::SlashFlatCompact),
            "slash-synopsis-rail" => Some(Self::SlashSynopsisRail),
            "slash-empty-state" => Some(Self::SlashEmptyState),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPickerPopupTrigger {
    Mention,
    Slash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPickerPopupSceneState {
    Results,
    Loading,
    Empty,
    Error,
}

impl ContextPickerPopupSceneState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Results => "results",
            Self::Loading => "loading",
            Self::Empty => "empty",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextPickerPopupStyle {
    WhisperDense,
    GroupedCatalog,
    LeadingVisuals,
    AccessoryBadges,
    FlatCompact,
    SynopsisRail,
    EmptyState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContextPickerPopupPlaygroundSpec {
    id: ContextPickerPopupPlaygroundId,
    trigger: ContextPickerPopupTrigger,
    style: ContextPickerPopupStyle,
    query: &'static str,
    selected_row_id: &'static str,
    note: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct PickerRow {
    id: &'static str,
    label: &'static str,
    meta: &'static str,
    description: &'static str,
    section: &'static str,
    accessory: &'static str,
    #[allow(dead_code)]
    leading_mark: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlashPickerTypographyVariantId {
    LauncherRegular,
    LauncherMediumSelected,
    SoftCompact,
    MonoCommand,
    QuietMetadata,
}

impl SlashPickerTypographyVariantId {
    pub const ALL: [Self; 5] = [
        Self::LauncherRegular,
        Self::LauncherMediumSelected,
        Self::SoftCompact,
        Self::MonoCommand,
        Self::QuietMetadata,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LauncherRegular => "launcher-regular",
            Self::LauncherMediumSelected => "launcher-medium-selected",
            Self::SoftCompact => "soft-compact",
            Self::MonoCommand => "mono-command",
            Self::QuietMetadata => "quiet-metadata",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::LauncherRegular => "Launcher Regular",
            Self::LauncherMediumSelected => "Medium Selected",
            Self::SoftCompact => "Soft Compact",
            Self::MonoCommand => "Mono Command",
            Self::QuietMetadata => "Quiet Metadata",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::LauncherRegular => {
                "40px rows, 14px regular labels, source metadata as flat text."
            }
            Self::LauncherMediumSelected => {
                "40px rows with only the selected command bumped to medium weight."
            }
            Self::SoftCompact => "36px rows, 13px regular labels, lighter selected fill.",
            Self::MonoCommand => "40px rows with slash commands in mono at regular weight.",
            Self::QuietMetadata => {
                "40px rows, regular labels, and source metadata without a badge fill."
            }
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "launcher-regular" => Some(Self::LauncherRegular),
            "launcher-medium-selected" => Some(Self::LauncherMediumSelected),
            "soft-compact" => Some(Self::SoftCompact),
            "mono-command" => Some(Self::MonoCommand),
            "quiet-metadata" => Some(Self::QuietMetadata),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SlashTypographySpec {
    id: SlashPickerTypographyVariantId,
    row_height: f32,
    label_size: f32,
    label_line_height: f32,
    meta_size: f32,
    selected_fill: f32,
    selected_weight: FontWeight,
    unselected_weight: FontWeight,
    label_font: SlashTypographyFont,
    metadata_style: SlashTypographyMetadataStyle,
}

#[derive(Debug, Clone, Copy)]
enum SlashTypographyFont {
    System,
    Mono,
}

#[derive(Debug, Clone, Copy)]
enum SlashTypographyMetadataStyle {
    FlatText,
    SoftBadge,
    BareText,
}

const SLASH_TYPOGRAPHY_SPECS: [SlashTypographySpec; 5] = [
    SlashTypographySpec {
        id: SlashPickerTypographyVariantId::LauncherRegular,
        row_height: 40.0,
        label_size: 14.0,
        label_line_height: 20.0,
        meta_size: 11.0,
        selected_fill: 0.20,
        selected_weight: FontWeight::NORMAL,
        unselected_weight: FontWeight::NORMAL,
        label_font: SlashTypographyFont::System,
        metadata_style: SlashTypographyMetadataStyle::FlatText,
    },
    SlashTypographySpec {
        id: SlashPickerTypographyVariantId::LauncherMediumSelected,
        row_height: 40.0,
        label_size: 14.0,
        label_line_height: 20.0,
        meta_size: 11.0,
        selected_fill: 0.23,
        selected_weight: FontWeight::MEDIUM,
        unselected_weight: FontWeight::NORMAL,
        label_font: SlashTypographyFont::System,
        metadata_style: SlashTypographyMetadataStyle::SoftBadge,
    },
    SlashTypographySpec {
        id: SlashPickerTypographyVariantId::SoftCompact,
        row_height: 36.0,
        label_size: 13.0,
        label_line_height: 18.0,
        meta_size: 10.5,
        selected_fill: 0.18,
        selected_weight: FontWeight::NORMAL,
        unselected_weight: FontWeight::NORMAL,
        label_font: SlashTypographyFont::System,
        metadata_style: SlashTypographyMetadataStyle::SoftBadge,
    },
    SlashTypographySpec {
        id: SlashPickerTypographyVariantId::MonoCommand,
        row_height: 40.0,
        label_size: 13.5,
        label_line_height: 20.0,
        meta_size: 11.0,
        selected_fill: 0.20,
        selected_weight: FontWeight::NORMAL,
        unselected_weight: FontWeight::NORMAL,
        label_font: SlashTypographyFont::Mono,
        metadata_style: SlashTypographyMetadataStyle::FlatText,
    },
    SlashTypographySpec {
        id: SlashPickerTypographyVariantId::QuietMetadata,
        row_height: 40.0,
        label_size: 14.0,
        label_line_height: 20.0,
        meta_size: 11.0,
        selected_fill: 0.20,
        selected_weight: FontWeight::NORMAL,
        unselected_weight: FontWeight::NORMAL,
        label_font: SlashTypographyFont::System,
        metadata_style: SlashTypographyMetadataStyle::BareText,
    },
];

const MENTION_ROWS: [PickerRow; 6] = [
    PickerRow {
        id: "mention-screenshot",
        label: "Screenshot",
        meta: "@screenshot",
        description: "Attach the most recent screenshot to ground the next reply.",
        section: "Desktop",
        accessory: "Desktop",
        leading_mark: "SC",
    },
    PickerRow {
        id: "mention-selection",
        label: "Selection",
        meta: "@selection",
        description: "Use the currently selected text from the frontmost app.",
        section: "Desktop",
        accessory: "Selection",
        leading_mark: "SE",
    },
    PickerRow {
        id: "mention-browser",
        label: "Browser URL",
        meta: "@browser",
        description: "Attach the active browser location without leaving the keyboard.",
        section: "Desktop",
        accessory: "Browser",
        leading_mark: "BR",
    },
    PickerRow {
        id: "mention-clipboard",
        label: "Clipboard",
        meta: "@clipboard",
        description: "Bring the latest clipboard contents into the composer.",
        section: "Memory",
        accessory: "Clipboard",
        leading_mark: "CL",
    },
    PickerRow {
        id: "mention-git-diff",
        label: "Git Diff",
        meta: "@git-diff",
        description: "Stage the current repository diff as context for the thread.",
        section: "Workspace",
        accessory: "Repo",
        leading_mark: "GD",
    },
    PickerRow {
        id: "mention-recent-scripts",
        label: "Recent Scripts",
        meta: "@recent-scripts",
        description: "Reference the scripts you just launched without re-explaining them.",
        section: "Workspace",
        accessory: "History",
        leading_mark: "RS",
    },
];

const SLASH_ROWS: [PickerRow; 6] = [
    PickerRow {
        id: "slash-compact",
        label: "Compact Thread",
        meta: "/compact",
        description: "Compress the current thread before continuing with a tighter context budget.",
        section: "Core",
        accessory: "Core",
        leading_mark: "CP",
    },
    PickerRow {
        id: "slash-clear",
        label: "Clear Thread",
        meta: "/clear",
        description: "Reset the current conversation while keeping the composer focused.",
        section: "Core",
        accessory: "Core",
        leading_mark: "CL",
    },
    PickerRow {
        id: "slash-context",
        label: "Current Context",
        meta: "/context",
        description: "Insert the current desktop context bundle with the minimal profile.",
        section: "Context",
        accessory: "Context",
        leading_mark: "CT",
    },
    PickerRow {
        id: "slash-browser",
        label: "Browser URL",
        meta: "/browser",
        description: "Insert just the active browser location instead of the full desktop bundle.",
        section: "Context",
        accessory: "Source",
        leading_mark: "BR",
    },
    PickerRow {
        id: "slash-review",
        label: "Review Diff",
        meta: "/review-diff",
        description:
            "Open the review-diff skill to inspect a patch with a strict code-review lens.",
        section: "Skills",
        accessory: "Claude",
        leading_mark: "RV",
    },
    PickerRow {
        id: "slash-gh-fix-ci",
        label: "Fix CI",
        meta: "/gh-fix-ci",
        description: "Use the GitHub CI skill to inspect failing checks before proposing edits.",
        section: "Skills",
        accessory: "GitHub",
        leading_mark: "CI",
    },
];

const SPECS: [ContextPickerPopupPlaygroundSpec; 14] = [
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionWhisperDense,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::WhisperDense,
        query: "scr",
        selected_row_id: "mention-screenshot",
        note: "Closest to the current runtime surface, just tighter and cleaner.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionGroupedCatalog,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::GroupedCatalog,
        query: "git",
        selected_row_id: "mention-git-diff",
        note: "Section headers help when the mention catalog grows past the built-in set.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionLeadingVisuals,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::LeadingVisuals,
        query: "bro",
        selected_row_id: "mention-browser",
        note: "Adds restrained leading markers without abandoning the whisper chrome.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionAccessoryBadges,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::AccessoryBadges,
        query: "cl",
        selected_row_id: "mention-clipboard",
        note: "Trades repeated inline tokens for lighter source badges.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionFlatCompact,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::FlatCompact,
        query: "sel",
        selected_row_id: "mention-selection",
        note: "Pushes further toward footer-density and flatter panel treatment.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionSynopsisRail,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::SynopsisRail,
        query: "rec",
        selected_row_id: "mention-recent-scripts",
        note: "Keeps the list dense while making the focused attachment easier to understand.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionEmptyState,
        trigger: ContextPickerPopupTrigger::Mention,
        style: ContextPickerPopupStyle::EmptyState,
        query: "xyz",
        selected_row_id: "mention-screenshot",
        note: "Focuses on no-match recovery instead of implying failure.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashWhisperDense,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::WhisperDense,
        query: "con",
        selected_row_id: "slash-context",
        note: "Baseline slash picker tuned to the existing mono command language.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashGroupedCatalog,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::GroupedCatalog,
        query: "gh",
        selected_row_id: "slash-gh-fix-ci",
        note: "Best for a mixed catalog of built-ins, Claude commands, and plugin skills.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashLeadingVisuals,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::LeadingVisuals,
        query: "bro",
        selected_row_id: "slash-browser",
        note: "Adds source markers without turning the slash picker into a dashboard.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashAccessoryBadges,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::AccessoryBadges,
        query: "re",
        selected_row_id: "slash-review",
        note: "Makes command ownership clearer when multiple skills share the same namespace.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashFlatCompact,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::FlatCompact,
        query: "cl",
        selected_row_id: "slash-clear",
        note: "Optimizes for speed when the user already knows the command they want.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashSynopsisRail,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::SynopsisRail,
        query: "fix",
        selected_row_id: "slash-gh-fix-ci",
        note: "Leans into learnability by giving the focused command more explanatory weight.",
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashEmptyState,
        trigger: ContextPickerPopupTrigger::Slash,
        style: ContextPickerPopupStyle::EmptyState,
        query: "zz",
        selected_row_id: "slash-context",
        note: "Keeps no-match slash discovery useful with a few canonical hints.",
    },
];

pub fn context_picker_popup_playground_story_variants() -> Vec<StoryVariant> {
    SPECS.iter().map(|spec| story_variant(*spec)).collect()
}

pub fn mention_picker_redesign_story_variants() -> Vec<StoryVariant> {
    specs_for_trigger(ContextPickerPopupTrigger::Mention)
        .iter()
        .map(|spec| story_variant(*spec))
        .collect()
}

pub fn slash_picker_redesign_story_variants() -> Vec<StoryVariant> {
    specs_for_trigger(ContextPickerPopupTrigger::Slash)
        .iter()
        .map(|spec| story_variant(*spec))
        .collect()
}

pub fn render_context_picker_popup_playground_story_preview(stable_id: &str) -> AnyElement {
    render_spec_surface(resolve_spec(stable_id).unwrap_or(SPECS[0]), false)
}

pub fn render_mention_picker_redesign_story_preview(stable_id: &str) -> AnyElement {
    render_spec_surface(
        resolve_trigger_spec(ContextPickerPopupTrigger::Mention, stable_id)
            .unwrap_or(specs_for_trigger(ContextPickerPopupTrigger::Mention)[0]),
        false,
    )
}

pub fn render_slash_picker_redesign_story_preview(stable_id: &str) -> AnyElement {
    render_spec_surface(
        resolve_trigger_spec(ContextPickerPopupTrigger::Slash, stable_id)
            .unwrap_or(specs_for_trigger(ContextPickerPopupTrigger::Slash)[0]),
        false,
    )
}

pub fn render_mention_picker_redesign_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_spec_surface(
        resolve_trigger_spec(ContextPickerPopupTrigger::Mention, stable_id)
            .unwrap_or(specs_for_trigger(ContextPickerPopupTrigger::Mention)[0]),
        true,
    )
}

pub fn render_slash_picker_redesign_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_spec_surface(
        resolve_trigger_spec(ContextPickerPopupTrigger::Slash, stable_id)
            .unwrap_or(specs_for_trigger(ContextPickerPopupTrigger::Slash)[0]),
        true,
    )
}

pub fn render_mention_picker_redesign_gallery() -> AnyElement {
    render_gallery(
        ContextPickerPopupTrigger::Mention,
        "Mention Picker Redesigns",
        "Seven directions for the `@` picker, all constrained to the current typography and whisper-chrome rules.",
    )
}

pub fn render_slash_picker_redesign_gallery() -> AnyElement {
    render_gallery(
        ContextPickerPopupTrigger::Slash,
        "Slash Command Redesigns",
        "Seven directions for the slash picker, tuned for clearer command ownership and faster scan speed.",
    )
}

pub fn slash_picker_typography_story_variants() -> Vec<StoryVariant> {
    SLASH_TYPOGRAPHY_SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "slash-picker-typography")
                .with_prop("variantId", spec.id.as_str())
        })
        .collect()
}

pub fn render_slash_picker_typography_story_preview(stable_id: &str) -> AnyElement {
    render_slash_typography_surface(resolve_slash_typography_spec(stable_id), false)
}

pub fn render_slash_picker_typography_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_slash_typography_surface(resolve_slash_typography_spec(stable_id), true)
}

pub fn render_slash_picker_typography_gallery() -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    div()
        .w_full()
        .h_full()
        .overflow_y_scrollbar()
        .child(
            story_container()
                .gap(px(18.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text.primary.to_rgb())
                                .child("Slash Picker Typography"),
                        )
                        .child(
                            div()
                                .max_w(px(760.0))
                                .text_xs()
                                .text_color(theme.colors.text.muted.to_rgb())
                                .child(
                                    "Five narrow typography treatments for the slash popup. Pick based on scan comfort, selected-row emphasis, and owner metadata weight.",
                                ),
                        ),
                )
                .children(SLASH_TYPOGRAPHY_SPECS.iter().copied().map(|spec| {
                    div()
                        .rounded(px(12.0))
                        .border_1()
                        .border_color(gpui::rgba(chrome.border_rgba))
                        .bg(gpui::rgba(chrome.surface_rgba))
                        .p(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(theme.colors.text.dimmed.with_opacity(0.55))
                                        .child(spec.id.as_str()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.colors.text.primary.to_rgb())
                                        .child(spec.id.name()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.colors.text.muted.to_rgb())
                                        .child(spec.id.description()),
                                ),
                        )
                        .child(render_slash_typography_surface(spec, false))
                })),
        )
        .into_any_element()
}

fn story_variant(spec: ContextPickerPopupPlaygroundSpec) -> StoryVariant {
    StoryVariant::default_named(spec.id.as_str(), spec.id.name())
        .description(spec.id.description())
        .with_prop("surface", "context-picker-popup-playground")
        .with_prop("variantId", spec.id.as_str())
        .with_prop(
            "trigger",
            match spec.trigger {
                ContextPickerPopupTrigger::Mention => "mention",
                ContextPickerPopupTrigger::Slash => "slash",
            },
        )
}

fn resolve_slash_typography_spec(stable_id: &str) -> SlashTypographySpec {
    let id = SlashPickerTypographyVariantId::from_stable_id(stable_id)
        .unwrap_or(SlashPickerTypographyVariantId::LauncherRegular);
    SLASH_TYPOGRAPHY_SPECS
        .iter()
        .copied()
        .find(|spec| spec.id == id)
        .unwrap_or(SLASH_TYPOGRAPHY_SPECS[0])
}

fn render_slash_typography_surface(spec: SlashTypographySpec, compact: bool) -> AnyElement {
    let shell = IntegratedSurfaceShellConfig {
        width: if compact { 380.0 } else { 560.0 },
        height: if compact { 255.0 } else { 305.0 },
        ..Default::default()
    };
    let visible_labels = SLASH_ROWS.iter().map(|row| row.meta).collect::<Vec<_>>();
    let metrics = context_picker_playground_overlay_metrics(
        shell,
        ContextPickerPopupTrigger::Slash,
        ContextPickerPopupSceneState::Results,
        true,
        visible_labels.iter().copied(),
    );

    IntegratedSurfaceShell::new(
        shell,
        render_slash_typography_chat_body(spec.id.name(), compact),
    )
    .footer(render_footer())
    .overlay(metrics.placement, render_slash_typography_dropdown(spec))
    .into_any_element()
}

fn render_slash_typography_chat_body(label: &str, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .gap(px(if compact { 10.0 } else { 12.0 }))
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(theme.colors.text.dimmed.with_opacity(0.55))
                .child(label.to_string()),
        )
        .child(
            div()
                .rounded(px(8.0))
                .bg(gpui::rgba(chrome.surface_rgba))
                .px(px(12.0))
                .py(px(10.0))
                .text_size(px(13.0))
                .text_color(theme.colors.text.secondary.to_rgb())
                .child("Compare the selected command row, owner metadata, and line rhythm."),
        )
        .child(
            div()
                .rounded(px(8.0))
                .bg(gpui::rgba(chrome.input_surface_rgba))
                .border_1()
                .border_color(gpui::rgba(chrome.divider_rgba))
                .px(px(12.0))
                .py(px(10.0))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(3.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_family(FONT_MONO)
                                .text_color(theme.colors.accent.selected.to_rgb())
                                .child("/compact"),
                        )
                        .child(
                            div()
                                .w(px(1.5))
                                .h(px(14.0))
                                .bg(theme.colors.text.primary.to_rgb()),
                        ),
                ),
        )
        .into_any_element()
}

fn render_slash_typography_dropdown(spec: SlashTypographySpec) -> AnyElement {
    let theme = get_cached_theme();
    let colors = InlineDropdownColors::popup_from_theme(&theme);
    let selected_row = SLASH_ROWS
        .iter()
        .find(|row| row.id == "slash-compact")
        .unwrap_or(&SLASH_ROWS[0]);
    let body = div()
        .w_full()
        .flex()
        .flex_col()
        .children(
            SLASH_ROWS
                .iter()
                .map(|row| render_slash_typography_row(spec, row, colors)),
        )
        .into_any_element();

    InlineDropdown::new(
        SharedString::from(format!("slash-typography-{}", spec.id.as_str())),
        body,
        colors,
    )
    .vertical_padding(4.0)
    .horizontal_padding(6.0)
    .synopsis(Some(InlineDropdownSynopsis {
        label: SharedString::from(selected_row.label),
        meta: SharedString::from(selected_row.meta),
        description: SharedString::from(selected_row.description),
    }))
    .into_any_element()
}

fn render_slash_typography_row(
    spec: SlashTypographySpec,
    row: &PickerRow,
    colors: InlineDropdownColors,
) -> AnyElement {
    let is_selected = row.id == "slash-compact";
    let label = row.meta;
    let label_hits = highlight_indices(label, "com");
    let label_hit_set = label_hits
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let foreground = if is_selected {
        colors.foreground
    } else {
        colors.foreground.opacity(MUTED_OP)
    };
    let selected_bg = colors.foreground.opacity(spec.selected_fill);
    let hover_bg = colors.foreground.opacity(GHOST);

    div()
        .id(SharedString::from(format!(
            "slash-typography-{}-{}",
            spec.id.as_str(),
            row.id
        )))
        .w_full()
        .h(px(spec.row_height))
        .flex()
        .items_center()
        .justify_between()
        .border_l(px(2.0))
        .border_color(if is_selected {
            colors.accent
        } else {
            gpui::transparent_black()
        })
        .pl(px(10.0))
        .pr(px(14.0))
        .py(px(4.0))
        .bg(if is_selected {
            selected_bg
        } else {
            gpui::transparent_black()
        })
        .when(!is_selected, |d| d.hover(|el| el.bg(hover_bg)))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(render_slash_typography_label(
                    label,
                    &label_hit_set,
                    spec,
                    if is_selected {
                        spec.selected_weight
                    } else {
                        spec.unselected_weight
                    },
                    foreground,
                    colors.accent,
                )),
        )
        .child(render_slash_typography_metadata(
            row.accessory,
            is_selected,
            spec,
            colors,
        ))
        .into_any_element()
}

fn render_slash_typography_label(
    text: &str,
    hits: &std::collections::HashSet<usize>,
    spec: SlashTypographySpec,
    weight: FontWeight,
    base: Hsla,
    accent: Hsla,
) -> AnyElement {
    let font_family = match spec.label_font {
        SlashTypographyFont::System => None,
        SlashTypographyFont::Mono => Some(FONT_MONO),
    };

    if hits.is_empty() {
        let mut label = div()
            .text_size(px(spec.label_size))
            .line_height(px(spec.label_line_height))
            .font_weight(weight)
            .text_color(base)
            .text_ellipsis()
            .child(SharedString::from(text.to_string()));
        if let Some(font) = font_family {
            label = label.font_family(font);
        }
        return label.into_any_element();
    }

    let mut spans: Vec<AnyElement> = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (ix, ch) in text.chars().enumerate() {
        let is_hit = hits.contains(&ix);
        if ix > 0 && is_hit != current_highlighted {
            spans.push(render_slash_typography_span(
                std::mem::take(&mut current),
                spec,
                font_family,
                weight,
                if current_highlighted { accent } else { base },
            ));
        }
        current_highlighted = is_hit;
        current.push(ch);
    }

    if !current.is_empty() {
        spans.push(render_slash_typography_span(
            current,
            spec,
            font_family,
            weight,
            if current_highlighted { accent } else { base },
        ));
    }

    div()
        .flex()
        .items_center()
        .text_ellipsis()
        .children(spans)
        .into_any_element()
}

fn render_slash_typography_span(
    text: String,
    spec: SlashTypographySpec,
    font_family: Option<&'static str>,
    weight: FontWeight,
    color: Hsla,
) -> AnyElement {
    let mut span = div()
        .text_size(px(spec.label_size))
        .line_height(px(spec.label_line_height))
        .font_weight(weight)
        .text_color(color)
        .child(SharedString::from(text));
    if let Some(font) = font_family {
        span = span.font_family(font);
    }
    span.into_any_element()
}

fn render_slash_typography_metadata(
    label: &str,
    is_selected: bool,
    spec: SlashTypographySpec,
    colors: InlineDropdownColors,
) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let text_color = if is_selected {
        colors.foreground.opacity(MUTED_OP)
    } else {
        colors.muted_foreground.opacity(HINT)
    };
    let text = div()
        .text_size(px(spec.meta_size))
        .line_height(px(14.0))
        .font_family(FONT_MONO)
        .text_color(text_color)
        .child(SharedString::from(label.to_string()));

    match spec.metadata_style {
        SlashTypographyMetadataStyle::BareText => text.into_any_element(),
        SlashTypographyMetadataStyle::FlatText => {
            div().px(px(4.0)).py(px(2.0)).child(text).into_any_element()
        }
        SlashTypographyMetadataStyle::SoftBadge => div()
            .px(px(6.0))
            .py(px(2.0))
            .rounded(px(4.0))
            .bg(gpui::rgba(chrome.badge_bg_rgba))
            .child(text)
            .into_any_element(),
    }
}

fn specs_for_trigger(
    trigger: ContextPickerPopupTrigger,
) -> &'static [ContextPickerPopupPlaygroundSpec] {
    match trigger {
        ContextPickerPopupTrigger::Mention => &SPECS[..7],
        ContextPickerPopupTrigger::Slash => &SPECS[7..],
    }
}

fn resolve_spec(stable_id: &str) -> Option<ContextPickerPopupPlaygroundSpec> {
    SPECS
        .iter()
        .find(|spec| spec.id.as_str() == stable_id)
        .copied()
}

fn resolve_trigger_spec(
    trigger: ContextPickerPopupTrigger,
    stable_id: &str,
) -> Option<ContextPickerPopupPlaygroundSpec> {
    specs_for_trigger(trigger)
        .iter()
        .find(|spec| spec.id.as_str() == stable_id)
        .copied()
}

fn render_gallery(trigger: ContextPickerPopupTrigger, title: &str, intro: &str) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let specs = specs_for_trigger(trigger);

    div()
        .w_full()
        .h_full()
        .overflow_y_scrollbar()
        .child(
            story_container()
                .gap(px(18.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text.primary.to_rgb())
                                .child(title.to_string()),
                        )
                        .child(
                            div()
                                .max_w(px(760.0))
                                .text_xs()
                                .text_color(theme.colors.text.muted.to_rgb())
                                .child(intro.to_string()),
                        ),
                )
                .children(specs.iter().copied().map(|spec| {
                    div()
                        .rounded(px(12.0))
                        .border_1()
                        .border_color(gpui::rgba(chrome.border_rgba))
                        .bg(gpui::rgba(chrome.surface_rgba))
                        .p(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(theme.colors.text.dimmed.with_opacity(0.55))
                                        .child(spec.id.as_str()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.colors.text.primary.to_rgb())
                                        .child(spec.id.name()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.colors.text.muted.to_rgb())
                                        .child(spec.note),
                                ),
                        )
                        .child(render_spec_surface(spec, false))
                })),
        )
        .into_any_element()
}

fn render_spec_surface(spec: ContextPickerPopupPlaygroundSpec, compact: bool) -> AnyElement {
    let shell = IntegratedSurfaceShellConfig {
        width: if compact { 380.0 } else { 560.0 },
        height: if compact { 255.0 } else { 305.0 },
        ..Default::default()
    };

    let show_synopsis = show_synopsis(spec);
    let visible_labels = rows_for_trigger(spec.trigger)
        .iter()
        .map(|row| row.label)
        .collect::<Vec<_>>();
    let metrics = context_picker_playground_overlay_metrics(
        shell,
        spec.trigger,
        scene_state(spec),
        show_synopsis,
        visible_labels.iter().copied(),
    );

    IntegratedSurfaceShell::new(shell, render_chat_body(spec, compact))
        .footer(render_footer())
        .overlay(metrics.placement, render_dropdown(spec))
        .into_any_element()
}

fn render_footer() -> AnyElement {
    let theme = get_cached_theme();
    let colors = PromptFooterColors::from_theme(&theme);
    let config =
        config_from_storybook_footer_selection_value(Some(FooterVariationId::Minimal.as_str()));

    PromptFooter::new(config, colors).into_any_element()
}

fn render_chat_body(spec: ContextPickerPopupPlaygroundSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let trigger = match spec.trigger {
        ContextPickerPopupTrigger::Mention => "@",
        ContextPickerPopupTrigger::Slash => "/",
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .gap(px(if compact { 10.0 } else { 12.0 }))
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(theme.colors.text.dimmed.with_opacity(0.55))
                .child("ACP composer"),
        )
        .child(
            div()
                .rounded(px(8.0))
                .bg(gpui::rgba(chrome.surface_rgba))
                .px(px(12.0))
                .py(px(10.0))
                .text_size(px(13.0))
                .text_color(theme.colors.text.secondary.to_rgb())
                .child("Summarize the issue and tell me what context is still missing."),
        )
        .child(
            div()
                .rounded(px(8.0))
                .bg(gpui::rgba(chrome.input_surface_rgba))
                .border_1()
                .border_color(gpui::rgba(chrome.divider_rgba))
                .px(px(12.0))
                .py(px(10.0))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(3.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.colors.text.primary.to_rgb())
                                .child("Use "),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_family(FONT_MONO)
                                .text_color(theme.colors.accent.selected.to_rgb())
                                .child(format!("{trigger}{}", spec.query)),
                        )
                        .child(
                            div()
                                .w(px(1.5))
                                .h(px(14.0))
                                .bg(theme.colors.text.primary.to_rgb()),
                        ),
                ),
        )
        .into_any_element()
}

fn render_dropdown(spec: ContextPickerPopupPlaygroundSpec) -> AnyElement {
    let theme = get_cached_theme();
    let colors = dropdown_colors(spec.style, &theme);

    if spec.style == ContextPickerPopupStyle::EmptyState {
        return render_empty_dropdown(spec, colors);
    }

    let source_rows = rows_for_trigger(spec.trigger);
    let context_items = source_rows
        .iter()
        .map(|row| picker_row_to_context_item(spec, row))
        .collect::<Vec<_>>();
    let rows: Vec<InlinePickerRow> = context_items
        .iter()
        .map(acp_context_picker_item_to_inline_picker_row)
        .collect();
    let requested_selected = source_rows
        .iter()
        .position(|row| row.id == spec.selected_row_id);
    let selected_index = inline_picker_normalize_selected_index(&rows, requested_selected);
    let mut children: Vec<AnyElement> = Vec::new();
    let mut last_section: Option<&str> = None;

    for (idx, row) in rows.iter().enumerate() {
        let source_row = &source_rows[idx];
        if show_sections(spec) && last_section != Some(source_row.section) {
            last_section = Some(source_row.section);
            let count = source_rows
                .iter()
                .filter(|candidate| candidate.section == source_row.section)
                .count();
            children.push(render_section_header(source_row.section, count));
        }

        children.push(render_row(spec, row, selected_index == Some(idx), colors));
    }

    let selected_row = selected_index
        .and_then(|idx| rows.get(idx))
        .unwrap_or(&rows[0]);
    let synopsis = show_synopsis(spec).then(|| InlineDropdownSynopsis {
        label: selected_row.title.clone(),
        meta: selected_row
            .token
            .clone()
            .unwrap_or_else(|| SharedString::from("")),
        description: selected_row
            .detail
            .clone()
            .unwrap_or_else(|| SharedString::from("")),
    });

    InlineDropdown::new(
        SharedString::from(format!("context-picker-{}", spec.id.as_str())),
        div()
            .w_full()
            .flex()
            .flex_col()
            .children(children)
            .into_any_element(),
        colors,
    )
    .vertical_padding(vertical_padding(spec.style))
    .horizontal_padding(horizontal_padding(spec.style))
    .synopsis(synopsis)
    .into_any_element()
}

fn render_empty_dropdown(
    spec: ContextPickerPopupPlaygroundSpec,
    colors: InlineDropdownColors,
) -> AnyElement {
    let theme = get_cached_theme();
    let (message, hints) = match spec.trigger {
        ContextPickerPopupTrigger::Mention => (
            "No matching context",
            vec!["@screenshot", "@clipboard", "@git-diff", "@recent-scripts"],
        ),
        ContextPickerPopupTrigger::Slash => (
            "No matching commands",
            vec!["/compact", "/clear", "/context", "/review-diff"],
        ),
    };

    let hint_elements = hints
        .into_iter()
        .map(|hint| {
            div()
                .px(px(6.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .bg(colors.foreground.opacity(GHOST))
                .border_1()
                .border_color(colors.foreground.opacity(0.08))
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_MONO)
                        .text_color(theme.colors.text.muted.with_opacity(HINT))
                        .child(hint),
                )
                .into_any_element()
        })
        .collect::<Vec<_>>();

    InlineDropdown::new(
        SharedString::from(format!("context-picker-empty-{}", spec.id.as_str())),
        div().into_any_element(),
        colors,
    )
    .vertical_padding(vertical_padding(spec.style))
    .horizontal_padding(horizontal_padding(spec.style))
    .empty_state(InlineDropdownEmptyState {
        message: SharedString::from(message),
        hints: hint_elements,
    })
    .into_any_element()
}

fn render_row(
    spec: ContextPickerPopupPlaygroundSpec,
    row: &InlinePickerRow,
    is_selected: bool,
    colors: InlineDropdownColors,
) -> AnyElement {
    let _ = spec;
    let label_hits = row
        .highlights
        .title
        .iter()
        .map(|range| range.start)
        .collect::<Vec<_>>();
    let meta_hits = row
        .highlights
        .token
        .iter()
        .map(|range| range.start)
        .collect::<Vec<_>>();

    render_soft_compact_picker_row(
        row.id.clone(),
        row.title.clone(),
        row.token.clone(),
        &label_hits,
        &meta_hits,
        is_selected,
        colors,
    )
    .h(px(SOFT_COMPACT_PICKER_ROW_HEIGHT))
    .into_any_element()
}

fn picker_row_to_context_item(
    spec: ContextPickerPopupPlaygroundSpec,
    row: &PickerRow,
) -> ContextPickerItem {
    let label_highlight_indices = highlight_indices(row.label, spec.query);
    let meta_highlight_indices = highlight_indices(row.meta, spec.query);
    let kind = match spec.trigger {
        ContextPickerPopupTrigger::Slash => {
            ContextPickerItemKind::SlashCommand(SlashCommandPayload::Default {
                name: row.meta.trim_start_matches('/').to_string(),
            })
        }
        ContextPickerPopupTrigger::Mention => ContextPickerItemKind::Portal(PortalKind::FileSearch),
    };

    ContextPickerItem {
        id: SharedString::from(row.id),
        label: SharedString::from(row.label),
        description: SharedString::from(row.description),
        meta: SharedString::from(row.meta),
        kind,
        score: 100,
        label_highlight_indices,
        meta_highlight_indices,
    }
}

fn render_section_header(label: &str, count: usize) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .px(px(8.0))
        .pt(px(6.0))
        .pb(px(2.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(theme.colors.text.dimmed.with_opacity(0.55))
                .child(label.to_uppercase()),
        )
        .child(
            div()
                .text_xs()
                .font_family(FONT_MONO)
                .text_color(theme.colors.text.dimmed.with_opacity(0.42))
                .child(count.to_string()),
        )
        .into_any_element()
}

#[allow(dead_code)]
fn render_accessory_badge(
    label: &str,
    trigger: ContextPickerPopupTrigger,
    is_selected: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let text = match trigger {
        ContextPickerPopupTrigger::Mention => theme.colors.text.secondary,
        ContextPickerPopupTrigger::Slash => theme.colors.text.on_accent,
    };
    let bg = match trigger {
        ContextPickerPopupTrigger::Mention => chrome.badge_bg_rgba,
        ContextPickerPopupTrigger::Slash => {
            if is_selected {
                chrome.accent_badge_bg_rgba
            } else {
                chrome.badge_bg_rgba
            }
        }
    };
    let border = match trigger {
        ContextPickerPopupTrigger::Mention => chrome.badge_border_rgba,
        ContextPickerPopupTrigger::Slash => {
            if is_selected {
                chrome.accent_badge_border_rgba
            } else {
                chrome.badge_border_rgba
            }
        }
    };

    div()
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(gpui::rgba(bg))
        .border_1()
        .border_color(gpui::rgba(border))
        .child(
            div()
                .text_size(px(10.5))
                .font_family(FONT_MONO)
                .text_color(gpui::rgb(text))
                .child(SharedString::from(label.to_string())),
        )
        .into_any_element()
}

#[allow(dead_code)]
fn render_leading_visual(
    row: &PickerRow,
    trigger: ContextPickerPopupTrigger,
    is_selected: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let accent = match trigger {
        ContextPickerPopupTrigger::Mention => theme.colors.accent.selected_subtle,
        ContextPickerPopupTrigger::Slash => theme.colors.accent.selected,
    };
    let bg = if is_selected {
        gpui::rgba(chrome.accent_badge_bg_rgba)
    } else {
        gpui::rgba((accent << 8) | 0x28)
    };
    let text = if is_selected {
        theme.colors.text.on_accent
    } else {
        theme.colors.text.secondary
    };

    div()
        .w(px(18.0))
        .h(px(18.0))
        .rounded(px(5.0))
        .bg(bg)
        .border_1()
        .border_color(gpui::rgba(chrome.badge_border_rgba))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(9.5))
                .font_family(FONT_MONO)
                .text_color(gpui::rgb(text))
                .child(row.leading_mark),
        )
        .into_any_element()
}

fn dropdown_colors(
    style: ContextPickerPopupStyle,
    theme: &crate::theme::Theme,
) -> InlineDropdownColors {
    let chrome = AppChromeColors::from_theme(theme);
    let mut colors = InlineDropdownColors::from_theme(theme);

    match style {
        ContextPickerPopupStyle::WhisperDense | ContextPickerPopupStyle::GroupedCatalog => {}
        ContextPickerPopupStyle::LeadingVisuals | ContextPickerPopupStyle::AccessoryBadges => {
            colors.surface_rgba = chrome.popup_surface_rgba;
        }
        ContextPickerPopupStyle::FlatCompact => {
            colors.surface_rgba = chrome.window_surface_rgba;
            colors.border_rgba = chrome.divider_rgba;
        }
        ContextPickerPopupStyle::SynopsisRail => {
            colors.surface_rgba = chrome.popup_surface_rgba;
            colors.divider_rgba = chrome.border_rgba;
        }
        ContextPickerPopupStyle::EmptyState => {
            colors.surface_rgba = chrome.surface_rgba;
            colors.border_rgba = chrome.divider_rgba;
        }
    }

    colors
}

fn vertical_padding(style: ContextPickerPopupStyle) -> f32 {
    match style {
        ContextPickerPopupStyle::FlatCompact => 2.0,
        ContextPickerPopupStyle::SynopsisRail => 5.0,
        _ => 4.0,
    }
}

fn horizontal_padding(style: ContextPickerPopupStyle) -> f32 {
    match style {
        ContextPickerPopupStyle::FlatCompact => 4.0,
        _ => 6.0,
    }
}

fn show_sections(spec: ContextPickerPopupPlaygroundSpec) -> bool {
    matches!(spec.style, ContextPickerPopupStyle::GroupedCatalog)
}

fn show_synopsis(spec: ContextPickerPopupPlaygroundSpec) -> bool {
    matches!(
        spec.style,
        ContextPickerPopupStyle::WhisperDense
            | ContextPickerPopupStyle::LeadingVisuals
            | ContextPickerPopupStyle::SynopsisRail
    )
}

fn scene_state(spec: ContextPickerPopupPlaygroundSpec) -> ContextPickerPopupSceneState {
    match spec.style {
        ContextPickerPopupStyle::EmptyState => ContextPickerPopupSceneState::Empty,
        _ => ContextPickerPopupSceneState::Results,
    }
}

fn rows_for_trigger(trigger: ContextPickerPopupTrigger) -> &'static [PickerRow] {
    match trigger {
        ContextPickerPopupTrigger::Mention => &MENTION_ROWS,
        ContextPickerPopupTrigger::Slash => &SLASH_ROWS,
    }
}

fn highlight_indices(text: &str, query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(start) = text_lower.find(&query_lower) {
        (start..start + query_lower.len()).collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        context_picker_popup_playground_story_variants, mention_picker_redesign_story_variants,
        slash_picker_redesign_story_variants, slash_picker_typography_story_variants,
        ContextPickerPopupPlaygroundId, SlashPickerTypographyVariantId,
    };
    use std::collections::HashSet;

    #[test]
    fn context_picker_popup_playground_variant_ids_are_unique() {
        let ids: HashSet<_> = context_picker_popup_playground_story_variants()
            .into_iter()
            .map(|variant| variant.stable_id())
            .collect();
        assert_eq!(ids.len(), ContextPickerPopupPlaygroundId::ALL.len());
    }

    #[test]
    fn context_picker_popup_playground_stable_ids_round_trip() {
        for id in ContextPickerPopupPlaygroundId::ALL {
            assert_eq!(
                ContextPickerPopupPlaygroundId::from_stable_id(id.as_str()),
                Some(id)
            );
        }
    }

    #[test]
    fn mention_and_slash_story_sets_each_expose_seven_variants() {
        assert_eq!(mention_picker_redesign_story_variants().len(), 7);
        assert_eq!(slash_picker_redesign_story_variants().len(), 7);
    }

    #[test]
    fn slash_typography_story_exposes_five_variants() {
        assert_eq!(slash_picker_typography_story_variants().len(), 5);
    }

    #[test]
    fn slash_typography_variant_ids_are_unique() {
        let ids: HashSet<_> = slash_picker_typography_story_variants()
            .into_iter()
            .map(|variant| variant.stable_id())
            .collect();
        assert_eq!(ids.len(), SlashPickerTypographyVariantId::ALL.len());
    }

    #[test]
    fn slash_typography_stable_ids_round_trip() {
        for id in SlashPickerTypographyVariantId::ALL {
            assert_eq!(
                SlashPickerTypographyVariantId::from_stable_id(id.as_str()),
                Some(id)
            );
        }
    }
}
