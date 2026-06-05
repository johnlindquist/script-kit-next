use std::ops::Range;

use crate::ai::message_parts::AiContextPart;
use crate::ai::window::context_picker::types::PortalKind;

const PREVIEW_TARGET_MAX_CHARS: usize = 48;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcpPortalReplacementTarget {
    ExactToken {
        char_range: Range<usize>,
        original_text: String,
        fallback_cursor: usize,
    },
    AppendAtCursor {
        cursor: usize,
    },
}

impl AcpPortalReplacementTarget {
    pub(crate) fn preview_label(&self) -> String {
        match self {
            Self::ExactToken { original_text, .. } => {
                format!("replaces {}", compact_preview_target_text(original_text))
            }
            Self::AppendAtCursor { .. } => "adds a new mention".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AcpPortalLaunchContract {
    pub portal_kind: PortalKind,
    pub query: String,
    pub replacement: AcpPortalReplacementTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcpPortalSessionState {
    Idle,
    Staged,
    Active,
    Accepted,
    Cancelled,
    Orphaned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcpPortalSessionEvent {
    Stage,
    Activate,
    Accept,
    Cancel,
    Orphan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcpPortalOpenRefusal {
    UnsupportedByHost,
    MissingHostCallback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AcpPortalIntent {
    Portal(AcpPortalLaunchContract),
    PreviewOnly {
        description: String,
        replacement: AcpPortalReplacementTarget,
    },
}

pub(crate) fn portal_kind_detail_label(kind: PortalKind) -> &'static str {
    match kind {
        PortalKind::FileSearch => "file portal",
        PortalKind::BrowserHistory => "browser history portal",
        PortalKind::BrowserTabs => "browser tabs portal",
        PortalKind::ClipboardHistory => "clipboard portal",
        PortalKind::DictationHistory => "dictation portal",
        PortalKind::ScriptSearch => "script portal",
        PortalKind::ScriptletSearch => "scriptlet portal",
        PortalKind::SkillSearch => "skill portal",
        PortalKind::NotesBrowse => "notes portal",
        PortalKind::AcpHistory => "history portal",
        PortalKind::Terminal => "terminal portal",
    }
}

fn is_fileish_typed_prefix(prefix: &str) -> bool {
    matches!(
        prefix,
        "file"
            | "dir"
            | "rs"
            | "ts"
            | "js"
            | "py"
            | "rb"
            | "go"
            | "java"
            | "swift"
            | "c"
            | "cpp"
            | "md"
            | "json"
            | "toml"
            | "yaml"
            | "xml"
            | "html"
            | "css"
            | "sh"
            | "img"
            | "vid"
            | "audio"
            | "sql"
            | "txt"
    )
}

fn compact_preview_target_text(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = collapsed.chars().take(PREVIEW_TARGET_MAX_CHARS).collect();
    if collapsed.chars().count() > PREVIEW_TARGET_MAX_CHARS {
        out.push('\u{2026}');
    }
    out
}

fn preview_only_inline_token_description(token: &str) -> Option<String> {
    if let Some(description) = crate::pasted_text::preview_description_for_token(token) {
        return Some(description);
    }
    if crate::pasted_image::token_looks_like_pasted_image(token) {
        return Some("Pasted image attachment".to_string());
    }
    None
}

fn preview_only_part_description(part: &AiContextPart) -> Option<String> {
    match part {
        AiContextPart::FilePath { label, .. }
            if crate::pasted_image::label_looks_like_pasted_image(label) =>
        {
            Some("Pasted image attachment".to_string())
        }
        _ => None,
    }
}

pub(crate) fn portal_target_from_part(part: &AiContextPart) -> Option<(PortalKind, String)> {
    match part {
        AiContextPart::ResourceUri { uri, label } => match uri.as_str() {
            "kit://clipboard-history" => Some((PortalKind::ClipboardHistory, label.clone())),
            "kit://dictation" => Some((PortalKind::DictationHistory, String::new())),
            "kit://scripts" => Some((PortalKind::ScriptSearch, label.clone())),
            _ if uri.starts_with("kit://clipboard-history?id=") => Some((
                PortalKind::ClipboardHistory,
                label
                    .strip_prefix("Clipboard: ")
                    .map(str::to_string)
                    .unwrap_or_else(|| label.clone()),
            )),
            _ if uri.starts_with("kit://dictation-history?id=") => {
                let query = uri
                    .strip_prefix("kit://dictation-history?id=")
                    .and_then(crate::dictation::get_history_entry)
                    .map(|entry| entry.preview)
                    .unwrap_or_else(|| label.clone());
                Some((PortalKind::DictationHistory, query))
            }
            _ => None,
        },
        AiContextPart::FilePath { label, .. } => {
            if crate::pasted_image::label_looks_like_pasted_image(label) {
                tracing::info!(
                    target: "script_kit::acp",
                    event = "acp_part_forced_preview_only",
                    reason = "pasted_image",
                    label = %label,
                );
                return None;
            }
            Some((PortalKind::FileSearch, label.clone()))
        }
        AiContextPart::SkillFile {
            label, skill_name, ..
        } => Some((
            PortalKind::SkillSearch,
            if skill_name.trim().is_empty() {
                label.clone()
            } else {
                skill_name.clone()
            },
        )),
        AiContextPart::FocusedTarget { target, .. } => match target.kind.as_str() {
            "script" => Some((PortalKind::ScriptSearch, target.label.clone())),
            "scriptlet" => Some((PortalKind::ScriptletSearch, target.label.clone())),
            "note" => Some((PortalKind::NotesBrowse, target.label.clone())),
            "browser_history_entry" => Some((PortalKind::BrowserHistory, target.label.clone())),
            "browser_tab" => Some((PortalKind::BrowserTabs, target.label.clone())),
            "clipboard_entry" => Some((PortalKind::ClipboardHistory, target.label.clone())),
            "skill" => Some((PortalKind::SkillSearch, target.label.clone())),
            "file" | "directory" => Some((
                PortalKind::FileSearch,
                target
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get("path"))
                    .and_then(|path| path.as_str())
                    .and_then(|path| {
                        std::path::Path::new(path)
                            .file_name()
                            .and_then(|name| name.to_str())
                            .map(ToString::to_string)
                    })
                    .unwrap_or_else(|| target.label.clone()),
            )),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn portal_target_from_inline_token(token: &str) -> Option<(PortalKind, String)> {
    if preview_only_inline_token_description(token).is_some() {
        return None;
    }

    match token {
        "@browser-history" => return Some((PortalKind::BrowserHistory, String::new())),
        "@tabs" | "@browser-tabs" => return Some((PortalKind::BrowserTabs, String::new())),
        "@clipboard" => return Some((PortalKind::ClipboardHistory, String::new())),
        "@dictation" => return Some((PortalKind::DictationHistory, String::new())),
        "@recent-scripts" => return Some((PortalKind::ScriptSearch, String::new())),
        "@terminal" => return Some((PortalKind::Terminal, String::new())),
        _ => {}
    }

    let (prefix, value) = crate::ai::context_mentions::typed_mention_token_parts(token)?;
    let value = value.trim().to_string();
    let kind = match prefix.as_str() {
        "browser-history" => PortalKind::BrowserHistory,
        "tabs" | "browser-tabs" => PortalKind::BrowserTabs,
        "dictation" => PortalKind::DictationHistory,
        "note" => PortalKind::NotesBrowse,
        "script" => PortalKind::ScriptSearch,
        "scriptlet" => PortalKind::ScriptletSearch,
        "skill" => PortalKind::SkillSearch,
        "clipboard" => PortalKind::ClipboardHistory,
        "history" => PortalKind::AcpHistory,
        "terminal" => PortalKind::Terminal,
        file_prefix if is_fileish_typed_prefix(file_prefix) => PortalKind::FileSearch,
        _ => return None,
    };
    let query = if kind == PortalKind::DictationHistory {
        crate::dictation::get_history_entry(&value)
            .map(|entry| entry.preview)
            .unwrap_or(value)
    } else {
        value
    };
    Some((kind, query))
}

pub(crate) fn picker_portal_query(portal_kind: PortalKind, session_query: &str) -> String {
    if portal_kind == PortalKind::DictationHistory {
        String::new()
    } else {
        session_query.to_string()
    }
}

pub(crate) fn decide_portal_open(
    is_allowed: bool,
    has_host_callback: bool,
) -> Result<(), AcpPortalOpenRefusal> {
    if !is_allowed {
        return Err(AcpPortalOpenRefusal::UnsupportedByHost);
    }
    if !has_host_callback {
        return Err(AcpPortalOpenRefusal::MissingHostCallback);
    }
    Ok(())
}

pub(crate) fn next_portal_state(
    state: AcpPortalSessionState,
    event: AcpPortalSessionEvent,
) -> Option<AcpPortalSessionState> {
    use AcpPortalSessionEvent as Event;
    use AcpPortalSessionState as State;

    match (state, event) {
        (State::Idle, Event::Stage) => Some(State::Staged),
        (State::Staged, Event::Activate) => Some(State::Active),
        (State::Active, Event::Accept) => Some(State::Accepted),
        (State::Active, Event::Cancel) => Some(State::Cancelled),
        (State::Active, Event::Orphan) => Some(State::Orphaned),
        _ => None,
    }
}

pub(crate) fn clear_terminal_portal_state(state: AcpPortalSessionState) -> AcpPortalSessionState {
    match state {
        AcpPortalSessionState::Accepted
        | AcpPortalSessionState::Cancelled
        | AcpPortalSessionState::Orphaned => AcpPortalSessionState::Idle,
        other => other,
    }
}

pub(crate) fn exact_replacement_target_for_range(
    current_text: &str,
    char_range: Range<usize>,
    fallback_cursor: usize,
) -> AcpPortalReplacementTarget {
    AcpPortalReplacementTarget::ExactToken {
        original_text: text_in_char_range(current_text, char_range.clone()),
        char_range,
        fallback_cursor,
    }
}

pub(crate) fn intent_from_part(
    part: &AiContextPart,
    replacement: AcpPortalReplacementTarget,
) -> AcpPortalIntent {
    if let Some((portal_kind, query)) = portal_target_from_part(part) {
        AcpPortalIntent::Portal(AcpPortalLaunchContract {
            portal_kind,
            query,
            replacement,
        })
    } else {
        let description = preview_only_part_description(part).unwrap_or_else(|| {
            crate::ai::window::context_preview::derive_context_preview_info(part).description
        });
        AcpPortalIntent::PreviewOnly {
            description,
            replacement,
        }
    }
}

pub(crate) fn intent_from_inline_token(
    token: &str,
    replacement: AcpPortalReplacementTarget,
) -> Option<AcpPortalIntent> {
    if let Some(description) = preview_only_inline_token_description(token) {
        tracing::info!(
            target: "script_kit::acp",
            event = "acp_inline_token_forced_preview_only",
            token = %token,
            description = %description,
        );
        return Some(AcpPortalIntent::PreviewOnly {
            description,
            replacement,
        });
    }

    let (portal_kind, query) = portal_target_from_inline_token(token)?;
    Some(AcpPortalIntent::Portal(AcpPortalLaunchContract {
        portal_kind,
        query,
        replacement,
    }))
}

pub(crate) fn format_intent_preview(intent: &AcpPortalIntent) -> String {
    match intent {
        AcpPortalIntent::Portal(contract) => {
            let query_hint = if contract.query.trim().is_empty() {
                String::new()
            } else {
                format!(" for \"{}\"", contract.query.trim())
            };
            format!(
                "{}{} • {} • Cmd+. / Cmd+Shift+O",
                portal_kind_detail_label(contract.portal_kind),
                query_hint,
                contract.replacement.preview_label(),
            )
        }
        AcpPortalIntent::PreviewOnly {
            description,
            replacement,
        } => format!(
            "{description} • {} • preview only",
            replacement.preview_label()
        ),
    }
}

fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
    text.char_indices()
        .nth(char_offset)
        .map(|(offset, _)| offset)
        .unwrap_or(text.len())
}

fn text_in_char_range(text: &str, char_range: Range<usize>) -> String {
    let start = char_to_byte_offset(text, char_range.start);
    let end = char_to_byte_offset(text, char_range.end);
    text[start..end].to_string()
}

pub(crate) fn apply_portal_replacement(
    current_text: &str,
    target: &AcpPortalReplacementTarget,
    replacement_text: &str,
) -> (String, usize, bool) {
    match target {
        AcpPortalReplacementTarget::ExactToken {
            char_range,
            original_text,
            fallback_cursor,
        } => {
            let current_segment = text_in_char_range(current_text, char_range.clone());
            if current_segment == *original_text {
                let next_text = crate::ai::context_mentions::replace_text_in_char_range(
                    current_text,
                    char_range.clone(),
                    replacement_text,
                );
                let next_cursor = crate::ai::context_mentions::caret_after_replacement(
                    char_range,
                    replacement_text,
                );
                (next_text, next_cursor, true)
            } else {
                let cursor = (*fallback_cursor).min(current_text.chars().count());
                let next_text = crate::ai::context_mentions::replace_text_in_char_range(
                    current_text,
                    cursor..cursor,
                    replacement_text,
                );
                let next_cursor = cursor + replacement_text.chars().count();
                (next_text, next_cursor, false)
            }
        }
        AcpPortalReplacementTarget::AppendAtCursor { cursor } => {
            let cursor = (*cursor).min(current_text.chars().count());
            let next_text = crate::ai::context_mentions::replace_text_in_char_range(
                current_text,
                cursor..cursor,
                replacement_text,
            );
            let next_cursor = cursor + replacement_text.chars().count();
            (next_text, next_cursor, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_only_when_the_current_text_matches_the_original_token() {
        let target = AcpPortalReplacementTarget::ExactToken {
            char_range: 8..20,
            original_text: "@file:foo.rs".to_string(),
            fallback_cursor: 8,
        };

        let (next_text, next_cursor, exact_match) =
            apply_portal_replacement("compare @file:foo.rs", &target, "@file:bar.rs ");

        assert!(exact_match);
        assert_eq!(next_text, "compare @file:bar.rs ");
        assert_eq!(next_cursor, "compare @file:bar.rs ".chars().count());
    }

    #[test]
    fn falls_back_when_the_token_changed() {
        let target = AcpPortalReplacementTarget::ExactToken {
            char_range: 8..20,
            original_text: "@file:foo.rs".to_string(),
            fallback_cursor: 8,
        };

        let (next_text, next_cursor, exact_match) =
            apply_portal_replacement("compare source and tests", &target, "@file:bar.rs ");

        assert!(!exact_match);
        assert_eq!(next_text, "compare @file:bar.rs source and tests");
        assert_eq!(next_cursor, "compare @file:bar.rs ".chars().count());
    }

    #[test]
    fn portal_preview_names_the_replacement_target() {
        let intent = intent_from_inline_token(
            "@browser-history:github",
            AcpPortalReplacementTarget::ExactToken {
                char_range: 0..23,
                original_text: "@browser-history:github".to_string(),
                fallback_cursor: 0,
            },
        )
        .expect("browser history token should produce a portal intent");

        assert_eq!(
            format_intent_preview(&intent),
            "browser history portal for \"github\" • replaces @browser-history:github • Cmd+. / Cmd+Shift+O"
        );
    }

    #[test]
    fn preview_only_mentions_stay_explicit() {
        let part = AiContextPart::TextBlock {
            label: "Pasted text".to_string(),
            source: "paste://text".to_string(),
            text: "hello".to_string(),
            mime_type: Some("text/plain".to_string()),
        };
        let intent = intent_from_part(
            &part,
            AcpPortalReplacementTarget::ExactToken {
                char_range: 0..12,
                original_text: "@paste:text1".to_string(),
                fallback_cursor: 0,
            },
        );

        assert_eq!(
            format_intent_preview(&intent),
            "Text block (text/plain, 5 B) • replaces @paste:text1 • preview only"
        );
    }

    #[test]
    fn pasted_image_alias_tokens_stay_preview_only() {
        let intent = intent_from_inline_token(
            "@img:paste1",
            AcpPortalReplacementTarget::ExactToken {
                char_range: 0..11,
                original_text: "@img:paste1".to_string(),
                fallback_cursor: 0,
            },
        )
        .expect("pasted image tokens should still render a preview");

        assert_eq!(
            format_intent_preview(&intent),
            "Pasted image attachment • replaces @img:paste1 • preview only"
        );
    }

    #[test]
    fn pasted_image_parts_stay_preview_only() {
        let intent = intent_from_part(
            &AiContextPart::FilePath {
                path: "/tmp/script-kit-pasted-image-1.png".to_string(),
                label: "Pasted image #1".to_string(),
            },
            AcpPortalReplacementTarget::ExactToken {
                char_range: 0..11,
                original_text: "@img:paste1".to_string(),
                fallback_cursor: 0,
            },
        );

        assert_eq!(
            format_intent_preview(&intent),
            "Pasted image attachment • replaces @img:paste1 • preview only"
        );
    }

    #[test]
    fn preview_labels_compact_long_replacement_targets() {
        let label = AcpPortalReplacementTarget::ExactToken {
            char_range: 0..85,
            original_text:
                "@file:\"/Users/me/dev/script-kit-gpui/src/components/some_really_long_file_name.rs\""
                    .to_string(),
            fallback_cursor: 0,
        }
        .preview_label();

        assert!(label.starts_with("replaces @file:\"/Users/me/dev/script-kit-gpui/src/"));
        assert!(label.ends_with('…'));
    }
}
