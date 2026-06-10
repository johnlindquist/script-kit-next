//! Universal attach resolution for spine `@source:` subsearch rows.
//!
//! Every context subsearch source the spine advertises must keep the same
//! promise on Enter: the selected result resolves into a compact
//! `@source:label` token in the input plus a `spine_mention_aliases` entry
//! carrying the real content, so the prompt plan attaches it at submit.
//!
//! Before this module existed only File and Clipboard were intercepted —
//! Enter on a `@notes:` / `@scripts:` / `@browser-history:` row fell through
//! to default launcher execution (opening the note, running the script) and
//! destroyed the prompt being built.

use super::*;

use std::ops::Range;

use crate::ai::message_parts::AiContextPart;
use crate::scripts::SearchResult;
use crate::spine::catalog_subsearch::{escape_ref_component, ContextSubsearchSource};
use crate::spine::SpineListAction;

/// The result of intercepting Enter on a rich subsearch row: the segment
/// resolution action plus the alias to register (token → content part)
/// before the action is applied.
pub(crate) struct SpineAttachOutcome {
    pub action: SpineListAction,
    /// `None` for sources (file, clipboard) whose alias registration is
    /// owned by the `ResolveSegment` apply arm for parity with the
    /// attachment portal path.
    pub alias: Option<(String, AiContextPart)>,
}

/// Compact display token for a resolved subsearch result: `@prefix:value`
/// with whitespace/reserved characters escaped.
pub(crate) fn compact_subsearch_token(prefix: &str, value: &str) -> String {
    let compact = crate::spine::text_preview::single_line_truncate(value, 40);
    format!("@{}:{}", prefix, escape_ref_component(&compact))
}

fn resolve_action(
    segment_index: usize,
    segment_byte_range: Range<usize>,
    token: &str,
    resolution_id: String,
    resolution_label: String,
    resolution_source: &'static str,
) -> SpineListAction {
    SpineListAction::ResolveSegment {
        segment_index,
        segment_byte_range,
        replacement: token.to_string().into(),
        resolution_id: resolution_id.into(),
        resolution_label: resolution_label.into(),
        resolution_source: resolution_source.into(),
        trailing_space: true,
    }
}

fn text_block(label: String, source: String, text: String) -> AiContextPart {
    AiContextPart::TextBlock {
        label,
        source,
        text,
        mime_type: None,
    }
}

