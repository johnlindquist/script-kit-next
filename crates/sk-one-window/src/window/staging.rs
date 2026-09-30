//! Putting a row or the focused thing in the line (⌘K, ↑) and what the list offers for it.

use super::*;

impl OneWindow {
    /// Over a staged subject's actions. Once it is a chip the input's placeholder asks for the
    /// words, so the list only offers the other way.
    pub(super) fn staged_head(&self, head: &str) -> String {
        if self.cfg.subject_chip && head == words::STAGED {
            words::OR_PICK_ONE.to_string()
        } else {
            head.to_string()
        }
    }

    pub(super) fn stage(
        &mut self,
        now: Ms,
        subject: Subject,
        rows: Option<Vec<Row>>,
        head: &str,
        fx: &mut Vec<Effect>,
    ) {
        let entry = subject.entry.and_then(|id| self.ledger.get(id)).cloned();
        let (rows, waiting) = match (rows, &entry) {
            (Some(rows), _) => (rows, false),
            (None, Some(e)) => (entry_rows(e), false),
            (None, None) => {
                fx.push(Effect::ActionsFor(subject.clone()));
                (default_actions(&subject), false)
            }
        };
        self.staged = Some(Staged {
            subject,
            rows,
            head: self.staged_head(head),
            waiting_for_rows: waiting,
            entry_state: entry.map(|e| e.state),
        });
        let user = self.user_turn(now);
        self.guard.arm_unless_user(now, user);
        self.home(now, String::new(), fx);
    }

    /// A staged entry whose state changed (a message went out) gets its new rows.
    pub(super) fn sync_staged(&mut self, now: Ms) {
        let Some(staged) = self.staged.as_ref() else {
            return;
        };
        let Some(entry) = staged
            .subject
            .entry
            .and_then(|id| self.ledger.get(id))
            .cloned()
        else {
            return;
        };
        if staged.entry_state == Some(entry.state) {
            return;
        }
        let label = staged.subject.label.clone();
        let share = label == words::SHARING;
        let staged = self.staged.as_mut().expect("staged");
        staged.entry_state = Some(entry.state);
        if !share {
            staged.rows = entry_rows(&entry);
            staged.subject = entry_subject(&entry, &label);
        }
        let user = self.user_turn(now);
        self.guard.arm_unless_user(now, user);
        if !self.in_view() {
            self.refresh_rows();
        }
    }
}
