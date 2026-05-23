use super::trigger_picker::{
    TriggerPickerAction, TriggerPickerMode, TriggerPickerRowKind, TriggerPickerSnapshot,
};

/// Owner-neutral key intent for the trigger picker. Owners (ScriptList main
/// input, later possibly detached popups) translate their platform key events
/// into this enum so dispatch stays consistent without forcing a shared
/// dispatcher — per Oracle iter 007: "share the key intent classifier, not
/// the whole dispatcher."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlinePickerKeyIntent {
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
    PageUp,
    PageDown,
    /// Enter-style accept: perform the selected row's action and close the
    /// picker (unless the action is an open-value qualifier like `source:`).
    Accept,
    /// Tab-style apply: perform the selected row's action but keep the picker
    /// open so the user can continue typing.
    Apply,
    /// First Escape: close the picker only. Owners decide whether a second
    /// Escape falls through to their normal behavior (e.g. `clear_filter`).
    Close,
    /// Cmd+P on capture rows: open the Captures inverse browser scoped to the
    /// active target. Not wired until commit 7.
    SecondaryAction,
    /// Cmd+N on capture rows: invoke the create-handler footer action. Not
    /// wired until commit 6.
    CreateAction,
}

/// Resolved outcome for the caller after applying an intent to a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerPickerIntentOutcome {
    /// No-op — the intent could not be applied (e.g. MoveDown on an empty
    /// snapshot). Caller should leave state untouched and let the keystroke
    /// fall through to normal dispatch.
    Ignored,
    /// Selection-only change. Caller re-renders but does not mutate input.
    SelectionChanged { new_index: usize },
    /// The picker wants to rewrite the filter input. Caller sets the input
    /// to `text` and decides whether to keep the picker open based on
    /// `keep_open` (false implies close-after-apply).
    ReplaceInput { text: String, keep_open: bool },
    /// The picker wants to close. Caller hides the snapshot and clears
    /// menu-syntax rendering. Does not rewrite input.
    Close,
    /// Caller should open the inverse browser scoped to `target`. Deferred to
    /// commit 7 — caller may ignore until then.
    OpenCaptures { target: Option<String> },
    /// Caller should invoke the create-handler scaffold for `target`.
    /// Deferred to commit 6 — caller may ignore until then.
    CreateHandler { target: Option<String> },
    /// Caller should seed the AI prompt for a new capture handler scaffold.
    AiScaffoldHandler {
        slug: String,
        nearest_targets: Vec<String>,
    },
    /// Caller should open the docs/help view for menu syntax. Deferred —
    /// owner may route to a no-op log line until the help view ships.
    OpenHelp,
}

/// Return the index of the first row in `snapshot.rows` that is selectable —
/// i.e. not a `FooterAction` row. Footer rows are displayed but not the
/// default selection target because their actions are "create handler" or
/// "open help," which only make sense after explicit user navigation.
pub fn first_selectable_index(snapshot: &TriggerPickerSnapshot) -> Option<usize> {
    snapshot.rows.iter().position(row_is_selectable)
}

/// Return the index of the last row in `snapshot.rows` that is selectable.
pub fn last_selectable_index(snapshot: &TriggerPickerSnapshot) -> Option<usize> {
    snapshot
        .rows
        .iter()
        .enumerate()
        .rfind(|(_, row)| row_is_selectable(row))
        .map(|(idx, _)| idx)
}

fn row_is_selectable(row: &super::trigger_picker::TriggerPickerRow) -> bool {
    row.enabled
        && (row.kind != TriggerPickerRowKind::FooterAction
            || matches!(
                row.action,
                super::trigger_picker::TriggerPickerAction::CreateHandler { .. }
            ))
}

/// Return the NEXT selectable index after `current`, skipping footer rows.
/// Wraps to the first selectable index when at the end. Returns `None` if
/// the snapshot has no selectable rows.
pub fn next_selectable_index(
    snapshot: &TriggerPickerSnapshot,
    current: Option<usize>,
) -> Option<usize> {
    let first = first_selectable_index(snapshot)?;
    let last = last_selectable_index(snapshot)?;
    let start = match current {
        Some(idx) if idx < last => idx + 1,
        _ => first,
    };

    for idx in start..=snapshot.rows.len().saturating_sub(1) {
        let row = snapshot.rows.get(idx)?;
        if row_is_selectable(row) {
            return Some(idx);
        }
    }
    Some(first)
}

