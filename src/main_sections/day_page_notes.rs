// Every note opens in the Day Page editor: today's page, past days, named notes and new ones.
//
// The editor keeps a trail of the documents opened from inside it (a `[[link]]`, ⌘P, a backlink),
// so Esc walks back through them before leaving. `[[` completes a note title as ghost text, and
// the same ghost seam is where a line prediction will plug in.

/// Where Esc goes once the editor's trail is empty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DayPageBack {
    /// Back out of the editor: to the launcher when it was opened from there, else hide.
    #[default]
    Leave,
    /// To the Notes list the note was picked from.
    NotesList,
}

/// How long typing has to pause before a ghost suggestion shows.
const DAY_PAGE_GHOST_PAUSE: std::time::Duration = std::time::Duration::from_millis(150);

/// What the space above the editor says about the open document, so it's clear what typing
/// goes into before and after the first keystroke.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DayPageHeading {
    /// Shown large, when the document's own text doesn't already start by naming it.
    pub(crate) title: Option<String>,
    /// One short line naming what's open. It sits outside the text, so it stays while the text
    /// scrolls.
    pub(crate) context: String,
}

/// The heading for the open document. Days get their date, which their file doesn't hold. A
/// note's first `# ` line is its title, so only a note without one gets its title shown here.
/// `first_line` is the document's first non-blank line.
pub(crate) fn day_page_heading(
    binding: &script_kit_gpui::day_page::DayPageBinding,
    bound_date: Option<chrono::NaiveDate>,
    today: chrono::NaiveDate,
    pinned: bool,
    new_note: bool,
    first_line: Option<&str>,
) -> Option<DayPageHeading> {
    use script_kit_gpui::day_page::DayPageBinding;
    match binding {
        DayPageBinding::Day => {
            let date = bound_date?;
            Some(DayPageHeading {
                title: Some(day_heading_title(date, today)),
                context: day_heading_relative(date, today),
            })
        }
        DayPageBinding::Note { title, .. } => {
            let names_itself = first_line
                .and_then(|line| line.trim_start().strip_prefix("# "))
                .is_some_and(|rest| !rest.trim().is_empty());
            let kind = if pinned {
                "Pinned note"
            } else if new_note {
                "New note"
            } else {
                "Note"
            };
            let title = if title.trim().is_empty() {
                "Untitled Note"
            } else {
                title.trim()
            };
            Some(if names_itself {
                DayPageHeading {
                    title: None,
                    context: if new_note {
                        kind.to_string()
                    } else {
                        format!("{kind} · {title}")
                    },
                }
            } else {
                DayPageHeading {
                    title: Some(title.to_string()),
                    context: kind.to_string(),
                }
            })
        }
        DayPageBinding::Fragment { .. } => None,
    }
}

/// "Thursday, October 1", with the year when it isn't this year's.
fn day_heading_title(date: chrono::NaiveDate, today: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    if date.year() == today.year() {
        date.format("%A, %B %-d").to_string()
    } else {
        date.format("%A, %B %-d, %Y").to_string()
    }
}

/// "Today", "Yesterday", "4 days ago", "Tomorrow", "In 3 days".
fn day_heading_relative(date: chrono::NaiveDate, today: chrono::NaiveDate) -> String {
    match (date - today).num_days() {
        0 => "Today".to_string(),
        -1 => "Yesterday".to_string(),
        1 => "Tomorrow".to_string(),
        days if days < 0 => format!("{} days ago", -days),
        days => format!("In {days} days"),
    }
}

/// The title a new note starts with: when it was made, like "Oct 1, 9:42 PM".
pub(crate) fn new_note_time_title(at: chrono::DateTime<chrono_tz::Tz>) -> String {
    at.format("%b %-d, %-I:%M %p").to_string()
}

/// ↵ after a title that is the whole document leaves a blank line under it, the way Markdown
/// sets a heading apart, and puts the cursor below that. With a new note's time-stamp title still
/// selected, ↵ keeps the title rather than typing over it. Returns the new text and cursor.
pub(crate) fn heading_enter_fixup(
    previous: &str,
    content: &str,
    cursor: usize,
    new_note_title: Option<&str>,
) -> Option<(String, usize)> {
    let kept_title = new_note_title
        .filter(|title| previous == format!("# {title}") && content == "# \n")
        .map(|_| previous);
    let title = kept_title.or_else(|| {
        let title = content.strip_suffix('\n')?;
        let is_lone_title = title == previous
            && !title.contains('\n')
            && title
                .strip_prefix("# ")
                .is_some_and(|rest| !rest.trim().is_empty());
        (is_lone_title && cursor == content.len()).then_some(title)
    })?;
    let fixed = format!("{title}\n\n");
    let end = fixed.len();
    Some((fixed, end))
}

