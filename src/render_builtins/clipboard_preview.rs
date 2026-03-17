impl ScriptListApp {
    /// Render the preview panel for clipboard history
    fn render_clipboard_preview_panel(
        &self,
        selected_entry: &Option<clipboard_history::ClipboardEntryMeta>,
        image_cache: &std::collections::HashMap<String, Arc<gpui::RenderImage>>,
        spacing: &designs::DesignSpacing,
        typography: &designs::DesignTypography,
        visual: &designs::DesignVisual,
    ) -> impl IntoElement {
        let bg_main = self.theme.colors.background.main;
        let ui_border = self.theme.colors.ui.border;
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_secondary = self.theme.colors.text.secondary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let bg_search_box = self.theme.colors.background.search_box;
        let accent = self.theme.colors.accent.selected;

        let opacity = self.theme.get_opacity();
        let panel_alpha = (opacity.main * 255.0 * 0.30) as u8;
        let content_alpha = (opacity.main * 255.0 * 0.40) as u32;
        let image_bg_alpha = (opacity.main * 255.0 * 0.15) as u32;

        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgba((bg_main << 8) | panel_alpha as u32))
            .border_l_1()
            .border_color(rgba(
                (ui_border << 8) | u32::from(ui_foundation::ALPHA_BORDER_SUBTLE),
            ))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .gap(px(spacing.gap_sm))
            .overflow_y_hidden()
            .font_family(typography.font_family);

        match selected_entry {
            Some(entry) => {
                let mut header = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.gap_sm))
                    .child(
                        div()
                            .px(px(spacing.padding_sm))
                            .py(px(spacing.padding_xs / 2.0))
                            .rounded(px(visual.radius_sm))
                            .bg(rgba(
                                (accent << 8) | u32::from(ui_foundation::ALPHA_BORDER_SUBTLE),
                            ))
                            .text_xs()
                            .text_color(rgb(accent))
                            .child(
                                clipboard_preview_helpers::content_type_label(&entry.content_type)
                                    .to_string(),
                            ),
                    );

                if entry.pinned {
                    header = header.child(
                        div()
                            .px(px(spacing.padding_sm))
                            .py(px(spacing.padding_xs / 2.0))
                            .rounded(px(visual.radius_sm))
                            .bg(rgba(
                                (accent << 8) | u32::from(ui_foundation::ALPHA_TINT_SUBTLE),
                            ))
                            .text_xs()
                            .text_color(rgb(accent))
                            .child("Pinned"),
                    );
                }

                let relative_time = clipboard_preview_helpers::relative_time(
                    chrono::Utc::now().timestamp(),
                    entry.timestamp,
                );
                let absolute_time = clipboard_preview_helpers::absolute_time(entry.timestamp);

                panel = panel
                    .child(header)
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted))
                            .child(format!("{relative_time} \u{00b7} {absolute_time}")),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(visual.border_thin))
                            .bg(rgba(
                                (ui_border << 8) | u32::from(ui_foundation::ALPHA_DIVIDER),
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_muted))
                            .child("Preview"),
                    );

                match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => {
                        let (content, is_partial) =
                            match clipboard_history::get_entry_content(&entry.id) {
                                Some(content) => (content, false),
                                None => (entry.text_preview.clone(), true),
                            };

                        let char_count = content.chars().count();
                        let line_count = content.lines().count().max(1);

                        panel = panel
                            .child(
                                div()
                                    .id("clipboard-preview-content")
                                    .w_full()
                                    .flex_1()
                                    .p(px(spacing.padding_md))
                                    .rounded(px(visual.radius_md))
                                    .bg(rgba((bg_search_box << 8) | content_alpha))
                                    .overflow_y_scroll()
                                    .child(
                                        div()
                                            .w_full()
                                            .font_family(typography.font_family_mono)
                                            .text_sm()
                                            .text_color(rgb(text_primary))
                                            .child(content),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_secondary))
                                    .child(format!(
                                        "{} characters \u{2022} {} lines",
                                        char_count, line_count
                                    )),
                            );

                        if is_partial {
                            panel = panel.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child("Showing cached preview because the full clipboard payload is unavailable."),
                            );
                        }
                    }
                    clipboard_history::ContentType::Image => {
                        let width = entry.image_width.unwrap_or(0);
                        let height = entry.image_height.unwrap_or(0);
                        let cached_image = image_cache.get(&entry.id).cloned();

                        let image_container = if let Some(render_image) = cached_image {
                            let max_size: f32 = 300.0;
                            let (display_w, display_h) = if width > 0 && height > 0 {
                                let w = width as f32;
                                let h = height as f32;
                                let scale = (max_size / w).min(max_size / h).min(1.0);
                                (w * scale, h * scale)
                            } else {
                                (max_size, max_size)
                            };

                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(spacing.gap_sm))
                                .child(
                                    gpui::img(move |_window: &mut Window, _cx: &mut App| {
                                        Some(Ok(render_image.clone()))
                                    })
                                    .w(px(display_w))
                                    .h(px(display_h))
                                    .object_fit(gpui::ObjectFit::Contain)
                                    .rounded(px(visual.radius_sm)),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(format!("{}×{} pixels", width, height)),
                                )
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(spacing.gap_sm))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child("Image preview loading"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child(format!("{}×{} pixels", width, height)),
                                )
                        };

                        panel = panel.child(
                            div()
                                .w_full()
                                .flex_1()
                                .p(px(spacing.padding_lg))
                                .rounded(px(visual.radius_md))
                                .bg(rgba((bg_search_box << 8) | image_bg_alpha))
                                .flex()
                                .items_center()
                                .justify_center()
                                .overflow_hidden()
                                .child(image_container),
                        );
                    }
                }
            }
            None => {
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child("No entry selected"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child("Use \u{2191}\u{2193} to inspect history"),
                        ),
                );
            }
        }

        panel
    }
}
