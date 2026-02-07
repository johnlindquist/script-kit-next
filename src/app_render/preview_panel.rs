fn preview_panel_typography_section_label_size(typography: designs::DesignTypography) -> f32 {
    typography.font_size_xs
}

fn preview_panel_typography_body_line_height(typography: designs::DesignTypography) -> f32 {
    typography.font_size_sm * typography.line_height_relaxed
}

impl ScriptListApp {
    #[allow(dead_code)]
    fn read_script_preview(path: &std::path::Path, max_lines: usize) -> String {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let preview: String = content
                    .lines()
                    .take(max_lines)
                    .collect::<Vec<_>>()
                    .join("\n");
                logging::log(
                    "UI",
                    &format!(
                        "Preview loaded: {} ({} lines read)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        content.lines().count().min(max_lines)
                    ),
                );
                preview
            }
            Err(e) => {
                logging::log("UI", &format!("Preview error: {} - {}", path.display(), e));
                format!("Error reading file: {}", e)
            }
        }
    }

    // NOTE: render_toasts() removed - now using gpui-component's NotificationList
    // via the Root wrapper. Toasts are flushed via flush_pending_toasts() in render().
    // See toast_manager.rs for the queue and main.rs for the flush logic.

    /// Get the command ID for a search result, used for config lookups (shortcuts, etc.)
    fn get_command_id_for_result(result: &scripts::SearchResult) -> Option<String> {
        match result {
            scripts::SearchResult::Script(m) => {
                // Script command ID: "script/{name}" (without extension)
                Some(format!("script/{}", m.script.name))
            }
            scripts::SearchResult::Scriptlet(m) => {
                // Scriptlet command ID: "scriptlet/{name}"
                Some(format!("scriptlet/{}", m.scriptlet.name))
            }
            scripts::SearchResult::BuiltIn(m) => {
                // Built-in command ID: "builtin/{id}"
                Some(format!("builtin/{}", m.entry.id))
            }
            scripts::SearchResult::App(m) => {
                // App command ID: "app/{bundle_id}" or "app/{name}"
                if let Some(ref bundle_id) = m.app.bundle_id {
                    Some(format!("app/{}", bundle_id))
                } else {
                    Some(format!(
                        "app/{}",
                        m.app.name.to_lowercase().replace(' ', "-")
                    ))
                }
            }
            // Window, Agent, and Fallback don't support shortcuts
            _ => None,
        }
    }

    /// Render the preview panel showing details of the selected script/scriptlet
    fn render_preview_panel(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        let preview_start = std::time::Instant::now();
        let filter_for_log = self.filter_text.clone();

        // Only log when meaningful state changed (flag set by render_script_list)
        // This eliminates cursor-blink log spam
        if self.log_this_render {
            logging::log(
                "PREVIEW_PERF",
                &format!(
                    "[PREVIEW_START] filter='{}' selected_idx={}",
                    filter_for_log, self.selected_index
                ),
            );
        }
        let _ = preview_start; // Used in PREVIEW_PANEL_DONE below

        // Get grouped results to map from selected_index to actual result (cached)
        // Clone to avoid borrow issues with self.selected_index access
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let selected_result = match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        };

        // Use design tokens for GLOBAL theming - design applies to ALL components
        let tokens = get_tokens(self.current_design);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let typography = tokens.typography();
        let visual = tokens.visual();

        // Map design tokens to local variables (all designs use tokens now)
        // In light mode, override text colors for readability on light backgrounds
        let is_light_mode = !self.theme.is_dark_mode();
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = if is_light_mode {
            self.theme.colors.text.primary // Pure black or dark gray
        } else {
            colors.text_primary
        };
        let text_muted = if is_light_mode {
            self.theme.colors.text.muted // Dark gray for light mode
        } else {
            colors.text_muted
        };
        let text_secondary = if is_light_mode {
            self.theme.colors.text.secondary // Medium gray for light mode
        } else {
            colors.text_secondary
        };
        let bg_search_box = if is_light_mode {
            // Use a subtle gray for code blocks in light mode
            // 0xf0f0f0 provides good contrast without being too dark
            0xf0f0f0
        } else {
            colors.background_tertiary
        };
        let border_radius = visual.radius_md;
        let font_family = typography.font_family;
        let section_label_font_size = preview_panel_typography_section_label_size(typography);
        let body_text_line_height = preview_panel_typography_body_line_height(typography);

        // Preview badge colors — light mode needs opaque fills for vibrancy readability
        // Light mode: use theme text colors (dark on light) for visible badges
        // Dark mode: use semi-transparent overlays that work with vibrancy
        let badge_bg = if is_light_mode {
            rgba(0x0000000Cu32) // black at ~5% → visible gray on light vibrancy
        } else {
            rgba((ui_border << 8) | 0x60) // border at 37% on dark
        };
        let badge_text = if is_light_mode {
            rgb(text_secondary) // dark gray for strong contrast
        } else {
            rgb(text_muted)
        };
        let badge_border = if is_light_mode {
            rgba(0x00000018u32) // black at ~9% border
        } else {
            rgba((ui_border << 8) | 0x40)
        };
        // Accent badge colors — yellow/gold design accent is unreadable on light backgrounds,
        // so use the theme's selected accent (typically blue) for light mode instead
        let light_accent = self.theme.colors.accent.selected;
        let accent_badge_bg = if is_light_mode {
            rgba((light_accent << 8) | 0x14) // theme accent at ~8%
        } else {
            rgba((colors.accent << 8) | 0x30) // design accent at ~19%
        };
        let accent_badge_border = if is_light_mode {
            rgba((light_accent << 8) | 0x30) // theme accent at ~19%
        } else {
            rgba((colors.accent << 8) | 0x50) // design accent at ~31%
        };
        let accent_badge_text = if is_light_mode {
            rgb(light_accent) // theme accent (blue) for light mode
        } else {
            rgb(colors.accent) // design accent (yellow/gold) fine on dark
        };

        // Get shortcut display string for the selected item (if any)
        // Check BOTH config.ts commands AND shortcut overrides file
        let shortcut_display: Option<String> = selected_result.as_ref().and_then(|result| {
            Self::get_command_id_for_result(result).and_then(|command_id| {
                // First check config.ts commands
                if let Some(hotkey) = self.config.get_command_shortcut(&command_id) {
                    return Some(hotkey.to_display_string());
                }
                // Then check shortcut overrides file (where ShortcutRecorder saves)
                // Uses cached version to avoid file I/O on every render
                let overrides = crate::shortcuts::get_cached_shortcut_overrides();
                if let Some(shortcut) = overrides.get(&command_id) {
                    return Some(shortcut.to_string());
                }
                None
            })
        });

        // Get opacity for vibrancy support from theme
        let opacity = self.theme.get_opacity();

        // Preview panel container with left border separator
        // Uses theme.opacity.preview to control background opacity (default 0 = transparent)
        let preview_alpha = (opacity.preview * 255.0) as u32;
        let mut panel = div()
            .w_full()
            .h_full()
            .when(preview_alpha > 0, |d| {
                d.bg(rgba((bg_main << 8) | preview_alpha))
            })
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x80))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(font_family);

        // P4: Compute match indices lazily for visible preview (only one result at a time)
        let computed_filter = self.computed_filter_text.clone();

        match selected_result {
            Some(ref result) => {
                // P4: Lazy match indices computation for preview panel
                let match_start = std::time::Instant::now();
                let match_indices =
                    scripts::compute_match_indices_for_result(result, &computed_filter);
                let match_elapsed = match_start.elapsed();
                if match_elapsed.as_micros() > 500 {
                    logging::log(
                        "FILTER_PERF",
                        &format!(
                            "[PREVIEW] match_indices for '{}' took {:.2}ms (filter='{}')",
                            result.name(),
                            match_elapsed.as_secs_f64() * 1000.0,
                            filter_for_log
                        ),
                    );
                }

                match result {
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
                                            .text_size(px(section_label_font_size))
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
                                    .line_height(px(body_text_line_height))
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
                                .text_size(px(section_label_font_size))
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

                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
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
                                        .text_color(rgba((text_muted << 8) | 0x99))
                                        .child("scriptlet: "),
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
                                    .line_height(px(body_text_line_height))
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
                                            .text_size(px(section_label_font_size))
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
                                .bg(rgba((ui_border << 8) | if is_light_mode { 0x30 } else { 0x60 }))
                                .my(px(spacing.padding_sm)),
                        );

                        // Content preview header
                        panel = panel.child(
                            div()
                                .text_size(px(section_label_font_size))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgba((text_muted << 8) | 0xCC))
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
                            .bg(rgba((bg_search_box << 8) | 0x80))
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

                    scripts::SearchResult::BuiltIn(builtin_match) => {
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
                                            .text_size(px(section_label_font_size))
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

                        // Description
                        panel = panel.child(
                            div()
                                .text_sm()
                                .line_height(px(body_text_line_height))
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
                                            .text_size(px(section_label_font_size))
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
                                        .text_size(px(section_label_font_size))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("PATH"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .line_height(px(body_text_line_height))
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
                                            .text_size(px(section_label_font_size))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgba((text_muted << 8) | 0xCC))
                                            .pb(px(spacing.padding_xs))
                                            .child("BUNDLE ID"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .line_height(px(body_text_line_height))
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
                                        .text_size(px(section_label_font_size))
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

                    scripts::SearchResult::Window(window_match) => {
                        let window = &window_match.window;

                        // Window title header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(window.title.clone()),
                        );

                        // App name
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_size(px(section_label_font_size))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("APPLICATION"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(window.app.clone()),
                                ),
                        );

                        // Bounds
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_size(px(section_label_font_size))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("POSITION & SIZE"),
                                )
                                .child(div().text_sm().text_color(rgb(text_secondary)).child(
                                    format!(
                                        "{}×{} at ({}, {})",
                                        window.bounds.width,
                                        window.bounds.height,
                                        window.bounds.x,
                                        window.bounds.y
                                    ),
                                )),
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
                                        .text_size(px(section_label_font_size))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgba((text_muted << 8) | 0xCC))
                                        .pb(px(spacing.padding_xs))
                                        .child("TYPE"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Window"),
                                ),
                        );
                    }

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
                                        .text_size(px(section_label_font_size))
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
                                            .text_size(px(section_label_font_size))
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
                                        .text_size(px(section_label_font_size))
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
                                        .text_size(px(section_label_font_size))
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

                }
            }
            None => {
                logging::log("UI", "Preview panel: No selection");
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child(
                            if self.filter_text.is_empty()
                                && self.scripts.is_empty()
                                && self.scriptlets.is_empty()
                            {
                                "No scripts or snippets found"
                            } else if !self.filter_text.is_empty() {
                                "No matching scripts"
                            } else {
                                "Select a script to preview"
                            },
                        ),
                );
            }
        }

        panel
    }
}
