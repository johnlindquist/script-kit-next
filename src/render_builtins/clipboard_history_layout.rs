        // Build preview panel for selected entry
        let selected_entry = filtered_entries
            .get(selected_index)
            .map(|(_, e)| (*e).clone());
        let preview_panel = self.render_clipboard_preview_panel(
            &selected_entry,
            &image_cache,
            &design_spacing,
            &design_typography,
            &design_visual,
        );

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
                    .child(format!("{} entries", self.cached_clipboard_entries.len())),
            );

        let list_pane = div()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .py(px(design_spacing.padding_xs))
            .child(list_element);

        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("clipboard_history", &hints);

        tracing::info!(
            target: "script_kit::prompt_chrome",
            surface = "clipboard_history",
            layout_mode = "expanded_scaffold",
            "clipboard_history_chrome_checkpoint"
        );

        crate::components::render_expanded_view_scaffold_with_hints(
            header_element,
            list_pane,
            preview_panel,
            hints,
            None,
        )
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("clipboard_history")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .into_any_element()
