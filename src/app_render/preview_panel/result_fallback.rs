                    scripts::SearchResult::Fallback(fallback_match) => {
                        // Fallback command preview
                        let fallback = &fallback_match.fallback;

                        // Header showing "Fallback"
                        let mut path_div = div()
                            .flex()
                            .flex_row()
                            .text_xs()
                            .font_family(typography.font_family_mono)
                            .pb(px(spacing.padding_xs))
                            .overflow_x_hidden()
                            .child(
                                div()
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("fallback: "),
                            );

                        path_div = path_div.child(
                            div()
                                .text_color(rgba((text_muted << 8) | 0x99))
                                .child(fallback.name().to_string()),
                        );

                        panel = panel.child(path_div);

                        // Fallback name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(fallback.label().to_string()),
                        );

                        // Description
                        panel = panel.child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_secondary))
                                .pb(px(spacing.padding_md))
                                .child(fallback.description().to_string()),
                        );

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
                                        .child("Fallback"),
                                ),
                        );
                    }
