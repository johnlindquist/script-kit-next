use super::*;

impl AiApp {
    pub(super) fn render_sidebar_group_header(
        &self,
        group: DateGroup,
        is_first_group: bool,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let group_label: SharedString = group.label().to_uppercase().into();
        div()
            .flex()
            .w_full()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(cx.theme().muted_foreground.opacity(0.6))
            .px_1()
            .pt(px(if is_first_group { 4. } else { 12. }))
            .pb(px(6.))
            .child(group_label)
    }

    /// Render a single chat item with title, relative time, and hover-revealed delete button
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
        let is_external_source = matches!(chat.source, ChatSource::ChatPrompt | ChatSource::Script);

        // Derive short model label from model_id (e.g., "claude-3-5-sonnet..." → "Sonnet")
        let model_badge: Option<SharedString> = if !chat.model_id.is_empty() {
            let short = Self::abbreviate_model_name(&chat.model_id);
            Some(short.into())
        } else {
            None
        };

        // Relative time for this chat
        let relative_time: SharedString = {
            let now = Utc::now();
            let diff = now - chat.updated_at;
            if diff.num_minutes() < 1 {
                "now".into()
            } else if diff.num_minutes() < 60 {
                format!("{}m", diff.num_minutes()).into()
            } else if diff.num_hours() < 24 {
                format!("{}h", diff.num_hours()).into()
            } else if diff.num_days() < 7 {
                format!("{}d", diff.num_days()).into()
            } else {
                chat.updated_at.format("%b %d").to_string().into()
            }
        };
        let full_chat_time: SharedString = chat
            .updated_at
            .format("%B %-d, %Y at %-I:%M %p")
            .to_string()
            .into();

        let selected_bg = cx.theme().muted.opacity(0.7);
        let hover_bg = cx.theme().muted.opacity(0.5);

        let title_color = if is_selected {
            cx.theme().foreground
        } else {
            cx.theme().sidebar_foreground
        };
        let description_color = if is_selected {
            cx.theme().sidebar_foreground
        } else {
            cx.theme().muted_foreground
        };

