use crate::menu_syntax::capture_schema::{CaptureFieldSchema, FieldRequirement};
use crate::menu_syntax::payload::{AdvancedQuery, CaptureInvocation};
use crate::menu_syntax::snippet_scriptlet::{
    parse_snippet_scriptlet_capture, SnippetLookup, SnippetScriptletOperation,
};

/// State the actions surface needs to discriminate which Power Syntax actions
/// to offer. Borrows the live parse so callers (the Cmd+K actions dialog) do
/// not have to clone payloads.
#[derive(Debug, Clone)]
pub enum MenuSyntaxActionState<'a> {
    CaptureComposer {
        target: &'a str,
        payload: &'a CaptureInvocation,
        schema: Option<&'a CaptureFieldSchema>,
    },
    RefineQuery {
        query: &'a AdvancedQuery,
    },
    CommandComposer {
        head: &'a str,
        argv: &'a [String],
    },
}

/// One row in the Cmd+K Power Syntax section. The `id` is stable for telemetry
/// and the actions-dialog's keyboard shortcut binding; the `label` is what the
/// user sees; the `kind` is the structured payload the executor reads to apply
/// the effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSyntaxAction {
    pub id: String,
    pub label: String,
    pub kind: MenuSyntaxActionKind,
    pub enabled: bool,
}

/// The structured effect the actions-dialog will dispatch when the user picks
/// the row. Pure data — no GPUI types here — so the spec layer is testable
/// without a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuSyntaxActionKind {
    Cancel,
    SaveAndCopyId,
    EditPayloadJson,
    ChangeHandler,
    OpenCapturesBrowser { target: String },
    DefaultTime { phrase: String },
    SnippetInsertField { key: String },
    SnippetCopyGeneratedMarkdown,
    SnippetCopyMarkdownPath,
    SnippetPrepareUpdateSelected,
    SnippetPrepareDeleteSelected,
    SaveFilterAsNamedSearch,
    AddToPinnedFilters,
    OpenAdvancedFilterBuilder,
    CopyFilterExpression,
    ShowCommandSchema,
    EditCommandArgv,
    RunWithLastArgv,
    EditScriptSource,
}

/// Returns the action rows to surface in the Cmd+K dialog while the user is
/// composing a Power Syntax expression. Pure function — same input always
/// returns the same output. The actions-dialog wraps these in an
/// `ActionSection::new("Power Syntax", ...)`; replacement vs prepend semantics
/// (capture/command replace, refine prepends) are the dialog's job, not ours.
pub fn current_actions(state: &MenuSyntaxActionState<'_>) -> Vec<MenuSyntaxAction> {
    match state {
        MenuSyntaxActionState::CaptureComposer {
            target,
            payload,
            schema,
        } => capture_actions(target, payload, *schema),
        MenuSyntaxActionState::RefineQuery { query } => refine_actions(query),
        MenuSyntaxActionState::CommandComposer { head, argv } => command_actions(head, argv),
    }
}

fn capture_actions(
    target: &str,
    payload: &CaptureInvocation,
    schema: Option<&CaptureFieldSchema>,
) -> Vec<MenuSyntaxAction> {
    if target.eq_ignore_ascii_case("snippet") {
        return snippet_capture_actions(payload);
    }

    let mut actions = vec![
        MenuSyntaxAction {
            id: "capture.cancel".into(),
            label: "Cancel without saving".into(),
            kind: MenuSyntaxActionKind::Cancel,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "capture.save_and_copy_id".into(),
            label: "Save and copy URL/ID to clipboard".into(),
            kind: MenuSyntaxActionKind::SaveAndCopyId,
            enabled: !payload.body.trim().is_empty(),
        },
        MenuSyntaxAction {
            id: "capture.edit_payload_json".into(),
            label: "Edit raw payload JSON in editor".into(),
            kind: MenuSyntaxActionKind::EditPayloadJson,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "capture.change_handler".into(),
            label: "Change handler".into(),
            kind: MenuSyntaxActionKind::ChangeHandler,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "capture.open_browser".into(),
            label: "Open captures inverse browser at this target".into(),
            kind: MenuSyntaxActionKind::OpenCapturesBrowser {
                target: target.to_string(),
            },
            enabled: true,
        },
    ];

    // Surface the "Default time → today 9am" affordance for cal payloads
    // missing a date. The actions-dialog inserts the literal `start:"today 9am"`
    // token into the input; the user can still edit before pressing Enter.
    if let Some(schema) = schema {
        let needs_date = schema
            .missing_required(payload)
            .iter()
            .any(|req| matches!(req, FieldRequirement::AnyDate));
        if needs_date {
            actions.push(MenuSyntaxAction {
                id: "capture.default_time_today_9am".into(),
                label: "Default Time → Today 9 AM".into(),
                kind: MenuSyntaxActionKind::DefaultTime {
                    phrase: "today 9am".into(),
                },
                enabled: true,
            });
        }
    }

    actions
}