/// Map a selected rich subsearch result to its attach outcome.
/// Returns `None` only for result/source combinations that are not
/// attachable rows (section headers, hint rows, mismatched results); the
/// caller must still consume Enter so the launcher default never executes
/// a row while a `@source:` segment is active.
pub(crate) fn attach_outcome_for_result(
    source: ContextSubsearchSource,
    result: &SearchResult,
    segment_index: usize,
    segment_byte_range: Range<usize>,
) -> Option<SpineAttachOutcome> {
    match (source, result) {
        (ContextSubsearchSource::File, SearchResult::File(file_match)) => {
            // Alias registration owned by the ResolveSegment apply arm
            // ("file" source) for parity with the file-search portal.
            let token = ScriptListApp::spine_file_mention_token(&file_match.file.path);
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("file/{}", file_match.file.path),
                    file_match.file.name.clone(),
                    "file",
                ),
                alias: None,
            })
        }
        (ContextSubsearchSource::Clipboard, SearchResult::ClipboardHistory(clip_match)) => {
            let token = format!(
                "@clipboard:{}",
                escape_ref_component(&clip_match.entry.id)
            );
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("clipboard/{}", clip_match.entry.id),
                    clip_match.title.clone(),
                    "clipboard",
                ),
                alias: None,
            })
        }
        (ContextSubsearchSource::Notes, SearchResult::Note(note_match)) => {
            let hit = &note_match.hit;
            let token = compact_subsearch_token("notes", &hit.title);
            let content = crate::notes::get_note(hit.id)
                .ok()
                .flatten()
                .map(|note| note.content)
                .unwrap_or_default();
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("notes/{}", hit.id),
                    hit.title.clone(),
                    "notes",
                ),
                alias: Some((
                    token,
                    text_block(
                        hit.title.clone(),
                        format!("spine:notes:{}", hit.id),
                        content,
                    ),
                )),
            })
        }
        (ContextSubsearchSource::BrowserHistory, SearchResult::BrowserHistory(history_match)) => {
            let hit = &history_match.hit;
            let label = if hit.title.trim().is_empty() {
                hit.domain.clone()
            } else {
                hit.title.clone()
            };
            let token = compact_subsearch_token("browser-history", &label);
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("browser-history/{}", hit.stable_key),
                    label.clone(),
                    "browser-history",
                ),
                alias: Some((
                    token,
                    text_block(
                        label,
                        format!("spine:browser-history:{}", hit.stable_key),
                        format!("{}\n{}", hit.title, hit.url),
                    ),
                )),
            })
        }
        (ContextSubsearchSource::Dictation, SearchResult::DictationHistory(dictation_match)) => {
            let token = compact_subsearch_token("dictation", &dictation_match.preview);
            let transcript = crate::dictation::get_history_entry(&dictation_match.id)
                .map(|entry| entry.transcript)
                .unwrap_or_else(|| dictation_match.preview.clone());
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("dictation/{}", dictation_match.id),
                    dictation_match.preview.clone(),
                    "dictation",
                ),
                alias: Some((
                    token,
                    text_block(
                        dictation_match.preview.clone(),
                        format!("spine:dictation:{}", dictation_match.id),
                        transcript,
                    ),
                )),
            })
        }
        (ContextSubsearchSource::History, SearchResult::AgentChatHistory(history_match)) => {
            let entry = &history_match.entry;
            let title = entry.title_display().to_string();
            let token = compact_subsearch_token("history", &title);
            let text = format!(
                "Conversation: {title}\nFirst message: {}\nLast reply: {}",
                entry.first_message.trim(),
                entry.preview.trim(),
            );
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("history/{}", entry.session_id),
                    title.clone(),
                    "history",
                ),
                alias: Some((
                    token,
                    text_block(title, format!("spine:history:{}", entry.session_id), text),
                )),
            })
        }
        (ContextSubsearchSource::Scripts, SearchResult::Script(script_match)) => {
            let script = &script_match.script;
            let path = script.path.to_string_lossy().to_string();
            let token = compact_subsearch_token("scripts", &script_match.filename);
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("scripts/{path}"),
                    script.name.clone(),
                    "scripts",
                ),
                alias: Some((
                    token,
                    AiContextPart::FilePath {
                        path,
                        label: script_match.filename.clone(),
                    },
                )),
            })
        }
        (ContextSubsearchSource::Scriptlets, SearchResult::Scriptlet(scriptlet_match)) => {
            let scriptlet = &scriptlet_match.scriptlet;
            let token = compact_subsearch_token("scriptlets", &scriptlet.name);
            let source_id = scriptlet
                .file_path
                .clone()
                .unwrap_or_else(|| scriptlet.name.clone());
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("scriptlets/{source_id}"),
                    scriptlet.name.clone(),
                    "scriptlets",
                ),
                alias: Some((
                    token,
                    text_block(
                        scriptlet.name.clone(),
                        format!("spine:scriptlets:{source_id}"),
                        scriptlet.code.clone(),
                    ),
                )),
            })
        }
        (ContextSubsearchSource::Skills, SearchResult::Skill(skill_match)) => {
            let skill = &skill_match.skill;
            let path = skill.path.to_string_lossy().to_string();
            let token = compact_subsearch_token("skills", &skill.title);
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("skills/{path}"),
                    skill.title.clone(),
                    "skills",
                ),
                alias: Some((
                    token,
                    AiContextPart::FilePath {
                        path,
                        label: skill.title.clone(),
                    },
                )),
            })
        }
        (
            ContextSubsearchSource::Calendar | ContextSubsearchSource::Notifications,
            SearchResult::SpineProjection(row),
        ) => {
            // Provider-JSON rows carry their content in title/subtitle.
            let prefix = source.prefix();
            let title = row.title.to_string();
            let token = compact_subsearch_token(prefix, &title);
            let text = match row.subtitle.as_ref() {
                Some(subtitle) => format!("{title}\n{subtitle}"),
                None => title.clone(),
            };
            Some(SpineAttachOutcome {
                action: resolve_action(
                    segment_index,
                    segment_byte_range,
                    &token,
                    format!("{prefix}/{}", row.id),
                    title.clone(),
                    match source {
                        ContextSubsearchSource::Calendar => "calendar",
                        _ => "notifications",
                    },
                ),
                alias: Some((
                    token,
                    text_block(title, format!("spine:{prefix}:{}", row.id), text),
                )),
            })
        }
        // SpineProjection utility rows (Open full File Search, hints, empty
        // guards) keep their own action — let the normal row accept run it.
        (_, SearchResult::SpineProjection(row)) => {
            if row.is_selectable {
                Some(SpineAttachOutcome {
                    action: row.action.clone(),
                    alias: None,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_token_escapes_and_truncates() {
        let token = compact_subsearch_token("notes", "grocery list for the week");
        assert_eq!(token, "@notes:grocery%20list%20for%20the%20week");
        let long = "x".repeat(120);
        let token = compact_subsearch_token("notes", &long);
        assert!(token.chars().count() <= "@notes:".len() + 40 + 1);
    }

    #[test]
    fn scriptlet_attach_resolves_to_text_block_with_code() {
        let scriptlet = std::sync::Arc::new(crate::scripts::Scriptlet {
            name: "Open GitHub".to_string(),
            description: None,
            code: "open https://github.com".to_string(),
            tool: "bash".to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            plugin_id: "main".to_string(),
            plugin_title: None,
            file_path: Some("/tmp/url.md#open-github".to_string()),
            command: Some("open-github".to_string()),
            alias: None,
        });
        let result = SearchResult::Scriptlet(crate::scripts::ScriptletMatch {
            scriptlet,
            score: 0,
            display_file_path: None,
            match_indices: Default::default(),
            match_evidence: None,
        });
        let outcome = attach_outcome_for_result(
            ContextSubsearchSource::Scriptlets,
            &result,
            0,
            0..12,
        )
        .expect("scriptlet rows must attach");
        let (token, part) = outcome.alias.expect("scriptlet alias");
        assert!(token.starts_with("@scriptlets:"));
        match part {
            AiContextPart::TextBlock { text, .. } => {
                assert_eq!(text, "open https://github.com");
            }
            other => panic!("expected TextBlock, got {other:?}"),
        }
        match outcome.action {
            SpineListAction::ResolveSegment {
                resolution_source, ..
            } => assert_eq!(resolution_source.as_ref(), "scriptlets"),
            other => panic!("expected ResolveSegment, got {other:?}"),
        }
    }

    #[test]
    fn browser_history_attach_includes_url_in_text() {
        let result = SearchResult::BrowserHistory(crate::scripts::BrowserHistoryMatch {
            hit: crate::browser_history::RootBrowserHistorySearchHit {
                stable_key: "abc".to_string(),
                provider_label: "Chrome".to_string(),
                profile_label: "Default".to_string(),
                title: "Rust Book".to_string(),
                url: "https://doc.rust-lang.org/book/".to_string(),
                domain: "doc.rust-lang.org".to_string(),
                last_visit_unix_ms: 0,
                visit_count: 3,
            },
            subtitle: "https://doc.rust-lang.org/book/".to_string(),
            score: 0,
        });
        let outcome = attach_outcome_for_result(
            ContextSubsearchSource::BrowserHistory,
            &result,
            0,
            0..20,
        )
        .expect("browser history rows must attach");
        let (token, part) = outcome.alias.expect("browser alias");
        assert!(token.starts_with("@browser-history:"));
        match part {
            AiContextPart::TextBlock { text, .. } => {
                assert!(text.contains("https://doc.rust-lang.org/book/"));
            }
            other => panic!("expected TextBlock, got {other:?}"),
        }
    }

    #[test]
    fn provider_json_row_attaches_title_and_subtitle() {
        use crate::spine::list::ss;
        let row = crate::spine::SpineListRow {
            id: ss("spine:provider-json:calendar:0"),
            kind: crate::spine::SpineListRowKind::ContextResult {
                context_type: ss("calendar"),
                result_id: ss("0"),
            },
            title: ss("Standup"),
            subtitle: Some(ss("9:30 AM · Daily")),
            meta: None,
            icon: Some(ss("calendar")),
            badges: vec![],
            score: 0,
            is_selectable: true,
            action_label: Some(ss("Attach")),
            action: SpineListAction::Noop,
        };
        let result = SearchResult::SpineProjection(row);
        let outcome =
            attach_outcome_for_result(ContextSubsearchSource::Calendar, &result, 0, 0..10)
                .expect("calendar rows must attach");
        let (token, part) = outcome.alias.expect("calendar alias");
        assert!(token.starts_with("@calendar:Standup"));
        match part {
            AiContextPart::TextBlock { text, .. } => {
                assert_eq!(text, "Standup\n9:30 AM · Daily");
            }
            other => panic!("expected TextBlock, got {other:?}"),
        }
        // The interception action must override the row's Noop.
        assert!(matches!(
            outcome.action,
            SpineListAction::ResolveSegment { .. }
        ));
    }
}
