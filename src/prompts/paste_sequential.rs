//! PasteSequentialPrompt - paste multiple lines sequentially.
//!
//! Users enter one item per line, then trigger pasting repeatedly.
//! Each trigger copies the next item to the clipboard and performs a paste.

use gpui::{div, prelude::*, px, Context, FocusHandle, Focusable, Render, Window};
use gpui_component::scroll::ScrollableElement;
use std::sync::Arc;

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::designs::{get_tokens, DesignVariant};
use crate::theme;
use crate::ui_foundation::{get_vibrancy_background, HexColorExt};

const PASTE_SEQUENTIAL_EMPTY_INPUT: &str = "Add at least one non-empty line to paste.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PasteSequentialKeyAction {
    TriggerNext,
    AppendNewline,
    DeleteChar,
    Ignore,
}

pub struct PasteSequentialPrompt {
    pub id: String,
    pub input_text: String,
    pub queued_items: Vec<String>,
    pub next_index: usize,
    pub status_message: String,
    pub error_message: Option<String>,
    pub focus_handle: FocusHandle,
    pub theme: Arc<theme::Theme>,
    pub design_variant: DesignVariant,
}

impl PasteSequentialPrompt {
    pub fn new(id: String, focus_handle: FocusHandle, theme: Arc<theme::Theme>) -> Self {
        Self {
            id,
            input_text: String::new(),
            queued_items: Vec::new(),
            next_index: 0,
            status_message: "Enter one item per line, then press Cmd+Enter or Start Pasting."
                .to_string(),
            error_message: None,
            focus_handle,
            theme,
            design_variant: DesignVariant::Default,
        }
    }

    fn begin_queue_if_needed(&mut self) -> Result<(), String> {
        if self.next_index < self.queued_items.len() {
            return Ok(());
        }

        self.queued_items = parse_sequential_items(&self.input_text);
        self.next_index = 0;

        if self.queued_items.is_empty() {
            return Err(PASTE_SEQUENTIAL_EMPTY_INPUT.to_string());
        }

        tracing::info!(
            event = "paste_sequential_queue_initialized",
            prompt_id = %self.id,
            item_count = self.queued_items.len(),
            "Initialized paste-sequential queue"
        );

        Ok(())
    }

    fn paste_next_item(&mut self) -> Result<(), String> {
        self.begin_queue_if_needed()?;

        let current_item = self
            .queued_items
            .get(self.next_index)
            .cloned()
            .ok_or_else(|| "No queued item available at current index".to_string())?;

        let mut clipboard = arboard::Clipboard::new()
            .map_err(|error| format!("Failed to open clipboard: {}", error))?;
        clipboard
            .set_text(current_item.clone())
            .map_err(|error| format!("Failed to write clipboard text: {}", error))?;

        #[cfg(target_os = "macos")]
        {
            crate::selected_text::simulate_paste_with_cg()
                .map_err(|error| format!("Failed to simulate paste: {}", error))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            return Err("Sequential paste is only supported on macOS".to_string());
        }

        self.next_index += 1;
        tracing::info!(
            event = "paste_sequential_item_pasted",
            prompt_id = %self.id,
            pasted_index = self.next_index,
            total_items = self.queued_items.len(),
            "Pasted queued clipboard item"
        );

        Ok(())
    }

    fn trigger_next(&mut self, cx: &mut Context<Self>) {
        match self.paste_next_item() {
            Ok(()) => {
                self.error_message = None;
                self.status_message =
                    progress_status_message(self.next_index, self.queued_items.len());
            }
            Err(error) => {
                tracing::warn!(
                    event = "paste_sequential_trigger_failed",
                    prompt_id = %self.id,
                    next_index = self.next_index,
                    queue_len = self.queued_items.len(),
                    error = %error,
                    "Paste-sequential trigger failed"
                );
                self.error_message = Some(error);
            }
        }

        cx.notify();
    }

    fn append_newline(&mut self, cx: &mut Context<Self>) {
        self.input_text.push('\n');
        cx.notify();
    }

    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if self.input_text.pop().is_some() {
            cx.notify();
        }
    }

    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.input_text.push(ch);
        cx.notify();
    }

    fn render_input_lines(&self, text_primary: u32, text_muted: u32) -> gpui::Div {
        if self.input_text.is_empty() {
            return div()
                .text_sm()
                .text_color(text_muted.to_rgb())
                .child("One item per line...");
        }

        let mut lines = div().flex().flex_col().gap(px(2.));
        for line in self.input_text.split('\n') {
            let display = if line.is_empty() {
                " ".to_string()
            } else {
                line.to_string()
            };
            lines = lines.child(
                div()
                    .text_sm()
                    .text_color(text_primary.to_rgb())
                    .child(display),
            );
        }

        lines
    }
}

