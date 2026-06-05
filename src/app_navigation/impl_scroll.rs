#[inline]
fn main_list_footer_overlay_height() -> gpui::Pixels {
    gpui::px(crate::window_resize::main_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT)
}

#[inline]
fn main_list_footer_reveal_clearance_height() -> gpui::Pixels {
    gpui::px(8.0)
}

pub(crate) fn main_list_footer_overlay_total_padding() -> gpui::Pixels {
    main_list_footer_overlay_height() + main_list_footer_reveal_clearance_height()
}

#[inline]
fn script_list_row_height(item: &GroupedListItem, ix: usize) -> f32 {
    match item {
        GroupedListItem::SectionHeader(..) => {
            if ix == 0 {
                crate::list_item::effective_first_section_header_height()
            } else {
                crate::list_item::effective_section_header_height()
            }
        }
        GroupedListItem::Status(..) => crate::list_item::effective_source_status_row_height(),
        GroupedListItem::Item(..) => crate::list_item::effective_list_item_height(),
    }
}

pub(crate) fn script_list_content_height(items: &[GroupedListItem]) -> f32 {
    items.iter().enumerate().map(|(ix, item)| script_list_row_height(item, ix)).sum()
}

fn script_list_pixel_top_for_item(items: &[GroupedListItem], ix: usize) -> f32 {
    items.iter().take(ix).enumerate().map(|(item_ix, item)| script_list_row_height(item, item_ix)).sum()
}

fn script_list_pixel_top_for_offset(items: &[GroupedListItem], offset: gpui::ListOffset) -> f32 {
    let offset_in_item = offset.offset_in_item.as_f32().max(0.0);
    let clamped_item_ix = offset.item_ix.min(items.len());
    script_list_pixel_top_for_item(items, clamped_item_ix) + offset_in_item
}

fn script_list_offset_for_pixel_top(
    items: &[GroupedListItem],
    scroll_top_px: f32,
) -> gpui::ListOffset {
    if items.is_empty() {
        return gpui::ListOffset {
            item_ix: 0,
            offset_in_item: gpui::px(0.0),
        };
    }

    let mut accumulated = 0.0_f32;
    for (ix, item) in items.iter().enumerate() {
        let item_height = script_list_row_height(item, ix);
        let item_bottom = accumulated + item_height;
        if scroll_top_px < item_bottom {
            return gpui::ListOffset {
                item_ix: ix,
                offset_in_item: gpui::px((scroll_top_px - accumulated).max(0.0)),
            };
        }
        accumulated = item_bottom;
    }

    gpui::ListOffset {
        item_ix: items.len(),
        offset_in_item: gpui::px(0.0),
    }
}

fn footer_safe_scroll_offset_for_item(
    items: &[GroupedListItem],
    current_offset: gpui::ListOffset,
    viewport_height: gpui::Pixels,
    footer_overlay_height: gpui::Pixels,
    target_ix: usize,
) -> Option<gpui::ListOffset> {
    if items.is_empty() || target_ix >= items.len() || viewport_height <= footer_overlay_height {
        return None;
    }

    let viewport_height = viewport_height.as_f32();
    let footer_overlay_height = footer_overlay_height.as_f32();
    let safe_viewport_height = viewport_height - footer_overlay_height;
    let max_scroll_top = (script_list_content_height(items) - safe_viewport_height).max(0.0);
    let current_scroll_top = script_list_pixel_top_for_offset(items, current_offset);
    let target_top = script_list_pixel_top_for_item(items, target_ix);
    let target_bottom = target_top + script_list_row_height(&items[target_ix], target_ix);
    let safe_bottom = current_scroll_top + safe_viewport_height;

    if target_bottom <= safe_bottom {
        return None;
    }

    let safe_scroll_top = (current_scroll_top + (target_bottom - safe_bottom)).min(max_scroll_top);
    Some(script_list_offset_for_pixel_top(items, safe_scroll_top))
}

#[inline]
fn scrollbar_fade_duration() -> std::time::Duration {
    crate::transitions::DURATION_MEDIUM + std::time::Duration::from_millis(50)
}

