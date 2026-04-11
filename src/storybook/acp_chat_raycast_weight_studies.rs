use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, StoryVariant};
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AcpPreviewState {
    Empty,
    Conversation,
    Tooling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChatRole {
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChatMessage {
    role: ChatRole,
    label: &'static str,
    body: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AcpChatWeightSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub description: &'static str,
    state: AcpPreviewState,
    input_weight: FontWeight,
    placeholder_weight: FontWeight,
    role_weight: FontWeight,
    user_weight: FontWeight,
    assistant_weight: FontWeight,
    tool_weight: FontWeight,
    plan_weight: FontWeight,
    toolbar_weight: FontWeight,
    metadata_weight: FontWeight,
    input_opacity: f32,
    metadata_opacity: f32,
    footer_opacity: f32,
}

const CONVERSATION_MESSAGES: &[ChatMessage] = &[
    ChatMessage { role: ChatRole::User, label: "You", body: "Rewrite the dropdown typography so the primary labels feel more Raycast-like." },
    ChatMessage { role: ChatRole::Assistant, label: "Assistant", body: "I’d keep the titles at regular or medium, make supporting metadata quieter, and rely on fill and spacing before semibold." },
    ChatMessage { role: ChatRole::Assistant, label: "Assistant", body: "The safest sweep is to compare a few selected-title weights without changing the shell proportions." },
];

const TOOLING_MESSAGES: &[ChatMessage] = &[
    ChatMessage { role: ChatRole::User, label: "You", body: "Implement 15 type-weight studies for the launcher and ACP chat." },
    ChatMessage { role: ChatRole::Assistant, label: "Assistant", body: "I’m wiring Storybook-only studies first so we can review the hierarchy before touching live surfaces." },
    ChatMessage { role: ChatRole::Tool, label: "Plan", body: "1. Add main-menu study page\n2. Add ACP study page\n3. Compile Storybook\n4. Launch for review" },
];

pub const SPECS: [AcpChatWeightSpec; 15] = [
    AcpChatWeightSpec { id: "raycast-balance", name: "Raycast Balance", family: "Balanced", description: "Medium composer, regular transcript copy, and subdued supporting labels.", state: AcpPreviewState::Conversation, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.56, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "quiet-composer", name: "Quiet Composer", family: "Balanced", description: "Lets the transcript carry the weight while the composer placeholder recedes.", state: AcpPreviewState::Conversation, input_weight: FontWeight::NORMAL, placeholder_weight: FontWeight::NORMAL, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.84, metadata_opacity: 0.54, footer_opacity: 0.42 },
    AcpChatWeightSpec { id: "role-forward", name: "Role Forward", family: "Balanced", description: "Promotes the tiny role labels so the turn structure is easier to parse at a glance.", state: AcpPreviewState::Conversation, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::SEMIBOLD, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.58, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "selected-medium-copy", name: "Selected Medium Copy", family: "Balanced", description: "Pushes both user and assistant copy to medium to test a firmer conversational texture.", state: AcpPreviewState::Conversation, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::MEDIUM, assistant_weight: FontWeight::MEDIUM, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.56, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "all-regular", name: "All Regular", family: "Balanced", description: "Uses opacity and layout instead of weight almost everywhere.", state: AcpPreviewState::Conversation, input_weight: FontWeight::NORMAL, placeholder_weight: FontWeight::NORMAL, role_weight: FontWeight::NORMAL, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::NORMAL, plan_weight: FontWeight::NORMAL, toolbar_weight: FontWeight::NORMAL, metadata_weight: FontWeight::NORMAL, input_opacity: 0.88, metadata_opacity: 0.52, footer_opacity: 0.40 },
    AcpChatWeightSpec { id: "tool-header-strong", name: "Tool Header Strong", family: "Structural", description: "Gives tool and plan surfaces more weight so they feel like structural blocks.", state: AcpPreviewState::Tooling, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::SEMIBOLD, plan_weight: FontWeight::SEMIBOLD, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::MEDIUM, input_opacity: 0.92, metadata_opacity: 0.60, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "plan-quiet", name: "Plan Quiet", family: "Structural", description: "Keeps the plan strip useful but typographically recessed.", state: AcpPreviewState::Tooling, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::NORMAL, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.52, footer_opacity: 0.42 },
    AcpChatWeightSpec { id: "toolbar-firmer", name: "Toolbar Firmer", family: "Structural", description: "Strengthens the bottom hint strip to feel more like Raycast’s explicit control language.", state: AcpPreviewState::Conversation, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::SEMIBOLD, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.56, footer_opacity: 0.52 },
    AcpChatWeightSpec { id: "metadata-split", name: "Metadata Split", family: "Structural", description: "Keeps role labels regular but makes small support surfaces medium.", state: AcpPreviewState::Tooling, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::NORMAL, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::MEDIUM, input_opacity: 0.92, metadata_opacity: 0.58, footer_opacity: 0.42 },
    AcpChatWeightSpec { id: "empty-state-raycast", name: "Empty State Raycast", family: "Structural", description: "Tests Raycast-like weight hierarchy on the empty composer and centered hints.", state: AcpPreviewState::Empty, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.90, metadata_opacity: 0.52, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "input-forward", name: "Input Forward", family: "Primary", description: "Lets the composer lead the surface with a stronger weight than the transcript.", state: AcpPreviewState::Conversation, input_weight: FontWeight::SEMIBOLD, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.96, metadata_opacity: 0.56, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "assistant-forward", name: "Assistant Forward", family: "Primary", description: "Gives assistant responses a medium weight while user prompts stay regular.", state: AcpPreviewState::Conversation, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::MEDIUM, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.56, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "user-forward", name: "User Forward", family: "Primary", description: "Gives user prompts more weight so the transcript feels more command-oriented.", state: AcpPreviewState::Conversation, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::MEDIUM, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.92, metadata_opacity: 0.56, footer_opacity: 0.44 },
    AcpChatWeightSpec { id: "raycast-crisp-chat", name: "Raycast Crisp Chat", family: "Primary", description: "The sharpest Raycast-like hierarchy: medium composer, regular copy, semibold plan title only.", state: AcpPreviewState::Tooling, input_weight: FontWeight::MEDIUM, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::SEMIBOLD, toolbar_weight: FontWeight::MEDIUM, metadata_weight: FontWeight::NORMAL, input_opacity: 0.94, metadata_opacity: 0.56, footer_opacity: 0.46 },
    AcpChatWeightSpec { id: "semibold-edges", name: "Semibold Edges", family: "Primary", description: "Uses semibold only at the composer and footer edges to see if the frame feels tighter.", state: AcpPreviewState::Empty, input_weight: FontWeight::SEMIBOLD, placeholder_weight: FontWeight::MEDIUM, role_weight: FontWeight::MEDIUM, user_weight: FontWeight::NORMAL, assistant_weight: FontWeight::NORMAL, tool_weight: FontWeight::MEDIUM, plan_weight: FontWeight::MEDIUM, toolbar_weight: FontWeight::SEMIBOLD, metadata_weight: FontWeight::NORMAL, input_opacity: 0.96, metadata_opacity: 0.54, footer_opacity: 0.50 },
];

pub fn acp_chat_raycast_weight_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id, spec.name)
                .description(spec.description)
                .with_prop("surface", "acpChat")
                .with_prop("family", spec.family)
                .with_prop("variantId", spec.id)
        })
        .collect()
}

