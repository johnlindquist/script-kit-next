//! Forms Jev fills in, assembled from its reading of the sentence.
//!
//! There is no form per intent. The slots Jev read (who, what, when, where, the thing it acts on)
//! and the subject staged in the line decide which fields appear and in what order; each field
//! says where its value came from. A field the reading needs but Jev couldn't fill is the one that
//! takes the caret, and Enter moves there instead of submitting.

use serde::{Deserialize, Serialize};

use crate::enter::EnterTarget;
use crate::ledger::Risk;
use crate::stage::{Subject, SubjectKind};
use crate::window::{Done, Key};

/// What the sentence does, as Jev read it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verb {
    Message,
    Remind,
    Schedule,
    /// Move something that's already on the calendar.
    Move,
    Note,
    /// A reading with no verb of its own: the form is just what Jev read.
    Other(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SlotName {
    Who,
    What,
    When,
    Where,
    /// The thing a move acts on ("Design review · Fri 2 PM").
    Event,
    Body,
}

/// Where a value came from, said under the field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    /// The typed sentence.
    YourWords,
    /// The subject in the line above the input.
    Staged(SubjectKind),
    /// Something Jev looked up: "calendar", "contacts", "notes".
    Known(String),
    /// What the verb does when you don't say ("Right now").
    Default,
    /// You picked it with ← →.
    You,
}

impl Source {
    pub fn words(&self) -> String {
        match self {
            Source::YourWords => "from your words".into(),
            Source::Staged(kind) => format!("from the {}", staged_noun(*kind)),
            Source::Known(from) => format!("from your {from}"),
            Source::Default => "unless you say".into(),
            Source::You => "you picked".into(),
        }
    }
}

fn staged_noun(kind: SubjectKind) -> &'static str {
    match kind {
        SubjectKind::Clipboard => "clipboard",
        SubjectKind::Selection => "selection",
        SubjectKind::Window => "front window",
        SubjectKind::Message => "message",
        SubjectKind::Answer => "answer",
        SubjectKind::Entry => "last run",
        SubjectKind::App => "app",
        SubjectKind::Script => "script",
        SubjectKind::Thing => "line above",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alt {
    pub value: String,
    pub from: Source,
}

/// One slot of Jev's reading.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub name: SlotName,
    pub value: String,
    pub from: Source,
    /// Where in the sentence Jev read it (a byte offset); `None` when it came from elsewhere.
    #[serde(default)]
    pub at: Option<usize>,
    /// Other values Jev considered, most likely first.
    #[serde(default)]
    pub alts: Vec<Alt>,
}

impl Slot {
    pub fn new(name: SlotName, value: impl Into<String>, from: Source) -> Self {
        Self {
            name,
            value: value.into(),
            from,
            at: None,
            alts: Vec::new(),
        }
    }

    pub fn at(mut self, at: usize) -> Self {
        self.at = Some(at);
        self
    }

    pub fn with_alt(mut self, value: impl Into<String>, from: Source) -> Self {
        self.alts.push(Alt {
            value: value.into(),
            from,
        });
        self
    }
}

/// Jev's reading of a sentence, as a form needs it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub verb: Verb,
    pub sentence: String,
    pub slots: Vec<Slot>,
    /// What was staged in the line when the sentence was read.
    #[serde(default)]
    pub subject: Option<Subject>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldState {
    Filled,
    /// The reading needs it and nobody said it.
    Needed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: SlotName,
    pub label: String,
    pub value: String,
    /// `None` for a field nobody filled.
    pub from: Option<Source>,
    pub alts: Vec<Alt>,
    pub state: FieldState,
}

impl Field {
    /// The words under the field: where its value came from, or that it needs you.
    pub fn source_words(&self) -> String {
        match &self.from {
            Some(from) => from.words(),
            None => "Needs you".into(),
        }
    }
}

/// The slots a verb can't go without.
fn needs(verb: &Verb) -> &'static [SlotName] {
    match verb {
        Verb::Message => &[SlotName::Who, SlotName::What],
        Verb::Remind | Verb::Schedule => &[SlotName::What, SlotName::When],
        Verb::Move => &[SlotName::Event, SlotName::When],
        Verb::Note => &[SlotName::What],
        Verb::Other(_) => &[],
    }
}

/// What a verb does about a slot you didn't say, when doing nothing is a real answer.
fn default_for(verb: &Verb, name: SlotName) -> Option<&'static str> {
    match (verb, name) {
        (Verb::Message, SlotName::When) => Some("Right now"),
        (Verb::Note, SlotName::Where) => Some("Today's note"),
        _ => None,
    }
}

