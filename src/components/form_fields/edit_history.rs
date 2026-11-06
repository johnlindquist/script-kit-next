//! Local text-field undo. Store one before-image per typing burst, not per character. Neither
//! password values nor earlier occupants of a rebound field enter this short, bounded history.
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
const STEPS: usize = 32;
const BYTES: usize = 2 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Typing,
    Backspace,
    Delete,
    Atomic,
}
pub(super) struct Snapshot {
    pub value: String,
    pub cursor: usize,
    pub anchor: Option<usize>,
}
impl Snapshot {
    fn capture(value: &str, cursor: usize, anchor: Option<usize>) -> Self {
        Self {
            value: value.into(),
            cursor,
            anchor,
        }
    }
}
#[derive(Default)]
pub(super) struct History {
    undo: VecDeque<Snapshot>,
    redo: VecDeque<Snapshot>,
    last: Option<(Kind, u64, Instant)>,
}
pub(super) struct Pending {
    before: Option<Snapshot>,
    rev: u64,
    kind: Kind,
}
impl History {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn break_group(&mut self) {
        self.last = None;
    }
    pub fn begin(
        &mut self,
        value: &str,
        cursor: usize,
        anchor: Option<usize>,
        rev: u64,
        kind: Option<Kind>,
    ) -> Option<Pending> {
        let Some(kind) = kind else {
            self.break_group();
            return None;
        };
        if value.len() > BYTES {
            self.reset();
            return None;
        }
        let merge = kind != Kind::Atomic
            && anchor.is_none_or(|a| a == cursor)
            && self.last.is_some_and(|(k, r, t)| {
                k == kind && r == rev && t.elapsed() < Duration::from_millis(900)
            });
        Some(Pending {
            before: (!merge).then(|| Snapshot::capture(value, cursor, anchor)),
            rev,
            kind,
        })
    }
    pub fn finish(&mut self, pending: Option<Pending>, value: &str, rev: u64) {
        let Some(p) = pending else {
            return;
        };
        if value.len() > BYTES {
            self.reset();
            return;
        }
        if rev == p.rev || p.before.as_ref().is_some_and(|s| s.value == value) {
            return;
        }
        self.redo.clear();
        if let Some(s) = p.before {
            push(&mut self.undo, s);
        }
        self.last = Some((p.kind, rev, Instant::now()));
    }
    pub fn step(
        &mut self,
        redo: bool,
        value: &str,
        cursor: usize,
        anchor: Option<usize>,
    ) -> Option<Snapshot> {
        self.break_group();
        if value.len() > BYTES {
            self.reset();
            return None;
        }
        let (from, to) = if redo {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        let next = from.pop_back()?;
        push(to, Snapshot::capture(value, cursor, anchor));
        Some(next)
    }
}
fn push(q: &mut VecDeque<Snapshot>, s: Snapshot) {
    q.push_back(s);
    while q.len() > STEPS || q.iter().map(|s| s.value.len()).sum::<usize>() > BYTES {
        q.pop_front();
    }
}
pub(super) fn kind(e: &gpui::KeyDownEvent, multiline: bool) -> Option<Kind> {
    let k = e.keystroke.key.as_str();
    let m = e.keystroke.modifiers;
    if m.platform {
        return (k.eq_ignore_ascii_case("x") || k.eq_ignore_ascii_case("v"))
            .then_some(Kind::Atomic);
    }
    if m.control {
        return None;
    }
    if crate::ui_foundation::is_key_backspace(k) {
        Some(Kind::Backspace)
    } else if crate::ui_foundation::is_key_delete(k) {
        Some(Kind::Delete)
    } else if multiline && crate::ui_foundation::is_key_enter(k) {
        Some(Kind::Atomic)
    } else if e
        .keystroke
        .key_char
        .as_ref()
        .is_some_and(|s| s.chars().any(|c| !c.is_control()))
    {
        Some(Kind::Typing)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjacent_typing_has_one_undo_and_new_typing_removes_redo() {
        let mut h = History::default();
        let a = h.begin("Draft", 5, None, 1, Some(Kind::Typing));
        h.finish(a, "Draft!", 2);
        let b = h.begin("Draft!", 6, None, 2, Some(Kind::Typing));
        h.finish(b, "Draft!!", 3);
        let old = h.step(false, "Draft!!", 7, None).unwrap();
        assert_eq!(old.value, "Draft");
        assert_eq!(old.cursor, 5);
        let c = h.begin("Draft", 5, None, 4, Some(Kind::Typing));
        h.finish(c, "Draft?", 5);
        assert!(h.step(true, "Draft?", 6, None).is_none());
    }
    #[test]
    fn selection_and_neighboring_field_are_independent() {
        let mut h = History::default();
        let mut other = History::default();
        let a = h.begin("ab", 0, Some(2), 1, Some(Kind::Typing));
        h.finish(a, "c", 2);
        let b = other.begin("First line", 10, None, 1, Some(Kind::Typing));
        other.finish(b, "First line.", 2);
        let old = h.step(false, "c", 1, None).unwrap();
        assert_eq!(
            (old.value.as_str(), old.cursor, old.anchor),
            ("ab", 0, Some(2))
        );
        assert_eq!(
            other.step(false, "First line.", 11, None).unwrap().value,
            "First line"
        );
    }
}
