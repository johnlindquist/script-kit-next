// Script list render method - extracted from app_render.rs
// This file is included via include!() macro in main.rs

// --- merged from part_000.rs ---
fn app_shell_footer_colors(theme: &crate::theme::Theme) -> PromptFooterColors {
    PromptFooterColors::from_theme(theme)
}

fn script_list_footer_primary_label() -> &'static str {
    "Run"
}

fn script_list_footer_info_label(
    window_tweaker_enabled: bool,
    is_dark_mode: bool,
    opacity_percent: i32,
    material: &str,
    appearance: &str,
) -> Option<String> {
    if window_tweaker_enabled && !is_dark_mode {
        Some(format!(
            "{}% | {} | {} | ⌘-/+ ⌘M ⌘⇧A",
            opacity_percent, material, appearance
        ))
    } else {
        None
    }
}

fn inline_calc_list_item_title(formatted_result: &str) -> String {
    format!("= {}", formatted_result)
}

fn inline_calc_list_copy_hint() -> &'static str {
    "↵ Copy"
}

fn render_inline_calc_list_item(
    calculator: &crate::calculator::CalculatorInlineResult,
    is_selected: bool,
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
) -> AnyElement {
    let tokens = get_tokens(design_variant);
    let spacing = tokens.spacing();
    let typography = tokens.typography();

    let result_title = inline_calc_list_item_title(&calculator.formatted);
    let result_text_color = if is_selected {
        theme.colors.accent.selected
    } else {
        theme.colors.text.primary
    };
    let hint_alpha = if is_selected { 0xD9 } else { 0x8C };

    div()
        .w_full()
        .h_full()
        .px(px(spacing.item_padding_x))
        .py(px(spacing.padding_xs))
        .when(is_selected, |div| {
            let selected_overlay_alpha = ((theme.get_opacity().selected.clamp(0.0, 1.0) * 255.0)
                .round() as u32)
                .max(0x2E);
            div.bg(rgba(
                (theme.colors.accent.selected_subtle << 8) | selected_overlay_alpha,
            ))
        })
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(spacing.gap_md))
        .child(
            div()
                .flex_1()
                .overflow_x_hidden()
                .text_size(px(typography.font_size_lg))
                .font_weight(typography.font_weight_semibold)
                .text_color(rgb(result_text_color))
                .child(result_title),
        )
        .child(
            div()
                .text_size(px(typography.font_size_xs))
                .text_color(rgba((theme.colors.text.muted << 8) | hint_alpha))
                .child(inline_calc_list_copy_hint()),
        )
        .into_any_element()
}

impl ScriptListApp {
    fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let render_list_start = std::time::Instant::now();
        let filter_for_log = self.filter_text.clone();

        // Get grouped or flat results based on filter state (cached) - MUST come first
        // to avoid borrow conflicts with theme access below
        // When filter is empty, use frecency-grouped results with RECENT/MAIN sections
        // When filtering, use flat fuzzy search results
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let get_results_elapsed = render_list_start.elapsed();

        // Deduplicate render logs: only log when meaningful state changes (not cursor blink)
        // This reduces log spam from ~2 logs/sec (cursor blink) to only on actual changes
        let state_changed = self.filter_text != self.last_render_log_filter
            || self.selected_index != self.last_render_log_selection
            || grouped_items.len() != self.last_render_log_item_count;

        // Set flag for render_preview_panel to check (called later in this render)
        self.log_this_render = state_changed;
        // Capture item count for deferred state update
        let item_count_for_log = grouped_items.len();

        if state_changed {
            logging::log(
                "RENDER_PERF",
                &format!(
                    "[RENDER_SCRIPT_LIST_START] filter='{}' computed_filter='{}' selected_idx={}",
                    filter_for_log, self.computed_filter_text, self.selected_index
                ),
            );
            logging::log(
                "RENDER_PERF",
                &format!(
                    "[RENDER_GET_RESULTS] filter='{}' items={} results={} took={:.2}ms",
                    filter_for_log,
                    grouped_items.len(),
                    flat_results.len(),
                    get_results_elapsed.as_secs_f64() * 1000.0
                ),
            );
        }

        // NOTE: Removed per-frame logging here - was causing 6 log calls per frame
        // which includes mutex locks and file I/O. Log only on cache MISS in get_grouped_results_cached.
        // Clone for use in closures and to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get design tokens for current design variant
        let tokens = get_tokens(self.current_design);
        let design_visual = tokens.visual();

