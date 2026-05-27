use std::ops::Range;

use crate::ai::context_contract::{
    context_attachment_specs, ContextAttachmentKind, ContextAttachmentSpec,
};

use super::list::{matches_query, ss, SpineListAction, SpineListRow, SpineListRowKind};

#[derive(Debug, Clone, Copy)]
struct ContextSubsearchSpec {
    prefix: &'static str,
    title: &'static str,
    subtitle: &'static str,
    icon: &'static str,
}

const CONTEXT_SUBSEARCH_SPECS: &[ContextSubsearchSpec] = &[
    ContextSubsearchSpec {
        prefix: "file",
        title: "Files",
        subtitle: "Search files inline",
        icon: "file-search",
    },
    ContextSubsearchSpec {
        prefix: "clipboard",
        title: "Clipboard History",
        subtitle: "Search clipboard history",
        icon: "clipboard",
    },
    ContextSubsearchSpec {
        prefix: "browser-history",
        title: "Browser History",
        subtitle: "Search browser history inline",
        icon: "globe",
    },
    ContextSubsearchSpec {
        prefix: "notes",
        title: "Notes",
        subtitle: "Search notes",
        icon: "notebook-text",
    },
    ContextSubsearchSpec {
        prefix: "history",
        title: "Agent Chat History",
        subtitle: "Search past conversations",
        icon: "message-circle",
    },
    ContextSubsearchSpec {
        prefix: "scripts",
        title: "Scripts",
        subtitle: "Search Script Kit scripts",
        icon: "file-code",
    },
    ContextSubsearchSpec {
        prefix: "dictation",
        title: "Dictation History",
        subtitle: "Search saved dictation",
        icon: "mic",
    },
    ContextSubsearchSpec {
        prefix: "scriptlets",
        title: "Scriptlets",
        subtitle: "Search snippets",
        icon: "scroll-text",
    },
    ContextSubsearchSpec {
        prefix: "skills",
        title: "Skills",
        subtitle: "Search plugin skills",
        icon: "workflow",
    },
];

pub(super) fn build_context_root_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    build_context_root_rows_with_preview(query, segment_index, segment_byte_range, None)
}

pub(super) fn build_context_root_rows_with_preview(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    live_preview: Option<&super::live_preview::SpineLivePreview>,
) -> Vec<SpineListRow> {
    let mut rows = Vec::new();

    for (rank, spec) in context_attachment_specs().iter().enumerate() {
        if let Some(row) = build_builtin_context_row(
            spec,
            rank,
            query,
            segment_index,
            segment_byte_range.clone(),
            live_preview,
        ) {
            rows.push(row);
        }
    }

    for (rank, spec) in CONTEXT_SUBSEARCH_SPECS.iter().enumerate() {
        if let Some(row) = build_subsearch_context_row(
            spec,
            rank,
            query,
            segment_index,
            segment_byte_range.clone(),
        ) {
            rows.push(row);
        }
    }

    rows
}

fn build_builtin_context_row(
    spec: &ContextAttachmentSpec,
    rank: usize,
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    live_preview: Option<&super::live_preview::SpineLivePreview>,
) -> Option<SpineListRow> {
    let mention = spec.mention?;

    if !context_spec_matches_query(spec, query) {
        return None;
    }

    let slug = mention_slug(mention);

    let title = live_preview
        .and_then(|lp| lp.title_for_context_kind(spec.kind))
        .unwrap_or_else(|| spec.label.to_string());

    let subtitle = live_preview
        .and_then(|lp| lp.subtitle_for_context_kind(spec.kind))
        .unwrap_or_else(|| spec.action_title.to_string());

    Some(SpineListRow {
        id: ss(format!("spine:@:builtin:{slug}")),
        kind: SpineListRowKind::ContextBuiltin {
            context_type: ss(slug),
        },
        title: ss(title),
        subtitle: Some(ss(subtitle)),
        meta: Some(ss(mention)),
        icon: Some(ss(icon_for_context_kind(spec.kind))),
        badges: vec![ss("@")],
        score: i32::MAX.saturating_sub(rank as i32),
        is_selectable: true,
        action_label: Some(ss("Attach")),
        action: SpineListAction::ResolveSegment {
            segment_index,
            segment_byte_range,
            replacement: ss(mention),
            resolution_id: ss(spec.action_id),
            resolution_label: ss(spec.label),
            resolution_source: ss("context-builtin"),
            trailing_space: true,
        },
    })
}

fn build_subsearch_context_row(
    spec: &ContextSubsearchSpec,
    rank: usize,
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Option<SpineListRow> {
    if !(matches_query(spec.prefix, query)
        || matches_query(spec.title, query)
        || matches_query(spec.subtitle, query))
    {
        return None;
    }

    let prefix_text = format!("@{}:", spec.prefix);

    Some(SpineListRow {
        id: ss(format!("spine:@:subsearch:{}", spec.prefix)),
        kind: SpineListRowKind::ContextSubSearch {
            context_type: ss(spec.prefix),
        },
        title: ss(prefix_text.clone()),
        subtitle: Some(ss(spec.subtitle)),
        meta: Some(ss(spec.title)),
        icon: Some(ss(spec.icon)),
        badges: vec![ss("@"), ss("search")],
        score: i32::MAX.saturating_sub(100 + rank as i32),
        is_selectable: true,
        action_label: Some(ss("Browse")),
        action: SpineListAction::InsertSegmentText {
            segment_index,
            segment_byte_range,
            text: ss(prefix_text),
            trailing_space: false,
        },
    })
}

fn context_spec_matches_query(spec: &ContextAttachmentSpec, query: &str) -> bool {
    let mention_matches = spec
        .mention
        .is_some_and(|mention| matches_query(mention, query));
    let mention_alias_matches = spec
        .mention_aliases
        .iter()
        .any(|alias| matches_query(alias, query));
    let slash_matches = spec
        .slash_command
        .is_some_and(|slash| matches_query(slash, query));
    let slash_alias_matches = spec
        .slash_aliases
        .iter()
        .any(|alias| matches_query(alias, query));

    matches_query(spec.label, query)
        || matches_query(spec.action_title, query)
        || matches_query(spec.action_id, query)
        || mention_matches
        || mention_alias_matches
        || slash_matches
        || slash_alias_matches
}

fn mention_slug(mention: &str) -> &str {
    mention.trim_start_matches('@')
}

fn icon_for_context_kind(kind: ContextAttachmentKind) -> &'static str {
    match kind {
        ContextAttachmentKind::Current => "scan-text",
        ContextAttachmentKind::Full => "layers",
        ContextAttachmentKind::Selection => "mouse-pointer-2",
        ContextAttachmentKind::Browser => "globe",
        ContextAttachmentKind::Window => "panel-top",
        ContextAttachmentKind::Diagnostics => "bug",
        ContextAttachmentKind::Screenshot => "image",
        ContextAttachmentKind::Clipboard => "clipboard",
        ContextAttachmentKind::FrontmostApp => "app-window",
        ContextAttachmentKind::MenuBar => "menu",
        ContextAttachmentKind::RecentScripts => "file-code",
        ContextAttachmentKind::GitStatus => "git-branch",
        ContextAttachmentKind::GitDiff => "git-compare",
        ContextAttachmentKind::Processes => "activity",
        ContextAttachmentKind::System => "monitor",
        ContextAttachmentKind::Dictation => "mic",
        ContextAttachmentKind::Calendar => "calendar",
        ContextAttachmentKind::Notifications => "bell",
    }
}