/// ↵ at the end of a timed line (`- 14:32 called Sam`) starts the next one with the current
/// time; ↵ on a timed line with nothing after its time ends the run and leaves a plain line.
/// Returns the new text and cursor.
pub(crate) fn timed_line_enter_fixup(
    previous: &str,
    content: &str,
    cursor: usize,
    now: &str,
) -> Option<(String, usize)> {
    let newline = cursor.checked_sub(1)?;
    if content.as_bytes().get(newline) != Some(&b'\n')
        || content.len() != previous.len() + 1
        || !content.is_char_boundary(cursor)
        || format!("{}{}", &content[..newline], &content[cursor..]) != previous
        || !content[cursor..]
            .split('\n')
            .next()
            .unwrap_or("")
            .is_empty()
    {
        return None;
    }
    let line_start = content[..newline].rfind('\n').map_or(0, |at| at + 1);
    let line = &content[line_start..newline];
    if line.contains("[ ]") || line.contains("[x]") {
        return None;
    }
    let time_only =
        script_kit_gpui::brain::substrate::split_day_line_time(&format!("{} .", line.trim()))
            .is_some_and(|(_, rest)| rest == ".");
    if time_only {
        let fixed = format!("{}{}", &content[..line_start], &content[cursor..]);
        return Some((fixed, line_start));
    }
    script_kit_gpui::brain::substrate::split_day_line_time(line)?;
    let stamp = format!("- {now} ");
    let fixed = format!("{}{stamp}{}", &content[..cursor], &content[cursor..]);
    Some((fixed, cursor + stamp.len()))
}

/// A note file's name without `.md`, which is what `[[links]]` name.
fn note_file_name(path: &std::path::Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
}
const DAY_PAGE_NOTE_ACTION_PREFIX: &str = "day_page:note:";
const DAY_PAGE_NOTES_SECTION_TITLE: &str = "Note";
const DAY_PAGE_LINKED_FROM_SECTION_TITLE: &str = "Linked from";

impl DayPageView {
    /// The open document as something the editor can open again.
    pub(crate) fn current_note_target(&self) -> Option<crate::notes::NoteTarget> {
        if self.session.is_viewing_fragment() {
            return None;
        }
        if let Some(id) = self
            .session
            .viewing_note_id()
            .and_then(crate::notes::NoteId::parse)
        {
            return Some(crate::notes::NoteTarget::Note(id));
        }
        self.session.bound_date().map(crate::notes::NoteTarget::Day)
    }

    /// The named note that's open, if one is.
    pub(crate) fn current_note_id(&self) -> Option<crate::notes::NoteId> {
        match self.current_note_target()? {
            crate::notes::NoteTarget::Note(id) => Some(id),
            _ => None,
        }
    }

    pub(crate) fn bind_note(
        &mut self,
        note_id: crate::notes::NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let note = crate::notes::get_note(note_id)?
            .filter(|note| !note.is_deleted())
            .ok_or_else(|| anyhow::anyhow!("That note isn't there anymore"))?;
        let path = crate::notes::note_file_path(note.id).ok().flatten();
        let name = path.as_deref().and_then(note_file_name);
        self.session.bind_note_content(
            note.id.as_str(),
            note.title.clone(),
            note.content.clone(),
            path,
            self.now(),
        )?;
        self.apply_loaded_content_to_editor(window, cx);
        self.open_note_pinned = note.is_pinned;
        self.opened_note_name = name;
        cx.notify();
        Ok(())
    }

    /// Makes a new note titled with the time and opens it with that title selected.
    fn bind_new_note(&mut self, window: &mut Window, cx: &mut Context<Self>) -> anyhow::Result<()> {
        let title = new_note_time_title(
            self.now()
                .with_timezone(&self.session.substrate().timezone()),
        );
        let id = crate::notes::new_note_to_edit(Some(&format!("# {title}")))?;
        self.bind_note(id, window, cx)?;
        self.new_note_title = Some(title);
        Ok(())
    }

    /// What the space above the editor says about the open document.
    pub(crate) fn heading(&self, cx: &App) -> Option<DayPageHeading> {
        let input = self.notes_editor.read(cx).input_state();
        let value = input.read(cx).value();
        let first_line = value.lines().find(|line| !line.trim().is_empty());
        day_page_heading(
            self.session.binding(),
            self.session.bound_date(),
            self.local_today(),
            self.open_note_pinned,
            self.new_note_title
                .as_ref()
                .is_some_and(|title| first_line == Some(format!("# {title}").as_str())),
            first_line,
        )
    }

