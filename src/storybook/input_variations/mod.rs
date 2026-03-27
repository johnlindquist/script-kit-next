//! Input variation registry for the design explorer compare mode.
//!
//! Mirrors the footer variation pattern: stable IDs, declarative specs,
//! semantic props, and a preview renderer that storybook can display
//! side-by-side for selection and adoption.

use gpui::*;

use crate::list_item::FONT_MONO;
use crate::ui_foundation::HexColorExt;

use super::StoryVariant;

/// Stable IDs for compare-ready input surface variations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputVariationId {
    Bare,
    Underline,
    Pill,
    SearchIcon,
    PromptPrefix,
}

impl InputVariationId {
    pub const ALL: [Self; 5] = [
        Self::Bare,
        Self::Underline,
        Self::Pill,
        Self::SearchIcon,
        Self::PromptPrefix,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::Underline => "underline",
            Self::Pill => "pill",
            Self::SearchIcon => "search-icon",
            Self::PromptPrefix => "prompt-prefix",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Bare => "Bare",
            Self::Underline => "Underline",
            Self::Pill => "Pill",
            Self::SearchIcon => "Search Icon",
            Self::PromptPrefix => "Prompt Prefix",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Bare => "Text-only prompt input with no chrome",
            Self::Underline => "Prompt input with a bottom border only",
            Self::Pill => "Rounded input with subtle fill and no heavy borders",
            Self::SearchIcon => "Search treatment with a leading magnifying glass",
            Self::PromptPrefix => "Terminal-style input with a prefixed > marker",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "bare" => Some(Self::Bare),
            "underline" => Some(Self::Underline),
            "pill" => Some(Self::Pill),
            "search-icon" => Some(Self::SearchIcon),
            "prompt-prefix" => Some(Self::PromptPrefix),
            _ => None,
        }
    }
}

/// Declarative spec for a single input variation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputVariationSpec {
    pub id: InputVariationId,
    pub placeholder: &'static str,
    pub helper_text: &'static str,
}

const INPUT_VARIATIONS: [InputVariationSpec; 5] = [
    InputVariationSpec {
        id: InputVariationId::Bare,
        placeholder: "Run Clipboard History",
        helper_text: "No container, just text + cursor",
    },
    InputVariationSpec {
        id: InputVariationId::Underline,
        placeholder: "Search scripts",
        helper_text: "One divider line anchors the field",
    },
    InputVariationSpec {
        id: InputVariationId::Pill,
        placeholder: "Open application",
        helper_text: "Rounded treatment with subtle elevation",
    },
    InputVariationSpec {
        id: InputVariationId::SearchIcon,
        placeholder: "Find command",
        helper_text: "Search affordance for broader discovery",
    },
    InputVariationSpec {
        id: InputVariationId::PromptPrefix,
        placeholder: "kit run clipboard-history",
        helper_text: "Terminal-inspired prompt prefix",
    },
];

/// Returns the full set of input variation specs (stable order).
pub fn input_variation_specs() -> &'static [InputVariationSpec] {
    &INPUT_VARIATIONS
}

/// Converts every spec into a `StoryVariant` with semantic props.
pub fn input_story_variants() -> Vec<StoryVariant> {
    input_variation_specs()
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "input")
                .with_prop("variantId", spec.id.as_str())
                .with_prop("placeholder", spec.placeholder)
                .with_prop("helperText", spec.helper_text)
        })
        .collect()
}

/// Renders a preview for the given stable ID inside a mock prompt shell.
///
/// Falls back to the first variation (Bare) if the ID is unknown.
pub fn render_input_story_preview(stable_id: &str) -> AnyElement {
    let spec = input_variation_specs()
        .iter()
        .find(|spec| spec.id.as_str() == stable_id)
        .copied()
        .unwrap_or(INPUT_VARIATIONS[0]);

    let field = render_input_field(spec);
    render_preview_shell("Sample prompt", field, spec.helper_text)
}