impl Focusable for PasteSequentialPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PasteSequentialPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let spacing = tokens.spacing();

        let vibrancy_bg = get_vibrancy_background(&self.theme);
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_secondary = self.theme.colors.text.secondary;
        let border_color = self.theme.colors.ui.border;
        let accent = self.theme.colors.accent.selected;
        let error_color = self.theme.colors.ui.error;
        let input_bg = self.theme.colors.background.search_box;

        let start_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.trigger_next(cx);
        });

        let button_colors = ButtonColors::from_theme(&self.theme);

        let container = div()
            .id(gpui::ElementId::Name("window:paste-sequential".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg))
            .text_color(text_primary.to_rgb())
            .p(px(spacing.padding_lg))
            .gap(px(spacing.gap_md))
            .child(div().text_lg().child("Paste Sequentially"))
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted.to_rgb())
                    .child("Enter items on separate lines. Each trigger pastes the next line."),
            )
            .child(
                div()
                    .id(gpui::ElementId::Name("paste-sequential-input".into()))
                    .flex_1()
                    .min_h(px(180.))
                    .w_full()
                    .rounded(px(8.))
                    .bg(input_bg.to_rgb())
                    .border_1()
                    .border_color(border_color.to_rgb())
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.padding_md))
                    .overflow_y_scrollbar()
                    .child(self.render_input_lines(text_primary, text_muted)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(text_secondary.to_rgb())
                    .child(format!(
                        "Queue progress: {}/{}",
                        self.next_index,
                        self.queued_items.len()
                    )),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(accent.to_rgb())
                    .child(self.status_message.clone()),
            )
            .when_some(self.error_message.clone(), |d, error| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(error_color.to_rgb())
                        .child(error),
                )
            })
            .child(
                div().mt_auto().flex().justify_end().child(
                    Button::new("Start Pasting", button_colors)
                        .variant(ButtonVariant::Primary)
                        .shortcut("Cmd+Enter")
                        .on_click(Box::new(move |event, window, cx| {
                            start_handler(event, window, cx);
                        })),
                ),
            );

        FocusablePrompt::new(container)
            .key_context("paste_sequential_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |_this, intercepted_key, _event, _window, _cx| {
                    !matches!(
                        intercepted_key,
                        FocusablePromptInterceptedKey::Escape
                            | FocusablePromptInterceptedKey::CmdW
                            | FocusablePromptInterceptedKey::CmdK
                    )
                },
                |this, event, _window, cx| {
                    let key = event.keystroke.key.as_str();

                    match classify_key_action(key, event.keystroke.modifiers.platform) {
                        Some(PasteSequentialKeyAction::TriggerNext) => {
                            this.trigger_next(cx);
                            return;
                        }
                        Some(PasteSequentialKeyAction::AppendNewline) => {
                            this.append_newline(cx);
                            return;
                        }
                        Some(PasteSequentialKeyAction::DeleteChar) => {
                            this.handle_backspace(cx);
                            return;
                        }
                        Some(PasteSequentialKeyAction::Ignore) => {
                            return;
                        }
                        None => {}
                    }

                    if let Some(ch) = printable_char(event.keystroke.key_char.as_deref()) {
                        this.handle_char(ch, cx);
                    }
                },
            )
    }
}

fn printable_char(key_char: Option<&str>) -> Option<char> {
    key_char
        .and_then(|value| value.chars().next())
        .filter(|ch| !ch.is_control())
}

fn classify_key_action(key: &str, has_platform_modifier: bool) -> Option<PasteSequentialKeyAction> {
    match key {
        "enter" | "Enter" | "return" | "Return" if has_platform_modifier => {
            Some(PasteSequentialKeyAction::TriggerNext)
        }
        "enter" | "Enter" | "return" | "Return" => Some(PasteSequentialKeyAction::AppendNewline),
        "backspace" | "Backspace" => Some(PasteSequentialKeyAction::DeleteChar),
        "escape" | "Escape" => Some(PasteSequentialKeyAction::Ignore),
        _ => None,
    }
}

fn parse_sequential_items(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn progress_status_message(pasted_count: usize, total: usize) -> String {
    if total == 0 {
        return "Queue is empty. Add items and trigger again.".to_string();
    }

    if pasted_count >= total {
        format!(
            "Pasted {}/{}. Queue complete. Trigger again to restart.",
            total, total
        )
    } else {
        format!(
            "Pasted {}/{}. Trigger again to paste next item.",
            pasted_count, total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_key_action, parse_sequential_items, progress_status_message,
        PasteSequentialKeyAction,
    };

    #[test]
    fn test_parse_sequential_items_trims_and_skips_empty_lines() {
        let parsed = parse_sequential_items("  alpha\n\n beta  \n   \n\tgamma\t\n");
        assert_eq!(parsed, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_progress_status_message_reports_completion_when_queue_done() {
        let message = progress_status_message(3, 3);
        assert!(message.contains("Queue complete"));
        assert!(message.contains("3/3"));
    }

    #[test]
    fn test_progress_status_message_reports_next_trigger_when_items_remain() {
        let message = progress_status_message(1, 4);
        assert!(message.contains("1/4"));
        assert!(message.contains("Trigger again"));
    }

    #[test]
    fn test_classify_key_action_handles_enter_and_return_variants() {
        assert_eq!(
            classify_key_action("enter", true),
            Some(PasteSequentialKeyAction::TriggerNext)
        );
        assert_eq!(
            classify_key_action("Enter", true),
            Some(PasteSequentialKeyAction::TriggerNext)
        );
        assert_eq!(
            classify_key_action("return", false),
            Some(PasteSequentialKeyAction::AppendNewline)
        );
        assert_eq!(
            classify_key_action("Return", false),
            Some(PasteSequentialKeyAction::AppendNewline)
        );
    }

    #[test]
    fn test_classify_key_action_handles_escape_and_backspace_variants() {
        assert_eq!(
            classify_key_action("backspace", false),
            Some(PasteSequentialKeyAction::DeleteChar)
        );
        assert_eq!(
            classify_key_action("Backspace", false),
            Some(PasteSequentialKeyAction::DeleteChar)
        );
        assert_eq!(
            classify_key_action("escape", false),
            Some(PasteSequentialKeyAction::Ignore)
        );
        assert_eq!(
            classify_key_action("Escape", false),
            Some(PasteSequentialKeyAction::Ignore)
        );
    }
}
