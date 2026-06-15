// Day Page past-day switcher (Cmd+P): find and swap to past days.
//
// The switcher lists `brain/days/YYYY-MM-DD.md` files newest-first with a
// first-line preview, filters on typed query text, and rebinds the Day Page
// document session to the chosen date. While open, focus moves from the
// editor to the Day Page root focus handle so typed characters filter the
// switcher instead of editing the document.

#[derive(Debug, Clone)]
pub(crate) struct DaySwitcherEntry {
    pub date: chrono::NaiveDate,
    pub preview: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DaySwitcherState {
    pub query: String,
    pub selected: usize,
    pub entries: Vec<DaySwitcherEntry>,
}

pub(crate) const DAY_SWITCHER_LIST_ID: &str = "day-page-day-switcher";

pub(crate) fn load_day_switcher_entries(substrate: &BrainSubstrate) -> Vec<DaySwitcherEntry> {
    let days_dir = substrate.paths().days_dir();
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(&days_dir) else {
        return entries;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(date) = chrono::NaiveDate::parse_from_str(stem, "%Y-%m-%d") else {
            continue;
        };
        let preview = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .map(|line| line.chars().take(80).collect::<String>())
            })
            .unwrap_or_default();
        entries.push(DaySwitcherEntry { date, preview });
    }
    entries.sort_by(|a, b| b.date.cmp(&a.date));
    entries
}

pub(crate) fn day_switcher_entry_label(date: chrono::NaiveDate, today: chrono::NaiveDate) -> String {
    let formatted = date.format("%Y-%m-%d · %A").to_string();
    if date == today {
        format!("Today · {formatted}")
    } else {
        formatted
    }
}

pub(crate) fn day_switcher_semantic_id(date: chrono::NaiveDate) -> String {
    format!("day-switcher-{date}")
}

