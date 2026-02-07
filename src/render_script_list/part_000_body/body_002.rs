        let mut main_div = div()
            .flex()
            .flex_col()
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h_full()
            .text_color(rgb(text_primary))
            .font_family(font_family)
            .key_context("script_list")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header: Search Input + Run + Actions + Logo
            // Use shared header layout constants for consistency with all prompts
            .child({
                // Use shared header constants for default design, design tokens for others
                let header_padding_x = if is_default_design {
                    HEADER_PADDING_X
                } else {
                    design_spacing.padding_lg
                };
                let header_padding_y = if is_default_design {
                    HEADER_PADDING_Y
                } else {
                    design_spacing.padding_sm
                };
                let header_gap = if is_default_design {
                    HEADER_GAP
                } else {
                    design_spacing.gap_md
                };
                let text_muted = color_resolver.empty_text_color();
                let _text_dimmed = color_resolver.dimmed_text_color();
                let accent_color = color_resolver.primary_accent();
                let search_box_bg = color_resolver.secondary_background_color();
                let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);

                div()
                    .w_full()
                    .px(px(header_padding_x))
                    .py(px(header_padding_y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(header_gap))
                    // Search input with cursor and selection support
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(input_height))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(typography_resolver.font_size_xl())))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    // Position indicator + result count - shown only when filtering
                    .when(!self.filter_text.is_empty() && item_count > 0, |el| {
                        // Compute 1-based position of the selected item among selectable items
                        let selected_position = {
                            let mut pos = 0u16;
                            for (i, item) in grouped_items.iter().enumerate() {
                                if matches!(item, GroupedListItem::Item(_)) {
                                    pos += 1;
                                }
                                if i == self.selected_index {
                                    break;
                                }
                            }
                            pos
                        };
                        let total_selectable = flat_results.len();

                        // Count results by type for a breakdown display
                        let mut scripts = 0u16;
                        let mut snippets = 0u16;
                        let mut commands = 0u16;
                        let mut apps = 0u16;
                        let mut others = 0u16;
                        for r in flat_results.iter() {
                            match r {
                                crate::scripts::SearchResult::Script(_) => scripts += 1,
                                crate::scripts::SearchResult::Scriptlet(_) => snippets += 1,
                                crate::scripts::SearchResult::BuiltIn(_) => commands += 1,
                                crate::scripts::SearchResult::App(_) => apps += 1,
                                _ => others += 1,
                            }
                        }
                        // Build a compact breakdown string (e.g., "3 scripts · 2 snippets")
                        let mut parts: Vec<String> = Vec::new();
                        if scripts > 0 {
                            parts.push(format!(
                                "{} {}",
                                scripts,
                                if scripts == 1 { "script" } else { "scripts" }
                            ));
                        }
                        if snippets > 0 {
                            parts.push(format!(
                                "{} {}",
                                snippets,
                                if snippets == 1 { "snippet" } else { "snippets" }
                            ));
                        }
                        if commands > 0 {
                            parts.push(format!(
                                "{} {}",
                                commands,
                                if commands == 1 { "command" } else { "commands" }
                            ));
                        }
                        if apps > 0 {
                            parts.push(format!(
                                "{} {}",
                                apps,
                                if apps == 1 { "app" } else { "apps" }
                            ));
                        }
                        if others > 0 {
                            parts.push(format!("{} other", others));
                        }
                        // If all results are the same type, just show total count
                        let count_text = if parts.len() <= 1 {
                            if total_selectable == 1 {
                                "1 result".to_string()
                            } else {
                                format!("{} results", total_selectable)
                            }
                        } else {
                            // Show at most 3 categories to avoid overflow
                            parts.truncate(3);
                            parts.join(" · ")
                        };

                        el
                            // Position indicator: "3 / 45" - helps orient in long result lists
                            .child(
                                div()
                                    .text_xs()
                                    .font_family(FONT_MONO)
                                    .text_color(rgba((text_muted << 8) | ALPHA_READABLE))
                                    .flex_shrink_0()
                                    .whitespace_nowrap()
                                    .child(format!("{} / {}", selected_position, total_selectable)),
                            )
                            // Result count breakdown
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .flex_shrink_0()
                                    .whitespace_nowrap()
                                    .child(count_text),
                            )
                    })
                    // Total item count in grouped view - subtle hint showing library size
                    .when(
                        self.filter_text.is_empty() && !flat_results.is_empty(),
                        |el| {
                            // Count items by type for a compact summary
                            let mut scripts_count = 0u16;
                            let mut snippets_count = 0u16;
                            let mut others_count = 0u16;
                            for r in flat_results.iter() {
                                match r {
                                    crate::scripts::SearchResult::Script(_) => scripts_count += 1,
                                    crate::scripts::SearchResult::Scriptlet(_) => {
                                        snippets_count += 1
                                    }
                                    _ => others_count += 1,
                                }
                            }
                            // Build compact summary (e.g., "42 scripts · 15 snippets")
                            let mut parts: Vec<String> = Vec::new();
                            if scripts_count > 0 {
                                parts.push(format!(
                                    "{} {}",
                                    scripts_count,
                                    if scripts_count == 1 {
                                        "script"
                                    } else {
                                        "scripts"
                                    }
                                ));
                            }
                            if snippets_count > 0 {
                                parts.push(format!(
                                    "{} {}",
                                    snippets_count,
                                    if snippets_count == 1 {
                                        "snippet"
                                    } else {
                                        "snippets"
                                    }
                                ));
                            }
                            if others_count > 0 {
                                parts.push(format!("{} other", others_count));
                            }
                            let summary = if parts.len() <= 1 {
                                let total = flat_results.len();
                                if total == 1 {
                                    "1 item".to_string()
                                } else {
                                    format!("{} items", total)
                                }
                            } else {
                                parts.truncate(3);
                                parts.join(" · ")
                            };

                            el.child(
                                div()
                                    .text_xs()
                                    .text_color(rgba((text_muted << 8) | ALPHA_COUNT_HINT))
                                    .flex_shrink_0()
                                    .whitespace_nowrap()
                                    .child(summary),
                            )
                        },
                    )
                    // "Ask AI [Tab]" keyboard hint - styled as non-clickable to match behavior
                    .child({
                        let hint_bg = (accent_color << 8) | ALPHA_HOVER_ACCENT;
                        let tab_bg = (search_box_bg << 8) | ALPHA_TAB_BADGE_BG;
                        div()
                            .id("ask-ai-button")
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(ASK_AI_BUTTON_GAP))
                            .px(px(ASK_AI_BUTTON_PADDING_X))
                            .py(px(ASK_AI_BUTTON_PADDING_Y))
                            .rounded(px(ASK_AI_BUTTON_RADIUS))
                            .bg(rgba(hint_bg))
                            .cursor_default()
                            // "Ask AI" text - YELLOW (accent)
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(accent_color))
                                    .child("Ask AI"),
                            )
                            // "Tab" badge - grey background at ALPHA_TAB_BADGE_BG opacity (no border)
                            .child(
                                div()
                                    .px(px(TAB_BADGE_PADDING_X))
                                    .py(px(TAB_BADGE_PADDING_Y))
                                    .rounded(px(TAB_BADGE_RADIUS))
                                    .bg(rgba(tab_bg))
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child("Tab"),
                            )
                    })
            })
            // Divider between header and list content
            // Use unified resolver for border color and spacing
            .child({
                let divider_margin = if is_default_design {
                    DIVIDER_MARGIN_DEFAULT
                } else {
                    spacing_resolver.margin_lg()
                };
                let border_color = color_resolver.border_color();
                let border_width = if is_default_design {
                    DIVIDER_BORDER_WIDTH_DEFAULT
                } else {
                    design_visual.border_thin
                };

                div()
                    .mx(px(divider_margin))
                    .h(px(border_width))
                    .bg(rgba((border_color << 8) | ALPHA_DIVIDER))
            });

        // Main content area - 50/50 split: List on left, Preview on right
        main_div = main_div
            // Uses min_h(px(0.)) to prevent flex children from overflowing
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.)) // Critical: allows flex container to shrink properly
                    .w_full()
                    .overflow_hidden()
                    // Left side: Script list (50% width) - uses uniform_list for auto-scrolling
                    .child(
                        div()
                            .w_1_2() // 50% width
                            .h_full() // Take full height
                            .min_h(px(0.)) // Allow shrinking
                            .child(list_element),
                    )
                    // Right side: Preview panel (50% width) with actions overlay
                    // Preview ALWAYS renders, actions panel overlays on top when visible
                    .child({
                        let preview_start = std::time::Instant::now();
                        let preview_panel = self.render_preview_panel(cx);
                        let preview_elapsed = preview_start.elapsed();
                        // Log preview panel render time only when state changed (reduces cursor-blink spam)
                        if state_changed {
                            logging::log(
                                "PREVIEW_PERF",
                                &format!(
                                    "[PREVIEW_PANEL_DONE] filter='{}' took {:.2}ms",
                                    filter_for_log,
                                    preview_elapsed.as_secs_f64() * 1000.0
                                ),
                            );
                        }
                        div()
                            .relative() // Enable absolute positioning for overlay
                            .w_1_2() // 50% width
                            .h_full() // Take full height
                            .min_h(px(0.)) // Allow shrinking
                            .overflow_hidden()
                            // Preview panel ALWAYS renders
                            // NOTE: Actions dialog is now rendered in a separate popup window
                            // (see actions/window.rs) - no inline overlay needed here
                            .child(preview_panel)
                    }),
            );

        // Footer: Logo left | Run Script ↵ | divider | Actions ⌘K right
        // Raycast-style footer with Script Kit branding using reusable PromptFooter component
        // Note: footer colors extracted earlier to avoid borrow conflict with render_preview_panel
        main_div = main_div.child({
            let handle_run = cx.entity().downgrade();
            let handle_actions = cx.entity().downgrade();

            let footer_colors = PromptFooterColors {
                accent: footer_accent,
                text_muted: footer_text_muted,
                border: footer_border,
                background: footer_background,
                is_light_mode: !self.theme.is_dark_mode(),
            };

            // Get the selected result for primary label and type indicator
            let footer_selected =
                grouped_items
                    .get(self.selected_index)
                    .and_then(|item| match item {
                        GroupedListItem::Item(idx) => flat_results.get(*idx),
                        GroupedListItem::SectionHeader(..) => None,
                    });
            let primary_label = footer_selected
                .map(|result| result.get_default_action_text())
                .unwrap_or("Run");
            let type_label = footer_selected
                .map(|result| result.type_label())
                .unwrap_or("");

            // Build footer config with type indicator and optional opacity info
            let mut footer_config = PromptFooterConfig::default().primary_label(primary_label);

            let window_tweaker_enabled = std::env::var("SCRIPT_KIT_WINDOW_TWEAKER")
                .map(|v| v == "1")
                .unwrap_or(false);
            if window_tweaker_enabled && !self.theme.is_dark_mode() {
                let opacity_percent = (self.theme.get_opacity().main * 100.0).round() as i32;
                let material = platform::get_current_material_name();
                let appearance = platform::get_current_appearance_name();
                footer_config = footer_config.info_label(format!(
                    "{}% | {} | {} | ⌘-/+ ⌘M ⌘⇧A",
                    opacity_percent, material, appearance
                ));
            } else if !type_label.is_empty() {
                footer_config = footer_config.info_label(type_label);
            }
            footer_config = footer_config.show_secondary(self.has_actions());

            PromptFooter::new(footer_config, footer_colors)
                .on_primary_click(Box::new(move |_, _window, cx| {
                    if let Some(app) = handle_run.upgrade() {
                        app.update(cx, |this, cx| {
                            this.execute_selected(cx);
                        });
                    }
                }))
                .on_secondary_click(Box::new(move |_, window, cx| {
                    if let Some(app) = handle_actions.upgrade() {
                        app.update(cx, |this, cx| {
                            this.toggle_actions(cx, window);
                        });
                    }
                }))
        });

        if let Some(panel) = log_panel {
            main_div = main_div.child(panel);
        }

        // Note: Toast notifications are now handled by gpui-component's NotificationList
        // via the Root wrapper. Toasts are flushed in render() via flush_pending_toasts().

        // Note: HUD overlay is added at the top-level render() method for all views

        // Log total render_script_list time and update tracking state (only if state changed)
        if state_changed {
            let total_elapsed = render_list_start.elapsed();
            logging::log(
                "RENDER_PERF",
                &format!(
                    "[RENDER_SCRIPT_LIST_END] filter='{}' total={:.2}ms",
                    filter_for_log,
                    total_elapsed.as_secs_f64() * 1000.0
                ),
            );
            // Deferred state update: update after all logging (including preview panel) is done
            self.last_render_log_filter = self.filter_text.clone();
            self.last_render_log_selection = self.selected_index;
            self.last_render_log_item_count = item_count_for_log;
        }

        main_div.into_any_element()
