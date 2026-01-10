//! ChatPrompt - Raycast-style chat interface
//!
//! Features:
//! - Message bubbles (user on right, assistant on left)
//! - Streaming response support with typing indicator
//! - Text input at bottom

use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FocusHandle, Focusable, Hsla, KeyDownEvent, Point,
    Render, ScrollHandle, Window,
};
use std::sync::Arc;

use crate::components::TextInputState;
use crate::logging;
use crate::protocol::{ChatMessagePosition, ChatPromptMessage};
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

/// Callback type for when user submits a message: (prompt_id, message_text)
pub type ChatSubmitCallback = Arc<dyn Fn(String, String) + Send + Sync>;

/// ChatPrompt - Raycast-style chat interface
pub struct ChatPrompt {
    pub id: String,
    pub messages: Vec<ChatPromptMessage>,
    pub placeholder: Option<String>,
    pub hint: Option<String>,
    pub footer: Option<String>,
    pub focus_handle: FocusHandle,
    pub input: TextInputState,
    pub on_submit: ChatSubmitCallback,
    pub theme: Arc<theme::Theme>,
    pub scroll_handle: ScrollHandle,
    scroll_offset: Point<f32>,
    prompt_colors: theme::PromptColors,
    streaming_message_id: Option<String>,
}

impl ChatPrompt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        placeholder: Option<String>,
        messages: Vec<ChatPromptMessage>,
        hint: Option<String>,
        footer: Option<String>,
        focus_handle: FocusHandle,
        on_submit: ChatSubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        let prompt_colors = theme.colors.prompt_colors();
        logging::log("PROMPTS", &format!("ChatPrompt::new id={}", id));

        Self {
            id,
            messages,
            placeholder,
            hint,
            footer,
            focus_handle,
            input: TextInputState::new(),
            on_submit,
            theme,
            scroll_handle: ScrollHandle::new(),
            scroll_offset: Point::default(),
            prompt_colors,
            streaming_message_id: None,
        }
    }

    pub fn add_message(&mut self, message: ChatPromptMessage, cx: &mut Context<Self>) {
        logging::log("CHAT", &format!("Adding message: {:?}", message.position));
        self.messages.push(message);
        cx.notify();
    }

    pub fn start_streaming(&mut self, message_id: String, position: ChatMessagePosition, cx: &mut Context<Self>) {
        let message = ChatPromptMessage {
            id: Some(message_id.clone()),
            text: String::new(),
            position,
            name: None,
            streaming: true,
        };
        self.messages.push(message);
        self.streaming_message_id = Some(message_id);
        cx.notify();
    }

    pub fn append_chunk(&mut self, message_id: &str, chunk: &str, cx: &mut Context<Self>) {
        if self.streaming_message_id.as_deref() == Some(message_id) {
            if let Some(msg) = self.messages.iter_mut().rev().find(|m| m.id.as_deref() == Some(message_id)) {
                msg.text.push_str(chunk);
                cx.notify();
            }
        }
    }

    pub fn complete_streaming(&mut self, message_id: &str, cx: &mut Context<Self>) {
        if let Some(msg) = self.messages.iter_mut().rev().find(|m| m.id.as_deref() == Some(message_id)) {
            msg.streaming = false;
        }
        if self.streaming_message_id.as_deref() == Some(message_id) {
            self.streaming_message_id = None;
        }
        cx.notify();
    }

    pub fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        self.streaming_message_id = None;
        cx.notify();
    }

    fn handle_submit(&mut self, _cx: &mut Context<Self>) {
        let text = self.input.text().to_string();
        if text.trim().is_empty() {
            return;
        }
        logging::log("CHAT", &format!("User submitted: {}", text));
        self.input.clear();
        (self.on_submit)(self.id.clone(), text);
    }

    fn render_message(&self, message: &ChatPromptMessage) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let is_user = message.position == ChatMessagePosition::Right;

        let bubble_bg = if is_user {
            rgba((colors.accent_color << 8) | 0xE0)
        } else {
            rgba((colors.code_bg << 8) | 0xC0)
        };

        let text_color: Hsla = if is_user { Hsla::white() } else { rgb(colors.text_primary).into() };

        let mut bubble = div()
            .max_w(px(400.0))
            .px(px(12.0))
            .py(px(8.0))
            .bg(bubble_bg)
            .rounded(px(12.0))
            .text_sm()
            .text_color(text_color)
            .child(message.text.clone());

        if message.streaming && message.text.is_empty() {
            bubble = bubble.child(div().text_xs().opacity(0.6).child("..."));
        }

        let mut row = div().w_full().flex().my(px(4.0));
        if is_user {
            row = row.flex_row_reverse();
        }
        row.child(bubble)
    }

    fn render_input(&self, _cx: &Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;
        let text = self.input.text();
        let cursor_pos = self.input.cursor();
        let chars: Vec<char> = text.chars().collect();
        let text_primary = colors.text_primary;
        let accent = colors.accent_color;

        // Build input with cursor
        let mut input_content = div().flex().flex_row().items_center();

        // Text before cursor
        if cursor_pos > 0 {
            let before: String = chars[..cursor_pos].iter().collect();
            input_content = input_content.child(div().text_color(rgb(text_primary)).child(before));
        }

        // Cursor
        input_content = input_content.child(
            div().w(px(2.0)).h(px(16.0)).bg(rgb(accent))
        );

        // Text after cursor
        if cursor_pos < chars.len() {
            let after: String = chars[cursor_pos..].iter().collect();
            input_content = input_content.child(div().text_color(rgb(text_primary)).child(after));
        }

        // Placeholder if empty
        if text.is_empty() {
            let placeholder = self.placeholder.clone().unwrap_or_else(|| "Type a message...".into());
            input_content = div()
                .text_color(rgb(colors.text_tertiary))
                .child(placeholder)
                .child(div().w(px(2.0)).h(px(16.0)).bg(rgb(accent)));
        }

        input_content
    }
}

