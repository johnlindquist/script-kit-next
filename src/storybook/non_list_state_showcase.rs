//! Canonical non-list information-state language for Storybook.
//!
//! These fixtures are intentionally presenter-only. They let designers compare
//! empty/help/form/setup/permission/recovery/About-style layouts before live
//! product surfaces migrate to the shared helpers.

use gpui::{div, prelude::*, px, AnyElement, Div, FontWeight, Stateful};

use crate::components::{
    non_list_action_row, non_list_callout, non_list_card, non_list_centered_shell,
    non_list_content_stack, non_list_footer_note, non_list_icon_glyph, non_list_intro,
    non_list_metrics, non_list_palette, non_list_requirement_row, render_simple_hint_strip,
    template_prompt_hints, universal_prompt_hints, Button, ButtonColors, ButtonVariant,
    NonListDensity, NonListMetrics, NonListPalette,
};
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonListStateShowcaseId {
    Empty,
    Help,
    Form,
    Setup,
    Permission,
    Recovery,
    About,
    Density,
}

impl NonListStateShowcaseId {
    pub const ALL: [Self; 8] = [
        Self::Empty,
        Self::Help,
        Self::Form,
        Self::Setup,
        Self::Permission,
        Self::Recovery,
        Self::About,
        Self::Density,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Help => "help",
            Self::Form => "form",
            Self::Setup => "setup",
            Self::Permission => "permission",
            Self::Recovery => "recovery",
            Self::About => "about",
            Self::Density => "density",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Help => "Help",
            Self::Form => "Form",
            Self::Setup => "Setup",
            Self::Permission => "Permission",
            Self::Recovery => "Recovery",
            Self::About => "About",
            Self::Density => "Density",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Empty => "Centered one-message state with one next move.",
            Self::Help => "Anchored guidance with concise examples and truthful footer hints.",
            Self::Form => "Field-first information hierarchy for prompt and power-user forms.",
            Self::Setup => "Requirement checklist for agent or first-run readiness.",
            Self::Permission => "Plain-language scope and two-action decision card.",
            Self::Recovery => "Calm failure state with retry and escape hatch.",
            Self::About => "Branded product identity without dashboard chrome.",
            Self::Density => "Compact and comfortable density comparison.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "help" => Some(Self::Help),
            "form" => Some(Self::Form),
            "setup" => Some(Self::Setup),
            "permission" => Some(Self::Permission),
            "recovery" => Some(Self::Recovery),
            "about" => Some(Self::About),
            "density" => Some(Self::Density),
            _ => None,
        }
    }
}

pub fn non_list_state_showcase_story_variants() -> Vec<StoryVariant> {
    NonListStateShowcaseId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "nonListState")
                .with_prop("representation", "presenterFixture")
                .with_prop("layout", id.as_str())
        })
        .collect()
}

pub fn render_non_list_state_showcase_preview(stable_id: &str) -> AnyElement {
    let id =
        NonListStateShowcaseId::from_stable_id(stable_id).unwrap_or(NonListStateShowcaseId::Empty);
    render_showcase_state(id, false)
}

pub fn render_non_list_state_showcase_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id =
        NonListStateShowcaseId::from_stable_id(stable_id).unwrap_or(NonListStateShowcaseId::Empty);
    render_showcase_state(id, true)
}

fn render_showcase_state(id: NonListStateShowcaseId, compact: bool) -> AnyElement {
    match id {
        NonListStateShowcaseId::Empty => render_empty_state(compact),
        NonListStateShowcaseId::Help => render_help_state(compact),
        NonListStateShowcaseId::Form => render_form_state(compact),
        NonListStateShowcaseId::Setup => render_setup_state(compact),
        NonListStateShowcaseId::Permission => render_permission_state(compact),
        NonListStateShowcaseId::Recovery => render_recovery_state(compact),
        NonListStateShowcaseId::About => render_about_state(compact),
        NonListStateShowcaseId::Density => render_density_state(compact),
    }
}

