//! Canonical shared component primitives for Storybook.
//!
//! These fixtures render real shared components with deterministic data so
//! visual changes to foundational UI pieces are easy to inspect in one place.

use gpui::{div, prelude::*, px, rgba, AnyElement, FontWeight, SharedString};

use crate::components::{
    Button, ButtonColors, ButtonVariant, SectionDivider, Toast, ToastAction, ToastColors,
    ToastVariant,
};
use crate::list_item::{ListItem, ListItemColors};
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentPrimitiveStateId {
    ListItems,
    Buttons,
    HintStrips,
    Toasts,
    SectionDividers,
    PromptShell,
}

impl ComponentPrimitiveStateId {
    pub const ALL: [Self; 6] = [
        Self::ListItems,
        Self::Buttons,
        Self::HintStrips,
        Self::Toasts,
        Self::SectionDividers,
        Self::PromptShell,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ListItems => "list-items",
            Self::Buttons => "buttons",
            Self::HintStrips => "hint-strips",
            Self::Toasts => "toasts",
            Self::SectionDividers => "section-dividers",
            Self::PromptShell => "prompt-shell",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ListItems => "List Items",
            Self::Buttons => "Buttons",
            Self::HintStrips => "Hint Strips",
            Self::Toasts => "Toasts",
            Self::SectionDividers => "Section Dividers",
            Self::PromptShell => "Prompt Shell",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ListItems => {
                "Shared launcher row states with descriptions, badges, and shortcuts."
            }
            Self::Buttons => "Reusable button variants and disabled/loading/focus states.",
            Self::HintStrips => {
                "Footer hint-strip parsing, leading content, and clickable styling."
            }
            Self::Toasts => "Toast variants with details and action buttons.",
            Self::SectionDividers => "Whisper divider chrome used by minimal surfaces.",
            Self::PromptShell => "Minimal prompt shell scaffold with header, list, and footer.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "list-items" => Some(Self::ListItems),
            "buttons" => Some(Self::Buttons),
            "hint-strips" => Some(Self::HintStrips),
            "toasts" => Some(Self::Toasts),
            "section-dividers" => Some(Self::SectionDividers),
            "prompt-shell" => Some(Self::PromptShell),
            _ => None,
        }
    }
}

pub fn component_primitive_state_story_variants() -> Vec<StoryVariant> {
    ComponentPrimitiveStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "component")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_component_primitive_state_preview(stable_id: &str) -> AnyElement {
    let id = ComponentPrimitiveStateId::from_stable_id(stable_id)
        .unwrap_or(ComponentPrimitiveStateId::ListItems);
    render_component_primitive_state(id, false)
}

pub fn render_component_primitive_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = ComponentPrimitiveStateId::from_stable_id(stable_id)
        .unwrap_or(ComponentPrimitiveStateId::ListItems);
    render_component_primitive_state(id, true)
}

fn render_component_primitive_state(id: ComponentPrimitiveStateId, compact: bool) -> AnyElement {
    match id {
        ComponentPrimitiveStateId::ListItems => {
            render_panel(id, render_list_item_states(compact), compact)
        }
        ComponentPrimitiveStateId::Buttons => {
            render_panel(id, render_button_states(compact), compact)
        }
        ComponentPrimitiveStateId::HintStrips => {
            render_panel(id, render_hint_strip_states(compact), compact)
        }
        ComponentPrimitiveStateId::Toasts => {
            render_panel(id, render_toast_states(compact), compact)
        }
        ComponentPrimitiveStateId::SectionDividers => {
            render_panel(id, render_section_divider_states(compact), compact)
        }
        ComponentPrimitiveStateId::PromptShell => render_prompt_shell_state(compact),
    }
}

