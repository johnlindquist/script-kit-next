//! "What did I run today": the ledger as the rows of a view under the same input.
//!
//! Each row is one entry, newest first. Enter on one puts it in the line above the input
//! ([`crate::window::Event::Stage`]) and the list becomes what you can do with it again.

use crate::contract::{ledger_words, words};
use crate::enter::EnterTarget;
use crate::ledger::{Entry, EntryState};
use crate::row::Row;
use crate::window::ViewSpec;

const KEY_PREFIX: &str = "entry:";

/// The view's name and the crumb it shows.
pub const VIEW: &str = "history";

/// Opens the history view: it takes the input, and its words narrow the rows.
pub fn view_spec() -> ViewSpec {
    ViewSpec {
        name: VIEW.to_string(),
        label: ledger_words::TODAY.to_string(),
        enter: Some(EnterTarget::new(VIEW, ledger_words::ACT_ON_IT)),
        takes_input: true,
        text: Some(String::new()),
        leave_text: Some(String::new()),
    }
}

/// Entries as rows, newest first; the first carries the view's header.
pub fn history_rows(entries: &[&Entry]) -> Vec<Row> {
    entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut row = Row::new(format!("{KEY_PREFIX}{}", e.id), e.title.clone())
                .with_icon(if e.icon.is_empty() { "check" } else { &e.icon })
                .with_sub(when(e))
                .with_enter_label(ledger_words::ACT_ON_IT);
            if i == 0 {
                row.head = Some(ledger_words::HISTORY_HEAD.to_string());
            }
            row
        })
        .collect()
}

/// The entry a history row stands for.
pub fn entry_id(key: &str) -> Option<u64> {
    key.strip_prefix(KEY_PREFIX)?.parse().ok()
}

fn when(e: &Entry) -> String {
    let at = if e.seeded {
        e.when.clone().unwrap_or_default()
    } else {
        words::JUST_NOW.to_string()
    };
    let state = match e.state {
        EntryState::Holding => words::SENDING,
        EntryState::Undone if e.held.is_some() => words::NOT_SENT,
        EntryState::Sent => words::SENT,
        EntryState::Done | EntryState::Undone => "",
    };
    [state, at.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{Ledger, NewEntry};

    #[test]
    fn rows_are_newest_first_under_one_header() {
        let mut l = Ledger::new();
        for title in ["Open Linear", "Moved standup"] {
            l.record(
                0,
                NewEntry {
                    title: title.into(),
                    ..NewEntry::default()
                },
            );
        }
        let rows = history_rows(&l.since(0));
        assert_eq!(rows[0].title, "Moved standup");
        assert_eq!(rows[0].head.as_deref(), Some(ledger_words::HISTORY_HEAD));
        assert!(rows[1].head.is_none());
        assert_eq!(entry_id(&rows[1].key), Some(1));
        assert_eq!(rows[0].sub.as_deref(), Some(words::JUST_NOW));
    }
}
