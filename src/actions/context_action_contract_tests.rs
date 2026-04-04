//! Context action contract tests
//!
//! Locks the exact IDs, titles, sections, and URI contracts for all context
//! actions exposed through the AI command bar. Tests fail if any context
//! action is removed, renamed, moved out of the "Context" section, or
//! dispatches to a changed URI.
//!
//! Run with: `cargo test --quiet context_action_contract`

use super::builders::get_ai_command_bar_actions;
use super::types::Action;
use crate::ai::message_parts::AiContextPart;

// =========================================================================
// Helpers
// =========================================================================

fn find_action<'a>(actions: &'a [Action], id: &str) -> Option<&'a Action> {
    actions.iter().find(|a| a.id == id)
}

/// The canonical set of context action IDs in their expected order.
const CONTEXT_ACTION_IDS: &[&str] = &[
    "chat:add_current_context",
    "chat:add_context_full",
    "chat:add_selection_context",
    "chat:add_browser_context",
    "chat:add_window_context",
    "chat:add_context_diagnostics",
    "chat:inspect_context",
    "chat:clear_context",
];

/// Resolves an action suffix to its dispatched AiContextPart via the
/// canonical context contract. `None` means the action has a side-effect
/// (clear/inspect) rather than producing a part.
fn expected_dispatch(suffix: &str) -> Option<AiContextPart> {
    use crate::ai::context_contract::{is_clear_context_action, ContextAttachmentKind};

    if let Some(kind) = ContextAttachmentKind::from_action_id(suffix) {
        return Some(kind.part());
    }

    if is_clear_context_action(suffix) || suffix == "inspect_context" {
        return None;
    }

    panic!("unexpected context action suffix: {suffix}");
}

/// Expected (id, title, section) triples for every context action.
const CONTEXT_ACTION_METADATA: &[(&str, &str, &str)] = &[
    ("chat:add_current_context", "Attach Current Context", "Context"),
    ("chat:add_context_full", "Attach Full Context", "Context"),
    ("chat:add_selection_context", "Attach Selected Text", "Context"),
    ("chat:add_browser_context", "Attach Browser URL", "Context"),
    ("chat:add_window_context", "Attach Focused Window", "Context"),
    ("chat:add_context_diagnostics", "Attach Context Diagnostics", "Context"),
    ("chat:inspect_context", "Inspect Context Receipt", "Context"),
    ("chat:clear_context", "Clear Context", "Context"),
];

// =========================================================================
// Tests
// =========================================================================

/// All 7 context actions exist in the builder output with exact IDs.
#[test]
fn context_action_contract_all_ids_present() {
    let actions = get_ai_command_bar_actions();

    for &expected_id in CONTEXT_ACTION_IDS {
        assert!(
            find_action(&actions, expected_id).is_some(),
            "missing context action: {expected_id}"
        );
    }
}

/// Each context action has the exact expected title and is in the "Context" section.
#[test]
fn context_action_contract_metadata_exact() {
    let actions = get_ai_command_bar_actions();

    for &(id, expected_title, expected_section) in CONTEXT_ACTION_METADATA {
        let action = find_action(&actions, id)
            .unwrap_or_else(|| panic!("action {id} not found"));
        assert_eq!(
            action.title, expected_title,
            "title mismatch for {id}: got {:?}",
            action.title
        );
        assert_eq!(
            action.section.as_deref(),
            Some(expected_section),
            "section mismatch for {id}: got {:?}",
            action.section
        );
    }
}

/// No context action IDs appear outside the "Context" section.
#[test]
fn context_action_contract_section_exclusive() {
    let actions = get_ai_command_bar_actions();

    for action in &actions {
        if CONTEXT_ACTION_IDS.contains(&action.id.as_str()) {
            assert_eq!(
                action.section.as_deref(),
                Some("Context"),
                "context action {} unexpectedly in section {:?}",
                action.id,
                action.section
            );
        }
    }
}

/// The diagnostics action dispatches to one explicit URI and the test
/// fails on any URI drift.
#[test]
fn context_action_contract_diagnostics_uri() {
    let part = expected_dispatch("add_context_diagnostics")
        .expect("diagnostics must produce a context part");
    match &part {
        AiContextPart::ResourceUri { uri, .. } => {
            assert_eq!(
                uri, "kit://context?diagnostics=1",
                "diagnostics URI contract violated"
            );
        }
        other => panic!("expected ResourceUri, got {other:?}"),
    }
}

