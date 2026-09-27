//! The clock: timers, held messages counting down, Jev's debounce firing, dismiss settling.

use super::*;

impl OneWindow {
    pub(super) fn tick(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        for change in self.holds.advance(now) {
            if let HeldChange::Sent { id } = change {
                self.sent(now, id, fx);
            }
        }
        if let Some(d) = self.height.tick(now) {
            fx.push(Effect::Height(d));
        }
        if self.redraws.first().is_some_and(|at| *at <= now) {
            self.redraws.retain(|at| *at > now);
            self.list_redrawn(now, fx);
        }
        if let Some((at, text)) = self.jev_due.clone() {
            if now >= at {
                self.jev_due = None;
                if text == self.text {
                    fx.push(Effect::AskJev { text });
                }
            }
        }
        if self.dismiss.as_ref().is_some_and(|d| now >= d.at) {
            let d = self.dismiss.take().expect("dismiss");
            self.staged = None;
            self.record_ran(now, &d.row, d.app, d.said);
            self.home(now, String::new(), fx);
        }
    }

    /// The list drew again: the height may start its pause over or stop holding.
    pub(super) fn list_redrawn(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        if self.in_view() {
            return;
        }
        if let Some(d) = self.height.redrawn(now) {
            fx.push(Effect::Height(d));
        }
    }

    pub(super) fn sent(&mut self, now: Ms, id: u64, fx: &mut Vec<Effect>) {
        let Some(h) = self.holds.get(id).cloned() else {
            return;
        };
        match (h.entry, h.run) {
            (Some(entry), Some(run)) => self.ledger.mark_sent_in_run(entry, run, now),
            (Some(entry), None) => self.ledger.mark_sent(entry, now),
            _ => {}
        }
        fx.push(Effect::Held {
            op: HeldOp::Sent,
            id,
            label: h.label,
            request: h.request,
        });
    }

    pub(super) fn sync_pause(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let paused = !self.in_view() && !self.text.trim().is_empty();
        for change in self.holds.set_paused(now, paused) {
            let (op, id) = match change {
                HeldChange::Sent { id } => {
                    self.sent(now, id, fx);
                    continue;
                }
                HeldChange::Paused { id } => (HeldOp::Paused, id),
                HeldChange::Resumed { id } => (HeldOp::Resumed, id),
            };
            if let Some(h) = self.holds.get(id) {
                fx.push(Effect::Held {
                    op,
                    id,
                    label: h.label.clone(),
                    request: h.request.clone(),
                });
            }
        }
    }

    pub(super) fn schedule(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let armed_end = self.guard.armed(now).then(|| self.guard.armed_until());
        let next = [
            self.holds.next_wake(now),
            self.height.next_wake(),
            self.list.jev.next_wake(now),
            armed_end,
            self.jev_due.as_ref().map(|(at, _)| *at),
            self.redraws.first().copied(),
            self.dismiss.as_ref().map(|d| d.at),
            self.list.reading_wake(now),
        ]
        .into_iter()
        .flatten()
        .filter(|t| *t > now)
        .min();
        if next != self.wake {
            self.wake = next;
            if let Some(t) = next {
                fx.push(Effect::WakeAt(t));
            }
        }
    }

    /// The next time the host must send `Tick`.
    pub fn next_wake(&self) -> Option<Ms> {
        self.wake
    }
}
