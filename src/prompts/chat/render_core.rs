use super::*;
use crate::components::{HintStrip, PromptFooter, PromptFooterColors, PromptFooterConfig};
use crate::ui::chrome::{
    alpha_from_opacity, ChromeStyle, HEADER_PADDING_X, HEADER_PADDING_Y, HINT_TEXT_OPACITY,
};

impl ChatPrompt {
    fn render_script_generation_hint_button(
        &self,
        id: &'static str,
        label: &'static str,
        action: ScriptGenerationAction,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme_colors = &self.theme.colors;
        let hint_text_rgba =
            (theme_colors.text.primary << 8) | alpha_from_opacity(HINT_TEXT_OPACITY);

        div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .cursor_pointer()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgba(hint_text_rgba))
            .hover(move |d| d.text_color(rgb(theme_colors.text.primary)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(theme_colors.accent.selected))
                    .child(label),
            )
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.handle_script_generation_action(action, cx);
            }))
            .into_any_element()
    }

    fn footer_hint_text(&self) -> gpui::SharedString {
        if let Some(text) = self.footer.clone() {
            return text.into();
        }

        if self.is_streaming() {
            "Esc Stop · ⌘K Actions".into()
        } else if self.script_generation_mode {
            "↵ Send · ⌘K Actions · Esc Back · ⌘↵ Save+Run".into()
        } else {
            "↵ Send · ⌘K Actions · Esc Back".into()
        }
    }

    fn render_minimal_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        HintStrip::new(self.footer_hint_text()).leading(
            self.render_script_generation_hint_actions(cx)
                .unwrap_or_else(|| div().min_w(px(0.0)).into_any_element()),
        )
    }

    fn render_rich_footer(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = PromptFooterColors::from_theme(&self.theme);
        let helper_text = self
            .hint
            .clone()
            .unwrap_or_else(|| "Shift+Enter newline".to_string());
        let footer_text = self.footer_hint_text().to_string();

        PromptFooter::new(
            PromptFooterConfig::new()
                .primary_label("Continue in Chat")
                .primary_shortcut("⌘↵")
                .secondary_label("Actions")
                .secondary_shortcut("⌘K")
                .helper_text(format!("{helper_text} · {footer_text}"))
                .show_logo(false)
                .show_info_label(false),
            colors,
        )
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match self.chrome {
            ChromeStyle::Minimal => self.render_minimal_footer(cx).into_any_element(),
            ChromeStyle::Rich => self.render_rich_footer(cx).into_any_element(),
        }
    }

    fn render_script_generation_hint_actions(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let show_actions = self.should_show_script_generation_actions();
        let status_message = self.script_generation_status.clone();

        if !show_actions && status_message.is_none() {
            return None;
        }

        let theme_colors = &self.theme.colors;

        let mut action_container = div()
            .id("chat-script-generation-hint-actions")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .min_w(px(0.0));

        if show_actions {
            action_container = action_container
                .child(self.render_script_generation_hint_button(
                    "chat-script-generation-save",
                    "Save",
                    ScriptGenerationAction::Save,
                    cx,
                ))
                .child(self.render_script_generation_hint_button(
                    "chat-script-generation-run",
                    "Run",
                    ScriptGenerationAction::Run,
                    cx,
                ));
        }

        if let Some(status) = status_message {
            let status_color = if self.script_generation_status_is_error {
                theme_colors.ui.error
            } else {
                theme_colors.ui.success
            };

            action_container = action_container.child(
                div()
                    .id("chat-script-generation-status")
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(status_color))
                    .max_w(px(180.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(status),
            );
        }

        Some(action_container.into_any_element())
    }
}

