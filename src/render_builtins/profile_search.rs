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
        let accent_color = self.theme.colors.accent.selected;

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
                    div()
                        .font_family(empty_font_family)
                        .text_color(rgb(empty_text_color))
                        .child(if filter.trim().is_empty() {
                            "No profiles"
                        } else {
                            "No matching profiles"
                        }),
                )
                .into_any_element()
        } else {
            let mut list = div()
                .id("profile-search-list")
                .relative()
                .w_full()
                .h_full()
                .min_h(px(0.))
                .overflow_y_scroll()
                .py(px(design_spacing.padding_xs));

            for (index, result) in results.iter().enumerate() {
                let is_selected = index == selected_index;
                let row_bg = if is_selected {
                    crate::ui_foundation::hex_to_rgba_with_opacity(
                        accent_color,
                        crate::theme::opacity::OPACITY_SELECTED,
                    )
                } else {
                    crate::ui_foundation::hex_to_rgba_with_opacity(
                        list_colors.background,
                        crate::theme::opacity::OPACITY_HIDDEN,
                    )
                };
                let status = if result.selected { "Current" } else { "Profile" };
                list = list.child(
                    div()
                        .id(format!("profile-search-row-{index}"))
                        .h(px(LIST_ITEM_HEIGHT))
                        .mx(px(4.0))
                        .px(px(14.0))
                        .py(px(4.0))
                        .rounded(px(8.0))
                        .bg(rgba(row_bg))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(if is_selected {
                                            FontWeight::SEMIBOLD
                                        } else {
                                            FontWeight::NORMAL
                                        })
                                        .text_color(rgb(text_primary))
                                        .child(result.profile.name.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child(format!(
                                            "{} · {}",
                                            crate::profile_search::source_label(result.profile.source),
                                            crate::profile_search::profile_model_label(&result.profile)
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_dimmed))
                                .child(status),
                        ),
                );
            }
            list.into_any_element()
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
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_muted))
                            .child(format!("{} · {}", crate::profile_search::source_label(profile.source), profile.id)),
                    )
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
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_primary))
                            .child(format!(
                                "CWD: {}",
                                profile
                                    .cwd
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_else(|| "Default".to_string())
                            )),
                    )
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
        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            hints, None,
        ));

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
