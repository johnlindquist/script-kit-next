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
    Calendar,
    Notifications,
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
            "calendar" => Some(Self::Calendar),
            "notifications" => Some(Self::Notifications),
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
            Self::Calendar => "calendar",
            Self::Notifications => "notifications",
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
            Self::Calendar => "Calendar Events",
            Self::Notifications => "Notifications",
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
            Self::Calendar => "calendar",
            Self::Notifications => "bell",
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

pub(crate) fn build_context_subsearch_section(
    source: ContextSubsearchSource,
    query: &str,
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
        ContextSubsearchSource::Calendar | ContextSubsearchSource::Notifications => vec![hint_row(
            "Loading\u{2026}",
            "Results are loaded by the launcher",
            source,
        )],
    };

    SpineListSection {
        id: ss(format!("spine-section-subsearch:{}", source.prefix())),
        title: ss(source.section_title()),
        subtitle: Some(ss(format!("@{}:", source.prefix()))),
        icon: Some(ss(source.icon())),
        rows,
    }
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
