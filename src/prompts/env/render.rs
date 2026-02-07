use super::*;

impl Focusable for EnvPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EnvPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let design_colors = tokens.colors();
        let design_typography = tokens.typography();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;

                // Handle submit/cancel first
                match env_key_action(key) {
                    Some(EnvKeyAction::Submit) => {
                        this.submit();
                        return;
                    }
                    Some(EnvKeyAction::Cancel) => {
                        this.submit_cancel();
                        return;
                    }
                    None => {}
                }

                // Delegate all other keys to TextInputState
                let key_char = event.keystroke.key_char.as_deref();
                let previous_text = this.input.text().to_string();
                let handled = this.input.handle_key(
                    key,
                    key_char,
                    modifiers.platform, // On macOS, platform = Cmd key
                    modifiers.alt,
                    modifiers.shift,
                    cx,
                );

                if handled {
                    if this.validation_error.is_some() && previous_text != this.input.text() {
                        this.validation_error = None;
                    }
                    cx.notify();
                }
            },
        );

        // Use design tokens for consistent styling
        let text_primary = design_colors.text_primary;
        let text_muted = design_colors.text_muted;
        let accent_color = design_colors.accent;
        let bg_surface = design_colors.background_secondary;
        let success_color = design_colors.success;
        let error_color = design_colors.error;

        // Build placeholder text for input
        let input_placeholder: SharedString =
            env_input_placeholder(&self.key, self.exists_in_keyring).into();

        // Build description text
        let description: SharedString = self
            .prompt
            .clone()
            .unwrap_or_else(|| env_default_description(&self.key, self.exists_in_keyring))
            .into();

        let input_is_empty = self.input.is_empty();
        let running_status = env_running_status(&self.key);

        // Full-window centered layout for API key input
        div()
            .id(gpui::ElementId::Name("window:env".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("env_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Main content area - centered vertically
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .px(px(32.))
                    .gap(px(24.))
                    // Large key icon at top
                    .child(
                        div()
                            .size(px(64.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(16.))
                            .bg(rgba(accent_color << 8 | 0x20)) // Accent with low alpha
                            .child(
                                svg()
                                    .external_path(if self.secret {
                                        IconName::EyeOff.external_path()
                                    } else {
                                        IconName::Settings.external_path()
                                    })
                                    .size(px(32.))
                                    .text_color(rgb(accent_color)),
                            ),
                    )
                    // Title - provider name or key name
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(text_primary))
                                    .child(self.title.clone().unwrap_or_else(|| self.key.clone())),
                            )
                            // Description
                            .child(
                                div()
                                    .text_base()
                                    .text_color(rgb(text_muted))
                                    .text_center()
                                    .child(description),
                            ),
                    )
                    // Input field with clearer label and focus treatment
                    .child(
                        div()
                            .w_full()
                            .max_w(px(400.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(text_muted))
                                    .child(env_input_label(self.secret)),
                            )
                            .child(
                                div()
                                    .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
                                    .px(px(16.))
                                    .py(px(12.))
                                    .rounded(px(12.))
                                    .bg(rgb(bg_surface))
                                    .border_1()
                                    .border_color(rgba(accent_color << 8 | 0x80))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(12.))
                                    // Lock icon inside input
                                    .child(
                                        svg()
                                            .external_path(if self.secret {
                                                IconName::EyeOff.external_path()
                                            } else {
                                                IconName::Settings.external_path()
                                            })
                                            .size(px(18.))
                                            .text_color(rgb(text_muted))
                                            .flex_shrink_0(),
                                    )
                                    // Input text area
                                    .child({
                                        div()
                                            .flex_1()
                                            .overflow_hidden()
                                            .text_lg()
                                            .text_color(if input_is_empty {
                                                rgb(text_muted)
                                            } else {
                                                rgb(text_primary)
                                            })
                                            // When empty: show cursor + placeholder
                                            .when(input_is_empty, |d: Div| {
                                                d.child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .child(
                                                            div()
                                                                .w(px(CURSOR_WIDTH))
                                                                .h(px(CURSOR_HEIGHT_LG))
                                                                .bg(rgb(accent_color)),
                                                        )
                                                        .child(
                                                            div()
                                                                .ml(px(4.))
                                                                .text_color(rgb(text_muted))
                                                                .child(input_placeholder.clone()),
                                                        ),
                                                )
                                            })
                                            // When has text: show masked dots or text with cursor
                                            .when(!input_is_empty, |d: Div| {
                                                d.child(
                                                    self.render_input_text(
                                                        text_primary,
                                                        accent_color,
                                                    ),
                                                )
                                            })
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child(env_storage_hint_text(self.secret)),
                            ),
                    )
                    .when_some(self.validation_error.clone(), |d: Div, error| {
                        d.child(
                            div()
                                .max_w(px(400.))
                                .w_full()
                                .text_xs()
                                .text_color(rgb(error_color))
                                .child(error),
                        )
                    })
                    // Running state indicator to clarify why this prompt is visible
                    .child(
                        div()
                            .max_w(px(400.))
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.))
                            .child(div().size(px(8.)).rounded_full().bg(rgb(accent_color)))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_muted))
                                    .child(running_status),
                            ),
                    )
                    // Status hint - show when key exists with modification date and delete option
                    .when(self.exists_in_keyring, |d: Div| {
                        let modified_text = self
                            .modified_at
                            .map(format_relative_time)
                            .unwrap_or_else(|| "previously".to_string());

                        // Create delete click handler
                        let handle_delete = cx.entity().downgrade();

                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.))
                                // Status line with checkmark and modification time
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(
                                            svg()
                                                .external_path(IconName::Check.external_path())
                                                .size(px(14.))
                                                .text_color(rgb(success_color)),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(text_muted))
                                                .child(format!("Configured {}", modified_text)),
                                        )
                                        // Separator dot
                                        .child(
                                            div().text_sm().text_color(rgb(text_muted)).child("·"),
                                        )
                                        // Delete link
                                        .child(
                                            div()
                                                .id("delete-secret")
                                                .text_sm()
                                                .text_color(rgb(error_color))
                                                .cursor_pointer()
                                                .hover(|s| s.opacity(0.8))
                                                .on_click(move |_event, _window, cx| {
                                                    if let Some(entity) = handle_delete.upgrade() {
                                                        entity.update(cx, |this, _cx| {
                                                            this.submit_delete();
                                                        });
                                                    }
                                                })
                                                .child("Delete"),
                                        ),
                                ),
                        )
                    }),
            )
            // Footer with submit action
            .child({
                let footer_colors = PromptFooterColors::from_theme(&self.theme);
                let primary_label = if self.exists_in_keyring {
                    "Update & Continue"
                } else {
                    "Save & Continue"
                };
                let footer_config = PromptFooterConfig::new()
                    .primary_label(primary_label)
                    .primary_shortcut("↵")
                    .helper_text("Script running")
                    .show_secondary(true)
                    .secondary_label("Cancel")
                    .secondary_shortcut("Esc");

                // Add click handlers
                let handle = cx.entity().downgrade();
                let handle_cancel = cx.entity().downgrade();
                PromptFooter::new(footer_config, footer_colors)
                    .on_primary_click(Box::new(move |_, _window, cx| {
                        if let Some(entity) = handle.upgrade() {
                            entity.update(cx, |this, _cx| {
                                this.submit();
                            });
                        }
                    }))
                    .on_secondary_click(Box::new(move |_, _window, cx| {
                        if let Some(entity) = handle_cancel.upgrade() {
                            entity.update(cx, |this, _cx| {
                                this.submit_cancel();
                            });
                        }
                    }))
            })
    }
}
