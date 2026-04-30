//! Presenter-backed Agent Chat states for Storybook.
//!
//! These fixtures mirror the live ACP chat surface without starting an agent,
//! opening popups, or depending on saved conversation history.

use gpui::{div, prelude::*, px, rgba, AnyElement, FontWeight};

use crate::list_item::FONT_MONO;
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcpChatStateId {
    Empty,
    Conversation,
    Streaming,
    ToolCall,
    Permission,
    Search,
    HistoryPopup,
    Setup,
}

impl AcpChatStateId {
    pub const ALL: [Self; 8] = [
        Self::Empty,
        Self::Conversation,
        Self::Streaming,
        Self::ToolCall,
        Self::Permission,
        Self::Search,
        Self::HistoryPopup,
        Self::Setup,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Conversation => "conversation",
            Self::Streaming => "streaming",
            Self::ToolCall => "tool-call",
            Self::Permission => "permission",
            Self::Search => "search",
            Self::HistoryPopup => "history-popup",
            Self::Setup => "setup",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Conversation => "Conversation",
            Self::Streaming => "Streaming",
            Self::ToolCall => "Tool Call",
            Self::Permission => "Permission",
            Self::Search => "Search",
            Self::HistoryPopup => "History Popup",
            Self::Setup => "Setup",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Empty => "No messages yet with composer placeholder and footer hints.",
            Self::Conversation => "Normal user and assistant transcript with footer controls.",
            Self::Streaming => "Assistant response actively streaming.",
            Self::ToolCall => "Collapsible tool/thinking block inside a conversation.",
            Self::Permission => "Inline permission request attached to a tool message.",
            Self::Search => "Conversation search bar with match counter.",
            Self::HistoryPopup => "Recent conversations popup opened from the footer.",
            Self::Setup => "Inline setup card when an ACP agent is not ready.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "conversation" => Some(Self::Conversation),
            "streaming" => Some(Self::Streaming),
            "tool-call" => Some(Self::ToolCall),
            "permission" => Some(Self::Permission),
            "search" => Some(Self::Search),
            "history-popup" => Some(Self::HistoryPopup),
            "setup" => Some(Self::Setup),
            _ => None,
        }
    }
}

pub fn acp_chat_state_story_variants() -> Vec<StoryVariant> {
    AcpChatStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "acpChat")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_acp_chat_state_preview(stable_id: &str) -> AnyElement {
    let id = AcpChatStateId::from_stable_id(stable_id).unwrap_or(AcpChatStateId::Conversation);
    render_acp_chat_state(id, false)
}

pub fn render_acp_chat_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = AcpChatStateId::from_stable_id(stable_id).unwrap_or(AcpChatStateId::Conversation);
    render_acp_chat_state(id, true)
}

fn render_acp_chat_state(id: AcpChatStateId, compact: bool) -> AnyElement {
    if id == AcpChatStateId::Setup {
        return render_setup_card(compact);
    }

    let theme = get_cached_theme();
    let width = if compact { 420.0 } else { 620.0 };
    let height = if compact { 300.0 } else { 420.0 };
    let composer = match id {
        AcpChatStateId::Empty => "",
        AcpChatStateId::Streaming => "Add visual state coverage for Agent Chat",
        AcpChatStateId::ToolCall | AcpChatStateId::Permission => {
            "Check the repo and make the Storybook states accurate"
        }
        AcpChatStateId::Search => "Find the earlier Storybook decision",
        AcpChatStateId::HistoryPopup => "",
        _ => "Summarize the Storybook cleanup plan",
    };

    div()
        .w_full()
        .min_h(px(if compact { 320.0 } else { 460.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .h(px(height))
                .rounded(px(10.0))
                .overflow_hidden()
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
                .bg(theme.colors.background.main.to_rgb())
                .relative()
                .flex()
                .flex_col()
                .child(render_composer(composer, id, compact))
                .when(id == AcpChatStateId::Search, |d| {
                    d.child(render_search_bar(compact))
                })
                .child(render_transcript(id, compact))
                .child(render_footer(id, compact))
                .when(id == AcpChatStateId::HistoryPopup, |d| {
                    d.child(render_history_popup(compact))
                }),
        )
        .into_any_element()
}

fn render_composer(text: &'static str, id: AcpChatStateId, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let placeholder = if matches!(id, AcpChatStateId::Empty | AcpChatStateId::HistoryPopup) {
        "Ask anything..."
    } else {
        "Follow up..."
    };

    div()
        .w_full()
        .px(px(12.0))
        .py(px(if compact { 8.0 } else { 10.0 }))
        .border_b_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x2E))
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .flex_1()
                .min_h(px(22.0))
                .text_size(px(if compact { 14.0 } else { 17.0 }))
                .line_height(px(22.0))
                .text_color(if text.is_empty() {
                    rgba((theme.colors.text.muted << 8) | 0xB3)
                } else {
                    rgba((theme.colors.text.primary << 8) | 0xEA)
                })
                .child(if text.is_empty() { placeholder } else { text }),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba((theme.colors.text.muted << 8) | 0xA0))
                .child("Claude Sonnet"),
        )
        .into_any_element()
}

