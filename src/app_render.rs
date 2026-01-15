impl ScriptListApp {
    /// Read the first N lines of a script file for preview
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
        let _preview_start = std::time::Instant::now();
        let filter_for_log = self.filter_text.clone();

        // Log every preview panel render call
        logging::log(
            "PREVIEW_PERF",
            &format!(
                "[PREVIEW_START] filter='{}' selected_idx={}",
                filter_for_log, self.selected_index
            ),
        );

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
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = colors.text_primary;
        let text_muted = colors.text_muted;
        let text_secondary = colors.text_secondary;
        let bg_search_box = colors.background_tertiary;
        let border_radius = visual.radius_md;
        let font_family = typography.font_family;

        // Get shortcut display string for the selected item (if any)
        // Check BOTH config.ts commands AND shortcut overrides file
        let shortcut_display: Option<String> = selected_result.as_ref().and_then(|result| {
            Self::get_command_id_for_result(result).and_then(|command_id| {
                // First check config.ts commands
                if let Some(hotkey) = self.config.get_command_shortcut(&command_id) {
                    return Some(hotkey.to_display_string());
                }
                // Then check shortcut overrides file (where ShortcutRecorder saves)
                if let Ok(overrides) = crate::shortcuts::load_shortcut_overrides() {
                    if let Some(shortcut) = overrides.get(&command_id) {
                        return Some(shortcut.to_string());
                    }
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

                        // Script name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(format!("{}.{}", script.name, script.extension)),
                        );

                        // Keyboard shortcut: prefer script metadata shortcut, fall back to config-based
                        let effective_shortcut =
                            script.shortcut.clone().or_else(|| shortcut_display.clone());
                        if let Some(shortcut_str) = effective_shortcut {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keyboard Shortcut"),
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
                                                    .text_color(rgb(colors.accent))
                                                    .child(shortcut_str),
                                            ),
                                    ),
                            );
                        }

                        // Description (if present)
                        if let Some(desc) = &script.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
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

                        // Code preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Code Preview"),
                        );

                        // Use cached syntax-highlighted lines (avoids file I/O and highlighting on every render)
                        let script_path = script.path.to_string_lossy().to_string();
                        let lang = script.extension.clone();
                        let cache_start = std::time::Instant::now();
                        let lines = self
                            .get_or_update_preview_cache(&script_path, &lang)
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

                        // Scriptlet name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(scriptlet.name.clone()),
                        );

                        // Description (if present)
                        if let Some(desc) = &scriptlet.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
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
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keyboard Shortcut"),
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
                                                    .text_color(rgb(colors.accent))
                                                    .child(shortcut),
                                            ),
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

                        // Content preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Content Preview"),
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
                            let highlighted = highlight_code_lines(&code_preview, lang);
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

                        // Built-in name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(builtin.name.clone()),
                        );

                        // Keyboard shortcut (if assigned via config.commands)
                        if let Some(ref shortcut_str) = shortcut_display {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keyboard Shortcut"),
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
                                                    .text_color(rgb(colors.accent))
                                                    .child(shortcut_str.clone()),
                                            ),
                                    ),
                            );
                        }

                        // Description
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Description"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(builtin.description.clone()),
                                ),
                        );

                        // Keywords
                        if !builtin.keywords.is_empty() {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keywords"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(builtin.keywords.join(", ")),
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

                        // Feature type indicator
                        let feature_type: String = match &builtin.feature {
                            builtins::BuiltInFeature::ClipboardHistory => {
                                "Clipboard History Manager".to_string()
                            }
                            builtins::BuiltInFeature::AppLauncher => {
                                "Application Launcher".to_string()
                            }
                            builtins::BuiltInFeature::App(name) => name.clone(),
                            builtins::BuiltInFeature::WindowSwitcher => {
                                "Window Manager".to_string()
                            }
                            builtins::BuiltInFeature::DesignGallery => "Design Gallery".to_string(),
                            builtins::BuiltInFeature::AiChat => "AI Assistant".to_string(),
                            builtins::BuiltInFeature::Notes => "Notes & Scratchpad".to_string(),
                            builtins::BuiltInFeature::MenuBarAction(_) => {
                                "Menu Bar Action".to_string()
                            }
                            builtins::BuiltInFeature::SystemAction(_) => {
                                "System Action".to_string()
                            }
                            builtins::BuiltInFeature::NotesCommand(_) => {
                                "Notes Command".to_string()
                            }
                            builtins::BuiltInFeature::AiCommand(_) => "AI Command".to_string(),
                            builtins::BuiltInFeature::ScriptCommand(_) => {
                                "Script Creation".to_string()
                            }
                            builtins::BuiltInFeature::PermissionCommand(_) => {
                                "Permission Management".to_string()
                            }
                            builtins::BuiltInFeature::FrecencyCommand(_) => {
                                "Suggested Items".to_string()
                            }
                            builtins::BuiltInFeature::UtilityCommand(_) => {
                                "Quick Utility".to_string()
                            }
                            builtins::BuiltInFeature::SettingsCommand(_) => "Settings".to_string(),
                            builtins::BuiltInFeature::FileSearch => "File Browser".to_string(),
                        };
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Feature Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(feature_type),
                                ),
                        );
                    }
                    scripts::SearchResult::App(app_match) => {
                        let app = &app_match.app;

                        // App name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(app.name.clone()),
                        );

                        // Keyboard shortcut (if assigned via config.commands)
                        if let Some(ref shortcut_str) = shortcut_display {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keyboard Shortcut"),
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
                                                    .text_color(rgb(colors.accent))
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
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Path"),
                                )
                                .child(
                                    div()
                                        .text_sm()
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
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Bundle ID"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Application"),
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Position & Size"),
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Backend"),
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
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Kit"),
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
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
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
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

    /// Get the ScriptInfo for the currently focused/selected script
    fn get_focused_script_info(&mut self) -> Option<ScriptInfo> {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the minimum score threshold for suggested items
        let min_score = self.config.get_suggested().min_score;

        // Get the result index from the grouped item
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx) {
                // Compute frecency path for each result type (same logic as app_impl.rs)
                let frecency_path: Option<String> = match result {
                    scripts::SearchResult::Script(m) => {
                        Some(m.script.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::Scriptlet(m) => {
                        Some(format!("scriptlet:{}", m.scriptlet.name))
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Check if excluded from frecency tracking
                        let excluded = &self.config.get_suggested().excluded_commands;
                        if m.entry.should_exclude_from_frecency(excluded) {
                            None
                        } else {
                            Some(format!("builtin:{}", m.entry.name))
                        }
                    }
                    scripts::SearchResult::App(m) => Some(m.app.path.to_string_lossy().to_string()),
                    scripts::SearchResult::Window(m) => {
                        Some(format!("window:{}:{}", m.window.app, m.window.title))
                    }
                    scripts::SearchResult::Agent(m) => {
                        Some(format!("agent:{}", m.agent.path.to_string_lossy()))
                    }
                    scripts::SearchResult::Fallback(_) => None, // Fallbacks don't track frecency
                };

                // Check if this item is "suggested" (has frecency data above min_score)
                let is_suggested = frecency_path
                    .as_ref()
                    .map(|path| self.frecency_store.get_score(path) >= min_score)
                    .unwrap_or(false);

                match result {
                    scripts::SearchResult::Script(m) => Some(
                        ScriptInfo::with_shortcut_and_alias(
                            &m.script.name,
                            m.script.path.to_string_lossy(),
                            m.script.shortcut.clone(),
                            m.script.alias.clone(),
                        )
                        .with_frecency(is_suggested, frecency_path),
                    ),
                    scripts::SearchResult::Scriptlet(m) => {
                        // Scriptlets use the markdown file path for edit/reveal actions
                        // Extract the path without anchor for file operations
                        let markdown_path = m
                            .scriptlet
                            .file_path
                            .as_ref()
                            .map(|p| p.split('#').next().unwrap_or(p).to_string())
                            .unwrap_or_else(|| format!("scriptlet:{}", &m.scriptlet.name));
                        Some(
                            ScriptInfo::scriptlet(
                                &m.scriptlet.name,
                                markdown_path,
                                m.scriptlet.shortcut.clone(),
                                m.scriptlet.alias.clone(),
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Built-ins use their id as identifier
                        // is_script=false: no editable file, hide "Edit Script" etc.
                        // Look up shortcut and alias from overrides for dynamic action menu
                        let command_id = format!("builtin/{}", &m.entry.id);
                        let shortcut = crate::shortcuts::load_shortcut_overrides()
                            .ok()
                            .and_then(|o| o.get(&command_id).map(|s| s.to_string()));
                        let alias = crate::aliases::load_alias_overrides()
                            .ok()
                            .and_then(|o| o.get(&command_id).cloned());
                        Some(
                            ScriptInfo::with_all(
                                &m.entry.name,
                                format!("builtin:{}", &m.entry.id),
                                false,
                                "Run",
                                shortcut,
                                alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::App(m) => {
                        // Apps use their path as identifier
                        // is_script=false: apps aren't editable scripts
                        // Look up shortcut and alias from overrides for dynamic action menu
                        let command_id = if let Some(ref bundle_id) = m.app.bundle_id {
                            format!("app/{}", bundle_id)
                        } else {
                            format!("app/{}", m.app.name.to_lowercase().replace(' ', "-"))
                        };
                        let shortcut = crate::shortcuts::load_shortcut_overrides()
                            .ok()
                            .and_then(|o| o.get(&command_id).map(|s| s.to_string()));
                        let alias = crate::aliases::load_alias_overrides()
                            .ok()
                            .and_then(|o| o.get(&command_id).cloned());
                        Some(
                            ScriptInfo::with_all(
                                &m.app.name,
                                m.app.path.to_string_lossy().to_string(),
                                false,
                                "Launch",
                                shortcut,
                                alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Window(m) => {
                        // Windows use their id as identifier
                        // is_script=false: windows aren't editable scripts
                        Some(
                            ScriptInfo::with_action_verb(
                                &m.window.title,
                                format!("window:{}", m.window.id),
                                false,
                                "Switch to",
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Agent(m) => {
                        // Agents use their path as identifier
                        Some(
                            ScriptInfo::new(
                                &m.agent.name,
                                format!("agent:{}", m.agent.path.to_string_lossy()),
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Fallback(m) => {
                        // Fallbacks use their name as identifier
                        // is_script depends on whether it's a built-in fallback or script-based
                        // Fallbacks don't track frecency, so is_suggested is always false
                        Some(ScriptInfo::with_action_verb(
                            m.fallback.name(),
                            format!("fallback:{}", m.fallback.name()),
                            !m.fallback.is_builtin(),
                            "Run",
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the full scriptlet with actions for the currently focused item
    ///
    /// This re-parses the markdown file to get the scriptlet's H3 actions
    /// and shared actions from the companion .actions.md file.
    /// Returns None if the focused item is not a scriptlet.
    pub fn get_focused_scriptlet_with_actions(&mut self) -> Option<crate::scriptlets::Scriptlet> {
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(scripts::SearchResult::Scriptlet(m)) = flat_results.get(idx) {
                // Get the file path from the UI scriptlet type
                let file_path = m.scriptlet.file_path.clone()?;
                let scriptlet_command = m.scriptlet.command.clone()?;

                // Extract just the file path (before #anchor)
                let file_only = file_path.split('#').next().unwrap_or(&file_path);

                // Read and parse the markdown file to get full scriptlet with actions
                if let Ok(content) = std::fs::read_to_string(file_only) {
                    let parsed_scriptlets =
                        crate::scriptlets::parse_markdown_as_scriptlets(&content, Some(file_only));

                    // Find the matching scriptlet by command
                    return parsed_scriptlets
                        .into_iter()
                        .find(|s| s.command == scriptlet_command);
                }
            }
        }

        None
    }

    fn render_actions_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for actions dialog
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC closes window from ActionsDialog too)
                // ActionsDialog has no other key handling, so we just call the global handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // Simple actions dialog stub with design tokens
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .rounded(px(design_visual.radius_lg))
            .p(px(design_spacing.padding_xl))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(div().text_lg().child("Actions (Cmd+K)"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(design_colors.text_muted))
                    .mt(px(design_spacing.margin_md))
                    .child("• Create script\n• Edit script\n• Reload\n• Settings\n• Quit"),
            )
            .child(
                div()
                    .mt(px(design_spacing.margin_lg))
                    .text_xs()
                    .text_color(rgb(design_colors.text_dimmed))
                    .child("Press Esc to close"),
            )
            .into_any_element()
    }
}

/// Helper function to render a group header style item with actual visual styling
fn render_group_header_item(
    ix: usize,
    is_selected: bool,
    style: &designs::group_header_variations::GroupHeaderStyle,
    spacing: &designs::DesignSpacing,
    typography: &designs::DesignTypography,
    visual: &designs::DesignVisual,
    colors: &designs::DesignColors,
) -> AnyElement {
    use designs::group_header_variations::GroupHeaderStyle;

    let name_owned = style.name().to_string();
    let desc_owned = style.description().to_string();

    let mut item_div = div()
        .id(ElementId::NamedInteger("gallery-header".into(), ix as u64))
        .w_full()
        .h(px(LIST_ITEM_HEIGHT))
        .px(px(spacing.padding_lg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.gap_md));

    if is_selected {
        // Use low-opacity white for vibrancy support (see VIBRANCY.md)
        item_div = item_div.bg(rgba((colors.background_selected << 8) | 0x0f)); // ~6% opacity
    }

    // Create the preview element based on the style
    let preview = match style {
        // Text Only styles - vary font weight and style
        GroupHeaderStyle::UppercaseLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::UppercaseCenter => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::SmallCapsLeft => {
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(colors.text_secondary))
                .child("MAIN") // Would use font-variant: small-caps if available
        }
        GroupHeaderStyle::BoldLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::LightLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::LIGHT)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),
        GroupHeaderStyle::MonospaceLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_family(typography.font_family_mono)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),

        // With Lines styles
        GroupHeaderStyle::LineLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().w(px(24.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::LineRight => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBothSides => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBelow => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineAbove => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DoubleLine => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap(px(1.0))
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border))),

        // With Background styles
        GroupHeaderStyle::PillBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(10.0))
                    .bg(rgba((colors.accent << 8) | 0x30))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::FullWidthBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.accent << 8) | 0x20))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::SubtleBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x90))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::GradientFade => {
            // Simulated with opacity fade
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .child(
                    div()
                        .px(px(16.0))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_secondary))
                        .child("~  MAIN  ~"),
                )
        }
        GroupHeaderStyle::BorderedBox => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),

        // Minimal styles
        GroupHeaderStyle::DotPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(4.0))
                    .h(px(4.0))
                    .rounded(px(2.0))
                    .bg(rgb(colors.text_muted)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DashPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("- MAIN"),
        GroupHeaderStyle::BulletPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(6.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(rgb(colors.accent)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::ArrowPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{25B8} MAIN"),
        GroupHeaderStyle::ChevronPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{203A} MAIN"),
        GroupHeaderStyle::Dimmed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .opacity(0.5)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),

        // Decorative styles
        GroupHeaderStyle::Bracketed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("[MAIN]"),
        GroupHeaderStyle::Quoted => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\"MAIN\""),
        GroupHeaderStyle::Tagged => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(6.0))
                    .py(px(1.0))
                    .bg(rgba((colors.accent << 8) | 0x40))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::Numbered => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(colors.accent))
                    .child("01."),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::IconPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .bg(rgb(colors.accent))
                    .rounded(px(1.0)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
    };

    item_div
        // Preview element
        .child(preview)
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child(name_owned),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text_muted))
                        .child(desc_owned),
                ),
        )
        .into_any_element()
}
