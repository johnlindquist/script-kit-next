                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        use super::{
                            ALPHA_CODE_BG,
                            ALPHA_DIVIDER_DARK,
                            ALPHA_DIVIDER_LIGHT,
                            ALPHA_MUTED_LABEL,
                            ALPHA_SECTION_HEADER,
                        };

                        let scriptlet = &scriptlet_match.scriptlet;

                        // Source indicator with match highlighting (e.g., "scriptlet: foo.md")
                        if let Some(ref display_file_path) = scriptlet_match.display_file_path {
                            // P4: Use lazily computed indices instead of stored (empty) ones
                            let filename_indices = &match_indices.filename_indices;

                            // Render filename with highlighted matched characters
                            let path_segments = render_path_with_highlights(
                                display_file_path,
                                display_file_path,
                                filename_indices,
                            );
                            let accent_color = colors.accent;

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
                                        .child("scriptlet: "),
                                );

                            for (text, is_highlighted) in path_segments {
                                let color = if is_highlighted {
                                    rgb(accent_color)
                                } else {
                                    rgba((text_muted << 8) | ALPHA_MUTED_LABEL)
                                };
                                path_div = path_div.child(div().text_color(color).child(text));
                            }

                            panel = panel.child(path_div);
                        }

                        // Scriptlet name header — extra bottom padding for visual hierarchy
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_md))
                                .child(scriptlet.name.clone()),
                        );

                        // Scriptlet metadata badges: tool type, group, alias, keyword
                        {
                            let mut slet_badges = div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .gap(px(spacing.gap_sm))
                                .pb(px(spacing.padding_sm));
                            slet_badges = slet_badges.child(
                                div()
                                    .px(px(6.))
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(badge_bg)
                                    .border_1()
                                    .border_color(badge_border)
                                    .text_xs()
                                    .text_color(badge_text)
                                    .child(scriptlet.tool_display_name().to_string()),
                            );
                            if let Some(ref group) = scriptlet.group {
                                if !group.is_empty() {
                                    slet_badges = slet_badges.child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(4.))
                                            .bg(badge_bg)
                                            .border_1()
                                            .border_color(badge_border)
                                            .text_xs()
                                            .text_color(badge_text)
                                            .child(group.clone()),
                                    );
                                }
                            }
                            if let Some(ref alias) = scriptlet.alias {
                                slet_badges = slet_badges.child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(accent_badge_bg)
                                        .border_1()
                                        .border_color(accent_badge_border)
                                        .text_xs()
                                        .text_color(accent_badge_text)
                                        .child(format!("alias: {}", alias)),
                                );
                            }
                            if let Some(ref keyword) = scriptlet.keyword {
                                slet_badges = slet_badges.child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(accent_badge_bg)
                                        .border_1()
                                        .border_color(accent_badge_border)
                                        .text_xs()
                                        .text_color(accent_badge_text)
                                        .child(format!("keyword: {}", keyword)),
                                );
                            }
                            panel = panel.child(slet_badges);
                        }

                        // Description (if present)
                        if let Some(desc) = &scriptlet.description {
                            panel = panel.child(
                                div()
                                    .text_sm()
                                    .line_height(px(20.0))
                                    .text_color(rgb(text_secondary))
                                    .pb(px(spacing.padding_lg))
                                    .child(desc.clone()),
                            );
                        }

                        // Shortcut: prefer inline shortcut from scriptlet, fall back to config-based
                        let effective_shortcut = scriptlet
                            .shortcut
                            .clone()
                            .or_else(|| shortcut_display.clone());
                        if let Some(shortcut) = effective_shortcut {
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
                                                    .child(shortcut),
                                            ),
                                    ),
                            );
                        }

                        // Divider — subtle separation before content preview
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba(
                                    (ui_border << 8)
                                        | if is_light_mode {
                                            ALPHA_DIVIDER_LIGHT
                                        } else {
                                            ALPHA_DIVIDER_DARK
                                        },
                                ))
                                .my(px(spacing.padding_sm)),
                        );

                        // Content preview header
                        panel = panel.child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgba((text_muted << 8) | ALPHA_SECTION_HEADER))
                                .pb(px(spacing.padding_sm))
                                .child("CONTENT PREVIEW"),
                        );

                        // Display scriptlet code with syntax highlighting
                        // PERF DEBUG: Detailed timing for each step
                        let total_start = std::time::Instant::now();

                        // Step 1: Cache key check
                        let cache_key = scriptlet.name.clone();
                        let is_cache_hit =
                            self.scriptlet_preview_cache_key.as_ref() == Some(&cache_key);

                        // Step 2: Get highlighted lines (from cache or compute)
                        let step2_start = std::time::Instant::now();
                        let lines = if is_cache_hit {
                            self.scriptlet_preview_cache_lines.clone()
                        } else {
                            // PERF: Truncate long lines to prevent minified code from
                            // creating thousands of syntax highlighting spans.
                            // 120 chars is a reasonable max for preview display.
                            const MAX_LINE_LENGTH: usize = 120;
                            let code_preview: String = scriptlet
                                .code
                                .lines()
                                .take(15)
                                .map(|line| {
                                    if line.len() > MAX_LINE_LENGTH {
                                        format!("{}...", &line[..MAX_LINE_LENGTH])
                                    } else {
                                        line.to_string()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n");

                            let lang = match scriptlet.tool.as_str() {
                                "bash" | "zsh" | "sh" => "bash",
                                "node" | "bun" => "js",
                                _ => &scriptlet.tool,
                            };
                            let is_dark = self.theme.is_dark_mode();
                            let highlighted = highlight_code_lines(&code_preview, lang, is_dark);
                            self.scriptlet_preview_cache_key = Some(cache_key);
                            self.scriptlet_preview_cache_lines = highlighted.clone();
                            highlighted
                        };
                        let step2_ms = step2_start.elapsed().as_secs_f64() * 1000.0;

                        // Step 3: Create container div
                        let step3_start = std::time::Instant::now();
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | ALPHA_CODE_BG))
                            .overflow_hidden()
                            .flex()
                            .flex_col();
                        let step3_ms = step3_start.elapsed().as_secs_f64() * 1000.0;

                        // Step 4: Create line divs with spans
                        let step4_start = std::time::Instant::now();
                        let line_count = lines.len();
                        let mut span_count = 0usize;

                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg));

                            if line.spans.is_empty() {
                                line_div = line_div.child(" ");
                            } else {
                                span_count += line.spans.len();
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }
                        let step4_ms = step4_start.elapsed().as_secs_f64() * 1000.0;

                        // Step 5: Add to panel
                        let step5_start = std::time::Instant::now();
                        panel = panel.child(code_container);
                        let step5_ms = step5_start.elapsed().as_secs_f64() * 1000.0;

                        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

                        // Always log for debugging
                        logging::log(
                            "CODE_PERF",
                            &format!(
                                "[SYNTAX] {} lines={} spans={} | cache={} get={:.2}ms container={:.2}ms lines={:.2}ms add={:.2}ms TOTAL={:.2}ms",
                                if is_cache_hit { "HIT" } else { "MISS" },
                                line_count,
                                span_count,
                                if is_cache_hit { "hit" } else { "miss" },
                                step2_ms,
                                step3_ms,
                                step4_ms,
                                step5_ms,
                                total_ms
                            ),
                        );
                    }
