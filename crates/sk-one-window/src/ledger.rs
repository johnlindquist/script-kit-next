//! Everything that ran, kept so the line above the input can say what ran last and so
//! "what did I run today" has an answer.

use serde::{Deserialize, Serialize};

use crate::Ms;
use crate::contract::{UNDOABLE_ICONS, words};

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
    /// Came from history rather than this session: from an earlier run of the app, or a trace.
    #[serde(default)]
    pub seeded: bool,
    /// When a traced entry ran, in words ("9:41 AM"); history from an earlier run of the app
    /// says how long ago instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    /// For history from an earlier run of the app: how long before `at` it ran.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub before: Ms,
    /// The host kept a way to take it back. Only then does ⌘Z offer it; a run marked undoable
    /// that nobody can undo never says "Undid".
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub undo_kept: bool,
    /// For "Undid · …" entries, the entry that was undone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_of: Option<u64>,
}

fn is_zero(ms: &Ms) -> bool {
    *ms == 0
}

impl Entry {
    /// How long ago it ran, at `now`.
    pub fn age(&self, now: Ms) -> Ms {
        now.saturating_sub(self.at).saturating_add(self.before)
    }

    /// When it ran, in words: a traced entry's recorded time, else how long ago ("just now",
    /// ">5m ago").
    pub fn when_words(&self, now: Ms) -> String {
        match &self.when {
            Some(when) if self.seeded => when.clone(),
            _ => words::ago(self.age(now)),
        }
    }

    /// ⌘Z can take it back: it can be undone, it hasn't been, and the host kept how.
    pub fn can_undo(&self) -> bool {
        self.risk == Risk::Undo && self.state == EntryState::Done && self.undo_kept
    }
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
    /// The host kept a way to take it back.
    pub undo_kept: bool,
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

    /// Seeds history from before this session. Nothing seeded can be undone: its way back
    /// went with the run of the app that kept it.
    pub fn seed(&mut self, mut entry: Entry) {
        self.next_id += 1;
        entry.id = self.next_id;
        entry.seeded = true;
        entry.undo_kept = false;
        self.entries.push(entry);
    }

    /// Saving `id` as a script failed: it no longer says it was saved.
    pub fn forget_script(&mut self, id: u64) {
        if let Some(entry) = self.get_mut(id) {
            entry.script = None;
        }
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
            before: 0,
            undo_kept: new.undo_kept && new.risk == Risk::Undo,
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
        self.sent_as_newest(id, id, now);
    }

    /// A held message that was one part of `run` went out: the run stays (or becomes again) the
    /// last thing that happened.
    pub fn mark_sent_in_run(&mut self, id: u64, run: u64, now: Ms) {
        let newest = if self.get(run).is_some() { run } else { id };
        self.sent_as_newest(id, newest, now);
    }

    fn sent_as_newest(&mut self, id: u64, newest: u64, now: Ms) {
        if let Some(entry) = self.get_mut(id) {
            entry.state = EntryState::Sent;
            entry.at = now;
            entry.seeded = false;
        }
        if let Some(ix) = self.entries.iter().position(|e| e.id == newest) {
            let entry = self.entries.remove(ix);
            self.entries.push(entry);
        }
    }

    pub fn mark_unsent(&mut self, id: u64) {
        if let Some(entry) = self.get_mut(id) {
            entry.state = EntryState::Undone;
        }
    }

    /// What to remember for the next run of the app: the newest `keep` entries that settled,
    /// oldest first, each carrying how long ago it ran. Nothing remembered can be undone, and a
    /// message still waiting is left out: it never went.
    pub fn remembered(&self, now: Ms, keep: usize) -> Vec<Entry> {
        let settled: Vec<&Entry> = self
            .entries
            .iter()
            .filter(|e| e.state != EntryState::Holding)
            .collect();
        settled[settled.len().saturating_sub(keep)..]
            .iter()
            .map(|e| Entry {
                at: 0,
                before: e.age(now),
                undo_kept: false,
                held: None,
                ..(*e).clone()
            })
            .collect()
    }

    /// `id` is the last thing that happened again, without changing when it happened.
    pub fn bring_forward(&mut self, id: u64) {
        if let Some(ix) = self.entries.iter().position(|e| e.id == id) {
            let entry = self.entries.remove(ix);
            self.entries.push(entry);
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

    /// Undo `id` and record that it was undone. Returns the new entry's id. Nothing happens
    /// unless the host kept a way to take it back.
    pub fn undo(&mut self, now: Ms, id: u64) -> Option<u64> {
        let title = {
            let entry = self.get_mut(id)?;
            if !entry.can_undo() {
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
        assert!(l.undo(25, b).is_none(), "nobody kept a way to undo it");
        let c = l.record(
            26,
            NewEntry {
                title: "Moved standup".into(),
                risk: Risk::Undo,
                undo_kept: true,
                ..NewEntry::default()
            },
        );
        let b = c;
        let u = l.undo(30, b).unwrap();
        assert_eq!(l.get(b).unwrap().state, EntryState::Undone);
        assert_eq!(l.last().unwrap().id, u);
        assert_eq!(l.last().unwrap().title, "Undid · Moved standup");
        assert!(l.undo(40, b).is_none(), "undoing twice does nothing");
    }

    /// The next run of the app remembers what ran and how long ago, never a way to undo it, and
    /// never a message that hadn't gone yet.
    #[test]
    fn remembered_runs_keep_their_age_and_lose_their_undo() {
        let mut l = Ledger::new();
        l.record(
            1_000,
            NewEntry {
                title: "Moved standup".into(),
                risk: Risk::Undo,
                undo_kept: true,
                ..NewEntry::default()
            },
        );
        l.record(
            2_000,
            NewEntry {
                title: "To Sarah".into(),
                risk: Risk::Hold,
                held: Some(1),
                ..NewEntry::default()
            },
        );
        let kept = l.remembered(61_000, 50);
        assert_eq!(kept.len(), 1, "a message still waiting is left out");
        let mut next = Ledger::new();
        for entry in kept {
            next.seed(Entry { at: 5, ..entry });
        }
        let back = next.last().unwrap();
        assert_eq!(back.title, "Moved standup");
        assert_eq!(back.when_words(5), ">1m ago");
        assert_eq!(back.when_words(5 + 4 * 60_000), ">5m ago");
        assert!(!back.can_undo());
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
    fn a_message_a_run_sent_leaves_the_run_last() {
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
        let run = l.record(
            0,
            NewEntry {
                title: "Did 2 steps".into(),
                risk: Risk::Undo,
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
        l.mark_sent_in_run(m, run, 8_000);
        assert_eq!(l.get(m).unwrap().state, EntryState::Sent);
        assert_eq!(l.last().unwrap().id, run);
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
