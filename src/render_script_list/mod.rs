// Script list render method - extracted from app_render.rs
// This file is included via include!() macro in main.rs
use crate::ui_foundation::{
    is_key_down as sk_is_key_down, is_key_enter as sk_is_key_enter,
    is_key_escape as sk_is_key_escape, is_key_tab as sk_is_key_tab, is_key_up as sk_is_key_up,
};

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

fn inline_calc_list_item_result_text_color(
    is_selected: bool,
    design_variant: DesignVariant,
    theme: &crate::theme::Theme,
    color_resolver: crate::theme::ColorResolver,
) -> u32 {
    if is_selected && design_variant != DesignVariant::Default {
        color_resolver.primary_accent()
    } else if is_selected {
        theme.colors.accent.selected
    } else {
        color_resolver.primary_text_color()
    }
}

fn inline_calc_list_item_hint_text_color(color_resolver: crate::theme::ColorResolver) -> u32 {
    color_resolver.empty_text_color()
}

fn inline_calc_list_item_selected_overlay_rgba(
    theme: &crate::theme::Theme,
    color_resolver: crate::theme::ColorResolver,
) -> u32 {
    let selected_overlay_alpha =
        ((theme.get_opacity().selected.clamp(0.0, 1.0) * 255.0).round() as u32).max(0x2E);
    (color_resolver.primary_accent() << 8) | selected_overlay_alpha
}

