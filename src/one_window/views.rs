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

pub(crate) const FORM: &str = "form";
pub(crate) const ANSWER: &str = "answer";
pub(crate) const NOTE: &str = "note";

/// A line being added to one of your notes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NoteView {
    pub(crate) title: String,
    /// What the note already says.
    pub(crate) lines: Vec<String>,
    /// The line Enter adds: what's in the input.
    pub(crate) line: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostedView {
    Form(Form),
    Answer(Answer),
    Note(NoteView),
}

impl HostedView {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            HostedView::Form(_) => FORM,
            HostedView::Answer(_) => ANSWER,
            HostedView::Note(_) => NOTE,
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
            HostedView::Note(note) => format!("Note · {}", note.title),
        }
    }

    pub(crate) fn enter(&self) -> EnterTarget {
        match self {
            HostedView::Form(form) => form.enter(),
            HostedView::Answer(answer) => answer.enter(),
            HostedView::Note(note) => EnterTarget::new(
                "note:add",
                if note.line.trim().is_empty() {
                    "Write the line first".to_string()
                } else {
                    format!("Add it to “{}”", note.title)
                },
            ),
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
            HostedView::Note(note) => note.lines.len() + 1,
        }
    }
}

/// The words the day note shows for each of its lines, without their timestamps.
fn day_note_lines() -> Vec<String> {
    let substrate = crate::brain::substrate::BrainSubstrate::default_kit();
    let (date, _) = substrate.capture_day_destination(chrono::Utc::now());
    let text = std::fs::read_to_string(substrate.paths().day_page(date)).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("- "))
        .map(|line| {
            let line = line.trim_start_matches("- ");
            let line = line.trim_start_matches("[ ] ").trim_start_matches("[x] ");
            match line.split_once(' ') {
                Some((time, rest)) if time.contains(':') && time.len() <= 8 => rest.to_string(),
                _ => line.to_string(),
            }
        })
        .collect()
}

fn append_to_day(entry: crate::brain::substrate::DayEntry) -> Result<(), &'static str> {
    crate::brain::substrate::BrainSubstrate::default_kit()
        .append_to_day(chrono::Utc::now(), entry)
        .map_err(|_| "one_window_note_write_failed")
}