    /// Saves what's open, then shows `target`. `back` says where Esc goes once the trail is empty.
    pub(crate) fn open_note_target(
        &mut self,
        target: crate::notes::NoteTarget,
        back: DayPageBack,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        self.leave_current_document(window, cx)?;
        self.trail.clear();
        self.back = back;
        self.show_note_target(target, window, cx)
    }

    /// Opens `target` from inside the editor; Esc comes back to what's open now.
    pub(crate) fn follow_to_note_target(
        &mut self,
        target: crate::notes::NoteTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let here = self
            .current_note_target()
            .map(|here| (here, self.document_label()));
        let going_to = match &target {
            crate::notes::NoteTarget::Today => crate::notes::NoteTarget::Day(self.today()),
            other => other.clone(),
        };
        let kept = self.leave_current_document(window, cx)?;
        if let Some(here) = here.filter(|(here, _)| kept && *here != going_to) {
            self.trail.push(here);
        }
        self.show_note_target(target, window, cx)
    }

    /// Where Esc goes from here, in words, when that's somewhere other than out of the editor.
    pub(crate) fn back_label(&self) -> Option<String> {
        if let Some((_, label)) = self.trail.last() {
            return Some(format!("Back to {label}"));
        }
        (self.back == DayPageBack::NotesList).then(|| "Back to Notes".to_string())
    }

    fn show_note_target(
        &mut self,
        target: crate::notes::NoteTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        use crate::notes::NoteTarget;
        match target {
            NoteTarget::Today => self.bind_today(window, cx),
            NoteTarget::Day(date) => self.bind_day(date, window, cx),
            NoteTarget::Note(id) => self.bind_note(id, window, cx)?,
            NoteTarget::New => self.bind_new_note(window, cx)?,
            NoteTarget::Append(line) => {
                let substrate = self.session.substrate();
                let (date, _) = substrate.capture_day_destination(self.now());
                substrate.append_to_captured_day(
                    date,
                    self.now(),
                    script_kit_gpui::brain::substrate::DayEntry::Capture { text: line },
                )?;
                self.bind_today(window, cx);
            }
            NoteTarget::Search => anyhow::bail!("The notes list opens under the input"),
        }
        self.focus_editor(window, cx);
        cx.notify();
        Ok(())
    }

    /// Saves the open document; a named note left empty is removed instead. Returns whether the
    /// document is still there to come back to.
    fn leave_current_document(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<bool> {
        self.clear_ghost(cx);
        if let Some(id) = self.current_note_id() {
            let content = self.notes_editor.read(cx).content(cx);
            let only_its_time_title = self
                .new_note_title
                .as_ref()
                .is_some_and(|title| content.trim() == format!("# {title}"));
            if only_its_time_title || content.trim().trim_start_matches('#').trim().is_empty() {
                script_kit_gpui::day_page::delete_note_permanently(&id.as_str())?;
                return Ok(false);
            }
        }
        if self.session.is_dirty() && !self.save_and_sync_footer(window, cx) {
            anyhow::bail!("Couldn't save {}", self.document_label());
        }
        if let Some(id) = self.current_note_id() {
            if self.new_note_title.take().is_some() {
                self.add_new_note_to_today(id)?;
            } else if let Some(old) = self.opened_note_name.take() {
                let new = crate::notes::note_file_path(id)?
                    .as_deref()
                    .and_then(note_file_name);
                if let Some(new) = new.filter(|new| *new != old) {
                    let days_dir = self.session.substrate().paths().days_dir();
                    crate::notes::rewrite_note_links(id, &old, &new, &days_dir)?;
                }
            }
        }
        Ok(true)
    }

    /// A note made today is listed on the day it was started, at the time it was started:
    /// `- 14:32 [[Trip ideas]]`.
    fn add_new_note_to_today(&self, id: crate::notes::NoteId) -> anyhow::Result<()> {
        let (Some(note), Some(name)) = (
            crate::notes::get_note(id)?,
            crate::notes::note_file_path(id)?
                .as_deref()
                .and_then(note_file_name),
        ) else {
            return Ok(());
        };
        let substrate = self.session.substrate();
        let (date, _) = substrate.capture_day_destination(note.created_at);
        substrate.append_to_captured_day(
            date,
            note.created_at,
            script_kit_gpui::brain::substrate::DayEntry::NoteLink { name },
        )?;
        Ok(())
    }

    /// Esc: dismiss a ghost suggestion, then walk back the trail; a past day opened on its own
    /// goes back to Today; otherwise leave the editor.
    pub(crate) fn escape_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.notes_editor.read(cx).has_inline_completion(cx) {
            self.clear_ghost(cx);
            return;
        }
        if self.session.is_viewing_fragment() {
            self.return_to_day_page(window, cx);
            return;
        }
        if let Some((previous, _)) = self.trail.pop() {
            if let Err(error) = self
                .leave_current_document(window, cx)
                .and_then(|_| self.show_note_target(previous, window, cx))
            {
                crate::line_notice::say(
                    cx,
                    crate::line_notice::FAILED,
                    format!("Couldn't go back to the last note: {error}"),
                );
            }
            return;
        }
        let today = self
            .now()
            .with_timezone(&self.session.substrate().timezone())
            .date_naive();
        if self.back == DayPageBack::Leave
            && self.session.bound_date().is_some_and(|date| date != today)
        {
            if let Err(error) = self
                .leave_current_document(window, cx)
                .and_then(|_| self.show_note_target(crate::notes::NoteTarget::Today, window, cx))
            {
                crate::line_notice::say(
                    cx,
                    crate::line_notice::FAILED,
                    format!("Couldn't open today's page: {error}"),
                );
            }
            return;
        }
        if let Err(error) = self.leave_current_document(window, cx) {
            crate::line_notice::say(
                cx,
                crate::line_notice::FAILED,
                format!("Couldn't save the note: {error}"),
            );
            return;
        }
        let back = self.back;
        let Some(app) = self.app.upgrade() else {
            return;
        };
        window.defer(cx, move |window, cx| {
            app.update(cx, |app, cx| match back {
                DayPageBack::NotesList => {
                    app.reset_to_script_list(cx);
                    app.one_window_open_notes_list(String::new(), cx);
                }
                DayPageBack::Leave => app.go_back_or_close(window, cx),
            });
        });
    }