/// Every add_* action maps to a ResourceUri; clear_context maps to None.
#[test]
fn context_action_contract_dispatch_uri_parity() {
    for &id in CONTEXT_ACTION_IDS {
        let suffix = id.strip_prefix("chat:").expect("all context IDs start with chat:");
        let dispatch = expected_dispatch(suffix);

        if suffix == "clear_context" || suffix == "inspect_context" {
            assert!(
                dispatch.is_none(),
                "{suffix} must not produce a context part"
            );
        } else {
            let part = dispatch.unwrap_or_else(|| {
                panic!("action {id} must dispatch a context part")
            });
            match &part {
                AiContextPart::ResourceUri { uri, .. } => {
                    assert!(
                        uri.starts_with("kit://context"),
                        "action {id} URI must start with kit://context, got: {uri}"
                    );
                }
                other => panic!("action {id} expected ResourceUri, got {other:?}"),
            }
        }
    }
}

/// Context action count stays aligned with the canonical context catalog.
#[test]
fn context_action_contract_count() {
    let actions = get_ai_command_bar_actions();

    let context_actions: Vec<_> = actions
        .iter()
        .filter(|a| a.section.as_deref() == Some("Context"))
        .collect();

    assert_eq!(
        context_actions.len(),
        20,
        "expected exactly 20 context actions, found {}: {:?}",
        context_actions.len(),
        context_actions.iter().map(|a| &a.id).collect::<Vec<_>>()
    );
}

/// Context actions appear contiguously in the builder output (no interleaving
/// with non-context actions).
#[test]
fn context_action_contract_contiguous() {
    let actions = get_ai_command_bar_actions();

    let positions: Vec<usize> = actions
        .iter()
        .enumerate()
        .filter(|(_, a)| a.section.as_deref() == Some("Context"))
        .map(|(i, _)| i)
        .collect();

    assert!(!positions.is_empty(), "no context actions found");

    let first = positions[0];
    let last = *positions.last().expect("non-empty");
    assert_eq!(
        last - first + 1,
        positions.len(),
        "context actions are not contiguous: positions = {positions:?}"
    );
}

/// The exact URI contract for every dispatched context part matches command_bar.rs.
/// This is the definitive parity test: if command_bar.rs changes a URI, this
/// test must be updated in lockstep.
#[test]
fn context_action_contract_full_uri_snapshot() {
    let expected: Vec<(&str, &str, &str)> = vec![
        (
            "add_current_context",
            "kit://context?profile=minimal",
            "Current Context",
        ),
        (
            "add_context_full",
            "kit://context",
            "Current Context (Full)",
        ),
        (
            "add_selection_context",
            "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
            "Selection",
        ),
        (
            "add_browser_context",
            "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0",
            "Browser URL",
        ),
        (
            "add_window_context",
            "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1",
            "Focused Window",
        ),
        (
            "add_context_diagnostics",
            "kit://context?diagnostics=1",
            "Context Diagnostics",
        ),
    ];

    for (suffix, expected_uri, expected_label) in expected {
        let part = expected_dispatch(suffix)
            .unwrap_or_else(|| panic!("{suffix} must produce a context part"));
        match part {
            AiContextPart::ResourceUri { uri, label } => {
                assert_eq!(
                    uri, expected_uri,
                    "URI mismatch for {suffix}"
                );
                assert_eq!(
                    label, expected_label,
                    "label mismatch for {suffix}"
                );
            }
            other => panic!("{suffix}: expected ResourceUri, got {other:?}"),
        }
    }
}

/// Final JSON summary of action ID -> title -> section -> dispatched URI.
/// This test prints the summary so an agent can parse and verify correctness.
#[test]
fn context_action_contract_json_summary() {
    let actions = get_ai_command_bar_actions();

    let mut entries = Vec::new();
    for &(id, title, section) in CONTEXT_ACTION_METADATA {
        let suffix = id.strip_prefix("chat:").expect("prefix");
        let uri = expected_dispatch(suffix).map(|part| match part {
            AiContextPart::ResourceUri { uri, .. } => uri,
            AiContextPart::FilePath { path, .. } => path,
            AiContextPart::FocusedTarget { target, .. } => target.semantic_id,
            AiContextPart::AmbientContext { label } => label,
        });

        // Verify builder still has this action
        assert!(
            find_action(&actions, id).is_some(),
            "summary: action {id} missing from builder"
        );

        entries.push(format!(
            r#"  {{"id": "{id}", "title": "{title}", "section": "{section}", "uri": {}}}"#,
            match &uri {
                Some(u) => format!(r#""{u}""#),
                None => "null".to_string(),
            }
        ));
    }

    let summary = format!("[\n{}\n]", entries.join(",\n"));
    // Emit as structured output for agent consumption
    tracing::info!(
        summary = %summary,
        "context_action_contract_json_summary"
    );
    // Also verify it parses as valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&summary).expect("summary must be valid JSON");
    assert!(parsed.is_array(), "summary must be a JSON array");
    assert_eq!(
        parsed.as_array().expect("array").len(),
        CONTEXT_ACTION_METADATA.len(),
        "summary entry count mismatch"
    );
}
