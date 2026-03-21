use super::*;
use crate::theme::opacity::{
    OPACITY_ACCENT_MEDIUM, OPACITY_BORDER, OPACITY_DISABLED, OPACITY_HOVER, OPACITY_SELECTED,
    OPACITY_TEXT_MUTED,
};

fn ai_main_panel_can_submit(
    input_value: &str,
    has_pending_image: bool,
    has_pending_context_parts: bool,
) -> bool {
    ai_window_can_submit_message(input_value, has_pending_image, has_pending_context_parts)
}

impl AiApp {
    pub(super) fn render_main_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Build input area at bottom:
        // Row 1: single composer surface containing [+] and input text field
        // Row 2: model picker + word count on left, submit/actions on right

        // Check if we have a pending image or context parts to show
        let has_pending_image = self.pending_image.is_some();
        let has_pending_context_parts = !self.pending_context_parts.is_empty();
        let has_pending_content = has_pending_image || has_pending_context_parts;
        let is_editing = self.editing_message_id.is_some();
        let input_value = self.input_state.read(cx).value().to_string();
        let input_is_empty =
            !ai_main_panel_can_submit(&input_value, has_pending_image, has_pending_context_parts);
        let input_word_count = if input_value.trim().is_empty() {
            0
        } else {
            input_value.split_whitespace().count()
        };
        let action_button_colors =
            crate::components::ButtonColors::from_theme(&crate::theme::get_cached_theme());
        let entity = cx.entity();
        let is_mouse_mode = self.input_mode == InputMode::Mouse;

