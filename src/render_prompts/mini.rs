mod __render_prompts_mini_docs {
    //! Mini prompt rendering: compact single-line input with reduced chrome.
    //! No list display, minimal padding, optional footer.
    //! This fragment is included via include!() macro in main.rs.
}

impl ScriptListApp {
    fn render_mini_prompt(
        &mut self,
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        tracing::info!(
            prompt_type = "mini",
            prompt_id = %id,
            choice_count = choices.len(),
            "render_mini_prompt: rendering compact single-line prompt"
        );

        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_typography = render_context.design_typography;
        let design_visual = render_context.design_visual;

        let text_primary = theme.colors.text.primary;
        let text_muted = theme.colors.text.muted;
        let accent_color = theme.colors.accent.selected;
        let input_is_empty = self.arg_input.is_empty();

        // Mini uses reduced padding (half of standard header padding)
        let mini_padding_x: f32 = HEADER_PADDING_X * 0.75;
        let mini_padding_y: f32 = HEADER_PADDING_Y * 0.75;

        // Filtered choices for selection (mini shows no list, but tracks selection for submit)
        let filtered_choices = self.get_filtered_arg_choices_owned();
        let has_choices = !choices.is_empty();
        let filtered_count = filtered_choices.len();

        // Key handler - Enter submits, Escape dismisses, arrow keys navigate hidden selection
        let prompt_id = id.clone();
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;
                let key_lower = key.to_lowercase();

                // Escape dismisses
                if ui_foundation::is_key_escape(key) {
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Enter submits
                if ui_foundation::is_key_enter(key) {
                    this.submit_arg_prompt_from_current_state(&prompt_id, cx);
                    cx.stop_propagation();
                    return;
                }

                // Arrow up/down for hidden selection navigation
                if ui_foundation::is_key_up(key) && !modifiers.shift {
                    if this.arg_selected_index > 0 {
                        this.arg_selected_index -= 1;
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }

                if ui_foundation::is_key_down(key) && !modifiers.shift {
                    let filtered = this.filtered_arg_choices();
                    if this.arg_selected_index < filtered.len().saturating_sub(1) {
                        this.arg_selected_index += 1;
                        cx.notify();
                    }
                    cx.stop_propagation();
                    return;
                }

                // Tab completion
                if key.eq_ignore_ascii_case("tab") && !has_cmd && !modifiers.alt && !modifiers.shift
                {
                    this.apply_arg_tab_completion(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Cmd+W closes
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Delegate text editing to TextInputState
                let old_text = this.arg_input.text().to_string();
                let prev_original_idx = this
                    .filtered_arg_choices()
                    .get(this.arg_selected_index)
                    .map(|(orig_idx, _)| *orig_idx);

                let handled = this.arg_input.handle_key(
                    &key_lower,
                    key_char,
                    modifiers.platform,
                    modifiers.alt,
                    modifiers.shift,
                    cx,
                );

                if handled {
                    if this.arg_input.text() != old_text {
                        // Update selection tracking but don't resize (mini is fixed height)
                        let new_selected_idx = {
                            let filtered = this.filtered_arg_choices();
                            if let Some(prev_idx) = prev_original_idx {
                                filtered
                                    .iter()
                                    .position(|(orig_idx, _)| *orig_idx == prev_idx)
                                    .unwrap_or(0)
                            } else {
                                0
                            }
                        };
                        this.arg_selected_index = new_selected_idx;
                    }
                    cx.notify();
                } else {
                    cx.propagate();
                }
            },
        );

        // Status text for mini footer
        let helper_text = if has_choices && filtered_count > 0 {
            Some(crate::panel::running_status_message(&format!(
                "{filtered_count} options · Enter to select"
            )))
        } else if !input_is_empty {
            Some(crate::panel::running_status_message("Enter to submit"))
        } else {
            Some(crate::panel::running_status_message(
                "type a value and press Enter",
            ))
        };

        let footer_colors = PromptFooterColors::from_theme(theme);
        let footer_config = PromptFooterConfig::new()
            .primary_label("Submit")
            .primary_shortcut("↵")
            .show_secondary(false)
            .show_logo(false);
        let footer_config = if let Some(helper) = helper_text {
            footer_config.helper_text(helper)
        } else {
            footer_config
        };

        // Create click handler for footer primary button
        let prompt_id_for_primary = id;
        let handle_primary = cx.entity().downgrade();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("mini_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Compact input row with reduced padding
            .child(
                div()
                    .w_full()
                    .px(px(mini_padding_x))
                    .py(px(mini_padding_y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child({
                        let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);
                        div()
                            .flex_1()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(input_height))
                            .text_size(px(design_typography.font_size_lg))
                            .text_color(if input_is_empty {
                                rgb(text_muted)
                            } else {
                                rgb(text_primary)
                            })
                            .when(input_is_empty, |d: gpui::Div| {
                                let is_cursor_visible = self.focused_input
                                    == FocusedInput::ArgPrompt
                                    && self.cursor_visible;
                                d.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .child(
                                            div()
                                                .w(px(CURSOR_WIDTH))
                                                .h(px(CURSOR_HEIGHT_LG))
                                                .when(is_cursor_visible, |d: gpui::Div| {
                                                    d.bg(rgb(text_primary))
                                                }),
                                        )
                                        .child(
                                            div()
                                                .ml(px(-(CURSOR_WIDTH)))
                                                .text_color(rgb(text_muted))
                                                .child(placeholder),
                                        ),
                                )
                            })
                            .when(!input_is_empty, |d: gpui::Div| {
                                d.child(self.render_arg_input_text(text_primary, accent_color))
                            })
                    }),
            )
            // Compact footer (no actions button)
            .child(
                PromptFooter::new(footer_config, footer_colors).on_primary_click(Box::new(
                    move |_, _window, cx| {
                        if let Some(app) = handle_primary.upgrade() {
                            app.update(cx, |this, cx| {
                                this.submit_arg_prompt_from_current_state(
                                    &prompt_id_for_primary,
                                    cx,
                                );
                            });
                        }
                    },
                )),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod mini_prompt_render_tests {
    const MINI_SOURCE: &str = include_str!("mini.rs");

    #[test]
    fn mini_prompt_has_compact_padding() {
        assert!(
            MINI_SOURCE.contains("HEADER_PADDING_X * 0.75"),
            "mini prompt should use reduced horizontal padding"
        );
        assert!(
            MINI_SOURCE.contains("HEADER_PADDING_Y * 0.75"),
            "mini prompt should use reduced vertical padding"
        );
    }

    #[test]
    fn mini_prompt_has_no_list_display() {
        // mini should not contain uniform_list
        assert!(
            !MINI_SOURCE.contains("uniform_list("),
            "mini prompt should not render a choice list"
        );
    }

    #[test]
    fn mini_prompt_has_key_handlers() {
        assert!(
            MINI_SOURCE.contains("is_key_escape(key)"),
            "mini prompt should handle Escape"
        );
        assert!(
            MINI_SOURCE.contains("is_key_enter(key)"),
            "mini prompt should handle Enter"
        );
        assert!(
            MINI_SOURCE.contains("stop_propagation()"),
            "mini prompt should stop propagation on handled keys"
        );
    }

    #[test]
    fn mini_prompt_emits_structured_logs() {
        assert!(
            MINI_SOURCE.contains("tracing::info!("),
            "mini prompt should emit structured log at info level"
        );
        assert!(
            MINI_SOURCE.contains("prompt_type = \"mini\""),
            "mini prompt log should include prompt_type field"
        );
    }
}