fn stage(
    id: &'static str,
    content: impl IntoElement,
    footer: Option<AnyElement>,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let width = if compact { 440.0 } else { 680.0 };
    let height = if compact { 300.0 } else { 430.0 };

    let panel = div()
        .id(id)
        .w(px(width))
        .h(px(height))
        .rounded(px(10.0))
        .overflow_hidden()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface)
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .overflow_hidden()
                .child(content),
        );

    div()
        .w_full()
        .min_h(px(if compact { 320.0 } else { 470.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(if let Some(footer) = footer {
            panel.child(footer)
        } else {
            panel
        })
        .into_any_element()
}

fn buttons() -> ButtonColors {
    ButtonColors::from_theme(&get_cached_theme())
}

fn primary_button(label: &'static str, shortcut: &'static str) -> AnyElement {
    Button::new(label, buttons())
        .variant(ButtonVariant::Primary)
        .shortcut(shortcut)
        .into_any_element()
}

fn ghost_button(label: &'static str, shortcut: &'static str) -> AnyElement {
    Button::new(label, buttons())
        .variant(ButtonVariant::Ghost)
        .shortcut(shortcut)
        .into_any_element()
}

fn render_empty_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Compact);

    stage(
        "non-list-empty-stage",
        non_list_centered_shell("non-list-empty", metrics.max_width, metrics.block_gap)
            .child(non_list_icon_glyph("?", palette, metrics))
            .child(non_list_intro(
                "Start a conversation",
                "Ask a question, attach context, or open Actions for more ways to begin.",
                palette,
                metrics,
            ))
            .child(non_list_action_row(vec![primary_button("Ask", "enter")])),
        Some(render_simple_hint_strip(universal_prompt_hints(), None)),
        compact,
    )
}

fn render_help_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Compact);

    let examples = non_list_card("non-list-help-examples", palette, metrics)
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(example_line(
            ";note",
            "Capture a note with tags and due date",
            palette,
        ))
        .child(example_line(
            ";todo",
            "Create a task from structured fields",
            palette,
        ))
        .child(example_line(
            "@file",
            "Attach a file or folder as context",
            palette,
        ));

    stage(
        "non-list-help-stage",
        div()
            .size_full()
            .px(px(32.0))
            .py(px(28.0))
            .child(
                non_list_content_stack("non-list-help", metrics.max_width, metrics.block_gap)
                    .child(non_list_intro(
                        "Power syntax help",
                        "Use short commands when you know them, or tab through fields when you want structure.",
                        palette,
                        metrics,
                    ))
                    .child(examples)
                    .child(non_list_footer_note(
                        "Keep help anchored near the input; long documentation belongs in Actions.",
                        palette,
                    )),
            ),
        Some(render_simple_hint_strip(universal_prompt_hints(), None)),
        compact,
    )
}

fn example_line(code: &'static str, text: &'static str, palette: NonListPalette) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .w(px(56.0))
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.accent)
                .child(code),
        )
        .child(
            div()
                .flex_1()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .text_color(palette.body)
                .child(text),
        )
}

fn render_form_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Compact);

    let form = non_list_content_stack("non-list-form", metrics.max_width, metrics.block_gap)
        .child(non_list_intro(
            "Create task",
            "Forms are field-first. Labels stay visible and helper text explains only the current choice.",
            palette,
            metrics,
        ))
        .child(field_block("Title", "Prepare release checklist", "Required", palette, metrics))
        .child(field_block("Project", "Script Kit GPUI", "Autocomplete accepts known projects", palette, metrics))
        .child(non_list_callout(
            "non-list-form-callout",
            "Validation",
            "Show one specific recovery sentence near the affected field.",
            palette,
            metrics,
        ));

    stage(
        "non-list-form-stage",
        div().size_full().px(px(32.0)).py(px(24.0)).child(form),
        Some(render_simple_hint_strip(template_prompt_hints(), None)),
        compact,
    )
}

fn field_block(
    label: &'static str,
    value: &'static str,
    help: &'static str,
    palette: NonListPalette,
    metrics: NonListMetrics,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.hint)
                .child(label),
        )
        .child(
            non_list_card("non-list-field", palette, metrics)
                .bg(palette.input)
                .child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .text_color(palette.title)
                        .child(value),
                ),
        )
        .child(non_list_footer_note(help, palette))
}

fn render_setup_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Comfortable);

    let requirements = non_list_card("non-list-setup-card", palette, metrics)
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(non_list_requirement_row("Agent binary", "Found", palette))
        .child(non_list_requirement_row(
            "Authentication",
            "Needs sign in",
            palette,
        ))
        .child(non_list_requirement_row(
            "Workspace trust",
            "Ready",
            palette,
        ));

    stage(
        "non-list-setup-stage",
        div().size_full().px(px(32.0)).py(px(28.0)).child(
            non_list_content_stack("non-list-setup", metrics.max_width, metrics.block_gap)
                .child(non_list_intro(
                    "Connect an agent",
                    "Setup states show requirements before actions so the next move is obvious.",
                    palette,
                    metrics,
                ))
                .child(requirements)
                .child(non_list_action_row(vec![
                    primary_button("Sign In", "enter"),
                    ghost_button("Open Logs", "cmd+k"),
                ])),
        ),
        Some(render_simple_hint_strip(universal_prompt_hints(), None)),
        compact,
    )
}

