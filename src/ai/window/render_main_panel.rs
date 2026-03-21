use super::*;
use crate::theme::opacity::{
    OPACITY_BORDER, OPACITY_DISABLED, OPACITY_SELECTED, OPACITY_TEXT_MUTED,
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
        // Row 1: single composer surface containing the input text field
        // Row 2: model picker + word count on left, submit/actions on right

        // Check if we have a pending image or context parts to show
        let has_pending_image = self.pending_image.is_some();
        let has_pending_context_parts = !self.pending_context_parts.is_empty();
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
            // Context resolution receipt (shown after submit when parts were resolved)
            .when(self.last_context_receipt.is_some(), |d| {
                d.child(self.render_context_receipt(cx))
            })
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
            // Composer row: one surface for text input
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

    /// Render a compact context-resolution receipt summary after submit,
    /// with an optional full inspector panel toggled via ⌥⌘I.
    fn render_context_receipt(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let receipt = match &self.last_context_receipt {
            Some(r) => r,
            None => return div().id("context-receipt-empty").into_any_element(),
        };

        let has_failures = receipt.has_failures();
        let duplicates_removed = self
            .last_prepared_message_receipt
            .as_ref()
            .and_then(|r| r.assembly.as_ref())
            .map(|a| a.duplicates_removed)
            .unwrap_or(0);

        let summary: SharedString = if has_failures {
            format!(
                "Context {} / {} resolved \u{00b7} {} failed \u{00b7} {} deduped",
                receipt.resolved,
                receipt.attempted,
                receipt.failures.len(),
                duplicates_removed
            )
            .into()
        } else if duplicates_removed > 0 {
            format!(
                "Context {} attached \u{00b7} {} deduped",
                receipt.resolved, duplicates_removed
            )
            .into()
        } else {
            format!("Context {} attached", receipt.resolved).into()
        };

        let (bg_color, text_color) = if has_failures {
            (
                cx.theme().danger.opacity(OPACITY_DISABLED),
                cx.theme().danger,
            )
        } else {
            (
                cx.theme().accent.opacity(OPACITY_DISABLED),
                cx.theme().accent,
            )
        };

        let shortcut_label: SharedString = "\u{2325}\u{2318}I".into();

        div()
            .id("context-receipt-summary")
            .flex()
            .flex_col()
            .gap(S2)
            .child(
                div()
                    .id("context-receipt-toggle")
                    .flex()
                    .items_center()
                    .gap(S2)
                    .px(S4)
                    .py(S1)
                    .rounded(R_MD)
                    .bg(bg_color)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_context_inspector(cx);
                    }))
                    .child(div().text_xs().text_color(text_color).child(summary))
                    .child(
                        div()
                            .ml_auto()
                            .text_xs()
                            .text_color(text_color.opacity(OPACITY_TEXT_MUTED))
                            .child(shortcut_label),
                    ),
            )
            .when(self.show_context_inspector, |container| {
                if let Some(prepared) = &self.last_prepared_message_receipt {
                    let json = serde_json::to_string_pretty(prepared).unwrap_or_else(|error| {
                        format!(
                            "{{\"error\":\"failed to serialize PreparedMessageReceipt: {}\"}}",
                            error
                        )
                    });
                    let json_text: SharedString = json.into();
                    container.child(
                        div()
                            .id("context-inspector")
                            .px(S4)
                            .py(S3)
                            .rounded(R_MD)
                            .bg(cx.theme().muted.opacity(OPACITY_DISABLED))
                            .max_h(px(300.0))
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child(json_text),
                            ),
                    )
                } else {
                    container
                }
            })
            .into_any_element()
    }

    /// Render chips representing pending context parts above the composer.
    ///
    /// Each ResourceUri chip includes an expand/collapse chevron that toggles
    /// an inline preview panel showing the source URI, profile, and payload
    /// summary. FilePath chips show only the close button (no preview needed).
    fn render_pending_context_chips(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = cx.theme().accent;
        let muted_fg = cx.theme().muted_foreground;
        let preview_idx = self.context_preview_index;

        let chips: Vec<_> = self
            .pending_context_parts
            .iter()
            .enumerate()
            .map(|(idx, part)| {
                let label: SharedString = part.label().to_string().into();
                let is_resource = matches!(
                    part,
                    crate::ai::message_parts::AiContextPart::ResourceUri { .. }
                );
                let is_previewed = preview_idx == Some(idx);
                let icon_name = if is_resource {
                    LocalIconName::Code
                } else {
                    LocalIconName::File
                };

                // Chip border highlights when its preview is open
                let chip_border = if is_previewed {
                    accent.opacity(OPACITY_SELECTED)
                } else {
                    accent.opacity(OPACITY_BORDER)
                };

                let mut chip = div()
                    .id(SharedString::from(format!("ctx-part-{}", idx)))
                    .flex()
                    .items_center()
                    .gap(S1)
                    .px(S2)
                    .py(S1)
                    .rounded(R_MD)
                    .bg(accent.opacity(OPACITY_DISABLED))
                    .border_1()
                    .border_color(chip_border)
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
                    );

                // Preview toggle (chevron) — only for ResourceUri chips
                if is_resource {
                    let chevron_icon = if is_previewed {
                        LocalIconName::ChevronDown
                    } else {
                        LocalIconName::ChevronRight
                    };
                    chip = chip.child(
                        div()
                            .id(SharedString::from(format!("ctx-preview-{}", idx)))
                            .cursor_pointer()
                            .hover(|el| el.text_color(accent))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_context_preview(idx, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(chevron_icon.external_path())
                                    .size(ICON_XS)
                                    .text_color(muted_fg),
                            ),
                    );
                }

                // Close button
                chip = chip.child(
                    div()
                        .id(SharedString::from(format!("ctx-remove-{}", idx)))
                        .cursor_pointer()
                        .hover(|el| el.text_color(cx.theme().danger))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            // Centralize preview index maintenance in remove_context_part().
                            this.remove_context_part(idx, cx);
                        }))
                        .child(
                            svg()
                                .external_path(LocalIconName::Close.external_path())
                                .size(ICON_XS)
                                .text_color(muted_fg),
                        ),
                );

                chip
            })
            .collect();

        let mut container = div()
            .id("pending-context-chips")
            .flex()
            .flex_col()
            .gap(S2)
            .child(div().flex().flex_row().flex_wrap().gap(S2).children(chips));

        // Inline preview panel — shown below chips when a ResourceUri is expanded
        if let Some((_, preview)) = self.active_context_preview() {
            container = container.child(self.render_context_preview_panel(&preview, cx));
        }

        container
    }

    /// Render the inline preview panel for an expanded context chip.
    fn render_context_preview_panel(
        &self,
        preview: &context_preview::ContextPreviewInfo,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let accent = theme.accent;
        let muted_fg = theme.muted_foreground;
        let fg = theme.foreground;

        let profile_label: SharedString = match preview.profile {
            context_preview::ContextPreviewProfile::Minimal => "Profile: minimal".into(),
            context_preview::ContextPreviewProfile::Full => "Profile: full".into(),
            context_preview::ContextPreviewProfile::Custom => "Profile: custom".into(),
            context_preview::ContextPreviewProfile::FilePath => "Type: file".into(),
        };

        // Visual distinction: full profile gets accent bg, minimal gets muted
        let profile_bg = match preview.profile {
            context_preview::ContextPreviewProfile::Full => accent.opacity(OPACITY_DISABLED),
            _ => theme.muted.opacity(OPACITY_DISABLED),
        };

        let uri_label: SharedString = preview.source_uri.clone().into();
        let desc_label: SharedString = preview.description.clone().into();

        let mut panel = div()
            .id("context-preview-panel")
            .flex()
            .flex_col()
            .gap(S1)
            .px(S3)
            .py(S2)
            .rounded(R_MD)
            .border_1()
            .border_color(accent.opacity(OPACITY_BORDER))
            .bg(theme.background.opacity(OPACITY_SELECTED))
            // Profile badge
            .child(
                div().flex().items_center().gap(S2).child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .px(S2)
                        .py(S0)
                        .rounded(R_SM)
                        .bg(profile_bg)
                        .text_color(fg)
                        .child(profile_label),
                ),
            )
            // Source URI
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(S1)
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_fg)
                            .child(SharedString::from("URI:")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(uri_label),
                    ),
            )
            // Description
            .child(div().text_xs().text_color(muted_fg).child(desc_label));

        // Diagnostics badge
        if preview.has_diagnostics {
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap(S1)
                    .child(
                        svg()
                            .external_path(LocalIconName::Warning.external_path())
                            .size(ICON_XS)
                            .text_color(theme.warning),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.warning)
                            .child(SharedString::from("Includes diagnostics")),
                    ),
            );
        }

        panel
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