impl Focusable for ChatPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ChatPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pending_auto_focus {
            self.pending_auto_focus = false;
            self.focus_handle.focus(window, cx);
        }

        // In setup mode, ensure focus handle is focused so keyboard events route here
        if self.needs_setup {
            self.focus_handle.focus(window, cx);
        }

        // Start cursor blink timer on first render (only needed when not in setup mode)
        if !self.needs_setup && !self.cursor_blink_started {
            self.cursor_blink_started = true;
            self.start_cursor_blink(cx);
        }

        // Process pending_submit on first render (used when Tab opens chat with query)
        // Skip if in setup mode or while providers are still loading
        if !self.needs_setup
            && !self.loading_providers
            && self.pending_submit
            && !self.input.is_empty()
        {
            self.pending_submit = false;
            logging::log(
                "CHAT",
                "Processing pending_submit - auto-submitting query from Tab",
            );
            self.handle_submit(cx);
        }

        // Process needs_initial_response on first render (used for scriptlets with pre-populated messages)
        // Skip if in setup mode or loading providers, requires built-in AI to be enabled
        if !self.needs_setup
            && !self.loading_providers
            && self.needs_initial_response
            && self.has_builtin_ai()
        {
            self.needs_initial_response = false;
            logging::log(
                "CHAT",
                "Processing needs_initial_response - auto-responding to initial messages",
            );
            self.handle_initial_response(cx);
        }

        self.ensure_conversation_turns_cache();
        let theme_colors = &self.theme.colors;

        let needs_setup = self.needs_setup;
        let on_configure = self.on_configure.clone();
        let on_claude_code = self.on_claude_code.clone();

        let handle_key = cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let key_lower = event.keystroke.key.to_ascii_lowercase();
            let key_char = event.keystroke.key_char.as_deref();
            let modifiers = &event.keystroke.modifiers;

            // Setup mode: keyboard navigation for Configure / Claude Code buttons
            if needs_setup {
                let (next_index, action, changed) = resolve_setup_card_key(
                    key,
                    event.keystroke.modifiers.shift,
                    this.setup_focus_index,
                );
                let handled = changed || !matches!(action, SetupCardAction::None);

                if changed {
                    this.setup_focus_index = next_index;
                    cx.notify();
                }

                match action {
                    SetupCardAction::ActivateConfigure => {
                        if let Some(ref callback) = on_configure {
                            logging::log("CHAT", "Setup key activate configure");
                            callback();
                        }
                    }
                    SetupCardAction::ActivateClaudeCode => {
                        if let Some(ref callback) = on_claude_code {
                            logging::log("CHAT", "Setup key activate Claude Code");
                            callback();
                        }
                    }
                    SetupCardAction::Escape => this.handle_escape(cx),
                    SetupCardAction::None => {}
                }

                if handled {
                    cx.stop_propagation();
                }
                return;
            }

            // Note: Actions menu keyboard navigation is handled by ActionsDialog window
            // We just need to handle ⌘K to open it via callback

            match resolve_chat_input_key_action(key, modifiers.platform, modifiers.shift) {
                ChatInputKeyAction::Escape => {
                    // Escape - stop streaming if active, otherwise close chat
                    if this.is_streaming() {
                        this.stop_streaming(cx);
                    } else {
                        this.handle_escape(cx);
                    }
                }
                ChatInputKeyAction::StopStreaming => {
                    if this.is_streaming() {
                        this.stop_streaming(cx);
                    }
                }
                ChatInputKeyAction::JumpToLatest => {
                    this.force_scroll_turns_to_bottom();
                    this.user_has_scrolled_up = false;
                    cx.notify();
                }
                ChatInputKeyAction::ToggleActions => this.toggle_actions_menu(cx),
                ChatInputKeyAction::ContinueInChat => {
                    if this.script_generation_mode {
                        this.handle_script_generation_action(
                            ScriptGenerationAction::SaveAndRun,
                            cx,
                        );
                    } else {
                        this.handle_continue_in_chat(cx);
                    }
                }
                ChatInputKeyAction::Submit => this.handle_submit(cx),
                ChatInputKeyAction::InsertNewline => {
                    this.input.insert_char('\n');
                    this.reset_cursor_blink();
                    cx.notify();
                }
                ChatInputKeyAction::CopyLastResponse => this.handle_copy_last_response(cx),
                ChatInputKeyAction::ClearConversation => this.handle_clear(cx),
                ChatInputKeyAction::Paste => {
                    if !this.handle_paste_for_image(cx) {
                        this.paste_text_from_clipboard(cx);
                    }
                }
                ChatInputKeyAction::DelegateToInput => {
                    let handled = this.input.handle_key(
                        key_lower.as_str(),
                        key_char,
                        modifiers.platform, // Cmd key on macOS
                        modifiers.alt,
                        modifiers.shift,
                        cx,
                    );

                    if handled {
                        this.reset_cursor_blink();
                        cx.notify();
                    }
                }
                ChatInputKeyAction::Ignore => {}
            }
        });

        let container_bg: Option<Hsla> = get_vibrancy_background(&self.theme).map(Hsla::from);
        let input_is_focused = self.focus_handle.is_focused(window);

        // If needs_setup, render setup card instead of normal chat
        if self.needs_setup {
            return div()
                .id("chat-prompt-setup")
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .when_some(container_bg, |d, bg| d.bg(bg))
                .key_context("chat_prompt_setup")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                // Setup card content
                .child(self.render_setup_card(cx))
                .into_any_element();
        }

        // If loading_providers, show a "Connecting to AI..." placeholder
        if self.loading_providers {
            return div()
                .id("chat-prompt-loading")
                .flex()
                .flex_col()
                .w_full()
                .h_full()
                .when_some(container_bg, |d, bg| d.bg(bg))
                .key_context("chat_prompt_loading")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme_colors.text.secondary))
                                .child("Connecting to AI..."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme_colors.text.tertiary))
                                .child("Loading providers and models"),
                        ),
                )
                .into_any_element();
        }

        // Input area at TOP
        let has_pending_image = self.pending_image.is_some();
        let input_area = div()
            .w_full()
            .px(px(HEADER_PADDING_X))
            .py(px(HEADER_PADDING_Y))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(crate::components::SectionDivider::new().id("chat-input-divider"))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            .when(has_pending_image, |d| {
                d.child(self.render_pending_image_preview(cx))
            })
            .child(self.render_input(input_is_focused));

        // Message list (conversation turns) - virtualized for large chats
        let has_turns = !self.conversation_turns_cache.is_empty();
        let messages_content = if has_turns {
            let entity = cx.entity();
            // TODO(codex-audit): This Vec clone exists to move turns into the list closure.
            // Consider Arc-backed snapshots to avoid per-render cloning.
            let turns_snapshot = self.conversation_turns_cache.clone();
            let show_scroll_to_latest =
                self.user_has_scrolled_up && !self.turns_list_is_at_bottom();
            let turns_list = list(self.turns_list_state.clone(), move |ix, _window, cx| {
                entity.update(cx, |this, cx| {
                    if let Some(turn) = turns_snapshot.get(ix) {
                        div()
                            .w_full()
                            .pb(px(8.0))
                            .child(this.render_turn(turn, ix, cx))
                            .into_any_element()
                    } else {
                        div().w_full().into_any_element()
                    }
                })
            })
            .with_sizing_behavior(ListSizingBehavior::Infer)
            .size_full()
            .px(px(HEADER_PADDING_X))
            .py(px(HEADER_PADDING_Y));

            div()
                .id("chat-messages")
                .relative()
                .flex_1()
                .min_h(px(0.))
                .on_scroll_wheel(
                    cx.listener(move |this, event: &ScrollWheelEvent, _window, cx| {
                        let delta_y = event.delta.pixel_delta(px(1.0)).y;
                        let direction = if delta_y > px(0.) {
                            ChatScrollDirection::Up
                        } else if delta_y < px(0.) {
                            ChatScrollDirection::Down
                        } else {
                            ChatScrollDirection::None
                        };

                        let at_bottom_before = this.turns_list_is_at_bottom();
                        let previous_manual_mode = this.user_has_scrolled_up;
                        let was_up = matches!(direction, ChatScrollDirection::Up);
                        let was_down = matches!(direction, ChatScrollDirection::Down);

                        tracing::debug!(
                            target: "script_kit::chat_scroll",
                            event = "wheel",
                            phase = "before",
                            direction = ?direction,
                            delta_y = ?delta_y,
                            at_bottom_before,
                            previous_manual_mode,
                            turn_count = this.conversation_turns_cache.len(),
                            scroll_top_item_ix = this.turns_list_state.logical_scroll_top().item_ix,
                        );

                        cx.spawn(async move |this, cx| {
                            cx.background_executor()
                                .timer(std::time::Duration::from_millis(1))
                                .await;

                            this.update(cx, |this, cx| {
                                let direction = if was_up {
                                    ChatScrollDirection::Up
                                } else if was_down {
                                    ChatScrollDirection::Down
                                } else {
                                    ChatScrollDirection::None
                                };

                                let at_bottom_after = this.turns_list_is_at_bottom();
                                this.apply_scroll_follow_decision(
                                    "wheel",
                                    direction,
                                    at_bottom_before,
                                    at_bottom_after,
                                    cx,
                                );
                            })
                            .ok();
                        })
                        .detach();
                    }),
                )
                .child(turns_list)
                .vertical_scrollbar(&self.turns_list_state)
                .when(show_scroll_to_latest, |el| {
                    el.child(
                        div()
                            .id("chat-scroll-to-latest-pill")
                            .absolute()
                            .bottom(px(12.0))
                            .left_0()
                            .right_0()
                            .flex()
                            .justify_center()
                            .child(
                                div()
                                    .id("chat-scroll-to-latest-button")
                                    .px(px(10.0))
                                    .py(px(5.0))
                                    .rounded_full()
                                    .bg(rgba((theme_colors.ui.border << 8) | 0xCC))
                                    .text_color(rgb(theme_colors.text.primary))
                                    .text_xs()
                                    .cursor_pointer()
                                    .hover(|d| d.bg(rgba((theme_colors.ui.border << 8) | 0xFF)))
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.force_scroll_turns_to_bottom();
                                        cx.notify();
                                    }))
                                    .child("Jump to latest"),
                            ),
                    )
                })
                .into_any_element()
        } else {
            div()
                .id("chat-messages")
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .px(px(HEADER_PADDING_X))
                .py(px(HEADER_PADDING_Y))
                .child(self.render_conversation_starters(cx))
                .into_any_element()
        };

        div()
            .id("chat-prompt")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(container_bg, |d, bg| d.bg(bg))
            .key_context("chat_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Input area
            .child(input_area)
            // Scrollable message area
            .child(messages_content)
            // Footer with model selector and footer actions
            .child(self.render_footer(cx))
            // Note: Actions menu is now handled by parent via on_show_actions callback
            // The parent opens the standard ActionsDialog window
            .into_any_element()
    }
}

