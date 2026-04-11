                    scripts::SearchResult::Fallback(fallback_match) => {
                        use super::{ALPHA_DIVIDER_DARK, ALPHA_MUTED_LABEL, ALPHA_SECTION_HEADER};

                        // Fallback command preview
                        let fallback = &fallback_match.fallback;
                        let fallback_name = fallback.display_name();
                        let fallback_label = fallback.display_label();
                        let fallback_description = fallback.display_description();

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
                                    .text_color(rgba((text_muted << 8) | ALPHA_MUTED_LABEL))
                                    .child("fallback: "),
                            );

                        path_div = path_div.child(
                            div()
                                .text_color(rgba((text_muted << 8) | ALPHA_MUTED_LABEL))
                                .child(fallback_name),
                        );

                        panel = panel.child(path_div);

                        // Fallback name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(fallback_label),
                        );

                        // Description
                        panel = panel.child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_secondary))
                                .pb(px(spacing.padding_md))
                                .child(fallback_description),
                        );

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | ALPHA_DIVIDER_DARK))
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
                                        .text_color(rgba((text_muted << 8) | ALPHA_SECTION_HEADER))
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
