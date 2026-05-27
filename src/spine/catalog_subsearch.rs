use std::ops::Range;
use std::sync::Arc;

use gpui::SharedString;

use super::list::{ss, SpineListAction, SpineListRow, SpineListRowKind, SpineListSection};

pub(crate) const SUBSEARCH_RENDER_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextSubsearchSource {
    File,
    BrowserHistory,
    Clipboard,
    Dictation,
    Scripts,
    Scriptlets,
    Skills,
    Notes,
    History,
}

impl ContextSubsearchSource {
    pub(crate) fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "file" => Some(Self::File),
            "browser-history" => Some(Self::BrowserHistory),
            "clipboard" => Some(Self::Clipboard),
            "dictation" => Some(Self::Dictation),
            "scripts" => Some(Self::Scripts),
            "scriptlets" => Some(Self::Scriptlets),
            "skills" => Some(Self::Skills),
            "notes" => Some(Self::Notes),
            "history" => Some(Self::History),
            _ => None,
        }
    }

    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::BrowserHistory => "browser-history",
            Self::Clipboard => "clipboard",
            Self::Dictation => "dictation",
            Self::Scripts => "scripts",
            Self::Scriptlets => "scriptlets",
            Self::Skills => "skills",
            Self::Notes => "notes",
            Self::History => "history",
        }
    }

    fn section_title(self) -> &'static str {
        match self {
            Self::File => "Files",
            Self::BrowserHistory => "Browser History",
            Self::Clipboard => "Clipboard",
            Self::Dictation => "Dictation",
            Self::Scripts => "Scripts",
            Self::Scriptlets => "Scriptlets",
            Self::Skills => "Skills",
            Self::Notes => "Notes",
            Self::History => "Conversations",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::BrowserHistory => "globe",
            Self::Clipboard => "clipboard",
            Self::Dictation => "mic",
            Self::Scripts => "file-code",
            Self::Scriptlets => "scroll-text",
            Self::Skills => "workflow",
            Self::Notes => "notebook-text",
            Self::History => "message-circle",
        }
    }
}

pub(crate) fn parse_context_subsearch<'a>(
    context_type: &str,
    sub_query: Option<&'a str>,
) -> Option<(ContextSubsearchSource, &'a str)> {
    let sq = sub_query?;
    let source = ContextSubsearchSource::from_prefix(context_type)?;
    Some((source, sq))
}

pub(crate) struct SpineSubsearchContext<'a> {
    pub(crate) scripts: &'a [Arc<crate::scripts::Script>],
    pub(crate) scriptlets: &'a [Arc<crate::scripts::Scriptlet>],
    pub(crate) skills: &'a [Arc<crate::plugins::PluginSkill>],
}

pub(crate) fn build_context_subsearch_section(
    source: ContextSubsearchSource,
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    ctx: Option<&SpineSubsearchContext<'_>>,
) -> SpineListSection {
    let rows = match source {
        ContextSubsearchSource::File => vec![hint_row(
            if query.trim().is_empty() {
                "Recent files"
            } else {
                "Searching files\u{2026}"
            },
            "File results are loaded by the launcher",
            ContextSubsearchSource::File,
        )],
        ContextSubsearchSource::BrowserHistory
        | ContextSubsearchSource::Clipboard
        | ContextSubsearchSource::Dictation
        | ContextSubsearchSource::Notes
        | ContextSubsearchSource::History => vec![hint_row(
            "Loading\u{2026}",
            "Results are loaded by the launcher",
            source,
        )],
        ContextSubsearchSource::Scripts
        | ContextSubsearchSource::Scriptlets
        | ContextSubsearchSource::Skills => vec![hint_row(
            "Loading\u{2026}",
            "Results are loaded by the launcher",
            source,
        )],
    };

    let final_rows = if rows.is_empty() {
        vec![empty_result_row(source, query)]
    } else {
        rows
    };

    SpineListSection {
        id: ss(format!("spine-section-subsearch:{}", source.prefix())),
        title: ss(source.section_title()),
        subtitle: Some(ss(format!("@{}:", source.prefix()))),
        icon: Some(ss(source.icon())),
        rows: final_rows,
    }
}

