                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        use super::ALPHA_SECTION_HEADER;

                        let builtin = &builtin_match.entry;

                        // Built-in name header — extra bottom padding for visual hierarchy
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_md))
                                .child(builtin.name.clone()),
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
                                            .text_color(rgba((text_muted << 8) | ALPHA_SECTION_HEADER))
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

                        // Description
                        panel = panel.child(
                            div()
                                .text_sm()
                                .line_height(px(20.0))
                                .text_color(rgb(text_secondary))
                                .pb(px(spacing.padding_lg))
                                .child(builtin.description.clone()),
                        );

                        // Keywords and feature type as subtle inline tags
                        let mut metadata_tags = preview_keyword_tags(&builtin.keywords);
                        let feature_tag = builtin_feature_annotation(&builtin.feature).to_lowercase();
                        if !metadata_tags
                            .iter()
                            .any(|tag| tag.eq_ignore_ascii_case(&feature_tag))
                        {
                            metadata_tags.push(feature_tag);
                        }

                        if !metadata_tags.is_empty() {
                            let mut tags_row = div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .gap(px(spacing.gap_sm))
                                .pb(px(spacing.padding_md));

                            for tag in metadata_tags {
                                tags_row = tags_row.child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(999.0))
                                        .bg(badge_bg)
                                        .border_1()
                                        .border_color(badge_border)
                                        .text_xs()
                                        .text_color(badge_text)
                                        .child(tag),
                                );
                            }

                            panel = panel.child(tags_row);
                        }
                    }