pub fn render_acp_chat_raycast_weight_story_preview(stable_id: &str) -> AnyElement {
    render_spec_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), false)
}

pub fn render_acp_chat_raycast_weight_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_spec_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), true)
}

pub fn render_acp_chat_raycast_weight_gallery() -> AnyElement {
    let theme = get_cached_theme();
    let mut root = story_container().gap_6().child(
        div().flex().flex_col().gap_1()
            .child(div().text_sm().text_color(theme.colors.text.tertiary.to_rgb()).child("ACP Chat"))
            .child(div().text_xs().text_color(theme.colors.text.muted.to_rgb()).child(
                "Fifteen full-surface ACP Chat studies that keep the existing shell and message rhythm but push the weight hierarchy toward Raycast.",
            )),
    );
    for family in ["Balanced", "Structural", "Primary"] {
        let mut section = story_section(family).gap(px(12.0));
        for spec in SPECS.iter().copied().filter(|spec| spec.family == family) {
            section = section.child(render_gallery_item(spec));
        }
        root = root.child(section);
    }
    root.into_any_element()
}

fn resolve_spec(stable_id: &str) -> Option<AcpChatWeightSpec> {
    SPECS.iter().copied().find(|spec| spec.id == stable_id)
}

fn render_gallery_item(spec: AcpChatWeightSpec) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .bg(theme.colors.background.title_bar.with_opacity(0.22))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .rounded(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child(spec.name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.colors.text.muted.to_rgb())
                        .child(spec.description),
                ),
        )
        .child(render_spec_stage(spec, false))
        .into_any_element()
}

