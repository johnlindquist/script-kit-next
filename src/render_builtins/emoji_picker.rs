impl ScriptListApp {
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

        let ordered_emojis =
            crate::emoji::filtered_ordered_emojis(&filter, selected_category);
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
                    match key_str.as_str() {
                        "up" | "arrowup" => {
                            *selected_index = (*selected_index).saturating_sub(cols);
                        }
                        "down" | "arrowdown" => {
                            *selected_index = (*selected_index + cols).min(filtered_len - 1);
                        }
                        "left" | "arrowleft" => {
                            *selected_index = (*selected_index).saturating_sub(1);
                        }
                        "right" | "arrowright" => {
                            *selected_index = (*selected_index + 1).min(filtered_len - 1);
                        }
                        "enter" | "return" => {
                            if let Some(emoji) = ordered_emojis.get(*selected_index) {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    emoji.emoji.to_string(),
                                ));
                                this.close_and_reset_window(cx);
                            }
                            return;
                        }
                        _ if key_raw == "Enter" => {
                            if let Some(emoji) = ordered_emojis.get(*selected_index) {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    emoji.emoji.to_string(),
                                ));
                                this.close_and_reset_window(cx);
                            }
                            return;
                        }
                        _ => return,
                    }

                    this.input_mode = InputMode::Keyboard;
                    this.hovered_index = None;
                    cx.notify();
                }
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
            let click_entity_handle = cx.entity().downgrade();
            let hover_entity_handle = cx.entity().downgrade();

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
                                    .gap(px(2.0))
                                    .children((0..row_count).filter_map(move |cell_offset| {
                                        let flat_emoji_index = row_start_index + cell_offset;
                                        let emoji = emojis_for_row.get(flat_emoji_index)?;
                                        let is_selected = flat_emoji_index == selected;
                                        let is_hovered = hovered == Some(flat_emoji_index)
                                            && current_input_mode == InputMode::Mouse;
                                        let click_entity = click_entity_row.clone();
                                        let hover_entity = hover_entity_row.clone();
                                        let emoji_value = emoji.emoji.to_string();
                                        let emoji_display = emoji_value.clone();

                                        Some(
                                            div()
                                                .id(flat_emoji_index)
                                                .flex_1()
                                                .h(px(44.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .rounded(px(6.0))
                                                .text_size(px(26.0))
                                                .cursor_pointer()
                                                .border_2()
                                                .border_color(if is_selected {
                                                    rgb(selected_border)
                                                } else {
                                                    rgba(0x00000000)
                                                })
                                                .when(is_hovered && !is_selected, |d| d.bg(hover_bg))
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
                                                .child(emoji_display),
                                        )
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
}
