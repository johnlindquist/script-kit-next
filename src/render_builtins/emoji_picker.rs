impl ScriptListApp {
    fn navigate_emoji_picker_horizontal(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let AppView::EmojiPickerView {
            filter,
            selected_index,
            selected_category,
        } = &mut self.current_view
        {
            let ordered_emojis = crate::emoji::filtered_ordered_emojis(filter, *selected_category);
            let filtered_len = ordered_emojis.len();
            if filtered_len == 0 {
                *selected_index = 0;
                self.hovered_index = None;
                cx.notify();
                return;
            }

            if *selected_index >= filtered_len {
                *selected_index = filtered_len - 1;
            }

            if delta < 0 {
                let step = delta.saturating_neg() as usize;
                *selected_index = selected_index.saturating_sub(step);
            } else {
                let step = delta as usize;
                *selected_index = selected_index.saturating_add(step).min(filtered_len - 1);
            }

            let row = crate::emoji::compute_scroll_row(*selected_index, &ordered_emojis);
            self.emoji_scroll_handle
                .scroll_to_item(row, gpui::ScrollStrategy::Nearest);

            self.input_mode = InputMode::Keyboard;
            self.hovered_index = None;
            cx.notify();
        }
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
                        this.handle_action(action_id, cx);
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
                    let filtered_len = ordered_emojis.len();
                    if filtered_len == 0 {
                        *selected_index = 0;
                        this.hovered_index = None;
                        cx.notify();
                        return;
                    }

                    if *selected_index >= filtered_len {
                        *selected_index = filtered_len - 1;
                    }

                    let cols = crate::emoji::GRID_COLS;
                    let navigated = match key {
                        _ if is_key_up(key) => {
                            *selected_index = (*selected_index).saturating_sub(cols);
                            true
                        }
                        _ if is_key_down(key) => {
                            *selected_index = (*selected_index + cols).min(filtered_len - 1);
                            true
                        }
                        _ if is_key_left(key) => {
                            this.navigate_emoji_picker_horizontal(-1, cx);
                            cx.stop_propagation();
                            return;
                        }
                        _ if is_key_right(key) => {
                            this.navigate_emoji_picker_horizontal(1, cx);
                            cx.stop_propagation();
                            return;
                        }
                        _ if is_key_enter(key) => {
                            if let Some(emoji) = ordered_emojis.get(*selected_index) {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    emoji.emoji.to_string(),
                                ));
                                this.close_and_reset_window(cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        _ => return,
                    };

                    if navigated {
                        let row =
                            crate::emoji::compute_scroll_row(*selected_index, &ordered_emojis);
                        this.emoji_scroll_handle
                            .scroll_to_item(row, gpui::ScrollStrategy::Nearest);
                        cx.stop_propagation();
                    }

                    this.input_mode = InputMode::Keyboard;
                    this.hovered_index = None;
                    cx.notify();
                }
            },
        );

        let handle_move_left_action = cx.listener(
            |this: &mut Self,
             _: &gpui_component::input::MoveLeft,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                if this.shortcut_recorder_state.is_some() {
                    cx.stop_propagation();
                    return;
                }

                this.navigate_emoji_picker_horizontal(-1, cx);
                cx.stop_propagation();
            },
        );

        let handle_move_right_action = cx.listener(
            |this: &mut Self,
             _: &gpui_component::input::MoveRight,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                if this.shortcut_recorder_state.is_some() {
                    cx.stop_propagation();
                    return;
                }

                this.navigate_emoji_picker_horizontal(1, cx);
                cx.stop_propagation();
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
            let rows_for_list = rows.clone();
            let emojis_for_list: Arc<Vec<crate::emoji::Emoji>> = Arc::new(ordered_emojis.clone());
            let selected = selected_index;
            let hovered = self.hovered_index;
            let current_input_mode = self.input_mode;
            let row_height = crate::emoji::GRID_ROW_HEIGHT;
            let cell_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x18);
            let hover_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x2c);
            let cell_border = rgba((ui_border << 8) | 0x3c);
            let selected_border = self.theme.colors.accent.selected;
            let selected_outline = rgba((selected_border << 8) | 0x80);
            let selected_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x36);
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

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
                                let click_entity_row = click_entity_handle.clone();
                                let hover_entity_row = hover_entity_handle.clone();
                                let emojis_for_row: Arc<Vec<crate::emoji::Emoji>> =
                                    Arc::clone(&emojis_for_list);
                                let row_start_index = *start_index;
                                let row_count = *count;

                                div()
                                    .id(EMOJI_ROW_ID_OFFSET + row_index)
                                    .w_full()
                                    .h(px(row_height))
                                    .flex()
                                    .items_center()
                                    .px(px(design_spacing.padding_lg))
                                    .gap(px(tile_gap))
                                    .children((0..row_count).map(move |col| -> AnyElement {
                                        let flat_emoji_index = row_start_index + col;
                                        let emoji = match emojis_for_row.get(flat_emoji_index) {
                                            Some(e) => e,
                                            None => return div().w(px(tile_size)).h(px(tile_size)).into_any_element(),
                                        };
                                        let is_selected = flat_emoji_index == selected;
                                        let is_hovered = hovered == Some(flat_emoji_index)
                                            && current_input_mode == InputMode::Mouse;
                                        let click_entity = click_entity_row.clone();
                                        let hover_entity = hover_entity_row.clone();
                                        let emoji_value = emoji.emoji.to_string();
                                        let emoji_display = emoji_value.clone();

                                        div()
                                            .id(EMOJI_CELL_ID_OFFSET + flat_emoji_index)
                                            .w(px(tile_size))
                                            .h(px(tile_size))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .tooltip(|window, cx| {
                                                gpui_component::tooltip::Tooltip::new("Copy emoji")
                                                    .key_binding(
                                                        gpui::Keystroke::parse("enter")
                                                            .ok()
                                                            .map(gpui_component::kbd::Kbd::new),
                                                    )
                                                    .build(window, cx)
                                            })
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
                                            .on_hover(
                                                move |is_hov: &bool,
                                                      _window: &mut Window,
                                                      cx: &mut gpui::App| {
                                                    if let Some(app) = hover_entity.upgrade() {
                                                        app.update(cx, |this, cx| {
                                                            if *is_hov {
                                                                this.input_mode = InputMode::Mouse;
                                                                if this.hovered_index
                                                                    != Some(flat_emoji_index)
                                                                {
                                                                    this.hovered_index =
                                                                        Some(flat_emoji_index);
                                                                    cx.notify();
                                                                }
                                                            } else if this.hovered_index
                                                                == Some(flat_emoji_index)
                                                            {
                                                                this.hovered_index = None;
                                                                cx.notify();
                                                            }
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
                                                    .text_size(px(28.0))
                                                    .border_1()
                                                    .bg(cell_bg)
                                                    .border_color(if is_selected {
                                                        selected_outline
                                                    } else {
                                                        cell_border
                                                    })
                                                    .when(is_hovered && !is_selected, |d| d.bg(hover_bg))
                                                    .when(is_selected, |d| d.bg(selected_bg))
                                                    .child(emoji_display),
                                            )
                                            .into_any_element()
                                    }))
                                    .child(div().flex_1())
                                    .into_any_element()
                            }
                            None => div().id(EMOJI_ROW_ID_OFFSET + row_index).h(px(row_height)).into_any_element(),
                        })
                        .collect()
                },
            )
            .w_full()
            .h_full()
            .track_scroll(&self.emoji_scroll_handle)
            .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("emoji_picker")
            .track_focus(&self.focus_handle)
            .on_action(handle_move_left_action)
            .on_action(handle_move_right_action)
            .on_key_down(handle_key)
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
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
                    ),
            )
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.0))
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .child(grid_element),
            )
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Copy")
                    .primary_shortcut("↵")
                    .show_secondary(false),
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }
}