fn render_panel(
    id: ComponentPrimitiveStateId,
    content: impl IntoElement,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 500.0 } else { 760.0 };
    let min_height = if compact { 310.0 } else { 450.0 };

    div()
        .w_full()
        .min_h(px(if compact { 330.0 } else { 490.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .min_h(px(min_height))
                .rounded(px(10.0))
                .overflow_hidden()
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
                .bg(theme.colors.background.main.to_rgb())
                .flex()
                .flex_col()
                .child(render_panel_header(id, compact))
                .child(content),
        )
        .into_any_element()
}

fn render_panel_header(id: ComponentPrimitiveStateId, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .px(px(if compact { 14.0 } else { 18.0 }))
        .py(px(if compact { 10.0 } else { 14.0 }))
        .flex()
        .flex_col()
        .gap(px(3.0))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child(id.name()),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(id.description()),
        )
        .child(SectionDivider::new())
        .into_any_element()
}

fn render_list_item_states(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ListItemColors::from_theme(&theme);
    let rows = vec![
        ListItem::new("Selected launcher row", colors)
            .description("Description, accent bar, badge, and shortcut visible")
            .tool_badge("ts")
            .shortcut("enter")
            .selected(true)
            .with_accent_bar(true),
        ListItem::new("Hovered secondary row", colors)
            .description("Hover tint stays weaker than selected focus")
            .tool_badge("md")
            .shortcut("cmd+k")
            .hovered(true)
            .with_accent_bar(true),
        ListItem::new("Plain result row", colors)
            .description("Normal search result without active state")
            .tool_badge("sh")
            .with_accent_bar(true),
        ListItem::new("Long title is sanitized and constrained", colors)
            .description("Line breaks are collapsed before row rendering")
            .shortcut("cmd+enter")
            .with_accent_bar(true),
    ];

    div()
        .w_full()
        .px(px(if compact { 10.0 } else { 14.0 }))
        .py(px(if compact { 8.0 } else { 12.0 }))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .children(rows.into_iter().map(|row| div().w_full().child(row)))
        .into_any_element()
}

fn render_button_states(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ButtonColors::from_theme(&theme);

    div()
        .w_full()
        .px(px(if compact { 14.0 } else { 18.0 }))
        .py(px(if compact { 14.0 } else { 20.0 }))
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(button_group(
            "Variants",
            vec![
                Button::new("Run", colors)
                    .variant(ButtonVariant::Primary)
                    .shortcut("enter")
                    .into_any_element(),
                Button::new("Actions", colors)
                    .variant(ButtonVariant::Ghost)
                    .shortcut("cmd+k")
                    .into_any_element(),
                Button::new("K", colors)
                    .variant(ButtonVariant::Icon)
                    .into_any_element(),
            ],
        ))
        .child(button_group(
            "States",
            vec![
                Button::new("Focused", colors)
                    .variant(ButtonVariant::Ghost)
                    .focused(true)
                    .into_any_element(),
                Button::new("Disabled", colors)
                    .variant(ButtonVariant::Ghost)
                    .disabled(true)
                    .into_any_element(),
                Button::new("Save", colors)
                    .variant(ButtonVariant::Primary)
                    .loading(true)
                    .loading_label("Saving")
                    .into_any_element(),
            ],
        ))
        .into_any_element()
}

fn button_group(label: &'static str, buttons: Vec<AnyElement>) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.muted.to_rgb())
                .child(label),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(8.0))
                .children(buttons),
        )
        .into_any_element()
}

fn render_hint_strip_states(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let leading = div()
        .text_xs()
        .text_color(theme.colors.text.dimmed.to_rgb())
        .child("4 selected");

    div()
        .w_full()
        .px(px(if compact { 12.0 } else { 18.0 }))
        .py(px(if compact { 14.0 } else { 20.0 }))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            crate::components::hint_strip::HintStrip::new(vec![
                SharedString::from("enter Run"),
                SharedString::from("cmd+enter AI"),
                SharedString::from("cmd+k Actions"),
            ])
            .leading(leading),
        )
        .child(crate::components::hint_strip::HintStrip::new(vec![
            SharedString::from("esc Back"),
            SharedString::from("tab Next"),
            SharedString::from("shift+tab Previous"),
        ]))
        .child(
            crate::components::hint_strip::HintStrip::new(vec![
                SharedString::from("cmd+c Copy"),
                SharedString::from("cmd+shift+c Copy Markdown"),
            ])
            .on_hint_click(0, |_, _, _| {}),
        )
        .into_any_element()
}