fn render_preview_shell(
    title: &'static str,
    field: AnyElement,
    helper_text: &'static str,
) -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w(px(420.))
        .h(px(180.))
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(theme.colors.ui.border.to_rgb())
        .bg(theme.colors.background.main.to_rgb())
        .overflow_hidden()
        .child(
            div()
                .flex_1()
                .min_h(px(0.))
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child(title),
                )
                .child(field)
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child(helper_text),
                ),
        )
        .into_any_element()
}

fn render_input_field(spec: InputVariationSpec) -> AnyElement {
    match spec.id {
        InputVariationId::Bare => render_bare_field(spec.placeholder),
        InputVariationId::Underline => render_underline_field(spec.placeholder),
        InputVariationId::Pill => render_pill_field(spec.placeholder),
        InputVariationId::SearchIcon => render_search_icon_field(spec.placeholder),
        InputVariationId::PromptPrefix => render_prompt_prefix_field(spec.placeholder),
    }
}

fn render_bare_field(placeholder: &'static str) -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w_full()
        .px(px(4.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_lg()
                .text_color(theme.colors.text.primary.to_rgb())
                .child(placeholder),
        )
        .child(render_cursor())
        .into_any_element()
}

fn render_underline_field(placeholder: &'static str) -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .px(px(4.))
                .py(px(8.))
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_lg()
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child(placeholder),
                )
                .child(render_cursor()),
        )
        .child(div().h(px(1.)).w_full().bg(theme.colors.ui.border.to_rgb()))
        .into_any_element()
}

fn render_pill_field(placeholder: &'static str) -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w_full()
        .px(px(14.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .bg(theme.colors.background.title_bar.to_rgb())
        .border_1()
        .border_color(theme.colors.ui.border.to_rgb())
        .rounded(px(18.))
        .child(
            div()
                .text_base()
                .text_color(theme.colors.text.primary.to_rgb())
                .child(placeholder),
        )
        .child(render_cursor())
        .into_any_element()
}

fn render_search_icon_field(placeholder: &'static str) -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w_full()
        .px(px(14.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .bg(theme.colors.background.title_bar.to_rgb())
        .border_1()
        .border_color(theme.colors.ui.border.to_rgb())
        .rounded(px(10.))
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("🔍"),
        )
        .child(
            div()
                .text_base()
                .text_color(theme.colors.text.primary.to_rgb())
                .child(placeholder),
        )
        .child(render_cursor())
        .into_any_element()
}

fn render_prompt_prefix_field(placeholder: &'static str) -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w_full()
        .px(px(4.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .font_family(FONT_MONO)
                .text_base()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(">"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_base()
                .text_color(theme.colors.text.primary.to_rgb())
                .child(placeholder),
        )
        .child(render_cursor())
        .into_any_element()
}

fn render_cursor() -> AnyElement {
    let theme = crate::theme::get_cached_theme();

    div()
        .w(px(1.5))
        .h(px(18.))
        .bg(theme.colors.accent.selected.to_rgb())
        .rounded(px(1.))
        .into_any_element()
}

/// Resolve a persisted storybook input selection value to an `InputVariationId`.
///
/// Falls back to `InputVariationId::Bare` when the value is `None` or unrecognised.
pub fn adopted_input_variation_id(selected: Option<&str>) -> InputVariationId {
    selected
        .and_then(InputVariationId::from_stable_id)
        .unwrap_or(InputVariationId::Bare)
}

/// Resolve the on-disk storybook input selection to an `InputVariationId`.
///
/// Reads the persisted `design-explorer-selections.json` for the
/// `"input-design-variations"` story and resolves via [`adopted_input_variation_id`].
pub fn adopted_input_variation() -> InputVariationId {
    let selected = super::load_selected_story_variant("input-design-variations");

    adopted_input_variation_id(selected.as_deref())
}

#[cfg(test)]
mod tests;
