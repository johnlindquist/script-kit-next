use std::ops::Range;

use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};

struct ProfileSpec {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    icon: &'static str,
}

const SPINE_PROFILES: &[ProfileSpec] = &[
    ProfileSpec {
        id: "creative",
        title: "Creative",
        description: "More exploratory and generative",
        icon: "sparkles",
    },
    ProfileSpec {
        id: "concise",
        title: "Concise",
        description: "Short, direct responses",
        icon: "minimize-2",
    },
    ProfileSpec {
        id: "technical",
        title: "Technical",
        description: "Precise engineering-focused responses",
        icon: "code",
    },
    ProfileSpec {
        id: "friendly",
        title: "Friendly",
        description: "Warm and approachable responses",
        icon: "smile",
    },
];

pub(super) fn build_profile_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    SPINE_PROFILES
        .iter()
        .enumerate()
        .filter(|(_, spec)| {
            let pipe_text = format!("|{}", spec.id);
            matches_query(spec.id, query)
                || matches_query(spec.title, query)
                || matches_query(&pipe_text, query)
                || matches_query(spec.description, query)
        })
        .map(|(rank, spec)| {
            let replacement = format!("|{}", spec.id);
            SpineListRow {
                id: ss(format!("spine:|:{}", spec.id)),
                kind: SpineListRowKind::Profile {
                    profile_id: ss(spec.id),
                },
                title: ss(replacement.clone()),
                subtitle: Some(ss(spec.description)),
                meta: Some(ss(spec.title)),
                icon: Some(ss(spec.icon)),
                badges: vec![ss("|")],
                score: i32::MAX.saturating_sub(rank as i32),
                is_selectable: true,
                action_label: Some(ss("Insert")),
                action: SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range: segment_byte_range.clone(),
                    replacement: ss(replacement.clone()),
                    resolution_id: ss(spec.id),
                    resolution_label: ss(replacement),
                    resolution_source: ss("profile"),
                    trailing_space: true,
                },
            }
        })
        .collect()
}
