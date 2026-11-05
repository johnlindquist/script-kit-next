//! Forms, answers and notes open in the main window, in place of the list, under the same input.
//!
//! The reducer owns which view is open, what Esc does, the Enter guard and the height; this holds
//! each open view's model, hands it the keys the reducer routes to it (`Effect::ViewKey`), and
//! reports back what ↵ says now and what finished.

use gpui::Context;
use sk_one_window::answer::{Answer, Fact};
use sk_one_window::form::{Form, FormKey, Reading, SlotName, Source, Verb};
use sk_one_window::{Done, EnterTarget, Event, Key, Row, SubjectKind, ViewOp, ViewSpec};

use crate::ScriptListApp;

mod render;

pub(crate) use render::header_element;

pub(crate) const FORM: &str = "form";
pub(crate) const ANSWER: &str = "answer";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostedView {
    Form(Form),
    Answer(Answer),
    Notes(super::notes::NotesList),
    Personal(super::personal_views::List),
    Reading(super::reading::ReadingView),
}

impl HostedView {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            HostedView::Form(_) => FORM,
            HostedView::Answer(_) => ANSWER,
            HostedView::Notes(_) => super::notes::NOTES,
            HostedView::Personal(_) => super::personal_views::NAME,
            HostedView::Reading(_) => super::reading::READING,
        }
    }

    /// The view's header words.
    pub(crate) fn label(&self) -> String {
        match self {
            HostedView::Form(form) => match &form.verb {
                Verb::Message => "Message".into(),
                Verb::Remind => "Reminder".into(),
                Verb::Schedule => "Event".into(),
                Verb::Move => "Move an event".into(),
                Verb::Note => "Note".into(),
                Verb::Other(_) => "Form".into(),
            },
            HostedView::Answer(_) => "Answer".into(),
            HostedView::Notes(list) => list.label(),
            HostedView::Personal(list) => list.label(),
            HostedView::Reading(_) => "How Jev reads it".into(),
        }
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        match self {
            HostedView::Form(form) => form.enter(),
            HostedView::Answer(answer) => answer.enter(),
            HostedView::Notes(list) => list.enter(),
            HostedView::Personal(list) => list.enter(),
            HostedView::Reading(reading) => reading.enter(),
        }
    }

    /// How many rows the view draws under its header.
    pub(crate) fn rows(&self) -> usize {
        match self {
            HostedView::Form(form) => form.fields.len(),
            HostedView::Answer(answer) => match &answer.shape {
                Some(sk_one_window::answer::Shape::List { items }) => items.len(),
                Some(sk_one_window::answer::Shape::Card { facts, .. }) => 1 + facts.len(),
                _ => 1,
            },
            HostedView::Notes(list) => list.found.len().max(1),
            HostedView::Personal(list) => list.data.rows().len().max(1),
            HostedView::Reading(reading) => reading.rows.len(),
        }
    }
}

fn append_to_day(entry: crate::brain::substrate::DayEntry) -> Result<(), &'static str> {
    crate::brain::substrate::BrainSubstrate::default_kit()
        .append_to_day(chrono::Utc::now(), entry)
        .map_err(|_| "one_window_note_write_failed")
}

impl ScriptListApp {
    /// Opens a view in place of the list. With `text`, the input shows those words while it's open;
    /// Esc brings back what was typed.
    pub(crate) fn one_window_open_hosted(
        &mut self,
        view: HostedView,
        text: Option<String>,
        cx: &mut Context<Self>,
    ) {
        // The host holds one view, so a view opened from another takes its place: Esc then
        // goes back to where the first one was opened from, not to a view nothing can draw.
        let replaces = self.one_window_hosted_view().is_some();
        let typed = self.filter_text.clone();
        self.one_window_host.view_text = Some(text.clone().unwrap_or_else(|| typed.clone()));
        let spec = ViewSpec {
            name: view.name().to_string(),
            label: view.label(),
            enter: Some(view.enter()),
            takes_input: true,
            text,
            leave_text: None,
        };
        self.one_window_host.view = Some(view);
        let op = if replaces {
            ViewOp::Replace(spec)
        } else {
            ViewOp::Open(spec)
        };
        // A symbol picker or composer owning the list keeps typing from the reducer, so it
        // hears the words first or going back would restore older ones.
        let mut turn = Vec::new();
        if self.one_window.text() != typed {
            turn.push(Event::Text(typed));
        }
        turn.push(Event::View(op));
        self.one_window_handle_turn(turn, cx);
    }

