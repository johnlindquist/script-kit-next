#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserTabsActivationAction {
    ActivateSelectedTab,
}

impl BrowserTabsActivationAction {
    fn failure_message(self, error: impl std::fmt::Display) -> String {
        match self {
            Self::ActivateSelectedTab => format!("Failed to activate tab: {error}"),
        }
    }

    fn generic_failure_message(self) -> &'static str {
        match self {
            Self::ActivateSelectedTab => "Failed to activate tab",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserTabsEmptyState {
    NoOpenTabs,
    NoFilteredMatches,
}

impl BrowserTabsEmptyState {
    fn from_filter(filter: &str) -> Self {
        if filter.is_empty() {
            Self::NoOpenTabs
        } else {
            Self::NoFilteredMatches
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::NoOpenTabs => "No open browser tabs",
            Self::NoFilteredMatches => "No browser tabs match your filter",
        }
    }
}

impl ScriptListApp {
    fn browser_tabs_visible_rows(&self, filter: &str) -> Vec<crate::browser_tabs::BrowserTabInfo> {
        crate::browser_tabs::fuzzy_search_browser_tabs(&self.cached_browser_tabs, filter)
            .into_iter()
            .map(|entry| entry.tab)
            .collect()
    }

    fn browser_tabs_selected_visible_row(
        &self,
        filter: &str,
        selected_index: usize,
    ) -> Option<crate::browser_tabs::BrowserTabInfo> {
        self.browser_tabs_visible_rows(filter)
            .get(selected_index)
            .cloned()
    }

    fn browser_tabs_dataset_and_visible_counts(&self, filter: &str) -> (usize, usize) {
        (
            self.cached_browser_tabs.len(),
            self.browser_tabs_visible_rows(filter).len(),
        )
    }

    fn browser_tabs_visible_row_labels(&self, filter: &str) -> Vec<String> {
        self.browser_tabs_visible_rows(filter)
            .into_iter()
            .map(|tab| tab.display_title().to_string())
            .collect()
    }

    fn browser_tabs_count_label(total_count: usize) -> String {
        let suffix = if total_count == 1 { "" } else { "s" };
        format!("{} tab{}", total_count, suffix)
    }

    fn render_browser_tabs(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::minimal_list(
                "render_builtins::browser_tabs",
                true,
            ),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;

        let filtered_tabs =
            crate::browser_tabs::fuzzy_search_browser_tabs(&self.cached_browser_tabs, &filter);
        let filtered_len = filtered_tabs.len();
        let selected_index = if let Some(reanchored) = self.builtin_reanchor_selection_from_scroll(
            selected_index,
            &self.browser_tabs_scroll_handle,
            filtered_len,
            8,
        ) {
            tracing::info!(
                target: "script_kit::scroll",
                event = "builtin_selection_resynced_from_scrollbar",
                view = "browser_tabs",
                reason = "render",
                selected_before = selected_index,
                selected_after = reanchored,
            );
            if let AppView::BrowserTabsView { selected_index, .. } = &mut self.current_view {
                *selected_index = reanchored;
            }
            reanchored
        } else {
            selected_index
        };
        let total_count = self.cached_browser_tabs.len();

        // Key handler — only navigation keys; character input flows through the shared GPUI Input.
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                if let AppView::BrowserTabsView {
                    filter,
                    selected_index,
                } = &mut this.current_view
                {
                    let filtered_tabs = crate::browser_tabs::fuzzy_search_browser_tabs(
                        &this.cached_browser_tabs,
                        filter,
                    );
                    let filtered_len = filtered_tabs.len();

                    if is_key_up(key) {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            this.browser_tabs_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            cx.notify();
                        }
                        cx.stop_propagation();
                    } else if is_key_down(key) {
                        if *selected_index < filtered_len.saturating_sub(1) {
                            *selected_index += 1;
                            this.browser_tabs_scroll_handle
                                .scroll_to_item(*selected_index, ScrollStrategy::Nearest);
                            cx.notify();
                        }
                        cx.stop_propagation();
                    } else if is_key_enter(key) {
                        if let Some(tab) = filtered_tabs.get(*selected_index).map(|m| m.tab.clone())
                        {
                            let activation_action =
                                BrowserTabsActivationAction::ActivateSelectedTab;
                            match crate::browser_tabs::activate_tab(&tab) {
                                Ok(()) => this.hide_main_and_reset(cx),
                                Err(error) => {
                                    this.toast_manager.push(
                                        components::toast::Toast::error(
                                            activation_action.failure_message(error),
                                            &this.theme,
                                        )
                                        .duration_ms(Some(TOAST_ERROR_MS)),
                                    );
                                    cx.notify();
                                }
                            }
                        }
                        cx.stop_propagation();
                    }
                }
            },
        );

