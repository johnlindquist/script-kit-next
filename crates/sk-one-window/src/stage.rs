//! Acting on something without a menu.
//!
//! `⌘K` puts the focused thing (the highlighted row when words are typed, else what "this" means)
//! in the line above the input, and the list fills with what you can do to it. `↑` from an empty
//! input does the same for the last thing that ran. There is no popup.

use serde::{Deserialize, Serialize};

use crate::contract::{SAVED_SCRIPT_FILE_NAME, words};
use crate::ledger::{Entry, EntryState};
use crate::row::{Row, RowAct};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SubjectKind {
    Entry,
    App,
    Script,
    Selection,
    Window,
    Message,
    /// An answer open in a view: "this" is what it says.
    Answer,
    #[default]
    Thing,
    /// Something copied, pasted into the empty input.
    Clipboard,
    /// A question waiting for its answer: its answers are the rows, and Esc lets it go.
    Question,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub kind: SubjectKind,
    /// What the line calls it: "Acting on", "Last", "Sending", "Selected", "Front window".
    pub label: String,
    pub title: String,
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub icon: String,
    /// The ledger entry, for entry subjects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<u64>,
    /// The row it came from, for row subjects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Staged {
    pub subject: Subject,
    pub rows: Vec<Row>,
    /// The list's header while staged.
    pub head: String,
    /// The host hasn't answered with this subject's actions yet.
    pub waiting_for_rows: bool,
    /// The entry's state when staged; a change rebuilds the rows.
    pub entry_state: Option<EntryState>,
    /// The ledger entry the line above the input showed when this was staged (`None`: nothing
    /// had run). While it stays staged the line keeps showing it, so news doesn't cover it.
    pub shown: Option<u64>,
}

impl Staged {
    /// The staged rows as the list's pinned rows: the first carries the header.
    pub fn pinned(&self) -> Vec<Row> {
        self.rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let mut r = r.clone();
                r.head = (i == 0).then(|| self.head.clone());
                r
            })
            .collect()
    }

    /// While staged, typing a row's words picks it instantly, like an app name would.
    pub fn matches(&self, text: &str) -> Vec<Row> {
        let t = text.trim().to_lowercase();
        if t.is_empty() {
            return Vec::new();
        }
        self.rows
            .iter()
            .filter(|r| {
                let title = r.title.to_lowercase();
                if t.contains(' ') {
                    title.contains(&t)
                } else {
                    title.split_whitespace().any(|w| w.starts_with(&t))
                }
            })
            .cloned()
            .collect()
    }
}

/// An entry as a subject: "Last · Moved standup".
pub fn entry_subject(entry: &Entry, label: &str) -> Subject {
    Subject {
        kind: SubjectKind::Entry,
        label: label.to_string(),
        title: entry.title.clone(),
        sub: match &entry.script {
            Some(name) => format!("saved as “{name}”"),
            None => match entry.risk {
                crate::ledger::Risk::Undo if !entry.can_undo() => {
                    crate::ledger::Risk::None.words().to_string()
                }
                risk => risk.words().to_string(),
            },
        },
        icon: if entry.icon.is_empty() {
            "check".into()
        } else {
            entry.icon.clone()
        },
        entry: Some(entry.id),
        row_key: None,
    }
}

/// A highlighted row as a subject.
pub fn row_subject(row: &Row) -> Subject {
    let kind = match row.icon.as_deref() {
        Some("app") => SubjectKind::App,
        Some("script") => SubjectKind::Script,
        _ => SubjectKind::Thing,
    };
    Subject {
        kind,
        label: words::ACTING_ON.into(),
        title: row.title.clone(),
        sub: String::new(),
        icon: row.icon.clone().unwrap_or_else(|| "builtin".into()),
        entry: None,
        row_key: Some(row.key.clone()),
    }
}

