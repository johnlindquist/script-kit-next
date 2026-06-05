//
// Pure decision module: given the live menu-syntax state and a Cmd+K action
// kind, return the side-effect the launcher should perform. Implements the
// four "low-risk" effects from the cmdk-safe-action-effects story
// (Cancel, Copy filter expression, Default time, Edit command argv) and
// returns `Unsupported` for everything else so the actions-dialog can fall
// back without surprise.

use crate::menu_syntax::actions::{MenuSyntaxActionKind, MenuSyntaxActionState};
use crate::menu_syntax::snippet_scriptlet::{parse_snippet_scriptlet_capture, SnippetLookup};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionEffect {
    /// Close the actions dialog and the menu-syntax composer without saving.
    Cancel,
    /// Replace the launcher's filter text with a new value.
    SetFilterText { new_text: String },
    /// Write a string to the system clipboard.
    WriteClipboard { content: String },
    /// The action kind is not in this pass's safe-effects allowlist; the
    /// dialog should ignore the row or surface a "not implemented" HUD.
    Unsupported,
}

pub fn apply_safe_effect(
    state: &MenuSyntaxActionState<'_>,
    kind: &MenuSyntaxActionKind,
) -> ActionEffect {
    use MenuSyntaxActionKind::*;

    match (state, kind) {
        // Cancel applies in every state.
        (_, Cancel) => ActionEffect::Cancel,

        // Copy filter expression — reconstruct from each state's source-of-truth.
        (MenuSyntaxActionState::CaptureComposer { payload, .. }, CopyFilterExpression) => {
            ActionEffect::WriteClipboard {
                content: payload.raw.clone(),
            }
        }
        (MenuSyntaxActionState::RefineQuery { query }, CopyFilterExpression) => {
            ActionEffect::WriteClipboard {
                content: query.raw.clone(),
            }
        }
        (MenuSyntaxActionState::CommandComposer { head, argv }, CopyFilterExpression) => {
            ActionEffect::WriteClipboard {
                content: format_command_filter(head, argv),
            }
        }

        // Default time — only applies in capture composer; appends `start:"<phrase>"`
        // to the raw filter expression. Receipt example:
        //   ";cal Design review" + DefaultTime { phrase: "today 9am" }
        //   → ";cal Design review start:\"today 9am\""
        (MenuSyntaxActionState::CaptureComposer { payload, .. }, DefaultTime { phrase }) => {
            let trimmed = payload.raw.trim_end();
            let new_text = format!(
                "{trimmed} start:{}",
                crate::menu_syntax::quote_for_filter_value(phrase)
            );
            ActionEffect::SetFilterText { new_text }
        }

        // Edit command argv — reset the filter text to just the head + trailing
        // space so the user types argv from a clean cursor position.
        (MenuSyntaxActionState::CommandComposer { head, .. }, EditCommandArgv) => {
            ActionEffect::SetFilterText {
                new_text: format!("!{head} "),
            }
        }

        (MenuSyntaxActionState::CaptureComposer { payload, .. }, SnippetInsertField { key })
            if payload.target.eq_ignore_ascii_case("snippet") =>
        {
            ActionEffect::SetFilterText {
                new_text: insert_snippet_metadata_field(&payload.raw, key),
            }
        }

        (MenuSyntaxActionState::CaptureComposer { payload, .. }, SnippetCopyGeneratedMarkdown)
            if payload.target.eq_ignore_ascii_case("snippet") =>
        {
            let content = parse_snippet_scriptlet_capture(payload)
                .and_then(|draft| {
                    crate::scriptlets::snippet_markdown_store::render_snippet_draft_markdown_preview(
                        &draft,
                    )
                })
                .unwrap_or_else(|_| payload.raw.clone());
            ActionEffect::WriteClipboard { content }
        }

        (MenuSyntaxActionState::CaptureComposer { payload, .. }, SnippetCopyMarkdownPath)
            if payload.target.eq_ignore_ascii_case("snippet") =>
        {
            ActionEffect::WriteClipboard {
                content: crate::scriptlets::snippet_markdown_store::default_snippets_markdown_path(
                )
                .display()
                .to_string(),
            }
        }

        (MenuSyntaxActionState::CaptureComposer { payload, .. }, SnippetPrepareUpdateSelected)
            if payload.target.eq_ignore_ascii_case("snippet") =>
        {
            snippet_prepare_selected_effect(payload, "update")
        }

        (MenuSyntaxActionState::CaptureComposer { payload, .. }, SnippetPrepareDeleteSelected)
            if payload.target.eq_ignore_ascii_case("snippet") =>
        {
            snippet_prepare_selected_effect(payload, "delete")
        }

        _ => ActionEffect::Unsupported,
    }
}