fn render_search_bar(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .px(px(12.0))
        .py(px(if compact { 4.0 } else { 6.0 }))
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_sm()
        .child(
            div()
                .text_color(rgba((theme.colors.text.muted << 8) | 0x80))
                .child("⌕"),
        )
        .child(
            div()
                .flex_1()
                .text_color(rgba((theme.colors.text.primary << 8) | 0xDD))
                .child("storybook"),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba((theme.colors.text.muted << 8) | 0x80))
                .child("2/5"),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba((theme.colors.text.muted << 8) | 0x55))
                .child("esc ×"),
        )
        .into_any_element()
}

fn render_transcript(id: AcpChatStateId, compact: bool) -> AnyElement {
    let body = div()
        .flex_1()
        .min_h(px(0.0))
        .overflow_hidden()
        .px(px(if compact { 8.0 } else { 10.0 }))
        .py(px(if compact { 8.0 } else { 10.0 }))
        .flex()
        .flex_col()
        .gap(px(if compact { 6.0 } else { 8.0 }));

    match id {
        AcpChatStateId::Empty | AcpChatStateId::HistoryPopup => render_empty_transcript(compact),
        AcpChatStateId::ToolCall => body
            .child(message(
                "You",
                "List the files involved in Storybook cleanup.",
                true,
                compact,
            ))
            .child(message(
                "Assistant",
                "I’ll inspect the Storybook modules and catalog registration first.",
                false,
                compact,
            ))
            .child(tool_block(
                "Read files",
                "complete",
                "src/stories/mod.rs\nsrc/storybook/mod.rs",
                compact,
            ))
            .into_any_element(),
        AcpChatStateId::Permission => body
            .child(message(
                "You",
                "Update the Storybook catalog and commit it.",
                true,
                compact,
            ))
            .child(tool_block(
                "Write file",
                "permission required",
                "src/storybook/acp_chat_states.rs",
                compact,
            ))
            .child(permission_card(compact))
            .into_any_element(),
        AcpChatStateId::Streaming => body
            .child(message(
                "You",
                "What state coverage is still missing?",
                true,
                compact,
            ))
            .child(message(
                "Assistant",
                "The next gaps are Agent Chat, built-in browsers, and lower-level component primitives",
                false,
                compact,
            ))
            .child(streaming_cursor())
            .into_any_element(),
        AcpChatStateId::Search => body
            .child(message(
                "You",
                "Find the Storybook migration rule.",
                true,
                compact,
            ))
            .child(highlight_message(
                "Assistant",
                "Storybook cleanup should preserve evidence before deleting exploratory stories.",
                compact,
            ))
            .child(message(
                "Assistant",
                "PNG-backed runtime fixture stories should stay out of the primary catalog.",
                false,
                compact,
            ))
            .into_any_element(),
        AcpChatStateId::Conversation | AcpChatStateId::Setup => body
            .child(message(
                "You",
                "Summarize the Storybook cleanup plan.",
                true,
                compact,
            ))
            .child(message(
                "Assistant",
                "Keep the main menu and dictation coverage, remove PNG fixture stories, then add canonical states for the app windows and popups.",
                false,
                compact,
            ))
            .child(message(
                "Assistant",
                "Each registered variant should declare liveSurface or presenterFixture evidence quality.",
                false,
                compact,
            ))
            .into_any_element(),
    }
}