    /// The view the reducer has on top, when it's one of these.
    pub(crate) fn one_window_hosted_view(&self) -> Option<&HostedView> {
        let top = self.one_window.views().top()?;
        self.one_window_host
            .view
            .as_ref()
            .filter(|view| view.name() == top.name)
    }

    /// A key the reducer routed to the open view. Returns what the view says back, for the same
    /// turn.
    pub(crate) fn one_window_hosted_view_key(
        &mut self,
        key: Key,
        cx: &mut Context<Self>,
    ) -> Vec<Event> {
        match self.one_window_hosted_view() {
            Some(HostedView::Form(_) | HostedView::Answer(_)) => {
                self.one_window_hosted_key(key, cx)
            }
            Some(HostedView::Reading(_)) if key == Key::Enter => {
                vec![Event::View(ViewOp::Back), Event::Key(Key::Enter)]
            }
            Some(HostedView::Reading(_)) | None => Vec::new(),
            Some(HostedView::Notes(_)) => self.one_window_notes_list_key(key, cx),
            Some(HostedView::Personal(_)) => self.personal_view_key(key, cx),
        }
    }

    /// Click and semantic selection load the same field into the existing main input. Neither submits it.
    pub(crate) fn one_window_pick_form_field(&mut self, id: &str, cx: &mut Context<Self>) -> bool {
        if !matches!(self.one_window_hosted_view(), Some(HostedView::Form(_))) {
            return false;
        }
        let Some(HostedView::Form(form)) = self.one_window_host.view.as_mut() else {
            return false;
        };
        let Some(i) = form
            .fields
            .iter()
            .position(|f| format!("form-field:{:?}", f.name).to_lowercase() == id)
        else {
            return false;
        };
        form.caret = Some(i);
        let words = form.fields[i].value.clone();
        self.one_window_host.view_text = Some(words.clone());
        self.one_window_set_input(words.clone(), cx);
        self.one_window_handle(Event::Text(words), cx);
        cx.notify();
        true
    }

