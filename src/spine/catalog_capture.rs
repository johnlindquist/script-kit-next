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
            action: SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range: segment_byte_range.clone(),
                text: ss(format!(";{}", spec.id)),
                trailing_space: true,
            },
        })
        .collect()
}
