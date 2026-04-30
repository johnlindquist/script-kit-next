use crate::ui_foundation::{is_key_down, is_key_enter, is_key_escape, is_key_space, is_key_up};

impl ScriptListApp {
    /// Available vibrancy material presets for the theme customizer
    const VIBRANCY_MATERIALS: &[(theme::VibrancyMaterial, &str)] = &[
        (theme::VibrancyMaterial::Hud, "HUD"),
        (theme::VibrancyMaterial::Popover, "Popover"),
        (theme::VibrancyMaterial::Menu, "Menu"),
        (theme::VibrancyMaterial::Sidebar, "Sidebar"),
        (theme::VibrancyMaterial::Content, "Content"),
    ];

    /// Available font size presets for the theme customizer
    const FONT_SIZE_PRESETS: &[(f32, &str)] = &[
        (12.0, "12"),
        (13.0, "13"),
        (14.0, "14"),
        (15.0, "15"),
        (16.0, "16"),
        (18.0, "18"),
        (20.0, "20"),
    ];

    /// Find the index of a vibrancy material in the presets array
    fn find_vibrancy_material_index(material: theme::VibrancyMaterial) -> usize {
        Self::VIBRANCY_MATERIALS
            .iter()
            .position(|(m, _)| *m == material)
            .unwrap_or(0)
    }

