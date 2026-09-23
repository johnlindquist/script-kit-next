//! gpui-ai renderers for Agent Chat transcript blocks.
//!
//! Agent Chat standardizes on the gpui-ai component grammar (the library's
//! own agent layout: user bubble → `ToolCall` cards → `Thinking` trace →
//! streamed answer). This module is the single adapter from Script Kit's
//! thread snapshot (`AgentChatThreadMessage` + `AgentChatToolCardMeta`) to the
//! gpui-ai snapshot types; the transcript owns collapse state and routes each
//! component's typed `Toggled` intent back to `toggle_collapsed`.
//!
//! Colors come from gpui-component 0.6's active theme, which
//! `theme::gpui_kit_bridge` keeps synced from the Script Kit theme.

use gpui::{
    div, AnyElement, InteractiveElement as _, IntoElement, ParentElement, SharedString, Styled,
    WeakEntity,
};
use gpui_ai::prelude::*;

use super::transcript::AgentChatTranscript;
use crate::ai::agent_chat::ui::thread::AgentChatThreadMessage;
use crate::ai::agent_chat::ui::tool_card::{AgentChatToolCardMeta, AgentChatToolStatus};

/// Stable gpui-ai identity for one thread message.
pub(crate) fn block_id(prefix: &str, message_id: u64) -> SharedString {
    SharedString::from(format!("agent-chat-{prefix}-{message_id}"))
}

/// Map the Pi tool lifecycle onto gpui-ai's progressive state.
pub(crate) fn tool_progress<T>(status: AgentChatToolStatus, content: T) -> Progressive<T> {
    match status {
        AgentChatToolStatus::Pending => Progressive::pending(content),
        AgentChatToolStatus::Running => Progressive::running(content),
        AgentChatToolStatus::Complete => Progressive::complete(content),
        AgentChatToolStatus::Failed => Progressive::failed(content, "Tool call failed"),
    }
}

/// Build the gpui-ai invocation snapshot for a structured tool message.
pub(crate) fn tool_invocation(
    msg: &AgentChatThreadMessage,
    meta: &AgentChatToolCardMeta,
    display_body: &str,
) -> Progressive<ToolInvocation> {
    let mut invocation = ToolInvocation::new(block_id("tool", msg.id), meta.tool_name.clone());
    if let Some(subject) = meta.subject.as_ref() {
        invocation = invocation.summary(subject.clone());
    }
    let output = meta
        .diff
        .clone()
        .map(|diff| format!("```diff\n{diff}\n```"))
        .unwrap_or_else(|| display_body.trim().to_string());
    if !output.is_empty() {
        invocation = invocation.output(output);
    }
    let status = if meta.is_error {
        AgentChatToolStatus::Failed
    } else {
        meta.status
    };
    tool_progress(status, invocation)
}

fn toggle_handler(
    entity: &WeakEntity<AgentChatTranscript>,
    message_id: u64,
    currently_collapsed: bool,
) -> impl Fn(bool, &mut gpui::App) + 'static {
    let entity = entity.clone();
    move |open: bool, cx: &mut gpui::App| {
        // Only flip when the requested state differs from the current one, so
        // a repeated event can never invert the user's intent.
        if open == currently_collapsed {
            if let Some(transcript) = entity.upgrade() {
                transcript.update(cx, |this, cx| this.toggle_collapsed(message_id, cx));
            }
        }
    }
}

/// Structured tool call rendered as a gpui-ai `ToolCall` card.
pub(crate) fn render_tool_call(
    msg: &AgentChatThreadMessage,
    meta: &AgentChatToolCardMeta,
    display_body: &str,
    is_collapsed: bool,
    entity: &WeakEntity<AgentChatTranscript>,
) -> AnyElement {
    let call = tool_invocation(msg, meta, display_body);
    let toggle = toggle_handler(entity, msg.id, is_collapsed);
    let selector = format!("agent-chat-tool-call-{}", msg.id);
    div()
        .debug_selector(move || selector.clone())
        .w_full()
        .child(
            ToolCall::new(&call)
                .open(!is_collapsed)
                .on_event(move |event, _window, cx| {
                    if let ToolCallEvent::Toggled { open, .. } = event {
                        toggle(*open, cx);
                    }
                }),
        )
        .into_any_element()
}

