use super::*;

impl Focusable for SelectPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SelectPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_ctrl = event.keystroke.modifiers.platform; // Cmd on macOS, Ctrl on others

                // Handle Ctrl/Cmd+A for select all
                if has_ctrl && key_str == "a" {
                    this.toggle_select_all_filtered(cx);
                    return;
                }

                match key_str.as_str() {
                    "up" | "arrowup" => this.move_up(cx),
                    "down" | "arrowdown" => this.move_down(cx),
                    "space" | " " => {
                        if has_ctrl {
                            this.toggle_selection(cx);
                        } else {
                            this.handle_char(' ', cx);
                        }
                    }
                    "enter" | "return" => this.submit(),
                    "escape" | "esc" => this.submit_cancel(),
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if should_append_to_filter(ch) {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        let (_main_bg, text_color, muted_color, border_color) =
            if self.design_variant == DesignVariant::Default {
                (
                    rgb(self.theme.colors.background.main),
                    rgb(self.theme.colors.text.secondary),
                    rgb(self.theme.colors.text.muted),
                    rgb(self.theme.colors.ui.border),
                )
            } else {
                (
                    rgb(colors.background),
                    rgb(colors.text_secondary),
                    rgb(colors.text_muted),
                    rgb(colors.border),
                )
            };
        let search_box_bg = rgb(resolve_search_box_bg_hex(
            &self.theme,
            self.design_variant,
            &colors,
        ));

        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| "Search...".to_string());

        let input_display = if self.filter_text.is_empty() {
            SharedString::from(placeholder)
        } else {
            SharedString::from(self.filter_text.clone())
        };

        // Search input
        let input_container = div()
            .id(gpui::ElementId::Name("input:select-filter".into()))
            .w_full()
            .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
            .px(px(spacing.item_padding_x))
            .py(px(spacing.padding_md))
            .bg(search_box_bg)
            .border_b_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(div().text_color(muted_color).child("🔍"))
            .child(
                div()
                    .flex_1()
                    .text_color(if self.filter_text.is_empty() {
                        muted_color
                    } else {
                        text_color
                    })
                    .child(input_display),
            )
            .when(self.multiple, |container| {
                container.child(
                    div()
                        .text_sm()
                        .text_color(muted_color)
                        .child(format!("{} selected", self.selected.len())),
                )
            });

        // Choices list
        let filtered_len = self.filtered_choices.len();
        let choices_content: AnyElement = if filtered_len == 0 {
            let empty_message = if self.filter_text.trim().is_empty() {
                "No choices available"
            } else {
                "No choices match your filter"
            };
            div()
                .w_full()
                .py(px(spacing.padding_xl))
                .px(px(spacing.item_padding_x))
                .text_color(muted_color)
                .child(empty_message)
                .into_any_element()
        } else {
            uniform_list(
                "select-choices",
                filtered_len,
                cx.processor(
                    move |this: &mut SelectPrompt,
                          visible_range: std::ops::Range<usize>,
                          _window,
                          _cx| {
                        let row_colors = UnifiedListItemColors::from_theme(&this.theme);
                        let mut rows = Vec::with_capacity(visible_range.len());

                        for display_idx in visible_range {
                            if let Some(&choice_idx) = this.filtered_choices.get(display_idx) {
                                if let Some(choice) = this.choices.get(choice_idx) {
                                    if let Some(indexed_choice) = this.choice_index.get(choice_idx)
                                    {
                                        let is_focused = display_idx == this.focused_index;
                                        let is_selected = this.selected.contains(&choice_idx);
                                        let is_selected_for_ui = if this.multiple {
                                            is_selected
                                        } else {
                                            is_focused
                                        };
                                        let semantic_id =
                                            choice.semantic_id.clone().unwrap_or_else(|| {
                                                indexed_choice.stable_semantic_id.clone()
                                            });
                                        let indicator =
                                            choice_selection_indicator(
                                                this.multiple,
                                                is_selected_for_ui,
                                            );
                                        let subtitle = indexed_choice
                                            .metadata
                                            .subtitle_text()
                                            .map(TextContent::plain);
                                        let title = highlighted_choice_title(
                                            &choice.name,
                                            &this.filter_text,
                                        );
                                        let trailing =
                                            indexed_choice.metadata.shortcut.clone().map(
                                                |shortcut| {
                                                    TrailingContent::Shortcut(SharedString::from(
                                                        shortcut,
                                                    ))
                                                },
                                            );

                                        rows.push(
                                            div()
                                                .id(display_idx)
                                                .w_full()
                                                .h(px(LIST_ITEM_HEIGHT))
                                                .border_b_1()
                                                .border_color(border_color)
                                                .child(
                                                    UnifiedListItem::new(
                                                        gpui::ElementId::Name(semantic_id.into()),
                                                        title,
                                                    )
                                                    .subtitle_opt(subtitle)
                                                    .leading(LeadingContent::Emoji(
                                                        indicator.into(),
                                                    ))
                                                    .trailing_opt(trailing)
                                                    .state(ItemState {
                                                        is_selected: is_focused,
                                                        is_hovered: false,
                                                        is_disabled: false,
                                                    })
                                                    .density(Density::Comfortable)
                                                    .colors(row_colors)
                                                    .with_accent_bar(is_selected_for_ui),
                                                ),
                                        );
                                    }
                                }
                            }
                        }

                        rows
                    },
                ),
            )
            .h_full()
            .w_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };

        let choices_container = div()
            .id(gpui::ElementId::Name("list:select-choices".into()))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .child(choices_content);

        div()
            .id(gpui::ElementId::Name("window:select".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // Only apply bg when vibrancy disabled
            .text_color(text_color)
            .key_context("select_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(input_container)
            .child(choices_container)
    }
}
