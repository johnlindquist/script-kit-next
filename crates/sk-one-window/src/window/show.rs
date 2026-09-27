//! What the window shows: the line above the input, Enter's target, the snapshot.

use super::*;

impl OneWindow {
    /// The line above the input: a set line, what's staged, a message about to go, the last thing
    /// that ran (dimmed), or what's focused (dimmed). It is always there and always one height.
    /// A message a view held on its own is that view's to show until the view finishes with it.
    pub fn slot_line(&self, now: Ms) -> SlotLine {
        let _ = now;
        if let Some(line) = &self.line {
            return line.clone();
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
                words::sending_in(m.seconds_left())
            };
            let mut line = SlotLine::new(SlotKind::Holding, label, m.label.clone())
                .with_icon("message")
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
            match e.state {
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
            return SlotLine::new(SlotKind::Recent, words::LAST, e.title.clone())
                .with_icon(e.icon.clone())
                .with_sub(parts.join(" · "))
                .with_hint("↑", "");
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
            slot_opacity: self.slot_opacity(&slot),
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

    /// Context (what's focused, what ran last) reads as background and fainter while you type; in a
    /// view everything steps back except a message waiting to go.
    pub(super) fn slot_opacity(&self, slot: &SlotLine) -> f32 {
        use crate::contract::slot_opacity as o;
        let typing = !self.text.trim().is_empty();
        match slot.kind {
            SlotKind::Holding => 1.0,
            _ if !self.views.is_empty() => o::IN_VIEW_NOT_HOLDING,
            SlotKind::Focused if typing => o::TYPING_FOCUSED,
            SlotKind::Focused => o::FOCUSED,
            SlotKind::Recent if typing => o::TYPING_RECENT,
            SlotKind::Recent => o::RECENT,
            SlotKind::Staged | SlotKind::Typing => 1.0,
        }
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
