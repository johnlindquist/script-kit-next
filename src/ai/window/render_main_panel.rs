use super::*;

impl AiApp {
    pub(super) fn render_main_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Build titlebar - just a spacer with border (title is now globally centered at window level)
        let titlebar = div()
            .id("ai-titlebar")
            .h(TITLEBAR_H)
            // NO .bg() - let vibrancy show through from root
            .border_b_1()
            .border_color(cx.theme().border);

        // Build input area at bottom - Raycast-style layout:
        // Row 1: [+ icon] [input field with magenta border]
        // Row 2: [Model picker with spinner] ... [Submit ↵] | [Actions ⌘K]

        // Use theme accent color for input border (follows theme)
        let input_border_color = cx.theme().accent;

        // Check if we have a pending image to show
        let has_pending_image = self.pending_image.is_some();
        let is_editing = self.editing_message_id.is_some();
        let input_is_empty = self.input_state.read(cx).value().is_empty() && !has_pending_image;
        let input_char_count = self.input_state.read(cx).value().len();
        let input_is_long = input_char_count > 2000;
        let input_is_very_long = input_char_count > 4000;

        let input_area = div()
            .id("ai-input-area")
            .flex()
            .flex_col()
            .w_full()
            // NO .bg() - let vibrancy show through from root
            .border_t_1()
            .border_color(cx.theme().border.opacity(0.4))
            .px(SP_7)
            .pt(SP_5)
            .pb(SP_5)
            .gap(SP_4)
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
            // Input row with + icon and accent border
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .w_full()
                    // Plus button on the left - opens attachments picker
                    .child(
                        div()
                            .id("attachments-btn")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.))
                            .rounded_full()
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
                                        .top(px(-1.))
                                        .right(px(-1.))
                                        .size(px(7.))
                                        .rounded_full()
                                        .bg(cx.theme().accent),
                                )
                            }),
                    )
                    // Input field with subtle accent border
                    .child(self.render_input_with_cursor(
                        if input_is_very_long {
                            cx.theme().danger.opacity(0.6)
                        } else if input_is_long {
                            cx.theme().warning.opacity(0.5)
                        } else {
                            input_border_color
                        },
                        cx,
                    )),
            )
            // Bottom row: Model picker left, actions right (reduced padding)
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
                            .gap_2()
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
                                        .gap(px(4.))
                                        .text_xs()
                                        .text_color(cx.theme().success.opacity(0.8))
                                        .child(
                                            svg()
                                                .external_path(LocalIconName::Check.external_path())
                                                .size(px(11.))
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
                            .gap_1()
                            .flex_shrink_0()
                            // Submit or Stop button
                            .child(if self.is_streaming {
                                div()
                                    .id("stop-btn")
                                    .flex()
                                    .items_center()
                                    .gap(px(5.))
                                    .px(px(10.))
                                    .py(px(4.))
                                    .rounded(px(6.))
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
                                    .child(div().size(px(8.)).rounded(px(1.)).bg(cx.theme().danger))
                                    .child("Stop")
                                    .child(
                                        div()
                                            .px(px(4.))
                                            .py(px(1.))
                                            .rounded(px(3.))
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
                                    .px(px(10.))
                                    .py(px(4.))
                                    .rounded(px(6.))
                                    .when(!input_is_empty, |d| {
                                        d.cursor_pointer()
                                            .hover(|s| s.bg(cx.theme().accent.opacity(0.15)))
                                    })
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if input_is_empty {
                                        cx.theme().muted_foreground.opacity(0.35)
                                    } else {
                                        cx.theme().accent
                                    })
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, window, cx| {
                                            this.submit_message(window, cx);
                                        }),
                                    )
                                    .child("Submit ↵")
                                    .into_any_element()
                            })
                            // Divider
                            .child(
                                div()
                                    .w(px(1.))
                                    .h(px(18.))
                                    .bg(cx.theme().border.opacity(0.6)),
                            )
                            // Actions ⌘K - opens command bar with AI-specific actions
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .px(px(10.))
                                    .py(px(4.))
                                    .rounded(px(6.))
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
        // Structure: titlebar (fixed) -> content area (flex_1, scrollable) -> input area (fixed)
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
            // Titlebar (fixed height)
            .child(titlebar)
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
