                    scripts::SearchResult::Agent(agent_match) => {
                        let agent = &agent_match.agent;

                        // Source indicator with agent path
                        let filename = agent
                            .path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "agent".to_string());

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
                                    .child("agent: "),
                            );

                        path_div = path_div.child(
                            div()
                                .text_color(rgba((text_muted << 8) | 0x99))
                                .child(filename),
                        );

                        panel = panel.child(path_div);

                        // Agent name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(agent.name.clone()),
                        );

                        // Description
                        if let Some(desc) = &agent.description {
                            panel = panel.child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_secondary))
                                    .pb(px(spacing.padding_md))
                                    .child(desc.clone()),
                            );
                        }

                        // Backend info
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("BACKEND"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(format!("{:?}", agent.backend)),
                                ),
                        );

                        // Kit info if available
                        if let Some(kit) = &agent.kit {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgba((text_muted << 8) | 0xCC))
                                            .pb(px(spacing.padding_xs))
                                            .child("KIT"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(kit.clone()),
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
                                        .child("Agent"),
                                ),
                        );
                    }

