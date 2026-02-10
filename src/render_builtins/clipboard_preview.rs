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
        // Use theme colors for consistency with main menu
        let bg_main = self.theme.colors.background.main;
        let ui_border = self.theme.colors.ui.border;
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_secondary = self.theme.colors.text.secondary;
        let bg_search_box = self.theme.colors.background.search_box;
        let accent = self.theme.colors.accent.selected;

        // Use theme opacity for vibrancy-compatible backgrounds
        let opacity = self.theme.get_opacity();
        let panel_alpha = (opacity.main * 255.0 * 0.3) as u8; // 30% of main opacity for subtle tint

        let mut panel = div()
            .w_full()
            .h_full()
            // Semi-transparent background to let vibrancy show through
            .bg(rgba((bg_main << 8) | panel_alpha as u32))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x30)) // Subtle border
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(typography.font_family);

        match selected_entry {
            Some(entry) => {
                // Header with content type
                let content_type_label = match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => "Text",
                    clipboard_history::ContentType::Image => "Image",
                };

                panel = panel.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .pb(px(spacing.padding_sm))
                        // Content type badge
                        .child(
                            div()
                                .px(px(spacing.padding_sm))
                                .py(px(spacing.padding_xs / 2.0))
                                .rounded(px(visual.radius_sm))
                                .bg(rgba((accent << 8) | 0x30))
                                .text_xs()
                                .text_color(rgb(accent))
                                .child(content_type_label),
                        )
                        // Pin indicator
                        .when(entry.pinned, |d| {
                            d.child(
                                div()
                                    .px(px(spacing.padding_sm))
                                    .py(px(spacing.padding_xs / 2.0))
                                    .rounded(px(visual.radius_sm))
                                    .bg(rgba((accent << 8) | 0x20))
                                    .text_xs()
                                    .text_color(rgb(accent))
                                    .child("📌 Pinned"),
                            )
                        }),
                );

                // Timestamp
                let now = chrono::Utc::now().timestamp();
                let age_secs = now - entry.timestamp;
                let relative_time = if age_secs < 60 {
                    "just now".to_string()
                } else if age_secs < 3600 {
                    format!("{} minutes ago", age_secs / 60)
                } else if age_secs < 86400 {
                    format!("{} hours ago", age_secs / 3600)
                } else {
                    format!("{} days ago", age_secs / 86400)
                };

                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_md))
                        .child(relative_time),
                );

                // Divider
                panel = panel.child(
                    div()
                        .w_full()
                        .h(px(visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .my(px(spacing.padding_sm)),
                );

                // Content preview
                panel = panel.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .pb(px(spacing.padding_sm))
                        .child("Content Preview"),
                );

                match entry.content_type {
                    clipboard_history::ContentType::Text
                    | clipboard_history::ContentType::Link
                    | clipboard_history::ContentType::File
                    | clipboard_history::ContentType::Color => {
                        // Fetch full content on-demand for preview
                        let content = clipboard_history::get_entry_content(&entry.id)
                            .unwrap_or_else(|| entry.text_preview.clone());
                        let char_count = content.chars().count();
                        let line_count = content.lines().count();

                        // Use subtle background for content preview - 20% opacity for vibrancy
                        let content_alpha = (opacity.main * 255.0 * 0.5) as u32;
                        panel = panel
                            .child(
                                div()
                                    .w_full()
                                    .flex_1()
                                    .p(px(spacing.padding_md))
                                    .rounded(px(visual.radius_md))
                                    .bg(rgba((bg_search_box << 8) | content_alpha))
                                    .overflow_hidden()
                                    .font_family(typography.font_family_mono)
                                    .text_sm()
                                    .text_color(rgb(text_primary))
                                    .child(content),
                            )
                            // Stats footer
                            .child(
                                div()
                                    .pt(px(spacing.padding_sm))
                                    .text_xs()
                                    .text_color(rgb(text_secondary))
                                    .child(format!(
                                        "{} characters • {} lines",
                                        char_count, line_count
                                    )),
                            );
                    }
                    clipboard_history::ContentType::Image => {
                        // Get image dimensions from metadata
                        let width = entry.image_width.unwrap_or(0);
                        let height = entry.image_height.unwrap_or(0);

                        // Try to get cached render image
                        let cached_image = image_cache.get(&entry.id).cloned();

                        let image_container = if let Some(render_image) = cached_image {
                            // Calculate display size that fits in the preview panel
                            // Max size is 300x300, maintain aspect ratio
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
                                .gap_2()
                                // Actual image thumbnail
                                .child(
                                    gpui::img(move |_window: &mut Window, _cx: &mut App| {
                                        Some(Ok(render_image.clone()))
                                    })
                                    .w(px(display_w))
                                    .h(px(display_h))
                                    .object_fit(gpui::ObjectFit::Contain)
                                    .rounded(px(visual.radius_sm)),
                                )
                                // Dimensions label below image
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(format!("{}×{} pixels", width, height)),
                                )
                        } else {
                            // Fallback if image not in cache (shouldn't happen)
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                .child(div().text_2xl().child("🖼️"))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child(format!("{}×{}", width, height)),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_muted))
                                        .child("Loading image..."),
                                )
                        };

                        // Image container - use very subtle background for vibrancy
                        let image_bg_alpha = (opacity.main * 255.0 * 0.15) as u32; // 15% of main opacity
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
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child("No entry selected"),
                );
            }
        }

        panel
    }
}
