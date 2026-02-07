                    scripts::SearchResult::Script(script_match) => {
                        let script = &script_match.script;

                        // Source indicator with match highlighting (e.g., "script: foo.ts")
                        let filename = &script_match.filename;
                        // P4: Use lazily computed indices instead of stored (empty) ones
                        let filename_indices = &match_indices.filename_indices;

                        // Render filename with highlighted matched characters
                        let path_segments =
                            render_path_with_highlights(filename, filename, filename_indices);
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
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("script: "),
                            );

                        for (text, is_highlighted) in path_segments {
                            let color = if is_highlighted {
                                rgb(accent_color)
                            } else {
                                rgba((text_muted << 8) | 0x99)
                            };
                            path_div = path_div.child(div().text_color(color).child(text));
                        }

                        panel = panel.child(path_div);

                        // Script name header — extra bottom padding for visual hierarchy
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_md))
                                .child(format!("{}.{}", script.name, script.extension)),
                        );

                        // Script metadata badges: kit name, alias, author, tags
                        {
                            let mut script_badges = div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .gap(px(spacing.gap_sm))
                                .pb(px(spacing.padding_sm));
                            let mut show_script_badges = false;
                            if let Some(ref kit) = script.kit_name {
                                show_script_badges = true;
                                script_badges = script_badges.child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(badge_bg)
                                        .border_1()
                                        .border_color(badge_border)
                                        .text_xs()
                                        .text_color(badge_text)
                                        .child(format!("kit: {}", kit)),
                                );
                            }
                            // Extension badge (e.g., "TypeScript", "JavaScript", "Shell")
                            {
                                let ext_display = match script.extension.as_str() {
                                    "ts" => "TypeScript",
                                    "js" => "JavaScript",
                                    "mjs" => "JavaScript",
                                    "sh" => "Shell",
                                    "bash" => "Bash",
                                    "zsh" => "Zsh",
                                    "py" => "Python",
                                    "rb" => "Ruby",
                                    _ => "",
                                };
                                if !ext_display.is_empty() {
                                    show_script_badges = true;
                                    script_badges = script_badges.child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(4.))
                                            .bg(badge_bg)
                                            .border_1()
                                            .border_color(badge_border)
                                            .text_xs()
                                            .text_color(badge_text)
                                            .child(ext_display.to_string()),
                                    );
                                }
                            }
                            if let Some(ref alias) = script.alias {
                                show_script_badges = true;
                                script_badges = script_badges.child(
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
                            // Author badge from typed metadata
                            if let Some(ref typed_meta) = script.typed_metadata {
                                if let Some(ref author) = typed_meta.author {
                                    show_script_badges = true;
                                    script_badges = script_badges.child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(4.))
                                            .bg(badge_bg)
                                            .border_1()
                                            .border_color(badge_border)
                                            .text_xs()
                                            .text_color(badge_text)
                                            .child(format!("by {}", author)),
                                    );
                                }
                                // Tags badges from typed metadata
                                for tag in &typed_meta.tags {
                                    show_script_badges = true;
                                    script_badges = script_badges.child(
                                        div()
                                            .px(px(6.))
                                            .py(px(2.))
                                            .rounded(px(4.))
                                            .bg(badge_bg)
                                            .border_1()
                                            .border_color(badge_border)
                                            .text_xs()
                                            .text_color(badge_text)
                                            .child(tag.clone()),
                                    );
                                }
                            }
                            if show_script_badges {
                                panel = panel.child(script_badges);
                            }
                        }

                        // Keyboard shortcut: prefer script metadata shortcut, fall back to config-based
                        let effective_shortcut =
                            script.shortcut.clone().or_else(|| shortcut_display.clone());
                        if let Some(shortcut_str) = effective_shortcut {
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
                                                    .child(shortcut_str),
                                            ),
                                    ),
                            );
                        }

                        // Description (if present)
                        if let Some(desc) = &script.description {
                            panel = panel.child(
                                div()
                                    .text_sm()
                                    .line_height(px(20.0))
                                    .text_color(rgb(text_secondary))
                                    .pb(px(spacing.padding_lg))
                                    .child(desc.clone()),
                            );
                        }

                        // Divider — subtle separation before code preview
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | if is_light_mode { 0x30 } else { 0x60 }))
                                .my(px(spacing.padding_sm)),
                        );

                        // Code preview header
                        panel = panel.child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgba((text_muted << 8) | 0xCC))
                                .pb(px(spacing.padding_sm))
                                .child("CODE PREVIEW"),
                        );

                        // Use cached syntax-highlighted lines (avoids file I/O and highlighting on every render)
                        let script_path = script.path.to_string_lossy().to_string();
                        let lang = script.extension.clone();
                        let is_dark = self.theme.is_dark_mode();
                        let cache_start = std::time::Instant::now();
                        let lines = self
                            .get_or_update_preview_cache(&script_path, &lang, is_dark)
                            .to_vec();
                        let cache_elapsed = cache_start.elapsed();
                        if cache_elapsed.as_micros() > 500 {
                            logging::log(
                                "FILTER_PERF",
                                &format!(
                                    "[PREVIEW] preview_cache for '{}' took {:.2}ms ({} lines, filter='{}')",
                                    script.name,
                                    cache_elapsed.as_secs_f64() * 1000.0,
                                    lines.len(),
                                    filter_for_log
                                ),
                            );
                        }

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
