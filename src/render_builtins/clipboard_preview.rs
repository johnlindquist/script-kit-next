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
                let relative_time = clipboard_preview_helpers::relative_time(
                    chrono::Utc::now().timestamp(),
                    entry.timestamp,
                );
                let absolute_time = clipboard_preview_helpers::absolute_time(entry.timestamp);
                let type_label =
                    clipboard_preview_helpers::content_type_label(&entry.content_type).to_string();
                let pinned_label = if entry.pinned { "Yes" } else { "No" }.to_string();

                let info_row = |label: &'static str, value: String| {
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(spacing.gap_md))
                        .child(div().text_xs().text_color(rgb(text_muted)).child(label))
                        .child(div().text_xs().text_color(rgb(text_primary)).child(value))
                };

                let mut information = div()
                    .id("clipboard-preview-information")
                    .w_full()
                    .flex_none()
                    .border_t_1()
                    .border_color(rgba(
                        (ui_border << 8) | u32::from(ui_foundation::ALPHA_DIVIDER),
                    ))
                    .pt(px(spacing.padding_md))
                    .flex()
                    .flex_col()
                    .gap(px(spacing.gap_sm))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_secondary))
                            .child("Information"),
                    )
                    .child(info_row("Type", type_label))
                    .child(info_row(
                        "Copied",
                        format!("{relative_time} \u{00b7} {absolute_time}"),
                    ))
                    .child(info_row("Size", format!("{} bytes", entry.byte_size)))
                    .child(info_row("Pinned", pinned_label));

                let content_area: AnyElement = match entry.content_type {
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
                        information = information
                            .child(info_row("Characters", char_count.to_string()))
                            .child(info_row("Lines", line_count.to_string()));

                        if is_partial {
                            information = information.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child("Showing cached preview because the full clipboard payload is unavailable."),
                            );
                        }

                        div()
                            .id("clipboard-preview-content-area")
                            .w_full()
                            .flex_1()
                            .min_h(px(0.0))
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
                            )
                            .into_any_element()
                    }
                    clipboard_history::ContentType::Image => {
                        let width = entry.image_width.unwrap_or(0);
                        let height = entry.image_height.unwrap_or(0);
                        let cached_image = image_cache.get(&entry.id).cloned();
                        let dimensions = if width > 0 && height > 0 {
                            format!("{}×{} pixels", width, height)
                        } else {
                            "Unknown".to_string()
                        };
                        let ocr_status = entry
                            .ocr_text
                            .as_ref()
                            .filter(|text| !text.is_empty())
                            .map(|_| "Available")
                            .unwrap_or("Not available")
                            .to_string();

                        information = information
                            .child(info_row("Dimensions", dimensions))
                            .child(info_row("OCR", ocr_status));

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

                            div().flex().flex_col().items_center().child(
                                gpui::img(move |_window: &mut Window, _cx: &mut App| {
                                    Some(Ok(render_image.clone()))
                                })
                                .w(px(display_w))
                                .h(px(display_h))
                                .object_fit(gpui::ObjectFit::Contain)
                                .rounded(px(visual.radius_sm)),
                            )
                        } else {
                            div().flex().flex_col().items_center().child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(text_primary))
                                    .child("Image preview loading"),
                            )
                        };

                        div()
                            .id("clipboard-preview-content-area")
                            .w_full()
                            .flex_1()
                            .min_h(px(0.0))
                            .p(px(spacing.padding_lg))
                            .rounded(px(visual.radius_md))
                            .bg(rgba((bg_search_box << 8) | image_bg_alpha))
                            .flex()
                            .items_center()
                            .justify_center()
                            .overflow_hidden()
                            .child(image_container)
                            .into_any_element()
                    }
                };

                panel = panel.child(content_area).child(information);
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
