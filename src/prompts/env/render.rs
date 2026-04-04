use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::theme::AppChromeColors;

impl Focusable for EnvPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EnvPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = AppChromeColors::from_theme(&self.theme);

        let text_primary = chrome.text_primary_hex;
        let text_muted = chrome.text_muted_hex;
        let accent_color = chrome.accent_hex;

        // Error/success from theme UI colors
        let success_color = self.theme.colors.ui.success;
        let error_color = self.theme.colors.ui.error;

        let input_is_empty = self.input.is_empty();

        tracing::info!(
            surface = "prompts::env",
            input_is_empty,
            exists_in_keyring = self.exists_in_keyring,
            "prompt_surface_rendered"
        );

        let title: SharedString = self
            .title
            .clone()
            .unwrap_or_else(|| self.key.clone())
            .into();
        let description: SharedString = self
            .prompt
            .clone()
            .unwrap_or_else(|| env_default_description(&self.key, self.exists_in_keyring))
            .into();

        // Whisper-chrome field surface
        let field_bg = rgba((text_muted << 8) | 0x1A);
        let field_border = rgba((text_muted << 8) | 0x33);

        // Input body: cursor + placeholder when empty, masked/text with cursor when filled
        let input_body = if input_is_empty {
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .child(
                    div()
                        .w(px(CURSOR_WIDTH))
                        .h(px(CURSOR_HEIGHT_LG))
                        .bg(rgb(accent_color)),
                )
                .child(
                    div()
                        .text_color(rgb(text_muted))
                        .child(SharedString::from(env_input_placeholder(
                            &self.key,
                            self.exists_in_keyring,
                        ))),
                )
        } else {
            div()
                .w_full()
                .text_color(rgb(text_primary))
                .child(self.render_input_text(text_primary, accent_color))
        };

        // Stacked minimal body — no hero icon, no centered card
        let mut body = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(16.0))
            // Title + description intro
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_primary))
                            .child(title),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_muted))
                            .child(description),
                    ),
            )
            // Labeled input section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(text_muted))
                            .child(env_input_label(self.secret)),
                    )
                    .child(
                        div()
                            .w_full()
                            .px(px(12.0))
                            .py(px(8.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(field_border)
                            .bg(field_bg)
                            .min_h(px(38.0))
                            .child(input_body),
                    ),
            )
            // Storage hint
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(text_muted))
                    .child(env_storage_hint_text(self.secret)),
            )
            // Running status
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(text_muted))
                    .child(env_running_status(&self.key)),
            );

        // Validation error
        if let Some(error) = self.validation_error.clone() {
            body = body.child(div().text_xs().text_color(rgb(error_color)).child(error));
        }

        // Existing key status + delete action
        if self.exists_in_keyring {
            let modified_text = self
                .modified_at
                .map(format_relative_time)
                .unwrap_or_else(|| "previously".to_string());
            let handle_delete = cx.entity().downgrade();

            body = body
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(success_color))
                        .child(SharedString::from(format!(
                            "Configured {}",
                            modified_text
                        ))),
                )
                .child(
                    div()
                        .id("delete-secret")
                        .text_xs()
                        .text_color(rgb(error_color))
                        .cursor_pointer()
                        .hover(|style| style.opacity(0.8))
                        .on_click(move |_event, _window, cx| {
                            if let Some(entity) = handle_delete.upgrade() {
                                entity.update(cx, |this, cx| {
                                    this.submit_delete(cx);
                                });
                            }
                        })
                        .child("Delete stored value"),
                );
        }

        let container = div()
            .id(gpui::ElementId::Name("window:env".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(rgb(text_primary))
            .font_family(crate::list_item::FONT_MONO)
            .child(
                div()
                    .id("env-body-scroll")
                    .flex_1()
                    .w_full()
                    .overflow_y_hidden()
                    .px(px(32.0))
                    .py(px(24.0))
                    .child(body),
            );
        // Footer is owned by the outer wrapper shell (render_prompts::other.rs)
        // which provides the canonical three-key hint strip.

        FocusablePrompt::new(container)
            .key_context("env_prompt")
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
                    let key = event.keystroke.key.as_str();
                    let modifiers = &event.keystroke.modifiers;

                    if matches!(env_key_action(key), Some(EnvKeyAction::Submit)) {
                        this.submit(cx);
                        return;
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
            )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn env_render_has_no_prompt_footer_or_hardcoded_hex() {
        const SOURCE: &str = include_str!("render.rs");
        let render_fn_end = SOURCE.find("#[cfg(test)]").unwrap_or(SOURCE.len());
        let render_code = &SOURCE[..render_fn_end];
        assert!(
            !render_code.contains("PromptFooter::new("),
            "env render should not use PromptFooter"
        );
        assert!(
            !render_code.contains("rgb(0x"),
            "env render should not use hardcoded rgb(0x...) hex colors"
        );
        assert!(
            !render_code.contains("rgba(0x"),
            "env render should not use hardcoded rgba(0x...) hex colors"
        );
    }

    #[test]
    fn env_render_uses_stacked_minimal_body_not_hero_chrome() {
        const SOURCE: &str = include_str!("render.rs");
        let render_fn_end = SOURCE.find("#[cfg(test)]").unwrap_or(SOURCE.len());
        let render_code = &SOURCE[..render_fn_end];
        assert!(
            !render_code.contains("InlinePromptInput::new("),
            "env render should not use InlinePromptInput"
        );
        assert!(
            !render_code.contains(".size(px(64.))"),
            "env render should not have a 64px hero icon"
        );
        assert!(
            !render_code.contains(".justify_center()"),
            "env render should not center content vertically"
        );
        assert!(
            render_code.contains("prompt_surface_rendered"),
            "env render should emit prompt_surface_rendered tracing"
        );
    }
}
