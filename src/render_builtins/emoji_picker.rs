#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmojiPickerEmptyState {
    NoEmojisFound,
    NoFilteredMatches,
}

impl EmojiPickerEmptyState {
    fn from_filter(filter: &str) -> Self {
        if filter.is_empty() {
            Self::NoEmojisFound
        } else {
            Self::NoFilteredMatches
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::NoEmojisFound => "No emojis found",
            Self::NoFilteredMatches => "No emojis match your filter",
        }
    }
}

impl ScriptListApp {
    /// Open or close ⌘K for the selected emoji.
    fn toggle_emoji_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.show_actions_popup {
            self.close_actions_popup(ActionsDialogHost::EmojiPicker, window, cx);
            cx.notify();
            return;
        }
        let Some(info) = self.selected_emoji_action_info() else {
            return;
        };
        let theme = std::sync::Arc::clone(&self.theme);
        let title = format!("{} {}", info.value, info.name);
        let placeholder = title.clone();
        let dialog = cx.new(move |cx| {
            let focus_handle = cx.focus_handle();
            let mut dialog = ActionsDialog::with_emoji(
                focus_handle,
                std::sync::Arc::new(|_action_id| {}), // Handled through handle_action.
                &info,
                theme,
            );
            dialog.set_config(crate::actions::ActionsDialogConfig {
                search_position: crate::actions::SearchPosition::Top,
                section_style: crate::actions::SectionStyle::Headers,
                anchor: crate::actions::AnchorPosition::Top,
                show_icons: true,
                search_placeholder: Some(placeholder),
                show_context_header: false,
                ..crate::actions::ActionsDialogConfig::default()
            });
            dialog
        });
        self.one_window_stage_in_view(
            ActionsDialogHost::EmojiPicker,
            dialog,
            Some(title),
            window,
            cx,
        );
        cx.notify();
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
        let _design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        let display =
            crate::emoji::display_ordered_emojis(&filter, selected_category, &self.emoji_head);
        let skin_tone = self.emoji_skin_tone;
        let ordered_emojis = display.emojis.clone();
        let cols = crate::emoji::GRID_COLS;

        #[derive(Clone)]
        enum EmojiGridRow {
            Header { title: String, count: usize },
            Cells { start_index: usize, count: usize },
        }