        // Unified color, typography, and spacing resolution
        let color_resolver = crate::theme::ColorResolver::new(&self.theme, self.current_design);
        let typography_resolver =
            crate::theme::TypographyResolver::new(&self.theme, self.current_design);
        let spacing_resolver = crate::theme::SpacingResolver::new(self.current_design);

        // For Default design, use header constants; for others, use spacing resolver
        let is_default_design = self.current_design == DesignVariant::Default;
        let design_spacing = tokens.spacing();

        let item_count = grouped_items.len();
        let _total_len = self.scripts.len() + self.scriptlets.len();

        // ============================================================
        // RENDER IS READ-ONLY
        // ============================================================
        // NOTE: State mutations (selection validation, list sync) are now done
        // in event handlers via sync_list_state() and validate_selection_bounds(),
        // not during render. This prevents the anti-pattern of mutating state
        // during render which can cause infinite render loops and inconsistent UI.
        //
        // Event handlers that call these methods:
        // - queue_filter_compute() - after filter text changes
        // - set_filter_text_immediate() - for immediate filter updates
        // - refresh_scripts() - after script reload
        // - reset_to_script_list() - on view transitions

        // Get scroll offset AFTER updates for scrollbar
        let scroll_offset = self.main_list_state.logical_scroll_top().item_ix;

        // ============================================================
        // IMMUTABLE BORROWS BLOCK - extract theme values for UI building
        // ============================================================

        // Extract theme values as owned copies for UI building
        let log_panel_bg = self.theme.colors.background.log_panel;
        let log_panel_border = self.theme.colors.ui.border;
        let log_panel_success = self.theme.colors.ui.success;

        // Pre-compute scrollbar colors (Copy type) - always use theme for consistency
        let scrollbar_colors = ScrollbarColors::from_theme(&self.theme);
        // Pre-compute list item colors for closure (Copy type)
        let theme_colors = ListItemColors::from_theme(&self.theme);

        // NOTE: Removed P4 perf log - called every render frame, causing log spam

        // Build script list using uniform_list for proper virtualized scrolling
        // Use unified color resolver for consistent empty state styling
        let empty_text_color = color_resolver.empty_text_color();
        let empty_font_family = typography_resolver.primary_font();

