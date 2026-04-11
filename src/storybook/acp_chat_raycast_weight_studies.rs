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
struct AcpChatWeightSpec {
    id: &'static str,
    name: &'static str,
    family: &'static str,
    description: &'static str,
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
    header_height: f32,
    composer_padding_y: f32,
    input_size: f32,
    model_size: f32,
    empty_hint_size: f32,
    role_size: f32,
    body_size: f32,
    body_line_height: f32,
    plan_title_size: f32,
    plan_body_size: f32,
    plan_line_height: f32,
    toolbar_size: f32,
    toolbar_height: f32,
    message_padding_y: f32,
    bubble_radius: f32,
    message_gap: f32,
    plan_padding_y: f32,
    input_opacity: f32,
    metadata_opacity: f32,
    footer_opacity: f32,
}

const CONVERSATION_MESSAGES: &[ChatMessage] = &[
    ChatMessage {
        role: ChatRole::User,
        label: "You",
        body: "Rewrite the dropdown typography so the primary labels feel more Raycast-like.",
    },
    ChatMessage {
        role: ChatRole::Assistant,
        label: "Assistant",
        body: "I’d keep the titles at regular or medium, make supporting metadata quieter, and rely on fill and spacing before semibold.",
    },
    ChatMessage {
        role: ChatRole::Assistant,
        label: "Assistant",
        body: "The safest sweep is to compare a few selected-title weights without changing the shell proportions.",
    },
];

const TOOLING_MESSAGES: &[ChatMessage] = &[
    ChatMessage {
        role: ChatRole::User,
        label: "You",
        body: "Implement 15 type-weight studies for the launcher and ACP chat.",
    },
    ChatMessage {
        role: ChatRole::Assistant,
        label: "Assistant",
        body: "I’m wiring Storybook-only studies first so we can review the hierarchy before touching live surfaces.",
    },
    ChatMessage {
        role: ChatRole::Tool,
        label: "Plan",
        body: "1. Add main-menu study page\n2. Add ACP study page\n3. Compile Storybook\n4. Launch for review",
    },
];