/// A row that acts on the staged subject. `say` is the sentence that does the same thing.
pub(crate) fn act(icon: &str, title: &str, say: &str, act: RowAct) -> Row {
    let row = Row::new(format!("act:{title}"), title)
        .with_enter_label(title)
        .with_icon(icon)
        .with_act(act);
    if title == "Send it to someone" {
        return row
            .with_enter_label("Unavailable")
            .with_sub("Messaging isn't available in the launcher yet");
    }
    if say.is_empty() {
        row
    } else {
        row.with_sub(format!("or say “{say}”"))
    }
}

/// What you can do to an entry: stop or send a waiting message; undo, run again; open its
/// script; share it.
pub fn entry_rows(entry: &Entry) -> Vec<Row> {
    let mut rows = Vec::new();
    if entry.state == EntryState::Holding {
        rows.push(act(
            "undo",
            "Don't send it",
            "wait, don't send",
            RowAct::Unsend,
        ));
        rows.push(act(
            "message",
            "Send it now",
            "send it now",
            RowAct::SendNow,
        ));
    } else {
        if entry.can_undo() {
            rows.push(act("undo", "Undo it", "undo that", RowAct::Undo));
        }
        if entry.state != EntryState::Undone || entry.held.is_some() {
            rows.push(act("again", "Run it again", "do that again", RowAct::Again));
        }
    }
    if let Some(name) = &entry.script {
        rows.push(act(
            "script",
            &format!("Open “{name}” in {SAVED_SCRIPT_FILE_NAME}"),
            "open that script",
            RowAct::OpenScript,
        ));
    }
    rows.push(act(
        "copy",
        "Share it",
        "share my last action",
        RowAct::Share,
    ));
    rows
}

/// How to share an entry: copy it as a script, copy a link, send it to someone.
pub fn share_rows() -> Vec<Row> {
    vec![
        act(
            "copy",
            "Copy it to the clipboard as a script",
            "copy it",
            RowAct::Host,
        ),
        act(
            "open",
            "Copy a link anyone on the team can open",
            "copy a link",
            RowAct::Host,
        ),
        act(
            "message",
            "Send it to someone",
            "send it to sarah",
            RowAct::Host,
        ),
    ]
}

/// What you can do to something that isn't an entry, until the host offers its own actions.
pub fn default_actions(subject: &Subject) -> Vec<Row> {
    let lower = subject.title.to_lowercase();
    match subject.kind {
        SubjectKind::App => vec![
            act("app", "Open it", &format!("open {lower}"), RowAct::Host),
            act("hide", "Quit it", "quit it", RowAct::Host),
            act(
                "shortcut",
                "Give it a keyboard shortcut",
                "give it a shortcut",
                RowAct::Host,
            ),
            act(
                "open",
                "Show it in Finder",
                "show it in finder",
                RowAct::Host,
            ),
        ],
        SubjectKind::Selection => vec![
            act("transform", "Rewrite it", "make it shorter", RowAct::Host),
            act("ask", "Explain it", "what does this mean", RowAct::Host),
            act(
                "message",
                "Send it to someone",
                "send this to mike",
                RowAct::Host,
            ),
            act(
                "append",
                "Add it to today's note",
                "add this to my notes",
                RowAct::Host,
            ),
        ],
        SubjectKind::Window => vec![
            act("window", "Tile it left", "tile it left", RowAct::Host),
            act("window", "Tile it right", "tile it right", RowAct::Host),
            act(
                "window",
                "Make it full screen",
                "make it full screen",
                RowAct::Host,
            ),
            act("hide", "Close it", "close it", RowAct::Host),
        ],
        SubjectKind::Message => {
            let first = lower
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string();
            vec![
                act("reply", "Reply", &format!("reply to {first}"), RowAct::Host),
                act(
                    "schedule",
                    "Remind me about it later",
                    "remind me about this at 4",
                    RowAct::Host,
                ),
                act("check", "Mark it done", "mark it done", RowAct::Host),
            ]
        }
        SubjectKind::Clipboard => crate::clipboard::actions(subject),
        // A question's rows are its answers, which the host that asked it supplies.
        SubjectKind::Question => Vec::new(),
        SubjectKind::Entry | SubjectKind::Script | SubjectKind::Answer | SubjectKind::Thing => {
            vec![
                act("open", "Open it", "open it", RowAct::Host),
                act("copy", "Copy it", "copy it", RowAct::Host),
                act(
                    "message",
                    "Send it to someone",
                    "send this to sarah",
                    RowAct::Host,
                ),
            ]
        }
    }
}