fn render_empty_transcript(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(if compact { 4.0 } else { 6.0 }))
        .text_xs()
        .text_color(rgba((theme.colors.text.muted << 8) | 0x70))
        .child("Type / for skills")
        .child("⇧↩ for newlines")
        .child("⌘P history · ⌘K actions")
        .child("⌘N new · ⌘W close")
        .into_any_element()
}

fn message(label: &'static str, body: &'static str, user: bool, compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .px(px(if compact { 8.0 } else { 12.0 }))
        .py(px(if compact { 6.0 } else { 8.0 }))
        .rounded(px(8.0))
        .bg(if user {
            rgba((theme.colors.text.primary << 8) | 0x06)
        } else {
            rgba((theme.colors.background.main << 8) | 0x00)
        })
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgba((theme.colors.text.muted << 8) | 0x99))
                .child(label),
        )
        .child(
            div()
                .pt(px(3.0))
                .text_size(px(if compact { 12.0 } else { 14.0 }))
                .line_height(px(if compact { 18.0 } else { 20.0 }))
                .text_color(rgba((theme.colors.text.primary << 8) | 0xE6))
                .child(body),
        )
        .into_any_element()
}

fn highlight_message(label: &'static str, body: &'static str, compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .rounded(px(6.0))
        .bg(rgba((theme.colors.accent.selected << 8) | 0x18))
        .border_l_2()
        .border_color(theme.colors.accent.selected.to_rgb())
        .child(message(label, body, false, compact))
        .into_any_element()
}

fn tool_block(
    title: &'static str,
    status: &'static str,
    body: &'static str,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .pl(px(if compact { 8.0 } else { 12.0 }))
        .pr(px(10.0))
        .py(px(4.0))
        .border_l_2()
        .border_color(rgba((theme.colors.accent.selected << 8) | 0x30))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .text_xs()
                .text_color(rgba((theme.colors.text.secondary << 8) | 0xAA))
                .child("▾")
                .child(title)
                .child(
                    div()
                        .text_color(rgba((theme.colors.text.muted << 8) | 0x88))
                        .child(status),
                ),
        )
        .child(
            div()
                .pt(px(4.0))
                .font_family(FONT_MONO)
                .text_xs()
                .line_height(px(17.0))
                .text_color(rgba((theme.colors.text.primary << 8) | 0xB8))
                .child(body),
        )
        .into_any_element()
}

fn permission_card(compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .mx(px(if compact { 8.0 } else { 12.0 }))
        .mt(px(2.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(rgba((theme.colors.accent.selected << 8) | 0x44))
        .bg(rgba((theme.colors.accent.selected_subtle << 8) | 0x50))
        .p(px(if compact { 8.0 } else { 10.0 }))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.text.primary.to_rgb())
                .child("Allow file edit?"),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba((theme.colors.text.secondary << 8) | 0xCC))
                .child("The agent wants to write src/storybook/acp_chat_states.rs."),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(action_chip("Allow once", true))
                .child(action_chip("Deny", false)),
        )
        .into_any_element()
}

fn streaming_cursor() -> AnyElement {
    let theme = get_cached_theme();

    div()
        .px(px(12.0))
        .text_sm()
        .text_color(theme.colors.accent.selected.to_rgb())
        .child("▌")
        .into_any_element()
}

