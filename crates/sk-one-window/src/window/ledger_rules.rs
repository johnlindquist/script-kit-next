//! Notices, irreversible runs, and ⌘Z on the last run.

use super::*;
use crate::contract::ledger_words;
use crate::notice::{Irreversible, Notice};

impl OneWindow {
    /// A notice goes in the ledger; the line says it unless something more pressing holds it.
    pub(super) fn on_notice(&mut self, now: Ms, notice: Notice) {
        let said = self.said();
        self.ledger.record(
            now,
            NewEntry {
                icon: notice.icon,
                title: notice.title,
                said,
                risk: Risk::None,
                ..NewEntry::default()
            },
        );
    }

    /// Something that can't be taken back waits in the line; `Effect::Held` `Sent` runs it.
    pub(super) fn on_irreversible(&mut self, now: Ms, run: Irreversible, fx: &mut Vec<Effect>) {
        let request = self.said();
        let before = self.holds.newest().map(|h| h.id);
        self.start_hold(now, run.title, request, Some(run.icon), fx);
        match self.holds.newest().map(|h| h.id) {
            Some(id) if Some(id) != before => self.runs_held.push(id),
            _ => {}
        }
        self.guard.arm(now);
        self.home(now, String::new(), fx);
    }

    /// Whether the waiting `id` is a run rather than a message.
    pub(super) fn holds_a_run(&self, id: u64) -> bool {
        self.runs_held.contains(&id)
    }

    /// The words for a waiting thing's countdown.
    pub(super) fn waits_label(&self, held: &Held) -> String {
        if self.holds_a_run(held.id) {
            ledger_words::runs_in(held.seconds_left())
        } else {
            words::sending_in(held.seconds_left())
        }
    }

    /// ⌘Z with nothing waiting undoes the last run, if it can be undone.
    pub(super) fn undo_last(&mut self, now: Ms, fx: &mut Vec<Effect>) -> bool {
        match self.ledger.last().cloned() {
            Some(e) if Self::undoable(&e) => {
                self.staged = None;
                self.entry_act(now, RowAct::Undo, &e, fx);
                true
            }
            _ => false,
        }
    }