    /// ⌘. or a click inside `[[Note title]]` opens that note; a title no note has yet makes it.
    pub(crate) fn activate_wiki_link_under_cursor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let (text, cursor) = self.notes_editor.read(cx).read_input(cx, |state| {
            (state.value().to_string(), state.selected_range().start)
        });
        let Some(title) = crate::notes::metadata::wiki_link_target_at(&text, cursor) else {
            return false;
        };
        let target = match crate::notes::resolve_note_ref(&title) {
            Ok(crate::notes::NoteRefResolution::Unique(id)) => {
                Ok(crate::notes::NoteTarget::Note(id))
            }
            Ok(crate::notes::NoteRefResolution::Ambiguous) => Err(format!(
                "More than one note is called “{title}”. Rename one to link it."
            )),
            Ok(crate::notes::NoteRefResolution::NotFound) => {
                let note = crate::notes::Note::with_content(format!("# {title}\n\n"));
                crate::notes::save_note(&note)
                    .map(|()| crate::notes::NoteTarget::Note(note.id))
                    .map_err(|error| error.to_string())
            }
            Err(error) => Err(error.to_string()),
        };
        let result = target.and_then(|target| {
            self.follow_to_note_target(target, window, cx)
                .map_err(|error| error.to_string())
        });
        if let Err(error) = result {
            crate::line_notice::say(cx, crate::line_notice::FAILED, error);
        }
        true
    }

    // ── Ghost text ──────────────────────────────────────────────────────────────────────────────

    /// Typing paused: offer a suggestion for the edit `generation` names, if it's still current.
    pub(crate) fn schedule_ghost_suggestion(&mut self, cx: &mut Context<Self>) {
        self.ghost_generation = self.ghost_generation.wrapping_add(1);
        let generation = self.ghost_generation;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(DAY_PAGE_GHOST_PAUSE).await;
            let _ = this.update(cx, |view, cx| {
                if view.ghost_generation == generation {
                    view.show_ghost_suggestion(cx);
                }
            });
        })
        .detach();
    }

    fn show_ghost_suggestion(&mut self, cx: &mut Context<Self>) {
        if self.read_mode || self.kit_resource_preview.is_some() {
            return;
        }
        let (text, selection) = self.notes_editor.read(cx).read_input(cx, |state| {
            (state.value().to_string(), state.selected_range())
        });
        let suffix = day_page_ghost_suffix(&text, selection, &|typed| {
            crate::notes::search_root_notes_meta_direct(
                typed,
                crate::notes::RootNotesSectionOptions {
                    enabled: true,
                    max_results: 8,
                    min_query_chars: 1,
                    search_content: false,
                },
            )
            .into_iter()
            .map(|hit| hit.title)
            .collect()
        });
        self.notes_editor
            .update(cx, |editor, cx| editor.sync_inline_completion(suffix, cx));
    }

    pub(crate) fn clear_ghost(&mut self, cx: &mut Context<Self>) {
        self.ghost_generation = self.ghost_generation.wrapping_add(1);
        self.notes_editor
            .update(cx, |editor, cx| editor.sync_inline_completion(None, cx));
    }

    /// ⌘→ takes the suggestion's next word and keeps the rest showing.
    pub(crate) fn accept_ghost_word(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let suffix = self.notes_editor.read(cx).read_input(cx, |state| {
            state.inline_completion_text().map(|text| text.to_string())
        });
        let Some(suffix) = suffix else {
            return false;
        };
        let word = crate::notes::ghost::first_word_acceptance_suffix(&suffix).to_string();
        let rest = suffix[word.len()..].to_string();
        self.notes_editor.update(cx, |editor, cx| {
            editor.input_state().update(cx, |state, cx| {
                state.replace(&word, window, cx);
            });
            editor.sync_inline_completion((!rest.is_empty()).then_some(rest), cx);
        });
        // The edit above schedules a fresh suggestion; the rest already shows.
        self.ghost_generation = self.ghost_generation.wrapping_add(1);
        true
    }
}