fn render_inline_calc_list_item(
    calculator: &crate::calculator::CalculatorInlineResult,
    is_selected: bool,
    theme: &crate::theme::Theme,
    design_variant: DesignVariant,
    color_resolver: crate::theme::ColorResolver,
) -> AnyElement {
    let tokens = get_tokens(design_variant);
    let spacing = tokens.spacing();
    let typography = tokens.typography();

    let result_title = inline_calc_list_item_title(&calculator.formatted);
    let result_text_color =
        inline_calc_list_item_result_text_color(is_selected, design_variant, theme, color_resolver);
    let hint_text_color = inline_calc_list_item_hint_text_color(color_resolver);
    let hint_alpha = if is_selected { 0xD9 } else { 0x8C };

    div()
        .w_full()
        .h_full()
        .px(px(spacing.item_padding_x))
        .py(px(spacing.padding_xs))
        .when(is_selected, |div| {
            div.bg(rgba(inline_calc_list_item_selected_overlay_rgba(
                theme,
                color_resolver,
            )))
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
                .text_color(rgba((hint_text_color << 8) | hint_alpha))
                .child(inline_calc_list_copy_hint()),
        )
        .into_any_element()
}

impl ScriptListApp {
    fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let render_list_start = std::time::Instant::now();
        let filter_for_log = self.filter_text.clone();
        let is_mini = self.main_window_mode == MainWindowMode::Mini;

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
        // Shell uses theme-first so non-default design variants keep the active
        // theme's colors while still using the variant's spacing and shape tokens.
        let color_resolver = crate::theme::ColorResolver::new_for_shell(&self.theme, self.current_design);
        let typography_resolver =
            crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design);
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
        let empty_font_family = typography_resolver.primary_font().to_string();

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
                    .font_family(empty_font_family.clone())
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
                                    // Hover gating is now handled by ListItem via GPUI input modality
                                    let is_hovered = current_hovered == Some(ix);

                                    // Create hover handler
                                    let hover_handler = cx.listener(
                                        move |this: &mut ScriptListApp,
                                              hovered: &bool,
                                              _window,
                                              cx| {
                                            if *hovered {
                                                this.input_mode = InputMode::Mouse;
                                                if this.hovered_index != Some(ix) {
                                                    this.hovered_index = Some(ix);
                                                    cx.notify();
                                                }
                                            } else if this.hovered_index == Some(ix) {
                                                this.hovered_index = None;
                                                cx.notify();
                                            }
                                        },
                                    );

                                    // Create click handler matching launcher click semantics
                                    let click_handler = cx.listener(
                                        move |this: &mut ScriptListApp,
                                              event: &gpui::ClickEvent,
                                              _window,
                                              cx| {
                                            let was_selected = this.selected_index == ix;
                                            // Always select the item on any click
                                            if !was_selected {
                                                this.selected_index = ix;
                                                cx.notify();
                                            }

                                            let click_count = event.click_count();
                                            if crate::ui_foundation::should_submit_selected_row_click(
                                                was_selected,
                                                click_count,
                                            ) {
                                                logging::log(
                                                    "UI",
                                                    &format!(
                                                        "Launcher row click submitting item {} (click_count={})",
                                                        ix, click_count
                                                    ),
                                                );
                                                this.execute_selected(cx);
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
                                            color_resolver,
                                        )
                                    } else if let Some(result) = flat_results_clone.get(*result_idx)
                                    {
                                        item_name = result.name();
                                        render_design_item(
                                            current_design,
                                            result,
                                            ix,
                                            is_selected,
                                            is_hovered,
                                            theme_colors,
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
                if event.keystroke.modifiers.platform {
                    tracing::debug!(
                        event = "script_list.key_down",
                        key = %event.keystroke.key,
                        cmd = true,
                        shift = event.keystroke.modifiers.shift,
                        mini_mode = (this.main_window_mode == MainWindowMode::Mini),
                        "script_list key_down: cmd+{}",
                        event.keystroke.key,
                    );
                }
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

                let key_str = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check SDK action shortcuts FIRST (before built-in shortcuts)
                // This allows scripts to override default shortcuts via setActions()
                if !this.action_shortcuts.is_empty() {
                    let key_combo =
                        shortcuts::keystroke_to_shortcut(key_str, &event.keystroke.modifiers);
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

                // If actions popup is open, route all keyboard events through the shared router
                let key_char = event.keystroke.key_char.as_ref().map(|s| s.as_ref());
                match this.route_key_to_actions_dialog(
                    key_str,
                    key_char,
                    &event.keystroke.modifiers,
                    ActionsDialogHost::MainList,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        this.handle_action(action_id, window, cx);
                        cx.notify();
                        return;
                    }
                    ActionsRoute::Handled => return,
                    ActionsRoute::NotHandled => {}
                }

                if has_cmd {
                    let has_shift = event.keystroke.modifiers.shift;

                    match key_str {
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
                            this.handle_action("add_shortcut".to_string(), window, cx);
                            return;
                        }
                        "k" => {
                            // Cmd+K - Toggle actions menu
                            let has = this.has_actions();
                            tracing::debug!(
                                event = "cmd_k.pressed",
                                has_actions = has,
                                selected_index = this.selected_index,
                                mini_mode = (this.main_window_mode == MainWindowMode::Mini),
                                "Cmd+K pressed, has_actions={}",
                                has,
                            );
                            if has {
                                logging::log("KEY", "Shortcut Cmd+K -> toggle_actions");
                                this.toggle_actions(cx, window);
                            }
                            return;
                        }
                        "i" => {
                            // Cmd+I - Toggle Info Panel
                            logging::log("KEY", "Shortcut Cmd+I -> toggle_info");
                            this.handle_action("toggle_info".to_string(), window, cx);
                            return;
                        }
                        "e" => {
                            // Cmd+E - Edit Script
                            logging::log("KEY", "Shortcut Cmd+E -> edit_script");
                            this.handle_action("edit_script".to_string(), window, cx);
                            return;
                        }
                        "f" if has_shift => {
                            // Cmd+Shift+F - Reveal in Finder
                            logging::log("KEY", "Shortcut Cmd+Shift+F -> reveal_in_finder");
                            this.handle_action("reveal_in_finder".to_string(), window, cx);
                            return;
                        }
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            logging::log("KEY", "Shortcut Cmd+Shift+C -> copy_path");
                            this.handle_action("copy_path".to_string(), window, cx);
                            return;
                        }
                        "d" if has_shift => {
                            // Cmd+Shift+D - Copy Deeplink
                            logging::log("KEY", "Shortcut Cmd+Shift+D -> copy_deeplink");
                            this.handle_action("copy_deeplink".to_string(), window, cx);
                            return;
                        }
                        "a" if has_shift => {
                            // Cmd+Shift+A - Add/Update Alias
                            logging::log("KEY", "Shortcut Cmd+Shift+A -> add_alias");
                            this.handle_action("add_alias".to_string(), window, cx);
                            return;
                        }
                        // Global shortcuts
                        "n" => {
                            // Cmd+N - Create Script
                            logging::log("KEY", "Shortcut Cmd+N -> create_script");
                            this.handle_action("create_script".to_string(), window, cx);
                            return;
                        }
                        "r" => {
                            // Cmd+R - Reload Scripts
                            logging::log("KEY", "Shortcut Cmd+R -> reload_scripts");
                            this.handle_action("reload_scripts".to_string(), window, cx);
                            return;
                        }
                        "," => {
                            // Cmd+, - Settings
                            logging::log("KEY", "Shortcut Cmd+, -> settings");
                            this.handle_action("settings".to_string(), window, cx);
                            return;
                        }
                        "q" => {
                            // Cmd+Q - Quit
                            logging::log("KEY", "Shortcut Cmd+Q -> quit");
                            this.handle_action("quit".to_string(), window, cx);
                            return;
                        }
                        _ => {}
                    }
                }

                // Actions popup keyboard routing is handled above via route_key_to_actions_dialog

                // LEGACY: Check if we're in fallback mode (no script matches, showing fallback commands)
                // Note: This is legacy code that handled a separate fallback rendering path.
                // Now fallbacks flow through GroupedListItem from grouping.rs, so this
                // branch should rarely (if ever) be triggered. The normal navigation below
                // handles fallback items in the unified list.
                if this.fallback_mode && !this.cached_fallbacks.is_empty() {
                    match key_str {
                        key if sk_is_key_up(key) => {
                            if this.fallback_selected_index > 0 {
                                this.fallback_selected_index -= 1;
                                cx.notify();
                            }
                        }
                        key if sk_is_key_down(key) => {
                            if this.fallback_selected_index
                                < this.cached_fallbacks.len().saturating_sub(1)
                            {
                                this.fallback_selected_index += 1;
                                cx.notify();
                            }
                        }
                        key if sk_is_key_enter(key) => {
                            if !this.gpui_input_focused {
                                this.execute_selected_fallback(cx);
                            }
                        }
                        key if sk_is_key_escape(key) => {
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
                match key_str {
                    key if sk_is_key_enter(key) => {
                        if !this.gpui_input_focused {
                            this.execute_selected(cx);
                        }
                    }
                    key if sk_is_key_escape(key) => {
                        // Clear filter first if there's text, otherwise close window
                        if !this.filter_text.is_empty() {
                            this.clear_filter(window, cx);
                        } else {
                            // Filter is empty - close window
                            this.close_and_reset_window(cx);
                        }
                    }
                    // Tab key: consumed by intercept_keystrokes in startup_new_tab.rs.
                    // This fallback only fires if the interceptor somehow misses;
                    // route to Tab AI quick terminal (harness surface).
                    key if sk_is_key_tab(key) => {
                        this.open_tab_ai_chat(cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            },
        );

        // Main container with system font and transparency
        // NOTE: Shadow disabled for vibrancy - shadows on transparent elements cause gray fill

        // Use unified color resolver for text and fonts
        let text_primary = color_resolver.primary_text_color();
        let font_family = typography_resolver.primary_font().to_string();

        // Extract footer colors BEFORE render_preview_panel (borrow checker).
        // Footer uses theme tokens directly so app-shell chrome stays consistent
        // across design variants (avoids design-token backgrounds like pure white).
        let footer_colors = app_shell_footer_colors(&self.theme);
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);

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
                    if is_mini {
                        crate::window_resize::mini_layout::HEADER_PADDING_X
                    } else {
                        HEADER_PADDING_X
                    }
                } else {
                    design_spacing.padding_lg
                };
                let header_padding_y = if is_default_design {
                    if is_mini {
                        crate::window_resize::mini_layout::HEADER_PADDING_Y
                    } else {
                        HEADER_PADDING_Y
                    }
                } else {
                    design_spacing.padding_sm
                };
                let header_gap = if is_default_design {
                    HEADER_GAP
                } else {
                    design_spacing.gap_md
                };
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
                            .when(!is_mini, |d| {
                                d.child(
                                    div()
                                        .id("ask-ai-button")
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(ASK_AI_BUTTON_GAP))
                                        .px(px(ASK_AI_BUTTON_PADDING_X))
                                        .py(px(ASK_AI_BUTTON_PADDING_Y))
                                        .rounded(px(ASK_AI_BUTTON_RADIUS))
                                        .bg(rgba(chrome.accent_badge_bg_rgba))
                                        .cursor_default()
                                        // "Ask AI" text - YELLOW (accent)
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(chrome.accent_badge_text_hex))
                                                .child("Ask AI"),
                                        )
                                        // "Tab" badge - grey background via chrome contract (no border)
                                        .child(
                                            div()
                                                .px(px(TAB_BADGE_PADDING_X))
                                                .py(px(TAB_BADGE_PADDING_Y))
                                                .rounded(px(TAB_BADGE_RADIUS))
                                                .bg(rgba(chrome.badge_bg_rgba))
                                                .text_xs()
                                                .text_color(rgb(chrome.badge_text_hex))
                                                .child("Tab"),
                                        ),
                                )
                            }),
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
                let border_width = if is_default_design {
                    DIVIDER_BORDER_WIDTH_DEFAULT
                } else {
                    design_visual.border_thin
                };

                div()
                    .mx(px(divider_margin))
                    .h(px(border_width))
                    .bg(rgba(chrome.divider_rgba))
            });

        if is_mini {
            // Mini mode: single column, toggle between list and info panel
            if self.show_info_panel {
                // Info panel replaces the list when toggled via Cmd+I
                let info_panel = self.render_preview_panel(cx);
                main_div = main_div.child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .w_full()
                        .overflow_hidden()
                        .child(div().w_full().h_full().min_h(px(0.)).child(info_panel)),
                );
            } else {
                main_div = main_div.child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .w_full()
                        .overflow_hidden()
                        .child(div().w_full().h_full().min_h(px(0.)).child(list_element)),
                );
            }

            // Compact hint strip instead of footer — uses shared mini_layout tokens
            // and opacity-blended text for a softer, Raycast-like launcher feel.
            let hint_text_hex = self.theme.colors.text.primary;
            let hint_opacity_byte = (crate::window_resize::mini_layout::HINT_TEXT_OPACITY * 255.0)
                .round() as u32;
            let hint_text_rgba = (hint_text_hex << 8) | hint_opacity_byte;
            main_div = main_div.child(
                div()
                    .w_full()
                    .h(px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT))
                    .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
                    .py(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    // Subtle top border to separate hint strip from list
                    .border_t(px(crate::window_resize::mini_layout::DIVIDER_HEIGHT))
                    .border_color(rgba(chrome.divider_rgba))
                    .child(crate::components::render_hint_icons_clickable(
                        vec![
                            crate::components::ClickableHint::new("↵ Run", cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
                                this.execute_selected(cx);
                            })),
                            crate::components::ClickableHint::new("⌘K Actions", cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
                                this.toggle_actions(cx, window);
                            })),
                            crate::components::ClickableHint::new("Tab AI", cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
                                this.open_tab_ai_chat(cx);
                            })),
                        ],
                        hint_text_rgba,
                    )),
            );

            if let Some(panel) = log_panel {
                main_div = main_div.child(panel);
            }

            if state_changed {
                let total_elapsed = render_list_start.elapsed();
                tracing::info!(
                    target: "RENDER_PERF",
                    category = "mini_render",
                    event = "render_script_list_end",
                    filter = %filter_for_log,
                    item_count = item_count_for_log,
                    selected_index = self.selected_index,
                    total_ms = format_args!("{:.2}", total_elapsed.as_secs_f64() * 1000.0),
                    mode = "mini",
                    "mini script list render complete"
                );
                self.last_render_log_filter = self.filter_text.clone();
                self.last_render_log_selection = self.selected_index;
                self.last_render_log_item_count = item_count_for_log;
            }

            return main_div.into_any_element();
        }

        // Main content area: list takes full width unless info panel is toggled (Cmd+I)
        {
            let content_row = div()
                .flex()
                .flex_row()
                .flex_1()
                .min_h(px(0.)) // Critical: allows flex container to shrink properly
                .w_full()
                .overflow_hidden()
                // Left side: Script list — full width when info hidden, 50% when shown
                .child(
                    div()
                        .when(self.show_info_panel, |d| d.w_1_2())
                        .when(!self.show_info_panel, |d| d.w_full())
                        .h_full()
                        .min_h(px(0.))
                        .child(list_element),
                )
                // Right side: Info panel (50% width), only rendered when toggled
                .when(self.show_info_panel, |row| {
                    let preview_start = std::time::Instant::now();
                    let preview_panel = self.render_preview_panel(cx);
                    let preview_elapsed = preview_start.elapsed();
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
                    row.child(
                        div()
                            .relative()
                            .flex()
                            .flex_col()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .overflow_hidden()
                            .when_some(
                                self.cached_main_window_preflight.clone(),
                                |d, receipt| {
                                    d.child(
                                        crate::main_window_preflight::render_main_window_preflight_receipt(
                                            self,
                                            &receipt,
                                        ),
                                    )
                                },
                            )
                            .child(div().flex_1().min_h(px(0.)).child(preview_panel)),
                    )
                });
            main_div = main_div.child(content_row);
        }

        // Footer: Logo left | Run Script ↵ | divider | Actions ⌘K right
        // Raycast-style footer with Script Kit branding using reusable PromptFooter component
        // Note: footer colors extracted earlier to avoid borrow conflict with render_preview_panel
        main_div = main_div.child({
            let handle_run = cx.entity().downgrade();
            let handle_actions = cx.entity().downgrade();

            // Resolve footer config — when the storybook feature is enabled,
            // load the persisted design variant; otherwise use hardcoded defaults.
            #[cfg(feature = "storybook")]
            let mut footer_config = {
                let selected_footer_variant =
                    script_kit_gpui::storybook::load_selected_story_variant(
                        "footer-layout-variations",
                    );
                let (spec, resolution) =
                    script_kit_gpui::storybook::resolve_footer_selection_spec(
                        selected_footer_variant.as_deref(),
                    );
                let mut fc = PromptFooterConfig::new()
                    .primary_label(spec.primary_label)
                    .primary_shortcut(spec.primary_shortcut)
                    .secondary_label(spec.secondary_label)
                    .secondary_shortcut(spec.secondary_shortcut)
                    .show_logo(spec.show_logo)
                    .show_primary(spec.show_primary)
                    .show_secondary(spec.show_secondary)
                    .show_info_label(spec.show_info_label);
                if let Some(helper_text) = spec.helper_text {
                    fc = fc.helper_text(helper_text);
                }
                if let Some(info_label) = spec.info_label {
                    fc = fc.info_label(info_label);
                }
                if state_changed {
                    tracing::info!(
                        event = "script_list_footer_selection_resolved",
                        requested_variant_id = resolution
                            .requested_variant_id
                            .as_deref()
                            .unwrap_or(""),
                        resolved_variant_id = resolution.resolved_variant_id.as_str(),
                        fallback_used = resolution.fallback_used,
                        "Resolved footer variant for live script list"
                    );
                }
                // Stash slot texts for later use after footer is built
                #[allow(unused_variables)]
                let left_slot_text = spec.left_slot_text;
                #[allow(unused_variables)]
                let right_slot_text = spec.right_slot_text;
                fc
            };

            #[cfg(not(feature = "storybook"))]
            let mut footer_config = PromptFooterConfig::new()
                .primary_label("Open Application")
                .primary_shortcut("↵")
                .secondary_label("Actions")
                .secondary_shortcut("⌘K")
                .show_logo(true)
                .show_primary(true)
                .show_secondary(true)
                .show_info_label(false);

            footer_config = footer_config.primary_label(script_list_footer_primary_label());

            let window_tweaker_enabled = std::env::var("SCRIPT_KIT_WINDOW_TWEAKER")
                .map(|v| v == "1")
                .unwrap_or(false);
            let opacity_percent = (self.theme.get_opacity().main * 100.0).round() as i32;
            let material = platform::get_current_material_name();
            let appearance = platform::get_current_appearance_name();
            if footer_config.show_info_label {
                if let Some(info_label) = script_list_footer_info_label(
                    window_tweaker_enabled,
                    self.theme.is_dark_mode(),
                    opacity_percent,
                    &material,
                    &appearance,
                ) {
                    footer_config = footer_config.info_label(info_label);
                }
            }
            if footer_config.show_secondary {
                footer_config = footer_config.show_secondary(self.has_actions());
                footer_config = footer_config.secondary_active(self.show_actions_popup);
            }

            #[allow(unused_mut)]
            let mut footer = PromptFooter::new(footer_config, footer_colors)
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
                }));

            // Slot text rendering is only available with storybook feature
            #[cfg(feature = "storybook")]
            {
                let selected_footer_variant =
                    script_kit_gpui::storybook::load_selected_story_variant(
                        "footer-layout-variations",
                    );
                let (spec, _) = script_kit_gpui::storybook::resolve_footer_selection_spec(
                    selected_footer_variant.as_deref(),
                );
                if let Some(left_slot_text) = spec.left_slot_text {
                    footer =
                        footer.left_slot(script_kit_gpui::storybook::render_footer_slot_text(
                            left_slot_text,
                            true,
                        ));
                }
                if let Some(right_slot_text) = spec.right_slot_text {
                    footer =
                        footer.right_slot(script_kit_gpui::storybook::render_footer_slot_text(
                            right_slot_text,
                            false,
                        ));
                }
            }

            footer
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
        app_shell_footer_colors, inline_calc_list_item_hint_text_color,
        inline_calc_list_item_result_text_color, inline_calc_list_item_selected_overlay_rgba,
        inline_calc_list_item_title, script_list_footer_info_label,
        script_list_footer_primary_label,
    };
    use crate::designs::DesignVariant;
    use crate::theme::ColorResolver;

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

    #[test]
    fn test_inline_calc_result_text_color_does_use_resolver_accent_when_selected_non_default() {
        let mut theme = crate::theme::Theme::default();
        theme.colors.accent.selected = 0x112233;
        let color_resolver = ColorResolver::new(&theme, DesignVariant::NeonCyberpunk);

        let color = inline_calc_list_item_result_text_color(
            true,
            DesignVariant::NeonCyberpunk,
            &theme,
            color_resolver,
        );

        assert_eq!(color, color_resolver.primary_accent());
        assert_ne!(color, theme.colors.accent.selected);
    }

    #[test]
    fn test_inline_calc_hint_text_color_does_use_color_resolver_muted_token() {
        let theme = crate::theme::Theme::default();
        let color_resolver = ColorResolver::new(&theme, DesignVariant::NeonCyberpunk);

        assert_eq!(
            inline_calc_list_item_hint_text_color(color_resolver),
            color_resolver.empty_text_color()
        );
    }

    #[test]
    fn test_inline_calc_selected_overlay_does_use_resolver_accent_with_theme_alpha() {
        let mut theme = crate::theme::Theme::default();
        theme.colors.accent.selected_subtle = 0x010203;
        let color_resolver = ColorResolver::new(&theme, DesignVariant::NeonCyberpunk);

        let expected_alpha =
            ((theme.get_opacity().selected.clamp(0.0, 1.0) * 255.0).round() as u32).max(0x2E);
        let expected = (color_resolver.primary_accent() << 8) | expected_alpha;

        assert_eq!(
            inline_calc_list_item_selected_overlay_rgba(&theme, color_resolver),
            expected
        );
    }
}

#[cfg(test)]
mod render_script_list_click_contract_tests {
    use std::fs;

    #[test]
    fn launcher_list_uses_shared_selected_row_click_helper() {
        let source = fs::read_to_string("src/render_script_list/mod.rs")
            .expect("Failed to read src/render_script_list/mod.rs");

        assert!(
            source.contains("should_submit_selected_row_click"),
            "render_script_list should use the shared selected-row click helper"
        );
        assert!(
            source.contains("let was_selected = this.selected_index == ix;"),
            "render_script_list click handler should capture whether the row was already selected"
        );
        assert!(
            source.contains("this.execute_selected(cx);"),
            "render_script_list click handler should still execute the selected row"
        );
    }
}