    pub(super) fn undoable(e: &Entry) -> bool {
        e.risk == Risk::Undo && e.state == EntryState::Done
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::{ENTER_GUARD_MS, MESSAGE_HOLD_MS, ledger_words, words};
    use crate::history;
    use crate::notice::{Irreversible, Notice};
    use crate::stage::entry_subject;
    use crate::window::{Done, Effect, Event, HeldOp, Key, OneWindow, ViewOp};
    use crate::{Risk, Row, SlotKind};

    fn held_ops(fx: &[Effect]) -> Vec<HeldOp> {
        fx.iter()
            .filter_map(|e| match e {
                Effect::Held { op, .. } => Some(*op),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_notice_is_the_last_line_now_and_after_the_window_comes_back() {
        let mut w = OneWindow::default();
        w.handle(0, Event::Notice(Notice::new("check", "Copied")));
        let line = w.slot_line(0);
        assert_eq!(
            (line.kind, line.title.as_str()),
            (SlotKind::Recent, "Copied")
        );
        assert!(line.sub.contains(words::JUST_NOW));

        w.handle(10, Event::Dismissed);
        w.handle(20, Event::Notice(Notice::new("info", "Backup finished")));
        w.handle(9_000, Event::Summon);
        let line = w.slot_line(9_000);
        assert_eq!(line.label, words::LAST);
        assert_eq!(line.title, "Backup finished");
    }

    #[test]
    fn a_notice_never_covers_a_waiting_run() {
        let mut w = OneWindow::default();
        w.handle(
            0,
            Event::Irreversible(Irreversible::new("trash", "Empty Trash")),
        );
        w.handle(100, Event::Notice(Notice::new("check", "Copied")));
        let line = w.slot_line(100);
        assert_eq!(line.kind, SlotKind::Holding);
        assert_eq!(line.title, "Empty Trash");
        assert_eq!(line.label, ledger_words::runs_in(8));
    }

    #[test]
    fn an_irreversible_run_waits_then_goes_once() {
        let mut w = OneWindow::default();
        let out = w.handle(
            0,
            Event::Irreversible(Irreversible::new("trash", "Empty Trash")),
        );
        assert_eq!(held_ops(&out.effects), vec![HeldOp::Start]);
        let out = w.handle(MESSAGE_HOLD_MS - 1, Event::Tick);
        assert!(held_ops(&out.effects).is_empty());
        let out = w.handle(MESSAGE_HOLD_MS, Event::Tick);
        assert_eq!(held_ops(&out.effects), vec![HeldOp::Sent]);
        let line = w.slot_line(MESSAGE_HOLD_MS);
        assert_eq!(line.title, "Empty Trash");
        assert!(line.sub.contains(ledger_words::RAN), "{:?}", line.sub);
        let out = w.handle(MESSAGE_HOLD_MS * 3, Event::Tick);
        assert!(held_ops(&out.effects).is_empty());
    }

    #[test]
    fn cmd_z_stops_an_irreversible_run_and_says_it_didnt_run() {
        let mut w = OneWindow::default();
        w.handle(
            0,
            Event::Irreversible(Irreversible::new("trash", "Empty Trash")),
        );
        let out = w.handle(ENTER_GUARD_MS, Event::Key(Key::CmdZ));
        assert_eq!(held_ops(&out.effects), vec![HeldOp::Unsent]);
        assert_eq!(w.slot_line(ENTER_GUARD_MS).title, "Didn't run: Empty Trash");
        let out = w.handle(MESSAGE_HOLD_MS * 2, Event::Tick);
        assert!(
            held_ops(&out.effects).is_empty(),
            "a stopped run never goes"
        );
    }

    #[test]
    fn typing_pauses_a_waiting_run() {
        let mut w = OneWindow::default();
        w.handle(
            0,
            Event::Irreversible(Irreversible::new("trash", "Empty Trash")),
        );
        w.handle(1_000, Event::Text("x".into()));
        assert_eq!(w.slot_line(1_000).label, words::PAUSED);
        let out = w.handle(30_000, Event::Tick);
        assert!(held_ops(&out.effects).is_empty());
    }

    #[test]
    fn cmd_z_undoes_the_last_run_that_can_be_undone() {
        let mut w = OneWindow::default();
        w.handle(
            0,
            Event::Done(Done {
                icon: "trash".into(),
                title: "Moved Notes.md to Trash".into(),
                risk: Some(Risk::Undo),
                ..Done::default()
            }),
        );
        let line = w.slot_line(0);
        assert_eq!(
            line.hints[0],
            ("⌘Z".to_string(), ledger_words::UNDO.to_string())
        );
        let out = w.handle(ENTER_GUARD_MS, Event::Key(Key::CmdZ));
        let undone: Vec<u64> = out
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Undo { entry } => Some(*entry),
                _ => None,
            })
            .collect();
        assert_eq!(undone, vec![1]);
        assert_eq!(
            w.slot_line(ENTER_GUARD_MS).title,
            "Undid · Moved Notes.md to Trash"
        );
        let out = w.handle(ENTER_GUARD_MS * 3, Event::Key(Key::CmdZ));
        assert!(
            !out.effects.iter().any(|e| matches!(e, Effect::Undo { .. })),
            "undoing twice does nothing"
        );
    }

    #[test]
    fn enter_in_the_history_view_acts_on_the_entry() {
        let mut w = OneWindow::default();
        w.handle(
            0,
            Event::Ran {
                row: Row::new("s:a", "Open Linear"),
                app: None,
            },
        );
        w.handle(1_000, Event::View(ViewOp::Open(history::view_spec())));
        assert_eq!(w.enter_target(1_000).label, ledger_words::ACT_ON_IT);
        w.handle(1_010, Event::Text("lin".into()));
        assert_eq!(
            w.views().top().map(|v| v.name.as_str()),
            Some(history::VIEW)
        );
        let rows = history::history_rows(&w.ledger().search("lin"));
        assert_eq!(rows[0].title, "Open Linear");
        let entry = w
            .ledger()
            .get(history::entry_id(&rows[0].key).unwrap())
            .unwrap()
            .clone();
        let out = w.handle(1_020, Event::Stage(entry_subject(&entry, words::ACTING_ON)));
        assert!(!out.effects.iter().any(|e| matches!(e, Effect::Run { .. })));
        let line = w.slot_line(1_020);
        assert_eq!(
            (line.kind, line.title.as_str()),
            (SlotKind::Staged, "Open Linear")
        );
        assert!(!w.in_view());
        assert!(w.text().is_empty());
        assert_eq!(w.snapshot(1_020).header, words::STAGED);
    }
}