/// Return the PREVIOUS selectable index before `current`, skipping footer rows.
/// Wraps to the last selectable index when at the beginning. Returns `None` if
/// the snapshot has no selectable rows.
pub fn prev_selectable_index(
    snapshot: &TriggerPickerSnapshot,
    current: Option<usize>,
) -> Option<usize> {
    let first = first_selectable_index(snapshot)?;
    let last = last_selectable_index(snapshot)?;
    let start = match current {
        Some(idx) if idx > first => idx - 1,
        _ => last,
    };

    for idx in (0..=start).rev() {
        let row = snapshot.rows.get(idx)?;
        if row_is_selectable(row) {
            return Some(idx);
        }
    }
    Some(last)
}

/// Apply an intent against a snapshot and the currently-selected index. The
/// caller provides the raw filter text so the classifier can compute text
/// replacements (e.g. `FixQualifier` needs the current input to know what to
/// replace).
pub fn apply_intent(
    intent: InlinePickerKeyIntent,
    snapshot: &TriggerPickerSnapshot,
    selected_index: Option<usize>,
    raw_filter_text: &str,
) -> TriggerPickerIntentOutcome {
    match intent {
        InlinePickerKeyIntent::MoveUp => match prev_selectable_index(snapshot, selected_index) {
            Some(new_index) => TriggerPickerIntentOutcome::SelectionChanged { new_index },
            None => TriggerPickerIntentOutcome::Ignored,
        },
        InlinePickerKeyIntent::MoveDown => match next_selectable_index(snapshot, selected_index) {
            Some(new_index) => TriggerPickerIntentOutcome::SelectionChanged { new_index },
            None => TriggerPickerIntentOutcome::Ignored,
        },
        InlinePickerKeyIntent::MoveHome => match first_selectable_index(snapshot) {
            Some(new_index) => TriggerPickerIntentOutcome::SelectionChanged { new_index },
            None => TriggerPickerIntentOutcome::Ignored,
        },
        InlinePickerKeyIntent::MoveEnd => match last_selectable_index(snapshot) {
            Some(new_index) => TriggerPickerIntentOutcome::SelectionChanged { new_index },
            None => TriggerPickerIntentOutcome::Ignored,
        },
        InlinePickerKeyIntent::PageUp | InlinePickerKeyIntent::PageDown => {
            // Inline picker sizes are small (~10 rows visible), so page = home/end.
            let idx = match intent {
                InlinePickerKeyIntent::PageUp => first_selectable_index(snapshot),
                _ => last_selectable_index(snapshot),
            };
            match idx {
                Some(new_index) => TriggerPickerIntentOutcome::SelectionChanged { new_index },
                None => TriggerPickerIntentOutcome::Ignored,
            }
        }
        InlinePickerKeyIntent::Close => TriggerPickerIntentOutcome::Close,
        InlinePickerKeyIntent::Accept | InlinePickerKeyIntent::Apply => {
            resolve_row_action(snapshot, selected_index, raw_filter_text, intent)
        }
        InlinePickerKeyIntent::SecondaryAction => {
            if let Some(TriggerPickerIntentOutcome::AiScaffoldHandler {
                slug,
                nearest_targets,
            }) = selected_create_handler_ai_scaffold(snapshot, selected_index)
            {
                return TriggerPickerIntentOutcome::AiScaffoldHandler {
                    slug,
                    nearest_targets,
                };
            }
            // Cmd+P: open Captures scoped to the active target. Only fires in
            // Capture mode. In AdvancedQuery mode we ignore.
            match snapshot.mode {
                TriggerPickerMode::Capture => TriggerPickerIntentOutcome::OpenCaptures {
                    target: snapshot.target.clone(),
                },
                _ => TriggerPickerIntentOutcome::Ignored,
            }
        }
        InlinePickerKeyIntent::CreateAction => {
            // Cmd+N: fire the footer CreateHandler action for the current
            // capture target (or None when bare `+`).
            match snapshot.mode {
                TriggerPickerMode::Capture => TriggerPickerIntentOutcome::CreateHandler {
                    target: snapshot.target.clone(),
                },
                _ => TriggerPickerIntentOutcome::Ignored,
            }
        }
    }
}

