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

    pub(crate) fn theme_font_family(&self) -> String {
        crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design)
            .primary_font()
            .to_string()
    }

    pub(crate) fn theme_font_size_xl(&self) -> f32 {
        crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design)
            .font_size_xl()
    }

    pub(crate) fn render_search_input(&self) -> gpui_component::input::Input {
        let search = self.current_main_menu_theme.def().search;
        let input_font_size = search.font_size;
        gpui_component::input::Input::new(&self.gpui_input_state)
            .w_full()
            .h(gpui::px(search.height))
            .line_height(gpui::px(search.height))
            .px(gpui::px(0.))
            .py(gpui::px(0.))
            .with_size(gpui_component::Size::Size(gpui::px(input_font_size)))
            .appearance(false)
            .bordered(false)
            .focus_bordered(false)
    }

    pub(crate) fn render_search_input_with_ghost(&self, _cx: &gpui::Context<Self>) -> gpui::Div {
        gpui::div().w_full().child(self.render_search_input())
    }

    pub(crate) fn render_builtin_main_input_count_label(
        &self,
        label: impl Into<gpui::SharedString>,
    ) -> gpui::AnyElement {
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        gpui::div()
            .flex_none()
            .whitespace_nowrap()
            .pr(gpui::px(self.current_main_menu_theme.def().search.text_inset_x))
            .text_sm()
            .text_color(gpui::rgba(chrome.text_hint_rgba))
            .child(label.into())
            .into_any_element()
    }

    pub(crate) fn render_builtin_main_input_shell(
        &self,
        trailing: Vec<gpui::AnyElement>,
    ) -> gpui::AnyElement {
        let menu_def = self.current_main_menu_theme.def();
        crate::components::main_view_chrome::render_main_view_input_shell(
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewInputChrome {
                body: self.render_search_input().into_any_element(),
                trailing,
            },
        )
    }

    pub(crate) fn render_builtin_main_input_header(
        &self,
        trailing: Vec<gpui::AnyElement>,
    ) -> crate::components::main_view_chrome::MainViewHeaderChrome {
        let shell = self.current_main_menu_theme.def().shell;
        crate::components::main_view_chrome::MainViewHeaderChrome {
            context: None,
            input: self.render_builtin_main_input_shell(trailing),
            padding_x: shell.header_padding_x,
            padding_y: shell.header_padding_y,
            gap: shell.header_gap,
        }
    }

    pub(crate) fn render_builtin_main_input_surface(
        &self,
        key_context: &'static str,
        trailing: Vec<gpui::AnyElement>,
        main: gpui::AnyElement,
        footer: Option<gpui::AnyElement>,
    ) -> gpui::AnyElement {
        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        crate::components::main_view_chrome::render_main_view_chrome(
            crate::components::main_view_chrome::render_main_view_shell()
                .text_color(gpui::rgb(chrome.text_primary_hex))
                .font_family(self.theme_font_family())
                .key_context(key_context)
                .track_focus(&self.focus_handle),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(trailing),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main,
                footer,
                overlays: Vec::new(),
            },
        )
    }

    pub(crate) fn render_generic_filterable_search_surface(
        &self,
        key_context: &'static str,
        count_label: String,
        list_element: gpui::AnyElement,
        footer: Option<gpui::AnyElement>,
    ) -> gpui::AnyElement {
        let content = gpui::div()
            .flex_1()
            .min_h(gpui::px(0.))
            .w_full()
            .overflow_hidden()
            .child(list_element);

        self.render_builtin_main_input_surface(
            key_context,
            vec![self.render_builtin_main_input_count_label(count_label)],
            content.into_any_element(),
            footer,
        )
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
        let (delta_kind, raw_delta_y, delta_lines): (&'static str, f32, f32) = match event.delta {
            gpui::ScrollDelta::Lines(point) => ("lines", point.y, point.y),
            gpui::ScrollDelta::Pixels(point) => {
                let pixels: f32 = point.y.into();
                ("pixels", pixels, pixels / avg_item_height)
            }
        };

        self.wheel_accum += -delta_lines;
        let steps = self.wheel_accum.trunc() as i32;
        if steps != 0 {
            self.wheel_accum -= steps as f32;
        }
        tracing::info!(
            target: "script_kit::scroll_trace",
            event = "SCROLL_TRACE wheel_steps",
            delta_kind,
            raw_delta_y,
            delta_lines,
            avg_item_height,
            steps,
            wheel_accum_after = self.wheel_accum,
            "SCROLL_TRACE wheel_steps"
        );
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
            tracing::info!(
                target: "script_kit::scroll_trace",
                event = "SCROLL_TRACE wheel_target.empty",
                current_selected,
                item_count,
                "SCROLL_TRACE wheel_target.empty"
            );
            return None;
        }

        let steps = self.builtin_scroll_wheel_steps(event);
        if steps == 0 {
            tracing::info!(
                target: "script_kit::scroll_trace",
                event = "SCROLL_TRACE wheel_target.no_whole_step",
                current_selected,
                item_count,
                wheel_accum = self.wheel_accum,
                "SCROLL_TRACE wheel_target.no_whole_step"
            );
            return None;
        }

        let max_index = item_count.saturating_sub(1) as i32;
        let target = (current_selected as i32 + steps).clamp(0, max_index) as usize;
        tracing::info!(
            target: "script_kit::scroll_trace",
            event = "SCROLL_TRACE wheel_target.result",
            current_selected,
            item_count,
            steps,
            target,
            "SCROLL_TRACE wheel_target.result"
        );
        Some(target)
    }

    /// Compute scrollbar metrics for a tracked uniform list.
    fn builtin_uniform_list_scrollbar_metrics(
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> Option<(usize, usize, Option<f32>)> {
        if total_items == 0 {
            tracing::info!(
                target: "script_kit::scroll_trace",
                event = "SCROLL_TRACE metrics.empty",
                total_items,
                fallback_visible_items,
                "SCROLL_TRACE metrics.empty"
            );
            return None;
        }

        let state = handle.0.borrow();
        let live_scroll_top = state.base_handle.logical_scroll_top().0;
        let deferred_item_index = state
            .deferred_scroll_to_item
            .map(|deferred| deferred.item_index);
        let has_item_size = state.last_item_size.is_some();
        let scroll_offset = crate::components::scrollbar::preferred_scroll_offset(
            live_scroll_top,
            deferred_item_index,
            has_item_size,
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
            let clamped_visible_items = visible_items.clamp(1, total_items);
            tracing::info!(
                target: "script_kit::scroll_trace",
                event = "SCROLL_TRACE metrics.measured",
                total_items,
                fallback_visible_items,
                live_scroll_top,
                deferred_item_index = ?deferred_item_index,
                has_item_size,
                scroll_offset,
                viewport_height,
                content_height,
                visible_items = clamped_visible_items,
                "SCROLL_TRACE metrics.measured"
            );

            Some((scroll_offset, clamped_visible_items, Some(viewport_height)))
        } else {
            tracing::info!(
                target: "script_kit::scroll_trace",
                event = "SCROLL_TRACE metrics.fallback",
                total_items,
                fallback_visible_items,
                live_scroll_top,
                deferred_item_index = ?deferred_item_index,
                has_item_size,
                scroll_offset,
                visible_items = fallback_visible_items,
                "SCROLL_TRACE metrics.fallback"
            );
            Some((scroll_offset, fallback_visible_items, None))
        }
    }

    fn note_builtin_selection_owned_wheel_scroll(&mut self, selected_index: usize) {
        self.builtin_wheel_owned_selected_index = Some(selected_index);
        tracing::info!(
            target: "script_kit::scroll_trace",
            event = "SCROLL_TRACE wheel_owned.note",
            selected_index,
            "SCROLL_TRACE wheel_owned.note"
        );
    }

    fn should_suppress_builtin_scroll_reanchor(&self, current_selected: usize) -> bool {
        self.builtin_wheel_owned_selected_index == Some(current_selected)
    }

    fn builtin_reanchor_selection_from_scroll(
        &self,
        current_selected: usize,
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> Option<usize> {
        let suppress = self.should_suppress_builtin_scroll_reanchor(current_selected);
        tracing::info!(
            target: "script_kit::scroll_trace",
            event = "SCROLL_TRACE reanchor.check",
            current_selected,
            total_items,
            fallback_visible_items,
            wheel_owned_selected_index = ?self.builtin_wheel_owned_selected_index,
            suppress,
            "SCROLL_TRACE reanchor.check"
        );
        if suppress {
            return None;
        }

        let (first_visible, visible_items, _) = Self::builtin_uniform_list_scrollbar_metrics(
            handle,
            total_items,
            fallback_visible_items,
        )?;
        let reanchored = crate::scrolling::selection_owned::reanchor_uniform_selection(
            current_selected,
            first_visible,
            visible_items,
            total_items,
        );
        tracing::info!(
            target: "script_kit::scroll_trace",
            event = "SCROLL_TRACE reanchor.result",
            current_selected,
            first_visible,
            visible_items,
            total_items,
            reanchored = ?reanchored,
            "SCROLL_TRACE reanchor.result"
        );
        reanchored
    }

    fn builtin_reanchor_selection_from_scroll_handle(
        current_selected: usize,
        handle: &gpui::ScrollHandle,
        total_items: usize,
    ) -> Option<usize> {
        if total_items == 0 || handle.children_count() == 0 {
            return None;
        }

        let first_visible = handle
            .logical_scroll_top()
            .0
            .min(total_items.saturating_sub(1));
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