        let list_element: AnyElement = if item_count == 0 {
            // Empty state rendering with icon and helpful messaging
            // - When filter is empty: "No scripts or snippets found" with code icon
            // - When filter has text: "No results match '...'" with search icon
            //
            // Note: This branch is rarely hit when filtering because grouping.rs now
            // appends fallbacks to the results. We only get here if there are truly
            // no results at all (including no fallbacks).
            use crate::designs::icon_variations::IconName;
            if self.filter_text.is_empty() {
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(EMPTY_STATE_GAP))
                    .font_family(empty_font_family)
                    // Large muted icon
                    .child(
                        svg()
                            .external_path(IconName::Code.external_path())
                            .size(px(EMPTY_STATE_ICON_SIZE))
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_ICON)),
                    )
                    .child(
                        div()
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_MESSAGE))
                            .text_size(px(EMPTY_STATE_MESSAGE_FONT_SIZE))
                            .font_weight(FontWeight::MEDIUM)
                            .child("No scripts or snippets found"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_HINT))
                            .child("Press ⌘N to create a new script"),
                    )
                    .into_any_element()
            } else {
                // Filtering but no results (including no fallbacks) - shouldn't normally happen
                let filter_display = if self.filter_text.chars().count() > 30 {
                    format!(
                        "{}...",
                        crate::utils::truncate_str_chars(&self.filter_text, 27)
                    )
                } else {
                    self.filter_text.clone()
                };
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(EMPTY_STATE_GAP))
                    .font_family(empty_font_family)
                    // Magnifying glass icon for search
                    .child(
                        svg()
                            .external_path(IconName::MagnifyingGlass.external_path())
                            .size(px(EMPTY_STATE_ICON_SIZE))
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_ICON)),
                    )
                    .child(
                        div()
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_MESSAGE))
                            .text_size(px(EMPTY_STATE_MESSAGE_FONT_SIZE))
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!("No results for \"{}\"", filter_display)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_HINT))
                            .child("Try a different search term or press Tab to ask AI"),
                    )
                    // Search tips: help users discover advanced search features
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba((empty_text_color << 8) | ALPHA_EMPTY_TIPS))
                            .pt(px(EMPTY_STATE_TIPS_MARGIN_TOP))
                            .child("Filters: tag:X, author:X, kit:X, is:cron/bg/watch, type:script/snippet"),
                    )
                    .into_any_element()
            }
        } else {
            // Use GPUI's list() component for variable-height items
            // Section headers render at 32px, regular items at 40px
            // This gives true visual compression for headers without the uniform_list hack

            // Clone grouped_items and flat_results for the closure
            let grouped_items_clone = grouped_items.clone();
            let flat_results_clone = flat_results.clone();

            // Calculate scrollbar parameters for variable-height items
            // Count section headers vs regular items to get true content height
            let mut header_count = 0_usize;
            let mut item_count_regular = 0_usize;
            for item in grouped_items.iter() {
                match item {
                    GroupedListItem::SectionHeader(..) => header_count += 1,
                    GroupedListItem::Item(_) => item_count_regular += 1,
                }
            }

            // Calculate true content height: headers at 32px (visual height), items at 40px
            let total_content_height = (header_count as f32 * SECTION_HEADER_HEIGHT)
                + (item_count_regular as f32 * LIST_ITEM_HEIGHT);

            // Estimated visible container height
            // Window is 500px, header is ~60px, remaining ~440px for list area
            // Use a slightly higher estimate to ensure scrollbar thumb reaches bottom
            // (underestimating visible items causes thumb to not reach bottom)
            let estimated_container_height = ESTIMATED_LIST_CONTAINER_HEIGHT;

            // Calculate visible items as a ratio of container to total content
            // This gives a more accurate thumb size for the scrollbar
            let visible_ratio = if total_content_height > 0.0 {
                (estimated_container_height / total_content_height).min(1.0)
            } else {
                1.0
            };
            let visible_items = ((item_count as f32) * visible_ratio).ceil() as usize;

            // Note: list state updates and scroll_to_selected_if_needed already done above
            // before the theme borrow section

            // Create scrollbar using pre-computed scrollbar_colors and scroll_offset
            let scrollbar =
                Scrollbar::new(item_count, visible_items, scroll_offset, scrollbar_colors)
                    .container_height(estimated_container_height)
                    .visibility_opacity(self.scrollbar_visibility.value());

            // Capture entity handle for use in the render closure
            let entity = cx.entity();

            // theme_colors was pre-computed above to avoid borrow conflicts
            let current_design = self.current_design;

            // Track filter for closure logging and highlighting
            let filter_for_closure = self.filter_text.clone();
            let filter_for_highlight = self.filter_text.clone();

            let variable_height_list =
                list(self.main_list_state.clone(), move |ix, _window, cx| {
                    let _item_render_start = std::time::Instant::now();

                    // Access entity state inside the closure
                    entity.update(cx, |this, cx| {
                        let current_selected = this.selected_index;
                        let current_hovered = this.hovered_index;
                        let current_input_mode = this.input_mode;

                        if let Some(grouped_item) = grouped_items_clone.get(ix) {
                            match grouped_item {
                                GroupedListItem::SectionHeader(label, icon) => {
                                    // Section header at 32px height (8px grid) for clear visual separation
                                    div()
                                        .id(ElementId::NamedInteger(
                                            "section-header".into(),
                                            ix as u64,
                                        ))
                                        .h(px(SECTION_HEADER_HEIGHT))
                                        .child(render_section_header(label, icon.as_deref(), theme_colors, ix == 0))
                                        .into_any_element()
                                }
                                GroupedListItem::Item(result_idx) => {
                                    // Regular item at 40px height (LIST_ITEM_HEIGHT)
                                    let is_selected = ix == current_selected;
                                    // Only show hover effect when in Mouse mode to prevent dual-highlight
                                    let is_hovered = current_hovered == Some(ix)
                                        && current_input_mode == InputMode::Mouse;

                                    // Create hover handler
                                    let hover_handler = cx.listener(
                                        move |this: &mut ScriptListApp,
                                              hovered: &bool,
                                              _window,
                                              cx| {
                                            let now = std::time::Instant::now();
                                            const HOVER_DEBOUNCE_MS: u64 = 16;

                                            if *hovered {
                                                // Mouse entered - switch to Mouse mode and set hovered_index
                                                // This re-enables hover effects after keyboard navigation
                                                this.input_mode = InputMode::Mouse;

                                                if this.hovered_index != Some(ix)
                                                    && now
                                                        .duration_since(this.last_hover_notify)
                                                        .as_millis()
                                                        >= HOVER_DEBOUNCE_MS as u128
                                                {
                                                    this.hovered_index = Some(ix);
                                                    this.last_hover_notify = now;
                                                    cx.notify();
                                                }
                                            } else if this.hovered_index == Some(ix) {
                                                // Mouse left - clear hovered_index if it was this item
                                                this.hovered_index = None;
                                                this.last_hover_notify = now;
                                                cx.notify();
                                            }
                                        },
                                    );

                                    // Create click handler with double-click support
                                    let click_handler = cx.listener(
                                        move |this: &mut ScriptListApp,
                                              event: &gpui::ClickEvent,
                                              _window,
                                              cx| {
                                            // Always select the item on any click
                                            if this.selected_index != ix {
                                                this.selected_index = ix;
                                                cx.notify();
                                            }

                                            // Check for double-click (mouse clicks only)
                                            if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                                if mouse_event.down.click_count == 2 {
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Double-click on item {}, executing",
                                                            ix
                                                        ),
                                                    );
                                                    this.execute_selected(cx);
                                                }
                                            }
                                        },
                                    );

                                    // Dispatch to design-specific item renderer
                                    // Note: Confirmation for dangerous builtins is now handled
                                    // via modal dialog, not inline overlay
                                    let design_render_start = std::time::Instant::now();
                                    let inline_calculator =
                                        this.inline_calculator_for_result_index(*result_idx);
                                    let mut item_name = "inline-calculator";
                                    let item_element = if let Some(calculator) = inline_calculator
                                    {
                                        let _legacy_calculator_renderer = render_calculator_item;
                                        render_inline_calc_list_item(
                                            calculator,
                                            is_selected,
                                            &this.theme,
                                            this.current_design,
                                        )
                                    } else if let Some(result) = flat_results_clone.get(*result_idx)
                                    {
                                        item_name = result.name();
                                        // Enable hover effects only when in Mouse mode
                                        let enable_hover = current_input_mode == InputMode::Mouse;
                                        render_design_item(
                                            current_design,
                                            result,
                                            ix,
                                            is_selected,
                                            is_hovered,
                                            theme_colors,
                                            enable_hover,
                                            &filter_for_highlight,
                                        )
                                    } else {
                                        item_name = "<missing-result>";
                                        div().h(px(LIST_ITEM_HEIGHT)).into_any_element()
                                    };
                                    let design_elapsed = design_render_start.elapsed();

                                    // Log slow items (>1ms)
                                    if design_elapsed.as_micros() > 1000 {
                                        logging::log(
                                            "FILTER_PERF",
                                            &format!(
                                                "[SLOW_ITEM] ix={} name='{}' design_render={:.2}ms filter='{}'",
                                                ix,
                                                item_name,
                                                design_elapsed.as_secs_f64() * 1000.0,
                                                filter_for_closure
                                            ),
                                        );
                                    }

                                    div()
                                        .id(ElementId::NamedInteger("script-item".into(), ix as u64))
                                        .h(px(LIST_ITEM_HEIGHT)) // Explicit 40px height (8px grid)
                                        .on_hover(hover_handler)
                                        .on_click(click_handler)
                                        .child(item_element)
                                        .into_any_element()
                                }
                            }
                        } else {
                            // Fallback for out-of-bounds index
                            div().h(px(LIST_ITEM_HEIGHT)).into_any_element()
                        }
                    })
                })
                // Enable proper scroll handling for mouse wheel/trackpad
                // ListSizingBehavior::Infer sets overflow.y = Overflow::Scroll internally
                // which is required for the list's hitbox to capture scroll wheel events
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .h_full();

            // Wrap list in a relative container with scrollbar overlay
            // CUSTOM SCROLL HANDLER: GPUI's list() component has issues measuring unmeasured items
            // (they appear as 0px height). This causes mouse scroll to fail to reach all items.
            // Solution: Intercept scroll wheel events and convert to index-based scrolling,
            // which works correctly like keyboard navigation does.
            //
            // Average item height for delta-to-index conversion:
            // Most items are LIST_ITEM_HEIGHT (40px), headers are SECTION_HEADER_HEIGHT (32px)
            // Use 44px as a reasonable average that feels natural for scrolling
            let avg_item_height = AVERAGE_ITEM_HEIGHT_FOR_SCROLL;

            // Capture item count for scroll handler logging
            let scroll_item_count = item_count;

            div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .w_full()
                .h_full()
                .on_scroll_wheel(cx.listener(
                    move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                        // Convert scroll delta to lines/items
                        // Lines: direct item count, Pixels: convert based on average item height
                        let delta_lines: f32 = match event.delta {
                            gpui::ScrollDelta::Lines(point) => point.y,
                            gpui::ScrollDelta::Pixels(point) => {
                                // Convert pixels to items using average item height
                                let pixels: f32 = point.y.into();
                                pixels / avg_item_height
                            }
                        };

                        // Accumulate smoothly for high-resolution trackpads
                        // Invert so scroll down (negative delta) moves selection down (positive)
                        this.wheel_accum += -delta_lines;

                        // Only apply integer steps when magnitude crosses 1.0
                        // This preserves smooth scrolling feel on trackpads
                        let steps = this.wheel_accum.trunc() as i32;
                        if steps != 0 {
                            // Subtract the applied steps from accumulator
                            this.wheel_accum -= steps as f32;

                            // Use the existing move_selection_by which handles section headers
                            // and properly updates scroll via scroll_to_selected_if_needed
                            this.move_selection_by(steps, cx);

                            // Log for observability
                            tracing::trace!(
                                delta = steps,
                                accum = this.wheel_accum,
                                new_index = this.selected_index,
                                total_items = scroll_item_count,
                                "Mouse wheel scroll - accumulated"
                            );
                        }
                    },
                ))
                .child(variable_height_list)
                .child(scrollbar)
                .into_any_element()
        };

        // Log panel - uses pre-extracted theme values to avoid borrow conflicts
        let log_panel = if self.show_logs {
            let logs = logging::get_last_logs(10);
            let mut log_container = div()
                .flex()
                .flex_col()
                .w_full()
                .bg(rgb(log_panel_bg))
                .border_t_1()
                .border_color(rgb(log_panel_border))
                .p(px(design_spacing.padding_md))
                .max_h(px(LOG_PANEL_MAX_HEIGHT))
                .font_family(FONT_MONO);

            for log_line in logs.iter().rev() {
                log_container = log_container.child(
                    div()
                        .text_color(rgb(log_panel_success))
                        .text_xs()
                        .child(log_line.clone()),
                );
            }
            Some(log_container)
        } else {
            None
        };

        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W only - ScriptList has special ESC handling below)
                if this.handle_global_shortcut_with_options(event, false, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check SDK action shortcuts FIRST (before built-in shortcuts)
                // This allows scripts to override default shortcuts via setActions()
                if !this.action_shortcuts.is_empty() {
                    let key_combo =
                        shortcuts::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
                    if let Some(action_name) = this.action_shortcuts.get(&key_combo).cloned() {
                        logging::log(
                            "ACTIONS",
                            &format!(
                                "SDK action shortcut matched: '{}' -> '{}'",
                                key_combo, action_name
                            ),
                        );
                        if this.trigger_action_by_name(&action_name, cx) {
                            return;
                        }
                    }
                }

                if has_cmd {
                    let has_shift = event.keystroke.modifiers.shift;

                    match key_str.as_str() {
                        "l" => {
                            logging::log("KEY", "Shortcut Cmd+L -> toggle_logs");
                            this.toggle_logs(cx);
                            return;
                        }
                        // Cmd+1 cycles through all designs
                        "1" => {
                            logging::log("KEY", "Shortcut Cmd+1 -> cycle_design");
                            this.cycle_design(cx);
                            return;
                        }
                        // Script context shortcuts (require a selected script)
                        // Note: More specific patterns (with shift) must come BEFORE less specific ones
                        "k" if has_shift => {
                            // Cmd+Shift+K - Add/Update Keyboard Shortcut
                            logging::log("KEY", "Shortcut Cmd+Shift+K -> add_shortcut");
                            this.handle_action("add_shortcut".to_string(), cx);
                            return;
                        }
                        "k" => {
                            // Cmd+K - Toggle actions menu
                            if this.has_actions() {
                                logging::log("KEY", "Shortcut Cmd+K -> toggle_actions");
                                this.toggle_actions(cx, window);
                            }
                            return;
                        }
                        "e" => {
                            // Cmd+E - Edit Script
                            logging::log("KEY", "Shortcut Cmd+E -> edit_script");
                            this.handle_action("edit_script".to_string(), cx);
                            return;
                        }
                        "f" if has_shift => {
                            // Cmd+Shift+F - Reveal in Finder
                            logging::log("KEY", "Shortcut Cmd+Shift+F -> reveal_in_finder");
                            this.handle_action("reveal_in_finder".to_string(), cx);
                            return;
                        }
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            logging::log("KEY", "Shortcut Cmd+Shift+C -> copy_path");
                            this.handle_action("copy_path".to_string(), cx);
                            return;
                        }
                        "d" if has_shift => {
                            // Cmd+Shift+D - Copy Deeplink
                            logging::log("KEY", "Shortcut Cmd+Shift+D -> copy_deeplink");
                            this.handle_action("copy_deeplink".to_string(), cx);
                            return;
                        }
                        "a" if has_shift => {
                            // Cmd+Shift+A - Add/Update Alias
                            logging::log("KEY", "Shortcut Cmd+Shift+A -> add_alias");
                            this.handle_action("add_alias".to_string(), cx);
                            return;
                        }
                        // Global shortcuts
                        "n" => {
                            // Cmd+N - Create Script
                            logging::log("KEY", "Shortcut Cmd+N -> create_script");
                            this.handle_action("create_script".to_string(), cx);
                            return;
                        }
                        "r" => {
                            // Cmd+R - Reload Scripts
                            logging::log("KEY", "Shortcut Cmd+R -> reload_scripts");
                            this.handle_action("reload_scripts".to_string(), cx);
                            return;
                        }
                        "," => {
                            // Cmd+, - Settings
                            logging::log("KEY", "Shortcut Cmd+, -> settings");
                            this.handle_action("settings".to_string(), cx);
                            return;
                        }
                        "q" => {
                            // Cmd+Q - Quit
                            logging::log("KEY", "Shortcut Cmd+Q -> quit");
                            this.handle_action("quit".to_string(), cx);
                            return;
                        }
                        _ => {}
                    }
                }

                // If confirm dialog is open, just return - key routing is handled by
                // the dedicated interceptors in app_impl.rs (Tab at line 462-478,
                // arrows at line 645-659, all others at line 920-928)
                // We must NOT dispatch here or it will double-fire toggle_focus!
                if crate::confirm::is_confirm_window_open() {
                    return;
                }

                // If actions popup is open, route keyboard events to it
                if this.show_actions_popup {
                    if let Some(ref dialog) = this.actions_dialog {
                        match key_str.as_str() {
                            "up" | "arrowup" => {
                                dialog.update(cx, |d, cx| d.move_up(cx));
                                // Notify actions window to re-render
                                cx.spawn(async move |_this, cx| {
                                    cx.update(notify_actions_window).ok();
                                })
                                .detach();
                                return;
                            }
                            "down" | "arrowdown" => {
                                dialog.update(cx, |d, cx| d.move_down(cx));
                                // Notify actions window to re-render
                                cx.spawn(async move |_this, cx| {
                                    cx.update(notify_actions_window).ok();
                                })
                                .detach();
                                return;
                            }
                            "enter" | "return" => {
                                // Get the selected action and execute it
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();
                                if let Some(action_id) = action_id {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );
                                    // Only close if action has close: true (default)
                                    if should_close {
                                        this.close_actions_popup(
                                            ActionsDialogHost::MainList,
                                            window,
                                            cx,
                                        );
                                    }
                                    this.handle_action(action_id, cx);
                                }
                                // Notify to update UI state after closing popup
                                cx.notify();
                                return;
                            }
                            "escape" | "esc" => {
                                this.close_actions_popup(ActionsDialogHost::MainList, window, cx);
                                cx.notify();
                                return;
                            }
                            "backspace" => {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                // Resize and notify actions window to re-render
                                let dialog_for_resize = dialog.clone();
                                cx.spawn(async move |_this, cx| {
                                    cx.update(|cx| {
                                        resize_actions_window(cx, &dialog_for_resize);
                                    })
                                    .ok();
                                })
                                .detach();
                                return;
                            }
                            _ => {
                                let modifiers = &event.keystroke.modifiers;

                                // Check for printable character input (only when no modifiers are held)
                                // This prevents Cmd+E from being treated as typing 'e' into the search
                                if !modifiers.platform && !modifiers.control && !modifiers.alt {
                                    if let Some(ref key_char) = event.keystroke.key_char {
                                        if let Some(ch) = key_char.chars().next() {
                                            if !ch.is_control() {
                                                dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                                // Resize and notify actions window to re-render
                                                let dialog_for_resize = dialog.clone();
                                                cx.spawn(async move |_this, cx| {
                                                    cx.update(|cx| {
                                                        resize_actions_window(
                                                            cx,
                                                            &dialog_for_resize,
                                                        );
                                                    })
                                                    .ok();
                                                })
                                                .detach();
                                                return;
                                            }
                                        }
                                    }
                                }

                                // Check if keystroke matches any action shortcut in the dialog
                                // This allows Cmd+E, Cmd+L, etc. to execute the corresponding action
                                let key_lower = key_str.to_lowercase();
                                let keystroke_shortcut =
                                    shortcuts::keystroke_to_shortcut(&key_lower, modifiers);

                                // Read dialog actions and look for matching shortcut
                                let dialog_ref = dialog.read(cx);
                                let mut matched_action: Option<String> = None;
                                for action in &dialog_ref.actions {
                                    if let Some(ref display_shortcut) = action.shortcut {
                                        let normalized =
                                            Self::normalize_display_shortcut(display_shortcut);
                                        if normalized == keystroke_shortcut {
                                            matched_action = Some(action.id.clone());
                                            break;
                                        }
                                    }
                                }
                                let _ = dialog_ref;

                                if let Some(action_id) = matched_action {
                                    logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "Actions dialog shortcut matched: {} -> {}",
                                            keystroke_shortcut, action_id
                                        ),
                                    );
                                    // Close the dialog using centralized helper
                                    this.close_actions_popup(
                                        ActionsDialogHost::MainList,
                                        window,
                                        cx,
                                    );
                                    // Execute the action
                                    this.handle_action(action_id, cx);
                                    cx.notify();
                                }
                                return;
                            }
                        }
                    }
                }

                // LEGACY: Check if we're in fallback mode (no script matches, showing fallback commands)
                // Note: This is legacy code that handled a separate fallback rendering path.
                // Now fallbacks flow through GroupedListItem from grouping.rs, so this
                // branch should rarely (if ever) be triggered. The normal navigation below
                // handles fallback items in the unified list.
                if this.fallback_mode && !this.cached_fallbacks.is_empty() {
                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            if this.fallback_selected_index > 0 {
                                this.fallback_selected_index -= 1;
                                cx.notify();
                            }
                        }
                        "down" | "arrowdown" => {
                            if this.fallback_selected_index
                                < this.cached_fallbacks.len().saturating_sub(1)
                            {
                                this.fallback_selected_index += 1;
                                cx.notify();
                            }
                        }
                        "enter" => {
                            if !this.gpui_input_focused {
                                this.execute_selected_fallback(cx);
                            }
                        }
                        "escape" => {
                            // Clear filter to exit fallback mode
                            this.clear_filter(window, cx);
                        }
                        _ => {}
                    }
                    return;
                }

                // Normal script list navigation
                // NOTE: Arrow keys are now handled by the arrow_interceptor in app_impl.rs
                // which fires before the Input component can consume them. This allows
                // input history navigation + list navigation to work correctly.
                match key_str.as_str() {
                    "enter" => {
                        if !this.gpui_input_focused {
                            this.execute_selected(cx);
                        }
                    }
                    "escape" => {
                        // Clear filter first if there's text, otherwise close window
                        if !this.filter_text.is_empty() {
                            this.clear_filter(window, cx);
                        } else {
                            // Filter is empty - close window
                            this.close_and_reset_window(cx);
                        }
                    }
                    // Tab key: Send query to AI chat if filter has text
                    // Note: This is a fallback - primary Tab handling is in app_impl.rs via intercept_keystrokes
                    "tab" | "Tab" => {
                        if !this.filter_text.is_empty() {
                            let query = this.filter_text.clone();

                            // Open AI window first
                            if let Err(e) = ai::open_ai_window(cx) {
                                logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                            } else {
                                // Set input in AI chat (don't auto-submit - let user review first)
                                ai::set_ai_input(cx, &query, false);
                            }

                            // Clear filter and close main window
                            this.clear_filter(window, cx);
                            this.close_and_reset_window(cx);
                        }
                    }
                    _ => {}
                }
            },
        );

        // Main container with system font and transparency
        // NOTE: Shadow disabled for vibrancy - shadows on transparent elements cause gray fill

        // Use unified color resolver for text and fonts
        let text_primary = color_resolver.primary_text_color();
        let font_family = typography_resolver.primary_font();

        // Extract footer colors BEFORE render_preview_panel (borrow checker).
        // Footer uses theme tokens directly so app-shell chrome stays consistent
        // across design variants (avoids design-token backgrounds like pure white).
        let footer_colors = app_shell_footer_colors(&self.theme);

        // NOTE: No .bg() here - Root provides vibrancy background for ALL content
        // This ensures main menu, AI chat, and all prompts have consistent styling

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
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            // Search input with cursor and selection support
                            .child(
                                div().flex_1().flex().flex_row().items_center().child(
                                    Input::new(&self.gpui_input_state)
                                        .w_full()
                                        .h(px(input_height))
                                        .px(px(0.))
                                        .py(px(0.))
                                        .with_size(Size::Size(px(
                                            typography_resolver.font_size_xl()
                                        )))
                                        .appearance(false)
                                        .bordered(false)
                                        .focus_bordered(false),
                                ),
                            )
                            // "Ask AI [Tab]" keyboard hint - styled as non-clickable to match behavior
                            .child(
                                div()
                                    .id("ask-ai-button")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(ASK_AI_BUTTON_GAP))
                                    .px(px(ASK_AI_BUTTON_PADDING_X))
                                    .py(px(ASK_AI_BUTTON_PADDING_Y))
                                    .rounded(px(ASK_AI_BUTTON_RADIUS))
                                    .bg(rgba((accent_color << 8) | ALPHA_HOVER_ACCENT))
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
                                            .bg(rgba((search_box_bg << 8) | ALPHA_TAB_BADGE_BG))
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .child("Tab"),
                                    ),
                            ),
                    )
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

            // Keep footer action copy stable; do not reflect selected-item type/action text.
            let mut footer_config =
                PromptFooterConfig::default().primary_label(script_list_footer_primary_label());

            let window_tweaker_enabled = std::env::var("SCRIPT_KIT_WINDOW_TWEAKER")
                .map(|v| v == "1")
                .unwrap_or(false);
            let opacity_percent = (self.theme.get_opacity().main * 100.0).round() as i32;
            let material = platform::get_current_material_name();
            let appearance = platform::get_current_appearance_name();
            if let Some(info_label) = script_list_footer_info_label(
                window_tweaker_enabled,
                self.theme.is_dark_mode(),
                opacity_percent,
                &material,
                &appearance,
            ) {
                footer_config = footer_config.info_label(info_label);
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
    }
}