    /// Return a human-readable name for a hex accent color
    fn accent_color_name(color: u32) -> &'static str {
        theme::accent_color_name(color)
    }

    /// Emit a structured scroll log line for builtin views.
    #[allow(clippy::too_many_arguments)]
    fn log_builtin_scroll_event(
        view: &'static str,
        action: &'static str,
        reason: &'static str,
        item_count: usize,
        selected_index: Option<usize>,
        target_item: Option<usize>,
        filter: Option<&str>,
        input_mode: &'static str,
    ) {
        tracing::debug!(
            target: "script_kit::scroll",
            view = view,
            action = action,
            reason = reason,
            item_count = item_count,
            selected_index = selected_index
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into())
                .as_str(),
            target_item = target_item
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".into())
                .as_str(),
            filter_len = filter
                .map(|v| v.chars().count().to_string())
                .unwrap_or_else(|| "none".into())
                .as_str(),
            input_mode = input_mode,
        );
    }

    /// Scroll a builtin uniform list to the top and emit a structured log.
    fn scroll_builtin_to_top_with_log(
        handle: &UniformListScrollHandle,
        view: &'static str,
        item_count: usize,
        filter: &str,
        input_mode: &'static str,
    ) {
        Self::log_builtin_scroll_event(
            view,
            "scroll_to_item",
            "filter_changed",
            item_count,
            Some(0),
            Some(0),
            Some(filter),
            input_mode,
        );
        handle.scroll_to_item(0, ScrollStrategy::Top);
    }

    /// Convert a wheel or trackpad event into whole-row builtin list steps.
    fn builtin_scroll_wheel_steps(&mut self, event: &gpui::ScrollWheelEvent) -> i32 {
        let avg_item_height = crate::list_item::effective_average_item_height_for_scroll();
        let delta_lines: f32 = match event.delta {
            gpui::ScrollDelta::Lines(point) => point.y,
            gpui::ScrollDelta::Pixels(point) => {
                let pixels: f32 = point.y.into();
                pixels / avg_item_height
            }
        };

        self.wheel_accum += -delta_lines;
        let steps = self.wheel_accum.trunc() as i32;
        if steps != 0 {
            self.wheel_accum -= steps as f32;
        }
        steps
    }

    /// Resolve the next selected row for builtin browsers from a wheel event.
    fn builtin_scroll_target_from_wheel(
        &mut self,
        event: &gpui::ScrollWheelEvent,
        current_selected: usize,
        item_count: usize,
    ) -> Option<usize> {
        if item_count == 0 {
            self.wheel_accum = 0.0;
            return None;
        }

        let steps = self.builtin_scroll_wheel_steps(event);
        if steps == 0 {
            return None;
        }

        let max_index = item_count.saturating_sub(1) as i32;
        Some((current_selected as i32 + steps).clamp(0, max_index) as usize)
    }

    /// Compute scrollbar metrics for a tracked uniform list.
    fn builtin_uniform_list_scrollbar_metrics(
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> Option<(usize, usize, Option<f32>)> {
        if total_items == 0 {
            return None;
        }

        let state = handle.0.borrow();
        let scroll_offset = crate::components::scrollbar::preferred_scroll_offset(
            state.base_handle.logical_scroll_top().0,
            state
                .deferred_scroll_to_item
                .map(|deferred| deferred.item_index),
            state.last_item_size.is_some(),
            total_items,
        );

        let fallback_visible_items = fallback_visible_items.max(1).min(total_items);

        if let Some(item_size) = state.last_item_size {
            let viewport_height = item_size.item.height.as_f32().max(0.0);
            let content_height = item_size.contents.height.as_f32().max(0.0);
            let visible_items = if content_height > 0.0 {
                ((viewport_height / content_height) * total_items as f32)
                    .ceil()
                    .max(1.0) as usize
            } else {
                fallback_visible_items
            };

            Some((
                scroll_offset,
                visible_items.clamp(1, total_items),
                Some(viewport_height),
            ))
        } else {
            Some((scroll_offset, fallback_visible_items, None))
        }
    }

    fn builtin_reanchor_selection_from_scroll(
        current_selected: usize,
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> Option<usize> {
        let (first_visible, visible_items, _) =
            Self::builtin_uniform_list_scrollbar_metrics(handle, total_items, fallback_visible_items)?;
        crate::scrolling::selection_owned::reanchor_uniform_selection(
            current_selected,
            first_visible,
            visible_items,
            total_items,
        )
    }

    fn builtin_reanchor_selection_from_scroll_handle(
        current_selected: usize,
        handle: &gpui::ScrollHandle,
        total_items: usize,
    ) -> Option<usize> {
        if total_items == 0 || handle.children_count() == 0 {
            return None;
        }

        let first_visible = handle.logical_scroll_top().0.min(total_items.saturating_sub(1));
        let last_visible = handle.bottom_item().min(total_items.saturating_sub(1));
        let visible_items = last_visible.saturating_sub(first_visible) + 1;

        crate::scrolling::selection_owned::reanchor_uniform_selection(
            current_selected,
            first_visible,
            visible_items,
            total_items,
        )
    }

    /// Build a vendor scrollbar bound to the tracked uniform-list handle.
    fn builtin_uniform_list_scrollbar(
        &self,
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> AnyElement {
        if Self::builtin_uniform_list_scrollbar_metrics(handle, total_items, fallback_visible_items)
            .is_none()
        {
            return div().into_any_element();
        }
        gpui_component::scroll::Scrollbar::vertical(handle)
            .scrollbar_show(gpui_component::scroll::ScrollbarShow::Always)
            .into_any_element()
    }
}

#[cfg(test)]
mod builtin_scroll_helpers_contract {
    const SOURCE: &str = include_str!("common.rs");

    #[test]
    fn builtin_helpers_include_wheel_delta_conversion() {
        assert!(
            SOURCE.contains("fn builtin_scroll_wheel_steps"),
            "builtin scroll helpers should convert raw wheel deltas into row steps"
        );
        assert!(
            SOURCE.contains("gpui::ScrollDelta::Pixels(point)"),
            "builtin wheel helpers should normalize pixel deltas for trackpads"
        );
        assert!(
            SOURCE.contains("fn builtin_scroll_target_from_wheel"),
            "builtin scroll helpers should expose a reusable wheel-to-selection target"
        );
        assert!(
            SOURCE.contains("fn builtin_reanchor_selection_from_scroll_handle"),
            "builtin scroll helpers should support ScrollHandle-based selection reanchor"
        );
    }

    #[test]
    fn builtin_uniform_list_scrollbar_uses_vendor_handle_path() {
        assert!(
            SOURCE.contains("gpui_component::scroll::Scrollbar::vertical(handle)"),
            "builtin uniform list scrollbars should be the GPUI vendor scrollbar bound to the real handle"
        );
        assert!(
            SOURCE.contains(".scrollbar_show(gpui_component::scroll::ScrollbarShow::Always)"),
            "builtin uniform list scrollbars should stay visible for launcher-family surfaces"
        );
    }
}
