impl ScriptListApp {
    /// Render the in-product SDK Reference browser (list + preview).
    ///
    /// Data source is [`crate::mcp_resources::sdk_reference_entries_for_ui`] —
    /// the same Rust objects that power the `kit://sdk-reference` MCP resource,
    /// so the in-app docs never drift from the agent-facing contract.
    fn render_sdk_reference_view(
        &mut self,
        filter: &str,
        selected_index: usize,
        entries: std::sync::Arc<[crate::mcp_resources::SdkFunctionRef]>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::expanded("sdk_reference", false),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;

        let visible_rows = crate::mcp_resources::sdk_reference_visible_rows(&entries, filter);
        let filtered_len = visible_rows.len();

        // Clamp selection if the filter shrank the visible set.
        let effective_selected = if filtered_len == 0 {
            0
        } else {
            selected_index.min(filtered_len.saturating_sub(1))
        };
        if effective_selected != selected_index {
            if let AppView::SdkReferenceView {
                selected_index: stored,
                ..
            } = &mut self.current_view
            {
                *stored = effective_selected;
            }
        }

        let preview_entry = visible_rows
            .get(effective_selected)
            .map(|row| row.entry.clone());

        // Key handler: ↑/↓ nav, Esc back, Cmd+C copy, Enter copy-and-back.
        let entries_for_keys = entries.clone();
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                if crate::ui_foundation::is_key_escape(key) {
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

                let (current_filter, current_selected) = if let AppView::SdkReferenceView {
                    filter,
                    selected_index,
                    ..
                } = &this.current_view
                {
                    (filter.clone(), *selected_index)
                } else {
                    return;
                };

                let visible = crate::mcp_resources::sdk_reference_visible_rows(
                    &entries_for_keys,
                    &current_filter,
                );
                let visible_len = visible.len();

                if crate::ui_foundation::is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::SdkReferenceView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if crate::ui_foundation::is_key_down(key) {
                    if current_selected + 1 < visible_len {
                        if let AppView::SdkReferenceView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if has_cmd && key.eq_ignore_ascii_case("c") {
                    if let Some(entry) = visible.get(current_selected).map(|row| row.entry) {
                        let markdown =
                            crate::mcp_resources::format_sdk_reference_entry_markdown(entry);
                        match crate::platform::copy_text_to_clipboard(&markdown) {
                            Ok(()) => {
                                this.show_hud(
                                    format!("Copied {} reference", entry.name),
                                    Some(2000),
                                    cx,
                                );
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "sdk_reference copy_text_to_clipboard failed");
                            }
                        }
                    }
                    cx.stop_propagation();
                } else if crate::ui_foundation::is_key_enter(key) {
                    if let Some(entry) = visible.get(current_selected).map(|row| row.entry) {
                        let markdown =
                            crate::mcp_resources::format_sdk_reference_entry_markdown(entry);
                        if let Err(e) = crate::platform::copy_text_to_clipboard(&markdown) {
                            tracing::warn!(error = %e, "sdk_reference enter-copy failed");
                        } else {
                            this.show_hud(
                                format!("Copied {} reference", entry.name),
                                Some(2000),
                                cx,
                            );
                        }
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        // Build list rows.
        let list_colors = ListItemColors::from_theme(&self.theme);
        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child(if filter.trim().is_empty() {
                    "No SDK functions available"
                } else {
                    "No SDK functions match your filter"
                })
                .into_any_element()
        } else {
            let entries_for_list = entries.clone();
            let visible_for_list: Vec<usize> =
                visible_rows.iter().map(|row| row.source_index).collect();
            let selected = effective_selected;

            div()
                .id("sdk-reference-list")
                .w_full()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .overflow_y_scrollbar()
                .children(visible_for_list.into_iter().enumerate().map(
                    move |(display_ix, original_idx)| {
                        let entry = entries_for_list
                            .get(original_idx)
                            .expect("visible index within bounds");
                        let is_selected = display_ix == selected;
                        let is_unsupported =
                            entry.support == crate::mcp_resources::SdkSupport::Unsupported;

                        let description = if entry.signature.is_empty() {
                            entry.category.clone()
                        } else {
                            format!("{}  ·  {}", entry.category, entry.signature)
                        };

                        let item = ListItem::new(entry.name.clone(), list_colors)
                            .description_opt(Some(description))
                            .selected(is_selected)
                            .with_accent_bar(true);

                        // Row wrapper: when the entry is unsupported, overlay a
                        // right-aligned "Unsupported in GPUI" pill on the item so
                        // the label is visible without changing filter ranking.
                        let row = div()
                            .id(gpui::ElementId::Integer(display_ix as u64))
                            .relative()
                            .w_full()
                            .child(item);

                        if is_unsupported {
                            row.child(
                                div()
                                    .absolute()
                                    .top(px(design_spacing.padding_sm))
                                    .right(px(design_spacing.padding_md))
                                    .px(px(design_spacing.padding_sm))
                                    .py(px(2.))
                                    .text_size(px(design_typography.font_size_xs))
                                    .text_color(rgb(text_muted))
                                    .child("Unsupported in GPUI"),
                            )
                        } else {
                            row
                        }
                    },
                ))
                .into_any_element()
        };

        // Preview panel.
        let preview_panel: AnyElement = match &preview_entry {
            Some(entry) => {
                let is_unsupported =
                    entry.support == crate::mcp_resources::SdkSupport::Unsupported;
                let unsupported_note = entry.unsupported_note.clone();

                let base = div()
                    .w_full()
                    .h_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .font_family(design_typography.font_family)
                    .flex()
                    .flex_col()
                    .gap(px(design_spacing.padding_sm));

                let base = if is_unsupported {
                    let banner_title = div()
                        .text_size(px(design_typography.font_size_sm))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(text_dimmed))
                        .child("⚠ Unsupported in GPUI");
                    let banner_body = div()
                        .text_size(px(design_typography.font_size_xs))
                        .text_color(rgb(text_muted))
                        .child(
                            unsupported_note.clone().unwrap_or_else(|| {
                                "This API is defined in scripts/kit-sdk.ts, but the GPUI app does not currently handle it. It will log a warning and may no-op or throw at runtime.".to_string()
                            }),
                        );
                    let banner = div()
                        .w_full()
                        .px(px(design_spacing.padding_md))
                        .py(px(design_spacing.padding_sm))
                        .flex()
                        .flex_col()
                        .gap(px(design_spacing.padding_xs))
                        .child(banner_title)
                        .child(banner_body);
                    base.child(banner)
                } else {
                    base
                };

                base.child(
                    div()
                        .text_size(px(design_typography.font_size_xl))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(text_primary))
                        .child(entry.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(design_typography.font_size_sm))
                        .text_color(rgb(text_dimmed))
                        .child(entry.signature.clone()),
                )
                .child(
                    div()
                        .text_size(px(design_typography.font_size_xs))
                        .text_color(rgb(text_muted))
                        .child(entry.category.clone()),
                )
                .child(
                    div()
                        .text_size(px(design_typography.font_size_md))
                        .text_color(rgb(text_primary))
                        .child(entry.description.clone()),
                )
                .into_any_element()
            }
            None => div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child("Select an SDK function")
                .into_any_element(),
        };

        let header_element = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .child(
                div().flex_1().flex().flex_row().items_center().child(
                    Input::new(&self.gpui_input_state)
                        .w_full()
                        .h(px(28.))
                        .px(px(0.))
                        .py(px(0.))
                        .with_size(Size::Size(px(design_typography.font_size_xl)))
                        .appearance(false)
                        .bordered(false)
                        .focus_bordered(false),
                ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(text_dimmed))
                    .child(format!(
                        "{} function{}",
                        entries.len(),
                        if entries.len() == 1 { "" } else { "s" },
                    )),
            );

        let list_pane = div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .py(px(design_spacing.padding_xs))
            .child(list_element);

        let hints: Vec<SharedString> = vec![
            "↵ Copy Markdown".into(),
            "⌘C Copy Markdown".into(),
            "↑↓ Navigate".into(),
            "Esc Back".into(),
        ];
        crate::components::emit_prompt_hint_audit("sdk_reference", &hints);

        let gpui_footer = crate::components::render_simple_hint_strip(hints, None);
        let footer = self.main_window_footer_slot(gpui_footer);

        crate::components::render_expanded_view_scaffold_with_footer(
            header_element,
            list_pane,
            preview_panel,
            footer,
        )
        .text_color(rgb(text_primary))
        .font_family(design_typography.font_family)
        .key_context("sdk_reference")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}