fn render_spec_stage(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 360.0 } else { 560.0 };
    let height = if compact { 250.0 } else { 392.0 };
    div()
        .w(px(width))
        .h(px(height))
        .bg(theme.colors.background.main.with_opacity(0.28))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.14))
        .rounded(px(14.0))
        .overflow_hidden()
        .child(render_chat_shell(spec, compact))
        .into_any_element()
}

fn render_chat_shell(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(theme.colors.background.main.to_rgb())
        .child(render_chat_header(spec, compact))
        .child(render_chat_body(spec, compact))
        .when(matches!(spec.state, AcpPreviewState::Tooling), |d| {
            d.child(render_plan_strip(spec, compact))
        })
        .child(render_toolbar(spec, compact))
        .into_any_element()
}

fn render_chat_header(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let input_size = if compact { 15.0 } else { 17.0 };
    let right_size = if compact { 11.0 } else { 12.0 };
    let input_text = match spec.state {
        AcpPreviewState::Empty => "",
        AcpPreviewState::Conversation => {
            "Tighten the launcher font weights to feel more like Raycast."
        }
        AcpPreviewState::Tooling => "Build font-weight studies for Storybook first.",
    };
    div()
        .w_full()
        .px(px(12.0))
        .py(px(10.0))
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .flex_1()
                .min_h(px(22.0))
                .text_size(px(input_size))
                .font_weight(if input_text.is_empty() {
                    spec.placeholder_weight
                } else {
                    spec.input_weight
                })
                .text_color(if input_text.is_empty() {
                    theme.colors.text.muted.with_opacity(spec.metadata_opacity)
                } else {
                    theme.colors.text.primary.with_opacity(spec.input_opacity)
                })
                .child(if input_text.is_empty() {
                    SharedString::from("Ask anything…")
                } else {
                    SharedString::from(input_text)
                }),
        )
        .child(
            div()
                .text_size(px(right_size))
                .font_weight(spec.metadata_weight)
                .text_color(theme.colors.text.muted.with_opacity(0.62))
                .child("Claude Sonnet"),
        )
        .into_any_element()
}

fn render_chat_body(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    match spec.state {
        AcpPreviewState::Empty => render_empty_body(spec, compact),
        AcpPreviewState::Conversation => render_message_list(CONVERSATION_MESSAGES, spec, compact),
        AcpPreviewState::Tooling => render_message_list(TOOLING_MESSAGES, spec, compact),
    }
}

fn render_empty_body(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let text_size = if compact { 11.0 } else { 12.0 };
    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .text_size(px(text_size))
        .text_color(theme.colors.text.muted.with_opacity(spec.footer_opacity))
        .child(
            div()
                .font_weight(spec.metadata_weight)
                .child("Type / for skills"),
        )
        .child(
            div()
                .font_weight(spec.metadata_weight)
                .child("⇧↩ for newlines"),
        )
        .child(
            div()
                .font_weight(spec.metadata_weight)
                .child("⌘P history · ⌘K actions"),
        )
        .into_any_element()
}

