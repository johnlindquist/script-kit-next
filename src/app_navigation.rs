// App navigation methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: move_selection_up, move_selection_down, scroll_to_selected, etc.

#[inline]
fn page_down_target_index(
    grouped_items: &[GroupedListItem],
    selected_index: usize,
    page_size: usize,
) -> usize {
    let Some(last_selectable) = grouped_items
        .iter()
        .rposition(|item| matches!(item, GroupedListItem::Item(_)))
    else {
        return selected_index;
    };

    if selected_index >= last_selectable {
        return selected_index;
    }

    let mut remaining = page_size;
    let mut target = selected_index;
    for i in (selected_index + 1)..=last_selectable {
        if matches!(grouped_items.get(i), Some(GroupedListItem::Item(_))) {
            target = i;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }

    target
}

#[inline]
fn wheel_scroll_target_index(current_item: usize, item_count: usize, delta_lines: f32) -> usize {
    if item_count == 0 {
        return 0;
    }

    let max_item = item_count.saturating_sub(1);
    let items_to_scroll = (-delta_lines).round() as i32;
    (current_item as i32 + items_to_scroll).clamp(0, max_item as i32) as usize
}

#[inline]
fn validated_selection_index(grouped_items: &[GroupedListItem], selected_index: usize) -> usize {
    list_item::coerce_selection(grouped_items, selected_index).unwrap_or(0)
}

impl ScriptListApp {
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        // Switch to keyboard mode and clear hover to prevent dual-highlight
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        // Find the first selectable (non-header) item index
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or before first selectable, wrap around to the last selectable item
        if let Some(first) = first_selectable {
            if self.selected_index <= first {
                // Wrap around to the last selectable item
                let last_selectable = grouped_items
                    .iter()
                    .rposition(|item| matches!(item, GroupedListItem::Item(_)));
                if let Some(last) = last_selectable {
                    self.selected_index = last;
                    self.scroll_to_selected_if_needed("keyboard_up_wrap");
                    self.trigger_scroll_activity(cx);
                    cx.notify();
                }
                return;
            }
        }

        if self.selected_index > 0 {
            let mut new_index = self.selected_index - 1;

            // Skip section headers when moving up
            while new_index > 0 {
                if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                    new_index -= 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at index 0
            if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_up");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        // Switch to keyboard mode and clear hover to prevent dual-highlight
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let item_count = grouped_items.len();

        // Find the last selectable (non-header) item index
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or after last selectable, wrap around to the first selectable item
        if let Some(last) = last_selectable {
            if self.selected_index >= last {
                // Wrap around to the first selectable item
                let first_selectable = grouped_items
                    .iter()
                    .position(|item| matches!(item, GroupedListItem::Item(_)));
                if let Some(first) = first_selectable {
                    self.selected_index = first;
                    self.scroll_to_selected_if_needed("keyboard_down_wrap");
                    self.trigger_scroll_activity(cx);
                    cx.notify();
                }
                return;
            }
        }

        if self.selected_index < item_count.saturating_sub(1) {
            let mut new_index = self.selected_index + 1;

            // Skip section headers when moving down
            while new_index < item_count.saturating_sub(1) {
                if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                    new_index += 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at the end
            if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_down");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Jump to the first selectable (non-header) item in the list
    fn move_selection_to_first(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        if let Some(first) = first_selectable {
            if self.selected_index != first {
                self.selected_index = first;
                self.scroll_to_selected_if_needed("jump_first");
                self.trigger_scroll_activity(cx);
                cx.notify();
            }
        }
    }

    /// Move selection up by approximately one page (~10 selectable items)
    /// Skips section headers and clamps to the first selectable item
    fn move_selection_page_up(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        let Some(first) = first_selectable else {
            return;
        };

        // Already at or before first selectable → no-op (don't wrap)
        if self.selected_index <= first {
            return;
        }

        // Count ~10 selectable items upward from current position
        const PAGE_SIZE: usize = 10;
        let mut remaining = PAGE_SIZE;
        let mut target = self.selected_index;
        for i in (first..self.selected_index).rev() {
            if matches!(grouped_items.get(i), Some(GroupedListItem::Item(_))) {
                target = i;
                remaining -= 1;
                if remaining == 0 {
                    break;
                }
            }
        }

        if target != self.selected_index {
            self.selected_index = target;
            self.scroll_to_selected_if_needed("page_up");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Move selection down by approximately one page (~10 selectable items)
    /// Skips section headers and clamps to the last selectable item
    fn move_selection_page_down(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        let Some(last) = last_selectable else {
            return;
        };

        // Already at or after last selectable → no-op (don't wrap)
        if self.selected_index >= last {
            return;
        }

        // Count ~10 selectable items downward from current position
        const PAGE_SIZE: usize = 10;
        let target = page_down_target_index(&grouped_items, self.selected_index, PAGE_SIZE);

        if target != self.selected_index {
            self.selected_index = target;
            self.scroll_to_selected_if_needed("page_down");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Jump to the last selectable (non-header) item in the list
    fn move_selection_to_last(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        if let Some(last) = last_selectable {
            if self.selected_index != last {
                self.selected_index = last;
                self.scroll_to_selected_if_needed("jump_last");
                self.trigger_scroll_activity(cx);
                cx.notify();
            }
        }
    }

    /// Scroll stabilization helper: only call scroll_to_reveal_item if we haven't already scrolled to this index.
    /// This prevents scroll jitter from redundant scroll calls.
    ///
    /// NOTE: Uses main_list_state (ListState) for the variable-height list() component,
    /// not the legacy list_scroll_handle (UniformListScrollHandle).
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
        // Switch to keyboard mode and clear hover to prevent dual-highlight
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let len = grouped_items.len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }

        // Find bounds for selectable items (non-headers)
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If no selectable items, nothing to do
        let (first, last) = match (first_selectable, last_selectable) {
            (Some(f), Some(l)) => (f, l),
            _ => return,
        };

        // Calculate target index, clamping to valid range
        let target = (self.selected_index as i32 + delta).clamp(first as i32, last as i32) as usize;

        // If moving down (positive delta), skip headers forward
        // If moving up (negative delta), skip headers backward
        let new_index = if delta > 0 {
            // Moving down - find next non-header at or after target
            let mut idx = target;
            while idx <= last {
                if matches!(grouped_items.get(idx), Some(GroupedListItem::Item(_))) {
                    break;
                }
                idx += 1;
            }
            idx.min(last)
        } else if delta < 0 {
            // Moving up - find next non-header at or before target
            let mut idx = target;
            while idx >= first {
                if matches!(grouped_items.get(idx), Some(GroupedListItem::Item(_))) {
                    break;
                }
                if idx == 0 {
                    break;
                }
                idx -= 1;
            }
            idx.max(first)
        } else {
            // delta == 0, no movement
            self.selected_index
        };

        // Final validation: ensure we're not on a header
        if matches!(
            grouped_items.get(new_index),
            Some(GroupedListItem::SectionHeader(..))
        ) {
            // Can't find a valid position, stay put
            return;
        }

        if new_index != self.selected_index {
            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("coalesced_nav");
            self.trigger_scroll_activity(cx);
            cx.notify();
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
        // Get current scroll position from ListState
        let current_item = self.main_list_state.logical_scroll_top().item_ix;

        // Get grouped results to find valid bounds
        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let item_count = grouped_items.len();

        // Convert delta to a clamped item index. Rounding acts as lightweight coalescing
        // so tiny high-frequency deltas do not trigger noisy per-event moves.
        let new_item = wheel_scroll_target_index(current_item, item_count, delta_lines);
        let items_to_scroll = (-delta_lines).round() as i32;

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
        // Get grouped results to validate against
        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let item_count = grouped_items.len();

        if item_count == 0 {
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
            return changed;
        }

        // List has items - coerce selection to a valid selectable item
        self.fallback_mode = false;
        self.cached_fallbacks.clear();

        let valid_idx = validated_selection_index(&grouped_items, self.selected_index);
        if valid_idx == 0
            && !grouped_items
                .iter()
                .any(|item| matches!(item, GroupedListItem::Item(_)))
        {
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

#[cfg(test)]
mod tests {
    use super::{page_down_target_index, validated_selection_index, wheel_scroll_target_index};
    use crate::list_item::GroupedListItem;

    #[test]
    fn test_move_selection_page_down_clamps_to_last_item() {
        let rows = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
            GroupedListItem::SectionHeader("Main".to_string(), None),
            GroupedListItem::Item(2),
        ];

        assert_eq!(page_down_target_index(&rows, 1, 10), 4);
        assert_eq!(page_down_target_index(&rows, 4, 10), 4);
    }

    #[test]
    fn test_handle_scroll_wheel_coalesces_rapid_deltas() {
        let item_count = 20;
        let start = 7;

        let after_first = wheel_scroll_target_index(start, item_count, 0.2);
        let after_second = wheel_scroll_target_index(after_first, item_count, 0.2);
        let after_third = wheel_scroll_target_index(after_second, item_count, -0.2);

        assert_eq!(after_first, start);
        assert_eq!(after_second, start);
        assert_eq!(after_third, start);
    }

    #[test]
    fn test_validate_selection_bounds_recovers_from_out_of_range_index() {
        let rows = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), None),
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
            GroupedListItem::SectionHeader("Main".to_string(), None),
            GroupedListItem::Item(2),
        ];
        let headers_only = vec![
            GroupedListItem::SectionHeader("A".to_string(), None),
            GroupedListItem::SectionHeader("B".to_string(), None),
        ];

        assert_eq!(validated_selection_index(&rows, 999), 4);
        assert_eq!(validated_selection_index(&rows, 0), 1);
        assert_eq!(validated_selection_index(&headers_only, 4), 0);
    }
}