        let muted_fg = cx.theme().muted_foreground;
        let is_renaming = self.renaming_chat_id == Some(chat_id);
        div()
            .id(SharedString::from(format!("chat-{}", chat_id)))
            .group("chat-item")
            .flex()
            .flex_col()
            .w_full()
            .px_2()
            .py(px(6.))
            .rounded(px(8.))
            .cursor_pointer()
            .when(is_selected, |d| d.bg(selected_bg))
            .when(!is_selected, |d| d.hover(|d| d.bg(hover_bg)))
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
                // Title row with relative time and hover-revealed delete
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .gap(px(4.))
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
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(title_color)
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(title),
                        )
                    })
                    // Relative time - hidden on hover to make room for delete
                    .child({
                        let tooltip_text = full_chat_time.clone();
                        div()
                            .id(SharedString::from(format!("time-{}", chat_id)))
                            .flex_shrink_0()
                            .text_xs()
                            .text_color(description_color.opacity(0.6))
                            .group_hover("chat-item", |s| s.opacity(0.))
                            .tooltip(move |window, cx| {
                                Tooltip::new(tooltip_text.clone()).build(window, cx)
                            })
                            .child(relative_time)
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
                                .px(px(4.))
                                .py(px(1.))
                                .rounded(px(4.))
                                .flex_shrink_0()
                                .cursor_pointer()
                                .bg(cx.theme().danger.opacity(0.15))
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .hover(|s| s.bg(cx.theme().danger.opacity(0.25)))
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
                                .size(px(18.))
                                .rounded(px(4.))
                                .flex_shrink_0()
                                .cursor_pointer()
                                .opacity(0.)
                                .group_hover("chat-item", |s| s.opacity(1.0))
                                .hover(|s| s.bg(cx.theme().danger.opacity(0.19)))
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
                                        .size(px(12.))
                                        .text_color(muted_fg.opacity(0.5)),
                                )
                        }
                    }),
            )
            .when_some(preview, |d, preview_text| {
                let (preview_role, preview_snippet) =
                    self.build_sidebar_preview_identity(chat_id, msg_count, &preview_text);
                let role_icon = Self::sidebar_preview_sender_icon(preview_role);
                let role_label = Self::sidebar_preview_sender_label(preview_role);
                let (role_accent, role_tint, role_label_color) =
                    Self::sidebar_preview_palette(preview_role, cx);

                d.child(
                    div()
                        .mt(SP_2)
                        .min_w_0()
                        .rounded(px(6.))
                        .border_l_2()
                        .border_color(role_accent)
                        .bg(role_tint)
                        .pl(SP_2)
                        .pr(SP_2)
                        .py(SP_1)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(SP_2)
                                .child(
                                    svg()
                                        .external_path(role_icon.external_path())
                                        .size(px(9.))
                                        .text_color(role_label_color),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(role_label_color)
                                        .child(role_label),
                                ),
                        )
                        .child(
                            div()
                                .mt(px(2.))
                                .text_xs()
                                .text_color(description_color)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(preview_snippet),
                        ),
                )
            })
            // Bottom row: model badge + message count + source indicator
            .when(
                model_badge.is_some() || msg_count > 0 || is_external_source,
                |d| {
                    d.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .when(is_external_source, |d| {
                                let source_label: &str = match chat.source {
                                    ChatSource::ChatPrompt => "Prompt",
                                    ChatSource::Script => "Script",
                                    _ => "",
                                };
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(2.))
                                        .child(
                                            svg()
                                                .external_path(
                                                    LocalIconName::Terminal.external_path(),
                                                )
                                                .size(px(10.))
                                                .text_color(cx.theme().accent.opacity(0.4)),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().accent.opacity(0.4))
                                                .child(source_label),
                                        ),
                                )
                            })
                            .when_some(model_badge, |d, badge| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(0.45))
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .child(badge),
                                )
                            })
                            .when(msg_count > 0, |d| {
                                let count_label = if msg_count == 1 {
                                    "1 msg".to_string()
                                } else {
                                    format!("{} msgs", msg_count)
                                };
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground.opacity(0.35))
                                        .child(count_label),
                                )
                            }),
                    )
                },
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
        } else if msg_count % 2 == 0 {
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

    fn sidebar_preview_sender_label(role: MessageRole) -> &'static str {
        match role {
            MessageRole::User => "You",
            MessageRole::Assistant => "Assistant",
            MessageRole::System => "System",
        }
    }

    fn sidebar_preview_sender_icon(role: MessageRole) -> LocalIconName {
        match role {
            MessageRole::User => LocalIconName::Terminal,
            MessageRole::Assistant => LocalIconName::MessageCircle,
            MessageRole::System => LocalIconName::Settings,
        }
    }

    fn sidebar_preview_palette(
        role: MessageRole,
        cx: &Context<Self>,
    ) -> (gpui::Hsla, gpui::Hsla, gpui::Hsla) {
        match role {
            MessageRole::User => (
                cx.theme().accent.opacity(OP_MSG_BORDER),
                cx.theme().accent.opacity(OP_USER_MSG_BG),
                cx.theme().accent.opacity(OP_STRONG),
            ),
            MessageRole::Assistant => (
                cx.theme().muted_foreground.opacity(OP_MEDIUM),
                cx.theme().muted.opacity(OP_ASSISTANT_MSG_BG),
                cx.theme().muted_foreground.opacity(OP_STRONG),
            ),
            MessageRole::System => (
                cx.theme().muted_foreground.opacity(OP_STRONG),
                cx.theme().muted.opacity(OP_ASSISTANT_MSG_BG + 0.04),
                cx.theme().muted_foreground.opacity(OP_NEAR_FULL),
            ),
        }
    }

    /// Abbreviate a model ID to a short display label for sidebar badges.
    /// e.g., "claude-3-5-sonnet-20241022" → "Sonnet"
    /// e.g., "gpt-4o-mini" → "GPT-4o Mini"
    /// e.g., "claude-3-5-haiku-20241022" → "Haiku"
    pub(super) fn abbreviate_model_name(model_id: &str) -> String {
        let lower = model_id.to_lowercase();
        if lower.contains("sonnet") {
            "Sonnet".to_string()
        } else if lower.contains("haiku") {
            "Haiku".to_string()
        } else if lower.contains("opus") {
            "Opus".to_string()
        } else if lower.contains("gpt-4o-mini") {
            "GPT-4o Mini".to_string()
        } else if lower.contains("gpt-4o") {
            "GPT-4o".to_string()
        } else if lower.contains("gpt-4") {
            "GPT-4".to_string()
        } else if lower.contains("gpt-3") {
            "GPT-3.5".to_string()
        } else if lower.contains("o1") || lower.contains("o3") {
            // OpenAI reasoning models
            let parts: Vec<&str> = model_id.split('-').collect();
            parts.first().unwrap_or(&model_id).to_uppercase()
        } else {
            // Fallback: take the most descriptive part
            let parts: Vec<&str> = model_id.split('-').collect();
            if parts.len() > 1 {
                // Skip version-like parts (dates, numbers)
                parts
                    .iter()
                    .find(|p| p.len() > 2 && !p.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(&parts[0])
                    .to_string()
            } else {
                model_id.to_string()
            }
        }
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
}
