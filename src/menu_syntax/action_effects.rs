// doc-anchor-removed: [[removed-docs Syntax#Cmd+K Safe Action Effects]]
//
// Pure decision module: given the live menu-syntax state and a Cmd+K action
// kind, return the side-effect the launcher should perform. Implements the
// four "low-risk" effects from the cmdk-safe-action-effects story
// (Cancel, Copy filter expression, Default time, Edit command argv) and
// returns `Unsupported` for everything else so the actions-dialog can fall
// back without surprise.

use crate::menu_syntax::actions::{MenuSyntaxActionKind, MenuSyntaxActionState};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::actions::MenuSyntaxActionState;
    use crate::menu_syntax::payload::{AdvancedQuery, CaptureAlias, CaptureInvocation};

    fn capture(raw: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: "cal".to_string(),
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
                content: ">deploy --prod --dry-run".to_string(),
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
                new_text: ">deploy ".to_string(),
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
}
