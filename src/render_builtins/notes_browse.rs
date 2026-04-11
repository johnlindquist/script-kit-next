impl ScriptListApp {
    fn notes_browse_filtered_notes(filter: &str) -> Vec<crate::notes::Note> {
        let result = if filter.trim().is_empty() {
            crate::notes::get_all_notes()
        } else {
            crate::notes::search_notes(filter)
        };

        match result {
            Ok(notes) => notes,
            Err(error) => {
                tracing::warn!(
                    event = "notes_browse_portal_load_failed",
                    filter = %filter,
                    error = %error,
                );
                Vec::new()
            }
        }
    }

    fn notes_browse_preview(content: &str) -> String {
        const LIMIT: usize = 280;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return "Empty note".to_string();
        }
        let mut preview: String = trimmed.chars().take(LIMIT).collect();
        if trimmed.chars().count() > LIMIT {
            preview.push('…');
        }
        preview
    }

    fn build_notes_browse_portal_part(
        &self,
        index: usize,
        note: &crate::notes::Note,
    ) -> crate::ai::message_parts::AiContextPart {
        let title = if note.title.trim().is_empty() {
            "Untitled Note".to_string()
        } else {
            note.title.clone()
        };
        let target = crate::ai::TabAiTargetContext {
            source: "NotesBrowse".to_string(),
            kind: "note".to_string(),
            semantic_id: crate::protocol::generate_semantic_id("note", index, &note.id.as_str()),
            label: title.clone(),
            metadata: Some(serde_json::json!({
                "noteId": note.id.as_str(),
                "title": title,
                "content": note.content,
                "preview": Self::notes_browse_preview(&note.content),
                "isPinned": note.is_pinned,
                "createdAt": note.created_at.to_rfc3339(),
                "updatedAt": note.updated_at.to_rfc3339(),
            })),
        };
        let label = crate::ai::format_explicit_target_chip_label(&target);
        crate::ai::message_parts::AiContextPart::FocusedTarget { target, label }
    }

    fn render_notes_browse_portal(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use gpui_component::scroll::ScrollableElement as _;

        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::expanded("notes_browse", false),
        );

        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let text_muted = self.theme.colors.text.muted;

        let filtered_notes = Self::notes_browse_filtered_notes(&filter);
        let total_notes = filtered_notes.len();
        let preview_note = filtered_notes.get(selected_index).cloned();
        let in_portal = self.is_in_attachment_portal();

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

                if crate::ui_foundation::is_key_escape(key) && !this.show_actions_popup {
                    if !this.clear_builtin_view_filter(cx) {
                        if this.is_in_attachment_portal() {
                            this.close_attachment_portal_cancel(cx);
                        } else {
                            this.go_back_or_close(window, cx);
                        }
                    }
                    cx.stop_propagation();
                    return;
                }

                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                let Some((current_filter, current_selected)) =
                    (match &this.current_view {
                        AppView::NotesBrowseView {
                            filter,
                            selected_index,
                        } => Some((filter.clone(), *selected_index)),
                        _ => None,
                    })
                else {
                    return;
                };

                let notes = Self::notes_browse_filtered_notes(&current_filter);
                let note_count = notes.len();

                if crate::ui_foundation::is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::NotesBrowseView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                            this.notes_browse_scroll_handle.scroll_to_item(*selected_index);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if crate::ui_foundation::is_key_down(key) {
                    if current_selected < note_count.saturating_sub(1) {
                        if let AppView::NotesBrowseView { selected_index, .. } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                            this.notes_browse_scroll_handle.scroll_to_item(*selected_index);
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if crate::ui_foundation::is_key_enter(key) {
                    if let Some(note) = notes.get(current_selected) {
                        if this.is_in_attachment_portal() {
                            let part =
                                this.build_notes_browse_portal_part(current_selected, note);
                            this.close_attachment_portal_with_part(part, cx);
                        }
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        let list_colors = ListItemColors::from_theme(&self.theme);
        let list_element: AnyElement = if filtered_notes.is_empty() {
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_muted))
                .font_family(design_typography.font_family)
                .child(if filter.is_empty() {
                    "No notes yet"
                } else {
                    "No notes match your filter"
                })
                .into_any_element()
        } else {
            let notes_for_closure = filtered_notes.clone();
            let selected = selected_index;

            div()
                .id("notes-browse-list")
                .w_full()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .track_scroll(&self.notes_browse_scroll_handle)
                .overflow_y_scrollbar()
                .children(notes_for_closure.into_iter().enumerate().map(
                    move |(display_ix, note)| {
                        let is_selected = display_ix == selected;
                        let title = if note.title.trim().is_empty() {
                            "Untitled Note".to_string()
                        } else {
                            note.title.clone()
                        };
                        let description = format!(
                            "{} · {} chars{}",
                            note.updated_at.format("%Y-%m-%d %H:%M"),
                            note.content.chars().count(),
                            if note.is_pinned { " · pinned" } else { "" }
                        );

                        let item = ListItem::new(title, list_colors)
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

        let preview_panel: AnyElement = match preview_note {
            Some(note) => {
                let title = if note.title.trim().is_empty() {
                    "Untitled Note".to_string()
                } else {
                    note.title.clone()
                };

                div()
                    .w_full()
                    .h_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .font_family(design_typography.font_family)
                    .child(
                        div()
                            .pb(px(design_spacing.padding_md))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(note.updated_at.format("%Y-%m-%d %H:%M").to_string()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_primary))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(title),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(if note.content.trim().is_empty() {
                                "Empty note".to_string()
                            } else {
                                note.content
                            }),
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
                .child("No note selected")
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
                    gpui_component::input::Input::new(&self.gpui_input_state)
                        .w_full()
                        .h(px(28.))
                        .px(px(0.))
                        .py(px(0.))
                        .with_size(gpui_component::Size::Size(px(
                            design_typography.font_size_xl,
                        )))
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
                        "{} note{}",
                        total_notes,
                        if total_notes == 1 { "" } else { "s" }
                    )),
            );

        let list_pane = div()
            .relative()
            .w_full()
            .h_full()
            .min_h(px(0.))
            .py(px(design_spacing.padding_xs))
            .child(list_element);

        let hints: Vec<SharedString> = if in_portal {
            vec!["↵ Attach Note".into(), "Esc Cancel".into()]
        } else {
            vec!["Esc Back".into()]
        };
        crate::components::emit_prompt_hint_audit("notes_browse", &hints);

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
        .key_context("notes_browse")
        .track_focus(&self.focus_handle)
        .on_key_down(handle_key)
        .into_any_element()
    }
}