/// Untyped tool output (no structured metadata) as a gpui-ai `ToolCall`.
pub(crate) fn render_plain_tool(
    msg: &AgentChatThreadMessage,
    is_collapsed: bool,
    entity: &WeakEntity<AgentChatTranscript>,
) -> AnyElement {
    let mut lines = msg.body.lines();
    let title = lines
        .next()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && line.len() < 80)
        .unwrap_or_else(|| "Tool".to_string());
    let rest = lines.collect::<Vec<_>>().join("\n");
    let mut invocation = ToolInvocation::new(block_id("tool", msg.id), title);
    if !rest.trim().is_empty() {
        invocation = invocation.output(rest.trim().to_string());
    }
    let call = Progressive::complete(invocation);
    let toggle = toggle_handler(entity, msg.id, is_collapsed);
    let selector = format!("agent-chat-tool-call-{}", msg.id);
    div()
        .debug_selector(move || selector.clone())
        .w_full()
        .child(
            ToolCall::new(&call)
                .open(!is_collapsed)
                .on_event(move |event, _window, cx| {
                    if let ToolCallEvent::Toggled { open, .. } = event {
                        toggle(*open, cx);
                    }
                }),
        )
        .into_any_element()
}

/// Model reasoning rendered as a gpui-ai `Thinking` disclosure.
pub(crate) fn render_thinking(
    msg: &AgentChatThreadMessage,
    is_collapsed: bool,
    streaming: bool,
    entity: &WeakEntity<AgentChatTranscript>,
) -> AnyElement {
    let trace = ThinkingTrace::new().prose(msg.body.clone());
    let trace = if streaming {
        Progressive::running(trace)
    } else {
        Progressive::complete(trace)
    };
    let toggle = toggle_handler(entity, msg.id, is_collapsed);
    let selector = format!("agent-chat-thinking-{}", msg.id);
    div()
        .debug_selector(move || selector.clone())
        .w_full()
        .child(
            Thinking::new(block_id("thinking", msg.id), &trace)
                .open(!is_collapsed)
                .on_event(move |event, _window, cx| {
                    let ThinkingEvent::Toggled { open, .. } = event;
                    toggle(*open, cx);
                }),
        )
        .into_any_element()
}

/// Assistant answer rendered as gpui-ai `StreamingText` (selectable markdown
/// with the library's streaming cursor while the turn is still running).
pub(crate) fn render_assistant_text(msg: &AgentChatThreadMessage, streaming: bool) -> AnyElement {
    let content = if streaming {
        let mut content = StreamedContent::running(String::new());
        content.append(&msg.body);
        content
    } else {
        StreamedContent::done(msg.body.to_string())
    };
    let selector = format!("agent-chat-assistant-text-{}", msg.id);
    div()
        .debug_selector(move || selector.clone())
        .w_full()
        .min_w_0()
        .child(StreamingText::new(block_id("answer", msg.id), &content))
        .into_any_element()
}

/// gpui-ai's human-turn bubble colors (accent surface + contrast ink),
/// resolved from the Script Kit theme so the bubble follows every theme.
pub(crate) fn user_bubble_colors(theme: &crate::theme::Theme) -> (gpui::Rgba, gpui::Rgba) {
    let colors = &theme.colors;
    let ink = crate::theme::gpui_integration::best_contrast_of_two(
        colors.accent.selected,
        colors.text.on_accent,
        colors.background.main,
    );
    (gpui::rgb(colors.accent.selected), gpui::rgb(ink))
}