fn label(verb: &Verb, name: SlotName) -> &'static str {
    match (verb, name) {
        (Verb::Message, SlotName::Who) => "To",
        (Verb::Message, SlotName::What) => "Message",
        (Verb::Note, SlotName::Where) => "Note",
        (_, SlotName::Who) => "Who",
        (_, SlotName::What) => "What",
        (_, SlotName::When) => "When",
        (_, SlotName::Where) => "Where",
        (_, SlotName::Event) => "Event",
        (_, SlotName::Body) => "Body",
    }
}

/// The slot the staged subject stands for when the sentence didn't put it anywhere.
fn subject_slot(verb: &Verb) -> SlotName {
    match verb {
        Verb::Move => SlotName::Event,
        _ => SlotName::What,
    }
}

/// Jev's reading as the fields a form shows, in the order the sentence said them. Slots Jev read
/// come first, in sentence order; the staged subject fills the slot it stands for; then what the
/// verb still needs (empty, for you to say) and what it does by default.
pub fn fields(reading: &Reading) -> Vec<Field> {
    let verb = &reading.verb;
    let mut slots: Vec<&Slot> = Vec::new();
    for slot in &reading.slots {
        if !slot.value.trim().is_empty() && slots.iter().all(|s| s.name != slot.name) {
            slots.push(slot);
        }
    }
    slots.sort_by_key(|s| (s.at.is_none(), s.at.unwrap_or(usize::MAX)));
    let mut out: Vec<Field> = slots
        .into_iter()
        .map(|s| Field {
            name: s.name,
            label: label(verb, s.name).into(),
            value: s.value.trim().to_string(),
            from: Some(s.from.clone()),
            alts: s.alts.clone(),
            state: FieldState::Filled,
        })
        .collect();
    let has = |out: &[Field], name: SlotName| out.iter().any(|f| f.name == name);
    if let Some(subject) = &reading.subject {
        let name = subject_slot(verb);
        if !has(&out, name) && !subject.title.trim().is_empty() {
            out.push(Field {
                name,
                label: label(verb, name).into(),
                value: subject.title.clone(),
                from: Some(Source::Staged(subject.kind)),
                alts: Vec::new(),
                state: FieldState::Filled,
            });
        }
    }
    for &name in needs(verb) {
        if !has(&out, name) {
            out.push(Field {
                name,
                label: label(verb, name).into(),
                value: String::new(),
                from: None,
                alts: Vec::new(),
                state: FieldState::Needed,
            });
        }
    }
    for name in [SlotName::When, SlotName::Where] {
        if let Some(value) = default_for(verb, name).filter(|_| !has(&out, name)) {
            out.push(Field {
                name,
                label: label(verb, name).into(),
                value: value.into(),
                from: Some(Source::Default),
                alts: Vec::new(),
                state: FieldState::Filled,
            });
        }
    }
    if out.is_empty() && !reading.sentence.trim().is_empty() {
        out.push(Field {
            name: SlotName::What,
            label: label(verb, SlotName::What).into(),
            value: reading.sentence.trim().to_string(),
            from: Some(Source::YourWords),
            alts: Vec::new(),
            state: FieldState::Filled,
        });
    }
    out
}

/// What a key in a form did.
#[derive(Clone, Debug, PartialEq)]
pub enum FormKey {
    /// The caret or a value moved; Enter may say something new.
    Changed,
    /// The form is done: record and finish it.
    Submit(Done),
    /// The form doesn't use this key.
    Unused,
}

/// A form open under the input: its fields, where the caret is, and what you picked.
#[derive(Clone, Debug, PartialEq)]
pub struct Form {
    pub verb: Verb,
    pub sentence: String,
    pub fields: Vec<Field>,
    /// The field that takes what you type next; `None` when every field is filled and none was
    /// picked yet.
    pub caret: Option<usize>,
}

impl Form {
    pub fn new(reading: &Reading) -> Self {
        let fields = fields(reading);
        let caret = fields.iter().position(|f| f.state == FieldState::Needed);
        Self {
            verb: reading.verb.clone(),
            sentence: reading.sentence.clone(),
            fields,
            caret,
        }
    }

