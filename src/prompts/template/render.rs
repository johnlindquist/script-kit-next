use super::*;

impl Focusable for TemplatePrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TemplatePrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();

                match key_str.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            this.prev_input(cx);
                        } else {
                            this.next_input(cx);
                        }
                    }
                    "enter" | "return" => this.submit(cx),
                    "escape" | "esc" => this.submit_cancel(),
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        let (_main_bg, text_color, muted_color, border_color) =
            if self.design_variant == DesignVariant::Default {
                (
                    rgb(self.theme.colors.background.main),
                    rgb(self.theme.colors.text.secondary),
                    rgb(self.theme.colors.text.muted),
                    rgb(self.theme.colors.ui.border),
                )
            } else {
                (
                    rgb(colors.background),
                    rgb(colors.text_secondary),
                    rgb(colors.text_muted),
                    rgb(colors.border),
                )
            };
        let error_color = rgb(self.theme.colors.accent.selected);

        let preview = self.preview_template();

        let mut container = div()
            .id(gpui::ElementId::Name("window:template".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // Only apply bg when vibrancy disabled
            .text_color(text_color)
            .p(px(spacing.padding_lg))
            .key_context("template_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key);

        // Preview section with live template
        container = container
            .child(div().text_sm().text_color(muted_color).child("Preview:"))
            .child(
                div()
                    .mt(px(spacing.padding_sm))
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.padding_md))
                    .bg(rgb(self.theme.colors.background.search_box))
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(4.))
                    .text_base()
                    .child(preview),
            );

        // Input fields section
        if self.inputs.is_empty() {
            container = container.child(
                div()
                    .mt(px(spacing.padding_lg))
                    .text_color(muted_color)
                    .child("No {{placeholders}} found in template"),
            );
        } else {
            container = container.child(
                div()
                    .mt(px(spacing.padding_lg))
                    .text_sm()
                    .text_color(muted_color)
                    .child(format!(
                        "Fill {} field(s). Required fields are marked with *.",
                        self.inputs.len()
                    )),
            );

            let mut previous_group: Option<String> = None;
            for (idx, input) in self.inputs.iter().enumerate() {
                if previous_group.as_deref() != Some(input.group.as_str()) {
                    previous_group = Some(input.group.clone());
                    container = container.child(
                        div()
                            .mt(px(spacing.padding_md))
                            .text_xs()
                            .text_color(muted_color)
                            .child(input.group.clone()),
                    );
                }

                let is_current = idx == self.current_input;
                let value = self.values.get(idx).cloned().unwrap_or_default();

                let display = if value.is_empty() {
                    SharedString::from(input.placeholder.clone())
                } else {
                    SharedString::from(value.clone())
                };

                // Use low-opacity for vibrancy support (see VIBRANCY.md)
                let field_bg = if is_current {
                    rgba((self.theme.colors.accent.selected_subtle << 8) | 0x0f)
                // ~6% opacity
                } else {
                    rgb(self.theme.colors.background.search_box)
                };

                let field_border = if is_current {
                    rgb(self.theme.colors.accent.selected)
                } else {
                    border_color
                };

                let text_display_color = if value.is_empty() {
                    muted_color
                } else {
                    text_color
                };

                let label = if input.required {
                    format!("{} *", input.label)
                } else {
                    input.label.clone()
                };

                let mut row = div()
                    .mt(px(spacing.padding_sm))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(140.))
                                    .text_sm()
                                    .text_color(muted_color)
                                    .child(label),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
                                    .px(px(spacing.item_padding_x))
                                    .py(px(spacing.padding_sm))
                                    .bg(field_bg)
                                    .border_1()
                                    .border_color(field_border)
                                    .rounded(px(4.))
                                    .text_color(text_display_color)
                                    .child(display),
                            ),
                    );

                if let Some(Some(error_message)) = self.validation_errors.get(idx) {
                    row = row.child(
                        div()
                            .pl(px(144.))
                            .text_xs()
                            .text_color(error_color)
                            .child(error_message.clone()),
                    );
                }

                container = container.child(row);
            }
        }

        let has_name_fields = self
            .inputs
            .iter()
            .any(|input| Self::is_name_field(&input.name));
        if has_name_fields {
            container = container.child(
                div()
                    .mt(px(spacing.padding_md))
                    .text_xs()
                    .text_color(muted_color)
                    .child("Naming tip: use lowercase letters, numbers, and hyphens."),
            );
        }

        // Help text at bottom
        container = container.child(
            div()
                .mt(px(spacing.padding_lg))
                .text_xs()
                .text_color(muted_color)
                .child("Tab: next field | Shift+Tab: previous | Enter: submit | Escape: cancel"),
        );

        container
    }
}
