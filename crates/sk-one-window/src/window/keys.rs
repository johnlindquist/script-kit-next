//! What each key does, in the list, in a view and while something is staged.

use super::*;

impl OneWindow {
    pub(super) fn on_key(&mut self, now: Ms, key: Key, fx: &mut Vec<Effect>) -> bool {
        if self.talk.is_some() {
            return self.talk_key(now, key, fx);
        }
        let armed = self.guard.armed(now);
        if self.in_view() {
            return match key {
                Key::Enter if armed => {
                    fx.push(Effect::EnterIgnored);
                    true
                }
                Key::Escape => {
                    self.on_view(now, ViewOp::Back, fx);
                    true
                }
                key => {
                    fx.push(Effect::ViewKey(key));
                    true
                }
            };
        }
        if self.away {
            return false;
        }
        if key == Key::Enter && armed {
            fx.push(Effect::EnterIgnored);
            return true;
        }
        let empty = self.text.is_empty();
        match &key {
            Key::CmdZ if empty => {
                if let Some(id) = self.holds.newest().map(|h| h.id) {
                    self.staged = None;
                    self.unsend(now, &[id], fx);
                    return true;
                }
            }
            Key::Up if empty && self.staged.is_none() && self.list.sel == 0 => {
                let target = self
                    .holds
                    .newest()
                    .and_then(|h| h.entry)
                    .or_else(|| self.ledger.last().map(|e| e.id));
                if let Some(id) = target {
                    let entry = self.ledger.get(id).expect("entry").clone();
                    let label = if entry.state == EntryState::Holding {
                        words::SENDING
                    } else {
                        words::LAST
                    };
                    self.stage(now, entry_subject(&entry, label), None, words::STAGED, fx);
                    return true;
                }
            }
            Key::CmdEnter if !self.text.trim().is_empty() => {
                return self.on_key(now, Key::CmdDigit(1), fx);
            }
            Key::CmdK => {
                let row = if self.text.trim().is_empty() {
                    None
                } else {
                    self.list.current_row(now)
                };
                let subject = row
                    .as_ref()
                    .map(row_subject)
                    .unwrap_or_else(|| self.focused.clone());
                self.stage(now, subject, None, words::STAGED, fx);
                return true;
            }
            Key::Escape if self.staged.is_some() => {
                self.staged = None;
                self.home(now, String::new(), fx);
                return true;
            }
            Key::CmdDigit(d) => {
                let i = (*d as usize).saturating_sub(1);
                if let Some(pos) = self.list.slot_position(i) {
                    self.list.sel = pos;
                    self.activate(now, fx);
                }
                return true;
            }
            _ => {}
        }
        let cancels = self.list.held.is_some() && matches!(key, Key::Up | Key::Down | Key::Escape);
        if cancels {
            self.list.held = None;
            self.list.asking = false;
        }
        match key {
            Key::Down => self.list.move_sel(1),
            Key::Up => self.list.move_sel(-1),
            Key::Enter => self.activate(now, fx),
            key if !cancels => {
                fx.push(Effect::PassKey(key));
                return false;
            }
            _ => {}
        }
        true
    }

    /// The highlight moves to the row at `pos` among the rows, or the last one. It is the user's
    /// own change, so Enter isn't guarded; a different row drops an Enter waiting on Jev.
    pub(super) fn on_select(&mut self, pos: usize) -> bool {
        if self.in_view() || self.list.visible().is_empty() {
            return false;
        }
        if pos != self.list.sel {
            self.list.held = None;
            self.list.asking = false;
        }
        self.list.sel = 0;
        self.list.move_sel(i32::try_from(pos).unwrap_or(i32::MAX));
        true
    }

    pub(super) fn activate(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let Some(item) = self.list.current() else {
            return;
        };
        match item {
            Item::Slot(i) => match self.list.effective(i, now).cloned() {
                Some(row) => {
                    self.list.asking = false;
                    self.run(now, row, RunHow::Slot, fx);
                }
                None => self.list.held = Some(i),
            },
            item => {
                if let Some(row) = self.list.row(&item).cloned() {
                    self.run(now, row, RunHow::Match, fx);
                }
            }
        }
    }

