//! ⌘K in a view: the view names what it has focused (a prompt's value, a file, a clipboard entry)
//! and lists that thing's actions itself. The line says "Acting on" it until the view lets go.
//! Unlike ⌘K on the list, nothing about the input or the list's words changes: the view is still
//! there underneath, and letting go shows it exactly as it was.

use super::*;

impl OneWindow {
    pub(super) fn stage_from_view(&mut self, subject: Subject, rows: Vec<Row>) {
        self.staged = Some(Staged {
            subject,
            rows,
            head: words::STAGED.into(),
            waiting_for_rows: false,
            entry_state: None,
        });
        self.refresh_rows();
    }

    /// Whether something was staged to let go of.
    pub(super) fn let_go(&mut self) -> bool {
        let staged = self.staged.take().is_some();
        if staged {
            self.refresh_rows();
        }
        staged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::SlotKind;

    fn clipboard_entry() -> Subject {
        Subject {
            kind: SubjectKind::Thing,
            label: words::ACTING_ON.into(),
            title: "github.com/script-kit/app/pull/812".into(),
            icon: "copy".into(),
            ..Subject::default()
        }
    }

    fn actions() -> Vec<Row> {
        vec![
            Row::new("action:paste", "Paste"),
            Row::new("action:copy", "Copy"),
        ]
    }

    #[test]
    fn a_view_stages_its_subject_with_its_own_actions() {
        let mut win = OneWindow::default();
        win.handle(0, Event::Text("clip".into()));
        let out = win.handle(
            10,
            Event::StageView {
                subject: clipboard_entry(),
                rows: actions(),
            },
        );
        assert!(out.consumed);
        assert!(
            !out.effects
                .iter()
                .any(|fx| matches!(fx, Effect::ActionsFor(_) | Effect::SetInput(_))),
            "the view already listed the actions and keeps its words: {:?}",
            out.effects
        );
        let line = win.slot_line(10);
        assert_eq!(line.kind, SlotKind::Staged);
        assert_eq!(line.label, words::ACTING_ON);
        assert_eq!(line.title, "github.com/script-kit/app/pull/812");
        assert!(line.hints.iter().any(|(key, _)| key == "esc"));
        let staged = win.staged().expect("staged");
        assert_eq!(staged.head, words::STAGED);
        assert_eq!(staged.rows, actions());
        assert_eq!(win.text(), "clip", "the view's words stay");
    }

    #[test]
    fn letting_go_puts_the_line_back() {
        let mut win = OneWindow::default();
        let before = win.slot_line(0);
        win.handle(
            0,
            Event::StageView {
                subject: clipboard_entry(),
                rows: actions(),
            },
        );
        assert!(win.handle(10, Event::LetGo).consumed);
        assert!(win.staged().is_none());
        assert_eq!(win.slot_line(10), before);
        assert!(
            !win.handle(20, Event::LetGo).consumed,
            "nothing is staged the second time"
        );
    }

    #[test]
    fn the_view_filters_its_actions_through_the_same_rows() {
        let mut win = OneWindow::default();
        win.handle(
            0,
            Event::StageView {
                subject: clipboard_entry(),
                rows: actions(),
            },
        );
        win.handle(10, Event::Actions(vec![Row::new("action:copy", "Copy")]));
        let staged = win.staged().expect("still staged");
        assert_eq!(staged.rows.len(), 1);
        assert_eq!(staged.pinned()[0].head.as_deref(), Some(words::STAGED));
    }
}
