// Script list render method - extracted from app_render.rs
// This file is included via include!() macro in main.rs

impl ScriptListApp {
    fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Get grouped or flat results based on filter state (cached) - MUST come first
        // to avoid borrow conflicts with theme access below
        // When filter is empty, use frecency-grouped results with RECENT/MAIN sections
        // When filtering, use flat fuzzy search results
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone for use in closures and to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get design tokens for current design variant
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_visual = tokens.visual();
        let design_typography = tokens.typography();

        // For Default design, use theme.colors for backward compatibility
        // For other designs, use design tokens
        let is_default_design = self.current_design == DesignVariant::Default;

        let item_count = grouped_items.len();
        let _total_len = self.scripts.len() + self.scriptlets.len();

        // ============================================================
        // MUTABLE OPERATIONS BLOCK - do all &mut self calls here BEFORE
        // taking immutable borrows of theme for UI building
        // ============================================================

        // Handle edge cases - keep selected_index in valid bounds
        // Use coerce_selection which tries down first, then up, handles all edge cases
        //
        // Note: Fallbacks now flow through GroupedListItem from get_grouped_results().
        // When filter_text is non-empty, fallbacks are appended to the results, so
        // item_count > 0 even when there are no regular matches. The old fallback_mode
        // logic (separate rendering path) is kept for backwards compatibility but
        // should rarely be triggered now.
        if item_count > 0 {
            // We have results (may include fallbacks) - exit legacy fallback mode
            self.fallback_mode = false;
            self.cached_fallbacks.clear();

            if let Some(valid_idx) =
                list_item::coerce_selection(&grouped_items, self.selected_index)
            {
                self.selected_index = valid_idx;
            } else {
                // No selectable items (list is all headers) - set to 0 as fallback
                self.selected_index = 0;
            }
        } else {
            // Empty list - reset selection state to avoid stale indices
            // This path is hit when filter is empty AND there are truly no items,
            // or in edge cases where even fallbacks aren't available
            self.selected_index = 0;
            self.hovered_index = None;
            self.last_scrolled_index = None;

            // Legacy fallback mode: only used if grouping.rs doesn't include fallbacks
            // (This is a safety net - normally grouping.rs appends fallbacks to results)
            self.fallback_mode = false;
            self.cached_fallbacks.clear();
        }

        // Update list state if item count changed
        // Use splice instead of reset to preserve scroll events and measurement cache
        // reset() drops scroll events until the list is painted, which can break scroll-driven UI
        let old_list_count = self.main_list_state.item_count();
        if old_list_count != item_count {
            self.main_list_state.splice(0..old_list_count, item_count);
            // Invalidate last_scrolled_index since list structure changed
            self.last_scrolled_index = None;
        }

        // Only scroll to reveal selection when selection actually changed
        // This prevents fighting trackpad/wheel scrolling and reduces redundant scroll calls
        self.scroll_to_selected_if_needed("render_list");

        // Get scroll offset AFTER updates for scrollbar
        let scroll_offset = self.main_list_state.logical_scroll_top().item_ix;

        // ============================================================
        // IMMUTABLE BORROWS BLOCK - extract theme values for UI building
        // ============================================================

        // Extract theme values as owned copies for UI building
        let log_panel_bg = self.theme.colors.background.log_panel;
        let log_panel_border = self.theme.colors.ui.border;
        let log_panel_success = self.theme.colors.ui.success;

        // Pre-compute scrollbar colors (Copy type)
        let scrollbar_colors = if is_default_design {
            ScrollbarColors::from_theme(&self.theme)
        } else {
            ScrollbarColors::from_design(&design_colors)
        };
        // Pre-compute list item colors for closure (Copy type)
        let theme_colors = ListItemColors::from_theme(&self.theme);

        let theme = &self.theme;

        logging::log_debug("PERF", "P4: Using ListItemColors for render closure");

        // Build script list using uniform_list for proper virtualized scrolling
        // Use design tokens for empty state styling
        let empty_text_color = if is_default_design {
            theme.colors.text.muted
        } else {
            design_colors.text_muted
        };
        let empty_font_family = if is_default_design {
            ".AppleSystemUIFont"
        } else {
            design_typography.font_family
        };