    fn one_window_hosted_key(&mut self, key: Key, cx: &mut Context<Self>) -> Vec<Event> {
        let Some(view) = self.one_window_host.view.as_mut() else {
            return Vec::new();
        };
        let before = view.enter();
        let mut caret_words = None;
        let finished = match view {
            HostedView::Form(form) => match form.key(&key) {
                FormKey::Submit(done) => Some(Ok(done)),
                FormKey::Changed => {
                    let value = form
                        .caret
                        .and_then(|i| form.fields.get(i))
                        .map(|f| f.value.clone())
                        .unwrap_or_default();
                    caret_words = Some(value);
                    None
                }
                FormKey::Unused => None,
            },
            HostedView::Answer(answer) if key == Key::Enter => answer.copy_text().map(|text| {
                crate::runtime_policy::route_text_copy(
                    crate::runtime_policy::owned_evaluation(),
                    &text,
                    |text| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.to_owned()));
                        Ok(())
                    },
                )
                .map(|receipt| Done {
                    icon: "copy".into(),
                    title: receipt.feedback(format!("Copied {text}")),
                    ..Done::default()
                })
                .map_err(|_| "one_window_copy_failed")
            }),
            HostedView::Answer(_)
            | HostedView::Notes(_)
            | HostedView::Reading(_)
            | HostedView::Personal(_) => None,
        };
        let mut events = Vec::new();
        if let Some(words) = caret_words {
            self.one_window_host.view_text = Some(words.clone());
            self.one_window_set_input(words.clone(), cx);
            events.push(Event::Text(words));
        }
        match finished {
            Some(Ok(done)) => {
                if let Some(HostedView::Form(form)) = &self.one_window_host.view {
                    if let Err(reason) = self.one_window_finish_form(form.clone()) {
                        tracing::warn!(target: "script_kit::one_window", reason, "form not saved");
                        self.one_window_notice(
                            crate::line_notice::FAILED,
                            "Couldn't complete this form. Nothing was sent or scheduled.",
                            cx,
                        );
                        return events;
                    }
                }
                events.push(Event::Done(done));
            }
            Some(Err(reason)) => {
                tracing::warn!(target: "script_kit::one_window", reason, "view did not finish");
            }
            None => {
                if let Some(view) = self.one_window_host.view.as_ref() {
                    let after = view.enter();
                    if after != before {
                        events.push(Event::ViewEnter(after));
                    }
                }
            }
        }
        events
    }

    /// What a finished form writes where it lives. A message waits in the line first; the
    /// reducer says when it goes.
    fn one_window_finish_form(&self, form: Form) -> Result<(), &'static str> {
        let value = |name: SlotName| {
            form.field(name)
                .map(|f| f.value.clone())
                .filter(|v| !v.is_empty())
        };
        match form.verb {
            Verb::Remind => append_to_day(crate::brain::substrate::DayEntry::Task {
                body: value(SlotName::What).unwrap_or_default(),
                tags: Vec::new(),
                due: value(SlotName::When),
            }),
            Verb::Note => append_to_day(crate::brain::substrate::DayEntry::Capture {
                text: value(SlotName::What).unwrap_or_default(),
            }),
            Verb::Message | Verb::Schedule | Verb::Move | Verb::Other(_) => {
                Err("form_executor_unavailable")
            }
        }
    }

    /// Words typed while a view is open go to it: the form's caret field, the note's new line.
    /// Returns ↵'s new words when they changed.
    pub(crate) fn one_window_view_typed(&mut self) -> Option<Event> {
        self.one_window_hosted_view()?;
        let text = self.one_window.text().to_string();
        if self.one_window_host.view_text.as_deref() == Some(text.as_str()) {
            return None;
        }
        self.one_window_host.view_text = Some(text.clone());
        let reading = self.one_window_jev_reading(&text);
        let view = self.one_window_host.view.as_mut()?;
        let before = view.enter();
        match view {
            HostedView::Form(form) => {
                let field = form.caret.and_then(|i| form.fields.get_mut(i))?;
                if field.value == text.trim() {
                    return None;
                }
                field.value = text.trim().to_string();
                field.from = (!field.value.is_empty()).then_some(Source::YourWords);
                field.state = if field.value.is_empty() {
                    sk_one_window::form::FieldState::Needed
                } else {
                    sk_one_window::form::FieldState::Filled
                };
            }
            HostedView::Notes(list) => list.typed(text),
            HostedView::Personal(list) => list.typed(text),
            HostedView::Reading(view) => {
                let words = reading.as_deref().map(|plan| plan.words.as_slice());
                *view = super::reading::ReadingView::new(
                    text,
                    words,
                    view.enter.clone(),
                    view.owner_tab,
                );
            }
            HostedView::Answer(_) => return None,
        }
        let after = view.enter();
        (after != before).then_some(Event::ViewEnter(after))
    }

    /// Keys an open view uses that the input or the launcher would otherwise take: Tab and ← → in a
    /// form (fields and their other values), and Tab in the Notes list (its order).
    /// Enter, Esc and the arrows up and down reach the reducer through the list's own paths.
    pub(crate) fn one_window_view_takes_key(
        &mut self,
        key: &str,
        modifiers: &gpui::Modifiers,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, crate::AppView::ScriptList)
            || modifiers.alt
            || modifiers.control
        {
            return false;
        }
        let key = key.to_ascii_lowercase();
        if matches!(key.as_str(), "right" | "arrowright")
            && !modifiers.platform
            && !modifiers.shift
            && self.one_window_open_reading(cx)
        {
            return true;
        }
        let key = match self.one_window_hosted_view() {
            Some(HostedView::Form(form)) if !modifiers.platform => match key.as_str() {
                "left" | "arrowleft" | "right" | "arrowright"
                    if modifiers.shift
                        || form
                            .caret
                            .and_then(|i| form.fields.get(i))
                            .is_none_or(|f| f.alts.is_empty()) =>
                {
                    return false
                }
                "tab" if modifiers.shift => Key::Up,
                "tab" => Key::Tab,
                "right" | "arrowright" => Key::Right,
                "left" | "arrowleft" => Key::Other("left".into()),
                _ => return false,
            },
            Some(HostedView::Notes(_))
                if !modifiers.platform && !modifiers.shift && key == "tab" =>
            {
                Key::Tab
            }
            _ => return false,
        };
        self.one_window_key(key, cx);
        true
    }

    /// → with the caret at the end of the sentence opens how Jev reads it, when there's a reading
    /// or a typed symbol to show and Jev's list is the one showing. Anywhere else → moves the
    /// caret.
    fn one_window_open_reading(&mut self, cx: &mut Context<Self>) -> bool {
        if self.one_window.in_view()
            || self.ghost_prediction.is_some()
            || !self.one_window_owns_list()
        {
            return false;
        }
        let text = self.one_window.text().to_string();
        let at_end = {
            let input = self.gpui_input_state.read(cx);
            input.selected_range().is_empty() && input.cursor() == input.value().len()
        };
        if text.trim().is_empty() || !at_end {
            return false;
        }
        let reading = self.one_window_jev_reading(&text);
        if reading.is_none()
            && !self.one_window.agent_owns()
            && sk_one_window::marks::typed_symbols(&text).is_empty()
        {
            return false;
        }
        let enter = self.one_window.enter_target(self.one_window_clock());
        let words = reading.as_deref().map(|plan| plan.words.as_slice());
        let view =
            super::reading::ReadingView::new(text, words, enter, self.one_window.agent_owns());
        self.one_window_open_hosted(HostedView::Reading(view), None, cx);
        true
    }

    /// After every turn: a view the reducer closed is gone here too.
    pub(crate) fn one_window_views_follow(&mut self, cx: &mut Context<Self>) {
        if self.one_window_host.view.is_some() && self.one_window_hosted_view().is_none() {
            self.one_window_host.view = None;
            self.one_window_host.view_text = None;
        }
        let covered = self.one_window.in_view();
        if covered != std::mem::replace(&mut self.one_window_host.covered, covered) {
            self.refresh_ghost_with_input(cx);
        }
    }

    // ------------------------------------------------------------------ what opens them

    /// ⌘V in the main window. Into the empty input it stages what was copied; otherwise the paste
    /// is the input's. Returns whether the window took it.
    pub(crate) fn one_window_stage_paste(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.one_window_owns_list() || !self.filter_text.is_empty() || self.one_window.in_view()
        {
            return false;
        }
        let Some(text) = self.one_window_clipboard_text(cx) else {
            return false;
        };
        if self.one_window.tabs_visible() && self.filter_text.is_empty() {
            return super::subject_tabs::copy_source(&text, 0).is_some_and(|s| {
                self.change_subjects(sk_one_window::window::subject_tabs::TabsEvent::Paste(s), cx)
            });
        }
        self.one_window_handle(Event::Paste(text), cx).taken()
    }

    /// The clipboard's text: the process-local copy under owned evaluation, which never reads the
    /// system pasteboard.
    fn one_window_clipboard_text(&self, cx: &mut Context<Self>) -> Option<String> {
        match crate::runtime_policy::owned_evaluation() {
            Some(policy) => policy
                .owned_copy_snapshot()
                .ok()
                .flatten()
                .map(|copy| copy.text().to_string()),
            None => cx.read_from_clipboard().and_then(|item| item.text()),
        }
    }

    /// One of the staged subject's rows ran. `None` when it isn't one this module answers.
    pub(crate) fn one_window_run_act(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> Option<anyhow::Result<()>> {
        if row.key == "act:Send it to someone" {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Messaging isn't available here yet. Nothing was sent.",
                cx,
            );
            return Some(Err(anyhow::anyhow!("messaging unavailable")));
        }
        let subject = self.one_window_host.acting_on.clone()?;
        if subject.kind != SubjectKind::Clipboard {
            return None;
        }
        let reading = |verb| Reading {
            verb,
            sentence: String::new(),
            slots: Vec::new(),
            subject: Some(subject.clone()),
        };
        match row.title.as_str() {
            "Remind me about it" => {
                let form = Form::new(&reading(Verb::Remind));
                self.one_window_open_hosted(HostedView::Form(form), Some(String::new()), cx);
            }
            "Add it to today's note" => self.one_window_open_note(subject.title.clone(), cx),
            "Show all of it" => {
                self.one_window_open_clipboard_answer("What did I copy?".into(), cx)
            }
            "Open it" => {
                let url = if subject.title.contains("://") {
                    subject.title.clone()
                } else {
                    format!("https://{}", subject.title)
                };
                if let Err(error) = crate::browser_history::open_browser_history_url(&url) {
                    return Some(Err(error));
                }
                self.one_window_ran_here(cx);
            }
            _ => return None,
        }
        Some(Ok(()))
    }

    /// Adds `line` to today's page and opens it.
    pub(crate) fn one_window_open_note(&mut self, line: String, cx: &mut Context<Self>) {
        self.open_note_in_editor_soon(
            crate::notes::NoteTarget::Append(line),
            crate::DayPageBack::Leave,
            cx,
        );
    }

    /// The calculator's answer, in place: the number it came to.
    pub(crate) fn one_window_open_calculator_answer(
        &mut self,
        result: &crate::calculator::CalculatorInlineResult,
        cx: &mut Context<Self>,
    ) {
        let question = result.raw_input.clone();
        let mut answer = Answer::asking(question.clone());
        answer.arrived(vec![Fact::new(
            result.formatted.clone(),
            format!("{} · {}", result.operation_name, result.words),
            Source::YourWords,
        )]);
        let view = HostedView::Answer(answer);
        let spec = ViewSpec {
            name: view.name().to_string(),
            label: view.label(),
            enter: Some(view.enter()),
            takes_input: true,
            text: None,
            leave_text: None,
        };
        self.one_window_host.view = Some(view);
        let mut turn = Vec::new();
        if self.one_window.text() != question {
            turn.push(Event::Text(question));
        }
        turn.push(Event::View(ViewOp::Open(spec)));
        self.one_window_handle_turn(turn, cx);
    }

    /// What was copied, as a card: the answer to "what did I copy".
    pub(crate) fn one_window_open_clipboard_answer(
        &mut self,
        question: String,
        cx: &mut Context<Self>,
    ) {
        let copied = self.one_window_clipboard_text(cx);
        let mut answer = Answer::asking(question);
        self.one_window_open_hosted(HostedView::Answer(answer.clone()), None, cx);
        let found = copied
            .as_deref()
            .and_then(sk_one_window::clipboard::subject)
            .map(|subject| {
                vec![Fact::new(
                    subject.title,
                    subject.sub,
                    Source::Staged(SubjectKind::Clipboard),
                )]
            })
            .unwrap_or_default();
        answer.arrived(found);
        self.one_window_answer_arrived(answer, cx);
    }

    /// The answer came in. ↵'s new words aren't the user's doing, so the reducer guards Enter.
    pub(crate) fn one_window_answer_arrived(&mut self, answer: Answer, cx: &mut Context<Self>) {
        let target = answer.enter();
        if let Some(HostedView::Answer(open)) = self.one_window_host.view.as_mut() {
            *open = answer;
        }
        self.one_window_handle(Event::ViewEnter(target), cx);
    }

    /// One of Jev's readings that fills in a form or a note instead of running at once.
    pub(crate) fn one_window_open_plan(
        &mut self,
        plan: &crate::intent::Plan,
        cx: &mut Context<Self>,
    ) -> bool {
        let [crate::intent::Step::Capture {
            target,
            body,
            when,
            subject,
            ..
        }] = plan.steps.as_slice()
        else {
            return false;
        };
        let at = |words: &str| plan.request.to_lowercase().find(&words.to_lowercase());
        let slot = |name, value: &str| {
            let slot = sk_one_window::form::Slot::new(name, value, Source::YourWords);
            match at(value) {
                Some(at) => slot.at(at),
                None => slot,
            }
        };
        let verb = match target.as_str() {
            "todo" | "reminder" => Verb::Remind,
            "note" => {
                match subject {
                    Some(subject) => self.capture_note_with_screen(body, subject, cx),
                    None => self.one_window_open_note(body.clone(), cx),
                }
                return true;
            }
            _ => return false,
        };
        let mut slots = vec![slot(SlotName::What, body)];
        if let Some(when) = when.as_deref() {
            slots.push(slot(SlotName::When, when));
        }
        let form = Form::new(&Reading {
            verb,
            sentence: plan.request.clone(),
            slots,
            subject: self.one_window.staged().map(|s| s.subject.clone()),
        });
        self.one_window_open_hosted(HostedView::Form(form), Some(String::new()), cx);
        true
    }
}