/// A short name for a sentence that did several things: each step's verb and what it was about.
pub fn script_name(said: &str, steps: &[String]) -> String {
    let parts: Vec<&str> = if steps.is_empty() {
        vec![said]
    } else {
        steps.iter().map(String::as_str).collect()
    };
    parts
        .iter()
        .take(3)
        .map(|t| step_name(t))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn step_name(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let Some(verb) = words.first() else {
        return String::new();
    };
    let mut i = 1;
    let prep = words
        .get(i)
        .filter(|w| matches!(w.to_lowercase().as_str(), "in" | "to" | "on" | "into"))
        .map(|w| w.to_lowercase());
    if prep.is_some() {
        i += 1;
    }
    if words
        .get(i)
        .is_some_and(|w| matches!(w.to_lowercase().as_str(), "the" | "my"))
    {
        i += 1;
    }
    let Some(obj) = words.get(i) else {
        return cap(&words.iter().take(3).copied().collect::<Vec<_>>().join(" "));
    };
    let obj: String = obj
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '#' | '\'' | '-' | '_'))
        .collect();
    let obj = if obj.starts_with('#') { obj } else { cap(&obj) };
    match prep {
        Some(p) => format!("{} {p} {obj}", cap(&verb.to_lowercase())),
        None => format!("{} {obj}", cap(&verb.to_lowercase())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Risk;

    fn entry(state: EntryState, risk: Risk) -> Entry {
        Entry {
            id: 1,
            at: 0,
            icon: "move".into(),
            title: "Moved standup".into(),
            said: "move standup to 10:30".into(),
            risk,
            state,
            app: None,
            script: None,
            held: None,
            seeded: false,
            when: None,
            before: 0,
            undo_kept: risk == Risk::Undo,
            undo_of: None,
        }
    }

    #[test]
    fn holding_entries_offer_stop_and_send() {
        let rows = entry_rows(&entry(EntryState::Holding, Risk::Hold));
        assert_eq!(rows[0].act, RowAct::Unsend);
        assert_eq!(rows[1].act, RowAct::SendNow);
    }

    #[test]
    fn undoable_done_entries_offer_undo_first() {
        let rows = entry_rows(&entry(EntryState::Done, Risk::Undo));
        assert_eq!(rows[0].act, RowAct::Undo);
        assert_eq!(rows[1].act, RowAct::Again);
        assert_eq!(rows.last().unwrap().act, RowAct::Share);
    }

    #[test]
    fn script_names_read_like_the_steps() {
        assert_eq!(
            script_name(
                "",
                &[
                    "post in #design the mock".into(),
                    "tell sarah it's up".into()
                ]
            ),
            "Post in #design + Tell Sarah"
        );
        assert_eq!(script_name("move standup to 10:30", &[]), "Move Standup");
    }

    #[test]
    fn staged_rows_match_their_words() {
        let s = Staged {
            subject: Subject::default(),
            rows: entry_rows(&entry(EntryState::Done, Risk::Undo)),
            head: words::STAGED.into(),
            waiting_for_rows: false,
            entry_state: None,
            shown: None,
        };
        assert_eq!(s.matches("undo").len(), 1);
        assert_eq!(s.pinned()[0].head.as_deref(), Some(words::STAGED));
        assert!(s.pinned()[1].head.is_none());
    }
}
