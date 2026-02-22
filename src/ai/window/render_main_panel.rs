use super::*;

fn ai_main_panel_can_submit(input_value: &str, has_pending_image: bool) -> bool {
    !input_value.is_empty() || has_pending_image
}

impl AiApp {
    pub(super) fn render_main_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Build input area at bottom:
        // Row 1: single composer surface containing [+] and input text field
        // Row 2: model picker + word count on left, submit/actions on right

        // Check if we have a pending image to show
        let has_pending_image = self.pending_image.is_some();
        let is_editing = self.editing_message_id.is_some();
        let input_is_empty =
            !ai_main_panel_can_submit(&self.input_state.read(cx).value(), has_pending_image);
        let input_char_count = self.input_state.read(cx).value().len();
        let input_is_long = input_char_count > 2000;
        let input_is_very_long = input_char_count > 4000;
        let composer_border_color = if input_is_very_long {
            cx.theme().danger.opacity(0.6)
        } else if input_is_long {
            cx.theme().warning.opacity(0.5)
        } else {
            cx.theme().accent
        };

        let input_area = div()
            .id("ai-input-area")
            .flex()
            .flex_col()
            .w_full()
            // NO .bg() - let vibrancy show through from root
            .border_t_1()
            .border_color(cx.theme().border.opacity(0.4))
            .px(PANEL_INSET_X)
            .py(S3)
            .gap(S2)
            // Handle image file drops
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            // Editing indicator (shown above input when editing a message)
            .when(is_editing, |d| d.child(self.render_editing_indicator(cx)))
            // Pending image preview (shown above input when image is attached)
            .when(has_pending_image, |d| {
                d.child(self.render_pending_image_preview(cx))
            })
            // Composer row: one surface for attachments + text input
            .child(
                div()
                    .id("ai-composer")
                    .flex()
                    .items_center()
                    .w_full()
                    .h(COMPOSER_H)
                    .px(S3)
                    .gap(S2)
                    .rounded(R_LG)
                    .border_1()
                    .border_color(composer_border_color.opacity(0.5))
                    .bg(cx.theme().muted.opacity(0.4))
                    // Plus button on the left - opens attachments picker
                    .child(
                        div()
                            .id("attachments-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(S6)
                            .rounded(R_MD)
                            .border_1()
                            .border_color(if has_pending_image {
                                cx.theme().accent.opacity(0.6)
                            } else {
                                cx.theme().muted_foreground.opacity(0.45)
                            })
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted.opacity(0.3)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.showing_attachments_picker {
                                    this.hide_attachments_picker(cx);
                                } else {
                                    this.hide_all_dropdowns(cx);
                                    this.show_attachments_picker(window, cx);
                                }
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Plus.external_path())
                                    .size(ICON_SM)
                                    .text_color(if has_pending_image {
                                        cx.theme().accent
                                    } else {
                                        cx.theme().muted_foreground
                                    }),
                            )
                            // Small accent dot when image is attached
                            .when(has_pending_image, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top(S0)
                                        .right(S0)
                                        .size(S2)
                                        .rounded_full()
                                        .bg(cx.theme().accent),
                                )
                            }),
                    )
                    .child(self.render_input_with_cursor(cx)),
            )
            // Bottom row: Model picker left, actions right
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .overflow_hidden()
                    // Left side: Model picker + char count
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .overflow_hidden()
                            .child(self.render_model_picker(cx))
                            // Word count (only shown when input has content)
                            .child({
                                let input_val = self.input_state.read(cx).value().to_string();
                                let word_count = input_val.split_whitespace().count();
                                let show_export = self.is_showing_export_feedback();
                                if show_export {
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(S1)
                                        .text_xs()
                                        .text_color(cx.theme().success.opacity(0.8))
                                        .child(
                                            svg()
                                                .external_path(LocalIconName::Check.external_path())
                                                .size(ICON_XS)
                                                .text_color(cx.theme().success.opacity(0.6)),
                                        )
                                        .child("Exported!")
                                        .into_any_element()
                                } else if word_count > 0 {
                                    let label = if word_count == 1 {
                                        "1 word".to_string()
                                    } else {
                                        format!("{} words", word_count)
                                    };
                                    let word_color = if input_is_very_long {
                                        cx.theme().danger.opacity(0.7)
                                    } else if input_is_long {
                                        cx.theme().warning.opacity(0.6)
                                    } else {
                                        cx.theme().muted_foreground.opacity(0.4)
                                    };
                                    div()
                                        .text_xs()
                                        .text_color(word_color)
                                        .child(label)
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            }),
                    )
                    // Right side: Submit and Actions as text labels
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S1)
                            .flex_shrink_0()
                            // Submit or Stop button
                            .child(if self.is_streaming {
                                div()
                                    .id("stop-btn")
                                    .flex()
                                    .items_center()
                                    .gap(S1)
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().danger.opacity(0.15)))
                                    .text_sm()
                                    .text_color(cx.theme().danger)
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _window, cx| {
                                            this.stop_streaming(cx);
                                        }),
                                    )
                                    .child(div().size(S2).bg(cx.theme().danger))
                                    .child("Stop")
                                    .child(
                                        div()
                                            .px(S1)
                                            .py(S0)
                                            .rounded(RADIUS_SM)
                                            .bg(cx.theme().danger.opacity(0.15))
                                            .text_xs()
                                            .text_color(cx.theme().danger.opacity(0.7))
                                            .child("Esc"),
                                    )
                                    .into_any_element()
                            } else {
                                div()
                                    .id("submit-btn")
                                    .flex()
                                    .items_center()
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .when(!input_is_empty, |d| {
                                        d.cursor_pointer()
                                            .hover(|s| s.bg(cx.theme().accent.opacity(0.15)))
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(|this, _, window, cx| {
                                                    let can_submit = {
                                                        let input_state = this.input_state.read(cx);
                                                        ai_main_panel_can_submit(
                                                            &input_state.value(),
                                                            this.pending_image.is_some(),
                                                        )
                                                    };

                                                    if can_submit {
                                                        this.submit_message(window, cx);
                                                    }
                                                }),
                                            )
                                    })
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if input_is_empty {
                                        cx.theme().muted_foreground.opacity(0.35)
                                    } else {
                                        cx.theme().accent
                                    })
                                    .child("Submit ↵")
                                    .into_any_element()
                            })
                            // Divider
                            .child(
                                div()
                                    .h(S4)
                                    .border_l_1()
                                    .border_color(cx.theme().border.opacity(0.6)),
                            )
                            // Actions ⌘K - opens command bar with AI-specific actions
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().accent.opacity(0.15)))
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().accent) // Yellow accent like main menu
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.show_command_bar(window, cx);
                                        }),
                                    )
                                    .child("Actions ⌘K"),
                            ),
                    ),
            );

        // Determine what to show in the content area
        let has_messages = !self.current_messages.is_empty() || self.is_streaming;

        // Build main layout
        // Structure: content area (flex_1, scrollable) -> input area (fixed)
        div()
            .id("ai-main-panel")
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .overflow_hidden()
            // Handle image file drops anywhere on the main panel
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            // Content area - this wrapper gets flex_1 to fill remaining space
            // The scrollable content goes inside this bounded container
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(if has_messages {
                        self.render_messages(cx).into_any_element()
                    } else {
                        self.render_welcome(cx).into_any_element()
                    }),
            )
            // Input area (fixed height, always visible at bottom)
            .child(input_area)
    }
}

#[cfg(test)]
mod tests {
    use super::ai_main_panel_can_submit;

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_text_present() {
        assert!(ai_main_panel_can_submit("hello", false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_pending_image_present_and_text_empty() {
        assert!(ai_main_panel_can_submit("", true));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_false_when_text_empty_and_no_pending_image() {
        assert!(!ai_main_panel_can_submit("", false));
    }
}