    pub(super) fn run(&mut self, now: Ms, row: Row, how: RunHow, fx: &mut Vec<Effect>) {
        self.picked = Some((row.clone(), how));
        let staged_entry = self
            .staged
            .as_ref()
            .and_then(|s| s.subject.entry)
            .or_else(|| self.share_last(&row));
        let from_staged = self
            .staged
            .as_ref()
            .is_some_and(|s| s.rows.iter().any(|r| r.key == row.key));
        if from_staged && row.act != RowAct::Share {
            self.staged = None;
            self.refresh_rows();
        }
        if row.takes_back_last() && self.take_back(now, row.target.as_deref(), fx) {
            return;
        }
        match row.act {
            RowAct::Host => fx.push(Effect::Run { row, how }),
            RowAct::AskJevAgain => {
                self.list.jev.reset();
                self.list.sel = 0;
                self.jev_due = None;
                fx.push(Effect::AskJev {
                    text: self.text.clone(),
                });
            }
            act => {
                let Some(entry) = staged_entry.and_then(|id| self.ledger.get(id).cloned()) else {
                    fx.push(Effect::Run { row, how });
                    return;
                };
                self.entry_act(now, act, &entry, fx);
            }
        }
    }

    /// A reading that takes something back: it stops the waiting messages it names, or else undoes
    /// the last thing that ran if that can be undone. Returns whether it did either.
    fn take_back(&mut self, now: Ms, target: Option<&str>, fx: &mut Vec<Effect>) -> bool {
        let ids: Vec<u64> = self
            .holds
            .live()
            .filter(|h| target.is_some_and(|t| t == "all" || t == h.label))
            .map(|h| h.id)
            .collect();
        if !ids.is_empty() {
            self.staged = None;
            self.unsend(now, &ids, fx);
            return true;
        }
        match self.ledger.last().cloned() {
            Some(e) if e.risk == Risk::Undo && e.state == EntryState::Done => {
                self.staged = None;
                self.entry_act(now, RowAct::Undo, &e, fx);
                true
            }
            _ => false,
        }
    }

    pub(super) fn entry_act(&mut self, now: Ms, act: RowAct, entry: &Entry, fx: &mut Vec<Effect>) {
        match act {
            RowAct::Unsend => {
                if let Some(id) = entry.held {
                    self.unsend(now, &[id], fx);
                }
            }
            RowAct::SendNow => {
                if let Some(h) = entry.held.and_then(|id| self.holds.send_now(now, id)) {
                    self.sent(now, h.id, fx);
                }
                self.home(now, String::new(), fx);
            }
            RowAct::Undo => {
                if self.ledger.undo(now, entry.id).is_some() {
                    fx.push(Effect::Undo { entry: entry.id });
                }
                self.home(now, String::new(), fx);
            }
            RowAct::Again => {
                if entry.held.is_some() || entry.risk == Risk::Hold {
                    self.start_hold(
                        now,
                        entry.title.clone(),
                        entry.said.clone(),
                        Some(entry.icon.clone()),
                        fx,
                    );
                    fx.push(Effect::Again { entry: entry.id });
                    self.home(now, String::new(), fx);
                    return;
                }
                fx.push(Effect::Again { entry: entry.id });
                let row = Row::new(format!("again:{}", entry.id), entry.title.clone())
                    .with_icon(entry.icon.clone());
                match entry.app.clone().filter(|_| self.cfg.dismiss_on_launch) {
                    Some(app) => {
                        self.away = true;
                        fx.push(Effect::Hide {
                            app: Some(app.clone()),
                        });
                        self.dismiss = Some(PendingDismiss {
                            at: now + DISMISS_SETTLE_MS,
                            row,
                            app: Some(app),
                            said: entry.said.clone(),
                        });
                    }
                    None => {
                        let id = self.ledger.record(
                            now,
                            NewEntry {
                                icon: entry.icon.clone(),
                                title: entry.title.clone(),
                                said: entry.said.clone(),
                                risk: entry.risk,
                                ..NewEntry::default()
                            },
                        );
                        if let Some(name) = entry.script.clone() {
                            self.ledger.set_script(id, name);
                        }
                        self.home(now, String::new(), fx);
                    }
                }
            }
            RowAct::OpenScript => {
                if let Some(name) = entry.script.clone() {
                    fx.push(Effect::OpenScript { name });
                }
            }
            RowAct::Share => {
                let subject = Subject {
                    label: words::SHARING.into(),
                    ..entry_subject(entry, words::SHARING)
                };
                self.stage(now, subject, Some(share_rows()), words::SHARE_HOW, fx);
            }
            RowAct::Host | RowAct::AskJevAgain => {}
        }
    }
}
