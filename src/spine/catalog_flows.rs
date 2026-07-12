//! `-` flow search section — the flow-roster twin of the `/` command search.
//!
//! Typing `-` at a segment start lists the mdflow flows for the effective
//! working directory (same corpus as the main-menu flow rows and the Flow
//! Desk: `flow_catalog().roster_for(...)` merged with package flows). Typing
//! after the sigil filters with the shared `filter_flows` ranking. Selecting
//! a row emits a `ResolveSegment` action with `resolution_source = "flow"`;
//! the Agent Chat composer stages the flow markdown as attached context
//! (skill-search parity) when it applies that resolution.

use std::ops::Range;

use super::list::{ss, SpineListAction, SpineListRow, SpineListRowKind, SpineListSection};
use crate::flows::catalog::{desk_flows, filter_flows, flow_catalog, RosterStatus};

const FLOW_SECTION_ROW_LIMIT: usize = 12;

pub(super) fn build_flow_section(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    current_cwd: Option<String>,
) -> SpineListSection {
    let cwd = crate::flows::resolve_flow_cwd(current_cwd);
    let roster = flow_catalog().roster_for(&cwd);
    let corpus = desk_flows(&roster);

    let mut rows: Vec<SpineListRow> = filter_flows(&corpus, query)
        .into_iter()
        .take(FLOW_SECTION_ROW_LIMIT)
        .enumerate()
        .map(|(rank, flow)| {
            let replacement = format!("-{}", flow.name.replace(' ', "-"));
            SpineListRow {
                id: ss(format!("spine:-:flow:{}", flow.id)),
                kind: SpineListRowKind::Flow {
                    flow_id: ss(flow.id.clone()),
                },
                title: ss(flow.friendly_name()),
                subtitle: flow.description.clone().map(ss),
                meta: Some(ss(flow.origin_label().to_string())),
                icon: Some(ss("flow")),
                badges: vec![ss("-")],
                score: i32::MAX.saturating_sub(rank as i32),
                is_selectable: true,
                action_label: None,
                action: SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range: segment_byte_range.clone(),
                    replacement: ss(replacement),
                    resolution_id: ss(flow.path.clone()),
                    resolution_label: ss(flow.friendly_name()),
                    resolution_source: ss("flow"),
                    trailing_space: true,
                },
            }
        })
        .collect();

    if rows.is_empty() && roster.status == RosterStatus::Loading {
        rows.push(SpineListRow {
            id: ss("spine:-:flows-loading"),
            kind: SpineListRowKind::Hint,
            title: ss("Loading flows\u{2026}"),
            subtitle: Some(ss("Reading the mdflow roster for this directory")),
            meta: Some(ss("Spine")),
            icon: Some(ss("flow")),
            badges: vec![],
            score: 0,
            is_selectable: false,
            action_label: None,
            action: SpineListAction::Noop,
        });
    }

    super::list::section_with_empty(
        "spine-section-flows",
        "Flows",
        Some(ss("Search and stage an mdflow flow")),
        Some(ss("flow")),
        rows,
        "No flow matches",
        "Try -scout or -components",
    )
}
