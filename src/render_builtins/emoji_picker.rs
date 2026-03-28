impl ScriptListApp {
    #[allow(dead_code)] // Called from startup_new_arrow.rs interceptor
    pub(crate) fn navigate_emoji_picker(
        &mut self,
        direction: crate::emoji::EmojiNavDirection,
        cx: &mut Context<Self>,
    ) -> bool {
        let AppView::EmojiPickerView {
            filter,
            selected_index,
            selected_category,
        } = &mut self.current_view
        else {
            return false;
        };

        let ordered_emojis = crate::emoji::filtered_ordered_emojis(filter, *selected_category);
        if ordered_emojis.is_empty() {
            *selected_index = 0;
            self.hovered_index = None;
            cx.notify();
            return false;
        }

        *selected_index = (*selected_index).min(ordered_emojis.len() - 1);

        let layout = crate::emoji::build_emoji_grid_layout(
            &ordered_emojis,
            crate::emoji::GRID_COLS,
            |emoji| emoji.category,
        );

        let old_index = *selected_index;
        *selected_index = layout.move_index(old_index, direction);
        let row = layout.scroll_row_for_index(*selected_index);

        tracing::debug!(
            target: "script_kit::scroll",
            event = "scroll_to_item",
            reason = "selection_changed",
            direction = ?direction,
            old_index,
            selected_index = *selected_index,
            row,
        );

        self.emoji_scroll_handle
            .scroll_to_item(row, ScrollStrategy::Nearest);

        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        cx.notify();
        true
    }

    fn render_emoji_picker(
        &mut self,
        filter: String,
        selected_index: usize,
        selected_category: Option<crate::emoji::EmojiCategory>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        let ordered_emojis = crate::emoji::filtered_ordered_emojis(&filter, selected_category);
        let cols = crate::emoji::GRID_COLS;

        #[derive(Clone)]
        enum EmojiGridRow {
            Header { title: String, count: usize },
            Cells { start_index: usize, count: usize },
        }

        let mut rows: Vec<EmojiGridRow> = Vec::new();
        {
            let mut flat_offset = 0;
            for category in crate::emoji::all_categories() {
                let category_count = ordered_emojis[flat_offset..]
                    .iter()
                    .take_while(|e| e.category == category)
                    .count();

                if category_count == 0 {
                    continue;
                }

                rows.push(EmojiGridRow::Header {
                    title: category.display_name().to_string(),
                    count: category_count,
                });

                let mut row_offset = 0;
                while row_offset < category_count {
                    let row_count = (category_count - row_offset).min(cols);
                    rows.push(EmojiGridRow::Cells {
                        start_index: flat_offset + row_offset,
                        count: row_count,
                    });
                    row_offset += row_count;
                }
                flat_offset += category_count;
            }
        }

        let filtered_len = ordered_emojis.len();
        let selected_index = if filtered_len == 0 {
            0
        } else {
            selected_index.min(filtered_len - 1)
        };

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
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::EmojiPicker,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {}
                    ActionsRoute::Handled => {
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        this.handle_action(action_id, window, cx);
                        return;
                    }
                }

                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::EmojiPickerView {
                    filter,
                    selected_index,
                    selected_category,
                } = &mut this.current_view
                {
                    let ordered_emojis =
                        crate::emoji::filtered_ordered_emojis(filter, *selected_category);

                    match key {
                        _ if is_key_enter(key) => {
                            if let Some(emoji) = ordered_emojis.get(*selected_index) {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    emoji.emoji.to_string(),
                                ));
                                this.close_and_reset_window(cx);
                            }
                            cx.stop_propagation();
                        }
                        _ => (),
                    }
                }
            },
        );

        let tile_size = crate::emoji::GRID_TILE_SIZE;
        let tile_gap = crate::emoji::GRID_TILE_GAP;
        const EMOJI_ROW_ID_OFFSET: usize = 10_000;
        const EMOJI_CELL_ID_OFFSET: usize = 20_000;

        let grid_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_dimmed))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No emojis found"
                } else {
                    "No emojis match your filter"
                })
                .into_any_element()
        } else {
            let row_height = crate::emoji::GRID_ROW_HEIGHT;
            let selected_outline = rgba((self.theme.colors.accent.selected << 8) | 0x80);
            let selected_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x2a);
            let idle_bg = rgba((ui_border << 8) | 0x10);
            let click_entity_handle = cx.entity().downgrade();

            let rows_for_list = rows.clone();
            let emojis_for_list = std::sync::Arc::new(ordered_emojis.clone());
            let selected = selected_index;

            uniform_list(
                "emoji-picker-grid",
                rows_for_list.len(),
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|row_index| match rows_for_list.get(row_index) {
                            Some(EmojiGridRow::Header { title, count }) => div()
                                .id(EMOJI_ROW_ID_OFFSET + row_index)
                                .w_full()
                                .h(px(row_height))
                                .px(px(design_spacing.padding_lg))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_primary))
                                        .child(title.clone()),
                                )
                                .child(
                                    div()
                                        .min_w(px(28.0))
                                        .px(px(design_spacing.padding_xs))
                                        .py(px(2.0))
                                        .rounded(px(design_visual.radius_md))
                                        .bg(rgba((ui_border << 8) | 0x18))
                                        .text_sm()
                                        .text_color(rgb(text_dimmed))
                                        .child(count.to_string()),
                                )
                                .into_any_element(),

                            Some(EmojiGridRow::Cells { start_index, count }) => {
                                let mut row_div = div()
                                    .id(EMOJI_ROW_ID_OFFSET + row_index)
                                    .w_full()
                                    .h(px(row_height))
                                    .flex()
                                    .items_center()
                                    .px(px(design_spacing.padding_lg))
                                    .gap(px(tile_gap));

                                for col in 0..*count {
                                    let flat_emoji_index = *start_index + col;
                                    let emoji = match emojis_for_list.get(flat_emoji_index) {
                                        Some(e) => e,
                                        None => {
                                            row_div = row_div
                                                .child(div().w(px(tile_size)).h(px(tile_size)));
                                            continue;
                                        }
                                    };

                                    let is_selected = flat_emoji_index == selected;
                                    let click_entity = click_entity_handle.clone();
                                    let emoji_value = emoji.emoji.to_string();
                                    let emoji_display = emoji_value.clone();

                                    row_div = row_div.child(
                                        div()
                                            .id(EMOJI_CELL_ID_OFFSET + flat_emoji_index)
                                            .w(px(tile_size))
                                            .h(px(tile_size))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .on_click(
                                                move |_event: &gpui::ClickEvent,
                                                      _window: &mut Window,
                                                      cx: &mut gpui::App| {
                                                    if let Some(app) = click_entity.upgrade() {
                                                        let emoji_value = emoji_value.clone();
                                                        app.update(cx, |this, cx| {
                                                            cx.write_to_clipboard(
                                                                gpui::ClipboardItem::new_string(
                                                                    emoji_value.clone(),
                                                                ),
                                                            );
                                                            this.show_hud(
                                                                format!("Copied {}", emoji_value),
                                                                Some(HUD_FLASH_MS),
                                                                cx,
                                                            );
                                                            this.close_and_reset_window(cx);
                                                        });
                                                    }
                                                },
                                            )
                                            .child(
                                                div()
                                                    .w_full()
                                                    .h_full()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(px(design_visual.radius_md))
                                                    .bg(idle_bg)
                                                    .text_size(px(28.0))
                                                    .when(is_selected, |d| {
                                                        d.bg(selected_bg)
                                                            .border_1()
                                                            .border_color(selected_outline)
                                                    })
                                                    .child(emoji_display),
                                            ),
                                    );
                                }

                                row_div.child(div().flex_1()).into_any_element()
                            }

                            None => div().w_full().h(px(row_height)).into_any_element(),
                        })
                        .collect::<Vec<_>>()
                },
            )
            .w_full()
            .h_full()
            .track_scroll(&self.emoji_scroll_handle)
            .into_any_element()
        };

        let list_scrollbar = self.builtin_uniform_list_scrollbar(
            &self.emoji_scroll_handle,
            rows.len(),
            8,
        );

        let header = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .child(
                div().flex_1().flex().flex_row().items_center().child(
                    Input::new(&self.gpui_input_state)
                        .w_full()
                        .h(px(28.0))
                        .px(px(0.0))
                        .py(px(0.0))
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
                    .child(format!("{} emojis", filtered_len)),
            );

        let content = div()
            .flex_1()
            .min_h(px(0.0))
            .w_full()
            .overflow_hidden()
            .py(px(design_spacing.padding_xs))
            .child(
                div()
                    .relative()
                    .w_full()
                    .h_full()
                    .child(grid_element)
                    .child(list_scrollbar),
            );

        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::grid(
                "render_builtins::emoji_picker",
                true,
            ),
        );

        crate::components::render_minimal_list_prompt_scaffold(
            header,
            content,
            crate::components::universal_prompt_hints(),
            None,
        )
        .rounded(px(design_visual.radius_lg))
        .text_color(rgb(text_primary))
        .font_family(design_typography.font_family)
        .key_context("emoji_picker")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}

#[cfg(test)]
mod emoji_picker_chrome_audit {
    #[test]
    fn emoji_picker_uses_minimal_chrome_footer() {
        let source = include_str!("emoji_picker.rs");
        assert!(
            source.contains("render_minimal_list_prompt_scaffold("),
            "emoji_picker should use render_minimal_list_prompt_scaffold"
        );
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert_eq!(
            source.matches(&legacy).count(),
            0,
            "emoji_picker should not use PromptFooter"
        );
    }
}

#[cfg(test)]
mod emoji_picker_spec_tests {
    fn read_source() -> String {
        include_str!("emoji_picker.rs").to_string()
    }

    #[test]
    fn emoji_picker_declares_grid_audit_and_canonical_footer() {
        let source = read_source();
        assert!(
            source.contains("PromptChromeAudit::grid(")
                && source.contains("\"render_builtins::emoji_picker\""),
            "emoji picker should declare grid layout in runtime audit"
        );
        assert!(
            source.contains("universal_prompt_hints()"),
            "emoji picker should use the shared three-key footer"
        );
        assert!(
            !source.contains("\"↵ Copy\"") && !source.contains("\"Esc Back\""),
            "emoji picker should not keep custom footer labels"
        );
    }
}