pub(crate) fn filtered_day_switcher_indices(
    state: &DaySwitcherState,
    today: chrono::NaiveDate,
) -> Vec<usize> {
    let query = state.query.trim().to_lowercase();
    state
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            if query.is_empty() {
                return true;
            }
            let label = day_switcher_entry_label(entry.date, today).to_lowercase();
            label.contains(&query) || entry.preview.to_lowercase().contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

impl DayPageView {
    fn local_today(&self) -> chrono::NaiveDate {
        Utc::now()
            .with_timezone(&self.session.substrate().timezone())
            .date_naive()
    }

    pub(crate) fn is_day_switcher_open(&self) -> bool {
        self.note_switcher.is_open()
    }

    pub(crate) fn toggle_day_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.note_switcher.is_open() {
            self.close_day_switcher(window, cx);
        } else {
            self.open_note_switcher(window, cx);
        }
    }

    pub(crate) fn open_day_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_note_switcher(window, cx);
    }

    pub(crate) fn open_note_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let notes = crate::notes::get_all_notes().unwrap_or_else(|error| {
            tracing::warn!(
                target: "script_kit::day_page",
                error = %error,
                "day_page_note_switcher_notes_load_failed"
            );
            Vec::new()
        });
        let mut rows = notes
            .iter()
            .map(|note| crate::actions::NoteSwitcherNoteInfo {
                id: note.id.as_str().to_string(),
                title: if note.title.trim().is_empty() {
                    "Untitled Note".to_string()
                } else {
                    note.title.clone()
                },
                char_count: note.char_count(),
                is_current: self
                    .session
                    .viewing_note_id()
                    .is_some_and(|id| id == note.id.as_str()),
                is_pinned: note.is_pinned,
                preview: note.preview(),
                relative_time: crate::formatting::format_relative_time_short_dt(note.updated_at),
            })
            .collect::<Vec<_>>();
        let day_entries = crate::notes::day_switcher::load_day_note_switcher_entries(
            &self.session.substrate().paths().days_dir(),
        );
        rows.extend(crate::notes::day_switcher::day_note_switcher_infos(
            &day_entries,
            self.session.bound_date(),
        ));
        let actions = crate::actions::get_note_switcher_actions(&rows);
        self.note_switcher.set_actions(actions, cx);
        self.note_switcher.open_centered(window, cx);
        self.wire_note_switcher_activation(window, cx);
        if let Some(dialog) = self.note_switcher.dialog() {
            dialog.update(cx, |dialog, cx| {
                dialog.set_context_title(None);
                cx.notify();
            });
        }
    }

    fn wire_note_switcher_activation(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(dialog) = self.note_switcher.dialog().cloned() else {
            return;
        };
        let resize_dialog = dialog.clone();
        let day_page = cx.entity().downgrade();
        let day_window = window.window_handle();
        let on_close_day_page = day_page.clone();
        dialog.update(cx, |dialog, _cx| {
            dialog.set_on_close(std::sync::Arc::new(move |cx| {
                let day_page = on_close_day_page.clone();
                cx.defer(move |cx| {
                    let _ = day_window.update(cx, |_root, window, cx| {
                        let Some(day_page) = day_page.upgrade() else {
                            return;
                        };
                        day_page.update(cx, |view, cx| {
                            view.note_switcher.mark_closed_externally();
                            view.focus_editor(window, cx);
                        });
                    });
                });
            }));
            dialog.set_on_activation(std::sync::Arc::new(move |activation, _window, cx| {
                match activation {
                    crate::actions::ActionsDialogActivation::Executed { action_id, .. } => {
                        let day_page = day_page.clone();
                        cx.defer(move |cx| {
                            let _ = day_window.update(cx, |_root, window, cx| {
                                let Some(day_page) = day_page.upgrade() else {
                                    return;
                                };
                                day_page.update(cx, |view, cx| {
                                    view.execute_note_switcher_action(&action_id, window, cx);
                                });
                            });
                        });
                    }
                    crate::actions::ActionsDialogActivation::DrillDownPushed { .. } => {
                        crate::actions::resize_actions_window(cx, &resize_dialog);
                    }
                    crate::actions::ActionsDialogActivation::NoSelection => {}
                }
            }));
        });
    }

    fn execute_note_switcher_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_switcher.close(cx);
        if let Some(date) = action_id.strip_prefix("daypage_") {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                self.bind_day(date, window, cx);
                self.focus_editor(window, cx);
            }
            return;
        }
        let Some(note_id_text) = action_id.strip_prefix("note_") else {
            return;
        };
        if let Some(date) = crate::notes::day_switcher::parse_day_note_action_id(note_id_text) {
            self.bind_day(date, window, cx);
            self.focus_editor(window, cx);
            return;
        }
        if let Some(note_id) = crate::notes::NoteId::parse(note_id_text) {
            let Ok(Some(note)) = crate::notes::get_note(note_id) else {
                return;
            };
            let path = crate::notes::note_file_path(note.id).ok().flatten();
            if let Err(error) = self.session.bind_note_content(
                note.id.as_str().to_string(),
                note.title.clone(),
                note.content.clone(),
                path,
                Utc::now(),
            ) {
                tracing::error!(
                    target: "script_kit::day_page",
                    error = %error,
                    "day_page_note_switcher_bind_note_failed"
                );
                return;
            }
            self.apply_loaded_content_to_editor(window, cx);
            self.focus_editor(window, cx);
            cx.notify();
        }
    }

    pub(crate) fn close_day_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.day_switcher = None;
        self.note_switcher.close(cx);
        self.focus_editor(window, cx);
        cx.notify();
    }

    /// Rebind the Day Page session to an arbitrary existing day.
    pub(crate) fn bind_day(
        &mut self,
        date: chrono::NaiveDate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = self.session.bind_date(date, Utc::now()) {
            tracing::error!(
                target: "script_kit::day_page",
                event = "day_page_bind_day_failed",
                date = %date,
                error = %error,
            );
            return;
        }
        self.apply_loaded_content_to_editor(window, cx);
        cx.notify();
    }

    fn accept_day_switcher_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let today = self.local_today();
        let Some(state) = self.day_switcher.as_ref() else {
            return;
        };
        let filtered = filtered_day_switcher_indices(state, today);
        let Some(entry_index) = filtered.get(state.selected.min(filtered.len().saturating_sub(1)))
        else {
            return;
        };
        let Some(entry) = state.entries.get(*entry_index) else {
            return;
        };
        let date = entry.date;
        self.close_day_switcher(window, cx);
        self.bind_day(date, window, cx);
        self.focus_editor(window, cx);
        tracing::info!(
            target: "script_kit::day_page",
            event = "day_page_switched_day",
            date = %date,
        );
    }

    /// Handle a key while the switcher is open. Returns true when consumed.
    pub(crate) fn handle_day_switcher_key(
        &mut self,
        key: &str,
        cmd: bool,
        shift: bool,
        alt: bool,
        control: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.note_switcher.is_open() {
            return false;
        }
        let exact_plain = !cmd && !shift && !alt && !control;
        if exact_plain && crate::ui_foundation::is_key_escape(key) {
            self.close_day_switcher(window, cx);
            return true;
        }
        if cmd && !shift && !alt && !control && key == "p" {
            self.close_day_switcher(window, cx);
            return true;
        }
        if exact_plain && matches!(key, "down" | "arrowdown" | "up" | "arrowup") {
            if matches!(key, "down" | "arrowdown") {
                self.note_switcher.select_next(cx);
            } else {
                self.note_switcher.select_prev(cx);
            }
            return true;
        }
        if exact_plain && key == "enter" {
            if let Some(action_id) = self.note_switcher.execute_selected_action(cx) {
                self.execute_note_switcher_action(&action_id, window, cx);
            }
            return true;
        }
        if exact_plain && key == "backspace" {
            self.note_switcher.handle_backspace(cx);
            return true;
        }
        if alt && !cmd && !shift && !control && key == "backspace" {
            self.note_switcher.handle_backspace_word(cx);
            return true;
        }
        if cmd && !shift && !alt && !control && key == "v" {
            self.note_switcher.handle_paste(cx);
            return true;
        }
        if !cmd && !control && !alt {
            let ch = if key == "space" {
                Some(' ')
            } else if key.chars().count() == 1 {
                key.chars().next()
            } else {
                None
            };
            if let Some(ch) = ch {
                self.note_switcher.handle_char(ch, cx);
                return true;
            }
        }
        // Swallow everything else so stray keys cannot edit the document
        // while the switcher overlays it.
        true
    }

    pub(crate) fn render_day_page_day_switcher_panel(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let state = self.day_switcher.clone()?;
        let app = self.app.upgrade()?;
        let app_state = app.read(cx);
        let theme = app_state.theme.clone();
        let item_colors = crate::list_item::ListItemColors::from_theme(&theme);
        let main_menu_theme = app_state.current_main_menu_theme;
        let editor_surface =
            crate::components::notes_editor::NotesEditorSurfaceStyle::from_theme(&theme);
        let text_secondary = theme.colors.text.secondary;

        let today = self.local_today();
        let filtered = filtered_day_switcher_indices(&state, today);
        let selected = if filtered.is_empty() {
            None
        } else {
            Some(state.selected.min(filtered.len() - 1))
        };

        let query_label = if state.query.is_empty() {
            "Open day… type to filter".to_string()
        } else {
            format!("Open day… {}", state.query)
        };

        let mut rows = div().flex().flex_col().w_full();
        rows = rows.child(
            div()
                .px_4()
                .py_2()
                .text_sm()
                .text_color(rgb(text_secondary))
                .child(query_label),
        );

        if filtered.is_empty() {
            rows = rows.child(
                div()
                    .px_4()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(text_secondary))
                    .child("No matching days"),
            );
        }

        for (row_ix, entry_index) in filtered.iter().enumerate() {
            let Some(entry) = state.entries.get(*entry_index) else {
                continue;
            };
            let is_selected = selected == Some(row_ix);
            let label = day_switcher_entry_label(entry.date, today);
            let click_handler = cx.listener(
                move |this: &mut DayPageView, _event: &gpui::MouseDownEvent, window, cx| {
                    if let Some(state) = this.day_switcher.as_mut() {
                        state.selected = row_ix;
                    }
                    this.accept_day_switcher_selection(window, cx);
                },
            );
            rows = rows.child(
                div()
                    .h(px(crate::list_item::effective_list_item_height_for_theme(
                        main_menu_theme,
                    )))
                    .on_mouse_down(gpui::MouseButton::Left, click_handler)
                    .child(
                        crate::list_item::ListItem::new(label, item_colors)
                            .index(row_ix)
                            .selected(is_selected)
                            .hovered(false)
                            .main_menu_theme(main_menu_theme)
                            .semantic_id(day_switcher_semantic_id(entry.date))
                            .description_opt(
                                (!entry.preview.is_empty()).then(|| entry.preview.clone()),
                            ),
                    ),
            );
        }

        Some(
            div()
                .id(DAY_SWITCHER_LIST_ID)
                .absolute()
                .inset_0()
                .bg(rgba(editor_surface.occlusion_rgba))
                .occlude()
                .overflow_y_scroll()
                .child(rows)
                .into_any_element(),
        )
    }
}