fn snippet_capture_actions(payload: &CaptureInvocation) -> Vec<MenuSyntaxAction> {
    let draft = parse_snippet_scriptlet_capture(payload).ok();
    let selected_ref = draft.as_ref().and_then(|draft| match &draft.lookup {
        Some(SnippetLookup::SelectedRef(id)) if !id.trim().is_empty() => Some(id.as_str()),
        _ => None,
    });
    let has_name_or_selection = draft
        .as_ref()
        .map(|draft| {
            draft
                .name
                .as_deref()
                .map(|name| !name.trim().is_empty())
                .unwrap_or(false)
                || selected_ref.is_some()
        })
        .unwrap_or(false);
    let has_keyword_or_selection = draft
        .as_ref()
        .map(|draft| {
            draft
                .keyword
                .as_deref()
                .map(|keyword| !keyword.trim().is_empty())
                .unwrap_or(false)
                || selected_ref.is_some()
        })
        .unwrap_or(false);
    let can_preview_generated_markdown = draft
        .as_ref()
        .map(|draft| {
            draft
                .name
                .as_deref()
                .map(|name| !name.trim().is_empty())
                .unwrap_or(false)
                && draft
                    .body
                    .as_deref()
                    .map(|body| !body.trim().is_empty())
                    .unwrap_or(false)
        })
        .unwrap_or(false);

    let mut actions = vec![
        MenuSyntaxAction {
            id: "capture.cancel".into(),
            label: "Cancel without saving".into(),
            kind: MenuSyntaxActionKind::Cancel,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "snippet.copy_command".into(),
            label: "Copy Snippet Command".into(),
            kind: MenuSyntaxActionKind::CopyFilterExpression,
            enabled: !payload.raw.trim().is_empty(),
        },
    ];

    if !has_name_or_selection {
        actions.push(MenuSyntaxAction {
            id: "snippet.insert_name_field".into(),
            label: "Insert Name Field".into(),
            kind: MenuSyntaxActionKind::SnippetInsertField { key: "name".into() },
            enabled: true,
        });
    }

    if !has_keyword_or_selection {
        actions.push(MenuSyntaxAction {
            id: "snippet.insert_keyword_field".into(),
            label: "Insert Keyword Field".into(),
            kind: MenuSyntaxActionKind::SnippetInsertField {
                key: "keyword".into(),
            },
            enabled: true,
        });
    }

    actions.extend([
        MenuSyntaxAction {
            id: "snippet.copy_generated_markdown".into(),
            label: "Copy Generated Markdown".into(),
            kind: MenuSyntaxActionKind::SnippetCopyGeneratedMarkdown,
            enabled: can_preview_generated_markdown,
        },
        MenuSyntaxAction {
            id: "snippet.copy_markdown_path".into(),
            label: "Copy snippets.md Path".into(),
            kind: MenuSyntaxActionKind::SnippetCopyMarkdownPath,
            enabled: true,
        },
    ]);

    if selected_ref.is_some() {
        let operation = draft.as_ref().map(|draft| &draft.operation);
        if !matches!(operation, Some(SnippetScriptletOperation::Update)) {
            actions.push(MenuSyntaxAction {
                id: "snippet.prepare_update_selected".into(),
                label: "Prepare Update for Selected Snippet".into(),
                kind: MenuSyntaxActionKind::SnippetPrepareUpdateSelected,
                enabled: true,
            });
        }
        if !matches!(operation, Some(SnippetScriptletOperation::Delete)) {
            actions.push(MenuSyntaxAction {
                id: "snippet.prepare_delete_selected".into(),
                label: "Prepare Delete for Selected Snippet".into(),
                kind: MenuSyntaxActionKind::SnippetPrepareDeleteSelected,
                enabled: true,
            });
        }
    }

    actions
}

fn refine_actions(_query: &AdvancedQuery) -> Vec<MenuSyntaxAction> {
    vec![
        MenuSyntaxAction {
            id: "refine.save_named_search".into(),
            label: "Save filter as named search".into(),
            kind: MenuSyntaxActionKind::SaveFilterAsNamedSearch,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "refine.add_pinned".into(),
            label: "Add to launcher pinned filters".into(),
            kind: MenuSyntaxActionKind::AddToPinnedFilters,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "refine.open_builder".into(),
            label: "Open advanced filter builder".into(),
            kind: MenuSyntaxActionKind::OpenAdvancedFilterBuilder,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "refine.copy_filter".into(),
            label: "Copy Filter".into(),
            kind: MenuSyntaxActionKind::CopyFilterExpression,
            enabled: true,
        },
    ]
}