#[cfg(test)]
mod render_script_list_footer_tests {
    use super::{
        app_shell_footer_colors, inline_calc_list_item_title, script_list_footer_info_label,
        script_list_footer_primary_label,
    };

    #[test]
    fn test_app_shell_footer_colors_use_theme_accent_tokens() {
        let theme = crate::theme::Theme::default();
        let colors = app_shell_footer_colors(&theme);

        assert_eq!(colors.accent, theme.colors.accent.selected);
        assert_eq!(colors.background, theme.colors.accent.selected_subtle);
        assert_eq!(colors.border, theme.colors.ui.border);
        assert_eq!(colors.text_muted, theme.colors.text.muted);
    }

    #[test]
    fn test_script_list_footer_primary_label_is_generic_run() {
        assert_eq!(script_list_footer_primary_label(), "Run");
    }

    #[test]
    fn test_script_list_footer_info_label_hidden_when_window_tweaker_disabled() {
        assert_eq!(
            script_list_footer_info_label(false, false, 75, "acrylic", "light"),
            None
        );
    }

    #[test]
    fn test_script_list_footer_info_label_hidden_in_dark_mode() {
        assert_eq!(
            script_list_footer_info_label(true, true, 75, "acrylic", "dark"),
            None
        );
    }

    #[test]
    fn test_script_list_footer_info_label_formats_window_tweaker_metadata() {
        assert_eq!(
            script_list_footer_info_label(true, false, 75, "acrylic", "light"),
            Some("75% | acrylic | light | ⌘-/+ ⌘M ⌘⇧A".to_string())
        );
    }

    #[test]
    fn test_truncate_str_chars_returns_valid_utf8_boundary_when_filter_text_is_multibyte() {
        let input = "é".repeat(45);
        let truncated = crate::utils::truncate_str_chars(&input, 27);

        assert_eq!(truncated.chars().count(), 27);
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn test_inline_calc_list_item_title_prefixes_equals_sign() {
        assert_eq!(inline_calc_list_item_title("1500"), "= 1500");
    }
}
