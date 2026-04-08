//! Context-picker popup playground — integrated surface scenes for compare mode.
//!
//! Four stable variants (`mention-dense`, `mention-grouped`, `slash-dense`,
//! `slash-grouped`) rendered via `IntegratedSurfaceShell` with a real
//! `PromptFooter` and real `InlineDropdown` anchored under the typed trigger.
//! No production ACP or live picker code is touched.

use gpui::*;

use crate::components::inline_dropdown::{
    render_dense_monoline_picker_row, InlineDropdown, InlineDropdownColors, InlineDropdownSynopsis,
};
use crate::components::prompt_footer::{PromptFooter, PromptFooterColors};
use crate::list_item::FONT_MONO;
use crate::storybook::{
    adopted_surface_live, config_from_storybook_footer_selection_value,
    playground_overlay_metrics::context_picker_playground_overlay_metrics, resolve_surface_live,
    AdoptableSurface, FooterVariationId, IntegratedSurfaceShell, IntegratedSurfaceShellConfig,
    StoryVariant, SurfaceSelectionResolution, VariationId,
};
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

// ---------------------------------------------------------------------------
// Variant IDs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextPickerPopupPlaygroundId {
    MentionDense,
    MentionGrouped,
    SlashDense,
    SlashGrouped,
}

impl ContextPickerPopupPlaygroundId {
    pub const ALL: [Self; 4] = [
        Self::MentionDense,
        Self::MentionGrouped,
        Self::SlashDense,
        Self::SlashGrouped,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MentionDense => "mention-dense",
            Self::MentionGrouped => "mention-grouped",
            Self::SlashDense => "slash-dense",
            Self::SlashGrouped => "slash-grouped",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MentionDense => "Mention Dense",
            Self::MentionGrouped => "Mention Grouped",
            Self::SlashDense => "Slash Dense",
            Self::SlashGrouped => "Slash Grouped",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::MentionDense => "Dense @-mention popup with synopsis strip.",
            Self::MentionGrouped => "Grouped @-mention popup with section headers.",
            Self::SlashDense => "Dense slash popup with mono meta commands.",
            Self::SlashGrouped => "Grouped slash popup for discovery-heavy command sets.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "mention-dense" => Some(Self::MentionDense),
            "mention-grouped" => Some(Self::MentionGrouped),
            "slash-dense" => Some(Self::SlashDense),
            "slash-grouped" => Some(Self::SlashGrouped),
            _ => None,
        }
    }
}

impl VariationId for ContextPickerPopupPlaygroundId {
    fn as_str(self) -> &'static str {
        ContextPickerPopupPlaygroundId::as_str(self)
    }

    fn name(self) -> &'static str {
        ContextPickerPopupPlaygroundId::name(self)
    }

    fn description(self) -> &'static str {
        ContextPickerPopupPlaygroundId::description(self)
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        ContextPickerPopupPlaygroundId::from_stable_id(value)
    }
}

// ---------------------------------------------------------------------------
// Selection (typed live representation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextPickerPopupPlaygroundSelection {
    pub id: ContextPickerPopupPlaygroundId,
    pub trigger: ContextPickerPopupTrigger,
    pub state: ContextPickerPopupSceneState,
    pub query: &'static str,
    pub show_sections: bool,
    pub show_synopsis: bool,
}

impl From<ContextPickerPopupPlaygroundSpec> for ContextPickerPopupPlaygroundSelection {
    fn from(spec: ContextPickerPopupPlaygroundSpec) -> Self {
        Self {
            id: spec.id,
            trigger: spec.trigger,
            state: spec.state,
            query: spec.query,
            show_sections: spec.show_sections,
            show_synopsis: spec.show_synopsis,
        }
    }
}

fn spec_from_context_picker_selection(
    selection: ContextPickerPopupPlaygroundSelection,
) -> ContextPickerPopupPlaygroundSpec {
    ContextPickerPopupPlaygroundSpec {
        id: selection.id,
        trigger: selection.trigger,
        state: selection.state,
        query: selection.query,
        show_sections: selection.show_sections,
        show_synopsis: selection.show_synopsis,
    }
}