impl Focusable for ChatPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ChatPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.prompt_colors;

        let handle_key = cx.listener(|this, event: &KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.to_lowercase();
            let key_char = event.keystroke.key_char.as_deref();

            match key.as_str() {
                "enter" if !event.keystroke.modifiers.shift => this.handle_submit(cx),
                "backspace" => { this.input.backspace(); cx.notify(); }
                _ => {
                    // Handle regular character input via key_char
                    if let Some(ch_str) = key_char {
                        for ch in ch_str.chars() {
                            if ch.is_ascii_graphic() || ch == ' ' {
                                this.input.insert_char(ch);
                            }
                        }
                        cx.notify();
                    }
                }
            }
        });

        let container_bg: Option<Hsla> = get_vibrancy_background(&self.theme).map(Hsla::from);

        let mut message_list = div().flex().flex_col().gap(px(8.0)).w_full().px(px(12.0)).py(px(8.0));
        for message in &self.messages {
            message_list = message_list.child(self.render_message(message));
        }

        if self.messages.is_empty() {
            message_list = message_list.child(
                div().flex_1().items_center().justify_center()
                    .text_color(rgb(colors.text_tertiary)).text_sm()
                    .child(self.placeholder.clone().unwrap_or_else(|| "Start a conversation...".into()))
            );
        }

        let input_area = div()
            .w_full().px(px(12.0)).py(px(8.0))
            .border_t_1().border_color(rgb(colors.quote_border))
            .bg(rgba((colors.code_bg << 8) | 0x40))
            .child(self.render_input(cx));

        div()
            .id("chat-prompt")
            .flex().flex_col().w_full().h_full()
            .when_some(container_bg, |d, bg| d.bg(bg))
            .key_context("chat_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .when_some(self.hint.clone(), |d, hint| {
                d.child(div().w_full().px(px(12.0)).py(px(8.0))
                    .border_b_1().border_color(rgb(colors.quote_border))
                    .text_sm().text_color(rgb(colors.text_secondary)).child(hint))
            })
            .child(div().id("chat-messages").flex_1().min_h(px(0.)).overflow_y_scroll()
                .track_scroll(&self.scroll_handle).child(message_list))
            .when_some(self.footer.clone(), |d, footer| {
                d.child(div().w_full().px(px(12.0)).py(px(4.0))
                    .text_xs().text_color(rgb(colors.text_tertiary)).child(footer))
            })
            .child(input_area)
    }
}