/// What the editor offers as ghost text at the cursor, VS Code style: only with nothing selected
/// and the cursor at the end of its line. `titles` lists note titles matching what's typed.
///
/// A `[[` link completes to a note title. Predicting the rest of a line is not offered yet.
pub(crate) fn day_page_ghost_suffix(
    text: &str,
    selection: std::ops::Range<usize>,
    titles: &dyn Fn(&str) -> Vec<String>,
) -> Option<String> {
    let cursor = selection.end;
    if selection.start != cursor || !text.is_char_boundary(cursor) {
        return None;
    }
    // Like an editor's inline suggestion: only at the end of the line, where the brackets the
    // editor closed for you still count as the end.
    let line_rest = text[cursor..].split('\n').next().unwrap_or_default();
    let closing = match line_rest {
        "" => "]]",
        "]]" => "",
        _ => return None,
    };
    let typed = crate::notes::metadata::open_wiki_link_prefix(text, cursor)?;
    if typed.trim().is_empty() {
        return None;
    }
    let typed_lower = typed.to_lowercase();
    titles(typed.trim()).into_iter().find_map(|title| {
        let rest = title
            .char_indices()
            .nth(typed.chars().count())
            .map_or(title.len(), |(at, _)| at);
        let starts = title[..rest].to_lowercase() == typed_lower;
        (starts && rest < title.len()).then(|| format!("{}{closing}", &title[rest..]))
    })
}