// ---------------------------------------------------------------------------
// Adoptable surface
// ---------------------------------------------------------------------------

pub struct ContextPickerPopupPlaygroundSurface;

impl AdoptableSurface for ContextPickerPopupPlaygroundSurface {
    type Id = ContextPickerPopupPlaygroundId;
    type Spec = ContextPickerPopupPlaygroundSpec;
    type Live = ContextPickerPopupPlaygroundSelection;

    const STORY_ID: &'static str = "context-picker-popup-playground";
    const DEFAULT_ID: Self::Id = ContextPickerPopupPlaygroundId::MentionDense;

    fn specs() -> &'static [Self::Spec] {
        &SPECS
    }

    fn spec_id(spec: &Self::Spec) -> Self::Id {
        spec.id
    }

    fn live_from_spec(spec: &Self::Spec) -> Self::Live {
        (*spec).into()
    }
}

pub fn resolve_context_picker_popup_playground_selection(
    selected: Option<&str>,
) -> (
    ContextPickerPopupPlaygroundSelection,
    SurfaceSelectionResolution,
) {
    resolve_surface_live::<ContextPickerPopupPlaygroundSurface>(selected)
}

pub fn adopted_context_picker_popup_playground_selection() -> ContextPickerPopupPlaygroundSelection
{
    adopted_surface_live::<ContextPickerPopupPlaygroundSurface>()
}

// ---------------------------------------------------------------------------
// Trigger, Scene State & Spec
// ---------------------------------------------------------------------------

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
pub struct ContextPickerPopupPlaygroundSpec {
    pub id: ContextPickerPopupPlaygroundId,
    pub trigger: ContextPickerPopupTrigger,
    pub state: ContextPickerPopupSceneState,
    pub query: &'static str,
    pub show_sections: bool,
    pub show_synopsis: bool,
}

// ---------------------------------------------------------------------------
// Row data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct PickerRow {
    id: &'static str,
    label: &'static str,
    meta: &'static str,
    section: &'static str,
    selected: bool,
}

const MENTION_ROWS: [PickerRow; 4] = [
    PickerRow {
        id: "mention-screenshot",
        label: "Screenshot",
        meta: "@screenshot",
        section: "Context",
        selected: true,
    },
    PickerRow {
        id: "mention-selection",
        label: "Selection",
        meta: "@selection",
        section: "Context",
        selected: false,
    },
    PickerRow {
        id: "mention-browser",
        label: "Browser URL",
        meta: "@browser",
        section: "Context",
        selected: false,
    },
    PickerRow {
        id: "mention-git-diff",
        label: "Git Diff",
        meta: "@git-diff",
        section: "System",
        selected: false,
    },
];

const SLASH_ROWS: [PickerRow; 4] = [
    PickerRow {
        id: "slash-context",
        label: "Current Context",
        meta: "/context",
        section: "Context",
        selected: true,
    },
    PickerRow {
        id: "slash-full-context",
        label: "Full Context",
        meta: "/context-full",
        section: "Context",
        selected: false,
    },
    PickerRow {
        id: "slash-browser",
        label: "Browser URL",
        meta: "/browser",
        section: "Sources",
        selected: false,
    },
    PickerRow {
        id: "slash-window",
        label: "Focused Window",
        meta: "/window",
        section: "Sources",
        selected: false,
    },
];

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

const SPECS: [ContextPickerPopupPlaygroundSpec; 4] = [
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionDense,
        trigger: ContextPickerPopupTrigger::Mention,
        state: ContextPickerPopupSceneState::Results,
        query: "scr",
        show_sections: false,
        show_synopsis: true,
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionGrouped,
        trigger: ContextPickerPopupTrigger::Mention,
        state: ContextPickerPopupSceneState::Results,
        query: "git",
        show_sections: true,
        show_synopsis: true,
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashDense,
        trigger: ContextPickerPopupTrigger::Slash,
        state: ContextPickerPopupSceneState::Results,
        query: "con",
        show_sections: false,
        show_synopsis: true,
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashGrouped,
        trigger: ContextPickerPopupTrigger::Slash,
        state: ContextPickerPopupSceneState::Results,
        query: "bro",
        show_sections: true,
        show_synopsis: true,
    },
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn context_picker_popup_playground_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "context-picker-popup-playground")
                .with_prop("variantId", spec.id.as_str())
        })
        .collect()
}

