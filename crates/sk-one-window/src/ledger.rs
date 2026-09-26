//! Everything that ran, kept so the line above the input can say what ran last and so
//! "what did I run today" has an answer.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::contract::UNDOABLE_ICONS;

/// How a run can be taken back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    /// Can't be taken back once it happens (a message): it waits first.
    Hold,
    /// Happens at once and can be undone (moving an event, trashing a file).
    Undo,
    /// Just happens (opening an app, an answer). It can only be run again.
    #[default]
    None,
}

impl Risk {
    /// Plain words for the risk, shown under a staged entry.
    pub fn words(self) -> &'static str {
        match self {
            Risk::Hold => "Waits before it goes",
            Risk::Undo => "Can be undone",
            Risk::None => "Nothing to undo",
        }
    }
}

/// The risk of a run: an explicit one wins, a message form waits, any other form can be undone,
/// and otherwise the icon decides.
pub fn risk_of(explicit: Option<Risk>, form_kind: Option<&str>, icon: Option<&str>) -> Risk {
    if let Some(risk) = explicit {
        return risk;
    }
    match form_kind {
        Some("message") => return Risk::Hold,
        Some(_) => return Risk::Undo,
        None => {}
    }
    match icon {
        Some(icon) if UNDOABLE_ICONS.contains(&icon) => Risk::Undo,
        _ => Risk::None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryState {
    Done,
    Undone,
    Holding,
    Sent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: u64,
    pub at: Ms,
    pub icon: String,
    pub title: String,
    /// The sentence that ran it.
    pub said: String,
    pub risk: Risk,
    pub state: EntryState,
    /// The app it launched, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// The saved script's name, once it was saved into `saved.ts`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    /// The held message this entry waits on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held: Option<u64>,
    /// Came from history rather than this session: shown with its time, not "just now".
    #[serde(default)]
    pub seeded: bool,
    /// When a seeded entry ran, in words ("9:41 AM").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    /// For "Undid · …" entries, the entry that was undone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_of: Option<u64>,
}

/// What a new entry needs; the ledger fills in id, time and state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewEntry {
    pub icon: String,
    pub title: String,
    pub said: String,
    pub risk: Risk,
    pub app: Option<String>,
    pub held: Option<u64>,
    pub undo_of: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct Ledger {
    entries: Vec<Entry>,
    next_id: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds history from before this session.
    pub fn seed(&mut self, mut entry: Entry) {
        self.next_id += 1;
        entry.id = self.next_id;
        entry.seeded = true;
        self.entries.push(entry);
    }

    pub fn record(&mut self, now: Ms, new: NewEntry) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        let state = if new.held.is_some() {
            EntryState::Holding
        } else {
            EntryState::Done
        };
        self.entries.push(Entry {
            id,
            at: now,
            icon: if new.icon.is_empty() {
                "check".into()
            } else {
                new.icon
            },
            title: new.title,
            said: new.said,
            risk: new.risk,
            state,
            app: new.app,
            script: None,
            held: new.held,
            seeded: false,
            when: None,
            undo_of: new.undo_of,
        });
        id
    }

    pub fn last(&self) -> Option<&Entry> {
        self.entries.last()
    }

    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// A held message went out: it is now the newest thing that happened.
    pub fn mark_sent(&mut self, id: u64, now: Ms) {
        if let Some(ix) = self.entries.iter().position(|e| e.id == id) {
            let mut entry = self.entries.remove(ix);
            entry.state = EntryState::Sent;
            entry.at = now;
            entry.seeded = false;
            self.entries.push(entry);
        }
    }

    pub fn mark_unsent(&mut self, id: u64) {
        if let Some(entry) = self.get_mut(id) {
            entry.state = EntryState::Undone;
        }
    }

    pub fn set_title(&mut self, id: u64, title: String) {
        if let Some(entry) = self.get_mut(id) {
            entry.title = title;
        }
    }

    pub fn set_script(&mut self, id: u64, name: String) {
        if let Some(entry) = self.get_mut(id) {
            entry.script = Some(name);
        }
    }

    /// Undo `id` and record that it was undone. Returns the new entry's id.
    pub fn undo(&mut self, now: Ms, id: u64) -> Option<u64> {
        let title = {
            let entry = self.get_mut(id)?;
            if entry.risk != Risk::Undo || entry.state != EntryState::Done {
                return None;
            }
            entry.state = EntryState::Undone;
            entry.title.clone()
        };
        Some(self.record(
            now,
            NewEntry {
                icon: "undo".into(),
                title: format!("Undid · {title}"),
                said: "undo that".into(),
                risk: Risk::None,
                undo_of: Some(id),
                ..NewEntry::default()
            },
        ))
    }

    /// Entries whose title or sentence contains every word of `query`, newest first.
    pub fn search(&self, query: &str) -> Vec<&Entry> {
        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
        self.entries
            .iter()
            .rev()
            .filter(|e| {
                let hay = format!("{} {}", e.title, e.said).to_lowercase();
                words.iter().all(|w| hay.contains(w.as_str()))
            })
            .collect()
    }

    /// Entries that ran in this session since `since`, newest first; seeded history counts as today.
    pub fn since(&self, since: Ms) -> Vec<&Entry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.seeded || e.at >= since)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_rules() {
        assert_eq!(risk_of(Some(Risk::None), Some("message"), None), Risk::None);
        assert_eq!(risk_of(None, Some("message"), None), Risk::Hold);
        assert_eq!(risk_of(None, Some("event"), None), Risk::Undo);
        assert_eq!(risk_of(None, None, Some("move")), Risk::Undo);
        assert_eq!(risk_of(None, None, Some("app")), Risk::None);
    }

    #[test]
    fn undo_only_undoable_done_entries() {
        let mut l = Ledger::new();
        let a = l.record(
            0,
            NewEntry {
                title: "Open Safari".into(),
                ..NewEntry::default()
            },
        );
        assert!(l.undo(10, a).is_none());
        let b = l.record(
            20,
            NewEntry {
                title: "Moved standup".into(),
                risk: Risk::Undo,
                ..NewEntry::default()
            },
        );
        let u = l.undo(30, b).unwrap();
        assert_eq!(l.get(b).unwrap().state, EntryState::Undone);
        assert_eq!(l.last().unwrap().id, u);
        assert_eq!(l.last().unwrap().title, "Undid · Moved standup");
        assert!(l.undo(40, b).is_none(), "undoing twice does nothing");
    }

    #[test]
    fn sent_messages_become_the_newest_entry() {
        let mut l = Ledger::new();
        let m = l.record(
            0,
            NewEntry {
                title: "To Sarah".into(),
                risk: Risk::Hold,
                held: Some(1),
                ..NewEntry::default()
            },
        );
        l.record(
            100,
            NewEntry {
                title: "Open Linear".into(),
                ..NewEntry::default()
            },
        );
        l.mark_sent(m, 8_000);
        let last = l.last().unwrap();
        assert_eq!(last.id, m);
        assert_eq!(last.state, EntryState::Sent);
    }

    #[test]
    fn search_matches_every_word() {
        let mut l = Ledger::new();
        l.record(
            0,
            NewEntry {
                title: "Moved standup to 10:30".into(),
                ..NewEntry::default()
            },
        );
        l.record(
            1,
            NewEntry {
                title: "Open Linear".into(),
                ..NewEntry::default()
            },
        );
        assert_eq!(l.search("standup moved").len(), 1);
        assert_eq!(l.search("").len(), 2);
    }
}
