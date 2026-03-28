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

        div()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .w_full()
            .h_full()
            // Sharp edges per .impeccable.md (no rounded corners on main containers)
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("clipboard_history")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with input - uses shared gpui_input_state for consistent cursor/selection
            .child(
                div()
                    .w_full()
                    .px(px(crate::ui::chrome::HEADER_PADDING_X))
                    .py(px(crate::ui::chrome::HEADER_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    // Search input - shared component with main menu
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
                    ),
            )
            // Whisper chrome: spacing defines structure, no explicit divider
            // Main content area - 50/50 split: List on left, Preview on right
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    // Left side: Clipboard list (50% width)
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .py(px(design_spacing.padding_xs))
                            .child(list_element),
                    )
                    // Right side: Preview panel (50% width)
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .min_h(px(0.))
                            .overflow_hidden()
                            .child(preview_panel),
                    ),
            )
            // Footer — canonical three-key hint strip
            .child({
                let hints = crate::components::universal_prompt_hints();
                crate::components::emit_prompt_hint_audit("clipboard_history", &hints);
                crate::components::render_simple_hint_strip(hints, None)
            })
            .into_any_element()