pub fn render_context_picker_popup_playground_story_preview(stable_id: &str) -> AnyElement {
    let (selection, resolution) =
        resolve_context_picker_popup_playground_selection(Some(stable_id));

    tracing::info!(
        event = "context_picker_popup_playground_selection_resolved",
        requested_variant_id = resolution.requested_variant_id.as_deref().unwrap_or(""),
        resolved_variant_id = %resolution.resolved_variant_id,
        fallback_used = resolution.fallback_used,
        "Resolved context picker popup playground selection"
    );

    let spec = spec_from_context_picker_selection(selection);

    let shell = IntegratedSurfaceShellConfig {
        width: 560.0,
        height: 300.0,
        ..Default::default()
    };

    let labels: Vec<&str> = rows_for_trigger(spec.trigger)
        .iter()
        .map(|row| row.label)
        .collect();

    let metrics = context_picker_playground_overlay_metrics(
        shell,
        spec.trigger,
        spec.state,
        spec.show_synopsis,
        labels.iter().copied(),
    );

    tracing::info!(
        event = "context_picker_popup_playground_state_built",
        variant_id = spec.id.as_str(),
        trigger = match spec.trigger {
            ContextPickerPopupTrigger::Mention => "mention",
            ContextPickerPopupTrigger::Slash => "slash",
        },
        state = spec.state.as_str(),
        grouped = spec.show_sections,
        "Built context picker popup playground state"
    );

    tracing::info!(
        event = "context_picker_popup_playground_overlay_wired",
        variant_id = spec.id.as_str(),
        overlay_left = metrics.placement.left,
        overlay_top = metrics.placement.top,
        overlay_width = metrics.placement.width,
        "Wired context-picker playground overlay through shared metrics"
    );

    IntegratedSurfaceShell::new(shell, render_chat_body(spec))
        .footer(render_footer())
        .overlay(metrics.placement, render_dropdown(spec))
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn render_footer() -> AnyElement {
    let theme = get_cached_theme();
    let colors = PromptFooterColors::from_theme(&theme);
    let config =
        config_from_storybook_footer_selection_value(Some(FooterVariationId::Minimal.as_str()));

    PromptFooter::new(config, colors).into_any_element()
}

fn render_chat_body(spec: ContextPickerPopupPlaygroundSpec) -> AnyElement {
    let theme = get_cached_theme();
    let trigger = match spec.trigger {
        ContextPickerPopupTrigger::Mention => "@",
        ContextPickerPopupTrigger::Slash => "/",
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child("ACP scene"),
        )
        .child(
            div()
                .rounded(px(8.0))
                .bg(theme.colors.background.title_bar.to_rgb())
                .px(px(12.0))
                .py(px(10.0))
                .text_sm()
                .text_color(theme.colors.text.secondary.to_rgb())
                .child("Explain why this bug reproduces only on macOS."),
        )
        .child(
            div()
                .rounded(px(8.0))
                .bg(theme.colors.background.search_box.to_rgb())
                .px(px(12.0))
                .py(px(10.0))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.colors.text.primary.to_rgb())
                                .child("Use "),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_family(FONT_MONO)
                                .text_color(theme.colors.accent.selected.to_rgb())
                                .child(format!("{trigger}{}", spec.query)),
                        )
                        .child(
                            div()
                                .w(px(1.5))
                                .h(px(14.0))
                                .bg(theme.colors.accent.selected.to_rgb()),
                        ),
                ),
        )
        .into_any_element()
}