/// The note rows ⌘K lists for the open document.
pub(crate) fn day_page_note_actions(view: &DayPageView) -> Vec<crate::actions::Action> {
    use crate::actions::{Action, ActionCategory};
    let row = |id: &str, title: String, detail: &str| {
        Action::new(
            format!("{DAY_PAGE_NOTE_ACTION_PREFIX}{id}"),
            title,
            Some(detail.to_string()),
            ActionCategory::ScriptContext,
        )
        .with_section(DAY_PAGE_NOTES_SECTION_TITLE)
    };
    let mut actions = vec![
        row("new", "New Note".into(), "Start a new note").with_shortcut("cmd+n"),
        row("list", "Notes".into(), "Find a note by name"),
    ];
    if view.session.bound_date() != Some(view.today()) || view.session.is_viewing_note() {
        actions.push(row("today", "Today".into(), "Open today's page").with_shortcut("cmd+t"));
    }
    let Some(note_id) = view.current_note_id() else {
        return actions;
    };
    let note = crate::notes::get_note(note_id).ok().flatten();
    let pinned = note.as_ref().is_some_and(|note| note.is_pinned);
    actions.push(row(
        "pin",
        if pinned { "Unpin Note" } else { "Pin Note" }.into(),
        if pinned {
            "Stop keeping this note first in Notes and search"
        } else {
            "Keep this note first in Notes and search"
        },
    ));
    actions.push(row(
        "copy_link",
        "Copy Link".into(),
        "Copy [[this note]] to link to it from another note",
    ));
    actions.push(row(
        "copy_deeplink",
        "Copy Deep Link".into(),
        "Copy a scriptkit:// link that opens this note from anywhere",
    ));
    actions.push(row(
        "duplicate",
        "Duplicate Note".into(),
        "Make a copy and open it",
    ));
    actions.push(row(
        "reveal",
        "Show in Finder".into(),
        "Show the note's Markdown file",
    ));
    actions.push(row(
        "delete",
        "Move to Trash".into(),
        "Move this note to the trash",
    ));
    for backlink in crate::notes::get_note_backlinks(note_id).unwrap_or_default() {
        actions.push(
            Action::new(
                format!("{DAY_PAGE_NOTE_ACTION_PREFIX}open:{}", backlink.id.as_str()),
                backlink.title,
                Some("Links to this note".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section(DAY_PAGE_LINKED_FROM_SECTION_TITLE),
        );
    }
    actions
}

impl DayPageView {
    fn today(&self) -> chrono::NaiveDate {
        self.now()
            .with_timezone(&self.session.substrate().timezone())
            .date_naive()
    }
}

impl ScriptListApp {
    /// Opens `target` in the note editor, which fills the window.
    pub(crate) fn open_note_in_editor(
        &mut self,
        target: crate::notes::NoteTarget,
        back: DayPageBack,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        if matches!(target, crate::notes::NoteTarget::Search) {
            self.reset_to_script_list(cx);
            self.one_window_open_notes_list(String::new(), cx);
            return Ok(());
        }
        if !matches!(self.current_view, AppView::DayPage { .. }) {
            self.show_day_page_view(window, cx);
        }
        let AppView::DayPage { entity } = &self.current_view else {
            anyhow::bail!("The note editor didn't open");
        };
        let entity = entity.clone();
        entity.update(cx, |view, cx| {
            view.open_note_target(target, back, window, cx)
        })?;
        cx.notify();
        Ok(())
    }

    /// Opens `target` in the note editor once the current update is done.
    pub(crate) fn open_note_in_editor_soon(
        &mut self,
        target: crate::notes::NoteTarget,
        back: DayPageBack,
        cx: &mut Context<Self>,
    ) {
        self.open_note_in_editor_deferred(target, back, false, cx);
    }

    /// Opens `target` onto a window that was just shown for it, so Esc closes the window again.
    pub(crate) fn open_note_onto_shown_window(
        &mut self,
        target: crate::notes::NoteTarget,
        cx: &mut Context<Self>,
    ) {
        self.open_note_in_editor_deferred(target, DayPageBack::Leave, true, cx);
    }

    fn open_note_in_editor_deferred(
        &mut self,
        target: crate::notes::NoteTarget,
        back: DayPageBack,
        direct: bool,
        cx: &mut Context<Self>,
    ) {
        self.one_window_host.note_open_revision=self.one_window_host.note_open_revision.wrapping_add(1);
        let revision=self.one_window_host.note_open_revision;
        let app = cx.entity();
        cx.defer(move |cx| {
            // The app has one window; the stored handle names it once the window is registered.
            let handle =
                crate::get_main_window_handle().or_else(|| match cx.windows().as_slice() {
                    [only] => Some(*only),
                    _ => None,
                });
            let Some(handle) = handle else {
                return;
            };
            let _ = handle.update(cx, |_, window, cx| {
                app.update(cx, |app, cx| {
                    if revision!=app.one_window_host.note_open_revision {return;}
                    if let Err(error) = app.open_note_in_editor(target, back, window, cx) {
                        app.one_window_notice(
                            crate::line_notice::FAILED,
                            format!("Couldn't open the note: {error}"),
                            cx,
                        );
                    } else if direct {
                        app.mark_opened_directly("note_opened_onto_hidden_window");
                    }
                });
            });
        });
    }

    /// Runs a `day_page:note:*` row. `None` when `action_id` isn't one.
    pub(crate) fn execute_day_page_note_action(
        &mut self,
        entity: Entity<DayPageView>,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<bool> {
        use crate::notes::NoteTarget;
        let action = action_id.strip_prefix(DAY_PAGE_NOTE_ACTION_PREFIX)?;
        let follow = |target: NoteTarget, window: &mut Window, cx: &mut Context<Self>| {
            let result = entity.update(cx, |view, cx| {
                view.follow_to_note_target(target, window, cx)
            });
            if let Err(error) = result {
                crate::line_notice::say(
                    cx,
                    crate::line_notice::FAILED,
                    format!("Couldn't open the note: {error}"),
                );
            }
        };
        if let Some(id) = action.strip_prefix("open:") {
            let id = crate::notes::NoteId::parse(id)?;
            follow(NoteTarget::Note(id), window, cx);
            return Some(true);
        }
        match action {
            "new" => follow(NoteTarget::New, window, cx),
            "today" => follow(NoteTarget::Today, window, cx),
            "list" => {
                let result = entity.update(cx, |view, cx| view.leave_current_document(window, cx));
                if let Err(error) = result {
                    self.one_window_notice(
                        crate::line_notice::FAILED,
                        format!("Couldn't save the note: {error}"),
                        cx,
                    );
                    return Some(true);
                }
                self.reset_to_script_list(cx);
                self.one_window_open_notes_list(String::new(), cx);
            }
            _ => {
                let id = entity.read(cx).current_note_id()?;
                let note = crate::notes::get_note(id).ok().flatten()?;
                self.run_note_action(entity, action, note, window, cx);
            }
        }
        Some(true)
    }

    fn run_note_action(
        &mut self,
        entity: Entity<DayPageView>,
        action: &str,
        note: crate::notes::Note,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = if note.title.trim().is_empty() {
            "Untitled Note".to_string()
        } else {
            note.title.clone()
        };
        match action {
            "pin" => {
                let pinned = !note.is_pinned;
                match script_kit_gpui::day_page::set_note_pinned(&note.id.as_str(), pinned) {
                    Ok(_) => {
                        entity.update(cx, |view, cx| {
                            view.open_note_pinned = pinned;
                            cx.notify();
                        });
                        self.one_window_notice(
                            crate::line_notice::DONE,
                            if pinned {
                                format!("Pinned “{title}”")
                            } else {
                                format!("Unpinned “{title}”")
                            },
                            cx,
                        )
                    }
                    Err(error) => {
                        let verb = if pinned { "pin" } else { "unpin" };
                        self.one_window_notice(
                            crate::line_notice::FAILED,
                            format!("Couldn't {verb} “{title}”: {error}"),
                            cx,
                        )
                    }
                }
            }
            "copy_link" => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(format!("[[{title}]]")));
                self.one_window_notice(crate::line_notice::DONE, "Copied the note link", cx);
            }
            "copy_deeplink" => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(format!(
                    "scriptkit://notes/{}",
                    note.id.as_str()
                )));
                self.one_window_notice(crate::line_notice::DONE, "Copied the deep link", cx);
            }
            "duplicate" => {
                let content = entity.read(cx).notes_editor.read(cx).content(cx);
                let copy = crate::notes::Note::with_content(duplicate_note_content(&content));
                match crate::notes::save_note(&copy) {
                    Ok(()) => {
                        let result = entity.update(cx, |view, cx| {
                            view.follow_to_note_target(
                                crate::notes::NoteTarget::Note(copy.id),
                                window,
                                cx,
                            )
                        });
                        if let Err(error) = result {
                            self.one_window_notice(
                                crate::line_notice::FAILED,
                                format!("Couldn't open the copy of “{title}”: {error}"),
                                cx,
                            );
                        }
                    }
                    Err(error) => self.one_window_notice(
                        crate::line_notice::FAILED,
                        format!("Couldn't duplicate “{title}”: {error}"),
                        cx,
                    ),
                }
            }
            "reveal" => match crate::notes::note_file_path(note.id) {
                Ok(Some(path)) => {
                    if let Err(error) = crate::platform::reveal_in_finder(&path) {
                        self.one_window_notice(
                            crate::line_notice::FAILED,
                            format!("Couldn't show “{title}” in Finder: {error}"),
                            cx,
                        );
                    }
                }
                _ => self.one_window_notice(
                    crate::line_notice::FAILED,
                    "This note has no file yet",
                    cx,
                ),
            },
            "delete" => {
                let id = note.id;
                entity.update(cx, |view, cx| view.escape_back(window, cx));
                self.one_window_hold(
                    "trash",
                    format!("Move “{title}” to the trash"),
                    Box::new(move |app, cx| {
                        let trashed = script_kit_gpui::day_page::trash_note(&id.as_str());
                        if let Err(error) = trashed {
                            app.one_window_notice(
                                crate::line_notice::FAILED,
                                format!("Couldn't move the note to the Trash: {error}"),
                                cx,
                            );
                        }
                    }),
                    cx,
                );
            }
            _ => {}
        }
    }
}