const SPECS: [AcpChatWeightSpec; 15] = [
    AcpChatWeightSpec {
        id: "raycast-balance",
        name: "Raycast Balance",
        family: "Balanced",
        description: "Medium composer, regular transcript copy, and subdued supporting labels with the baseline chat rhythm.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 46.0,
        composer_padding_y: 10.0,
        input_size: 17.0,
        model_size: 12.0,
        empty_hint_size: 12.0,
        role_size: 11.0,
        body_size: 14.0,
        body_line_height: 20.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 8.0,
        bubble_radius: 8.0,
        message_gap: 8.0,
        plan_padding_y: 8.0,
        input_opacity: 0.92,
        metadata_opacity: 0.56,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "quiet-composer",
        name: "Quiet Composer",
        family: "Balanced",
        description: "Smaller composer text, denser message spacing, and lighter support surfaces so the transcript carries the scan path.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::NORMAL,
        placeholder_weight: FontWeight::NORMAL,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 42.0,
        composer_padding_y: 8.5,
        input_size: 15.8,
        model_size: 11.0,
        empty_hint_size: 11.5,
        role_size: 10.5,
        body_size: 13.4,
        body_line_height: 19.0,
        plan_title_size: 10.5,
        plan_body_size: 12.4,
        plan_line_height: 17.5,
        toolbar_size: 10.5,
        toolbar_height: 28.0,
        message_padding_y: 7.0,
        bubble_radius: 7.0,
        message_gap: 6.0,
        plan_padding_y: 7.0,
        input_opacity: 0.84,
        metadata_opacity: 0.54,
        footer_opacity: 0.42,
    },
    AcpChatWeightSpec {
        id: "role-forward",
        name: "Role Forward",
        family: "Balanced",
        description: "Promotes the role labels and opens vertical rhythm slightly so turn ownership becomes easier to parse.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::SEMIBOLD,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 45.0,
        composer_padding_y: 9.5,
        input_size: 16.6,
        model_size: 11.4,
        empty_hint_size: 12.0,
        role_size: 12.0,
        body_size: 14.0,
        body_line_height: 20.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 8.0,
        bubble_radius: 8.0,
        message_gap: 9.0,
        plan_padding_y: 8.0,
        input_opacity: 0.92,
        metadata_opacity: 0.58,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "selected-medium-copy",
        name: "Selected Medium Copy",
        family: "Balanced",
        description: "Pushes message copy larger and firmer with taller bubbles to test a more editorial conversation texture.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::MEDIUM,
        assistant_weight: FontWeight::MEDIUM,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 48.0,
        composer_padding_y: 10.5,
        input_size: 17.2,
        model_size: 12.0,
        empty_hint_size: 12.0,
        role_size: 11.0,
        body_size: 15.0,
        body_line_height: 21.5,
        plan_title_size: 11.2,
        plan_body_size: 13.6,
        plan_line_height: 19.5,
        toolbar_size: 11.0,
        toolbar_height: 31.0,
        message_padding_y: 9.5,
        bubble_radius: 9.0,
        message_gap: 9.0,
        plan_padding_y: 8.5,
        input_opacity: 0.92,
        metadata_opacity: 0.56,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "all-regular",
        name: "All Regular",
        family: "Balanced",
        description: "Uses size, density, and opacity rather than weight almost everywhere, with the flattest overall hierarchy.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::NORMAL,
        placeholder_weight: FontWeight::NORMAL,
        role_weight: FontWeight::NORMAL,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::NORMAL,
        plan_weight: FontWeight::NORMAL,
        toolbar_weight: FontWeight::NORMAL,
        metadata_weight: FontWeight::NORMAL,
        header_height: 41.0,
        composer_padding_y: 8.0,
        input_size: 15.6,
        model_size: 10.8,
        empty_hint_size: 11.0,
        role_size: 10.0,
        body_size: 13.0,
        body_line_height: 18.8,
        plan_title_size: 10.2,
        plan_body_size: 12.2,
        plan_line_height: 17.2,
        toolbar_size: 10.0,
        toolbar_height: 27.0,
        message_padding_y: 7.0,
        bubble_radius: 7.0,
        message_gap: 6.0,
        plan_padding_y: 6.8,
        input_opacity: 0.88,
        metadata_opacity: 0.52,
        footer_opacity: 0.40,
    },
    AcpChatWeightSpec {
        id: "tool-header-strong",
        name: "Tool Header Strong",
        family: "Structural",
        description: "Gives the tooling transcript and plan surfaces more scale and stronger structural labels.",
        state: AcpPreviewState::Tooling,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::SEMIBOLD,
        plan_weight: FontWeight::SEMIBOLD,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::MEDIUM,
        header_height: 46.0,
        composer_padding_y: 10.0,
        input_size: 17.0,
        model_size: 12.0,
        empty_hint_size: 12.0,
        role_size: 11.2,
        body_size: 14.2,
        body_line_height: 20.2,
        plan_title_size: 12.0,
        plan_body_size: 13.8,
        plan_line_height: 19.2,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 8.0,
        bubble_radius: 8.0,
        message_gap: 8.0,
        plan_padding_y: 9.0,
        input_opacity: 0.92,
        metadata_opacity: 0.60,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "plan-quiet",
        name: "Plan Quiet",
        family: "Structural",
        description: "Keeps the plan strip useful but typographically recessed, with a tighter and smaller support system.",
        state: AcpPreviewState::Tooling,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::NORMAL,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 44.0,
        composer_padding_y: 9.0,
        input_size: 16.6,
        model_size: 11.4,
        empty_hint_size: 11.5,
        role_size: 10.8,
        body_size: 13.8,
        body_line_height: 19.4,
        plan_title_size: 10.2,
        plan_body_size: 12.0,
        plan_line_height: 17.0,
        toolbar_size: 10.8,
        toolbar_height: 29.0,
        message_padding_y: 7.4,
        bubble_radius: 7.5,
        message_gap: 7.0,
        plan_padding_y: 6.8,
        input_opacity: 0.92,
        metadata_opacity: 0.52,
        footer_opacity: 0.42,
    },
    AcpChatWeightSpec {
        id: "toolbar-firmer",
        name: "Toolbar Firmer",
        family: "Structural",
        description: "Strengthens the bottom control strip with a larger type lockup and a taller footer frame.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::SEMIBOLD,
        metadata_weight: FontWeight::NORMAL,
        header_height: 45.0,
        composer_padding_y: 9.5,
        input_size: 16.8,
        model_size: 11.4,
        empty_hint_size: 12.0,
        role_size: 11.0,
        body_size: 13.8,
        body_line_height: 19.8,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 12.2,
        toolbar_height: 34.0,
        message_padding_y: 8.0,
        bubble_radius: 8.0,
        message_gap: 8.0,
        plan_padding_y: 8.0,
        input_opacity: 0.92,
        metadata_opacity: 0.56,
        footer_opacity: 0.52,
    },
    AcpChatWeightSpec {
        id: "metadata-split",
        name: "Metadata Split",
        family: "Structural",
        description: "Regular role labels but firmer small support surfaces, with a slightly smaller transcript body.",
        state: AcpPreviewState::Tooling,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::NORMAL,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::MEDIUM,
        header_height: 44.0,
        composer_padding_y: 9.0,
        input_size: 16.4,
        model_size: 11.8,
        empty_hint_size: 11.6,
        role_size: 10.2,
        body_size: 13.6,
        body_line_height: 19.2,
        plan_title_size: 11.0,
        plan_body_size: 12.8,
        plan_line_height: 18.0,
        toolbar_size: 10.8,
        toolbar_height: 29.0,
        message_padding_y: 7.4,
        bubble_radius: 7.5,
        message_gap: 7.0,
        plan_padding_y: 7.8,
        input_opacity: 0.92,
        metadata_opacity: 0.58,
        footer_opacity: 0.42,
    },
    AcpChatWeightSpec {
        id: "empty-state-raycast",
        name: "Empty State Raycast",
        family: "Structural",
        description: "Tests Raycast-like empty-state hierarchy with a larger composer and smaller centered support copy.",
        state: AcpPreviewState::Empty,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 47.0,
        composer_padding_y: 10.0,
        input_size: 17.4,
        model_size: 12.0,
        empty_hint_size: 12.4,
        role_size: 11.0,
        body_size: 14.0,
        body_line_height: 20.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 8.0,
        bubble_radius: 8.0,
        message_gap: 8.0,
        plan_padding_y: 8.0,
        input_opacity: 0.90,
        metadata_opacity: 0.52,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "input-forward",
        name: "Input Forward",
        family: "Primary",
        description: "Lets the composer lead the surface with a bigger entry line and a slightly larger shell frame.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::SEMIBOLD,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 49.0,
        composer_padding_y: 10.6,
        input_size: 18.4,
        model_size: 12.0,
        empty_hint_size: 12.2,
        role_size: 11.0,
        body_size: 14.0,
        body_line_height: 20.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 8.0,
        bubble_radius: 8.5,
        message_gap: 8.0,
        plan_padding_y: 8.0,
        input_opacity: 0.96,
        metadata_opacity: 0.56,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "assistant-forward",
        name: "Assistant Forward",
        family: "Primary",
        description: "Gives assistant responses more body size and a little more breathing room than user turns.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::MEDIUM,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 46.0,
        composer_padding_y: 9.8,
        input_size: 17.0,
        model_size: 11.8,
        empty_hint_size: 12.0,
        role_size: 11.0,
        body_size: 14.8,
        body_line_height: 21.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 9.0,
        bubble_radius: 8.5,
        message_gap: 9.0,
        plan_padding_y: 8.0,
        input_opacity: 0.92,
        metadata_opacity: 0.56,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "user-forward",
        name: "User Forward",
        family: "Primary",
        description: "Gives user turns more size and presence so the transcript reads as more command-oriented.",
        state: AcpPreviewState::Conversation,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::MEDIUM,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 46.0,
        composer_padding_y: 9.8,
        input_size: 17.0,
        model_size: 11.8,
        empty_hint_size: 12.0,
        role_size: 11.0,
        body_size: 14.8,
        body_line_height: 21.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 11.0,
        toolbar_height: 30.0,
        message_padding_y: 9.0,
        bubble_radius: 8.5,
        message_gap: 9.0,
        plan_padding_y: 8.0,
        input_opacity: 0.92,
        metadata_opacity: 0.56,
        footer_opacity: 0.44,
    },
    AcpChatWeightSpec {
        id: "raycast-crisp-chat",
        name: "Raycast Crisp Chat",
        family: "Primary",
        description: "A sharper Raycast-like hierarchy with compact copy, a firmer plan heading, and cleaner small-surface framing.",
        state: AcpPreviewState::Tooling,
        input_weight: FontWeight::MEDIUM,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::SEMIBOLD,
        toolbar_weight: FontWeight::MEDIUM,
        metadata_weight: FontWeight::NORMAL,
        header_height: 45.0,
        composer_padding_y: 9.4,
        input_size: 16.8,
        model_size: 11.5,
        empty_hint_size: 11.8,
        role_size: 10.8,
        body_size: 14.0,
        body_line_height: 20.0,
        plan_title_size: 12.0,
        plan_body_size: 13.0,
        plan_line_height: 18.2,
        toolbar_size: 11.4,
        toolbar_height: 31.0,
        message_padding_y: 7.8,
        bubble_radius: 7.8,
        message_gap: 7.5,
        plan_padding_y: 8.2,
        input_opacity: 0.94,
        metadata_opacity: 0.56,
        footer_opacity: 0.46,
    },
    AcpChatWeightSpec {
        id: "semibold-edges",
        name: "Semibold Edges",
        family: "Primary",
        description: "Uses semibold only at the composer and footer edges while keeping the center of the surface comparatively light.",
        state: AcpPreviewState::Empty,
        input_weight: FontWeight::SEMIBOLD,
        placeholder_weight: FontWeight::MEDIUM,
        role_weight: FontWeight::MEDIUM,
        user_weight: FontWeight::NORMAL,
        assistant_weight: FontWeight::NORMAL,
        tool_weight: FontWeight::MEDIUM,
        plan_weight: FontWeight::MEDIUM,
        toolbar_weight: FontWeight::SEMIBOLD,
        metadata_weight: FontWeight::NORMAL,
        header_height: 49.0,
        composer_padding_y: 10.8,
        input_size: 18.0,
        model_size: 12.0,
        empty_hint_size: 12.0,
        role_size: 11.0,
        body_size: 14.0,
        body_line_height: 20.0,
        plan_title_size: 11.0,
        plan_body_size: 13.0,
        plan_line_height: 18.5,
        toolbar_size: 12.0,
        toolbar_height: 34.0,
        message_padding_y: 8.0,
        bubble_radius: 8.5,
        message_gap: 8.0,
        plan_padding_y: 8.0,
        input_opacity: 0.96,
        metadata_opacity: 0.54,
        footer_opacity: 0.50,
    },
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
        div().flex().flex_col().gap_1().child(
            div()
                .text_sm()
                .text_color(theme.colors.text.tertiary.to_rgb())
                .child("ACP Chat"),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.colors.text.muted.to_rgb())
                .child(
                    "Fifteen full-surface ACP Chat studies with wider variation across font size, line-height, footer strength, and message density.",
                ),
        ),
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
    let input_text = match spec.state {
        AcpPreviewState::Empty => "",
        AcpPreviewState::Conversation => {
            "Tighten the launcher font weights to feel more like Raycast."
        }
        AcpPreviewState::Tooling => "Build font-weight studies for Storybook first.",
    };
    div()
        .w_full()
        .h(px(scale(spec.header_height, compact)))
        .px(px(scale(12.0, compact)))
        .py(px(scale(spec.composer_padding_y, compact)))
        .border_b_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(scale(10.0, compact)))
        .child(
            div()
                .flex_1()
                .min_h(px(scale(22.0, compact)))
                .text_size(px(scale(spec.input_size, compact)))
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
                .text_size(px(scale(spec.model_size, compact)))
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
    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(scale(6.0, compact)))
        .child(
            div()
                .text_size(px(scale(spec.empty_hint_size + 1.0, compact)))
                .font_weight(spec.metadata_weight)
                .text_color(theme.colors.text.primary.with_opacity(0.58))
                .child("Ask ACP Chat"),
        )
        .child(
            div()
                .text_size(px(scale(spec.empty_hint_size, compact)))
                .text_color(theme.colors.text.muted.with_opacity(spec.footer_opacity))
                .child("Type / for skills"),
        )
        .child(
            div()
                .text_size(px(scale(spec.empty_hint_size, compact)))
                .font_weight(spec.metadata_weight)
                .text_color(theme.colors.text.muted.with_opacity(spec.footer_opacity))
                .child("⇧↩ for newlines"),
        )
        .child(
            div()
                .text_size(px(scale(spec.empty_hint_size, compact)))
                .text_color(theme.colors.text.muted.with_opacity(spec.footer_opacity))
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
        .px(px(scale(8.0, compact)))
        .py(px(scale(8.0, compact)))
        .gap(px(scale(spec.message_gap, compact)));
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
        .px(px(scale(10.0, compact)))
        .py(px(scale(spec.message_padding_y, compact)))
        .bg(body_bg)
        .rounded(px(scale(spec.bubble_radius, compact)))
        .border_l_2()
        .border_color(match message.role {
            ChatRole::User => theme.colors.accent.selected.with_opacity(0.50),
            ChatRole::Assistant => theme.colors.ui.border.with_opacity(0.28),
            ChatRole::Tool => theme.colors.accent.selected.with_opacity(0.28),
        })
        .child(
            div()
                .text_size(px(scale(spec.role_size, compact)))
                .font_weight(spec.role_weight)
                .text_color(theme.colors.text.muted.with_opacity(spec.metadata_opacity))
                .child(message.label),
        )
        .child(
            div()
                .pt(px(scale(4.0, compact)))
                .text_size(px(scale(spec.body_size, compact)))
                .font_weight(body_weight)
                .line_height(px(scale(spec.body_line_height, compact)))
                .text_color(theme.colors.text.primary.with_opacity(0.90))
                .child(message.body),
        )
        .into_any_element()
}