fn context_result_row(
    source: ContextSubsearchSource,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    stable_id: String,
    title: String,
    subtitle: String,
    inline_text: String,
    score: i32,
) -> SpineListRow {
    SpineListRow {
        id: ss(format!("spine:@:result:{}:{}", source.prefix(), stable_id)),
        kind: SpineListRowKind::ContextResult {
            context_type: ss(source.prefix()),
            result_id: ss(stable_id.clone()),
        },
        title: ss(title.clone()),
        subtitle: Some(ss(subtitle)),
        meta: Some(ss(inline_text.clone())),
        icon: Some(ss(source.icon())),
        badges: vec![ss("@")],
        score,
        is_selectable: true,
        action_label: Some(ss("Attach")),
        action: SpineListAction::ResolveSegment {
            segment_index,
            segment_byte_range,
            replacement: ss(inline_text),
            resolution_id: ss(stable_id),
            resolution_label: ss(title),
            resolution_source: ss(source.prefix()),
            trailing_space: true,
        },
    }
}

fn empty_result_row(source: ContextSubsearchSource, query: &str) -> SpineListRow {
    SpineListRow {
        id: ss(format!("spine:@:result:{}:empty", source.prefix())),
        kind: SpineListRowKind::Empty,
        title: ss(if query.trim().is_empty() {
            format!("Type to search {}", source.section_title().to_lowercase())
        } else {
            format!("No {} matches", source.section_title().to_lowercase())
        }),
        subtitle: Some(ss(format!("Try a different @{}: query", source.prefix()))),
        icon: Some(ss("info")),
        meta: None,
        badges: vec![],
        score: 0,
        is_selectable: false,
        action_label: None,
        action: SpineListAction::Noop,
    }
}

// --- Browser history provider ---

