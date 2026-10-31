use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::theme::AppChromeColors;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    Sizable as _,
};

impl Focusable for EnvPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EnvPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = AppChromeColors::from_theme(&self.theme);
        let text = crate::components::prompt_text_palette(&self.theme);

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

        let metrics = crate::components::FormFieldMetrics::from_colors(
            crate::components::FormFieldColors::from_theme(&self.theme),
        );

        // Input body: cursor + placeholder when empty, masked/text with cursor when filled
        let input_body = if input_is_empty {
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .child(crate::components::text_input::pulse_cursor_bar(
                    div()
                        .w(px(CURSOR_WIDTH))
                        .h(px(CURSOR_HEIGHT_LG))
                        .bg(rgb(accent_color)),
                    "env-input-cursor-pulse",
                ))
                .child(div().text_color(text.placeholder).child(SharedString::from(
                    env_input_placeholder(&self.key, self.exists_in_keyring),
                )))
        } else {
            div()
                .w_full()
                .text_color(text.primary)
                .child(self.render_input_text(chrome.text_primary_hex, accent_color))
        };

        let input_body = div()
            .w_full()
            .flex()
            .items_center()
            .gap(px(metrics.field_gap_px))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .overflow_x_hidden()
                    .font_family(crate::list_item::FONT_MONO)
                    .child(input_body),
            )
            .when(self.secret && !input_is_empty, |row| {
                let label = self.visibility_label();
                row.child(
                    Button::new("env:visibility")
                        .accessibility_id(self.visibility_id())
                        .accessibility_label(label)
                        .icon(if self.reveal_secret {
                            gpui_component::IconName::EyeOff
                        } else {
                            gpui_component::IconName::Eye
                        })
                        .label(if self.reveal_secret { "Hide" } else { "Show" })
                        .xsmall()
                        .text()
                        .tab_stop(false)
                        .tooltip(format!("{label} · ⌃⇧R"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.focus_handle.focus(window, cx);
                            this.toggle_secret_reveal(cx);
                        })),
                )
            });

        // Stacked minimal body — no hero icon, no centered card
        let mut body = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(metrics.field_stack_gap_px))
            // Title + description intro
            .child(crate::components::prompt_form_intro(
                title,
                description,
                text.primary,
                text.help,
                6.0,
            ))
            .child(crate::components::labeled_control(
                &self.theme,
                "env-value",
                env_input_label(self.secret),
                self.focus_handle.is_focused(window),
                self.validation_error.clone().map(Into::into),
                input_body.into_any_element(),
            ))
            // Storage hint
            .child(crate::components::prompt_form_help(
                env_storage_hint_text(self.secret, self.secret_store_error.as_ref()),
                text.help,
            ));

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
                        .child(SharedString::from(format!("Configured {}", modified_text))),
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

        if let Some(error) = &self.secret_store_error {
            body = body.child(div().text_xs().text_color(rgb(error_color)).child(
                SharedString::from(format!("Storage status: {}", error.kind_str())),
            ));
        }

        let container = div()
            .id(gpui::ElementId::Name("window:env".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(text.primary)
            .child(
                crate::components::render_inset_prompt_body(
                    "env-body-scroll",
                    body,
                    crate::components::PromptBodyInsets::MainMenu(self.design_variant),
                )
                .flex_1()
                .overflow_y_scroll(),
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

                    if modifiers.control
                        && modifiers.shift
                        && !modifiers.platform
                        && !modifiers.alt
                        && key.eq_ignore_ascii_case("r")
                        && this.can_toggle_visibility()
                    {
                        this.toggle_secret_reveal(cx);
                        cx.stop_propagation();
                        return;
                    }
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
                        if this.input.text().is_empty() {
                            this.reveal_secret = false;
                        }
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
        assert!(
            !render_code.contains("text_muted_hex"),
            "env render should not keep a local muted text ladder"
        );
    }
}
