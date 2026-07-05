use std::ops::Range;
use std::path::{Path, PathBuf};

use super::list::{ss, SpineListAction, SpineListRow, SpineListRowKind, SpineListSection};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct CwdRootRowsParams<'a> {
    pub current_cwd: Option<&'a Path>,
    pub recents: &'a [PathBuf],
}

pub(super) fn build_cwd_root_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    params: CwdRootRowsParams<'_>,
) -> Vec<SpineListRow> {
    let q = query.trim().to_ascii_lowercase();
    let mut rows = Vec::new();
    let static_specs: &[(&str, &str, &str)] = &[
        ("Script Kit", "~/.scriptkit/", "code"),
        ("Home", "~/", "folder"),
        ("Desktop", "~/Desktop/", "monitor"),
        ("Documents", "~/Documents/", "file-text"),
        ("Downloads", "~/Downloads/", "download"),
        ("Developer", "~/dev/", "code"),
    ];
    let static_paths: Vec<PathBuf> = static_specs
        .iter()
        .map(|(_, path, _)| {
            PathBuf::from(shellexpand::tilde(path).to_string().trim_end_matches('/'))
        })
        .collect();

    if let Some(current_cwd) = params.current_cwd {
        let current_path = current_cwd.to_string_lossy().to_string();
        let current_subtitle = prettify_cwd_path(current_cwd);
        if q.is_empty()
            || "current".contains(&q)
            || current_path.to_ascii_lowercase().contains(&q)
            || current_subtitle.to_ascii_lowercase().contains(&q)
        {
            rows.push(SpineListRow {
                id: ss("spine:>:dir:current"),
                kind: SpineListRowKind::Hint,
                title: ss("Current"),
                subtitle: Some(ss(current_subtitle)),
                meta: None,
                icon: Some(ss("folder")),
                badges: vec![],
                score: i32::MAX,
                is_selectable: true,
                action_label: None,
                action: SpineListAction::ResolveSegment {
                    segment_index,
                    segment_byte_range: segment_byte_range.clone(),
                    replacement: ss(">:current"),
                    resolution_id: ss(current_path),
                    resolution_label: ss("Current"),
                    resolution_source: ss("cwd"),
                    trailing_space: true,
                },
            });
        }
    }

    let mut seen_recent_paths = Vec::<PathBuf>::new();
    for recent in params.recents {
        if params.current_cwd == Some(recent.as_path())
            || static_paths.iter().any(|static_path| static_path == recent)
            || seen_recent_paths.iter().any(|seen| seen == recent)
        {
            continue;
        }
        let title = recent
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string())
            .unwrap_or_else(|| recent.to_string_lossy().to_string());
        let subtitle = prettify_cwd_path(recent);
        if !matches_cwd_query(&title, &subtitle, &q) {
            continue;
        }
        let path = recent.to_string_lossy().to_string();
        let rank = seen_recent_paths.len();
        rows.push(SpineListRow {
            id: ss(format!("spine:>:dir:recent:{path}")),
            kind: SpineListRowKind::Hint,
            title: ss(title.clone()),
            subtitle: Some(ss(subtitle)),
            meta: Some(ss("Recent")),
            icon: Some(ss("clock")),
            badges: vec![],
            score: i32::MAX.saturating_sub(1 + rank as i32),
            is_selectable: true,
            action_label: None,
            action: SpineListAction::ResolveSegment {
                segment_index,
                segment_byte_range: segment_byte_range.clone(),
                replacement: ss(format!(">:{}", title.to_ascii_lowercase())),
                resolution_id: ss(path),
                resolution_label: ss(title),
                resolution_source: ss("cwd"),
                trailing_space: true,
            },
        });
        seen_recent_paths.push(recent.clone());
    }

    let static_score_offset = 1 + params.recents.len() as i32;
    for (rank, (title, path, icon)) in static_specs.iter().enumerate() {
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
            score: i32::MAX.saturating_sub(static_score_offset + rank as i32),
            is_selectable: true,
            action_label: None,
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

fn matches_cwd_query(title: &str, subtitle: &str, query: &str) -> bool {
    query.is_empty()
        || "recent".contains(query)
        || title.to_ascii_lowercase().contains(query)
        || subtitle.to_ascii_lowercase().contains(query)
}

fn prettify_cwd_path(path: &Path) -> String {
    let raw = path.to_string_lossy().to_string();
    let Some(home) = dirs::home_dir() else {
        return raw;
    };
    if path == home {
        return "~/".to_string();
    }
    if let Ok(relative) = path.strip_prefix(&home) {
        let relative = relative.to_string_lossy();
        return format!("~/{relative}");
    }
    raw
}

pub(super) fn build_cwd_section(
    parse: &super::types::SpineParse,
    projection: &super::types::SpineCursorProjection,
    params: CwdRootRowsParams<'_>,
) -> SpineListSection {
    let range = super::list::active_segment_range(parse, projection);
    let rows = build_cwd_root_rows(
        &projection.active_query,
        projection.active_segment_index,
        range,
        params,
    );

    SpineListSection {
        id: ss("spine-section-cwd"),
        title: ss("Agent Working Directory"),
        subtitle: Some(ss(
            "Where this chat reads and writes files and runs commands",
        )),
        icon: Some(ss("folder")),
        rows: if rows.is_empty() {
            vec![SpineListRow {
                id: ss("spine:>:empty"),
                kind: SpineListRowKind::Empty,
                title: ss("No matching directories"),
                subtitle: Some(ss(
                    "Try Current, Script Kit, Home, Desktop, Documents, Downloads, or Developer",
                )),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(rows: &[SpineListRow]) -> Vec<String> {
        rows.iter().map(|row| row.title.to_string()).collect()
    }

    #[test]
    fn cwd_rows_put_current_then_script_kit_before_static_roots() {
        let current = Path::new("/tmp/script-kit-gpui-cwd");
        let rows = build_cwd_root_rows(
            "",
            0,
            0..1,
            CwdRootRowsParams {
                current_cwd: Some(current),
                recents: &[],
            },
        );

        assert_eq!(
            titles(&rows),
            vec![
                "Current",
                "Script Kit",
                "Home",
                "Desktop",
                "Documents",
                "Downloads",
                "Developer",
            ]
        );
        assert!(rows[0].score > rows[1].score);
        assert!(rows[1].score > rows[2].score);
        assert_eq!(
            rows[0].subtitle.as_ref().map(|value| value.as_ref()),
            Some("/tmp/script-kit-gpui-cwd")
        );
        match &rows[0].action {
            SpineListAction::ResolveSegment {
                resolution_id,
                resolution_label,
                resolution_source,
                ..
            } => {
                assert_eq!(resolution_id.as_ref(), "/tmp/script-kit-gpui-cwd");
                assert_eq!(resolution_label.as_ref(), "Current");
                assert_eq!(resolution_source.as_ref(), "cwd");
            }
            other => panic!("expected cwd resolve action, got {other:?}"),
        }
    }

    #[test]
    fn cwd_rows_omit_current_when_no_current_cwd_is_available() {
        let rows = build_cwd_root_rows("", 0, 0..1, CwdRootRowsParams::default());

        assert_eq!(rows.first().map(|row| row.title.as_ref()), Some("Script Kit"));
        assert!(!titles(&rows).iter().any(|title| title == "Current"));
    }

    #[test]
    fn cwd_section_explains_agent_working_directory_purpose() {
        let parse = crate::spine::parse_spine(">");
        let projection = crate::spine::project_cursor(&parse, 1);
        let section = build_cwd_section(
            &parse,
            &projection,
            CwdRootRowsParams {
                current_cwd: Some(Path::new("/tmp/current")),
                recents: &[],
            },
        );

        assert_eq!(section.title.as_ref(), "Agent Working Directory");
        assert_eq!(
            section.subtitle.as_ref().map(|value| value.as_ref()),
            Some("Where this chat reads and writes files and runs commands")
        );
        assert_eq!(section.rows.first().map(|row| row.title.as_ref()), Some("Current"));
    }

    #[test]
    fn cwd_rows_order_current_recents_then_statics() {
        let current = Path::new("/tmp/current");
        let recents = vec![PathBuf::from("/tmp/recent-project")];
        let rows = build_cwd_root_rows(
            "",
            0,
            0..1,
            CwdRootRowsParams {
                current_cwd: Some(current),
                recents: &recents,
            },
        );

        assert_eq!(
            titles(&rows).into_iter().take(3).collect::<Vec<_>>(),
            vec!["Current", "recent-project", "Script Kit"]
        );
        assert!(rows[0].score > rows[1].score);
        assert!(rows[1].score > rows[2].score);
        assert_eq!(rows[1].icon.as_ref().map(|icon| icon.as_ref()), Some("clock"));
    }

    #[test]
    fn cwd_rows_dedupe_recents_against_current_and_statics() {
        let home = dirs::home_dir().expect("home dir");
        let current = Path::new("/tmp/current");
        let recents = vec![
            current.to_path_buf(),
            home,
            PathBuf::from("/tmp/kept"),
            PathBuf::from("/tmp/kept"),
        ];
        let rows = build_cwd_root_rows(
            "",
            0,
            0..1,
            CwdRootRowsParams {
                current_cwd: Some(current),
                recents: &recents,
            },
        );

        let row_titles = titles(&rows);
        assert_eq!(
            row_titles.iter().filter(|title| title.as_str() == "kept").count(),
            1
        );
        assert_eq!(
            row_titles.iter().filter(|title| title.as_str() == "Home").count(),
            1
        );
    }

    #[test]
    fn cwd_rows_filter_recents_by_title_or_subtitle() {
        let recents = vec![
            PathBuf::from("/tmp/alpha-project"),
            PathBuf::from("/tmp/beta-project"),
        ];
        let rows = build_cwd_root_rows(
            "alpha",
            0,
            0..1,
            CwdRootRowsParams {
                current_cwd: None,
                recents: &recents,
            },
        );

        let row_titles = titles(&rows);
        assert!(row_titles.contains(&"alpha-project".to_string()));
        assert!(!row_titles.contains(&"beta-project".to_string()));
    }
}