fn render_permission_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Compact);

    stage(
        "non-list-permission-stage",
        div().size_full().px(px(32.0)).py(px(30.0)).child(
            non_list_content_stack("non-list-permission", metrics.max_width, metrics.block_gap)
                .child(non_list_intro(
                    "Allow file edit?",
                    "The agent wants to modify one file in this workspace.",
                    palette,
                    metrics,
                ))
                .child(non_list_callout(
                    "non-list-permission-scope",
                    "Scope",
                    "src/components/non_list_state.rs. No network request or shell command is part of this action.",
                    palette,
                    metrics,
                ))
                .child(non_list_action_row(vec![
                    primary_button("Allow", "enter"),
                    ghost_button("Deny", "esc"),
                ])),
        ),
        Some(render_simple_hint_strip(vec!["enter Allow".into(), "esc Deny".into()], None)),
        compact,
    )
}

fn render_recovery_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Compact);

    stage(
        "non-list-recovery-stage",
        div().size_full().px(px(32.0)).py(px(30.0)).child(
            non_list_content_stack("non-list-recovery", metrics.max_width, metrics.block_gap)
                .child(non_list_icon_glyph("!", palette, metrics))
                .child(non_list_intro(
                    "Update check failed",
                    "Script Kit could not reach the release server. Try again or open releases in the browser.",
                    palette,
                    metrics,
                ))
                .child(non_list_footer_note(
                    "Technical detail stays short and local to the recovery action.",
                    palette,
                ))
                .child(non_list_action_row(vec![
                    primary_button("Try Again", "enter"),
                    ghost_button("Open Releases", "cmd+k"),
                ])),
        ),
        Some(render_simple_hint_strip(universal_prompt_hints(), None)),
        compact,
    )
}

fn render_about_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);
    let metrics = non_list_metrics(NonListDensity::Comfortable);

    stage(
        "non-list-about-stage",
        div().size_full().px(px(32.0)).py(px(22.0)).child(
            non_list_content_stack("non-list-about", 560.0, 12.0)
                .items_center()
                .child(non_list_icon_glyph("SK", palette, metrics))
                .child(
                    div()
                        .text_size(px(28.0))
                        .line_height(px(34.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(palette.title)
                        .child("Script Kit"),
                )
                .child(
                    div()
                        .max_w(px(440.0))
                        .text_center()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .text_color(palette.body)
                        .child("A keyboard-first launcher for running scripts, agents, and workflows."),
                )
                .child(non_list_action_row(vec![
                    ghost_button("GitHub", "cmd+1"),
                    ghost_button("Discord", "cmd+2"),
                    ghost_button("Updates", "cmd+3"),
                ]))
                .child(non_list_callout(
                    "non-list-about-update",
                    "Version",
                    "Branded states can use a larger title, but the rest of the hierarchy stays compact.",
                    palette,
                    metrics,
                )),
        ),
        None,
        compact,
    )
}

fn render_density_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = non_list_palette(&theme);

    let compact_metrics = non_list_metrics(NonListDensity::Compact);
    let comfortable_metrics = non_list_metrics(NonListDensity::Comfortable);

    let content = div()
        .size_full()
        .px(px(24.0))
        .py(px(24.0))
        .flex()
        .flex_row()
        .gap(px(12.0))
        .child(density_card(
            "Compact",
            "Prompts, forms, inline recovery.",
            palette,
            compact_metrics,
        ))
        .child(density_card(
            "Comfortable",
            "About, setup, permission explanations.",
            palette,
            comfortable_metrics,
        ));

    stage("non-list-density-stage", content, None, compact)
}

fn density_card(
    title: &'static str,
    body: &'static str,
    palette: NonListPalette,
    metrics: NonListMetrics,
) -> Stateful<Div> {
    non_list_card("non-list-density-card", palette, metrics)
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(metrics.item_gap))
        .child(non_list_intro(title, body, palette, metrics))
        .child(non_list_requirement_row(
            "Max width",
            format!("{}px", metrics.max_width),
            palette,
        ))
        .child(non_list_requirement_row(
            "Title",
            format!("{} / {}", metrics.title_size, metrics.title_line),
            palette,
        ))
        .child(non_list_requirement_row(
            "Card padding",
            format!("{} x {}", metrics.card_padding_x, metrics.card_padding_y),
            palette,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_list_showcase_covers_required_layouts() {
        let ids: Vec<_> = non_list_state_showcase_story_variants()
            .into_iter()
            .map(|variant| variant.stable_id())
            .collect();

        for required in [
            "empty",
            "help",
            "form",
            "setup",
            "permission",
            "recovery",
            "about",
            "density",
        ] {
            assert!(
                ids.iter().any(|id| id == required),
                "missing non-list showcase layout {required}"
            );
        }
    }

    #[test]
    fn non_list_showcase_variants_are_presenter_fixtures() {
        for variant in non_list_state_showcase_story_variants() {
            assert_eq!(
                variant.props.get("surface").map(String::as_str),
                Some("nonListState")
            );
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("presenterFixture")
            );
            assert!(variant.props.contains_key("layout"));
        }
    }
}
