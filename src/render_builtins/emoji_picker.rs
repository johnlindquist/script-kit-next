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
            for category in crate::emoji::ALL_CATEGORIES.iter().copied() {
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

                let key_raw = event.keystroke.key.as_str();
                let key_str = key_raw.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                match this.route_key_to_actions_dialog(
                    &key_str,
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

                if key_str == "escape" && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                if has_cmd && key_str == "w" {
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
                    let navigated = match key_str.as_str() {
                        "up" | "arrowup" => {
                            *selected_index = (*selected_index).saturating_sub(cols);
                            true
                        }
                        "down" | "arrowdown" => {
                            *selected_index = (*selected_index + cols).min(filtered_len - 1);
                            true
                        }
                        "left" | "arrowleft" => {
                            this.navigate_emoji_picker_horizontal(-1, cx);
                            cx.stop_propagation();
                            return;
                        }
                        "right" | "arrowright" => {
                            this.navigate_emoji_picker_horizontal(1, cx);
                            cx.stop_propagation();
                            return;
                        }
                        "enter" | "return" => {
                            if let Some(emoji) = ordered_emojis.get(*selected_index) {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    emoji.emoji.to_string(),
                                ));
                                this.close_and_reset_window(cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        _ if key_raw == "Enter" => {
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
            let hover_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x30);
            let selected_border = self.theme.colors.accent.selected;
            // Keep selection visible but subtle in dense emoji rows (~50% outline, ~14% fill).
            let selected_outline = rgba((selected_border << 8) | 0x80);
            let selected_bg = rgba((self.theme.colors.accent.selected_subtle << 8) | 0x24);
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();
            let grid_cols = cols;

            uniform_list(
                "emoji-picker-grid",
                rows_for_list.len(),
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|row_index| match rows_for_list.get(row_index) {
                            Some(EmojiGridRow::Header { title, count }) => div()
                                .id(row_index)
                                .w_full()
                                .px(px(design_spacing.padding_lg))
                                .py(px(4.0))
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child(format!("{} {}", title, count))
                                .into_any_element(),
                            Some(EmojiGridRow::Cells { start_index, count }) => {
                                let click_entity_row = click_entity_handle.clone();
                                let hover_entity_row = hover_entity_handle.clone();
                                let emojis_for_row: Arc<Vec<crate::emoji::Emoji>> =
                                    Arc::clone(&emojis_for_list);
                                let row_start_index = *start_index;
                                let row_count = *count;

                                div()
                                    .id(row_index)
                                    .w_full()
                                    .flex()
                                    .px(px(design_spacing.padding_lg))
                                    .gap(px(1.0))
                                    .children((0..grid_cols).map(move |col| -> AnyElement {
                                        if col >= row_count {
                                            // Invisible spacer to maintain consistent cell width
                                            return div().flex_1().h(px(40.0)).into_any_element();
                                        }
                                        let flat_emoji_index = row_start_index + col;
                                        let emoji = match emojis_for_row.get(flat_emoji_index) {
                                            Some(e) => e,
                                            None => return div().flex_1().h(px(40.0)).into_any_element(),
                                        };
                                        let is_selected = flat_emoji_index == selected;
                                        let is_hovered = hovered == Some(flat_emoji_index)
                                            && current_input_mode == InputMode::Mouse;
                                        let click_entity = click_entity_row.clone();
                                        let hover_entity = hover_entity_row.clone();
                                        let emoji_value = emoji.emoji.to_string();
                                        let emoji_display = emoji_value.clone();

                                        div()
                                            .id(flat_emoji_index)
                                            .flex_1()
                                            .h(px(40.0))
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
                                                                Some(1200),
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
                                                    .size(px(34.0))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(px(6.0))
                                                    .text_size(px(26.0))
                                                    .border_1()
                                                    .border_color(if is_selected {
                                                        selected_outline
                                                    } else {
                                                        gpui::transparent_black().into()
                                                    })
                                                    .when(is_selected, |d| d.bg(selected_bg))
                                                    .when(is_hovered && !is_selected, |d| d.bg(hover_bg))
                                                    .child(emoji_display),
                                            )
                                            .into_any_element()
                                    }))
                                    .into_any_element()
                            }
                            None => div().id(row_index).h(px(36.0)).into_any_element(),
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

#[cfg(test)]
mod emoji_picker_tests {
    use std::fs;

    fn read_emoji_picker_source() -> String {
        fs::read_to_string("src/render_builtins/emoji_picker.rs")
            .expect("Failed to read src/render_builtins/emoji_picker.rs")
    }

    #[test]
    fn test_render_emoji_picker_builds_category_headers_and_eight_cell_rows() {
        let source = read_emoji_picker_source();

        assert!(
            source.contains("enum EmojiGridRow"),
            "render_emoji_picker should define local EmojiGridRow enum"
        );
        assert!(
            source.contains("Header { title: String, count: usize }"),
            "EmojiGridRow should include Header variant with title and count"
        );
        assert!(
            source.contains("Cells { start_index: usize, count: usize }"),
            "EmojiGridRow should include Cells variant with start index and count"
        );
        assert!(
            source.contains("(category_count - row_offset).min(cols)"),
            "emoji grid should chunk category rows using shared GRID_COLS constant"
        );
    }

    #[test]
    fn test_render_emoji_picker_handles_navigation_and_enter_copy() {
        let source = read_emoji_picker_source();

        for key_arm in [
            "\"up\" | \"arrowup\"",
            "\"down\" | \"arrowdown\"",
            "\"left\" | \"arrowleft\"",
            "\"right\" | \"arrowright\"",
            "\"enter\" | \"return\"",
        ] {
            assert!(
                source.contains(key_arm),
                "Expected key handler arm `{}` in render_emoji_picker",
                key_arm
            );
        }

        assert!(
            source.contains("cx.write_to_clipboard(gpui::ClipboardItem::new_string("),
            "render_emoji_picker should copy selected emoji on Enter/click"
        );
    }

    #[test]
    fn test_render_emoji_picker_wires_horizontal_input_actions_to_grid_navigation() {
        let source = read_emoji_picker_source();

        assert!(
            source.contains("gpui_component::input::MoveLeft"),
            "emoji picker should listen for MoveLeft action from Input"
        );
        assert!(
            source.contains("gpui_component::input::MoveRight"),
            "emoji picker should listen for MoveRight action from Input"
        );
        assert!(
            source.contains(".on_action(handle_move_left_action)")
                && source.contains(".on_action(handle_move_right_action)"),
            "emoji picker container should register left/right action handlers"
        );
    }

    #[test]
    fn test_render_emoji_picker_consumes_horizontal_arrow_keys() {
        let source = read_emoji_picker_source();

        let left_arm = source
            .find("\"left\" | \"arrowleft\" => {")
            .expect("left arrow key arm should exist");
        let left_section_end = (left_arm + 220).min(source.len());
        let left_section = &source[left_arm..left_section_end];
        assert!(
            left_section.contains("this.navigate_emoji_picker_horizontal(-1, cx);")
                && left_section.contains("cx.stop_propagation();"),
            "left arrow key handling should navigate grid and stop propagation to Input"
        );

        let right_arm = source
            .find("\"right\" | \"arrowright\" => {")
            .expect("right arrow key arm should exist");
        let right_section_end = (right_arm + 220).min(source.len());
        let right_section = &source[right_arm..right_section_end];
        assert!(
            right_section.contains("this.navigate_emoji_picker_horizontal(1, cx);")
                && right_section.contains("cx.stop_propagation();"),
            "right arrow key handling should navigate grid and stop propagation to Input"
        );
    }

    #[test]
    fn test_render_emoji_picker_uses_shared_input_focus_and_scroll_handles() {
        let source = read_emoji_picker_source();

        assert!(
            source.contains("Input::new(&self.gpui_input_state)"),
            "emoji picker header should use shared gpui input state"
        );
        assert!(
            source.contains(".track_focus(&self.focus_handle)"),
            "emoji picker should track focus with app focus handle"
        );
        assert!(
            source.contains(".track_scroll(&self.emoji_scroll_handle)"),
            "emoji picker grid should track emoji scroll handle"
        );
    }

    #[test]
    fn test_render_emoji_picker_uses_subtle_outline_when_cell_is_selected() {
        let source = read_emoji_picker_source();

        assert!(
            source.contains("let selected_outline = rgba((selected_border << 8) | 0x80);"),
            "selected emoji cell should use a subtle alpha-blended outline color"
        );
        assert!(
            source.contains(".border_1()"),
            "emoji cells should keep a 1px border for consistent layout"
        );
        assert!(
            source.contains("gpui::transparent_black().into()"),
            "unselected emoji cells should keep transparent borders to avoid layout shift"
        );
        assert!(
            source.contains(".when(is_selected, |d| d.bg(selected_bg))"),
            "selected emoji cell should apply subtle rounded background highlight"
        );
        assert!(
            source.contains(".size(px(34.0))") && source.contains(".h(px(40.0))"),
            "selected emoji indicator should use a tighter rounded square around the emoji"
        );
    }
}