fn render_dropdown(spec: ContextPickerPopupPlaygroundSpec) -> AnyElement {
    let theme = get_cached_theme();
    let colors = InlineDropdownColors::from_theme(&theme);

    let rows = rows_for_trigger(spec.trigger);

    tracing::info!(
        event = "context_picker_popup_playground_rows_built",
        variant_id = spec.id.as_str(),
        row_count = rows.len(),
        grouped = spec.show_sections,
        "Built picker playground rows"
    );

    let mut children: Vec<AnyElement> = Vec::new();
    let mut last_section: Option<&str> = None;

    for row in rows.iter() {
        if spec.show_sections && last_section != Some(row.section) {
            last_section = Some(row.section);
            children.push(render_section_header(row.section));
        }

        children.push(
            render_dense_monoline_picker_row(
                SharedString::from(row.id),
                SharedString::from(row.label),
                SharedString::from(row.meta),
                &highlight_indices(row.label, spec.query),
                &highlight_indices(row.meta, spec.query),
                row.selected,
                colors.foreground,
                colors.muted_foreground,
            )
            .into_any_element(),
        );
    }

    let body = div().w_full().flex().flex_col().children(children);

    let selected_row = rows.iter().find(|r| r.selected).unwrap_or(&rows[0]);
    let synopsis = spec.show_synopsis.then(|| InlineDropdownSynopsis {
        label: SharedString::from(selected_row.label),
        meta: SharedString::from(selected_row.meta),
        description: SharedString::from(match spec.trigger {
            ContextPickerPopupTrigger::Mention => "Attach this context to the next message.",
            ContextPickerPopupTrigger::Slash => "Insert this command into the composer.",
        }),
    });

    InlineDropdown::new(
        SharedString::from("context-picker-playground"),
        body.into_any_element(),
        colors,
    )
    .vertical_padding(3.0)
    .synopsis(synopsis)
    .into_any_element()
}

fn render_section_header(label: &str) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .px(px(8.0))
        .pt(px(6.0))
        .pb(px(2.0))
        .text_xs()
        .font_family(FONT_MONO)
        .text_color(theme.colors.text.dimmed.with_opacity(0.55))
        .child(label.to_uppercase())
        .into_any_element()
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        context_picker_popup_playground_story_variants,
        resolve_context_picker_popup_playground_selection, ContextPickerPopupPlaygroundId,
    };
    use std::collections::HashSet;

    #[test]
    fn context_picker_popup_playground_variant_ids_are_unique() {
        let ids: HashSet<_> = context_picker_popup_playground_story_variants()
            .into_iter()
            .map(|v| v.stable_id())
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
    fn resolve_slash_grouped_variant_no_fallback() {
        let (selection, resolution) =
            resolve_context_picker_popup_playground_selection(Some("slash-grouped"));
        assert_eq!(selection.id, ContextPickerPopupPlaygroundId::SlashGrouped);
        assert_eq!(resolution.resolved_variant_id, "slash-grouped");
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_mention_grouped_variant_no_fallback() {
        let (selection, resolution) =
            resolve_context_picker_popup_playground_selection(Some("mention-grouped"));
        assert_eq!(selection.id, ContextPickerPopupPlaygroundId::MentionGrouped);
        assert_eq!(resolution.resolved_variant_id, "mention-grouped");
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_unknown_variant_uses_fallback() {
        let (selection, resolution) =
            resolve_context_picker_popup_playground_selection(Some("nonexistent"));
        assert_eq!(selection.id, ContextPickerPopupPlaygroundId::MentionDense);
        assert_eq!(resolution.resolved_variant_id, "mention-dense");
        assert!(resolution.fallback_used);
    }

    #[test]
    fn resolve_none_defaults_to_mention_dense() {
        let (selection, resolution) = resolve_context_picker_popup_playground_selection(None);
        assert_eq!(selection.id, ContextPickerPopupPlaygroundId::MentionDense);
        assert_eq!(resolution.resolved_variant_id, "mention-dense");
        assert!(!resolution.fallback_used);
    }
}