impl ScriptListApp {
    /// Opens a view in place of the list. With `text`, the input shows those words while it's open;
    /// Esc brings back what was typed.
    pub(crate) fn one_window_open_view(
        &mut self,
        view: HostedView,
        text: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let spec = ViewSpec {
            name: view.name().to_string(),
            label: view.label(),
            enter: Some(view.enter()),
            takes_input: true,
            text,
            leave_text: None,
        };
        self.one_window_host.view = Some(view);
        let px = self.one_window_natural_height();
        self.one_window_handle_turn(
            vec![Event::View(ViewOp::Open(spec)), Event::Content { px }],
            cx,
        );
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
    pub(crate) fn one_window_view_key(&mut self, key: Key, cx: &mut Context<Self>) -> Vec<Event> {
        let top = self
            .one_window
            .views()
            .top()
            .map(|top| top.name.clone())
            .unwrap_or_default();
        match top.as_str() {
            FORM | ANSWER | NOTE => self.one_window_hosted_key(key, cx),
            _ => {
                tracing::info!(
                    target: "script_kit::one_window",
                    event = "effect_not_hosted",
                    view = %top,
                    key = ?key,
                );
                Vec::new()
            }
        }
    }

    fn one_window_hosted_key(&mut self, key: Key, cx: &mut Context<Self>) -> Vec<Event> {
        let Some(view) = self.one_window_host.view.as_mut() else {
            return Vec::new();
        };
        let before = view.enter();
        let finished = match view {
            HostedView::Form(form) => match form.key(&key) {
                FormKey::Submit(done) => Some(Ok(done)),
                FormKey::Changed => {
                    let value = form
                        .caret
                        .and_then(|i| form.fields.get(i))
                        .map(|f| f.value.clone())
                        .unwrap_or_default();
                    self.one_window_host.view_text = Some(value.clone());
                    self.one_window_set_input(value, cx);
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
            HostedView::Note(note) if key == Key::Enter && !note.line.trim().is_empty() => {
                let line = note.line.trim().to_string();
                let title = note.title.clone();
                Some(
                    append_to_day(crate::brain::substrate::DayEntry::Capture {
                        text: line.clone(),
                    })
                    .map(|()| Done {
                        icon: "append".into(),
                        title: format!("Added to “{title}” · {line}"),
                        ..Done::default()
                    }),
                )
            }
            HostedView::Answer(_) | HostedView::Note(_) => None,
        };
        let mut events = Vec::new();
        match finished {
            Some(Ok(done)) => {
                if let Some(HostedView::Form(form)) = &self.one_window_host.view {
                    if let Err(reason) = self.one_window_finish_form(form.clone()) {
                        tracing::warn!(target: "script_kit::one_window", reason, "form not saved");
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
            Verb::Message | Verb::Schedule | Verb::Move | Verb::Other(_) => Ok(()),
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
            HostedView::Note(note) => note.line = text,
            HostedView::Answer(_) => return None,
        }
        let after = view.enter();
        (after != before).then_some(Event::ViewEnter(after))
    }

    /// Tab and ← → belong to an open form: fields and their other values, not the input's caret or
    /// the launcher's Tab. Enter, Esc and the arrows up and down reach the reducer through the
    /// list's own paths.
    pub(crate) fn one_window_view_takes_key(
        &mut self,
        key: &str,
        modifiers: &gpui::Modifiers,
        cx: &mut Context<Self>,
    ) -> bool {
        if !matches!(self.current_view, crate::AppView::ScriptList)
            || !matches!(self.one_window_hosted_view(), Some(HostedView::Form(_)))
            || modifiers.platform
            || modifiers.alt
            || modifiers.control
        {
            return false;
        }
        let key = match key.to_ascii_lowercase().as_str() {
            "tab" if modifiers.shift => Key::Up,
            "tab" => Key::Tab,
            "right" | "arrowright" => Key::Right,
            "left" | "arrowleft" => Key::Other("left".into()),
            _ => return false,
        };
        self.one_window_key(key, cx);
        true
    }

    /// After every turn: a view the reducer closed is gone here too, and when what's drawn in
    /// place of the list changed size (a view opened or closed, a subject was staged or let go)
    /// the reducer hears the window's new natural height.
    pub(crate) fn one_window_views_follow(&mut self, cx: &mut Context<Self>) {
        if self.one_window_host.view.is_some() && self.one_window_hosted_view().is_none() {
            self.one_window_host.view = None;
            self.one_window_host.view_text = None;
        }
        let drawn = self.one_window_view_height();
        if drawn != self.one_window_host.view_px {
            self.one_window_host.view_px = drawn;
            let px = self.one_window_natural_height();
            self.one_window_handle(Event::Content { px }, cx);
        }
    }

    // ------------------------------------------------------------------ what opens them

    /// ⌘V in the main window. Into the empty input it stages what was copied; otherwise the paste
    /// is the input's. Returns whether the window took it.
    pub(crate) fn one_window_paste(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.one_window_owns_list() {
            return false;
        }
        let Some(text) = self.one_window_clipboard_text(cx) else {
            return false;
        };
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
            "Send it to someone" => {
                let form = Form::new(&reading(Verb::Message));
                self.one_window_open_view(HostedView::Form(form), Some(String::new()), cx);
            }
            "Remind me about it" => {
                let form = Form::new(&reading(Verb::Remind));
                self.one_window_open_view(HostedView::Form(form), Some(String::new()), cx);
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

    /// Today's note under the input, with `line` ready to add.
    pub(crate) fn one_window_open_note(&mut self, line: String, cx: &mut Context<Self>) {
        let note = NoteView {
            title: "Today".into(),
            lines: day_note_lines(),
            line: line.clone(),
        };
        self.one_window_host.view_text = Some(line.clone());
        self.one_window_open_view(HostedView::Note(note), Some(line), cx);
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
        let px = self.one_window_natural_height();
        let mut turn = Vec::new();
        if self.one_window.text() != question {
            turn.push(Event::Text(question));
        }
        turn.extend([Event::View(ViewOp::Open(spec)), Event::Content { px }]);
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
        self.one_window_open_view(HostedView::Answer(answer.clone()), None, cx);
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
        let px = self.one_window_natural_height();
        self.one_window_handle(Event::Content { px }, cx);
    }

    /// One of Jev's readings that fills in a form or a note instead of running at once.
    pub(crate) fn one_window_open_plan(
        &mut self,
        plan: &crate::intent::Plan,
        cx: &mut Context<Self>,
    ) -> bool {
        let [crate::intent::Step::Capture {
            target, body, when, ..
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
                self.one_window_open_note(body.clone(), cx);
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
        self.one_window_open_view(HostedView::Form(form), Some(String::new()), cx);
        true
    }
}