fn format_command_filter(head: &str, argv: &[String]) -> String {
    let mut out = format!("!{head}");
    for a in argv {
        out.push(' ');
        out.push_str(a);
    }
    out
}

fn insert_snippet_metadata_field(raw: &str, key: &str) -> String {
    let trimmed = raw.trim_end();
    let token = format!("{}:", key.trim());
    if token == ":" {
        return trimmed.to_string();
    }
    if let Some(idx) = find_double_dash_token(trimmed) {
        let (before, after) = trimmed.split_at(idx);
        let before = before.trim_end();
        if before.is_empty() {
            format!("{token} {after}")
        } else {
            format!("{before} {token} {after}")
        }
    } else if trimmed.is_empty() {
        token
    } else {
        format!("{trimmed} {token}")
    }
}

fn snippet_prepare_selected_effect(
    payload: &crate::menu_syntax::CaptureInvocation,
    operation: &str,
) -> ActionEffect {
    let Ok(draft) = parse_snippet_scriptlet_capture(payload) else {
        return ActionEffect::Unsupported;
    };
    let Some(SnippetLookup::SelectedRef(id)) = draft.lookup.as_ref() else {
        return ActionEffect::Unsupported;
    };
    let id = id.trim();
    if id.is_empty() {
        return ActionEffect::Unsupported;
    }
    if operation == "delete" {
        return ActionEffect::SetFilterText {
            new_text: format!(";snippet delete @snippet:{id}"),
        };
    }

    let mut parts = vec![format!(";snippet {operation} @snippet:{id}")];
    parts.extend(snippet_metadata_tokens(&draft.metadata));
    if let Some(body) = draft
        .body
        .as_deref()
        .map(str::trim)
        .filter(|body| !body.is_empty())
    {
        parts.push("--".to_string());
        parts.push(body.to_string());
    }
    ActionEffect::SetFilterText {
        new_text: parts.join(" "),
    }
}

fn snippet_metadata_tokens(metadata: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    metadata
        .iter()
        .filter_map(|(key, value)| snippet_metadata_token(key, value))
        .collect()
}

fn snippet_metadata_token(key: &str, value: &serde_json::Value) -> Option<String> {
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    let value = match value {
        serde_json::Value::String(value) => value.trim().to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Array(values) => values
            .iter()
            .filter_map(|value| value.as_str().map(str::trim))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(","),
        _ => return None,
    };
    if value.is_empty() {
        return None;
    }
    Some(format!(
        "{key}:{}",
        crate::menu_syntax::quote_for_filter_value(&value)
    ))
}

