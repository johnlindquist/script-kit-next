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
                let filter_display = if self.filter_text.len() > 30 {
                    format!("{}...", &self.filter_text[..27])
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
                    .visible(self.is_scrolling);

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
                                    if let Some(result) = flat_results_clone.get(*result_idx) {
                                        let is_selected = ix == current_selected;
                                        // Only show hover effect when in Mouse mode to prevent dual-highlight
                                        let is_hovered = current_hovered == Some(ix) && current_input_mode == InputMode::Mouse;

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
                                                if let gpui::ClickEvent::Mouse(mouse_event) = event
                                                {
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
                                        // Enable hover effects only when in Mouse mode
                                        let enable_hover = current_input_mode == InputMode::Mouse;
                                        let item_element = render_design_item(
                                            current_design,
                                            result,
                                            ix,
                                            is_selected,
                                            is_hovered,
                                            theme_colors,
                                            enable_hover,
                                            &filter_for_highlight,
                                        );
                                        let design_elapsed = design_render_start.elapsed();

                                        // Log slow items (>1ms)
                                        if design_elapsed.as_micros() > 1000 {
                                            logging::log(
                                                "FILTER_PERF",
                                                &format!(
                                                    "[SLOW_ITEM] ix={} name='{}' design_render={:.2}ms filter='{}'",
                                                    ix,
                                                    result.name(),
                                                    design_elapsed.as_secs_f64() * 1000.0,
                                                    filter_for_closure
                                                ),
                                            );
                                        }

                                        div()
                                            .id(ElementId::NamedInteger(
                                                "script-item".into(),
                                                ix as u64,
                                            ))
                                            .h(px(LIST_ITEM_HEIGHT)) // Explicit 40px height (8px grid)
                                            .on_hover(hover_handler)
                                            .on_click(click_handler)
                                            .child(item_element)
                                            .into_any_element()
                                    } else {
                                        // Fallback for missing result
                                        div().h(px(LIST_ITEM_HEIGHT)).into_any_element()
                                    }
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
