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
    #[default]
    Thing,
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
            None => entry.risk.words().to_string(),
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

fn act(icon: &str, title: &str, act: RowAct) -> Row {
    Row {
        key: format!("act:{title}"),
        title: title.to_string(),
        enter_label: Some(title.to_string()),
        icon: Some(icon.to_string()),
        score: 100,
        head: None,
        learned: false,
        fallback: false,
        act,
    }
}

/// What you can do to an entry: stop or send a waiting message; undo, run again; open its
/// script; share it.
pub fn entry_rows(entry: &Entry) -> Vec<Row> {
    let mut rows = Vec::new();
    if entry.state == EntryState::Holding {
        rows.push(act("undo", "Don't send it", RowAct::Unsend));
        rows.push(act("message", "Send it now", RowAct::SendNow));
    } else {
        if entry.risk == crate::ledger::Risk::Undo && entry.state == EntryState::Done {
            rows.push(act("undo", "Undo it", RowAct::Undo));
        }
        if entry.state != EntryState::Undone || entry.held.is_some() {
            rows.push(act("again", "Run it again", RowAct::Again));
        }
    }
    if let Some(name) = &entry.script {
        rows.push(act(
            "script",
            &format!("Open “{name}” in {SAVED_SCRIPT_FILE_NAME}"),
            RowAct::OpenScript,
        ));
    }
    rows.push(act("copy", "Share it", RowAct::Share));
    rows
}

/// How to share an entry: copy it as a script, copy a link, send it to someone.
pub fn share_rows(entry: &Entry) -> Vec<Row> {
    let name = entry.script.clone().unwrap_or_else(|| entry.title.clone());
    vec![
        Row::new("share:copy", "Copy it to the clipboard as a script")
            .with_icon("copy")
            .with_enter_label(format!("Copy “{name}”")),
        Row::new("share:link", "Copy a link anyone on the team can open").with_icon("open"),
        Row::new("share:send", "Send it to someone").with_icon("message"),
    ]
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
        };
        assert_eq!(s.matches("undo").len(), 1);
        assert_eq!(s.pinned()[0].head.as_deref(), Some(words::STAGED));
        assert!(s.pinned()[1].head.is_none());
    }
}