fn render_footer(id: AcpChatStateId, compact: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .w_full()
        .h(px(if compact { 27.0 } else { 31.0 }))
        .px(px(12.0))
        .border_t_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x2E))
        .flex()
        .items_center()
        .justify_between()
        .font_family(FONT_MONO)
        .text_xs()
        .text_color(rgba((theme.colors.text.muted << 8) | 0x8F))
        .child(if id == AcpChatStateId::Streaming {
            "● streaming"
        } else {
            "⌘P history"
        })
        .child("⌘K actions")
        .child(if id == AcpChatStateId::Empty {
            "⌘N new"
        } else {
            "⌘↵ send"
        })
        .into_any_element()
}

fn render_history_popup(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 300.0 } else { 360.0 };

    div()
        .absolute()
        .left(px(28.0))
        .bottom(px(if compact { 36.0 } else { 42.0 }))
        .w(px(width))
        .rounded(px(10.0))
        .border_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
        .bg(theme.colors.background.main.to_rgb())
        .p(px(10.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child("Recent Conversations"),
        )
        .child(history_row(
            "Storybook cleanup plan",
            "Today · 9 messages",
            true,
        ))
        .child(history_row(
            "ACP popup audit",
            "Yesterday · 14 messages",
            false,
        ))
        .child(history_row(
            "Notes handoff design",
            "Apr 20 · 7 messages",
            false,
        ))
        .into_any_element()
}

fn history_row(title: &'static str, subtitle: &'static str, selected: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .rounded(px(7.0))
        .px(px(8.0))
        .py(px(7.0))
        .when(selected, |d| {
            d.bg(rgba((theme.colors.accent.selected_subtle << 8) | 0x88))
        })
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.text.primary.to_rgb())
                .child(title),
        )
        .child(
            div()
                .pt(px(2.0))
                .text_xs()
                .text_color(rgba((theme.colors.text.muted << 8) | 0x99))
                .child(subtitle),
        )
        .into_any_element()
}

fn render_setup_card(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 420.0 } else { 560.0 };

    div()
        .w_full()
        .min_h(px(if compact { 320.0 } else { 460.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .rounded(px(10.0))
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
                .bg(theme.colors.background.main.to_rgb())
                .p(px(if compact { 16.0 } else { 22.0 }))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child("Set up Agent Chat"),
                )
                .child(
                    div()
                        .text_sm()
                        .line_height(px(20.0))
                        .text_color(rgba((theme.colors.text.secondary << 8) | 0xDD))
                        .child(
                            "Claude Code needs an authenticated agent before this chat can start.",
                        ),
                )
                .child(setup_requirement("Agent binary", "Found"))
                .child(setup_requirement("Authentication", "Needs sign in"))
                .child(setup_requirement("Workspace trust", "Ready"))
                .child(
                    div()
                        .pt(px(4.0))
                        .flex()
                        .gap(px(8.0))
                        .child(action_chip("Retry", true))
                        .child(action_chip("Open Settings", false)),
                ),
        )
        .into_any_element()
}

fn setup_requirement(label: &'static str, status: &'static str) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .rounded(px(8.0))
        .border_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x3D))
        .px(px(10.0))
        .py(px(8.0))
        .flex()
        .items_center()
        .justify_between()
        .text_sm()
        .child(
            div()
                .text_color(theme.colors.text.primary.to_rgb())
                .child(label),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba((theme.colors.text.secondary << 8) | 0xB3))
                .child(status),
        )
        .into_any_element()
}

fn action_chip(label: &'static str, active: bool) -> AnyElement {
    let theme = get_cached_theme();

    div()
        .h(px(24.0))
        .px(px(9.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(if active {
            rgba((theme.colors.accent.selected << 8) | 0xFF)
        } else {
            rgba((theme.colors.text.secondary << 8) | 0xB3)
        })
        .bg(if active {
            rgba((theme.colors.accent.selected_subtle << 8) | 0x80)
        } else {
            rgba((theme.colors.text.primary << 8) | 0x08)
        })
        .border_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x44))
        .child(label)
        .into_any_element()
}
