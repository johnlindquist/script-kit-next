use std::ops::Range;

use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};

struct StyleSpec {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    icon: &'static str,
    /// The rewrite instruction the agent actually receives when this style
    /// is part of the submitted prompt plan.
    instruction: &'static str,
}

const SPINE_STYLES: &[StyleSpec] = &[
    StyleSpec {
        id: "professional",
        title: "Professional",
        description: "Polished workplace tone",
        icon: "briefcase",
        instruction: "Rewrite the attached selection in a polished, professional workplace tone. Preserve the meaning and approximate length; return only the rewritten text.",
    },
    StyleSpec {
        id: "concise",
        title: "Concise",
        description: "Shorten without losing meaning",
        icon: "minimize-2",
        instruction: "Rewrite the attached selection to be significantly more concise without losing meaning. Return only the rewritten text.",
    },
    StyleSpec {
        id: "friendly",
        title: "Friendly",
        description: "Warmer tone",
        icon: "smile",
        instruction: "Rewrite the attached selection in a warmer, friendlier tone. Preserve the meaning; return only the rewritten text.",
    },
    StyleSpec {
        id: "direct",
        title: "Direct",
        description: "Plainspoken and direct",
        icon: "arrow-right",
        instruction: "Rewrite the attached selection to be plainspoken and direct. Cut hedging and filler; return only the rewritten text.",
    },
];

/// Rewrite instruction for a known style id, used by the prompt plan so the
/// agent receives an explicit tone instruction instead of a bare `/rewrite`.
pub(crate) fn style_instruction(style_id: &str) -> Option<&'static str> {
    SPINE_STYLES
        .iter()
        .find(|spec| spec.id == style_id)
        .map(|spec| spec.instruction)
}

/// Whether the style id is a known catalog style.
pub(crate) fn is_known_style(style_id: &str) -> bool {
    SPINE_STYLES.iter().any(|spec| spec.id == style_id)
}

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
                title: ss(spec.title),
                subtitle: Some(ss(spec.description)),
                meta: None,
                icon: Some(ss(spec.icon)),
                badges: vec![ss(".")],
                score: i32::MAX.saturating_sub(rank as i32),
                is_selectable: true,
                action_label: None,
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