fn render_toast_states(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let action = || ToastAction::new("Retry", Box::new(|_, _, _| {}));

    div()
        .w_full()
        .px(px(if compact { 12.0 } else { 18.0 }))
        .py(px(if compact { 10.0 } else { 16.0 }))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            Toast::new(
                "Storybook catalog rebuilt",
                ToastColors::from_theme(&theme, ToastVariant::Success),
            )
            .variant(ToastVariant::Success)
            .dismissible(false),
        )
        .child(
            Toast::new(
                "Runtime screenshot fixture ignored",
                ToastColors::from_theme(&theme, ToastVariant::Warning),
            )
            .variant(ToastVariant::Warning)
            .details("Registered stories use presenter fixtures instead.")
            .action(action()),
        )
        .child(
            Toast::new(
                "Failed to attach selected file",
                ToastColors::from_theme(&theme, ToastVariant::Error),
            )
            .variant(ToastVariant::Error)
            .persistent()
            .action(action()),
        )
        .child(
            Toast::new(
                "New ACP context available",
                ToastColors::from_theme(&theme, ToastVariant::Info),
            )
            .variant(ToastVariant::Info),
        )
        .into_any_element()
}

fn render_section_divider_states(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .px(px(if compact { 16.0 } else { 22.0 }))
        .py(px(if compact { 16.0 } else { 22.0 }))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(divider_sample("Launcher header", "Input and result rows"))
        .child(SectionDivider::new())
        .child(divider_sample(
            "Preview pane",
            "Details, metadata, and code snippets",
        ))
        .child(SectionDivider::new())
        .child(divider_sample(
            "Footer boundary",
            "Hint strip remains visually quiet",
        ))
        .child(
            div()
                .text_xs()
                .text_color(theme.colors.text.muted.to_rgb())
                .child("Dividers use the shared chrome opacity constants."),
        )
        .into_any_element()
}

fn divider_sample(title: &'static str, subtitle: &'static str) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child(title),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(subtitle),
        )
        .into_any_element()
}

fn render_prompt_shell_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ListItemColors::from_theme(&theme);
    let width = if compact { 500.0 } else { 760.0 };
    let height = if compact { 320.0 } else { 440.0 };
    let header = div()
        .flex_1()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_lg()
                .text_color(theme.colors.text.primary.to_rgb())
                .child("Script Kit"),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("3 results"),
        );
    let content = div()
        .w_full()
        .h_full()
        .py(px(4.0))
        .flex()
        .flex_col()
        .child(
            ListItem::new("Run focused script", colors)
                .description("Minimal shell keeps header, list, and footer stable")
                .shortcut("enter")
                .selected(true)
                .with_accent_bar(true),
        )
        .child(
            ListItem::new("Ask AI about selection", colors)
                .description("Uses the shared footer hint strip")
                .shortcut("cmd+enter")
                .with_accent_bar(true),
        )
        .child(
            ListItem::new("Open actions", colors)
                .description("Actions shortcut is present across prompt shells")
                .shortcut("cmd+k")
                .with_accent_bar(true),
        );

    let shell = crate::components::render_minimal_list_prompt_shell(
        10.0,
        None,
        header,
        content,
        vec![
            SharedString::from("enter Run"),
            SharedString::from("cmd+enter AI"),
            SharedString::from("cmd+k Actions"),
        ],
        None,
    );

    div()
        .w_full()
        .min_h(px(if compact { 340.0 } else { 480.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .h(px(height))
                .overflow_hidden()
                .child(shell),
        )
        .into_any_element()
}