/// A duplicate's first line says it's a copy, so `[[links]]` keep pointing at one note.
fn duplicate_note_content(content: &str) -> String {
    let (first, rest) = content.split_once('\n').unwrap_or((content, ""));
    let first = if first.trim().is_empty() {
        "Untitled Note copy".to_string()
    } else {
        format!("{} copy", first.trim_end())
    };
    if rest.is_empty() && !content.contains('\n') {
        first
    } else {
        format!("{first}\n{rest}")
    }
}

/// A note made from typed words and attached context: the words are its title, and each
/// attachment follows under its own heading. `context` pairs each label with its text.
pub(crate) fn typed_note_content(words: &str, context: &[(String, String)]) -> String {
    let words = words.trim();
    let title = if !words.is_empty() {
        words.lines().next().unwrap_or(words).trim().to_string()
    } else if let Some((label, _)) = context.first() {
        label.clone()
    } else {
        "Quick note".to_string()
    };
    let mut content = format!("# {title}\n");
    let rest = words.lines().skip(1).collect::<Vec<_>>().join("\n");
    if !rest.trim().is_empty() {
        content.push_str(&format!("\n{}\n", rest.trim()));
    }
    for (label, text) in context {
        content.push_str(&format!("\n## {label}\n\n{}\n", text.trim()));
    }
    content
}