#[inline]
fn scrollbar_fade_opacity(progress: f32) -> crate::transitions::Opacity {
    use crate::transitions::Lerp;
    let eased = crate::transitions::ease_in_quad(progress.clamp(0.0, 1.0));
    crate::transitions::Opacity::VISIBLE.lerp(&crate::transitions::Opacity::INVISIBLE, eased)
}

impl ScriptListApp {
    pub(crate) fn main_list_scroll_receipt(&mut self) -> serde_json::Value {
        let viewport_height = self.main_list_state.viewport_bounds().size.height;
        let footer_height = main_list_footer_overlay_total_padding();
        let scroll_offset = self.main_list_state.logical_scroll_top();
        let (content_height, selected_row_top, selected_row_bottom, item_count) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            let content_height = script_list_content_height(&grouped_items);
            let selected_row_top = grouped_items
                .get(self.selected_index)
                .map(|_| script_list_pixel_top_for_item(&grouped_items, self.selected_index));
            let selected_row_bottom = grouped_items.get(self.selected_index).map(|item| {
                selected_row_top.unwrap_or(0.0) + script_list_row_height(item, self.selected_index)
            });
            (
                content_height,
                selected_row_top,
                selected_row_bottom,
                grouped_items.len(),
            )
        };
        let scroll_top = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            script_list_pixel_top_for_offset(&grouped_items, scroll_offset)
        };
        let viewport_height_px = viewport_height.as_f32().max(0.0);
        let footer_height_px = footer_height.as_f32().max(0.0);
        let safe_viewport_height = (viewport_height_px - footer_height_px).max(0.0);
        let max_scroll_top = (content_height - safe_viewport_height).max(0.0);
        let selected_row_top_in_view = selected_row_top.map(|top| top - scroll_top);
        let selected_row_bottom_in_view = selected_row_bottom.map(|bottom| bottom - scroll_top);
        let selected_row_visible = selected_row_top_in_view
            .zip(selected_row_bottom_in_view)
            .map(|(top, bottom)| top >= 0.0 && bottom <= viewport_height_px)
            .unwrap_or(false);
        let selected_row_above_footer = selected_row_bottom_in_view
            .map(|bottom| bottom <= safe_viewport_height)
            .unwrap_or(false);

        serde_json::json!({
            "scrollTop": scroll_top,
            "scrollTopItem": scroll_offset.item_ix,
            "scrollTopOffset": scroll_offset.offset_in_item.as_f32(),
            "contentHeight": content_height,
            "viewportHeight": viewport_height_px,
            "footerHeight": footer_height_px,
            "safeViewportHeight": safe_viewport_height,
            "maxScrollTop": max_scroll_top,
            "selectedIndex": self.selected_index,
            "selectedRowTop": selected_row_top_in_view,
            "selectedRowBottom": selected_row_bottom_in_view,
            "selectedRowVisible": selected_row_visible,
            "selectedRowAboveFooter": selected_row_above_footer,
            "itemCount": item_count,
        })
    }

    pub(crate) fn reveal_main_list_selection_above_footer(&mut self, reason: &str) {
        self.scroll_to_selected_if_needed(reason);
    }

    pub(crate) fn schedule_main_list_selection_reveal_above_footer(
        &mut self,
        reason: &'static str,
        cx: &mut Context<Self>,
    ) {
        const ATTEMPTS: usize = 5;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(16);

        self.last_scrolled_index = None;
        cx.spawn(async move |this, cx| {
            for _ in 0..ATTEMPTS {
                cx.background_executor().timer(RETRY_DELAY).await;
                let revealed = cx
                    .update(|cx| {
                        this.update(cx, |app, cx| {
                            let viewport_height = app.main_list_state.viewport_bounds().size.height;
                            if viewport_height <= main_list_footer_overlay_total_padding() {
                                app.last_scrolled_index = None;
                                cx.notify();
                                return false;
                            }

                            app.last_scrolled_index = None;
                            app.reveal_main_list_selection_above_footer(reason);
                            cx.notify();
                            true
                        })
                    })
                    .unwrap_or(false);
                if revealed {
                    break;
                }
            }
        })
        .detach();
    }

    pub(crate) fn sync_main_list_selection_to_visible_window(&mut self, reason: &'static str) {
        if reason == "render" && self.last_scrolled_index.is_none() {
            return;
        }

        let viewport_height = self.main_list_state.viewport_bounds().size.height;
        let safe_height = viewport_height - main_list_footer_overlay_total_padding();
        if safe_height <= gpui::px(0.0) {
            return;
        }

        let (grouped_items, _) = self.get_grouped_results_cached();
        let scroll_top = self.main_list_state.logical_scroll_top();
        let Some(target) = crate::scrolling::selection_owned::reanchor_grouped_selection(
            &grouped_items,
            self.selected_index,
            scroll_top,
            safe_height,
        ) else {
            return;
        };

        tracing::info!(
            target: "script_kit::scroll",
            event = "launcher_selection_resynced_from_scrollbar",
            reason,
            selected_before = self.selected_index,
            selected_after = target,
            scroll_top_item_ix = scroll_top.item_ix,
        );
        self.selected_index = target;
        self.last_scrolled_index = Some(target);
    }

    fn adjust_selected_item_above_footer_overlay(&mut self, target: usize) {
        let viewport_height = self.main_list_state.viewport_bounds().size.height;
        if viewport_height <= gpui::px(0.0) {
            return;
        }

        let adjusted_scroll_offset = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            footer_safe_scroll_offset_for_item(
                &grouped_items,
                self.main_list_state.logical_scroll_top(),
                viewport_height,
                main_list_footer_overlay_total_padding(),
                target,
            )
        };

        if let Some(scroll_offset) = adjusted_scroll_offset {
            self.main_list_state.scroll_to(scroll_offset);
        }
    }

    fn scroll_to_selected_if_needed(&mut self, reason: &str) {
        let target = self.selected_index;

        // Check if we've already scrolled to this index
        if self.last_scrolled_index == Some(target) {
            tracing::trace!(
                target: "SCROLL_STATE",
                reason,
                target,
                "skip scroll reveal; target already revealed"
            );
            return;
        }

        let before_top = self.main_list_state.logical_scroll_top().item_ix;

        // Use perf guard for scroll timing
        let _scroll_perf = crate::perf::ScrollPerfGuard::new();

        // Perform the scroll using ListState for variable-height list
        // This scrolls the actual list() component used in render_script_list
        self.main_list_state.scroll_to_reveal_item(target);
        self.adjust_selected_item_above_footer_overlay(target);
        if self.main_list_state.viewport_bounds().size.height
            > main_list_footer_overlay_total_padding()
        {
            self.last_scrolled_index = Some(target);
        } else {
            self.last_scrolled_index = None;
        }

        let after_top = self.main_list_state.logical_scroll_top().item_ix;

        tracing::debug!(
            target: "SCROLL_STATE",
            reason,
            target,
            before_top,
            after_top,
            "revealed selected item"
        );
    }

    /// Trigger scroll activity - shows the scrollbar and schedules fade-out
    ///
    /// This should be called whenever scroll-related activity occurs:
    /// - Keyboard up/down navigation
    /// - scroll_to_item calls
    /// - Mouse wheel scrolling (if tracked)
    fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
        const SCROLLBAR_IDLE_DELAY: std::time::Duration = std::time::Duration::from_millis(1000);
        const SCROLLBAR_FADE_TICK: std::time::Duration = std::time::Duration::from_millis(16);

        let now = std::time::Instant::now();
        self.last_scroll_time = Some(now);
        self.scrollbar_visibility = crate::transitions::Opacity::VISIBLE;
        self.scrollbar_fade_gen = self.scrollbar_fade_gen.wrapping_add(1);
        let fade_gen = self.scrollbar_fade_gen;

        tracing::debug!(
            target: "SCROLL_STATE",
            fade_gen,
            "Scrollbar activity detected; scheduling fade-out"
        );

        // Schedule fade-out after 1000ms of inactivity
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SCROLLBAR_IDLE_DELAY).await;

            let should_start_fade = cx
                .update(|cx| {
                    this.update(cx, |app, _cx| {
                        if app.scrollbar_fade_gen != fade_gen {
                            return false;
                        }

                        app.last_scroll_time
                            .map(|last_time| last_time.elapsed() >= SCROLLBAR_IDLE_DELAY)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if !should_start_fade {
                tracing::trace!(
                    target: "SCROLL_STATE",
                    fade_gen,
                    "Skipping scrollbar fade due to newer activity"
                );
                return;
            }

            let fade_duration = scrollbar_fade_duration();
            let fade_start = std::time::Instant::now();

            loop {
                let elapsed = fade_start.elapsed();
                let t = (elapsed.as_secs_f32() / fade_duration.as_secs_f32()).clamp(0.0, 1.0);
                let opacity = scrollbar_fade_opacity(t);

                let continue_fade = cx
                    .update(|cx| {
                        this.update(cx, |app, cx| {
                            if app.scrollbar_fade_gen != fade_gen {
                                return false;
                            }

                            app.scrollbar_visibility = opacity;
                            cx.notify();
                            t < 1.0
                        })
                    })
                    .unwrap_or(false);

                if !continue_fade {
                    break;
                }

                cx.background_executor().timer(SCROLLBAR_FADE_TICK).await;
            }
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
                let first_selectable = self.main_menu_result_caches.first_selectable_index();
                let last_selectable = self.main_menu_result_caches.last_selectable_index();

                if let (Some(first), Some(last)) = (first_selectable, last_selectable) {
                    let target =
                        (clamped_index as i32 + delta).clamp(first as i32, last as i32) as usize;

                    let new_index = if delta > 0 {
                        let mut idx = target;
                        while idx < last
                            && matches!(
                                grouped_items.get(idx),
                                Some(
                                    GroupedListItem::SectionHeader(..)
                                        | GroupedListItem::Status(..)
                                )
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
                                Some(
                                    GroupedListItem::SectionHeader(..)
                                        | GroupedListItem::Status(..)
                                )
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
                        Some(GroupedListItem::SectionHeader(..) | GroupedListItem::Status(..))
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
        }

        // Always invalidate reveal cache: filtering can replace every visible
        // row while preserving the same count, so the selected item can end up
        // offscreen even when item_count is unchanged.
        self.last_scrolled_index = None;

        tracing::debug!(
            target: "SCROLL_STATE",
            old_list_count,
            item_count,
            selected_index = self.selected_index,
            "synced list state"
        );

        if self.selected_index < item_count {
            self.main_list_state
                .scroll_to_reveal_item(self.selected_index);
            self.adjust_selected_item_above_footer_overlay(self.selected_index);
        }
    }

    /// Force GPUI's measured list items to be rebuilt for same-count row replacements.
    ///
    /// Filter-history recalls can replace every row while preserving the same
    /// item count. `sync_list_state` keeps that path cheap for ordinary syncs,
    /// so filter changes replace the list state identity to avoid stale
    /// measured row elements being painted under fresh footer/preflight state.
    pub fn sync_list_state_for_filter_replacement(&mut self) {
        let (grouped_items, _) = self.get_grouped_results_cached();
        let item_count = grouped_items.len();
        let old_list_count = self.main_list_state.item_count();

        self.last_scrolled_index = None;

        if old_list_count != item_count {
            self.main_list_state.splice(0..old_list_count, item_count);

            if crate::logging::filter_perf_trace_enabled() {
                tracing::info!(
                    target: "SCROLL_STATE",
                    old_list_count,
                    item_count,
                    selected_index = self.selected_index,
                    "spliced list state for filter replacement"
                );
            }

            return;
        }

        if item_count == 0 {
            if crate::logging::filter_perf_trace_enabled() {
                tracing::info!(
                    target: "SCROLL_STATE",
                    old_list_count,
                    item_count,
                    selected_index = self.selected_index,
                    "skipped empty list state replacement for filter replacement"
                );
            }

            return;
        }

        self.main_list_row_generation = self.main_list_row_generation.wrapping_add(1);
        self.main_list_state = ListState::new(
            item_count,
            ListAlignment::Top,
            px(crate::list_item::effective_average_item_height_for_scroll()),
        );

        if crate::logging::filter_perf_trace_enabled() {
            tracing::info!(
                target: "SCROLL_STATE",
                old_list_count,
                item_count,
                selected_index = self.selected_index,
                row_generation = self.main_list_row_generation,
                "replaced list state for filter replacement"
            );
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
                let has_selectable = self.main_menu_result_caches.has_selectable_grouped_item();
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

                self.main_menu_fallback_state.clear();

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
                self.main_menu_fallback_state.clear();

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
                cx.background_executor().timer(NavCoalescer::WINDOW).await;
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
mod scroll_fade_tests {
    use super::{
        footer_safe_scroll_offset_for_item, scrollbar_fade_duration, scrollbar_fade_opacity,
    };
    use crate::list_item::GroupedListItem;

    #[test]
    fn test_scrollbar_fade_duration_does_match_medium_plus_50ms_when_computed() {
        assert_eq!(
            scrollbar_fade_duration(),
            crate::transitions::DURATION_MEDIUM + std::time::Duration::from_millis(50)
        );
    }

    #[test]
    fn test_scrollbar_fade_opacity_does_stay_visible_when_progress_is_zero() {
        assert_eq!(
            scrollbar_fade_opacity(0.0),
            crate::transitions::Opacity::VISIBLE
        );
    }

    #[test]
    fn test_scrollbar_fade_opacity_does_turn_invisible_when_progress_is_one() {
        assert_eq!(
            scrollbar_fade_opacity(1.0),
            crate::transitions::Opacity::INVISIBLE
        );
    }

    #[test]
    fn test_scrollbar_fade_opacity_does_use_ease_in_curve_when_progress_is_midpoint() {
        let midpoint = scrollbar_fade_opacity(0.5).value();
        assert!((midpoint - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_footer_safe_scroll_offset_moves_selected_row_above_overlay() {
        let rows = vec![
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
            GroupedListItem::Item(2),
            GroupedListItem::Item(3),
            GroupedListItem::Item(4),
            GroupedListItem::Item(5),
            GroupedListItem::Item(6),
            GroupedListItem::Item(7),
            GroupedListItem::Item(8),
        ];

        let adjusted = footer_safe_scroll_offset_for_item(
            &rows,
            gpui::ListOffset {
                item_ix: 0,
                offset_in_item: gpui::px(0.0),
            },
            gpui::px(360.0),
            gpui::px(30.0),
            8,
        )
        .expect("target should be pushed above the footer overlay");

        assert_eq!(adjusted.item_ix, 0);
        assert_eq!(adjusted.offset_in_item, gpui::px(30.0));
    }

    #[test]
    fn test_footer_safe_scroll_offset_allows_trailing_scroll_budget_for_last_row() {
        let rows = vec![
            GroupedListItem::Item(0),
            GroupedListItem::Item(1),
            GroupedListItem::Item(2),
            GroupedListItem::Item(3),
            GroupedListItem::Item(4),
            GroupedListItem::Item(5),
            GroupedListItem::Item(6),
            GroupedListItem::Item(7),
            GroupedListItem::Item(8),
        ];

        let adjusted = footer_safe_scroll_offset_for_item(
            &rows,
            gpui::ListOffset {
                item_ix: 0,
                offset_in_item: gpui::px(0.0),
            },
            gpui::px(360.0),
            gpui::px(30.0),
            8,
        )
        .expect("last row should get the extra footer-height trailing scroll budget");

        assert_eq!(adjusted.item_ix, 0);
        assert_eq!(adjusted.offset_in_item, gpui::px(30.0));
    }
}