fn find_double_dash_token(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'-'
            && bytes[i + 1] == b'-'
            && (i == 0 || bytes[i - 1].is_ascii_whitespace())
            && (i + 2 == bytes.len() || bytes[i + 2].is_ascii_whitespace())
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::actions::MenuSyntaxActionState;
    use crate::menu_syntax::payload::{AdvancedQuery, CaptureAlias, CaptureInvocation};

    fn capture(raw: &str) -> CaptureInvocation {
        capture_for_target(raw, "cal")
    }

    fn capture_for_target(raw: &str, target: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: target.to_string(),
            alias_form: CaptureAlias::CapturePrefix,
            body: "Design review".to_string(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: raw.to_string(),
        }
    }

    fn refine(raw: &str) -> AdvancedQuery {
        AdvancedQuery {
            free_text: String::new(),
            predicates: vec![],
            source_filters: Default::default(),
            raw: raw.to_string(),
        }
    }

    #[test]
    fn cancel_in_capture_state_returns_cancel() {
        let inv = capture(";cal Design review");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::Cancel),
            ActionEffect::Cancel
        );
    }

    #[test]
    fn cancel_in_refine_state_returns_cancel() {
        let q = refine(":foo");
        let state = MenuSyntaxActionState::RefineQuery { query: &q };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::Cancel),
            ActionEffect::Cancel
        );
    }

    #[test]
    fn cancel_in_command_state_returns_cancel() {
        let argv: Vec<String> = vec!["--foo".into()];
        let state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::Cancel),
            ActionEffect::Cancel
        );
    }

    #[test]
    fn copy_filter_in_capture_state_writes_payload_raw() {
        let inv = capture(";cal Design review start:\"today 9am\"");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
            ActionEffect::WriteClipboard {
                content: ";cal Design review start:\"today 9am\"".to_string(),
            }
        );
    }

    #[test]
    fn copy_filter_in_refine_state_writes_query_raw() {
        let q = refine(":kit#tags#urgent");
        let state = MenuSyntaxActionState::RefineQuery { query: &q };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
            ActionEffect::WriteClipboard {
                content: ":kit#tags#urgent".to_string(),
            }
        );
    }

    #[test]
    fn copy_filter_in_command_state_reconstructs_head_plus_argv() {
        let argv: Vec<String> = vec!["--prod".into(), "--dry-run".into()];
        let state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::CopyFilterExpression),
            ActionEffect::WriteClipboard {
                content: "!deploy --prod --dry-run".to_string(),
            }
        );
    }

    #[test]
    fn default_time_appends_start_phrase_per_story_receipt() {
        // Story receipt example: ";cal Design review" + DefaultTime("today 9am")
        // → filterText:";cal Design review start:\"today 9am\""
        let inv = capture(";cal Design review");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        let kind = MenuSyntaxActionKind::DefaultTime {
            phrase: "today 9am".to_string(),
        };
        assert_eq!(
            apply_safe_effect(&state, &kind),
            ActionEffect::SetFilterText {
                new_text: ";cal Design review start:\"today 9am\"".to_string(),
            }
        );
    }

    #[test]
    fn default_time_trims_trailing_whitespace_before_appending() {
        let inv = capture(";cal Design review   ");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        let kind = MenuSyntaxActionKind::DefaultTime {
            phrase: "today 9am".to_string(),
        };
        match apply_safe_effect(&state, &kind) {
            ActionEffect::SetFilterText { new_text } => {
                assert_eq!(new_text, ";cal Design review start:\"today 9am\"");
            }
            other => panic!("expected SetFilterText, got {other:?}"),
        }
    }

    #[test]
    fn default_time_outside_capture_returns_unsupported() {
        let q = refine(":foo");
        let state = MenuSyntaxActionState::RefineQuery { query: &q };
        let kind = MenuSyntaxActionKind::DefaultTime {
            phrase: "today 9am".to_string(),
        };
        assert_eq!(apply_safe_effect(&state, &kind), ActionEffect::Unsupported);
    }

    #[test]
    fn edit_command_argv_resets_to_head_plus_space() {
        let argv: Vec<String> = vec!["--prod".into(), "--dry-run".into()];
        let state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::EditCommandArgv),
            ActionEffect::SetFilterText {
                new_text: "!deploy ".to_string(),
            }
        );
    }

    #[test]
    fn edit_command_argv_outside_command_returns_unsupported() {
        let inv = capture(";cal");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::EditCommandArgv),
            ActionEffect::Unsupported
        );
    }

    #[test]
    fn unsafe_action_kinds_return_unsupported() {
        // The story restricts this pass to 4 effects; everything else stays
        // Unsupported (the dialog falls back without surprise).
        let inv = capture(";cal");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &inv,
            schema: None,
        };
        for kind in [
            MenuSyntaxActionKind::SaveAndCopyId,
            MenuSyntaxActionKind::EditPayloadJson,
            MenuSyntaxActionKind::ChangeHandler,
            MenuSyntaxActionKind::OpenCapturesBrowser {
                target: "cal".into(),
            },
            MenuSyntaxActionKind::SnippetInsertField { key: "name".into() },
            MenuSyntaxActionKind::SnippetCopyGeneratedMarkdown,
            MenuSyntaxActionKind::SnippetCopyMarkdownPath,
            MenuSyntaxActionKind::SnippetPrepareUpdateSelected,
            MenuSyntaxActionKind::SnippetPrepareDeleteSelected,
            MenuSyntaxActionKind::SaveFilterAsNamedSearch,
            MenuSyntaxActionKind::AddToPinnedFilters,
            MenuSyntaxActionKind::OpenAdvancedFilterBuilder,
            MenuSyntaxActionKind::ShowCommandSchema,
            MenuSyntaxActionKind::RunWithLastArgv,
            MenuSyntaxActionKind::EditScriptSource,
        ] {
            assert_eq!(
                apply_safe_effect(&state, &kind),
                ActionEffect::Unsupported,
                "kind {kind:?} should be Unsupported in this pass"
            );
        }
    }

    #[test]
    fn snippet_insert_name_field_inserts_before_body_delimiter() {
        let inv = capture_for_target(";snippet keyword:fj -- const value = 1", "snippet");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &inv,
            schema: None,
        };

        assert_eq!(
            apply_safe_effect(
                &state,
                &MenuSyntaxActionKind::SnippetInsertField { key: "name".into() }
            ),
            ActionEffect::SetFilterText {
                new_text: ";snippet keyword:fj name: -- const value = 1".to_string()
            }
        );
    }

    #[test]
    fn snippet_copy_generated_markdown_uses_snippets_markdown_format() {
        let inv = capture_for_target(
            ";snippet name:Fetch keyword:fj description:Fetches -- const value = 1",
            "snippet",
        );
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &inv,
            schema: None,
        };

        match apply_safe_effect(&state, &MenuSyntaxActionKind::SnippetCopyGeneratedMarkdown) {
            ActionEffect::WriteClipboard { content } => {
                assert!(content.contains("## Fetch"));
                assert!(content.contains("keyword: fj"));
                assert!(content.contains("description: Fetches"));
                assert!(content.contains("const value = 1"));
            }
            other => panic!("expected markdown clipboard, got {other:?}"),
        }
    }

    #[test]
    fn snippet_copy_markdown_path_uses_scriptkit_scriptlets_file() {
        let inv = capture_for_target(";snippet name:Fetch -- const value = 1", "snippet");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &inv,
            schema: None,
        };

        match apply_safe_effect(&state, &MenuSyntaxActionKind::SnippetCopyMarkdownPath) {
            ActionEffect::WriteClipboard { content } => {
                assert!(content.ends_with("plugins/main/scriptlets/snippets.md"));
            }
            other => panic!("expected path clipboard, got {other:?}"),
        }
    }

    #[test]
    fn snippet_prepare_delete_selected_rewrites_to_non_destructive_delete_command() {
        let inv = capture_for_target(";snippet add @snippet:fj -- const value = 1", "snippet");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &inv,
            schema: None,
        };

        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::SnippetPrepareDeleteSelected),
            ActionEffect::SetFilterText {
                new_text: ";snippet delete @snippet:fj".to_string()
            }
        );
    }

    #[test]
    fn snippet_prepare_update_selected_preserves_metadata_and_body() {
        let inv = capture_for_target(
            ";snippet add @snippet:fj description:Fetches -- const value = 1",
            "snippet",
        );
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &inv,
            schema: None,
        };

        assert_eq!(
            apply_safe_effect(&state, &MenuSyntaxActionKind::SnippetPrepareUpdateSelected),
            ActionEffect::SetFilterText {
                new_text: ";snippet update @snippet:fj description:\"Fetches\" -- const value = 1"
                    .to_string()
            }
        );
    }
}