fn render_plan_strip(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .mx(px(scale(8.0, compact)))
        .mb(px(scale(6.0, compact)))
        .px(px(scale(12.0, compact)))
        .py(px(scale(spec.plan_padding_y, compact)))
        .rounded(px(scale(spec.bubble_radius, compact)))
        .bg(theme.colors.accent.selected.with_opacity(0.08))
        .border_1()
        .border_color(theme.colors.accent.selected.with_opacity(0.22))
        .child(
            div()
                .text_size(px(scale(spec.plan_title_size, compact)))
                .font_weight(spec.plan_weight)
                .text_color(theme.colors.text.muted.with_opacity(spec.metadata_opacity))
                .child("Plan"),
        )
        .child(
            div()
                .pt(px(scale(4.0, compact)))
                .text_size(px(scale(spec.plan_body_size, compact)))
                .line_height(px(scale(spec.plan_line_height, compact)))
                .font_weight(spec.metadata_weight)
                .text_color(theme.colors.text.primary.with_opacity(0.82))
                .child(
                    "Audit the launcher weights, add a matching ACP study page, then compare before adoption.",
                ),
        )
        .into_any_element()
}

fn render_toolbar(spec: AcpChatWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .h(px(scale(spec.toolbar_height, compact)))
        .px(px(scale(12.0, compact)))
        .border_t_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(scale(14.0, compact)))
        .font_family(FONT_MONO)
        .text_size(px(scale(spec.toolbar_size, compact)))
        .font_weight(spec.toolbar_weight)
        .text_color(theme.colors.text.muted.with_opacity(spec.footer_opacity))
        .child("⌘P history")
        .child("⌘K actions")
        .child("⌘N new")
        .into_any_element()
}

fn scale(value: f32, compact: bool) -> f32 {
    if compact {
        value * 0.84
    } else {
        value
    }
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