fn command_actions(_head: &str, argv: &[String]) -> Vec<MenuSyntaxAction> {
    let mut actions = vec![
        MenuSyntaxAction {
            id: "command.show_schema".into(),
            label: "Show command schema/help".into(),
            kind: MenuSyntaxActionKind::ShowCommandSchema,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "command.edit_argv".into(),
            label: "Edit Command Arguments".into(),
            kind: MenuSyntaxActionKind::EditCommandArgv,
            enabled: true,
        },
        MenuSyntaxAction {
            id: "command.edit_script".into(),
            label: "Edit Script".into(),
            kind: MenuSyntaxActionKind::EditScriptSource,
            enabled: true,
        },
    ];
    if !argv.is_empty() {
        actions.insert(
            2,
            MenuSyntaxAction {
                id: "command.run_with_last_argv".into(),
                label: "Run with last argv".into(),
                kind: MenuSyntaxActionKind::RunWithLastArgv,
                enabled: true,
            },
        );
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture_schema::builtin_schema;
    use crate::menu_syntax::payload::{CaptureAlias, DatePhrase, DateRole};

    fn empty_invocation(target: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: target.to_string(),
            alias_form: CaptureAlias::CapturePrefix,
            body: String::new(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: format!("+{target}"),
        }
    }

    fn invocation_with_body(target: &str, body: &str) -> CaptureInvocation {
        let mut inv = empty_invocation(target);
        inv.body = body.into();
        inv.raw = format!("+{target} {body}");
        inv
    }

    fn empty_query() -> AdvancedQuery {
        AdvancedQuery {
            free_text: String::new(),
            predicates: vec![],
            source_filters: Default::default(),
            raw: ":".into(),
        }
    }

    fn ids(actions: &[MenuSyntaxAction]) -> Vec<&str> {
        actions.iter().map(|a| a.id.as_str()).collect()
    }

    #[test]
    fn capture_state_returns_baseline_actions() {
        let payload = invocation_with_body("todo", "Renew passport");
        let schema = builtin_schema("todo");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "todo",
            payload: &payload,
            schema: schema.as_ref(),
        };
        let actions = current_actions(&state);
        assert!(ids(&actions).contains(&"capture.cancel"));
        assert!(ids(&actions).contains(&"capture.save_and_copy_id"));
        assert!(ids(&actions).contains(&"capture.edit_payload_json"));
        assert!(ids(&actions).contains(&"capture.change_handler"));
        assert!(ids(&actions).contains(&"capture.open_browser"));
        // todo with body never adds the default-time action
        assert!(!ids(&actions).contains(&"capture.default_time_today_9am"));
    }

    #[test]
    fn capture_save_disabled_when_body_empty() {
        let payload = empty_invocation("todo");
        let schema = builtin_schema("todo");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "todo",
            payload: &payload,
            schema: schema.as_ref(),
        };
        let actions = current_actions(&state);
        let save = actions
            .iter()
            .find(|a| a.id == "capture.save_and_copy_id")
            .expect("save action present");
        assert!(!save.enabled, "empty body must disable save");
    }

    #[test]
    fn cal_without_date_offers_default_time() {
        let payload = invocation_with_body("cal", "Design review");
        let schema = builtin_schema("cal");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &payload,
            schema: schema.as_ref(),
        };
        let actions = current_actions(&state);
        let dt = actions
            .iter()
            .find(|a| a.id == "capture.default_time_today_9am")
            .expect("cal w/o date must offer default-time");
        assert_eq!(dt.label, "Default Time → Today 9 AM");
        assert!(matches!(
            dt.kind,
            MenuSyntaxActionKind::DefaultTime { ref phrase } if phrase == "today 9am"
        ));
    }

    #[test]
    fn cal_with_date_does_not_offer_default_time() {
        let mut payload = invocation_with_body("cal", "Design review");
        payload.date_phrases.push(DatePhrase {
            role: DateRole::Start,
            source: "friday 2pm".into(),
            source_span: (0, 10),
        });
        let schema = builtin_schema("cal");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &payload,
            schema: schema.as_ref(),
        };
        let actions = current_actions(&state);
        assert!(!ids(&actions).contains(&"capture.default_time_today_9am"));
    }

    #[test]
    fn capture_open_browser_carries_target_slug() {
        let payload = invocation_with_body("note", "Some thought");
        let schema = builtin_schema("note");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "note",
            payload: &payload,
            schema: schema.as_ref(),
        };
        let actions = current_actions(&state);
        let open = actions
            .iter()
            .find(|a| a.id == "capture.open_browser")
            .unwrap();
        match &open.kind {
            MenuSyntaxActionKind::OpenCapturesBrowser { target } => assert_eq!(target, "note"),
            other => panic!("expected OpenCapturesBrowser, got {other:?}"),
        }
    }

    #[test]
    fn snippet_capture_state_returns_authoring_actions() {
        let payload = invocation_with_body("snippet", "add -- const value = 1");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &payload,
            schema: None,
        };
        let actions = current_actions(&state);

        assert_eq!(
            ids(&actions),
            vec![
                "capture.cancel",
                "snippet.copy_command",
                "snippet.insert_name_field",
                "snippet.insert_keyword_field",
                "snippet.copy_generated_markdown",
                "snippet.copy_markdown_path",
            ]
        );
        assert!(
            !actions
                .iter()
                .find(|action| action.id == "snippet.copy_generated_markdown")
                .unwrap()
                .enabled,
            "preview stays disabled until name and body are both present"
        );
    }

    #[test]
    fn snippet_capture_with_name_and_keyword_enables_markdown_preview() {
        let payload =
            invocation_with_body("snippet", "add name:Fetch keyword:fj -- const value = 1");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &payload,
            schema: None,
        };
        let actions = current_actions(&state);

        assert!(!ids(&actions).contains(&"snippet.insert_name_field"));
        assert!(!ids(&actions).contains(&"snippet.insert_keyword_field"));
        assert!(
            actions
                .iter()
                .find(|action| action.id == "snippet.copy_generated_markdown")
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn snippet_capture_with_selected_ref_offers_update_delete_preparation() {
        let payload = invocation_with_body("snippet", "add @snippet:fj -- const value = 1");
        let state = MenuSyntaxActionState::CaptureComposer {
            target: "snippet",
            payload: &payload,
            schema: None,
        };
        let actions = current_actions(&state);

        assert!(ids(&actions).contains(&"snippet.prepare_update_selected"));
        assert!(ids(&actions).contains(&"snippet.prepare_delete_selected"));
        assert!(
            !ids(&actions).contains(&"snippet.insert_name_field"),
            "selected ref satisfies name/selection workflow"
        );
    }

    #[test]
    fn refine_state_returns_four_actions() {
        let q = empty_query();
        let state = MenuSyntaxActionState::RefineQuery { query: &q };
        let actions = current_actions(&state);
        assert_eq!(actions.len(), 4);
        assert_eq!(
            ids(&actions),
            vec![
                "refine.save_named_search",
                "refine.add_pinned",
                "refine.open_builder",
                "refine.copy_filter",
            ]
        );
    }

    #[test]
    fn command_state_baseline_three_actions() {
        let argv: Vec<String> = vec![];
        let state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        let actions = current_actions(&state);
        assert_eq!(actions.len(), 3);
        assert_eq!(
            ids(&actions),
            vec![
                "command.show_schema",
                "command.edit_argv",
                "command.edit_script",
            ]
        );
    }

    #[test]
    fn command_state_with_recent_argv_inserts_run_with_last() {
        let argv = vec!["prod".to_string(), "--dry-run".to_string()];
        let state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        let actions = current_actions(&state);
        assert_eq!(actions.len(), 4);
        // Run-with-last is inserted between edit_argv and edit_script.
        assert_eq!(
            ids(&actions),
            vec![
                "command.show_schema",
                "command.edit_argv",
                "command.run_with_last_argv",
                "command.edit_script",
            ]
        );
    }

    #[test]
    fn action_ids_are_unique_within_each_state() {
        let payload = invocation_with_body("cal", "Design review");
        let schema = builtin_schema("cal");
        let cap_state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &payload,
            schema: schema.as_ref(),
        };
        let refine_state = MenuSyntaxActionState::RefineQuery {
            query: &empty_query(),
        };
        let argv = vec!["x".to_string()];
        let cmd_state = MenuSyntaxActionState::CommandComposer {
            head: "deploy",
            argv: &argv,
        };
        for state in [&cap_state, &refine_state, &cmd_state] {
            let actions = current_actions(state);
            let mut sorted_ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
            sorted_ids.sort();
            let unique_count = sorted_ids
                .windows(2)
                .filter(|pair| pair[0] != pair[1])
                .count()
                + 1;
            assert_eq!(
                unique_count,
                actions.len(),
                "duplicate action ids: {sorted_ids:?}"
            );
        }
    }

    #[test]
    fn all_action_labels_are_non_empty() {
        let payload = invocation_with_body("cal", "Design review");
        let schema = builtin_schema("cal");
        let cap_state = MenuSyntaxActionState::CaptureComposer {
            target: "cal",
            payload: &payload,
            schema: schema.as_ref(),
        };
        for state in [
            &cap_state,
            &MenuSyntaxActionState::RefineQuery {
                query: &empty_query(),
            },
            &MenuSyntaxActionState::CommandComposer {
                head: "deploy",
                argv: &[],
            },
        ] {
            for action in current_actions(state) {
                assert!(!action.label.trim().is_empty(), "empty label: {action:?}");
                assert!(!action.id.trim().is_empty(), "empty id: {action:?}");
            }
        }
    }
}
