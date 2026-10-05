//! What the window shows: the line above the input, Enter's target, the snapshot.

use super::*;
use crate::contract::ledger_words;

impl OneWindow {
    /// The line above the input only reports: a message waiting to go, a question waiting for its
    /// answer, what a view or a notice says, the model's chat, or the last thing that ran. What
    /// you act on is the input's chip ([`OneWindow::chip`]). It is always there, always one height
    /// and never empty: before anything has run it names what's behind the window. Without
    /// `subject_chip` the line also names what's staged or focused.
    ///
    /// Its key hints are only the keys that do what they say right now: ⌘Z while ⌘Z takes
    /// something back, ↑ while ↑ stages the last run.
    pub fn slot_line(&self, now: Ms) -> SlotLine {
        if !self.cfg.subject_chip {
            return self.one_line(now);
        }
        let mut line = self.chip_line(now);
        let take_back = self.can_take_back();
        let up = self.text.is_empty()
            && self.staged.is_none()
            && !self.in_view()
            && self.list.sel == 0
            && self.last_target().is_some();
        line.hints.retain(|(key, _)| match key.as_str() {
            "⌘Z" => take_back,
            "↑" => up,
            _ => true,
        });
        line
    }

    fn chip_line(&self, now: Ms) -> SlotLine {
        if let Some(line) = self.holding_line() {
            let staged = self
                .staged
                .as_ref()
                .is_some_and(|s| s.subject.entry.is_some());
            return line.lit(staged);
        }
        if let Some(line) = self.question_line() {
            return line;
        }
        if let Some(line) = self.line.as_ref().filter(|l| !Self::recent_kind(l.kind)) {
            return line.clone();
        }
        if let Some(line) = self.work_line() {
            return line;
        }
        self.recent_line(now).unwrap_or_else(|| self.behind_line())
    }

    /// A question waiting for its answer (its answers are the rows): what it asks and why, lit
    /// until it is answered or let go.
    fn question_line(&self) -> Option<SlotLine> {
        let staged = self.staged.as_ref()?;
        (staged.subject.kind == SubjectKind::Question)
            .then(|| Self::staged_line(&staged.subject).lit(true))
    }

    /// What's behind the window: its title, and the app it's in when that says more. With nothing
    /// in front, the desktop.
    fn behind_line(&self) -> SlotLine {
        let f = &self.focused;
        if f.title.is_empty() {
            return SlotLine::new(SlotKind::Focused, words::BEHIND, words::THE_DESKTOP)
                .with_icon("window");
        }
        let sub = match f.kind {
            SubjectKind::Window if !f.sub.is_empty() && f.sub != f.title => {
                format!("in {}", f.sub)
            }
            SubjectKind::Window => String::new(),
            _ => f.sub.clone(),
        };
        SlotLine::new(SlotKind::Focused, words::BEHIND, f.title.clone())
            .with_icon(Self::focused_icon(f))
            .with_sub(sub)
    }

    /// What just happened: voice landing in an app, the newest message waiting to go, or the
    /// last thing that ran; lit while ↑ has it staged. While something else is staged the line
    /// keeps what it showed then, so a notice never covers it (it gets the line once let go).
    /// Nothing before anything happened, and nothing without `subject_chip`.
    pub fn recent_line(&self, now: Ms) -> Option<SlotLine> {
        if !self.cfg.subject_chip {
            return None;
        }
        if let Some(line) = self.line.as_ref().filter(|l| Self::recent_kind(l.kind)) {
            return Some(line.clone());
        }
        let staged = self.staged.as_ref().and_then(|s| s.subject.entry);
        if let Some(line) = self.holding_line() {
            return Some(line.lit(staged.is_some()));
        }
        let e = self.shown_entry()?;
        let line = self.ran_line(e, now);
        Some(if staged.is_some() {
            line.lit(true)
        } else if line.hints.is_empty() {
            line.with_hint("↑", "")
        } else {
            line
        })
    }

    /// When the last run's age words change on their own ("just now" → ">1m ago"), while the line
    /// shows them. A waiting message's countdown wakes through its hold.
    pub(super) fn slot_ages_at(&self, now: Ms) -> Option<Ms> {
        if self.slot_line(now).kind != SlotKind::Recent {
            return None;
        }
        let e = self.shown_entry()?;
        if e.seeded && e.when.is_some() {
            return None;
        }
        Some(now + words::ago_changes_in(e.age(now)))
    }

    /// The ledger entry the line names: the one ↑ staged, the one shown when something else was
    /// staged, or the last one.
    fn shown_entry(&self) -> Option<&crate::ledger::Entry> {
        match &self.staged {
            Some(staged) => match staged.subject.entry {
                Some(id) => self.ledger.get(id),
                None => staged.shown.and_then(|id| self.ledger.get(id)),
            },
            None => self.ledger.last(),
        }
    }