        let mut rows: Vec<EmojiGridRow> = Vec::new();
        {
            let mut flat_offset = 0;
            for section in &display.sections {
                rows.push(EmojiGridRow::Header {
                    title: section.title.clone(),
                    count: section.count,
                });
                let mut row_offset = 0;
                while row_offset < section.count {
                    let row_count = (section.count - row_offset).min(cols);
                    rows.push(EmojiGridRow::Cells {
                        start_index: flat_offset + row_offset,
                        count: row_count,
                    });
                    row_offset += row_count;
                }
                flat_offset += section.count;
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
                    ActionsRoute::Execute {
                        action_id,
                        should_close,
                    } => {
                        this.execute_actions_route_action(
                            ActionsDialogHost::EmojiPicker,
                            action_id,
                            should_close,
                            window,
                            cx,
                        );
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

                if is_key_enter(key) {
                    let selected = this.selected_emoji_action_info();
                    if let Some(emoji) = selected {
                        let keep_open = modifiers.alt;
                        let outcome = if has_cmd {
                            this.handle_emoji_action("emoji:emoji_copy", cx)
                        } else {
                            this.paste_emoji_from_picker(
                                &emoji.value,
                                &emoji.base,
                                &emoji.name,
                                keep_open,
                                cx,
                            )
                        };
                        this.show_outcome_feedback(&outcome, cx);
                    }
                    cx.stop_propagation();
                }
            },
        );

        let tile_size = crate::emoji::GRID_TILE_SIZE;
        let tile_gap = crate::emoji::GRID_TILE_GAP;
        let glyph_size = tile_size * crate::emoji::GRID_GLYPH_SCALE;
        const EMOJI_ROW_ID_OFFSET: usize = 10_000;
        const EMOJI_CELL_ID_OFFSET: usize = 20_000;

        let grid_element: AnyElement = if filtered_len == 0 {
            crate::components::render_simple_empty_state(
                "emoji-picker-empty",
                EmojiPickerEmptyState::from_filter(&filter).message(),
                "star",
                None,
                &self.theme,
                cx,
            )
        } else {
            let row_height = crate::emoji::GRID_ROW_HEIGHT;
            let selected_outline = rgba((self.theme.colors.accent.selected << 8) | 0x80);
            let idle_outline = rgba(self.theme.colors.accent.selected << 8);
            let selected_bg = rgba((self.theme.colors.text.primary << 8) | 0x2a);
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
                                    .justify_center()
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
                                    let emoji_base = emoji.emoji.to_string();
                                    let emoji_name = emoji.name.to_string();
                                    let emoji_value = emoji.with_tone(skin_tone).to_string();
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
                                                        app.update(cx, |this, cx| {
                                                            let outcome = this.paste_emoji_from_picker(
                                                                &emoji_value,
                                                                &emoji_base,
                                                                &emoji_name,
                                                                false,
                                                                cx,
                                                            );
                                                            this.show_outcome_feedback(&outcome, cx);
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
                                                    .border_1()
                                                    .border_color(idle_outline)
                                                    .bg(idle_bg)
                                                    .text_size(px(glyph_size))
                                                    .when(is_selected, |d| {
                                                        d.bg(selected_bg)
                                                            .border_color(selected_outline)
                                                    })
                                                    .child(emoji_display),
                                            ),
                                    );
                                }

                                for _ in *count..cols {
                                    row_div = row_div
                                        .child(div().w(px(tile_size)).h(px(tile_size)));
                                }

                                row_div.into_any_element()
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

        let list_scrollbar =
            self.builtin_uniform_list_scrollbar(&self.emoji_scroll_handle, rows.len(), 8);
        let list_colors = crate::list_item::ListItemColors::from_theme(&self.theme);

        let content = div()
            .flex_1()
            .min_h(px(0.0))
            .w_full()
            .overflow_hidden()
            .py(px(design_spacing.padding_xs))
            .child(
                crate::components::builtin_leading_separator::render_builtin_leading_separator(
                    if filter.trim().is_empty() {
                        "Emoji"
                    } else {
                        "Results"
                    },
                    None,
                    list_colors,
                    self.current_main_menu_theme,
                ),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .h_full()
                    .child(grid_element)
                    .child(list_scrollbar),
            );

        crate::components::emit_prompt_chrome_audit(&crate::components::PromptChromeAudit::grid(
            "render_builtins::emoji_picker",
            true,
        ));

        // Enter records usage, copies the emoji, and pastes it into the
        // previous app — "Paste" is the truthful primary verb (same precedent
        // as clipboard history), not the universal "Run".
        let emoji_hints = crate::components::universal_prompt_hints_with_primary_label("Paste");
        // Runtime receipt that the relabeled `↵ Paste` anatomy stays universal
        // (chaos battery 06/OF-3): every peer list builtin emits this audit;
        // the emoji picker was the one surface with no is_universal receipt.
        crate::components::emit_prompt_hint_audit("emoji_picker", &emoji_hints);
        let gpui_footer = crate::components::render_simple_hint_strip(emoji_hints, None);
        let footer = self.main_window_footer_slot(gpui_footer);
        let menu_def = self.current_main_menu_theme.def();
        let shell = menu_def.shell;

        crate::components::main_view_chrome::render_main_view_chrome_footer_flush(
            crate::components::main_view_chrome::render_main_view_shell()
                .text_color(rgb(text_primary))
                .font_family(self.theme_font_family())
                .key_context("emoji_picker")
                .track_focus(&self.focus_handle)
                .on_key_down(handle_key),
            &self.theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header: self.render_builtin_main_input_header(
                    vec![self
                        .render_builtin_main_input_count_label(format!("{} emojis", filtered_len))],
                    cx,
                ),
                divider: crate::components::main_view_chrome::MainViewDividerChrome {
                    margin_x: shell.divider_margin_x,
                    height: shell.divider_height,
                    visible: shell.divider_height > 0.0,
                },
                main: content.into_any_element(),
                footer,
                overlays: Vec::new(),
            },
        )
    }
}

#[cfg(test)]
mod emoji_picker_chrome_audit {
    #[test]
    fn emoji_picker_uses_native_footer_slot() {
        let source = include_str!("emoji_picker.rs");
        assert!(
            source.contains("main_window_footer_slot("),
            "emoji_picker should route its GPUI footer through main_window_footer_slot"
        );
        assert!(
            source.contains("render_simple_hint_strip("),
            "emoji_picker should build its hint strip via render_simple_hint_strip"
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
        // Enter on this surface copies + pastes + hides — the shared three-key
        // footer must say "Paste", not "Run" (same contract as clipboard
        // history). A footer hint may never advertise a verb Enter doesn't do.
        assert!(
            source.contains("universal_prompt_hints_with_primary_label(\"Paste\")"),
            "emoji picker should use the shared three-key footer with the truthful Paste primary verb"
        );
        assert!(
            !source.contains("\"↵ Copy\"") && !source.contains("\"Esc Back\""),
            "emoji picker should not keep custom footer labels"
        );
    }
}
