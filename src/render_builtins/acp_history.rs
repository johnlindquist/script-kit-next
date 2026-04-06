impl ScriptListApp {
    /// Render the ACP conversation history browser (list + preview).
    fn render_acp_history(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use gpui_component::scroll::ScrollableElement as _;

        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::expanded("acp_history", false),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let _design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;

        // Load history entries on each render (small JSONL, fast)
        let all_entries = crate::ai::acp::history::load_history();

        // Filter
        let filtered_entries: Vec<(usize, &crate::ai::acp::history::AcpHistoryEntry)> =
            if filter.is_empty() {
                all_entries.iter().enumerate().collect()
            } else {
                let filter_lower = filter.to_lowercase();
                all_entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        e.first_message.to_lowercase().contains(&filter_lower)
                            || e.timestamp.to_lowercase().contains(&filter_lower)
                    })
                    .collect()
            };
        let filtered_len = filtered_entries.len();

        // Load preview for selected entry
        let selected_session_id = filtered_entries
            .get(selected_index)
            .map(|(_, e)| e.session_id.clone());
        let preview_conversation = selected_session_id
            .as_deref()
            .and_then(crate::ai::acp::history::load_conversation);

        // Key handler
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
                let has_cmd = event.keystroke.modifiers.platform;
                let modifiers = &event.keystroke.modifiers;

                // Route keys to actions dialog first if open
                match this.route_key_to_actions_dialog(
                    key,
                    event.keystroke.key_char.as_deref(),
                    modifiers,
                    ActionsDialogHost::AcpHistory,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {}
                    ActionsRoute::Handled => return,
                    ActionsRoute::Execute { action_id } => {
                        this.handle_action(action_id, window, cx);
                        return;
                    }
                }

                // ESC: Clear filter first if present, otherwise go back/close
                if is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                // Extract current view state
                let view_state = if let AppView::AcpHistoryView {
                    filter,
                    selected_index,
                } = &this.current_view
                {
                    Some((filter.clone(), *selected_index))
                } else {
                    None
                };

                let Some((current_filter, current_selected)) = view_state else {
                    return;
                };

                // Recompute filtered list
                let entries = crate::ai::acp::history::load_history();
                let filtered: Vec<(usize, &crate::ai::acp::history::AcpHistoryEntry)> =
                    if current_filter.is_empty() {
                        entries.iter().enumerate().collect()
                    } else {
                        let fl = current_filter.to_lowercase();
                        entries
                            .iter()
                            .enumerate()
                            .filter(|(_, e)| {
                                e.first_message.to_lowercase().contains(&fl)
                                    || e.timestamp.to_lowercase().contains(&fl)
                            })
                            .collect()
                    };
                let current_filtered_len = filtered.len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::AcpHistoryView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.acp_history_scroll_handle
                                .scroll_to_item(*selected_index);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_down(key) {
                    if current_selected < current_filtered_len.saturating_sub(1) {
                        if let AppView::AcpHistoryView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.acp_history_scroll_handle
                                .scroll_to_item(*selected_index);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_enter(key) {
                    // Resume: load the selected conversation into ACP chat
                    if let Some((_, entry)) = filtered.get(current_selected) {
                        let session_id = entry.session_id.clone();
                        let first_message = entry.first_message.clone();
                        tracing::info!(
                            event = "acp_history_item_resumed",
                            session_id = %session_id,
                        );
                        this.resume_acp_conversation_from_history(
                            &session_id,
                            &first_message,
                            window,
                            cx,
                        );
                    }
                    cx.stop_propagation();
                } else if key.eq_ignore_ascii_case("backspace") && has_cmd {
                    // Cmd+Backspace: delete selected conversation
                    if let Some((_, entry)) = filtered.get(current_selected) {
                        let session_id = entry.session_id.clone();
                        if let Err(e) = crate::ai::acp::history::delete_conversation(&session_id) {
                            tracing::warn!(
                                event = "acp_history_delete_failed",
                                session_id = %session_id,
                                error = %e,
                            );
                        } else {
                            // Clamp selection after delete
                            let new_len = current_filtered_len.saturating_sub(1);
                            if let AppView::AcpHistoryView { selected_index, .. } =
                                &mut this.current_view
                            {
                                if *selected_index >= new_len && new_len > 0 {
                                    *selected_index = new_len - 1;
                                } else if new_len == 0 {
                                    *selected_index = 0;
                                }
                                this.acp_history_scroll_handle
                                    .scroll_to_item(*selected_index);
                            }
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        // Build list
        let list_colors = ListItemColors::from_theme(&self.theme);

        let list_element: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No conversation history"
                } else {
                    "No conversations match your filter"
                })
                .into_any_element()
        } else {
            let entries_for_closure: Vec<(
                usize,
                crate::ai::acp::history::AcpHistoryEntry,
            )> = filtered_entries
                .iter()
                .map(|(i, e)| (*i, (*e).clone()))
                .collect();
            let selected = selected_index;

            div()
                .id("acp-history-list")
                .w_full()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .track_scroll(&self.acp_history_scroll_handle)
                .overflow_y_scrollbar()
                .children(entries_for_closure.into_iter().enumerate().map(
                    move |(display_ix, (_original_ix, entry))| {
                        let is_selected = display_ix == selected;

                        // Truncate first message for display
                        let name = if entry.first_message.len() > 80 {
                            format!("{}…", &entry.first_message[..80])
                        } else {
                            entry.first_message.clone()
                        };

                        let description = format!(
                            "{} messages · {}",
                            entry.message_count, entry.timestamp
                        );

                        let item = ListItem::new(name, list_colors)
                            .description_opt(Some(description))
                            .selected(is_selected)
                            .with_accent_bar(true);

                        div()
                            .id(gpui::ElementId::Integer(display_ix as u64))
                            .child(item)
                    },
                ))
                .into_any_element()
        };

        // Build preview panel
        let preview_panel: AnyElement = match &preview_conversation {
            Some(conv) => {
                let mut blocks: Vec<AnyElement> = Vec::new();
                for msg in &conv.messages {
                    let role_color = if msg.role == "user" {
                        text_primary
                    } else {
                        text_dimmed
                    };
                    blocks.push(
                        div()
                            .w_full()
                            .pb(px(design_spacing.padding_md))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(msg.role.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(role_color))
                                    .child(msg.body.clone()),
                            )
                            .into_any_element(),
                    );
                }
                div()
                    .w_full()
                    .h_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .font_family(design_typography.font_family)
                    .children(blocks)
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
                .child("No conversation selected")
                .into_any_element(),
        };

        // Header with input and count
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
                        "{} conversation{}",
                        all_entries.len(),
                        if all_entries.len() == 1 { "" } else { "s" }
                    )),
            );

        // List pane
        let list_pane = div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .py(px(design_spacing.padding_xs))
            .child(list_element);

        let hints: Vec<SharedString> = vec![
            "↵ Resume".into(),
            "⌘⌫ Delete".into(),
            "Esc Back".into(),
        ];
        crate::components::emit_prompt_hint_audit("acp_history", &hints);

        // Assemble via shared expanded-view scaffold
        crate::components::render_expanded_view_scaffold_with_hints(
            header_element,
            list_pane,
            preview_panel,
            hints,
            None,
        )
        .text_color(rgb(text_primary))
        .font_family(design_typography.font_family)
        .key_context("acp_history")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }

    /// Resume an ACP conversation from history by opening ACP chat with
    /// the saved messages loaded.
    fn resume_acp_conversation_from_history(
        &mut self,
        session_id: &str,
        first_message: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(chat_entity) = crate::ai::acp::chat_window::get_detached_acp_view_entity() {
            let resumed = chat_entity.update(cx, |chat_view, cx| {
                chat_view.resume_from_history(session_id, cx)
            });
            if !resumed {
                let fallback_input = first_message.to_string();
                chat_entity.update(cx, |chat_view, cx| {
                    chat_view.set_input(fallback_input, cx);
                });
            }

            self.reset_to_script_list(cx);
            return;
        }

        self.open_tab_ai_acp_with_entry_intent(None, cx);

        if let AppView::AcpChatView { entity } = &self.current_view {
            let resumed =
                entity.update(cx, |chat_view, cx| chat_view.resume_from_history(session_id, cx));
            if !resumed {
                entity.update(cx, |chat_view, cx| {
                    chat_view.set_input(first_message.to_string(), cx);
                });
            }
        }
    }
}

#[cfg(test)]
mod acp_history_scroll_contract {
    const SOURCE: &str = include_str!("acp_history.rs");

    #[test]
    fn acp_history_tracks_scroll_and_keeps_selection_visible() {
        assert!(
            SOURCE.contains(".track_scroll(&self.acp_history_scroll_handle)"),
            "ACP history list should track scroll so selection changes can reposition the viewport"
        );
        assert!(
            SOURCE.contains("this.acp_history_scroll_handle"),
            "ACP history keyboard navigation should scroll the selected row into view"
        );
    }
}