    /// A new reading of edited words. Values you picked stay yours; the caret stays on its field
    /// unless a field now needs you.
    pub fn reread(&mut self, reading: &Reading) {
        let at = self.caret.and_then(|i| self.fields.get(i)).map(|f| f.name);
        let picked: Vec<Field> = self
            .fields
            .iter()
            .filter(|f| f.from == Some(Source::You))
            .cloned()
            .collect();
        let mut next = Form::new(reading);
        for pick in picked {
            if let Some(f) = next.fields.iter_mut().find(|f| f.name == pick.name) {
                *f = pick;
            }
        }
        next.caret = next
            .fields
            .iter()
            .position(|f| f.state == FieldState::Needed)
            .or_else(|| at.and_then(|name| next.fields.iter().position(|f| f.name == name)));
        *self = next;
    }

    pub fn field(&self, name: SlotName) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    fn value(&self, name: SlotName) -> &str {
        self.field(name).map_or("", |f| f.value.as_str())
    }

    fn needed(&self) -> Option<usize> {
        self.fields
            .iter()
            .position(|f| f.state == FieldState::Needed)
    }

    /// A message whose text is only the instruction ("tell sarah") says nothing yet.
    fn echo(&self) -> Option<usize> {
        let plain = |s: &str| {
            s.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let sentence = plain(&self.sentence);
        self.fields.iter().position(|f| {
            self.verb == Verb::Message
                && f.name == SlotName::What
                && f.from == Some(Source::YourWords)
                && plain(&f.value) == sentence
        })
    }

    pub fn key(&mut self, key: &Key) -> FormKey {
        let n = self.fields.len();
        if n == 0 {
            return FormKey::Unused;
        }
        match key {
            Key::Tab | Key::Down => {
                self.caret = Some(self.caret.map_or(0, |i| (i + 1) % n));
                FormKey::Changed
            }
            Key::Up => {
                self.caret = Some(self.caret.map_or(n - 1, |i| (i + n - 1) % n));
                FormKey::Changed
            }
            Key::Right => self.cycle(1),
            Key::Other(k) if k.eq_ignore_ascii_case("left") => self.cycle(-1),
            Key::Enter => match self.needed().or_else(|| self.echo()) {
                Some(i) => {
                    self.caret = Some(i);
                    FormKey::Changed
                }
                None => FormKey::Submit(self.done()),
            },
            _ => FormKey::Unused,
        }
    }

    /// ← → on the caret's field: Jev's other values, and the one it had.
    fn cycle(&mut self, dir: i32) -> FormKey {
        let Some(f) = self.caret.and_then(|i| self.fields.get_mut(i)) else {
            return FormKey::Unused;
        };
        if f.alts.is_empty() {
            return FormKey::Unused;
        }
        let take = if dir > 0 { 0 } else { f.alts.len() - 1 };
        let next = f.alts.remove(take);
        if f.state == FieldState::Filled {
            let old = Alt {
                value: std::mem::take(&mut f.value),
                from: f.from.take().unwrap_or(Source::YourWords),
            };
            if dir > 0 {
                f.alts.push(old);
            } else {
                f.alts.insert(0, old);
            }
        }
        f.value = next.value;
        f.from = Some(Source::You);
        f.state = FieldState::Filled;
        FormKey::Changed
    }

    /// What ↵ says: what's missing, what goes where, or the verb.
    pub fn enter(&self) -> EnterTarget {
        let who = self.value(SlotName::Who);
        let who = if who.is_empty() { "them" } else { who };
        if let Some(i) = self.needed() {
            let f = &self.fields[i];
            let label = match (&self.verb, f.name) {
                (Verb::Message, SlotName::Who) => "Say who gets it".to_string(),
                (Verb::Message, SlotName::What) => format!("Say what to tell {who}"),
                (_, SlotName::When) => "Say when".to_string(),
                (_, name) => format!("Say the {}", label(&self.verb, name).to_lowercase()),
            };
            return EnterTarget::new(format!("form:needs:{:?}", f.name), label);
        }
        if self.echo().is_some() {
            return EnterTarget::new("form:say", format!("Say what to tell {who}"));
        }
        let when = self.value(SlotName::When);
        let later = if when.is_empty() || when == "Right now" {
            String::new()
        } else {
            format!(" · {when}")
        };
        match self.verb {
            Verb::Message => {
                let what = self.field(SlotName::What);
                match what {
                    Some(w) if w.from != Some(Source::YourWords) => EnterTarget::new(
                        format!("form:{}{later}", w.value),
                        format!("Send “{}” to {who}{later}", w.value),
                    ),
                    _ => EnterTarget::new("form", format!("Send to {who}{later}")),
                }
            }
            Verb::Remind => EnterTarget::new("form", format!("Remind me{later}")),
            Verb::Schedule => EnterTarget::new("form", "Add it to the calendar"),
            Verb::Move => EnterTarget::new("form", format!("Move it{later}")),
            Verb::Note => EnterTarget::new(
                "form",
                format!("Add it to {}", self.value(SlotName::Where).trim()),
            ),
            Verb::Other(_) => EnterTarget::new("form", "Do it"),
        }
    }

    /// The form in one line, as the line above the input and the ledger say it.
    pub fn summary(&self) -> String {
        let when = self.value(SlotName::When);
        let join = |parts: &[&str]| {
            parts
                .iter()
                .filter(|p| !p.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" · ")
        };
        match self.verb {
            Verb::Message => {
                let later = if when == "Right now" { "" } else { when };
                join(&[
                    &format!(
                        "To {}: “{}”",
                        self.value(SlotName::Who),
                        self.value(SlotName::What)
                    ),
                    later,
                ])
            }
            Verb::Move => format!("{} → {when}", self.value(SlotName::Event)),
            Verb::Note => format!(
                "Added to “{}” · {}",
                self.value(SlotName::Where),
                self.value(SlotName::What)
            ),
            _ => join(&[
                self.value(SlotName::What),
                self.value(SlotName::Who),
                when,
                self.value(SlotName::Where),
            ]),
        }
    }

    fn done(&self) -> Done {
        let (icon, kind, risk) = match &self.verb {
            Verb::Message => ("message", "message", Risk::Hold),
            Verb::Remind => ("schedule", "reminder", Risk::Undo),
            Verb::Schedule => ("schedule", "event", Risk::Undo),
            Verb::Move => ("move", "move", Risk::Undo),
            Verb::Note => ("append", "note", Risk::Undo),
            Verb::Other(_) => ("check", "form", Risk::Undo),
        };
        Done {
            icon: icon.into(),
            title: self.summary(),
            risk: Some(risk),
            form_kind: Some(kind.into()),
            stay: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(fields: &[Field]) -> Vec<(SlotName, &str, FieldState)> {
        fields
            .iter()
            .map(|f| (f.name, f.value.as_str(), f.state))
            .collect()
    }

    #[test]
    fn a_reminder_asks_for_the_time_it_wasnt_given() {
        let form = Form::new(&Reading {
            verb: Verb::Remind,
            sentence: "remind me to call mom".into(),
            slots: vec![Slot::new(SlotName::What, "call mom", Source::YourWords).at(10)],
            subject: None,
        });
        assert_eq!(
            names(&form.fields),
            vec![
                (SlotName::What, "call mom", FieldState::Filled),
                (SlotName::When, "", FieldState::Needed),
            ]
        );
        assert_eq!(form.caret, Some(1), "the unfilled time takes the caret");
        assert_eq!(form.enter().label, "Say when");
        assert_eq!(form.fields[1].source_words(), "Needs you");
    }

    #[test]
    fn a_message_follows_the_sentence_and_holds_when_sent() {
        let mut form = Form::new(&Reading {
            verb: Verb::Message,
            sentence: "tell sarah the beta slipped to oct 14".into(),
            slots: vec![
                Slot::new(
                    SlotName::What,
                    "The beta slipped to oct 14",
                    Source::YourWords,
                )
                .at(11),
                Slot::new(SlotName::Who, "Sarah", Source::Known("contacts".into()))
                    .at(5)
                    .with_alt("Sarah Chen", Source::Known("contacts".into())),
            ],
            subject: None,
        });
        assert_eq!(
            names(&form.fields),
            vec![
                (SlotName::Who, "Sarah", FieldState::Filled),
                (
                    SlotName::What,
                    "The beta slipped to oct 14",
                    FieldState::Filled
                ),
                (SlotName::When, "Right now", FieldState::Filled),
            ]
        );
        assert_eq!(form.fields[0].label, "To");
        assert_eq!(form.fields[0].source_words(), "from your contacts");
        assert_eq!(form.fields[2].source_words(), "unless you say");
        assert_eq!(form.caret, None);
        assert_eq!(form.enter().label, "Send to Sarah");
        assert_eq!(form.key(&Key::Tab), FormKey::Changed);
        assert_eq!(form.key(&Key::Right), FormKey::Changed);
        assert_eq!(form.fields[0].value, "Sarah Chen");
        assert_eq!(form.fields[0].source_words(), "you picked");
        let FormKey::Submit(done) = form.key(&Key::Enter) else {
            panic!("a filled message sends");
        };
        assert_eq!(done.title, "To Sarah Chen: “The beta slipped to oct 14”");
        assert_eq!(done.risk, Some(Risk::Hold));
        assert_eq!(done.form_kind.as_deref(), Some("message"));
    }

    #[test]
    fn a_staged_subject_fills_what_the_sentence_left_out() {
        let form = Form::new(&Reading {
            verb: Verb::Message,
            sentence: "send this to mo".into(),
            slots: vec![Slot::new(SlotName::Who, "Mo", Source::YourWords).at(13)],
            subject: Some(Subject {
                kind: SubjectKind::Clipboard,
                title: "github.com/script-kit/app/pull/812".into(),
                ..Subject::default()
            }),
        });
        let what = form
            .field(SlotName::What)
            .expect("the clipboard is the message");
        assert_eq!(what.value, "github.com/script-kit/app/pull/812");
        assert_eq!(what.source_words(), "from the clipboard");
        assert_eq!(
            form.enter().label,
            "Send “github.com/script-kit/app/pull/812” to Mo"
        );
    }

    #[test]
    fn a_calendar_move_names_the_event_it_found() {
        let form = Form::new(&Reading {
            verb: Verb::Move,
            sentence: "push standup to 10:30".into(),
            slots: vec![
                Slot::new(SlotName::When, "10:30 AM", Source::YourWords).at(15),
                Slot::new(
                    SlotName::Event,
                    "Standup · 9:30 AM",
                    Source::Known("calendar".into()),
                )
                .at(5),
            ],
            subject: None,
        });
        assert_eq!(
            names(&form.fields),
            vec![
                (SlotName::Event, "Standup · 9:30 AM", FieldState::Filled),
                (SlotName::When, "10:30 AM", FieldState::Filled),
            ]
        );
        assert_eq!(form.fields[0].source_words(), "from your calendar");
        assert_eq!(form.enter().label, "Move it · 10:30 AM");
        assert_eq!(form.summary(), "Standup · 9:30 AM → 10:30 AM");
    }

    #[test]
    fn an_unknown_shape_shows_what_jev_read_and_needs_nothing() {
        let form = Form::new(&Reading {
            verb: Verb::Other("rename".into()),
            sentence: "rename the q4 doc to launch plan".into(),
            slots: vec![
                Slot::new(SlotName::Body, "launch plan", Source::YourWords).at(20),
                Slot::new(SlotName::What, "Q4 doc", Source::Known("files".into())).at(7),
            ],
            subject: None,
        });
        assert_eq!(
            names(&form.fields),
            vec![
                (SlotName::What, "Q4 doc", FieldState::Filled),
                (SlotName::Body, "launch plan", FieldState::Filled),
            ]
        );
        assert_eq!(form.caret, None);
        assert_eq!(form.enter().label, "Do it");
        let bare = Form::new(&Reading {
            verb: Verb::Other("?".into()),
            sentence: "zorp the flim".into(),
            slots: Vec::new(),
            subject: None,
        });
        assert_eq!(
            names(&bare.fields),
            vec![(SlotName::What, "zorp the flim", FieldState::Filled)]
        );
    }

    #[test]
    fn enter_on_a_missing_field_moves_the_caret_there_instead_of_sending() {
        let mut form = Form::new(&Reading {
            verb: Verb::Message,
            sentence: "tell sarah".into(),
            slots: vec![Slot::new(SlotName::Who, "Sarah", Source::YourWords).at(5)],
            subject: None,
        });
        form.caret = Some(0);
        assert_eq!(form.enter().label, "Say what to tell Sarah");
        assert_eq!(form.key(&Key::Enter), FormKey::Changed);
        assert_eq!(form.caret, Some(1));
    }

    #[test]
    fn a_reread_keeps_what_you_picked() {
        let reading = |when: &str| Reading {
            verb: Verb::Remind,
            sentence: format!("remind me to stretch {when}"),
            slots: vec![
                Slot::new(SlotName::What, "stretch", Source::YourWords).at(13),
                Slot::new(SlotName::When, when, Source::YourWords)
                    .at(21)
                    .with_alt("4:00 PM", Source::Known("habits".into())),
            ],
            subject: None,
        };
        let mut form = Form::new(&reading("at 3"));
        form.caret = Some(1);
        form.key(&Key::Right);
        assert_eq!(form.fields[1].value, "4:00 PM");
        form.reread(&reading("at 3 please"));
        assert_eq!(form.fields[1].value, "4:00 PM");
        assert_eq!(form.caret, Some(1));
    }
}
