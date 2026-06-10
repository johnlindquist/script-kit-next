use std::ops::Range;

use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};

struct CaptureSpec {
    id: &'static str,
    title: &'static str,
    subtitle: &'static str,
    icon: &'static str,
}

const CAPTURE_SPECS: &[CaptureSpec] = &[
    CaptureSpec {
        id: "todo",
        title: "Todo",
        subtitle: "Capture a todo item",
        icon: "check-square",
    },
    CaptureSpec {
        id: "note",
        title: "Note",
        subtitle: "Capture a note",
        icon: "notebook-text",
    },
    CaptureSpec {
        id: "link",
        title: "Link",
        subtitle: "Capture a link",
        icon: "link",
    },
    CaptureSpec {
        id: "snippet",
        title: "Snippet",
        subtitle: "Capture a code snippet",
        icon: "code",
    },
    CaptureSpec {
        id: "bookmark",
        title: "Bookmark",
        subtitle: "Bookmark the frontmost page",
        icon: "bookmark",
    },
];

pub(super) fn build_capture_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    CAPTURE_SPECS
        .iter()
        .enumerate()
        .filter(|(_, spec)| {
            matches_query(spec.id, query)
                || matches_query(spec.title, query)
                || matches_query(spec.subtitle, query)
        })
        .map(|(rank, spec)| SpineListRow {
            id: ss(format!("spine:;:{}", spec.id)),
            kind: SpineListRowKind::CaptureTarget {
                target: ss(spec.id),
            },
            title: ss(spec.title),
            subtitle: Some(ss(spec.subtitle)),
            meta: None,
            icon: Some(ss(spec.icon)),
            badges: vec![ss(";")],
            score: i32::MAX.saturating_sub(rank as i32),
            is_selectable: true,
            action_label: Some(ss("Insert")),
            // A4 decision (2026-06-09): accepting a capture target converts
            // the typed `;to` prefix into the canonical postfix spelling
            // (`todo; `), which hands the input to the capture form.
            action: SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range: segment_byte_range.clone(),
                text: ss(format!("{};", spec.id)),
                trailing_space: true,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_semicolon_lists_all_capture_targets() {
        let rows = build_capture_rows("", 0, 0..1);
        assert_eq!(rows.len(), CAPTURE_SPECS.len());
    }

    /// A4 decision (2026-06-09): accepting `;to` → Todo converts the input to
    /// the postfix spelling `todo; `, which hands off to the capture form.
    #[test]
    fn accepting_target_inserts_postfix_spelling() {
        let rows = build_capture_rows("to", 0, 0..3);
        let todo = rows
            .iter()
            .find(|row| row.id.as_ref() == "spine:;:todo")
            .expect("todo row for ';to'");
        match &todo.action {
            SpineListAction::InsertSegmentText {
                text,
                trailing_space,
                ..
            } => {
                assert_eq!(text.as_ref(), "todo;");
                assert!(*trailing_space);
            }
            other => panic!("expected InsertSegmentText, got {other:?}"),
        }
    }
}