    /// What the input asks for, when it differs from the launcher's own: the words for an
    /// attached chip, or a reply in the model's chat. Only with `subject_chip`.
    pub fn placeholder(&self) -> Option<&'static str> {
        if !self.cfg.subject_chip {
            return None;
        }
        if self.staged.is_some() {
            return Some(words::SAY_WHAT_TO_DO);
        }
        self.views
            .top()
            .is_some_and(|f| f.name == crate::chat::VIEW)
            .then_some(words::REPLY_TO_THE_MODEL)
    }

    /// The subject as a chip in the input. While the input is empty and nothing is staged, what's
    /// focused is offered at the input's end with the key that attaches it; once Space, ⌘K or ↑
    /// stages something, it sits at the input's start with the key that lets go, in a view too.
    /// Only the list offers one, and only with `subject_chip`.
    pub fn chip(&self) -> Option<SubjectChip> {
        if !self.cfg.subject_chip {
            return None;
        }
        if let Some(staged) = &self.staged {
            return Some(SubjectChip::of(&staged.subject, true));
        }
        if !self.views.is_empty() {
            return None;
        }
        (self.text.is_empty() && !self.focused.title.is_empty()).then(|| {
            let mut chip = SubjectChip::of(&self.focused, false);
            if self.lessons.teaches_space() {
                chip.word = words::USE.to_string();
            }
            chip
        })
    }

    /// Everything in one line, as the prototype drew it: what's staged, a message about to go,
    /// the last thing that ran (dimmed), or what's focused (dimmed).
    fn one_line(&self, now: Ms) -> SlotLine {
        if let Some(line) = &self.line {
            return line.clone();
        }
        if let Some(line) = self.work_line() {
            return line;
        }
        if let Some(staged) = &self.staged {
            return Self::staged_line(&staged.subject).with_hint("esc", "let go");
        }
        if let Some(line) = self.holding_line() {
            return line;
        }
        if let Some(e) = self.ledger.last() {
            return self.ran_line(e, now).with_hint("↑", "");
        }
        let f = &self.focused;
        SlotLine::new(SlotKind::Focused, words::FOCUSED, f.title.clone())
            .with_icon(Self::focused_icon(f))
            .with_sub(f.sub.clone())
            .with_hint("⌘K", "act on it")
    }

    /// Lines about what just happened rather than what's in front of you.
    fn recent_kind(kind: SlotKind) -> bool {
        matches!(
            kind,
            SlotKind::Recent | SlotKind::Holding | SlotKind::Typing
        )
    }

    fn focused_icon(f: &Subject) -> String {
        if f.icon.is_empty() {
            "window".to_string()
        } else {
            f.icon.clone()
        }
    }

    fn staged_line(s: &Subject) -> SlotLine {
        SlotLine::new(SlotKind::Staged, s.label.clone(), s.title.clone())
            .with_icon(if s.icon.is_empty() {
                "builtin".to_string()
            } else {
                s.icon.clone()
            })
            .with_sub(s.sub.clone())
    }

    /// The newest message waiting to go. A message a view held on its own is that view's to show
    /// until the view finishes with it.
    fn holding_line(&self) -> Option<SlotLine> {
        let shown: Vec<&Held> = self
            .holds
            .live()
            .filter(|h| h.entry.is_some() || self.views.is_empty())
            .collect();
        let m = shown.last()?;
        let live = shown.len();
        let label = if self.holds.paused() {
            words::PAUSED.to_string()
        } else {
            self.waits_label(m)
        };
        let mut line = SlotLine::new(SlotKind::Holding, label, m.label.clone())
            .with_icon(self.waits_icon(m))
            .with_hint("⌘Z", ledger_words::STOP)
            .with_hint("↑", "more");
        if live > 1 {
            line = line.with_sub(format!("+ {} more", live - 1));
        }
        line.progress = Some(m.progress());
        Some(line)
    }

    /// What `e` did, as it went, how long ago, and ⌘Z when ⌘Z would take back this one. A
    /// failure keeps its own icon and is drawn at full strength.
    fn ran_line(&self, e: &Entry, now: Ms) -> SlotLine {
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
        parts.push(e.when_words(now));
        let failed = e.icon == crate::contract::FAILED_ICON;
        let icon = if failed {
            crate::contract::FAILED_ICON
        } else {
            crate::contract::RECENT_ICON
        };
        let mut line = SlotLine::new(SlotKind::Recent, words::LAST, e.title.clone())
            .with_icon(icon)
            .with_sub(parts.join(" · "));
        line.failed = failed;
        let newest = self.ledger.last().is_some_and(|last| last.id == e.id);
        if newest && Self::undoable(e) {
            line.with_hint("⌘Z", ledger_words::UNDO)
        } else {
            line
        }
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
            chip: self.chip(),
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

    pub(super) fn style_of(&self, slot: &SlotLine) -> SlotStyle {
        SlotStyle::new(slot, !self.text.trim().is_empty(), self.in_view())
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
