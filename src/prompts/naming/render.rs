use super::*;

impl Focusable for NamingPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NamingPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let spacing = tokens.spacing();

        let vibrancy_bg = get_vibrancy_background(&self.theme);
        let text_primary = rgb(self.theme.colors.text.primary);
        let text_secondary = rgb(self.theme.colors.text.secondary);
        let text_muted = rgb(self.theme.colors.text.muted);
        let border_color = rgb(self.theme.colors.ui.border);
        let accent_color = rgb(self.theme.colors.accent.selected);
        let error_color = rgb(self.theme.colors.ui.error);
        let input_bg = rgb(self.theme.colors.background.search_box);
        let preview_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x20);

        let title = format!("Name your {}", self.target.display_name().to_lowercase());
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| "Friendly name".to_string());
        let hint = self
            .hint
            .clone()
            .unwrap_or_else(|| "Enter: submit • Esc: cancel".to_string());
        let validation_message = self
            .validation_error
            .as_ref()
            .map(NamingValidationError::message);
        let input_value = if self.friendly_name.is_empty() {
            SharedString::from(placeholder)
        } else {
            SharedString::from(self.friendly_name.clone())
        };
        let input_text_color = if self.friendly_name.is_empty() {
            text_muted
        } else {
            text_primary
        };
        let input_border_color = if self.validation_error.is_some() {
            error_color
        } else {
            border_color
        };
        let preview_filename = self.filename_preview();
        let preview_path = self.target_directory.join(&preview_filename);

        let container = div()
            .id(gpui::ElementId::Name("window:naming".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg))
            .text_color(text_primary)
            .p(px(spacing.padding_lg))
            .gap(px(spacing.gap_md))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(spacing.gap_sm))
                    .child(div().text_lg().child(title))
                    .child(
                        div().text_sm().text_color(text_muted).child(
                            "Friendly name is used for display and converted to a filename.",
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(spacing.gap_sm))
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child("Friendly Name"),
                    )
                    .child(
                        div()
                            .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
                            .px(px(spacing.item_padding_x))
                            .py(px(spacing.padding_md))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border_color)
                            .rounded(px(8.))
                            .text_color(input_text_color)
                            .child(input_value),
                    )
                    .when_some(validation_message, |d, message| {
                        d.child(div().text_xs().text_color(error_color).child(message))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(spacing.gap_sm))
                    .mt(px(spacing.margin_sm))
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child("Filename Preview"),
                    )
                    .child(
                        div()
                            .px(px(spacing.item_padding_x))
                            .py(px(spacing.padding_md))
                            .bg(preview_bg)
                            .border_1()
                            .border_color(border_color)
                            .rounded(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(spacing.gap_sm))
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(text_secondary)
                                            .child(preview_filename.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_muted)
                                            .child(preview_path.display().to_string()),
                                    )
                                    .child(div().text_xs().text_color(text_muted).child(format!(
                                        "Target: {} • Extension: {}",
                                        self.target.as_str(),
                                        self.extension_label()
                                    ))),
                            ),
                    ),
            )
            .child(
                div()
                    .mt_auto()
                    .text_xs()
                    .text_color(accent_color)
                    .child(hint),
            );

        FocusablePrompt::new(container)
            .key_context("naming_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |this, intercepted_key, _event, _window, _cx| match intercepted_key {
                    FocusablePromptInterceptedKey::Escape => {
                        this.submit_cancel();
                        true
                    }
                    FocusablePromptInterceptedKey::CmdW | FocusablePromptInterceptedKey::CmdK => {
                        false
                    }
                },
                |this, event, _window, cx| {
                    let key = event.keystroke.key.as_str();

                    if is_key_enter(key) {
                        this.submit(cx);
                        return;
                    }

                    if is_key_backspace(key) {
                        this.handle_backspace(cx);
                        return;
                    }

                    if let Some(ch) = printable_char(event.keystroke.key_char.as_deref()) {
                        this.handle_char(ch, cx);
                    }
                },
            )
    }
}
