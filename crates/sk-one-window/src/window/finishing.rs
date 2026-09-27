//! What happens when something ran or finished: the ledger, holds, undo, learning, saved scripts.

use super::*;

impl OneWindow {
    pub(super) fn done(&mut self, now: Ms, done: Done, fx: &mut Vec<Effect>) {
        let said = self.said();
        let risk = risk_of(
            done.risk,
            done.form_kind.as_deref(),
            Some(done.icon.as_str()),
        );
        if done.icon != "change" {
            let holds_it = risk == Risk::Hold
                && self.cfg.hold_ms > 0
                && done.form_kind.as_deref() == Some("message");
            if holds_it {
                self.start_hold(now, done.title.clone(), said, Some(done.icon.clone()), fx);
            } else {
                let adopted = self.adopt_held(now);
                let id = self.ledger.record(
                    now,
                    NewEntry {
                        icon: done.icon.clone(),
                        title: done.title.clone(),
                        said: said.clone(),
                        risk,
                        ..NewEntry::default()
                    },
                );
                for held in adopted {
                    self.holds.set_run(held, id);
                }
                if self.cfg.auto_save
                    && matches!(done.icon.as_str(), "steps" | "delegate")
                    && !said.trim().is_empty()
                {
                    let steps = match &self.steps {
                        Some((sentence, steps)) if *sentence == said => steps.clone(),
                        _ => Vec::new(),
                    };
                    let name = script_name(&said, &steps);
                    self.ledger.set_script(id, name.clone());
                    fx.push(Effect::SaveScript {
                        entry: id,
                        name,
                        said,
                    });
                }
            }
        }
        if !done.stay {
            self.guard.arm(now);
            self.home(now, String::new(), fx);
        }
    }

    pub(super) fn start_hold(
        &mut self,
        now: Ms,
        label: String,
        request: String,
        icon: Option<String>,
        fx: &mut Vec<Effect>,
    ) {
        if self.cfg.hold_ms == 0 {
            fx.push(Effect::Held {
                op: HeldOp::Sent,
                id: 0,
                label,
                request,
            });
            return;
        }
        let id = self.holds.start(now, label.clone(), request.clone());
        if let Some(icon) = icon {
            let entry = self.ledger.record(
                now,
                NewEntry {
                    icon,
                    title: label.clone(),
                    said: request.clone(),
                    risk: Risk::Hold,
                    held: Some(id),
                    ..NewEntry::default()
                },
            );
            self.holds.set_entry(id, entry);
        }
        fx.push(Effect::Held {
            op: HeldOp::Start,
            id,
            label,
            request,
        });
    }

    /// Messages a view held on its own get their own line in the ledger, just before the run that
    /// sent them. Returns the messages adopted.
    pub(super) fn adopt_held(&mut self, now: Ms) -> Vec<u64> {
        let orphans: Vec<(u64, String, String)> = self
            .holds
            .live()
            .filter(|h| h.entry.is_none())
            .map(|h| (h.id, h.label.clone(), h.request.clone()))
            .collect();
        let mut adopted = Vec::with_capacity(orphans.len());
        for (id, label, request) in orphans {
            let entry = self.ledger.record(
                now,
                NewEntry {
                    icon: "message".into(),
                    title: label,
                    said: request,
                    risk: Risk::Hold,
                    held: Some(id),
                    ..NewEntry::default()
                },
            );
            self.holds.set_entry(id, entry);
            adopted.push(id);
        }
        adopted
    }

    pub(super) fn unsend(&mut self, now: Ms, ids: &[u64], fx: &mut Vec<Effect>) {
        let mut labels = Vec::new();
        let mut runs = 0;
        for id in ids {
            if let Some(h) = self.holds.unsend(now, *id) {
                if let Some(entry) = h.entry {
                    self.ledger.mark_unsent(entry);
                }
                fx.push(Effect::Held {
                    op: HeldOp::Unsent,
                    id: h.id,
                    label: h.label.clone(),
                    request: h.request,
                });
                runs += usize::from(self.holds_a_run(h.id));
                labels.push(h.label);
            }
        }
        if labels.is_empty() {
            return;
        }
        let title = if labels.len() == 1 {
            crate::contract::ledger_words::stopped(&labels[0], runs == 1)
        } else {
            format!("Didn't send {} messages", labels.len())
        };
        self.staged = None;
        self.done(
            now,
            Done {
                icon: "undo".into(),
                title,
                ..Done::default()
            },
            fx,
        );
    }

    pub(super) fn record_ran(&mut self, now: Ms, row: &Row, app: Option<String>, said: String) {
        self.learn_from(now, &said, row);
        self.ledger.record(
            now,
            NewEntry {
                icon: row.icon.clone().unwrap_or_else(|| "check".into()),
                title: row.title.clone(),
                said,
                risk: Risk::None,
                app,
                ..NewEntry::default()
            },
        );
    }
}
