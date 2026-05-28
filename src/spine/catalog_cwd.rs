use std::ops::Range;

use super::list::{ss, SpineListAction, SpineListRow, SpineListRowKind, SpineListSection};

pub(super) fn build_cwd_root_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    let q = query.trim().to_ascii_lowercase();
    let mut rows = Vec::new();

    let specs: &[(&str, &str, &str)] = &[
        ("Home", "~/", "folder"),
        ("Desktop", "~/Desktop/", "monitor"),
        ("Documents", "~/Documents/", "file-text"),
        ("Downloads", "~/Downloads/", "download"),
        ("Developer", "~/dev/", "code"),
    ];

    for (rank, (title, path, icon)) in specs.iter().enumerate() {
        if !q.is_empty()
            && !title.to_ascii_lowercase().contains(&q)
            && !path.to_ascii_lowercase().contains(&q)
        {
            continue;
        }
        let shortname = title.to_ascii_lowercase();
        rows.push(SpineListRow {
            id: ss(format!("spine:>:dir:{shortname}")),
            kind: SpineListRowKind::Hint,
            title: ss(*title),
            subtitle: None,
            meta: None,
            icon: Some(ss(*icon)),
            badges: vec![],
            score: i32::MAX.saturating_sub(rank as i32),
            is_selectable: true,
            action_label: Some(ss("Set CWD")),
            action: SpineListAction::ResolveSegment {
                segment_index,
                segment_byte_range: segment_byte_range.clone(),
                replacement: ss(format!(">:{shortname}")),
                resolution_id: ss(shellexpand::tilde(path)
                    .to_string()
                    .trim_end_matches('/')
                    .to_string()),
                resolution_label: ss(*title),
                resolution_source: ss("cwd"),
                trailing_space: true,
            },
        });
    }

    rows
}

pub(super) fn build_cwd_section(
    parse: &super::types::SpineParse,
    projection: &super::types::SpineCursorProjection,
) -> SpineListSection {
    let range = super::list::active_segment_range(parse, projection);
    let rows = build_cwd_root_rows(
        &projection.active_query,
        projection.active_segment_index,
        range,
    );

    SpineListSection {
        id: ss("spine-section-cwd"),
        title: ss("Directory"),
        subtitle: Some(ss("Choose a working directory")),
        icon: Some(ss("folder")),
        rows: if rows.is_empty() {
            vec![SpineListRow {
                id: ss("spine:>:empty"),
                kind: SpineListRowKind::Empty,
                title: ss("No matching directories"),
                subtitle: Some(ss("Try Home, Desktop, Documents, Downloads, or Developer")),
                icon: Some(ss("folder")),
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