/// The text inside a resolved context block, without its `<context …>` / `<attachment …>` tags.
fn context_block_text(block: &str) -> &str {
    let block = block.trim();
    let Some((first, rest)) = block.split_once('\n') else {
        return block;
    };
    if !(first.starts_with('<') && first.ends_with('>')) {
        return block;
    }
    match rest.rsplit_once('\n') {
        Some((inner, last)) if last.trim().starts_with("</") => inner,
        _ if rest.trim().starts_with("</") => "",
        _ => rest,
    }
}

impl ScriptListApp {
    /// One of Jev's note readings that takes the clipboard, the selection or the browser page
    /// along. `words` is what was typed; when nothing was, it reads "the <subject>".
    pub(crate) fn capture_note_with_screen(
        &mut self,
        words: &str,
        subject: &str,
        cx: &mut Context<Self>,
    ) {
        use crate::ai::context_contract::ContextAttachmentKind;
        let words = if words == format!("the {subject}") {
            ""
        } else {
            words
        };
        let part = |kind: ContextAttachmentKind| {
            let spec = kind.spec();
            crate::ai::message_parts::AiContextPart::ResourceUri {
                uri: spec.uri.to_string(),
                label: spec.label.to_string(),
            }
        };
        match subject {
            "clipboard" => {
                let Some(clip) = cx
                    .read_from_clipboard()
                    .and_then(|item| item.text())
                    .filter(|clip| !clip.trim().is_empty())
                else {
                    self.one_window_notice(
                        crate::line_notice::FAILED,
                        "The clipboard is empty",
                        cx,
                    );
                    return;
                };
                let content = typed_note_content(words, &[("Clipboard".to_string(), clip)]);
                match crate::notes::new_note_to_edit(Some(&content)) {
                    Ok(id) => self.open_note_in_editor_soon(
                        crate::notes::NoteTarget::Note(id),
                        DayPageBack::Leave,
                        cx,
                    ),
                    Err(error) => self.one_window_notice(
                        crate::line_notice::FAILED,
                        format!("Couldn't make a note of the clipboard: {error}"),
                        cx,
                    ),
                }
            }
            "selection" => {
                self.save_typed_note(words, &[part(ContextAttachmentKind::Selection)], cx)
            }
            "page" => self.save_typed_note(words, &[part(ContextAttachmentKind::Browser)], cx),
            _ => self.save_typed_note(words, &[], cx),
        }
    }

    /// Saves `words` and the text of each attached `parts` as a new note, then opens it.
    pub(crate) fn save_typed_note(
        &mut self,
        words: &str,
        parts: &[crate::ai::message_parts::AiContextPart],
        cx: &mut Context<Self>,
    ) {
        let mut context = Vec::new();
        for part in parts {
            match crate::ai::message_parts::resolve_context_part_to_prompt_block(
                part,
                &self.scripts,
                &self.scriptlets,
            ) {
                Ok(block) => context.push((
                    part.label().to_string(),
                    context_block_text(&block).to_string(),
                )),
                Err(error) => {
                    self.one_window_notice(
                        crate::line_notice::FAILED,
                        format!("Couldn't read {} for the note: {error}", part.label()),
                        cx,
                    );
                    return;
                }
            }
        }
        if words.trim().is_empty() && context.is_empty() {
            self.open_note_in_editor_soon(crate::notes::NoteTarget::New, DayPageBack::Leave, cx);
            return;
        }
        match crate::notes::new_note_to_edit(Some(&typed_note_content(words, &context))) {
            Ok(id) => self.open_note_in_editor_soon(
                crate::notes::NoteTarget::Note(id),
                DayPageBack::Leave,
                cx,
            ),
            Err(error) => self.one_window_notice(
                crate::line_notice::FAILED,
                format!("Couldn't save the note: {error}"),
                cx,
            ),
        }
    }
}
