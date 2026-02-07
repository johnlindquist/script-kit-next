                    scripts::SearchResult::App(app_match) => {
                        let app = &app_match.app;

                        // App name header — extra bottom padding for visual hierarchy
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_md))
                                .child(app.name.clone()),
                        );

                        // Keyboard shortcut (if assigned via config.commands)
                        if let Some(ref shortcut_str) = shortcut_display {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_lg))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgba((text_muted << 8) | 0xCC))
                                            .pb(px(spacing.padding_xs))
                                            .child("KEYBOARD SHORTCUT"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(spacing.gap_sm))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(accent_badge_text)
                                                    .child(shortcut_str.clone()),
                                            ),
                                    ),
                            );
                        }

                        // Path
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_lg))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("PATH"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .line_height(px(20.0))
                                        .text_color(rgb(text_secondary))
                                        .child(app.path.to_string_lossy().to_string()),
                                ),
                        );

                        // Bundle ID (if available)
                        if let Some(bundle_id) = &app.bundle_id {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_lg))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgba((text_muted << 8) | 0xCC))
                                            .pb(px(spacing.padding_xs))
                                            .child("BUNDLE ID"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .line_height(px(20.0))
                                            .text_color(rgb(text_secondary))
                                            .child(bundle_id.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("TYPE"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Application"),
                                ),
                        );
                    }