fn render_message_list(
    messages: &[ChatMessage],
    spec: AcpChatWeightSpec,
    compact: bool,
) -> AnyElement {
    let mut column = div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .overflow_hidden()
        .px(px(8.0))
        .pb(px(8.0))
        .gap(px(if compact { 6.0 } else { 8.0 }));
    for message in messages.iter().copied() {
        column = column.child(render_message(message, spec, compact));
    }
    column.into_any_element()
}

fn render_message(message: ChatMessage, spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let body_weight = match message.role {
        ChatRole::User => spec.user_weight,
        ChatRole::Assistant => spec.assistant_weight,
        ChatRole::Tool => spec.tool_weight,
    };
    let body_bg = match message.role {
        ChatRole::User => theme.colors.accent.selected.with_opacity(0.10),
        ChatRole::Assistant => theme.colors.text.primary.with_opacity(0.04),
        ChatRole::Tool => theme.colors.accent.selected.with_opacity(0.06),
    };
    div()
        .w_full()
        .px(px(10.0))
        .py(px(if compact { 6.0 } else { 8.0 }))
        .bg(body_bg)
        .rounded(px(8.0))
        .border_l_2()
        .border_color(match message.role {
            ChatRole::User => theme.colors.accent.selected.with_opacity(0.50),
            ChatRole::Assistant => theme.colors.ui.border.with_opacity(0.28),
            ChatRole::Tool => theme.colors.accent.selected.with_opacity(0.28),
        })
        .child(
            div()
                .text_xs()
                .font_weight(spec.role_weight)
                .text_color(theme.colors.text.muted.with_opacity(spec.metadata_opacity))
                .child(message.label),
        )
        .child(
            div()
                .pt(px(4.0))
                .text_size(px(if compact { 13.0 } else { 14.0 }))
                .font_weight(body_weight)
                .line_height(px(if compact { 18.0 } else { 20.0 }))
                .text_color(theme.colors.text.primary.with_opacity(0.90))
                .child(message.body),
        )
        .into_any_element()
}

fn render_plan_strip(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div().mx(px(8.0)).mb(px(6.0)).px(px(12.0)).py(px(if compact { 6.0 } else { 8.0 }))
        .rounded(px(8.0)).bg(theme.colors.accent.selected.with_opacity(0.08))
        .border_1().border_color(theme.colors.accent.selected.with_opacity(0.22))
        .child(
            div().text_xs().font_weight(spec.plan_weight)
                .text_color(theme.colors.text.muted.with_opacity(spec.metadata_opacity))
                .child("Plan"),
        )
        .child(
            div().pt(px(4.0)).text_size(px(if compact { 12.0 } else { 13.0 }))
                .font_weight(spec.metadata_weight)
                .text_color(theme.colors.text.primary.with_opacity(0.82))
                .child("Audit the launcher weights, add a matching ACP study page, then compare before adoption."),
        )
        .into_any_element()
}

fn render_toolbar(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .h(px(if compact { 28.0 } else { 30.0 }))
        .px(px(12.0))
        .border_t_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(14.0))
        .font_family(FONT_MONO)
        .text_size(px(if compact { 10.0 } else { 11.0 }))
        .font_weight(spec.toolbar_weight)
        .text_color(theme.colors.text.muted.with_opacity(spec.footer_opacity))
        .child("⌘P history")
        .child("⌘K actions")
        .child("⌘N new")
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{acp_chat_raycast_weight_story_variants, SPECS};

    #[test]
    fn acp_chat_raycast_story_exposes_fifteen_variants() {
        assert_eq!(acp_chat_raycast_weight_story_variants().len(), 15);
        assert_eq!(SPECS.len(), 15);
    }
}