fn selected_create_handler_ai_scaffold(
    snapshot: &TriggerPickerSnapshot,
    selected_index: Option<usize>,
) -> Option<TriggerPickerIntentOutcome> {
    let row = selected_index.and_then(|idx| snapshot.rows.get(idx))?;
    let TriggerPickerAction::CreateHandler { target: Some(slug) } = &row.action else {
        return None;
    };
    let nearest_targets = snapshot
        .rows
        .iter()
        .filter(|candidate| candidate.kind == TriggerPickerRowKind::CaptureTarget)
        .filter_map(|candidate| {
            candidate
                .token
                .as_deref()
                .and_then(|token| token.strip_prefix(';'))
                .map(str::to_string)
        })
        .collect();
    Some(TriggerPickerIntentOutcome::AiScaffoldHandler {
        slug: slug.clone(),
        nearest_targets,
    })
}

fn resolve_row_action(
    snapshot: &TriggerPickerSnapshot,
    selected_index: Option<usize>,
    raw_filter_text: &str,
    _intent: InlinePickerKeyIntent,
) -> TriggerPickerIntentOutcome {
    let idx = selected_index.or_else(|| first_selectable_index(snapshot));
    let Some(idx) = idx else {
        return TriggerPickerIntentOutcome::Ignored;
    };
    let Some(row) = snapshot.rows.get(idx) else {
        return TriggerPickerIntentOutcome::Ignored;
    };

    match &row.action {
        TriggerPickerAction::InsertToken { token, keep_open } => {
            let text = apply_token_insertion(raw_filter_text, token);
            TriggerPickerIntentOutcome::ReplaceInput {
                text,
                keep_open: *keep_open,
            }
        }
        TriggerPickerAction::ReplaceInput { text } => TriggerPickerIntentOutcome::ReplaceInput {
            text: text.clone(),
            keep_open: false,
        },
        TriggerPickerAction::FixQualifier { bad, good } => {
            let text = rewrite_token_substring(raw_filter_text, bad, good);
            TriggerPickerIntentOutcome::ReplaceInput {
                text,
                keep_open: false,
            }
        }
        TriggerPickerAction::ExecuteCaptureHandler { .. } => {
            // Reserved for commit 4/5: handler execution path will be wired
            // through the existing menu_syntax_execution adapter.
            TriggerPickerIntentOutcome::Ignored
        }
        TriggerPickerAction::OpenCaptures { target } => TriggerPickerIntentOutcome::OpenCaptures {
            target: target.clone(),
        },
        TriggerPickerAction::CreateHandler { target } => {
            let slug = target.clone().unwrap_or_default();
            let nearest_targets = if slug.is_empty() {
                Vec::new()
            } else {
                snapshot
                    .rows
                    .iter()
                    .filter(|candidate| candidate.kind == TriggerPickerRowKind::CaptureTarget)
                    .filter_map(|candidate| {
                        candidate
                            .token
                            .as_deref()
                            .and_then(|token| token.strip_prefix(';'))
                            .map(str::to_string)
                    })
                    .collect()
            };
            TriggerPickerIntentOutcome::AiScaffoldHandler {
                slug,
                nearest_targets,
            }
        }
        TriggerPickerAction::OpenHelp => TriggerPickerIntentOutcome::OpenHelp,
    }
}

/// Replace the first `<token>:` or bare `<token>` occurrence inside
/// `raw_filter_text` with `replacement`, preserving the rest of the string.
/// If `bad` is not found, returns `raw_filter_text` unchanged.
fn rewrite_token_substring(raw_filter_text: &str, bad: &str, good: &str) -> String {
    match raw_filter_text.find(bad) {
        Some(idx) => {
            let prefix = &raw_filter_text[..idx];
            let suffix = &raw_filter_text[idx + bad.len()..];
            format!("{prefix}{good}{suffix}")
        }
        None => raw_filter_text.to_string(),
    }
}

