use std::ops::Range;

use super::list::{ss, SpineListAction, SpineListRow, SpineListRowKind, SpineListSection};

struct FilterQualifier {
    token: &'static str,
    title: &'static str,
    subtitle: &'static str,
    example: &'static str,
}

const FILTER_QUALIFIERS: &[FilterQualifier] = &[
    FilterQualifier {
        token: "type:script",
        title: "Scripts only",
        subtitle: "Limit results to runnable scripts",
        example: ":type:script git",
    },
    FilterQualifier {
        token: "type:scriptlet",
        title: "Scriptlets only",
        subtitle: "Limit results to scriptlets",
        example: ":type:scriptlet shell",
    },
    FilterQualifier {
        token: "type:skill",
        title: "Skills only",
        subtitle: "Find agent skills",
        example: ":type:skill review",
    },
    FilterQualifier {
        token: "type:builtin",
        title: "Built-ins only",
        subtitle: "Limit results to built-in commands",
        example: ":type:builtin clipboard",
    },
    FilterQualifier {
        token: "type:app",
        title: "Apps only",
        subtitle: "Limit results to applications",
        example: ":type:app safari",
    },
    FilterQualifier {
        token: "type:window",
        title: "Windows only",
        subtitle: "Limit results to open windows",
        example: ":type:window chrome",
    },
    FilterQualifier {
        token: "type:agent",
        title: "Agents only",
        subtitle: "Limit results to agents",
        example: ":type:agent",
    },
    FilterQualifier {
        token: "shortcut:any",
        title: "Has any shortcut",
        subtitle: "Items with keyboard shortcuts",
        example: ":shortcut:any",
    },
    FilterQualifier {
        token: "shortcut:none",
        title: "Has no shortcut",
        subtitle: "Items without keyboard shortcuts",
        example: ":shortcut:none",
    },
    FilterQualifier {
        token: "source:",
        title: "Source filter",
        subtitle: "Broad match against plugin or kit name",
        example: ":source:main inbox",
    },
    FilterQualifier {
        token: "tag:",
        title: "Tag filter",
        subtitle: "Filter by metadata tag",
        example: ":#work type:script",
    },
];

pub(super) fn build_filter_qualifier_section(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> SpineListSection {
    let q = query.trim().trim_start_matches(':').to_ascii_lowercase();

    let rows: Vec<SpineListRow> = FILTER_QUALIFIERS
        .iter()
        .enumerate()
        .filter(|(_, qual)| {
            q.is_empty()
                || qual.token.to_ascii_lowercase().contains(&q)
                || qual.title.to_ascii_lowercase().contains(&q)
        })
        .map(|(rank, qual)| SpineListRow {
            id: ss(format!("spine:::qualifier:{}", qual.token)),
            kind: SpineListRowKind::Hint,
            title: ss(format!(":{}", qual.token)),
            subtitle: Some(ss(qual.subtitle)),
            meta: Some(ss(qual.title)),
            icon: Some(ss("filter")),
            badges: vec![ss(":")],
            score: i32::MAX.saturating_sub(rank as i32),
            is_selectable: true,
            action_label: Some(ss("Insert")),
            action: SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range: segment_byte_range.clone(),
                text: ss(format!(":{}", qual.token)),
                trailing_space: true,
            },
        })
        .collect();

    SpineListSection {
        id: ss("spine-section-filter"),
        title: ss("Refine Search"),
        subtitle: Some(ss("Filter unified search results")),
        icon: Some(ss("filter")),
        rows: if rows.is_empty() {
            vec![SpineListRow {
                id: ss("spine:::qualifier:empty"),
                kind: SpineListRowKind::Empty,
                title: ss("No matching qualifiers"),
                subtitle: Some(ss("Try :type: or :shortcut:")),
                icon: Some(ss("info")),
                meta: None,
                badges: vec![],
                score: 0,
                is_selectable: false,
                action_label: None,
                action: SpineListAction::Noop,
            }]
        } else {
            rows
        },
    }
}