        let list_element: AnyElement = if item_count == 0 {
            // When there's no filter text, show "No scripts or snippets found"
            // When filtering, show Raycast-style fallback list instead of "No results"
            // Empty list handling:
            // - When filter is empty: "No scripts or snippets found"
            // - When filter has text: "No results match '...'" (rare - fallbacks usually exist)
            //
            // Note: This branch is rarely hit when filtering because grouping.rs now
            // appends fallbacks to the results. We only get here if there are truly
            // no results at all (including no fallbacks).
            if self.filter_text.is_empty() {
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(empty_text_color))
                    .font_family(empty_font_family)
                    .child("No scripts or snippets found")
                    .into_any_element()
            } else {
                // Filtering but no results (including no fallbacks) - shouldn't normally happen
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(empty_text_color))
                    .font_family(empty_font_family)
                    .child(format!("No results match '{}'", self.filter_text))
                    .into_any_element()
            }
        } else {
            // Use GPUI's list() component for variable-height items
            // Section headers render at 24px, regular items at 48px
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
                    GroupedListItem::SectionHeader(_) => header_count += 1,
                    GroupedListItem::Item(_) => item_count_regular += 1,
                }
            }

            // Calculate true content height: headers at 24px, items at 48px
            let total_content_height = (header_count as f32 * SECTION_HEADER_HEIGHT)
                + (item_count_regular as f32 * LIST_ITEM_HEIGHT);

            // Estimated visible container height
            // Window is 500px, header is ~60px, remaining ~440px for list area
            // Use a slightly higher estimate to ensure scrollbar thumb reaches bottom
            // (underestimating visible items causes thumb to not reach bottom)
            let estimated_container_height = 440.0_f32;

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

            let variable_height_list =
                list(self.main_list_state.clone(), move |ix, _window, cx| {
                    // Access entity state inside the closure
                    entity.update(cx, |this, cx| {
                        let current_selected = this.selected_index;
                        let current_hovered = this.hovered_index;

                        if let Some(grouped_item) = grouped_items_clone.get(ix) {
                            match grouped_item {
                                GroupedListItem::SectionHeader(label) => {
                                    // Section header at 24px height (SECTION_HEADER_HEIGHT)
                                    div()
                                        .id(ElementId::NamedInteger(
                                            "section-header".into(),
                                            ix as u64,
                                        ))
                                        .h(px(SECTION_HEADER_HEIGHT))
                                        .child(render_section_header(label, theme_colors))
                                        .into_any_element()
                                }
                                GroupedListItem::Item(result_idx) => {
                                    // Regular item at 48px height (LIST_ITEM_HEIGHT)
                                    if let Some(result) = flat_results_clone.get(*result_idx) {
                                        let is_selected = ix == current_selected;
                                        let is_hovered = current_hovered == Some(ix);

                                        // Create hover handler
                                        let hover_handler = cx.listener(
                                            move |this: &mut ScriptListApp,
                                                  hovered: &bool,
                                                  _window,
                                                  cx| {
                                                let now = std::time::Instant::now();
                                                const HOVER_DEBOUNCE_MS: u64 = 16;

                                                if *hovered {
                                                    // Mouse entered - set hovered_index with debounce
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

                                        // Check if this item requires confirmation and is pending
                                        let pending_id = this.pending_confirmation.clone();
                                        let is_pending_confirmation = match result {
                                            scripts::SearchResult::BuiltIn(bm) => {
                                                pending_id.as_ref() == Some(&bm.entry.id)
                                            }
                                            _ => false,
                                        };

                                        // Dispatch to design-specific item renderer
                                        let item_element = render_design_item(
                                            current_design,
                                            result,
                                            ix,
                                            is_selected,
                                            is_hovered,
                                            theme_colors,
                                        );

                                        // Wrap with confirmation overlay if pending
                                        let final_element = if is_pending_confirmation && is_selected {
                                            // Create confirmation overlay
                                            let warning_bg = rgb(0xB85C00); // Orange/warning background
                                            let confirm_name = match result {
                                                scripts::SearchResult::BuiltIn(bm) => {
                                                    format!("⚠️ Confirm {}? (Enter)", bm.entry.name)
                                                }
                                                _ => "⚠️ Confirm? (Enter)".to_string(),
                                            };

                                            div()
                                                .w_full()
                                                .h(px(LIST_ITEM_HEIGHT))
                                                .flex()
                                                .items_center()
                                                .px(px(16.))
                                                .bg(warning_bg)
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .text_color(gpui::white())
                                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                                        .text_size(px(14.))
                                                        .child(confirm_name)
                                                )
                                                .into_any_element()
                                        } else {
                                            item_element
                                        };

                                        div()
                                            .id(ElementId::NamedInteger(
                                                "script-item".into(),
                                                ix as u64,
                                            ))
                                            .h(px(LIST_ITEM_HEIGHT)) // Explicit 48px height
                                            .on_hover(hover_handler)
                                            .on_click(click_handler)
                                            .child(final_element)
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
            // Most items are LIST_ITEM_HEIGHT (48px), headers are SECTION_HEADER_HEIGHT (24px)
            // Use 44px as a reasonable average that feels natural for scrolling
            let avg_item_height = 44.0_f32;

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
                .max_h(px(120.))
                .font_family("SF Mono");

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
                            this.toggle_logs(cx);
                            return;
                        }
                        "k" => {
                            this.toggle_actions(cx, window);
                            return;
                        }
                        // Cmd+1 cycles through all designs
                        "1" => {
                            this.cycle_design(cx);
                            return;
                        }
                        // Script context shortcuts (require a selected script)
                        "e" => {
                            // Cmd+E - Edit Script
                            this.handle_action("edit_script".to_string(), cx);
                            return;
                        }
                        "f" if has_shift => {
                            // Cmd+Shift+F - Reveal in Finder
                            this.handle_action("reveal_in_finder".to_string(), cx);
                            return;
                        }
                        "c" if has_shift => {
                            // Cmd+Shift+C - Copy Path
                            this.handle_action("copy_path".to_string(), cx);
                            return;
                        }
                        // Global shortcuts
                        "n" => {
                            // Cmd+N - Create Script
                            this.handle_action("create_script".to_string(), cx);
                            return;
                        }
                        "r" => {
                            // Cmd+R - Reload Scripts
                            this.handle_action("reload_scripts".to_string(), cx);
                            return;
                        }
                        "," => {
                            // Cmd+, - Settings
                            this.handle_action("settings".to_string(), cx);
                            return;
                        }
                        "q" => {
                            // Cmd+Q - Quit
                            this.handle_action("quit".to_string(), cx);
                            return;
                        }
                        _ => {}
                    }
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
                            "enter" => {
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
                                        this.show_actions_popup = false;
                                        this.actions_dialog = None;
                                        // Close the actions window
                                        cx.spawn(async move |_this, cx| {
                                            cx.update(close_actions_window).ok();
                                        })
                                        .detach();
                                        this.focus_main_filter(window, cx);
                                    }
                                    this.handle_action(action_id, cx);
                                }
                                // Notify to update UI state after closing popup
                                cx.notify();
                                return;
                            }
                            "escape" => {
                                this.show_actions_popup = false;
                                this.actions_dialog = None;
                                // Close the actions window
                                cx.spawn(async move |_this, cx| {
                                    cx.update(close_actions_window).ok();
                                })
                                .detach();
                                this.focus_main_filter(window, cx);
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
                                // Route character input to the dialog for search
                                if let Some(ref key_char) = event.keystroke.key_char {
                                    if let Some(ch) = key_char.chars().next() {
                                        if !ch.is_control() {
                                            dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                            // Resize and notify actions window to re-render
                                            let dialog_for_resize = dialog.clone();
                                            cx.spawn(async move |_this, cx| {
                                                cx.update(|cx| {
                                                    resize_actions_window(cx, &dialog_for_resize);
                                                })
                                                .ok();
                                            })
                                            .detach();
                                        }
                                    }
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
                match key_str.as_str() {
                    "up" | "arrowup" => {
                        // Input history navigation: Up arrow when filter is empty
                        if this.filter_text.is_empty() {
                            if let Some(text) = this.input_history.navigate_up() {
                                tracing::debug!(entry = %text, "Input history: navigate up");
                                this.filter_text = text.clone();
                                let text_for_input = text.clone();
                                let text_len = text_for_input.len();
                                this.gpui_input_state.update(cx, |state, input_cx| {
                                    state.set_value(text_for_input, window, input_cx);
                                    // Ensure cursor is at end with no selection after programmatic set_value
                                    state.set_selection(text_len, text_len, window, input_cx);
                                });
                                // Reset selection since results will change
                                this.selected_index = 0;
                                this.last_scrolled_index = None;
                                this.invalidate_grouped_cache();
                                cx.notify();
                                return;
                            }
                        }
                        let _key_perf = crate::perf::KeyEventPerfGuard::new();
                        match this.nav_coalescer.record(NavDirection::Up) {
                            NavRecord::ApplyImmediate => this.move_selection_up(cx),
                            NavRecord::Coalesced => {}
                            NavRecord::FlushOld { dir, delta } => {
                                if delta != 0 {
                                    this.apply_nav_delta(dir, delta, cx);
                                }
                                this.move_selection_up(cx);
                            }
                        }
                        this.ensure_nav_flush_task(cx);
                    }
                    "down" | "arrowdown" => {
                        // Input history navigation: Down arrow when navigating history
                        if this.input_history.current_index().is_some() {
                            if let Some(text) = this.input_history.navigate_down() {
                                tracing::debug!(entry = %text, "Input history: navigate down");
                                this.filter_text = text.clone();
                                let text_for_input = text.clone();
                                let text_len = text_for_input.len();
                                this.gpui_input_state.update(cx, |state, input_cx| {
                                    state.set_value(text_for_input, window, input_cx);
                                    // Ensure cursor is at end with no selection after programmatic set_value
                                    state.set_selection(text_len, text_len, window, input_cx);
                                });
                                // Reset selection since results will change
                                this.selected_index = 0;
                                this.last_scrolled_index = None;
                                this.invalidate_grouped_cache();
                                cx.notify();
                                return;
                            } else {
                                // Went past newest entry - clear input and reset to empty state
                                tracing::debug!("Input history: past newest, clearing input");
                                this.input_history.reset_navigation();
                                this.filter_text.clear();
                                this.gpui_input_state.update(cx, |state, input_cx| {
                                    state.set_value(String::new(), window, input_cx);
                                    // Ensure cursor is at start (empty string, so 0..0)
                                    state.set_selection(0, 0, window, input_cx);
                                });
                                // Reset selection since results will change
                                this.selected_index = 0;
                                this.last_scrolled_index = None;
                                this.invalidate_grouped_cache();
                                cx.notify();
                                return;
                            }
                        }
                        let _key_perf = crate::perf::KeyEventPerfGuard::new();
                        match this.nav_coalescer.record(NavDirection::Down) {
                            NavRecord::ApplyImmediate => this.move_selection_down(cx),
                            NavRecord::Coalesced => {}
                            NavRecord::FlushOld { dir, delta } => {
                                if delta != 0 {
                                    this.apply_nav_delta(dir, delta, cx);
                                }
                                this.move_selection_down(cx);
                            }
                        }
                        this.ensure_nav_flush_task(cx);
                    }
                    "enter" => {
                        if !this.gpui_input_focused {
                            this.execute_selected(cx);
                        }
                    }
                    "escape" => {
                        // First check if we have a pending confirmation to clear
                        if this.pending_confirmation.is_some() {
                            logging::log("KEY", "ESC - clearing pending confirmation");
                            this.pending_confirmation = None;
                            cx.notify();
                        } else if !this.filter_text.is_empty() {
                            // Clear filter first if there's text
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
                                // Set input and submit to AI
                                ai::set_ai_input(cx, &query, true);
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
        // Use theme opacity settings for background transparency
        let opacity = self.theme.get_opacity();

        // Use design tokens for background color (or theme for Default design)
        let bg_hex = if is_default_design {
            theme.colors.background.main
        } else {
            design_colors.background
        };

        // Create box shadows from theme
        let box_shadows = self.create_box_shadows();

        // Use design tokens for border radius
        let border_radius = if is_default_design {
            12.0 // Default radius
        } else {
            design_visual.radius_lg
        };

        // Use design tokens for text color
        let text_primary = if is_default_design {
            theme.colors.text.primary
        } else {
            design_colors.text_primary
        };

        // Use design tokens for font family
        let font_family = if is_default_design {
            ".AppleSystemUIFont"
        } else {
            design_typography.font_family
        };

        // Extract footer colors BEFORE render_preview_panel (borrow checker)
        let footer_accent = if is_default_design {
            theme.colors.accent.selected
        } else {
            design_colors.accent
        };
        let footer_text_muted = if is_default_design {
            theme.colors.text.muted
        } else {
            design_colors.text_muted
        };
        let footer_border = if is_default_design {
            theme.colors.ui.border
        } else {
            design_colors.border
        };
        let footer_background = if is_default_design {
            theme.colors.accent.selected_subtle // Match selected item bg
        } else {
            design_colors.background_selected
        };

        // VIBRANCY: Remove background from content div - let gpui-component Root's
        // semi-transparent background handle vibrancy effect. Content areas should NOT
        // have their own backgrounds to allow blur to show through.
        let _bg_with_alpha = crate::ui_foundation::hex_to_rgba_with_opacity(bg_hex, opacity.main);

        let mut main_div = div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .w_full()
            .h_full()
            .rounded(px(border_radius))
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
                let text_muted = if is_default_design {
                    theme.colors.text.muted
                } else {
                    design_colors.text_muted
                };
                let _text_dimmed = if is_default_design {
                    theme.colors.text.dimmed
                } else {
                    design_colors.text_dimmed
                };
                let accent_color = if is_default_design {
                    theme.colors.accent.selected
                } else {
                    design_colors.accent
                };
                let search_box_bg = if is_default_design {
                    theme.colors.background.search_box
                } else {
                    design_colors.background_secondary
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
                    // Search input with cursor and selection support
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(input_height))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    // "Ask AI [Tab]" button - yellow text, grey badge, hover state
                    .child({
                        // Hover background: accent color at 15% opacity
                        let hover_bg = (accent_color << 8) | 0x26;
                        let tab_bg = (search_box_bg << 8) | 0x4D; // 30% opacity
                        div()
                            .id("ask-ai-button")
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.))
                            .px(px(6.))
                            .py(px(4.))
                            .rounded(px(4.))
                            .cursor_pointer()
                            .hover(move |s| s.bg(rgba(hover_bg)))
                            // "Ask AI" text - YELLOW (accent)
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(accent_color))
                                    .child("Ask AI"),
                            )
                            // "Tab" badge - grey background at 30% opacity (no border)
                            .child(
                                div()
                                    .px(px(6.))
                                    .py(px(2.))
                                    .rounded(px(4.))
                                    .bg(rgba(tab_bg))
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child("Tab"),
                            )
                    })
            })
            // Subtle divider - semi-transparent
            // Use design tokens for border color and spacing
            .child({
                let divider_margin = if is_default_design {
                    16.0
                } else {
                    design_spacing.margin_lg
                };
                let border_color = if is_default_design {
                    theme.colors.ui.border
                } else {
                    design_colors.border
                };
                let border_width = if is_default_design {
                    1.0
                } else {
                    design_visual.border_thin
                };

                div()
                    .mx(px(divider_margin))
                    .h(px(border_width))
                    .bg(rgba((border_color << 8) | 0x60))
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
                    .child(
                        div()
                            .relative() // Enable absolute positioning for overlay
                            .w_1_2() // 50% width
                            .h_full() // Take full height
                            .min_h(px(0.)) // Allow shrinking
                            .overflow_hidden()
                            // Preview panel ALWAYS renders
                            // NOTE: Actions dialog is now rendered in a separate popup window
                            // (see actions/window.rs) - no inline overlay needed here
                            .child(self.render_preview_panel(cx)),
                    ),
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
            };

            // Get the primary action label from the selected item
            // Falls back to "Run" if no item selected or not a regular item
            let primary_label = grouped_items
                .get(self.selected_index)
                .and_then(|item| match item {
                    GroupedListItem::Item(idx) => flat_results.get(*idx),
                    GroupedListItem::SectionHeader(_) => None,
                })
                .map(|result| result.get_default_action_text())
                .unwrap_or("Run");

            PromptFooter::new(
                PromptFooterConfig::default().primary_label(primary_label),
                footer_colors,
            )
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

        main_div.into_any_element()
    }
}