/// Produce the new filter text after a picker row's `InsertToken` action.
/// Capture (`;`, with legacy `+` input still accepted) and command (`!`) rows
/// replace the command head. Refine (`:`) rows replace only the active
/// qualifier token so multi-qualifier composition keeps the rest of the query
/// intact.
fn apply_token_insertion(raw_filter_text: &str, token: &str) -> String {
    let trimmed = raw_filter_text.trim_start();
    if trimmed.starts_with(':') && (token.starts_with(':') || !token_is_root_source_head(token)) {
        return apply_advanced_query_token_insertion(raw_filter_text, token);
    }
    if (trimmed.starts_with(';') && token.starts_with(';'))
        || (trimmed.starts_with('>') && token.starts_with('>'))
    {
        return token.to_string();
    }
    token.to_string()
}

fn token_is_root_source_head(token: &str) -> bool {
    crate::menu_syntax::SOURCE_HEAD_SPECS
        .iter()
        .any(|spec| spec.canonical == token)
}

fn apply_advanced_query_token_insertion(raw_filter_text: &str, token: &str) -> String {
    let leading_ws_len = raw_filter_text.len() - raw_filter_text.trim_start().len();
    let colon_idx = leading_ws_len;
    let token_body = token.trim_start_matches(':');
    let after_colon = colon_idx + 1;
    if raw_filter_text.len() <= after_colon {
        return format!("{}:{token_body}", &raw_filter_text[..colon_idx]);
    }

    let tail = &raw_filter_text[after_colon..];
    let active_start_in_tail = tail
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let active_abs = after_colon + active_start_in_tail;
    let prefix = &raw_filter_text[..active_abs];
    format!("{prefix}{token_body}")
}

#[cfg(test)]
#[allow(clippy::derivable_impls)]
mod tests {
    use super::super::trigger_picker::{
        build_trigger_picker_snapshot, TriggerPickerContext, TriggerPickerRowKind,
    };
    use super::*;

    fn ctx() -> TriggerPickerContext {
        TriggerPickerContext::default()
    }

    #[test]
    fn first_selectable_skips_footer_rows() {
        let snap = build_trigger_picker_snapshot("+", &ctx()).expect("plus snapshot");
        let idx = first_selectable_index(&snap).expect("selectable row");
        assert_eq!(
            snap.rows[idx].kind,
            TriggerPickerRowKind::CaptureTarget,
            "first selectable row should be the first target, not the footer"
        );
    }