        let input_area = div()
            .id("ai-input-area")
            .flex()
            .flex_col()
            .w_full()
            // NO .bg() - let vibrancy show through from root
            .border_t_1()
            .border_color(cx.theme().border.opacity(OPACITY_DISABLED))
            .px(MSG_PX)
            .py(S4)
            .gap(S3)
            // Handle image file drops
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            // Editing indicator (shown above input when editing a message)
            .when(is_editing, |d| d.child(self.render_editing_indicator(cx)))
            // Pending context part chips (shown above input when parts are attached)
            .when(has_pending_context_parts, |d| {
                d.child(self.render_pending_context_chips(cx))
            })
            // Pending image preview (shown above input when image is attached)
            .when(has_pending_image, |d| {
                d.child(self.render_pending_image_preview(cx))
            })
            // Composer row: one surface for attachments + text input
            .child(
                div()
                    .id("ai-composer")
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    .min_h(COMPOSER_H)
                    .px(S3)
                    .py(S2)
                    .gap(S2)
                    .rounded(R_LG)
                    .border_1()
                    .border_color(cx.theme().border.opacity(OPACITY_SELECTED))
                    .bg(cx.theme().muted.opacity(OPACITY_DISABLED))
                    // Plus button on the left
                    .child(
                        div()
                            .id("attachments-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_shrink_0()
                            .size(S6)
                            .rounded(R_MD)
                            .border_1()
                            .border_color(if has_pending_content {
                                cx.theme().accent.opacity(OPACITY_ACCENT_MEDIUM)
                            } else {
                                cx.theme().muted_foreground.opacity(OPACITY_BORDER)
                            })
                            .cursor_pointer()
                            .when(is_mouse_mode, |d| {
                                d.hover(|s| s.bg(cx.theme().muted.opacity(OPACITY_HOVER)))
                            })
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
                                    .text_color(if has_pending_content {
                                        cx.theme().accent
                                    } else {
                                        cx.theme().muted_foreground
                                    }),
                            )
                            // Small accent dot when image or context part is attached
                            .when(has_pending_content, |d| {
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
            // Bottom row: Model picker left, submit right
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .overflow_hidden()
                    // Left side: Model picker + word count
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .overflow_hidden()
                            .child(self.render_model_picker(cx))
                            .when(input_word_count > 0, |d| {
                                let label: SharedString =
                                    format!("~{} words", input_word_count).into();
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(
                                            cx.theme().muted_foreground.opacity(OPACITY_TEXT_MUTED),
                                        )
                                        .child(label),
                                )
                            }),
                    )
                    // Right side: Submit/Stop + Actions
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(S2)
                            .flex_shrink_0()
                            // Actions ⌘K
                            .child({
                                let actions_entity = entity.clone();
                                crate::components::Button::new("Actions", action_button_colors)
                                    .id("ai-actions-btn")
                                    .variant(crate::components::ButtonVariant::Ghost)
                                    .shortcut("⌘K")
                                    .on_click(Box::new(move |_, window, cx| {
                                        actions_entity.update(cx, |this, cx| {
                                            this.show_command_bar(window, cx);
                                        });
                                    }))
                            })
                            // Submit or Stop button
                            .child(if self.is_streaming {
                                let stop_entity = entity.clone();
                                crate::components::Button::new("Stop", action_button_colors)
                                    .id("stop-btn")
                                    .variant(crate::components::ButtonVariant::Ghost)
                                    .shortcut("Esc")
                                    .on_click(Box::new(move |_, _window, cx| {
                                        stop_entity.update(cx, |this, cx| {
                                            this.stop_streaming(cx);
                                        });
                                    }))
                                    .into_any_element()
                            } else {
                                let submit_button =
                                    crate::components::Button::new("Submit", action_button_colors)
                                        .id("submit-btn")
                                        .variant(crate::components::ButtonVariant::Ghost)
                                        .shortcut("↵")
                                        .disabled(input_is_empty);
                                if input_is_empty {
                                    submit_button.into_any_element()
                                } else {
                                    let submit_entity = entity.clone();
                                    submit_button
                                        .on_click(Box::new(move |_, window, cx| {
                                            submit_entity.update(cx, |this, cx| {
                                                this.submit_message(window, cx);
                                            });
                                        }))
                                        .into_any_element()
                                }
                            }),
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
            // Content area - fills remaining space above the input area.
            // min_h_0 is critical: without it a flex child won't shrink below its
            // content size, preventing overflow/scroll from working.
            .child(
                div()
                    .flex_1()
                    .min_h_0()
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

    /// Render chips representing pending context parts above the composer.
    fn render_pending_context_chips(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = cx.theme().accent;
        let muted_fg = cx.theme().muted_foreground;

        let chips: Vec<_> = self
            .pending_context_parts
            .iter()
            .enumerate()
            .map(|(idx, part)| {
                let label: SharedString = part.label().to_string().into();
                let icon_name = match part {
                    crate::ai::message_parts::AiContextPart::ResourceUri { .. } => {
                        LocalIconName::Code
                    }
                    crate::ai::message_parts::AiContextPart::FilePath { .. } => {
                        LocalIconName::File
                    }
                };

                div()
                    .id(SharedString::from(format!("ctx-part-{}", idx)))
                    .flex()
                    .items_center()
                    .gap(S1)
                    .px(S2)
                    .py(S1)
                    .rounded(R_MD)
                    .bg(accent.opacity(OPACITY_DISABLED))
                    .border_1()
                    .border_color(accent.opacity(OPACITY_BORDER))
                    .child(
                        svg()
                            .external_path(icon_name.external_path())
                            .size(ICON_XS)
                            .text_color(accent),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .overflow_hidden()
                            .text_ellipsis()
                            .max_w(px(160.0))
                            .child(label),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("ctx-remove-{}", idx)))
                            .cursor_pointer()
                            .hover(|el| el.text_color(cx.theme().danger))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_context_part(idx, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Close.external_path())
                                    .size(ICON_XS)
                                    .text_color(muted_fg),
                            ),
                    )
            })
            .collect();

        div()
            .id("pending-context-chips")
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(S2)
            .children(chips)
    }
}

#[cfg(test)]
mod tests {
    use super::ai_main_panel_can_submit;

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_text_present() {
        assert!(ai_main_panel_can_submit("hello", false, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_pending_image_present_and_text_empty() {
        assert!(ai_main_panel_can_submit("", true, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_false_when_text_empty_and_no_pending_image() {
        assert!(!ai_main_panel_can_submit("", false, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_false_for_whitespace_without_image() {
        assert!(!ai_main_panel_can_submit("   ", false, false));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_context_parts_present() {
        assert!(ai_main_panel_can_submit("", false, true));
    }

    #[test]
    fn test_ai_main_panel_can_submit_returns_true_when_context_parts_and_text_present() {
        assert!(ai_main_panel_can_submit("hello", false, true));
    }
}
