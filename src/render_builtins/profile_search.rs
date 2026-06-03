fn profile_search_row_description(result: &crate::profile_search::ProfileSearchResult) -> String {
    format!(
        "{} · {}",
        crate::profile_search::source_label(result.profile.source),
        crate::profile_search::profile_model_label(&result.profile)
    )
}

fn profile_search_row_status_accessory(
    result: &crate::profile_search::ProfileSearchResult,
    list_colors: &crate::list_item::ListItemColors,
    selected: bool,
) -> AnyElement {
    let status = if result.selected {
        "Current"
    } else {
        "Profile"
    };
    div()
        .max_w(px(88.0))
        .overflow_hidden()
        .text_xs()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(rgba(crate::list_item::row_description_text_rgba(
            list_colors,
            selected,
        )))
        .child(status)
        .into_any_element()
}

impl ScriptListApp {
    fn render_profile_search(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::expanded("profile_search", false),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let color_resolver =
            crate::theme::ColorResolver::new_for_shell(&self.theme, self.current_design);
        let typography_resolver =
            crate::theme::TypographyResolver::new_theme_first(&self.theme, self.current_design);
        let empty_text_color = color_resolver.empty_text_color();
        let empty_font_family = typography_resolver.primary_font().to_string();
        let list_colors = crate::list_item::ListItemColors::from_theme(&self.theme);
        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;
        let ui_border = self.theme.colors.ui.border;

        let results = self.profile_search_results_for_filter(&filter);
        let filtered_len = results.len();
        let selected_index = selected_index.min(filtered_len.saturating_sub(1));
        let selected = results.get(selected_index).cloned();

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
                if is_key_escape(key) {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }
                if is_key_up(key) {
                    this.move_profile_search_selection(true, cx);
                    cx.stop_propagation();
                    return;
                }
                if is_key_down(key) {
                    this.move_profile_search_selection(false, cx);
                    cx.stop_propagation();
                    return;
                }
                if is_key_enter(key) {
                    cx.stop_propagation();
                    this.select_profile_search_result(cx);
                    return;
                }
            },
        );

        let header_element = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(self.render_search_input()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(text_dimmed))
                    .child(format!("{filtered_len} profiles")),
            );

        let list_pane = if results.is_empty() {
            div()
                .id("profile-search-list")
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    crate::list_item::EmptyState::new(
                        if filter.trim().is_empty() {
                            "No profiles"
                        } else {
                            "No matching profiles"
                        },
                        empty_text_color,
                        &empty_font_family,
                    )
                    .into_element(),
                )
                .into_any_element()
        } else {
            // Profile Search is a split-pane built-in, but its rows must use the
            // same shared ListItem chrome as Clipboard History and File Search.
            // This surface originally hand-built row padding/backgrounds while the
            // profile preview was introduced; ListItem owns selected/hover/theme
            // behavior so row metrics and opacity packing do not drift here.
            let profile_results_for_list = results.clone();
            let current_selected = selected_index;
            let profile_hovered = self.hovered_index;
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();
            let list_element = uniform_list(
                "profile-search-results",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some(result) = profile_results_for_list.get(ix) {
                                let is_selected = ix == current_selected;
                                let is_hovered = profile_hovered == Some(ix);

                                let click_entity = click_entity_handle.clone();
                                let click_handler =
                                    move |_event: &gpui::ClickEvent,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app) = click_entity.upgrade() {
                                            app.update(cx, |this, cx| {
                                                if let AppView::ProfileSearchView {
                                                    selected_index,
                                                    ..
                                                } = &mut this.current_view
                                                {
                                                    *selected_index = ix;
                                                }
                                                this.list_scroll_handle
                                                    .scroll_to_item(ix, ScrollStrategy::Nearest);
                                                cx.notify();
                                            });
                                        }
                                        cx.stop_propagation();
                                    };

                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler =
                                    move |hovered: &bool,
                                          _window: &mut Window,
                                          cx: &mut gpui::App| {
                                        if let Some(app) = hover_entity.upgrade() {
                                            app.update(cx, |this, cx| {
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
                                            });
                                        }
                                    };

                                div()
                                    .id(ix)
                                    .cursor_pointer()
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(
                                        ListItem::new(result.profile.name.clone(), list_colors)
                                            .description(profile_search_row_description(result))
                                            .selected(is_selected)
                                            .hovered(is_hovered)
                                            .with_accent_bar(true)
                                            .semantic_id(format!(
                                                "profile-search-row:{}",
                                                result.profile.id
                                            ))
                                            .trailing_accessory(
                                                profile_search_row_status_accessory(
                                                    result,
                                                    &list_colors,
                                                    is_selected,
                                                ),
                                            ),
                                    )
                            } else {
                                div().id(ix).h(px(LIST_ITEM_HEIGHT))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element();
            let list_scrollbar =
                self.builtin_uniform_list_scrollbar(&self.list_scroll_handle, filtered_len, 8);

            div()
                .id("profile-search-list")
                .relative()
                .w_full()
                .h_full()
                .min_h(px(0.))
                .overflow_hidden()
                .py(px(design_spacing.padding_xs))
                .child(list_element)
                .child(list_scrollbar)
                .into_any_element()
        };

        let preview_pane = div()
            .id("profile-search-preview")
            .w_full()
            .h_full()
            .min_h(px(0.))
            .overflow_y_scroll()
            .p(px(16.0))
            .border_l_1()
            .border_color(rgba(crate::ui_foundation::hex_to_rgba_with_opacity(
                ui_border,
                crate::theme::opacity::OPACITY_SUBTLE,
            )))
            .child(if let Some(result) = selected {
                let profile = result.profile;
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .id("profile-search-preview-title")
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(text_primary))
                            .child(profile.name.clone()),
                    )
                    .child(div().text_sm().text_color(rgb(text_muted)).child(format!(
                        "{} · {}",
                        crate::profile_search::source_label(profile.source),
                        profile.id
                    )))
                    .child(
                        div()
                            .id("profile-search-preview-model")
                            .text_sm()
                            .text_color(rgb(text_primary))
                            .child(format!(
                                "Model: {} ({})",
                                crate::profile_search::profile_model_label(&profile),
                                crate::profile_search::backend_label(profile.backend)
                            )),
                    )
                    .child(div().text_sm().text_color(rgb(text_primary)).child(format!(
                                "CWD: {}",
                                profile
                                    .cwd
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_else(|| "Default".to_string())
                            )))
                    .child(
                        div()
                            .id("profile-search-preview-tools")
                            .text_sm()
                            .text_color(rgb(text_primary))
                            .child(format!(
                                "Tools: {}",
                                crate::profile_search::profile_tools_label(&profile)
                            )),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .line_height(px(20.0))
                            .child(crate::profile_search::profile_prompt_summary(&profile)),
                    )
                    .into_any_element()
            } else {
                div()
                    .text_color(rgb(text_dimmed))
                    .child("Select a profile to preview its model, tools, cwd, and prompt.")
                    .into_any_element()
            });

        let hints: Vec<SharedString> = vec!["↵ Select Profile".into(), "Esc Back".into()];
        crate::components::emit_prompt_hint_audit("profile_search", &hints);
        let footer =
            self.main_window_footer_slot(crate::components::render_simple_hint_strip(hints, None));

        crate::components::render_expanded_view_scaffold_with_footer(
            header_element,
            list_pane,
            preview_pane.into_any_element(),
            footer,
        )
        .id("profile-search-root")
        .key_context("ProfileSearchView")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}
