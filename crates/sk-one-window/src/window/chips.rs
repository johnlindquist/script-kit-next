//! The chips at the input's start: the attached subject, then what `@` attached. They behave like
//! part of the input: ⌫ at the caret's start removes the last chip, ← highlights the chip before
//! the caret and ⌫ removes that one, → walks back to the words, and Esc lets go of them all.

use super::*;

impl OneWindow {
    /// What `@` attached, in order.
    pub fn context(&self) -> &[ContextChip] {
        &self.context
    }

    /// The chip ← highlighted, counted from the attached subject when there is one.
    pub fn chip_at(&self) -> Option<usize> {
        self.chip_at
    }

    /// The chips in the input: the attached subject, then what `@` attached.
    fn chip_count(&self) -> usize {
        usize::from(self.staged.is_some() && self.cfg.subject_chip) + self.context.len()
    }

    pub(super) fn attach(&mut self, chip: ContextChip) -> bool {
        if self.in_view() {
            return false;
        }
        self.chip_at = None;
        if !self.context.iter().any(|c| c.token == chip.token) {
            self.context.push(chip);
        }
        true
    }

    /// Everything the chips hold goes with the words: a fresh input has none.
    pub(super) fn drop_chips(&mut self) {
        self.context.clear();
        self.chip_at = None;
    }

    /// ⌫, ←, → and Esc as the chips take them; `None` leaves the key to the rest of the window.
    /// The host sends ⌫ and ← only with the caret at the input's start and nothing selected.
    pub(super) fn chip_key(&mut self, now: Ms, key: &Key, fx: &mut Vec<Effect>) -> Option<bool> {
        let n = self.chip_count();
        match key {
            Key::Backspace if n > 0 => {
                let at = self.chip_at.take().unwrap_or(n - 1);
                self.remove_chip(at);
                Some(true)
            }
            Key::Left if n > 0 => {
                self.chip_at = Some(self.chip_at.map_or(n - 1, |at| at.saturating_sub(1)));
                Some(true)
            }
            Key::Right if self.chip_at.is_some() => {
                self.chip_at = self.chip_at.map(|at| at + 1).filter(|&at| at < n);
                Some(true)
            }
            Key::Escape if self.chip_at.is_some() => {
                self.chip_at = None;
                Some(true)
            }
            Key::Escape
                if !self.context.is_empty() && self.staged.is_none() && !self.agent_owns =>
            {
                self.home(now, String::new(), fx);
                Some(true)
            }
            _ => {
                self.chip_at = None;
                None
            }
        }
    }

    /// The subject chip is let go the way Esc lets go of it, but the words stay.
    fn remove_chip(&mut self, at: usize) {
        let subject = usize::from(self.staged.is_some() && self.cfg.subject_chip);
        if at < subject {
            self.let_go();
        } else if at - subject < self.context.len() {
            self.context.remove(at - subject);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chip(token: &str, title: &str) -> ContextChip {
        ContextChip {
            token: token.into(),
            title: title.into(),
            icon: "file".into(),
        }
    }

    fn key(w: &mut OneWindow, now: Ms, key: Key) -> Outcome {
        w.handle(now, Event::Key(key))
    }

    fn tokens(w: &OneWindow) -> Vec<&str> {
        w.context().iter().map(|c| c.token.as_str()).collect()
    }

    #[test]
    fn backspace_at_the_start_removes_the_newest_chip_and_then_the_one_before() {
        let mut w = OneWindow::default();
        w.handle(0, Event::Attach(chip("@selection", "Selected text")));
        w.handle(1, Event::Attach(chip("@clipboard", "Clipboard")));
        // The same thing twice is one chip.
        w.handle(2, Event::Attach(chip("@selection", "Selected text")));
        assert_eq!(tokens(&w), ["@selection", "@clipboard"]);

        assert!(key(&mut w, 10, Key::Backspace).consumed);
        assert_eq!(tokens(&w), ["@selection"]);
        assert!(key(&mut w, 20, Key::Backspace).consumed);
        assert!(tokens(&w).is_empty());
        // With no chips left, ⌫ is the input's again.
        assert!(!key(&mut w, 30, Key::Backspace).consumed);
    }

    #[test]
    fn left_walks_back_over_the_chips_and_backspace_removes_the_highlighted_one() {
        let mut w = OneWindow::default();
        for (t, title) in [
            ("@selection", "Selected text"),
            ("@clipboard", "Clipboard"),
            ("@git-diff", "Git diff"),
        ] {
            w.handle(0, Event::Attach(chip(t, title)));
        }
        assert!(key(&mut w, 10, Key::Left).consumed);
        assert_eq!(w.chip_at(), Some(2));
        key(&mut w, 20, Key::Left);
        key(&mut w, 30, Key::Left);
        key(&mut w, 40, Key::Left);
        assert_eq!(w.chip_at(), Some(0), "← stops at the first chip");
        key(&mut w, 50, Key::Right);
        assert_eq!(w.chip_at(), Some(1));

        key(&mut w, 60, Key::Backspace);
        assert_eq!(tokens(&w), ["@selection", "@git-diff"]);
        assert_eq!(w.chip_at(), None, "the caret is back in the words");

        // → past the last chip goes back to the words; so does typing.
        key(&mut w, 70, Key::Left);
        key(&mut w, 80, Key::Right);
        assert_eq!(w.chip_at(), None);
        key(&mut w, 90, Key::Left);
        w.handle(100, Event::Text("fix".into()));
        assert_eq!(w.chip_at(), None);
        // Esc with a chip highlighted only lets go of the highlight.
        key(&mut w, 110, Key::Left);
        assert!(key(&mut w, 120, Key::Escape).consumed);
        assert_eq!(w.chip_at(), None);
        assert_eq!(tokens(&w).len(), 2);
    }

    #[test]
    fn the_subject_is_the_first_chip_and_backspace_lets_it_go_keeping_the_words() {
        let mut w = OneWindow::default();
        w.handle(
            0,
            Event::Focused(Subject {
                kind: SubjectKind::Window,
                title: "notes.md".into(),
                sub: "Zed".into(),
                ..Subject::default()
            }),
        );
        key(&mut w, 10, Key::Space);
        assert!(w.chip().is_some_and(|c| c.attached));
        w.handle(20, Event::Attach(chip("@selection", "Selected text")));
        w.handle(30, Event::Text("summarize".into()));

        key(&mut w, 1_000, Key::Left);
        key(&mut w, 1_010, Key::Left);
        assert_eq!(w.chip_at(), Some(0), "the subject chip comes first");
        key(&mut w, 1_020, Key::Backspace);
        assert!(
            w.chip().is_none_or(|c| !c.attached),
            "the subject was let go"
        );
        assert_eq!(tokens(&w), ["@selection"]);
        assert_eq!(w.text(), "summarize");
    }

    #[test]
    fn esc_and_a_fresh_input_let_go_of_every_chip() {
        let mut w = OneWindow::default();
        w.handle(0, Event::Attach(chip("@selection", "Selected text")));
        w.handle(1, Event::Text("summarize @selection ".into()));
        let out = key(&mut w, 10, Key::Escape);
        assert!(out.consumed);
        assert!(tokens(&w).is_empty());
        assert!(out.effects.contains(&Effect::SetInput(String::new())));

        w.handle(20, Event::Attach(chip("@clipboard", "Clipboard")));
        w.handle(30, Event::Summon);
        assert!(tokens(&w).is_empty(), "the window comes back without them");
    }
}
