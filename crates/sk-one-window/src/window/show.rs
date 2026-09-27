//! What the window shows: the line above the input, Enter's target, the snapshot.

use super::*;
use crate::contract::ledger_words;

impl OneWindow {
    /// The line above the input: a set line, what's staged, a message about to go, the last thing
    /// that ran (dimmed), or what's focused (dimmed). It is always there and always one height.
    /// A message a view held on its own is that view's to show until the view finishes with it.
    pub fn slot_line(&self, now: Ms) -> SlotLine {
        let _ = now;
        if let Some(line) = &self.line {
            return line.clone();
        }
        if let Some(line) = self.work_line() {
            return line;
        }
        if let Some(staged) = &self.staged {
            let s = &staged.subject;
            return SlotLine::new(SlotKind::Staged, s.label.clone(), s.title.clone())
                .with_icon(if s.icon.is_empty() {
                    "builtin".to_string()
                } else {
                    s.icon.clone()
                })
                .with_sub(s.sub.clone())
                .with_hint("esc", "let go");
        }
        let shown: Vec<&Held> = self
            .holds
            .live()
            .filter(|h| h.entry.is_some() || self.views.is_empty())
            .collect();
        if let Some(m) = shown.last() {
            let live = shown.len();
            let paused = self.holds.paused();
            let label = if paused {
                words::PAUSED.to_string()
            } else {
                self.waits_label(m)
            };
            let mut line = SlotLine::new(SlotKind::Holding, label, m.label.clone())
                .with_icon(self.waits_icon(m))
                .with_hint("⌘Z", "undo")
                .with_hint("↑", "more");
            if live > 1 {
                line = line.with_sub(format!("+ {} more", live - 1));
            }
            line.progress = Some(m.progress());
            return line;
        }
        if let Some(e) = self.ledger.last() {
            let mut parts: Vec<String> = Vec::new();
            let ran = e.held.is_some_and(|id| self.holds_a_run(id));
            match e.state {
                EntryState::Undone if ran => parts.push(ledger_words::NOT_RUN.into()),
                EntryState::Sent if ran => parts.push(ledger_words::RAN.into()),
                EntryState::Undone if e.held.is_some() => parts.push(words::NOT_SENT.into()),
                EntryState::Sent => parts.push(words::SENT.into()),
                _ => {}
            }
            if let Some(name) = &e.script {
                parts.push(format!("saved as “{name}”"));
            }
            let when = if e.seeded {
                e.when.clone().unwrap_or_default()
            } else {
                words::JUST_NOW.into()
            };
            if !when.is_empty() {
                parts.push(when);
            }
            let line = SlotLine::new(SlotKind::Recent, words::LAST, e.title.clone())
                .with_icon(e.icon.clone())
                .with_sub(parts.join(" · "));
            let line = if Self::undoable(e) {
                line.with_hint("⌘Z", ledger_words::UNDO)
            } else {
                line
            };
            return line.with_hint("↑", "");
        }
        let f = &self.focused;
        SlotLine::new(SlotKind::Focused, words::FOCUSED, f.title.clone())
            .with_icon(if f.icon.is_empty() {
                "window".to_string()
            } else {
                f.icon.clone()
            })
            .with_sub(f.sub.clone())
            .with_hint("⌘K", "act on it")
    }

    pub fn enter_target(&self, now: Ms) -> EnterTarget {
        match self.views.top() {
            Some(top) => top
                .enter
                .clone()
                .unwrap_or_else(|| EnterTarget::new(top.name.clone(), "")),
            None => self.list.target(now),
        }
    }

    pub fn snapshot(&self, now: Ms) -> Snapshot {
        let view = self.views.top().map(|t| t.name.clone());
        let slot = self.slot_line(now);
        Snapshot {
            slot_opacity: self.style_of(&slot).opacity,
            slot,
            header: if view.is_some() {
                String::new()
            } else {
                self.list.header()
            },
            crumb: self.views.crumb(),
            view,
            enter: self.enter_target(now),
            armed: self.guard.armed(now),
            items: self.list.layout(),
            sel: self.list.sel,
            text: self.text.clone(),
        }
    }

    /// How strongly the line above the input is drawn now. Context (what's focused, what ran last)
    /// reads as background and fainter while you type; in a view everything steps back except a
    /// message waiting to go.
    pub fn slot_style(&self, now: Ms) -> SlotStyle {
        self.style_of(&self.slot_line(now))
    }

    fn style_of(&self, slot: &SlotLine) -> SlotStyle {
        SlotStyle::new(slot.kind, !self.text.trim().is_empty(), self.in_view())
    }

    /// Whether Enter pressed now would be ignored.
    pub fn enter_armed(&self, now: Ms) -> bool {
        self.guard.armed(now)
    }

    /// How long the guard has left, for the ↵ button's fill.
    pub fn enter_armed_left(&self, now: Ms) -> Ms {
        self.guard
            .armed_until()
            .saturating_sub(now)
            .min(ENTER_GUARD_MS)
    }

    pub fn held_state(&self, id: u64) -> Option<HeldState> {
        self.holds.get(id).map(|h| h.state)
    }
}
