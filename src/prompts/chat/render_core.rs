use super::*;

impl ChatPrompt {
    fn render_footer_button(
        &self,
        id: &'static str,
        label: &'static str,
        shortcut: Option<&'static str>,
        accent_color: u32,
        muted_color: u32,
    ) -> AnyElement {
        div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(2.0))
            .rounded(px(4.0))
            .cursor_default()
            .child(div().text_xs().text_color(rgb(accent_color)).child(label))
            .when_some(shortcut, |d, shortcut| {
                d.child(div().text_xs().text_color(rgb(muted_color)).child(shortcut))
            })
            .into_any_element()
    }

    fn render_script_generation_footer_button(
        &self,
        id: &'static str,
        label: &'static str,
        shortcut: Option<&'static str>,
        action: ScriptGenerationAction,
        footer_colors: PromptFooterColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let button_font_size = (self.theme.get_fonts().ui_size - 2.0).max(10.0);
        let hover_bg = rgba((footer_colors.background << 8) | 0x26);
        let active_bg = rgba((footer_colors.background << 8) | 0x3a);

        div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(2.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(move |d| d.bg(hover_bg))
            .active(move |d| d.bg(active_bg))
            .child(
                div()
                    .text_size(px(button_font_size))
                    .text_color(rgb(footer_colors.accent))
                    .child(label),
            )
            .when_some(shortcut, |d, shortcut| {
                d.child(
                    div()
                        .text_size(px(button_font_size))
                        .text_color(rgb(footer_colors.text_muted))
                        .child(shortcut),
                )
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.handle_script_generation_action(action, cx);
            }))
            .into_any_element()
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let footer_colors = PromptFooterColors::from_theme(&self.theme);
        let footer_bg = theme::hover_overlay_bg(
            &self.theme,
            if self.theme.is_dark_mode() {
                CHAT_LAYOUT_FOOTER_BG_DARK_ALPHA
            } else {
                CHAT_LAYOUT_FOOTER_BG_LIGHT_ALPHA
            },
        );
        let model_text = self.model.clone().unwrap_or_else(|| "Select Model".into());
        let footer_height = crate::window_resize::layout::FOOTER_HEIGHT;
        let divider = || {
            div()
                .w(px(1.0))
                .h(px(16.0))
                .mx(px(4.0))
                .bg(rgba((footer_colors.border << 8) | 0x40))
                .into_any_element()
        };

        let mut left_side = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .flex_1()
            .min_w(px(0.0))
            .overflow_hidden()
            .child(
                svg()
                    .external_path(crate::utils::get_logo_path())
                    .size(px(16.0))
                    .ml(px(2.0))
                    .text_color(rgb(footer_colors.accent)),
            )
            .child(
                div()
                    .max_w(px(420.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_xs()
                    .text_color(rgb(footer_colors.accent))
                    .child(model_text),
            );

        if self.script_generation_mode {
            if let Some(script_actions) = self.render_script_generation_footer_actions(cx) {
                left_side = left_side.child(divider()).child(script_actions);
            }
        }

        let mut right_side = div().flex().flex_row().items_center().min_w(px(0.));
        let primary_button = if self.script_generation_mode {
            self.render_script_generation_footer_button(
                "chat-footer-primary-button",
                "Save and Run",
                Some("⌘↵"),
                ScriptGenerationAction::SaveAndRun,
                footer_colors,
                cx,
            )
        } else {
            self.render_footer_button(
                "chat-footer-primary-button",
                "Continue in Chat",
                Some("⌘↵"),
                footer_colors.accent,
                footer_colors.text_muted,
            )
        };

        right_side = right_side
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(footer_colors.text_muted))
                    .child("Shift+Enter newline"),
            )
            .child(divider())
            .child(primary_button)
            .child(divider())
            .child(self.render_footer_button(
                "chat-footer-secondary-button",
                "Actions",
                Some("⌘K"),
                footer_colors.accent,
                footer_colors.text_muted,
            ));

        div()
            .id("chat-footer")
            .w_full()
            .h(px(footer_height))
            .min_h(px(footer_height))
            .max_h(px(footer_height))
            .flex_shrink_0()
            .overflow_hidden()
            .px(px(CHAT_LAYOUT_PADDING_X))
            .pt(px(0.0))
            .pb(px(2.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(rgba((footer_colors.border << 8) | 0x50))
            .bg(footer_bg)
            .child(left_side)
            .child(right_side)
    }

    fn render_script_generation_footer_actions(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let show_actions = self.should_show_script_generation_actions();
        let status_message = self.script_generation_status.clone();

        if !show_actions && status_message.is_none() {
            return None;
        }

        let theme_colors = &self.theme.colors;
        let footer_colors = PromptFooterColors::from_theme(&self.theme);

        let mut action_container = div()
            .id("chat-script-generation-footer-actions")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.0))
            .min_w(px(0.0));

        if show_actions {
            action_container = action_container
                .child(self.render_script_generation_footer_button(
                    "chat-script-generation-save",
                    "Save",
                    None,
                    ScriptGenerationAction::Save,
                    footer_colors,
                    cx,
                ))
                .child(self.render_script_generation_footer_button(
                    "chat-script-generation-run",
                    "Run",
                    None,
                    ScriptGenerationAction::Run,
                    footer_colors,
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
        let colors = &self.prompt_colors;

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
                // Header with back button and title
                .child(self.render_header())
                // Setup card content
                .child(self.render_setup_card(cx))
                .into_any_element();
        }

        // If loading_providers, show a "Connecting to AI..." placeholder
        if self.loading_providers {
            let colors = &self.prompt_colors;
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
                .child(self.render_header())
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
                                .text_color(rgb(colors.text_secondary))
                                .child("Connecting to AI..."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(colors.text_tertiary))
                                .child("Loading providers and models"),
                        ),
                )
                .into_any_element();
        }

        // Input area at TOP
        let has_pending_image = self.pending_image.is_some();
        let input_area = div()
            .w_full()
            .px(px(CHAT_LAYOUT_PADDING_X))
            .py(px(CHAT_LAYOUT_SECTION_PADDING_Y))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .border_b_1()
            .border_color(rgba((colors.quote_border << 8) | CHAT_LAYOUT_BORDER_ALPHA))
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
            .px(px(CHAT_LAYOUT_PADDING_X))
            .py(px(CHAT_LAYOUT_MESSAGES_PADDING_Y));

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

                        let is_at_bottom = this.turns_list_is_at_bottom();
                        let next_state = next_chat_scroll_follow_state(
                            this.user_has_scrolled_up,
                            direction,
                            is_at_bottom,
                        );

                        if next_state != this.user_has_scrolled_up {
                            logging::log(
                                "CHAT",
                                &format!(
                                    "Scroll follow changed: manual_mode={} direction={:?} at_bottom={}",
                                    next_state, direction, is_at_bottom
                                ),
                            );
                            this.user_has_scrolled_up = next_state;
                            cx.notify();
                        }
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
                                    .bg(rgba((colors.quote_border << 8) | 0xCC))
                                    .text_color(rgb(colors.text_primary))
                                    .text_xs()
                                    .cursor_pointer()
                                    .hover(|d| d.bg(rgba((colors.quote_border << 8) | 0xFF)))
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
                .px(px(CHAT_LAYOUT_PADDING_X))
                .py(px(CHAT_LAYOUT_MESSAGES_PADDING_Y))
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
            // Header with back button and title
            .child(self.render_header())
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
