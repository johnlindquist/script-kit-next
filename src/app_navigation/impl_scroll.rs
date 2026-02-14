impl ScriptListApp {
    fn scroll_to_selected_if_needed(&mut self, _reason: &str) {
        let target = self.selected_index;

        // Check if we've already scrolled to this index
        if self.last_scrolled_index == Some(target) {
            return;
        }

        // Use perf guard for scroll timing
        let _scroll_perf = crate::perf::ScrollPerfGuard::new();

        // Perform the scroll using ListState for variable-height list
        // This scrolls the actual list() component used in render_script_list
        self.main_list_state.scroll_to_reveal_item(target);
        self.last_scrolled_index = Some(target);
    }

    /// Trigger scroll activity - shows the scrollbar and schedules fade-out
    ///
    /// This should be called whenever scroll-related activity occurs:
    /// - Keyboard up/down navigation
    /// - scroll_to_item calls
    /// - Mouse wheel scrolling (if tracked)
    fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
        self.is_scrolling = true;
        self.last_scroll_time = Some(std::time::Instant::now());

        // Schedule fade-out after 1000ms of inactivity
        cx.spawn(async move |this, cx| {
            Timer::after(std::time::Duration::from_millis(1000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    // Only hide if no new scroll activity occurred
                    if let Some(last_time) = app.last_scroll_time {
                        if last_time.elapsed() >= std::time::Duration::from_millis(1000) {
                            app.is_scrolling = false;
                            cx.notify();
                        }
                    }
                })
            });
        })
        .detach();

        cx.notify();
    }

    /// Apply a coalesced navigation delta in the given direction
    #[allow(dead_code)]
    fn apply_nav_delta(&mut self, dir: NavDirection, delta: i32, cx: &mut Context<Self>) {
        let signed = match dir {
            NavDirection::Up => -delta,
            NavDirection::Down => delta,
        };
        self.move_selection_by(signed, cx);
    }

    /// Move selection by a signed delta (positive = down, negative = up)
    /// Used by NavCoalescer for batched movements
    ///
    /// IMPORTANT: This must use grouped results and skip section headers,
    /// just like move_selection_up/down. Otherwise, holding arrow keys
    /// can land on headers causing navigation to feel "stuck".
    fn move_selection_by(&mut self, delta: i32, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let selection_update = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            let len = grouped_items.len();

            if len == 0 {
                None
            } else {
                let clamped_index = self.selected_index.min(len.saturating_sub(1));
                let first_selectable = self.cached_grouped_first_selectable_index;
                let last_selectable = self.cached_grouped_last_selectable_index;

                if let (Some(first), Some(last)) = (first_selectable, last_selectable) {
                    let target =
                        (clamped_index as i32 + delta).clamp(first as i32, last as i32) as usize;

                    let new_index = if delta > 0 {
                        let mut idx = target;
                        while idx < last
                            && matches!(
                                grouped_items.get(idx),
                                Some(GroupedListItem::SectionHeader(..))
                            )
                        {
                            idx += 1;
                        }
                        idx
                    } else if delta < 0 {
                        let mut idx = target;
                        while idx > first
                            && matches!(
                                grouped_items.get(idx),
                                Some(GroupedListItem::SectionHeader(..))
                            )
                        {
                            idx -= 1;
                        }
                        idx
                    } else {
                        clamped_index
                    };

                    let resolved_index = if matches!(
                        grouped_items.get(new_index),
                        Some(GroupedListItem::SectionHeader(..))
                    ) {
                        clamped_index
                    } else {
                        new_index
                    };

                    if resolved_index != clamped_index {
                        Some((resolved_index, "coalesced_nav"))
                    } else {
                        Some((clamped_index, "coalesced_nav_clamp"))
                    }
                } else {
                    Some((clamped_index, "coalesced_nav_clamp"))
                }
            }
        };

        if let Some((new_index, reason)) = selection_update {
            self.set_selected_index(new_index, reason, cx);
        } else {
            self.selected_index = 0;
        }
    }

    /// Handle mouse wheel scroll events by converting to item-based scrolling.
    ///
    /// This bypasses GPUI's pixel-based scroll which has height calculation issues
    /// with variable-height items. Instead, we convert the scroll delta to item
    /// indices and use scroll_to_reveal_item() like keyboard navigation does.
    ///
    /// # Arguments
    /// * `delta_lines` - Scroll delta in "lines" (positive = scroll content up/view down)
    #[allow(dead_code)]
    pub fn handle_scroll_wheel(&mut self, delta_lines: f32, cx: &mut Context<Self>) {
        // Compute wheel movement targets while grouped results are borrowed.
        let (current_item, new_item, items_to_scroll) = {
            let current_item = self.main_list_state.logical_scroll_top().item_ix;
            let (grouped_items, _) = self.get_grouped_results_cached();
            let item_count = grouped_items.len();
            let new_item = wheel_scroll_target_index(current_item, item_count, delta_lines);
            let items_to_scroll = (-delta_lines).round() as i32;
            (current_item, new_item, items_to_scroll)
        };

        tracing::debug!(
            target: "SCROLL_STATE",
            delta_lines,
            current_item,
            new_item,
            items_to_scroll,
            "Mouse wheel scroll"
        );

        // Only scroll if we're moving to a different item
        if new_item != current_item {
            self.main_list_state.scroll_to_reveal_item(new_item);
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Synchronize the GPUI list component state with the current grouped results.
    ///
    /// Call this method after any operation that may change the number of items
    /// in the list (filter changes, data refresh, view transitions).
    ///
    /// This method handles:
    /// - Updating the list component's item count via splice()
    /// - Invalidating scroll tracking when structure changes
    ///
    /// Note: This is separate from validate_selection_bounds() which handles
    /// ensuring the selected index is valid.
    pub fn sync_list_state(&mut self) {
        let (grouped_items, _) = self.get_grouped_results_cached();
        let item_count = grouped_items.len();

        let old_list_count = self.main_list_state.item_count();
        if old_list_count != item_count {
            self.main_list_state.splice(0..old_list_count, item_count);
            // Invalidate scroll tracking since list structure changed
            self.last_scrolled_index = None;
            // Restore scroll to selected item to prevent viewport jumping to top.
            // splice(0..old, new) resets GPUI's logical_scroll_top to item 0.
            // Callers that want to reset scroll (filter changes, view resets)
            // will override by calling scroll_to_reveal_item(0) afterward.
            if self.selected_index < item_count {
                self.main_list_state
                    .scroll_to_reveal_item(self.selected_index);
            }
        }
    }

    /// Validate and correct selection bounds after list structure changes.
    ///
    /// Call this method from event handlers after any operation that may change
    /// the number of items in the list (filter changes, data refresh, view transitions).
    ///
    /// This replaces the anti-pattern of mutating selection during render.
    /// By validating in event handlers, render remains a pure function of state.
    ///
    /// # Returns
    /// `true` if selection was changed, `false` if it was already valid.
    pub fn validate_selection_bounds(&mut self, cx: &mut Context<Self>) -> bool {
        enum ValidationState {
            Empty,
            NonEmpty {
                valid_idx: usize,
                has_selectable: bool,
            },
        }

        let validation_state = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            let item_count = grouped_items.len();

            if item_count == 0 {
                ValidationState::Empty
            } else {
                let clamped_index = self.selected_index.min(item_count.saturating_sub(1));
                let has_selectable = self.cached_grouped_first_selectable_index.is_some();
                ValidationState::NonEmpty {
                    valid_idx: validated_selection_index(&grouped_items, clamped_index),
                    has_selectable,
                }
            }
        };

        match validation_state {
            ValidationState::Empty => {
                // Empty list - reset all selection state
                let changed = self.selected_index != 0
                    || self.hovered_index.is_some()
                    || self.last_scrolled_index.is_some();

                self.selected_index = 0;
                self.hovered_index = None;
                self.last_scrolled_index = None;

                // Clear legacy fallback state
                self.fallback_mode = false;
                self.cached_fallbacks.clear();

                if changed {
                    cx.notify();
                }
                changed
            }
            ValidationState::NonEmpty {
                valid_idx,
                has_selectable,
            } => {
                // List has items - coerce selection to a valid selectable item
                self.fallback_mode = false;
                self.cached_fallbacks.clear();

                if valid_idx == 0 && !has_selectable {
                    // No selectable items (list is all headers) - reset to 0
                    if self.selected_index != 0 {
                        self.selected_index = 0;
                        cx.notify();
                        return true;
                    }
                } else if self.selected_index != valid_idx {
                    self.selected_index = valid_idx;
                    cx.notify();
                    return true;
                }

                false
            }
        }
    }

    /// Ensure the navigation flush task is running. Spawns a background task
    /// that periodically flushes pending navigation deltas.
    #[allow(dead_code)]
    fn ensure_nav_flush_task(&mut self, cx: &mut Context<Self>) {
        if self.nav_coalescer.flush_task_running {
            return;
        }
        self.nav_coalescer.flush_task_running = true;
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(NavCoalescer::WINDOW).await;
                let keep_running = cx
                    .update(|cx| {
                        this.update(cx, |this, cx| {
                            // Flush any pending navigation delta
                            if let Some((dir, delta)) = this.nav_coalescer.flush_pending() {
                                this.apply_nav_delta(dir, delta, cx);
                            }
                            // Check if we should keep running
                            let now = std::time::Instant::now();
                            let recently_active = now.duration_since(this.nav_coalescer.last_event)
                                < NavCoalescer::WINDOW;
                            if !recently_active && this.nav_coalescer.pending_delta == 0 {
                                // No recent activity and no pending delta - stop the task
                                this.nav_coalescer.flush_task_running = false;
                                this.nav_coalescer.reset();
                                false
                            } else {
                                true
                            }
                        })
                    })
                    .unwrap_or(Ok(false))
                    .unwrap_or(false);
                if !keep_running {
                    break;
                }
            }
        })
        .detach();
    }
}
