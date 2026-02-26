use super::types::*;
use super::*;

fn ai_sidebar_row_hover_enabled(input_mode: InputMode, is_selected: bool) -> bool {
    !is_selected && input_mode == InputMode::Mouse
}

fn ai_sidebar_delete_hover_enabled(input_mode: InputMode) -> bool {
    input_mode == InputMode::Mouse
}

impl AiApp {
    pub(super) fn render_sidebar_group_header(
        &self,
        group: DateGroup,
        _is_first_group: bool,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let group_label: SharedString = group.label().into();
        div()
            .flex()
            .w_full()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(cx.theme().muted_foreground.opacity(0.6))
            .px(SIDEBAR_INSET_X)
            .mt(S4)
            .mb(S2)
            .child(group_label)
    }

    /// Render a single chat item with title + preview and hover-revealed delete button.
    pub(super) fn render_chat_item(
        &self,
        chat: &Chat,
        selected_id: Option<ChatId>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let chat_id = chat.id;
        let is_selected = selected_id == Some(chat_id);

        let title: SharedString = if chat.title.is_empty() {
            "New Chat".into()
        } else {
            chat.title.clone().into()
        };

        let preview = self.message_previews.get(&chat_id).cloned();
        let msg_count = self.message_counts.get(&chat_id).copied().unwrap_or(0);
        let preview_snippet: SharedString = preview
            .as_deref()
            .map(|preview_text| {
                let (_preview_role, preview_snippet) =
                    self.build_sidebar_preview_identity(chat_id, msg_count, preview_text);
                preview_snippet
            })
            .unwrap_or_else(|| "No messages yet".to_string())
            .into();

        let selected_bg = cx.theme().list_active;
        let hover_bg = cx.theme().list_hover;

        let title_color = if is_selected {
            cx.theme().foreground
        } else {
            cx.theme().sidebar_foreground
        };
        let preview_color = cx.theme().muted_foreground.opacity(0.75);

        let muted_fg = cx.theme().muted_foreground;
        let is_renaming = self.renaming_chat_id == Some(chat_id);
        let is_mouse_mode = ai_sidebar_delete_hover_enabled(self.input_mode);
        let row_hover_enabled = ai_sidebar_row_hover_enabled(self.input_mode, is_selected);
        div()
            .id(SharedString::from(format!("chat-{}", chat_id)))
            .group("chat-item")
            .flex()
            .flex_col()
            .justify_center()
            .w_full()
            .min_h(SIDEBAR_ROW_H)
            .px(SIDEBAR_INSET_X)
            .py(S2)
            .gap(S1)
            .rounded(R_MD)
            .cursor_pointer()
            .when(is_selected, |d| d.bg(selected_bg))
            .when(row_hover_enabled, |d| d.hover(|d| d.bg(hover_bg)))
            .on_click(
                cx.listener(move |this, event: &gpui::ClickEvent, window, cx| {
                    if event.click_count() == 2 {
                        this.start_rename(chat_id, window, cx);
                    } else {
                        this.select_chat(chat_id, window, cx);
                    }
                }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .gap(S1)
                    .when(is_renaming, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(self.rename_input_state.clone()),
                        )
                    })
                    .when(!is_renaming, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(title_color)
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(title),
                        )
                    })
                    // Delete button - visible on hover only.
                    // Two-step: first click shows "Delete?", second click confirms.
                    .child({
                        let is_confirming = self.pending_delete_chat_id == Some(chat_id);
                        if is_confirming {
                            // Confirmation state: show red "Delete?" label
                            div()
                                .id(SharedString::from(format!("del-{}", chat_id)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .px(S1)
                                .py(S1)
                                .rounded(R_SM)
                                .flex_shrink_0()
                                .cursor_pointer()
                                .bg(cx.theme().danger.opacity(0.15))
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .when(is_mouse_mode, |d| {
                                    d.hover(|s| s.bg(cx.theme().danger.opacity(0.25)))
                                })
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        this.pending_delete_chat_id = None;
                                        this.delete_chat_by_id(chat_id, cx);
                                    }),
                                )
                                .child("Delete?")
                        } else {
                            div()
                                .id(SharedString::from(format!("del-{}", chat_id)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(S5)
                                .rounded(R_SM)
                                .flex_shrink_0()
                                .cursor_pointer()
                                .opacity(0.)
                                .when(is_mouse_mode, |d| {
                                    d.group_hover("chat-item", |s| s.opacity(1.0))
                                        .hover(|s| s.bg(cx.theme().danger.opacity(0.19)))
                                })
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        this.pending_delete_chat_id = Some(chat_id);
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    svg()
                                        .external_path(LocalIconName::Trash.external_path())
                                        .size(S3)
                                        .text_color(muted_fg.opacity(0.5)),
                                )
                        }
                    }),
            )
            .child(
                div()
                    .w_full()
                    .text_xs()
                    .text_color(preview_color)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(preview_snippet),
            )
    }

    fn build_sidebar_preview_identity(
        &self,
        chat_id: ChatId,
        msg_count: usize,
        preview_text: &str,
    ) -> (MessageRole, String) {
        let selected_last_role = if self.selected_chat_id == Some(chat_id) {
            self.current_messages.last().map(|message| message.role)
        } else {
            None
        };

        let role = Self::resolve_sidebar_preview_role(selected_last_role, msg_count, preview_text);
        let (_, clean_line) = Self::extract_sidebar_preview_line_and_role(preview_text);
        let preview_source = if clean_line.is_empty() {
            Self::fallback_sidebar_preview_line(preview_text)
        } else {
            clean_line
        };

        let mut preview_snippet: String = preview_source.chars().take(50).collect();
        if preview_snippet.is_empty() {
            preview_snippet = "...".to_string();
        }

        (role, preview_snippet)
    }

    fn resolve_sidebar_preview_role(
        selected_last_role: Option<MessageRole>,
        msg_count: usize,
        preview_text: &str,
    ) -> MessageRole {
        let (prefixed_role, _) = Self::extract_sidebar_preview_line_and_role(preview_text);

        selected_last_role
            .or(prefixed_role)
            .or_else(|| {
                if msg_count > 1 && Self::looks_like_assistant_continuation(preview_text) {
                    Some(MessageRole::Assistant)
                } else {
                    None
                }
            })
            .or_else(|| Self::infer_sidebar_preview_role_from_message_count(msg_count))
            .unwrap_or(MessageRole::Assistant)
    }

    fn infer_sidebar_preview_role_from_message_count(msg_count: usize) -> Option<MessageRole> {
        if msg_count == 0 {
            None
        } else if msg_count.is_multiple_of(2) {
            Some(MessageRole::Assistant)
        } else {
            Some(MessageRole::User)
        }
    }

    fn looks_like_assistant_continuation(preview_text: &str) -> bool {
        if preview_text
            .split("\n\n")
            .filter(|paragraph| !paragraph.trim().is_empty())
            .count()
            >= 2
        {
            return true;
        }

        preview_text.lines().map(|line| line.trim()).any(|line| {
            line.starts_with("```")
                || line.starts_with('#')
                || line.starts_with("- ")
                || line.starts_with("* ")
                || line.starts_with("1. ")
                || line.starts_with("2. ")
                || line.starts_with("3. ")
        })
    }

    fn extract_sidebar_preview_line_and_role(preview_text: &str) -> (Option<MessageRole>, String) {
        let mut prefixed_role = None;

        for raw_line in preview_text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((role, without_prefix)) = Self::parse_sidebar_preview_prefixed_line(line) {
                prefixed_role.get_or_insert(role);
                if Self::is_visible_sidebar_preview_line(without_prefix) {
                    return (prefixed_role, without_prefix.to_string());
                }
                continue;
            }

            if Self::is_visible_sidebar_preview_line(line) {
                return (prefixed_role, line.to_string());
            }
        }

        (prefixed_role, String::new())
    }

    fn fallback_sidebar_preview_line(preview_text: &str) -> String {
        for raw_line in preview_text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((_role, without_prefix)) = Self::parse_sidebar_preview_prefixed_line(line) {
                if !without_prefix.is_empty() {
                    return without_prefix.to_string();
                }
                continue;
            }

            return line.to_string();
        }

        String::new()
    }

    fn parse_sidebar_preview_prefixed_line(line: &str) -> Option<(MessageRole, &str)> {
        let trimmed = line.trim();
        if let Some((token, remainder)) = trimmed.split_once(':') {
            if let Some(role) = Self::parse_sidebar_role_token(token) {
                return Some((role, remainder.trim_start()));
            }
        }
        if let Some((token, remainder)) = trimmed.split_once(" - ") {
            if let Some(role) = Self::parse_sidebar_role_token(token) {
                return Some((role, remainder.trim_start()));
            }
        }

        None
    }

    fn parse_sidebar_role_token(token: &str) -> Option<MessageRole> {
        let normalized = token
            .trim()
            .trim_matches(|c: char| {
                c.is_whitespace()
                    || c == '*'
                    || c == '_'
                    || c == '`'
                    || c == '['
                    || c == ']'
                    || c == '('
                    || c == ')'
                    || c == '{'
                    || c == '}'
                    || c == '>'
                    || c == '-'
            })
            .to_ascii_lowercase();

        match normalized.as_str() {
            "you" | "user" => Some(MessageRole::User),
            "assistant" | "ai" | "bot" => Some(MessageRole::Assistant),
            "system" | "sys" => Some(MessageRole::System),
            _ => None,
        }
    }

    fn is_visible_sidebar_preview_line(line: &str) -> bool {
        !line.is_empty()
            && !line.starts_with('#')
            && !line.starts_with("```")
            && !line.chars().all(|c| c == '-' || c == '*' || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sidebar_preview_line_and_role_strips_common_prefixes_when_present() {
        let (role, preview_line) =
            AiApp::extract_sidebar_preview_line_and_role("**Assistant**: Here is the summary");

        assert_eq!(role, Some(MessageRole::Assistant));
        assert_eq!(preview_line, "Here is the summary");
    }

    #[test]
    fn test_extract_sidebar_preview_line_and_role_uses_following_line_when_prefix_is_empty() {
        let (role, preview_line) =
            AiApp::extract_sidebar_preview_line_and_role("You:\n\nCan you review this?");

        assert_eq!(role, Some(MessageRole::User));
        assert_eq!(preview_line, "Can you review this?");
    }

    #[test]
    fn test_resolve_sidebar_preview_role_uses_selected_last_role_when_available() {
        let role = AiApp::resolve_sidebar_preview_role(
            Some(MessageRole::System),
            2,
            "Assistant: this should not win",
        );

        assert_eq!(role, MessageRole::System);
    }

    #[test]
    fn test_resolve_sidebar_preview_role_falls_back_to_assistant_for_multiparagraph_continuation() {
        let role = AiApp::resolve_sidebar_preview_role(
            None,
            3,
            "First paragraph of a long assistant answer.\n\nSecond paragraph continues context.",
        );

        assert_eq!(role, MessageRole::Assistant);
    }

    #[test]
    fn test_resolve_sidebar_preview_role_uses_turn_parity_when_no_other_hints_exist() {
        let assistant_role = AiApp::resolve_sidebar_preview_role(None, 2, "Looks good.");
        let user_role = AiApp::resolve_sidebar_preview_role(None, 3, "Can you help?");

        assert_eq!(assistant_role, MessageRole::Assistant);
        assert_eq!(user_role, MessageRole::User);
    }

    #[test]
    fn test_ai_sidebar_row_hover_enabled_only_for_unselected_rows_in_mouse_mode() {
        assert!(ai_sidebar_row_hover_enabled(InputMode::Mouse, false));
        assert!(!ai_sidebar_row_hover_enabled(InputMode::Mouse, true));
        assert!(!ai_sidebar_row_hover_enabled(InputMode::Keyboard, false));
    }

    #[test]
    fn test_ai_sidebar_delete_hover_enabled_only_in_mouse_mode() {
        assert!(ai_sidebar_delete_hover_enabled(InputMode::Mouse));
        assert!(!ai_sidebar_delete_hover_enabled(InputMode::Keyboard));
    }
}
