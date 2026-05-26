use std::ops::Range;

use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};

struct StyleSpec {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    icon: &'static str,
}

const SPINE_STYLES: &[StyleSpec] = &[
    StyleSpec {
        id: "professional",
        title: "Professional",
        description: "Polished workplace tone",
        icon: "briefcase",
    },
    StyleSpec {
        id: "concise",
        title: "Concise",
        description: "Shorten without losing meaning",
        icon: "minimize-2",
    },
    StyleSpec {
        id: "friendly",
        title: "Friendly",
        description: "Warmer tone",
        icon: "smile",
    },
    StyleSpec {
        id: "direct",
        title: "Direct",
        description: "Plainspoken and direct",
        icon: "arrow-right",
    },
];

pub(super) fn build_style_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    SPINE_STYLES
        .iter()
        .enumerate()
        .filter(|(_, spec)| {
            let dot_text = format!(".{}", spec.id);
            matches_query(spec.id, query)
                || matches_query(spec.title, query)
                || matches_query(&dot_text, query)
                || matches_query(spec.description, query)
        })
        .map(|(rank, spec)| {
            let replacement = format!(".{}", spec.id);
            SpineListRow {
                id: ss(format!("spine:.:{}", spec.id)),
                kind: SpineListRowKind::Style {
                    style_id: ss(spec.id),
                },
                title: ss(replacement.clone()),
                subtitle: Some(ss(spec.description)),
                meta: Some(ss(spec.title)),
                icon: Some(ss(spec.icon)),
                badges: vec![ss(".")],
                score: i32::MAX.saturating_sub(rank as i32),
                is_selectable: true,
                action_label: Some(ss("Insert")),
                action: SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range: segment_byte_range.clone(),
                    replacement: ss(replacement.clone()),
                    resolution_id: ss(spec.id),
                    resolution_label: ss(replacement),
                    resolution_source: ss("style"),
                    trailing_space: true,
                },
            }
        })
        .collect()
}