#[cfg(test)]
mod chat_footer_button_click_handler_tests {
    const CHAT_RENDER_CORE_SOURCE: &str = include_str!("render_core.rs");

    fn fn_source(name: &str) -> &'static str {
        let marker = format!("fn {}(", name);
        let start = CHAT_RENDER_CORE_SOURCE
            .find(&marker)
            .unwrap_or_else(|| panic!("missing function: {}", name));
        let tail = &CHAT_RENDER_CORE_SOURCE[start..];
        let end = tail
            .find("\n    fn ")
            .or_else(|| tail.find("\n}\n\nimpl Focusable"))
            .unwrap_or(tail.len());
        &tail[..end]
    }

    #[test]
    fn test_chat_footer_uses_shared_hint_strip_tokens() {
        let non_test_source = CHAT_RENDER_CORE_SOURCE
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or(CHAT_RENDER_CORE_SOURCE);
        let footer_body = fn_source("render_footer");
        let minimal_footer_body = fn_source("render_minimal_footer");
        let hint_text_body = fn_source("footer_hint_text");

        assert!(
            !footer_body.contains("PromptFooter::new("),
            "Chat minimal footer should not render via PromptFooter"
        );
        assert!(
            minimal_footer_body.contains("HintStrip::new"),
            "Chat minimal footer should render through the shared HintStrip component"
        );
        assert!(
            footer_body.contains("render_minimal_footer")
                && footer_body.contains("render_rich_footer"),
            "Chat footer should branch on chrome style"
        );
        assert!(
            footer_body.contains("ChromeStyle::Minimal")
                && footer_body.contains("ChromeStyle::Rich"),
            "Chat footer should use the ChromeStyle enum"
        );
        assert!(
            minimal_footer_body.contains("render_script_generation_hint_actions"),
            "Chat minimal footer should keep script generation actions in the hint strip"
        );
        assert!(
            hint_text_body.contains("Esc Stop · ⌘K Actions"),
            "Streaming chat footer should show stop and actions shortcuts"
        );
        assert!(
            hint_text_body.contains("↵ Send · ⌘K Actions · Esc Back"),
            "Default chat footer should show send, actions, and back shortcuts"
        );
        assert!(
            hint_text_body.contains("⌘↵ Save+Run"),
            "Script generation chat footer should show the save-and-run shortcut"
        );
        assert!(
            !non_test_source.contains("enum ChatFooterButtonAction"),
            "Chat footer should not rely on the removed footer button enum"
        );
    }
}
