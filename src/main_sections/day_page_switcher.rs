// Day Page past-day switcher (Cmd+P): find and swap to past days.
//
// The switcher lists `brain/days/YYYY-MM-DD.md` files newest-first with a
// first-line preview, filters on typed query text, and rebinds the Day Page
// document session to the chosen date. While open, focus moves from the
// editor to the Day Page root focus handle so typed characters filter the
// switcher instead of editing the document.

// The former inline `DaySwitcherState`/`DaySwitcherEntry` machinery and its
// free helpers were dead (the field was never populated). The live past-day
// switcher is the `note_switcher` CommandBar popup below.

impl DayPageView {
    fn local_today(&self) -> chrono::NaiveDate {
        self.now()
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
        if let Err(error) = crate::notes::ensure_notes_db_open() {
            tracing::warn!(
                target: "script_kit::day_page",
                error = %error,
                "day_page_note_switcher_notes_init_failed"
            );
        }
        let destination = crate::notes::search_model::NoteSearchDestination::OpenHere;
        let search_state = crate::notes::search_model::load_note_search_state(
            "",
            &crate::notes::notes_brain_days_dir(),
            1,
            None,
        );
        let current_id = self
            .session
            .viewing_note_id()
            .and_then(crate::notes::NoteId::parse)
            .map(crate::notes::search_model::NoteSearchDocumentId::Note)
            .or_else(|| {
                self.session
                    .bound_date()
                    .map(crate::notes::search_model::NoteSearchDocumentId::Day)
            });
        let actions = crate::actions::get_canonical_note_search_actions(&search_state, current_id);
        self.note_switcher.set_actions(actions, cx);
        self.note_switcher.open(window, cx);
        if let Some(dialog) = self.note_switcher.dialog() {
            dialog.update(cx, |dialog, cx| {
                dialog.set_context_title(Some(destination.primary_verb().to_string()));
                cx.notify();
            });
        }
    }

    fn execute_note_switcher_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_switcher.close(cx);
        let target = if let Some(date) = action_id
            .strip_prefix("daypage_")
            .and_then(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        {
            crate::notes::NoteTarget::Day(date)
        } else if let Some(note_id_text) = action_id.strip_prefix("note_") {
            if let Some(date) = crate::notes::day_switcher::parse_day_note_action_id(note_id_text) {
                crate::notes::NoteTarget::Day(date)
            } else if let Some(note_id) = crate::notes::NoteId::parse(note_id_text) {
                crate::notes::NoteTarget::Note(note_id)
            } else {
                return;
            }
        } else {
            return;
        };
        if let Err(error) = self.follow_to_note_target(target, window, cx) {
            tracing::warn!(
                target: "script_kit::day_page",
                action_id,
                error = %error,
                "day_page_note_switcher_open_failed"
            );
            crate::line_notice::say(cx, crate::line_notice::FAILED, error.to_string());
        }
    }

    pub(crate) fn close_day_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        if let Err(error) = self.session.bind_date(date, self.now()) {
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

    /// Handle a key while the switcher is open. Returns true when consumed.
    pub(crate) fn handle_day_switcher_key(
        &mut self,
        key: &str,
        modifiers: gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.note_switcher.is_open() {
            return false;
        }
        let cmd = modifiers.platform;
        let shift = modifiers.shift;
        let alt = modifiers.alt;
        let control = modifiers.control;
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
            self.note_switcher.handle_backspace(window, cx);
            return true;
        }
        if alt && !cmd && !shift && !control && key == "backspace" {
            self.note_switcher.handle_backspace_word(window, cx);
            return true;
        }
        if cmd && !shift && !alt && !control && key == "v" {
            self.note_switcher.handle_paste(window, cx);
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
                self.note_switcher.handle_char(ch, window, cx);
                return true;
            }
        }
        // Swallow everything else so stray keys cannot edit the document
        // while the switcher overlays it.
        true
    }
}
