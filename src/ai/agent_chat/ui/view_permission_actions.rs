impl AgentChatView {
    fn permission_request_tool_call_id(request: &AgentChatApprovalRequest) -> Option<&str> {
        let tool_call_id = request.preview.as_ref()?.tool_call_id.trim();
        if tool_call_id.is_empty() {
            None
        } else {
            Some(tool_call_id)
        }
    }

    fn permission_request_matches_message(
        msg: &AgentChatThreadMessage,
        request: &AgentChatApprovalRequest,
    ) -> bool {
        msg.tool_call_id
            .as_deref()
            .zip(Self::permission_request_tool_call_id(request))
            .is_some_and(|(msg_id, request_id)| msg_id == request_id)
    }

    fn selected_permission_option<'a>(
        &self,
        request: &'a AgentChatApprovalRequest,
    ) -> Option<(usize, &'a AgentChatApprovalOption)> {
        let index = self.normalized_permission_index(request.options.len());
        request.options.get(index).map(|option| (index, option))
    }

    fn first_allow_once_option(
        request: &AgentChatApprovalRequest,
    ) -> Option<(usize, &AgentChatApprovalOption)> {
        request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| !option.is_reject() && !option.is_persistent_allow())
    }

    fn first_allow_option(
        request: &AgentChatApprovalRequest,
    ) -> Option<(usize, &AgentChatApprovalOption)> {
        request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| !option.is_reject())
    }

    fn first_reject_option(
        request: &AgentChatApprovalRequest,
    ) -> Option<(usize, &AgentChatApprovalOption)> {
        request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| option.is_reject())
    }

    fn preferred_allow_option<'a>(
        &self,
        request: &'a AgentChatApprovalRequest,
    ) -> Option<(usize, &'a AgentChatApprovalOption)> {
        match self.selected_permission_option(request) {
            Some((index, option)) if !option.is_reject() => Some((index, option)),
            _ => {
                Self::first_allow_once_option(request).or_else(|| Self::first_allow_option(request))
            }
        }
    }

    fn approve_preferred_allow_option(
        &mut self,
        request: &AgentChatApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some((index, option)) = self.preferred_allow_option(request) {
            self.permission_index = index;
            self.approve_permission(Some(option.option_id.clone()), cx);
            true
        } else {
            false
        }
    }

    fn approve_reject_option(
        &mut self,
        request: &AgentChatApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some((index, option)) = Self::first_reject_option(request) {
            self.permission_index = index;
            self.approve_permission(Some(option.option_id.clone()), cx);
            true
        } else {
            self.approve_permission(None, cx);
            true
        }
    }

    fn toggle_permission_options(
        &mut self,
        request: &AgentChatApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if request.options.len() <= 1 {
            return false;
        }

        if !self.permission_options_open {
            if let Some((index, _)) = self.preferred_allow_option(request) {
                self.permission_index = index;
            }
        }

        self.permission_options_open = !self.permission_options_open;
        self.notify_semantic_change(cx);
        true
    }

    fn normalized_permission_index(&self, option_count: usize) -> usize {
        if option_count == 0 {
            0
        } else {
            self.permission_index.min(option_count - 1)
        }
    }

    fn step_permission_index(current: usize, option_count: usize, reverse: bool) -> usize {
        if option_count == 0 {
            return 0;
        }

        if reverse {
            if current == 0 {
                option_count - 1
            } else {
                current - 1
            }
        } else {
            (current + 1) % option_count
        }
    }

    /// Handle key events when an inline permission card is active.
    /// Returns `true` if the key was consumed.
    fn handle_permission_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        request: &AgentChatApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;
        let option_count = request.options.len();
        self.permission_index = self.normalized_permission_index(option_count);

        if modifiers.platform
            && !modifiers.alt
            && !modifiers.control
            && key.eq_ignore_ascii_case("y")
        {
            return self.approve_preferred_allow_option(request, cx);
        }

        if modifiers.platform
            && modifiers.alt
            && !modifiers.control
            && key.eq_ignore_ascii_case("a")
        {
            self.toggle_permission_options(request, cx);
            return true;
        }

        if modifiers.platform
            && modifiers.alt
            && !modifiers.control
            && key.eq_ignore_ascii_case("z")
        {
            return self.approve_reject_option(request, cx);
        }

        if crate::ui_foundation::is_key_up(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, true);
            self.permission_options_open = option_count > 1;
            self.notify_semantic_change(cx);
            return true;
        }

        if crate::ui_foundation::is_key_down(key) {
            self.permission_index =
                Self::step_permission_index(self.permission_index, option_count, false);
            self.permission_options_open = option_count > 1;
            self.notify_semantic_change(cx);
            return true;
        }

        // J/K navigation (vim-style, unmodified only)
        match key {
            "j" | "J" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, false);
                self.permission_options_open = option_count > 1;
                self.notify_semantic_change(cx);
                return true;
            }
            "k" | "K" => {
                self.permission_index =
                    Self::step_permission_index(self.permission_index, option_count, true);
                self.permission_options_open = option_count > 1;
                self.notify_semantic_change(cx);
                return true;
            }
            _ => {}
        }

        if crate::ui_foundation::is_key_escape(key) && self.permission_options_open {
            self.permission_options_open = false;
            self.notify_semantic_change(cx);
            return true;
        }

        if crate::ui_foundation::is_key_escape(key) {
            self.approve_permission(None, cx);
            return true;
        }

        if crate::ui_foundation::is_key_enter(key) {
            if let Some(option) = request
                .options
                .get(self.normalized_permission_index(option_count))
            {
                self.approve_permission(Some(option.option_id.clone()), cx);
            } else {
                let _ = self.approve_preferred_allow_option(request, cx);
            }
            return true;
        }

        // 1-9 instant pick
        if let Ok(digit) = key.parse::<usize>() {
            if digit >= 1 {
                let idx = digit - 1;
                if let Some(option) = request.options.get(idx) {
                    self.permission_index = idx;
                    self.approve_permission(Some(option.option_id.clone()), cx);
                    return true;
                }
            }
        }

        false
    }

    fn render_permission_section(title: &'static str, text: String) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();

        div()
            .pt(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .opacity(0.48)
                    .child(title),
            )
            .child(
                div()
                    .mt(px(4.0))
                    .max_h(px(120.0))
                    .overflow_y_hidden()
                    .border_l_2()
                    .border_color(rgba((theme.colors.ui.border << 8) | 0x18))
                    .bg(rgba((theme.colors.text.primary << 8) | 0x04))
                    .pl(px(10.0))
                    .pr(px(8.0))
                    .py(px(6.0))
                    .text_xs()
                    .opacity(0.76)
                    .child(text),
            )
            .into_any_element()
    }

    fn permission_preview_chrome(
        kind: AgentChatApprovalPreviewKind,
        theme: &crate::theme::Theme,
    ) -> PermissionPreviewChrome {
        let chrome = AppChromeColors::from_theme(theme);
        let base_hex = match kind {
            AgentChatApprovalPreviewKind::Read => theme.colors.text.primary,
            AgentChatApprovalPreviewKind::Write => theme.colors.accent.selected,
            AgentChatApprovalPreviewKind::Execute => theme.colors.ui.warning,
            AgentChatApprovalPreviewKind::Generic => theme.colors.ui.border,
        };

        PermissionPreviewChrome {
            badge: chrome.semantic_chip_colors(theme, base_hex),
            accent_rgba: crate::ui_foundation::hex_to_rgba_with_opacity(
                base_hex,
                theme.get_opacity().text_strong,
            ),
            title_text_rgba: chrome.text_strong_rgba,
            subject_text_rgba: chrome.text_muted_rgba,
        }
    }

    fn render_permission_header(preview: &AgentChatApprovalPreview) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let chrome = Self::permission_preview_chrome(preview.kind, &theme);

        div()
            .pt(px(6.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .px(px(7.0))
                            .py(px(3.0))
                            .rounded(px(999.0))
                            .bg(rgba(chrome.badge.bg_rgba))
                            .border_1()
                            .border_color(rgba(chrome.badge.border_rgba))
                            .text_xs()
                            .text_color(rgb(chrome.badge.text_hex))
                            .child(preview.kind.badge_label()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgba(chrome.title_text_rgba))
                            .child(preview.tool_title.clone()),
                    ),
            )
            .when_some(preview.subject.clone(), |d, subject| {
                d.child(
                    div()
                        .pt(px(4.0))
                        .text_sm()
                        .text_color(rgba(chrome.subject_text_rgba))
                        .child(subject),
                )
            })
            .into_any_element()
    }

    fn render_permission_option_row(
        option: &AgentChatApprovalOption,
        index: usize,
        is_selected: bool,
        view: WeakEntity<AgentChatView>,
    ) -> gpui::AnyElement {
        let theme = theme::get_cached_theme();
        let chrome = AppChromeColors::from_theme(&theme);
        let deny_colors = crate::theme::DangerActionColors::from_theme(&theme);
        let option_id = option.option_id.clone();

        let (bg, hover_bg, caption) = if option.is_reject() {
            (
                rgba(if is_selected {
                    deny_colors.hover_rgba
                } else {
                    deny_colors.rest_rgba
                }),
                rgba(deny_colors.hover_rgba),
                "Deny this request",
            )
        } else if option.is_persistent_allow() {
            (
                rgba(if is_selected {
                    chrome.accent_badge_border_rgba
                } else {
                    chrome.accent_badge_bg_rgba
                }),
                rgba(chrome.accent_badge_bg_rgba),
                "Remember this choice",
            )
        } else {
            (
                rgba(if is_selected {
                    chrome.accent_badge_bg_rgba
                } else {
                    chrome.whisper_surface_rgba
                }),
                rgba(chrome.hover_rgba),
                "Approve once",
            )
        };

        div()
            .id(SharedString::from(format!("perm-opt-{index}")))
            .mt(px(4.0))
            .pl(px(10.0))
            .pr(px(6.0))
            .py(px(6.0))
            .cursor_pointer()
            .bg(bg)
            .hover(move |d| d.bg(hover_bg))
            .on_click(move |_event, _window, cx| {
                if let Some(entity) = view.upgrade() {
                    entity.update(cx, |this, cx| {
                        this.permission_index = index;
                        this.approve_permission(Some(option_id.clone()), cx);
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(option.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba(chrome.placeholder_text_rgba))
                            .child(format!("{}", index + 1)),
                    ),
            )
            .child(
                div()
                    .pt(px(2.0))
                    .text_xs()
                    .text_color(rgba(chrome.placeholder_text_rgba))
                    .child(caption),
            )
            .into_any_element()
    }

    fn render_permission_inline_card(
        request: &AgentChatApprovalRequest,
        selected_index: usize,
        options_open: bool,
        view: WeakEntity<AgentChatView>,
    ) -> gpui::AnyElement {
        use gpui_ai::prelude::{ApprovalCard, ApprovalEvent, ApprovalTone};

        let preview = request.preview.clone();
        let selected_index = selected_index.min(request.options.len().saturating_sub(1));
        let has_persistent_allow = request
            .options
            .iter()
            .any(|option| option.is_persistent_allow());
        let show_options_button = request.options.len() > 2;
        let shortcut_hint = if has_persistent_allow || show_options_button {
            "\u{2318}Y Allow \u{00b7} \u{2318}\u{2325}A Options \u{00b7} \u{2318}\u{2325}Z Deny"
        } else {
            "\u{2318}Y Allow \u{00b7} \u{2318}\u{2325}Z Deny \u{00b7} Esc Cancel"
        };
        let allow_label = Self::preferred_allow_label(request, selected_index);
        let reject_label = Self::first_reject_option(request)
            .map(|(_, option)| option.name.clone())
            .unwrap_or_else(|| "Deny".to_string());

        let event_request = request.clone();
        let event_view = view.clone();
        let mut card = ApprovalCard::new("agent_chat-inline-permission-card", request.title.clone())
            .tone(ApprovalTone::Default)
            .allow_always(has_persistent_allow)
            .approve_label(allow_label)
            .reject_label(reject_label)
            .note(shortcut_hint)
            .on_event(move |event, _window, cx| {
                let Some(entity) = event_view.upgrade() else {
                    return;
                };
                let request = event_request.clone();
                entity.update(cx, |this, cx| match event {
                    ApprovalEvent::Approved { .. } => {
                        let _ = this.approve_preferred_allow_option(&request, cx);
                    }
                    ApprovalEvent::ApprovedAlways { .. } => {
                        let _ = this.approve_persistent_allow_option(&request, cx);
                    }
                    ApprovalEvent::Rejected { .. } => {
                        let _ = this.approve_reject_option(&request, cx);
                    }
                });
            });

        if let Some(preview) = preview.clone() {
            if let Some(summary) = preview.summary.clone() {
                card = card.description(summary);
            }
            card = card.child(Self::render_permission_header(&preview));
            if let Some(input) = preview.input_preview {
                card = card.child(Self::render_permission_section("Input", input));
            }
            if let Some(output) = preview.output_preview {
                card = card.child(Self::render_permission_section("Output", output));
            }
        } else {
            card = card.description(request.body.clone());
        }

        if show_options_button {
            let options_request = request.clone();
            let options_view = view.clone();
            card = card.child(
                div()
                    .id("agent_chat-inline-permission-options")
                    .pt(px(6.0))
                    .text_xs()
                    .opacity(0.72)
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        if let Some(entity) = options_view.upgrade() {
                            entity.update(cx, |this, cx| {
                                let _ = this.toggle_permission_options(&options_request, cx);
                            });
                        }
                    })
                    .child(if options_open {
                        "More options \u{2318}\u{2325}A \u{25BE}"
                    } else {
                        "More options \u{2318}\u{2325}A \u{25B8}"
                    }),
            );
        }
        if options_open && request.options.len() > 1 {
            card = card.child(div().pt(px(6.0)).children(request.options.iter().enumerate().map(
                |(i, option)| {
                    Self::render_permission_option_row(option, i, i == selected_index, view.clone())
                },
            )));
        }

        div()
            .debug_selector(|| "agent_chat-inline-permission-card".to_string())
            .w_full()
            .mt(px(6.0))
            .child(card)
            .into_any_element()
    }

    fn preferred_allow_label(request: &AgentChatApprovalRequest, selected_index: usize) -> String {
        request
            .options
            .get(selected_index)
            .filter(|option| !option.is_reject() && !option.is_persistent_allow())
            .or_else(|| Self::first_allow_once_option(request).map(|(_, option)| option))
            .map(|option| option.name.clone())
            .unwrap_or_else(|| "Allow".to_string())
    }

    fn approve_persistent_allow_option(
        &mut self,
        request: &AgentChatApprovalRequest,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some((index, option)) = request
            .options
            .iter()
            .enumerate()
            .find(|(_, option)| option.is_persistent_allow())
        {
            self.permission_index = index;
            self.approve_permission(Some(option.option_id.clone()), cx);
            true
        } else {
            self.approve_preferred_allow_option(request, cx)
        }
    }
}
