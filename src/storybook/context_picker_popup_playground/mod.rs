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
    config_from_storybook_footer_selection_value, FooterVariationId, IntegratedOverlayAnchor,
    IntegratedOverlayPlacement, IntegratedSurfaceShell, IntegratedSurfaceShellConfig, StoryVariant,
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

// ---------------------------------------------------------------------------
// Trigger & Spec
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPickerPopupTrigger {
    Mention,
    Slash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextPickerPopupPlaygroundSpec {
    pub id: ContextPickerPopupPlaygroundId,
    pub trigger: ContextPickerPopupTrigger,
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
        query: "scr",
        show_sections: false,
        show_synopsis: true,
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::MentionGrouped,
        trigger: ContextPickerPopupTrigger::Mention,
        query: "git",
        show_sections: true,
        show_synopsis: true,
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashDense,
        trigger: ContextPickerPopupTrigger::Slash,
        query: "con",
        show_sections: false,
        show_synopsis: true,
    },
    ContextPickerPopupPlaygroundSpec {
        id: ContextPickerPopupPlaygroundId::SlashGrouped,
        trigger: ContextPickerPopupTrigger::Slash,
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
    let spec = SPECS
        .iter()
        .find(|s| s.id.as_str() == stable_id)
        .copied()
        .unwrap_or(SPECS[0]);

    tracing::info!(
        event = "context_picker_popup_playground_rendered",
        variant_id = spec.id.as_str(),
        trigger = match spec.trigger {
            ContextPickerPopupTrigger::Mention => "mention",
            ContextPickerPopupTrigger::Slash => "slash",
        },
        grouped = spec.show_sections,
        "Rendered context picker popup playground preview"
    );

    IntegratedSurfaceShell::new(
        IntegratedSurfaceShellConfig {
            width: 560.0,
            height: 300.0,
            ..Default::default()
        },
        render_chat_body(spec),
    )
    .footer(render_footer())
    .overlay(
        IntegratedOverlayPlacement::new(
            IntegratedOverlayAnchor::Composer,
            overlay_left(spec.trigger),
            118.0,
            340.0,
        ),
        render_dropdown(spec),
    )
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

fn overlay_left(trigger: ContextPickerPopupTrigger) -> f32 {
    match trigger {
        ContextPickerPopupTrigger::Mention => 92.0,
        ContextPickerPopupTrigger::Slash => 76.0,
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
    use super::{context_picker_popup_playground_story_variants, ContextPickerPopupPlaygroundId};
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
}