        let list_colors = ListItemColors::from_theme(&self.theme);
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child(BrowserTabsEmptyState::from_filter(&filter).message())
                .into_any_element()
        } else {
            let app_icons: std::collections::HashMap<String, crate::app_launcher::DecodedIcon> =
                self.apps
                    .iter()
                    .filter_map(|app| {
                        app.bundle_id
                            .clone()
                            .and_then(|bundle_id| app.icon.clone().map(|icon| (bundle_id, icon)))
                    })
                    .collect();
            let tabs_for_closure: Vec<(crate::browser_tabs::BrowserTabInfo, list_item::IconKind)> =
                filtered_tabs
                    .iter()
                    .map(|tab_match| {
                        let tab = tab_match.tab.clone();
                        let icon = browser_tab_icon_for_render(&tab, &app_icons);
                        (tab, icon)
                    })
                    .collect();
            let selected = selected_index;
            let hovered = self.hovered_index;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

            uniform_list(
                "browser-tabs",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((tab, icon)) = tabs_for_closure.get(ix) {
                                let tab = tab.clone();
                                let is_selected = ix == selected;
                                let is_hovered = hovered == Some(ix);

                                let description = if !tab.title.trim().is_empty() {
                                    Some(tab.url.to_string())
                                } else if !tab.browser_name.is_empty() {
                                    Some(tab.browser_name.to_string())
                                } else {
                                    None
                                };
                                let display_title = tab.display_title().to_string();

                                let click_entity = click_entity_handle.clone();
                                let hover_entity = hover_entity_handle.clone();
                                let tab_for_click = tab.clone();
                                let activation_action =
                                    BrowserTabsActivationAction::ActivateSelectedTab;
                                let click_handler =
                                    move |event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app_entity) = click_entity.upgrade() {
                                            let tab = tab_for_click.clone();
                                            app_entity.update(cx, |this, cx| {
                                                if let AppView::BrowserTabsView {
                                                    selected_index,
                                                    ..
                                                } = &mut this.current_view
                                                {
                                                    *selected_index = ix;
                                                }
                                                cx.notify();

                                                if let gpui::ClickEvent::Mouse(mouse_event) = event
                                                {
                                                    if mouse_event.down.click_count == 2 {
                                                        if crate::browser_tabs::activate_tab(&tab)
                                                            .is_ok()
                                                        {
                                                            this.hide_main_and_reset(cx);
                                                        } else {
                                                            this.toast_manager.push(
                                                                components::toast::Toast::error(
                                                                    activation_action
                                                                        .generic_failure_message(),
                                                                    &this.theme,
                                                                )
                                                                .duration_ms(Some(TOAST_ERROR_MS)),
                                                            );
                                                            cx.notify();
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    };

                                let hover_handler =
                                    move |is_hovered: &bool,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app) = hover_entity.upgrade() {
                                            app.update(cx, |this, cx| {
                                                if *is_hovered {
                                                    this.input_mode = InputMode::Mouse;
                                                    if this.hovered_index != Some(ix) {
                                                        this.hovered_index = Some(ix);
                                                        cx.notify();
                                                    }
                                                } else if this.hovered_index == Some(ix) {
                                                    this.hovered_index = None;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                    };

                                div()
                                    .id(ix)
                                    .cursor_pointer()
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(
                                        ListItem::new(display_title, list_colors)
                                            .icon_kind(icon.clone())
                                            .description_opt(description)
                                            .selected(is_selected)
                                            .hovered(is_hovered)
                                            .with_accent_bar(true),
                                    )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.browser_tabs_scroll_handle)
            .into_any_element()
        };

        // Build layout following the process_manager / favorites pattern:
        // outer div with shared GPUI Input in header, divider, scrollable list, footer.
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(self.theme_font_family())
            .key_context("browser_tabs")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with shared GPUI Input
            .child(
                div()
                    .w_full()
                    .px(px(crate::ui::chrome::HEADER_PADDING_X))
                    .py(px(crate::ui::chrome::HEADER_PADDING_Y))
                    .min_h(px(crate::panel::HEADER_BUTTON_HEIGHT))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            self.render_search_input()
                        ),
                    )
                    .child(
                        div()
                            .text_size(px(design_typography.font_size_sm))
                            .text_color(rgb(text_dimmed))
                            .child(Self::browser_tabs_count_label(total_count)),
                    ),
            )
            // Divider
            .child(crate::components::SectionDivider::new())
            // Scrollable list
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .on_scroll_wheel(cx.listener(
                        move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                            let view_state = if let AppView::BrowserTabsView {
                                filter,
                                selected_index,
                            } = &this.current_view
                            {
                                Some((filter.clone(), *selected_index))
                            } else {
                                None
                            };

                            let Some((current_filter, current_selected)) = view_state else {
                                return;
                            };

                            let filtered_len = crate::browser_tabs::fuzzy_search_browser_tabs(
                                &this.cached_browser_tabs,
                                &current_filter,
                            )
                            .len();

                            let Some(new_selected) = this.builtin_scroll_target_from_wheel(
                                event,
                                current_selected,
                                filtered_len,
                            ) else {
                                if filtered_len > 0 {
                                    cx.stop_propagation();
                                }
                                return;
                            };

                            if let AppView::BrowserTabsView { selected_index, .. } =
                                &mut this.current_view
                            {
                                *selected_index = new_selected;
                            }

                            this.browser_tabs_scroll_handle
                                .scroll_to_item(new_selected, ScrollStrategy::Nearest);
                            this.note_builtin_selection_owned_wheel_scroll(new_selected);

                            Self::log_builtin_scroll_event(
                                "browser_tabs",
                                "scroll_to_item",
                                "wheel",
                                filtered_len,
                                Some(new_selected),
                                Some(new_selected),
                                Some(&current_filter),
                                "mouse",
                            );
                            cx.notify();
                            cx.stop_propagation();
                        },
                    ))
                    .child(list_element)
                    .child(self.builtin_uniform_list_scrollbar(
                        &self.browser_tabs_scroll_handle,
                        filtered_len,
                        8,
                    )),
            )
            // Footer
            .when_some(
                self.main_window_footer_slot(crate::components::render_simple_hint_strip(
                    vec![
                        gpui::SharedString::from("↵ Open Tab"),
                        gpui::SharedString::from("Esc Back"),
                    ],
                    None,
                )),
                |d, footer| d.child(footer),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod browser_tabs_scroll_contract {
    const SOURCE: &str = include_str!("browser_tabs.rs");

    #[test]
    fn browser_tabs_use_wheel_contract_and_vendor_scrollbar() {
        assert!(
            SOURCE.contains(".on_scroll_wheel(cx.listener("),
            "browser tabs should intercept wheel scrolling on the list pane"
        );
        assert!(
            SOURCE.contains("builtin_scroll_target_from_wheel("),
            "browser tabs should use shared wheel delta conversion"
        );
        assert!(
            SOURCE.contains("builtin_reanchor_selection_from_scroll("),
            "browser tabs should reanchor selection after handle movement"
        );
        assert!(
            SOURCE.contains("builtin_uniform_list_scrollbar("),
            "browser tabs should attach the shared vendor scrollbar helper"
        );
    }
}

fn browser_tab_icon_for_render(
    tab: &crate::browser_tabs::BrowserTabInfo,
    app_icons: &std::collections::HashMap<String, crate::app_launcher::DecodedIcon>,
) -> list_item::IconKind {
    // Prefer per-site favicon (fetched from Google's favicon service)
    if let Some(favicon) = script_kit_gpui::favicons::cached_favicon(&tab.url) {
        return list_item::IconKind::Image(favicon);
    }

    // Fall back to browser app icon
    if let Some(icon) = app_icons.get(tab.browser_bundle_id.as_ref()) {
        return list_item::IconKind::Image(icon.clone());
    }

    // Last resort: browser-specific emoji
    match tab.browser_name.as_ref() {
        "Safari" => list_item::IconKind::Emoji("🧭".to_string()),
        "Google Chrome" => list_item::IconKind::Emoji("🌐".to_string()),
        "Arc" => list_item::IconKind::Emoji("🅰️".to_string()),
        "Brave Browser" => list_item::IconKind::Emoji("🦁".to_string()),
        "Microsoft Edge" => list_item::IconKind::Emoji("🌊".to_string()),
        _ => list_item::IconKind::Emoji("🔗".to_string()),
    }
}

#[cfg(test)]
mod browser_tabs_chrome_tests {
    fn production_source() -> &'static str {
        include_str!("browser_tabs.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source should exist")
    }

    #[test]
    fn browser_tabs_uses_shared_input_and_chrome() {
        let source = production_source();

        assert!(
            source.contains("self.render_search_input()"),
            "browser tabs must use the shared GPUI input"
        );
        assert!(
            !source.contains(&["Input::new(&self.", "gpui_input_state)"].concat()),
            "browser tabs should delegate GPUI input construction to render_search_input"
        );
        assert!(
            source.contains("HEADER_PADDING_X") && source.contains("HEADER_PADDING_Y"),
            "browser tabs must use shared chrome header padding"
        );
        assert!(
            source.contains("SectionDivider::new()"),
            "browser tabs must use SectionDivider"
        );
        assert!(
            source.contains("render_simple_hint_strip("),
            "browser tabs must use render_simple_hint_strip for footer"
        );
        assert!(
            source.contains(".key_context(\"browser_tabs\")"),
            "browser tabs must keep a dedicated key context"
        );
        assert!(
            !source.contains("CURSOR_WIDTH") && !source.contains("CURSOR_HEIGHT_LG"),
            "browser tabs must not build a custom fake cursor — use the shared Input"
        );
    }
}