fn build_browser_history_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    let query = query.trim();
    if query.is_empty() {
        return vec![hint_row(
            "Type a page title or domain",
            "Example: @browser-history:docs",
            ContextSubsearchSource::BrowserHistory,
        )];
    }

    let options = crate::browser_history::RootBrowserHistorySectionOptions {
        enabled: true,
        max_results: SUBSEARCH_RENDER_LIMIT,
        min_query_chars: 0,
        ..Default::default()
    };

    crate::browser_history::search_root_browser_history_meta_direct(query, options)
        .into_iter()
        .enumerate()
        .map(|(rank, hit)| {
            let ref_text = format!("@browser-history:{}", escape_ref_component(&hit.stable_key));
            let title = if hit.title.trim().is_empty() {
                hit.domain.clone()
            } else {
                hit.title.clone()
            };
            context_result_row(
                ContextSubsearchSource::BrowserHistory,
                segment_index,
                segment_byte_range.clone(),
                hit.stable_key,
                title,
                format!("{} · {}", hit.domain, hit.provider_label),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect()
}

// --- Dictation history provider ---

fn build_dictation_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    let options = crate::dictation::RootDictationHistorySectionOptions {
        enabled: true,
        max_results: SUBSEARCH_RENDER_LIMIT,
        min_query_chars: 0,
        ..Default::default()
    };

    crate::dictation::search_root_dictation_history_direct(query, options)
        .into_iter()
        .enumerate()
        .map(|(rank, hit)| {
            let title = single_line_truncate(&hit.preview, 72);
            let ref_text = format!("@dictation:{}", escape_ref_component(&hit.id));
            context_result_row(
                ContextSubsearchSource::Dictation,
                segment_index,
                segment_byte_range.clone(),
                format!("dictation/{}", hit.id),
                title,
                format!("Dictation · {}", hit.target),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect()
}

// --- Notes provider ---

fn build_notes_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    let options = crate::notes::RootNotesSectionOptions {
        enabled: true,
        max_results: SUBSEARCH_RENDER_LIMIT,
        min_query_chars: 0,
        ..Default::default()
    };

    crate::notes::search_root_notes_meta_direct(query, options)
        .into_iter()
        .enumerate()
        .map(|(rank, hit)| {
            let id_str = hit.id.to_string();
            let ref_text = format!("@notes:{}", escape_ref_component(&id_str));
            context_result_row(
                ContextSubsearchSource::Notes,
                segment_index,
                segment_byte_range.clone(),
                format!("notes/{}", id_str),
                hit.title.clone(),
                format!("{} chars", hit.char_count),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect()
}

// --- ACP history provider ---

fn build_acp_history_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Vec<SpineListRow> {
    crate::ai::acp::history::search_history_direct(query, SUBSEARCH_RENDER_LIMIT)
        .into_iter()
        .enumerate()
        .map(|(rank, hit)| {
            let entry = hit.entry;
            let ref_text = format!("@history:{}", escape_ref_component(&entry.session_id));
            context_result_row(
                ContextSubsearchSource::History,
                segment_index,
                segment_byte_range.clone(),
                format!("acp-history/{}", entry.session_id),
                entry.title_display().to_string(),
                format!("{} messages", entry.message_count),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect()
}

// --- Script/scriptlet/skill providers (need app state) ---

fn build_script_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    scripts: &[Arc<crate::scripts::Script>],
) -> Vec<SpineListRow> {
    let q = query.trim().to_lowercase();
    let mut rows: Vec<SpineListRow> = scripts
        .iter()
        .filter(|s| {
            q.is_empty()
                || s.name.to_lowercase().contains(&q)
                || s.path.to_string_lossy().to_lowercase().contains(&q)
        })
        .take(SUBSEARCH_RENDER_LIMIT)
        .enumerate()
        .map(|(rank, script)| {
            let cmd_id = script.launcher_command_id();
            let ref_text = format!("@scripts:{}", escape_ref_component(&cmd_id));
            context_result_row(
                ContextSubsearchSource::Scripts,
                segment_index,
                segment_byte_range.clone(),
                cmd_id,
                script.name.clone(),
                script.path.display().to_string(),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect();
    rows.truncate(SUBSEARCH_RENDER_LIMIT);
    rows
}

fn build_scriptlet_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> Vec<SpineListRow> {
    let q = query.trim().to_lowercase();
    scriptlets
        .iter()
        .filter(|s| q.is_empty() || s.name.to_lowercase().contains(&q))
        .take(SUBSEARCH_RENDER_LIMIT)
        .enumerate()
        .map(|(rank, scriptlet)| {
            let cmd_id = scriptlet.launcher_command_id();
            let ref_text = format!("@scriptlets:{}", escape_ref_component(&cmd_id));
            context_result_row(
                ContextSubsearchSource::Scriptlets,
                segment_index,
                segment_byte_range.clone(),
                cmd_id,
                scriptlet.name.clone(),
                scriptlet.description.as_deref().unwrap_or("").to_string(),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect()
}

fn build_skill_rows(
    query: &str,
    segment_index: usize,
    segment_byte_range: Range<usize>,
    skills: &[Arc<crate::plugins::PluginSkill>],
) -> Vec<SpineListRow> {
    let q = query.trim().to_lowercase();
    skills
        .iter()
        .filter(|s| {
            q.is_empty()
                || s.title.to_lowercase().contains(&q)
                || s.skill_id.to_lowercase().contains(&q)
        })
        .take(SUBSEARCH_RENDER_LIMIT)
        .enumerate()
        .map(|(rank, skill)| {
            let stable_id = format!("{}:{}", skill.plugin_id, skill.skill_id);
            let ref_text = format!("@skills:{}", escape_ref_component(&stable_id));
            context_result_row(
                ContextSubsearchSource::Skills,
                segment_index,
                segment_byte_range.clone(),
                stable_id,
                skill.title.clone(),
                skill.description.clone(),
                ref_text,
                i32::MAX.saturating_sub(rank as i32),
            )
        })
        .collect()
}

// --- Helpers ---

fn hint_row(title: &str, subtitle: &str, source: ContextSubsearchSource) -> SpineListRow {
    SpineListRow {
        id: ss(format!("spine:@:subsearch-hint:{}", source.prefix())),
        kind: SpineListRowKind::Hint,
        title: SharedString::from(title.to_string()),
        subtitle: Some(SharedString::from(subtitle.to_string())),
        icon: Some(ss(source.icon())),
        meta: None,
        badges: vec![],
        score: 0,
        is_selectable: false,
        action_label: None,
        action: SpineListAction::Noop,
    }
}

pub(crate) fn escape_ref_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            ' ' => out.push_str("%20"),
            '\n' | '\r' | '\t' => out.push_str("%20"),
            '%' => out.push_str("%25"),
            '#' => out.push_str("%23"),
            '@' => out.push_str("%40"),
            _ => out.push(ch),
        }
    }
    out
}

fn single_line_truncate(input: &str, max_chars: usize) -> String {
    super::text_preview::single_line_truncate(input, max_chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_subsearch_prefixes() {
        let (source, query) = parse_context_subsearch("file", Some("readme")).unwrap();
        assert_eq!(source, ContextSubsearchSource::File);
        assert_eq!(query, "readme");
    }

    #[test]
    fn unknown_prefix_returns_none() {
        assert!(parse_context_subsearch("unknown", Some("foo")).is_none());
    }

    #[test]
    fn escape_ref_handles_special_chars() {
        assert_eq!(escape_ref_component("hello world"), "hello%20world");
        assert_eq!(escape_ref_component("a@b"), "a%40b");
        assert_eq!(escape_ref_component("50%"), "50%25");
    }
}
