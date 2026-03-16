use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::ui_foundation::{is_key_backspace, is_key_enter, is_key_tab, printable_char};
use gpui::FontWeight;

impl Focusable for TemplatePrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TemplatePrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let spacing = tokens.spacing();

        let text_primary = rgb(self.theme.colors.text.primary);
        let text_secondary = rgb(self.theme.colors.text.secondary);
        let text_muted = rgb(self.theme.colors.text.muted);
        let border_color = rgb(self.theme.colors.ui.border);
        let error_color = rgb(self.theme.colors.ui.error);
        let input_bg = rgb(self.theme.colors.background.search_box);
        let current_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x20);
        let current_border = rgb(self.theme.colors.accent.selected);

        let description = if self.inputs.is_empty() {
            "This template has no editable placeholders. Review the preview and press Enter to submit."
                .to_string()
        } else {
            format!(
                "Fill {} field(s). The preview updates as you type.",
                self.inputs.len()
            )
        };

        let preview = self.preview_template();

        let mut content = div()
            .id(gpui::ElementId::Name("window:template".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(text_primary)
            .gap(px(spacing.gap_lg))
            .child(crate::components::prompt_form_intro(
                "Complete the template",
                description,
                text_primary,
                text_muted,
                spacing.gap_sm,
            ))
            .child(crate::components::prompt_form_section(
                "Preview",
                text_secondary,
                spacing.gap_sm,
                crate::components::prompt_surface(current_bg, border_color).child(
                    div()
                        .w_full()
                        .text_sm()
                        .text_color(text_primary)
                        .child(preview),
                ),
            ));

        if self.inputs.is_empty() {
            content = content.child(crate::components::prompt_form_help(
                "No {{placeholders}} found in template.",
                text_secondary,
            ));
        } else {
            let mut fields = div().w_full().flex().flex_col().gap(px(spacing.gap_lg));
            let mut previous_group: Option<String> = None;

            for (idx, input) in self.inputs.iter().enumerate() {
                if !input.group.is_empty()
                    && previous_group.as_deref() != Some(input.group.as_str())
                {
                    previous_group = Some(input.group.clone());
                    fields = fields.child(
                        div()
                            .w_full()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_muted)
                            .child(input.group.clone()),
                    );
                }

                let is_current = idx == self.current_input;
                let value = self.values.get(idx).cloned().unwrap_or_default();
                let label = if input.required {
                    format!("{} *", input.label)
                } else {
                    input.label.clone()
                };
                let display = if value.is_empty() {
                    SharedString::from(input.placeholder.clone())
                } else {
                    SharedString::from(value.clone())
                };
                let display_color = if value.is_empty() {
                    text_muted
                } else {
                    text_primary
                };
                let field_bg = if is_current { current_bg } else { input_bg };
                let field_border = if is_current {
                    current_border
                } else {
                    border_color
                };
                let validation_message =
                    self.validation_errors.get(idx).and_then(|m| m.as_ref());

                let field_section = crate::components::prompt_form_section(
                    label,
                    text_secondary,
                    spacing.gap_sm,
                    crate::components::prompt_surface(field_bg, field_border)
                        .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
                        .text_color(display_color)
                        .child(display),
                )
                .when_some(validation_message, |d, message| {
                    d.child(crate::components::prompt_form_help(
                        message.clone(),
                        error_color,
                    ))
                });

                fields = fields.child(field_section);
            }

            if self.inputs.iter().any(|input| Self::is_name_field(&input.name)) {
                fields = fields.child(crate::components::prompt_form_help(
                    "Naming tip: use lowercase letters, numbers, and hyphens.",
                    text_muted,
                ));
            }

            content = content.child(fields);
        }

        FocusablePrompt::new(content)
            .key_context("template_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |this, intercepted_key, _event, _window, _cx| match intercepted_key {
                    FocusablePromptInterceptedKey::Escape => {
                        this.submit_cancel();
                        true
                    }
                    _ => false,
                },
                |this, event, _window, cx| {
                    let key_str = event.keystroke.key.as_str();

                    if is_key_tab(key_str) {
                        if event.keystroke.modifiers.shift {
                            this.prev_input(cx);
                        } else {
                            this.next_input(cx);
                        }
                    } else if is_key_enter(key_str) {
                        this.submit(cx);
                    } else if is_key_backspace(key_str) {
                        this.handle_backspace(cx);
                    } else if let Some(ch) = printable_char(event.keystroke.key_char.as_deref()) {
                        this.handle_char(ch, cx);
                    }
                },
            )
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("render.rs");

    #[test]
    fn template_render_uses_shared_create_flow_helpers() {
        assert!(
            SOURCE.contains("prompt_form_intro("),
            "render.rs should use prompt_form_intro"
        );
        assert!(
            SOURCE.contains("prompt_form_section("),
            "render.rs should use prompt_form_section"
        );
        assert!(
            SOURCE.contains("prompt_form_help("),
            "render.rs should use prompt_form_help"
        );
        assert!(
            SOURCE.contains("prompt_surface("),
            "render.rs should use prompt_surface"
        );
    }

    #[test]
    fn template_render_no_longer_renders_inline_shortcut_footer_text() {
        let production_code = SOURCE
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(SOURCE);

        assert!(
            !production_code.contains(
                "Tab: next field | Shift+Tab: previous | Enter: submit | Escape: cancel"
            ),
            "render.rs production code should not contain inline shortcut footer text"
        );
    }
}
