use super::*;

impl NotesApp {
    pub(super) fn render_editor_footer(
        &self,
        is_preview: bool,
        in_focus_mode: bool,
        window_hovered: bool,
        char_count: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let word_count = self.get_word_count(cx);
        let cursor_line_info = self.get_cursor_line_info(cx);
        let selection_stats = self.get_selection_stats(cx);
        let note_position = self.get_note_position();
        let has_unsaved = self.has_unsaved_changes;
        let show_saved = !has_unsaved
            && self
                .last_save_confirmed
                .map(|t| t.elapsed() < Duration::from_millis(SAVED_FLASH_MS))
                .unwrap_or(false);
        let relative_time = self.get_relative_time();
        let has_history_back = !self.history_back.is_empty();
        let has_history_forward = !self.history_forward.is_empty();
        let trash_count = self.deleted_notes.len();
        let is_trash_view = self.view_mode == NotesViewMode::Trash;
        let reading_time = self.get_reading_time(cx);
        let sort_label = match self.sort_mode {
            NotesSortMode::Updated => "updated ↓",
            NotesSortMode::Created => "created ↓",
            NotesSortMode::Alphabetical => "A→Z",
        };
        let auto_sizing_off = !self.auto_sizing_enabled;
        let action_feedback = self
            .get_action_feedback()
            .map(|(msg, accent)| (msg.to_string(), accent));
        let created_date = self
            .selected_note_id
            .and_then(|id| self.get_visible_notes().iter().find(|n| n.id == id))
            .map(|note| note.created_at.format("%b %d, %Y").to_string());

        div()
            .flex()
            .items_center()
            .gap_2()
            .h(px(FOOTER_HEIGHT))
            .px_3()
            .border_t_1()
            .border_color(cx.theme().border.opacity(OPACITY_SECTION_BORDER))
            .when(in_focus_mode && !window_hovered, |d| d.opacity(0.))
            .when(in_focus_mode && window_hovered, |d| {
                d.opacity(OPACITY_DISABLED)
            })
            .when(!in_focus_mode && !window_hovered, |d| {
                d.opacity(OPACITY_SUBTLE)
            })
            .when(!in_focus_mode && window_hovered, |d| d.opacity(1.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .id("footer-history-back")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(if has_history_back {
                                cx.theme().muted_foreground
                            } else {
                                cx.theme().muted_foreground.opacity(OPACITY_DISABLED)
                            })
                            .when(has_history_back, |d| {
                                d.cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_back(window, cx);
                            }))
                            .child("‹"),
                    )
                    .child(
                        div()
                            .id("footer-history-forward")
                            .min_w(px(MIN_TARGET_SIZE))
                            .min_h(px(MIN_TARGET_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(if has_history_forward {
                                cx.theme().muted_foreground
                            } else {
                                cx.theme().muted_foreground.opacity(OPACITY_DISABLED)
                            })
                            .when(has_history_forward, |d| {
                                d.cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_forward(window, cx);
                            }))
                            .child("›"),
                    )
                    .when(has_unsaved, |d| {
                        d.child(div().text_xs().text_color(cx.theme().accent).child("●"))
                    })
                    .when(show_saved, |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().accent.opacity(OPACITY_MUTED))
                                .child("✓"),
                        )
                    })
                    .when_some(action_feedback.clone(), |d, (msg, accent)| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(if accent {
                                    cx.theme().accent
                                } else {
                                    cx.theme().muted_foreground.opacity(OPACITY_MUTED)
                                })
                                .child(msg),
                        )
                    })
                    .when_some(note_position, |d, (pos, total)| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{}/{}", pos, total)),
                        )
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .when_some(cursor_line_info, |d, (line, total)| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(format!("Ln {}/{}", line, total)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(FOOTER_SEP),
                        )
                    })
                    .child(if let Some((sel_words, sel_chars)) = selection_stats {
                        div().text_xs().text_color(cx.theme().accent).child(format!(
                            "{}/{} words{}{}/{} chars",
                            sel_words, word_count, FOOTER_SEP, sel_chars, char_count,
                        ))
                    } else {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} words{}{} chars",
                                word_count, FOOTER_SEP, char_count,
                            ))
                    })
                    .when(!reading_time.is_empty(), |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(FOOTER_SEP),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                .child(reading_time.clone()),
                        )
                    }),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_1()
                    .when(auto_sizing_off && !is_trash_view, |d| {
                        d.child(
                            div()
                                .id("footer-auto-size-off")
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().accent))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.enable_auto_sizing(window, cx);
                                }))
                                .child("⤢ auto-size"),
                        )
                    })
                    .when(!is_trash_view, |d| {
                        d.child(
                            div()
                                .id("footer-sort-indicator")
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().muted_foreground))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cycle_sort_mode(cx);
                                }))
                                .child(sort_label),
                        )
                    })
                    .when(!is_trash_view && trash_count > 0, |d| {
                        d.child(
                            div()
                                .id("footer-trash-badge")
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().muted_foreground))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.set_view_mode(NotesViewMode::Trash, window, cx);
                                }))
                                .child(format!("trash ({})", trash_count)),
                        )
                    })
                    .when(is_trash_view && trash_count > 0, |d| {
                        d.child(
                            div()
                                .id("footer-empty-trash")
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().foreground))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.empty_trash(cx);
                                }))
                                .child("empty trash"),
                        )
                    })
                    .when(is_trash_view, |d| {
                        d.child(
                            div()
                                .id("footer-back-to-notes")
                                .text_xs()
                                .text_color(cx.theme().accent)
                                .cursor_pointer()
                                .hover(|s| s.text_color(cx.theme().foreground))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                                }))
                                .child("back to notes"),
                        )
                    })
                    .when(window_hovered, |d| {
                        d.when_some(created_date.clone(), |d, date| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                    .child(date),
                            )
                        })
                    })
                    .when_some(relative_time, |d, time| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                                .child(time),
                        )
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                            .child(if is_preview { "MD" } else { "TXT" }),
                    ),
            )
            .into_any_element()
    }
}
