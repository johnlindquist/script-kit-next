//! The clock: timers, held messages counting down, Jev's debounce firing, dismiss settling.

use super::*;

impl OneWindow {
    pub(super) fn tick(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        for change in self.holds.advance(now) {
            if let HeldChange::Sent { id } = change {
                self.sent(now, id, fx);
            }
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
        if !self.user_turn(now) {
            self.talk_decide(now, fx);
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
            self.list.jev.next_wake(now),
            armed_end,
            self.jev_due.as_ref().map(|(at, _)| *at),
            self.dismiss.as_ref().map(|d| d.at),
            self.list.reading_wake(now),
            self.slot_ages_at(now),
            // Ages in the visible tab bar, including the copy expiry boundary.
            self.tabs_visible()
                .then(|| {
                    self.subject_pages()
                        .iter()
                        .filter(|p| p.source.subject.kind != SubjectKind::Thing)
                        .map(|p| {
                            let age = p.age(now);
                            let unit = match age {
                                0..60_000 => 1000,
                                60_000..3_600_000 => 60_000,
                                3_600_000..86_400_000 => 3_600_000,
                                _ => 86_400_000,
                            };
                            now + unit - age % unit
                        })
                        .min()
                })
                .flatten(),
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