    #[test]
    fn last_selectable_skips_footer_rows() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        let idx = last_selectable_index(&snap).expect("selectable row");
        assert!(
            snap.rows[idx].kind != TriggerPickerRowKind::FooterAction,
            "last selectable row must not be the footer"
        );
    }

    #[test]
    fn next_selectable_advances_past_non_footer_rows() {
        let snap = build_trigger_picker_snapshot("+", &ctx()).expect("plus snapshot");
        let first = first_selectable_index(&snap).unwrap();
        let second = next_selectable_index(&snap, Some(first)).unwrap();
        assert_eq!(second, first + 1);
    }

    #[test]
    fn next_selectable_wraps_at_end() {
        let snap = build_trigger_picker_snapshot("+", &ctx()).expect("plus snapshot");
        let last = last_selectable_index(&snap).unwrap();
        let wrapped = next_selectable_index(&snap, Some(last)).unwrap();
        let first = first_selectable_index(&snap).unwrap();
        assert_eq!(wrapped, first, "selection wraps back to first row");
    }

    #[test]
    fn prev_selectable_wraps_at_start() {
        let snap = build_trigger_picker_snapshot("+", &ctx()).expect("plus snapshot");
        let first = first_selectable_index(&snap).unwrap();
        let wrapped = prev_selectable_index(&snap, Some(first)).unwrap();
        let last = last_selectable_index(&snap).unwrap();
        assert_eq!(wrapped, last, "selection wraps back to last row");
    }

    #[test]
    fn accept_source_head_from_exact_colon_inserts_root_filter_and_closes() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        let files_idx = snap
            .rows
            .iter()
            .position(|r| r.token.as_deref() == Some("files:"))
            .expect("files source head row");
        let outcome = apply_intent(InlinePickerKeyIntent::Accept, &snap, None, ":");
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(files_idx, 0, "files: should be the first source head");
                assert_eq!(text, "files:");
                assert!(!keep_open, "Accept on source head should close the picker");
            }
            other => panic!("expected ReplaceInput, got {other:?}"),
        }
    }

    #[test]
    fn accept_advanced_head_from_exact_colon_keeps_popup_open() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        let type_idx = snap
            .rows
            .iter()
            .position(|r| r.token.as_deref() == Some("type:"))
            .expect("type head row");
        let outcome = apply_intent(InlinePickerKeyIntent::Accept, &snap, Some(type_idx), ":");
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(text, ":type:");
                assert!(keep_open, "Accept on advanced head should keep picker open");
            }
            other => panic!("expected ReplaceInput keep_open=true, got {other:?}"),
        }
    }

    #[test]
    fn apply_on_capture_target_commits_text_and_closes_picker() {
        let snap = build_trigger_picker_snapshot("+", &ctx()).expect("plus snapshot");
        let todo_idx = snap
            .rows
            .iter()
            .position(|r| r.id == "target:todo")
            .expect("todo target row");
        let outcome = apply_intent(InlinePickerKeyIntent::Apply, &snap, Some(todo_idx), "+");
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(text, ";todo ");
                assert!(
                    !keep_open,
                    "Tab on a capture target should commit the target and enter composer mode"
                );
            }
            other => panic!("expected ReplaceInput, got {other:?}"),
        }
    }

    #[test]
    fn apply_on_command_row_commits_command_and_closes_picker() {
        use std::path::PathBuf;

        let script = std::sync::Arc::new(crate::scripts::Script {
            name: "Deploy Prod".to_string(),
            path: PathBuf::from("/tmp/deploy-prod.ts"),
            extension: "ts".to_string(),
            plugin_id: "main".to_string(),
            ..Default::default()
        });
        let ctx = TriggerPickerContext {
            scripts: vec![script],
            ..Default::default()
        };
        let snap = build_trigger_picker_snapshot("!dep", &ctx).expect("command snapshot");
        match apply_intent(InlinePickerKeyIntent::Apply, &snap, Some(0), "!dep") {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(text, ">deploy-prod ");
                assert!(!keep_open);
            }
            other => panic!("expected ReplaceInput, got {other:?}"),
        }
    }

    #[test]
    fn apply_intent_on_open_value_row_keeps_popup_open() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        let type_idx = snap
            .rows
            .iter()
            .position(|r| r.token.as_deref() == Some("type:"))
            .expect("type head row");
        let outcome = apply_intent(InlinePickerKeyIntent::Apply, &snap, Some(type_idx), ":");
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(text, ":type:");
                assert!(
                    keep_open,
                    "Apply on open-value qualifier must keep picker open"
                );
            }
            other => panic!("expected ReplaceInput keep_open=true, got {other:?}"),
        }
    }

    #[test]
    fn apply_on_advanced_query_replaces_only_active_token() {
        let snap = build_trigger_picker_snapshot(":type:script sour", &ctx()).expect("snapshot");
        let source_idx = snap
            .rows
            .iter()
            .position(|r| r.id == "qualifier:source:")
            .expect("source row");
        let outcome = apply_intent(
            InlinePickerKeyIntent::Apply,
            &snap,
            Some(source_idx),
            ":type:script sour",
        );
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(text, ":type:script source:");
                assert!(keep_open);
            }
            other => panic!("expected ReplaceInput keep_open=true, got {other:?}"),
        }
    }

    #[test]
    fn accept_on_open_value_row_keeps_picker_open() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        let type_idx = snap
            .rows
            .iter()
            .position(|r| r.token.as_deref() == Some("type:"))
            .expect("type head row");
        let outcome = apply_intent(InlinePickerKeyIntent::Accept, &snap, Some(type_idx), ":");
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { keep_open, .. } => {
                assert!(
                    keep_open,
                    "Accept on open-value qualifier should keep the popup open for value entry"
                );
            }
            other => panic!("expected ReplaceInput, got {other:?}"),
        }
    }

    #[test]
    fn accept_has_shortcut_completion_closes_without_space() {
        for input in ["has:short", "has:shortc"] {
            let snap = build_trigger_picker_snapshot(input, &ctx()).expect("has snapshot");
            let shortcut_idx = snap
                .rows
                .iter()
                .position(|r| r.id == "qualifier:has:shortcut")
                .expect("has:shortcut row");
            let outcome = apply_intent(
                InlinePickerKeyIntent::Accept,
                &snap,
                Some(shortcut_idx),
                input,
            );

            match outcome {
                TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                    assert_eq!(text, "has:shortcut");
                    assert!(
                        !keep_open,
                        "Accept on concrete has:shortcut must close the picker"
                    );
                }
                other => panic!("expected ReplaceInput for {input}, got {other:?}"),
            }
        }
    }

    #[test]
    fn accept_preserves_open_value_rows() {
        for (input, token, expected) in [
            (":", "type:", ":type:"),
            (":", "tag:", ":tag:"),
            (":", "has:", ":has:"),
        ] {
            let snap = build_trigger_picker_snapshot(input, &ctx()).expect("snapshot");
            let row_idx = snap
                .rows
                .iter()
                .position(|r| r.token.as_deref() == Some(token))
                .expect(token);
            let outcome = apply_intent(InlinePickerKeyIntent::Accept, &snap, Some(row_idx), input);

            match outcome {
                TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                    assert_eq!(text, expected);
                    assert!(keep_open, "{token} should keep the picker open");
                }
                other => panic!("expected ReplaceInput keep_open=true, got {other:?}"),
            }
        }
    }

    #[test]
    fn fix_qualifier_rewrites_typo_in_place() {
        let snap = build_trigger_picker_snapshot(":typ:script", &ctx()).expect("typo snapshot");
        let fix_idx = snap
            .rows
            .iter()
            .position(|r| r.kind == TriggerPickerRowKind::UnknownQualifierFix)
            .expect("fix row");
        let outcome = apply_intent(
            InlinePickerKeyIntent::Accept,
            &snap,
            Some(fix_idx),
            ":typ:script",
        );
        match outcome {
            TriggerPickerIntentOutcome::ReplaceInput { text, keep_open } => {
                assert_eq!(text, ":type:script");
                assert!(!keep_open);
            }
            other => panic!("expected ReplaceInput, got {other:?}"),
        }
    }

    #[test]
    fn close_intent_returns_close_outcome() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::Close, &snap, None, ":"),
            TriggerPickerIntentOutcome::Close
        );
    }

    #[test]
    fn primary_action_on_create_handler_footer_routes_to_ai_scaffold() {
        let snap = build_trigger_picker_snapshot(";gcal", &ctx()).expect("capture snapshot");
        let footer_idx = snap
            .rows
            .iter()
            .position(|row| {
                row.kind == TriggerPickerRowKind::FooterAction
                    && matches!(
                        row.action,
                        TriggerPickerAction::CreateHandler {
                            target: Some(ref target)
                        } if target == "gcal"
                    )
            })
            .expect("create handler footer");

        let outcome = apply_intent(
            InlinePickerKeyIntent::Accept,
            &snap,
            Some(footer_idx),
            ";gcal",
        );

        assert_eq!(
            outcome,
            TriggerPickerIntentOutcome::AiScaffoldHandler {
                slug: "gcal".to_string(),
                nearest_targets: Vec::new(),
            }
        );
    }

    #[test]
    fn secondary_action_on_create_handler_footer_routes_to_ai_scaffold() {
        let snap = build_trigger_picker_snapshot(";gcal", &ctx()).expect("capture snapshot");
        let footer_idx = snap
            .rows
            .iter()
            .position(|row| {
                row.kind == TriggerPickerRowKind::FooterAction
                    && matches!(
                        row.action,
                        TriggerPickerAction::CreateHandler {
                            target: Some(ref target)
                        } if target == "gcal"
                    )
            })
            .expect("create handler footer");

        let outcome = apply_intent(
            InlinePickerKeyIntent::SecondaryAction,
            &snap,
            Some(footer_idx),
            ";gcal",
        );

        match outcome {
            TriggerPickerIntentOutcome::AiScaffoldHandler {
                slug,
                nearest_targets,
            } => {
                assert_eq!(slug, "gcal");
                assert!(nearest_targets.is_empty());
            }
            other => panic!("expected AiScaffoldHandler, got {other:?}"),
        }
    }

    #[test]
    fn secondary_action_on_capture_mode_opens_captures_for_target() {
        let snap = build_trigger_picker_snapshot(";todo", &ctx()).expect("plus todo snapshot");
        let outcome = apply_intent(InlinePickerKeyIntent::SecondaryAction, &snap, None, ";todo");
        assert_eq!(
            outcome,
            TriggerPickerIntentOutcome::OpenCaptures {
                target: Some("todo".to_string())
            }
        );
    }

    #[test]
    fn secondary_action_in_query_mode_is_ignored() {
        let snap = build_trigger_picker_snapshot(":", &ctx()).expect("colon snapshot");
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::SecondaryAction, &snap, None, ":"),
            TriggerPickerIntentOutcome::Ignored
        );
    }

    #[test]
    fn create_action_on_capture_mode_fires_create_handler() {
        let snap = build_trigger_picker_snapshot(";todo", &ctx()).expect("plus todo snapshot");
        let outcome = apply_intent(InlinePickerKeyIntent::CreateAction, &snap, None, ";todo");
        assert_eq!(
            outcome,
            TriggerPickerIntentOutcome::CreateHandler {
                target: Some("todo".to_string())
            }
        );
    }

    #[test]
    fn move_home_and_end_return_first_and_last_selectable() {
        let snap = build_trigger_picker_snapshot("+", &ctx()).expect("plus snapshot");
        let first = first_selectable_index(&snap).unwrap();
        let last = last_selectable_index(&snap).unwrap();
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::MoveHome, &snap, Some(last), "+"),
            TriggerPickerIntentOutcome::SelectionChanged { new_index: first }
        );
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::MoveEnd, &snap, Some(first), "+"),
            TriggerPickerIntentOutcome::SelectionChanged { new_index: last }
        );
    }

    #[test]
    fn move_intents_on_empty_snapshot_ignored() {
        let empty = TriggerPickerSnapshot {
            mode: TriggerPickerMode::AdvancedQuery,
            target: None,
            rows: Vec::new(),
        };
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::MoveDown, &empty, None, ""),
            TriggerPickerIntentOutcome::Ignored
        );
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::MoveUp, &empty, None, ""),
            TriggerPickerIntentOutcome::Ignored
        );
    }

    #[test]
    fn disabled_rows_are_not_selectable() {
        let snapshot = TriggerPickerSnapshot {
            mode: TriggerPickerMode::Command,
            target: None,
            rows: vec![super::super::trigger_picker::TriggerPickerRow {
                id: "disabled".to_string(),
                mode: TriggerPickerMode::Command,
                kind: TriggerPickerRowKind::Command,
                title: "Disabled".to_string(),
                token: Some("!disabled".to_string()),
                subtitle: None,
                detail: None,
                example: None,
                badges: vec!["duplicate".to_string()],
                action: TriggerPickerAction::InsertToken {
                    token: "!disabled ".to_string(),
                    keep_open: false,
                },
                enabled: false,
            }],
        };

        assert_eq!(first_selectable_index(&snapshot), None);
        assert_eq!(
            apply_intent(InlinePickerKeyIntent::Accept, &snapshot, None, "!dis"),
            TriggerPickerIntentOutcome::Ignored
        );
    }

    #[test]
    fn rewrite_token_substring_handles_missing_token_gracefully() {
        assert_eq!(
            rewrite_token_substring("hello world", "missing", "replaced"),
            "hello world"
        );
    }

    #[test]
    fn rewrite_token_substring_replaces_first_occurrence() {
        assert_eq!(
            rewrite_token_substring(":typ:script git", "typ:script", "type:script"),
            ":type:script git"
        );
    }
}
