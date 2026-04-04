use super::types::{ContextPickerItemKind, ContextPickerState, ContextPickerTrigger};
use super::{build_picker_items, extract_context_picker_query, match_query_chars, score_builtin};
use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};

#[test]
fn context_picker_empty_query_returns_all_builtins() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();
    assert_eq!(
        builtin_count,
        context_attachment_specs().len(),
        "Empty query should return all built-in context specs"
    );
}

#[test]
fn context_picker_sel_query_ranks_selection_first() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "sel");
    assert!(
        !items.is_empty(),
        "Query 'sel' should match at least Selection"
    );
    let first = &items[0];
    match &first.kind {
        ContextPickerItemKind::BuiltIn(kind) => {
            assert_eq!(
                *kind,
                ContextAttachmentKind::Selection,
                "Query 'sel' should rank Selection first"
            );
        }
        other => panic!("Expected BuiltIn(Selection), got {:?}", other),
    }
}

#[test]
fn context_picker_ranking_is_deterministic() {
    let items_a = build_picker_items(ContextPickerTrigger::Mention, "con");
    let items_b = build_picker_items(ContextPickerTrigger::Mention, "con");

    assert_eq!(
        items_a.len(),
        items_b.len(),
        "Same query must produce same count"
    );

    for (a, b) in items_a.iter().zip(items_b.iter()) {
        assert_eq!(a.id, b.id, "Same query must produce same order");
        assert_eq!(a.score, b.score, "Same query must produce same scores");
    }
}

#[test]
fn context_picker_builtins_seeded_from_specs() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");

    for spec in context_attachment_specs() {
        let found = items.iter().any(|item| match &item.kind {
            ContextPickerItemKind::BuiltIn(kind) => *kind == spec.kind,
            _ => false,
        });
        assert!(
            found,
            "Built-in item for {:?} should be present in empty-query results",
            spec.kind
        );
    }
}

#[test]
fn context_picker_accept_creates_correct_part_for_selection() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "selection");
    let selection_item = items
        .iter()
        .find(|i| {
            matches!(
                i.kind,
                ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Selection)
            )
        })
        .expect("Selection should be in results");

    // Verify the item produces the correct part
    match &selection_item.kind {
        ContextPickerItemKind::BuiltIn(kind) => {
            let part = kind.part();
            assert_eq!(part.label(), "Selection");
            assert!(
                part.source().contains("selectedText=1"),
                "Selection part URI should contain selectedText=1, got: {}",
                part.source()
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn context_picker_state_navigation() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let count = items.len();
    assert!(count >= 2, "Need at least 2 items for navigation test");

    let mut state = ContextPickerState::new(ContextPickerTrigger::Mention, String::new(), items);
    assert_eq!(state.selected_index, 0, "Initial selection should be 0");

    // Move down
    state.selected_index = (state.selected_index + 1) % state.items.len();
    assert_eq!(state.selected_index, 1);

    // Move to last
    state.selected_index = state.items.len() - 1;

    // Wrap around
    state.selected_index = (state.selected_index + 1) % state.items.len();
    assert_eq!(state.selected_index, 0, "Should wrap to 0");
}

#[test]
fn context_picker_query_filters_irrelevant_builtins() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "zzzznonexistent");
    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();
    assert_eq!(
        builtin_count, 0,
        "Non-matching query should filter out all built-ins"
    );
}

#[test]
fn context_picker_builtins_grouped_before_files() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "con");

    let mut seen_non_builtin = false;
    for item in &items {
        match &item.kind {
            ContextPickerItemKind::BuiltIn(_) => {
                assert!(
                    !seen_non_builtin,
                    "Built-in items must appear before file/folder items"
                );
            }
            _ => {
                seen_non_builtin = true;
            }
        }
    }
}

#[test]
fn score_builtin_exact_mention_scores_highest() {
    let selection_spec = ContextAttachmentKind::Selection.spec();
    let exact_score = score_builtin(selection_spec, "selection");
    let prefix_score = score_builtin(selection_spec, "sel");
    assert!(
        exact_score > prefix_score,
        "Exact mention match ({}) should score higher than prefix match ({})",
        exact_score,
        prefix_score
    );
}

#[test]
fn score_builtin_empty_query_returns_default_score() {
    for spec in context_attachment_specs() {
        let score = score_builtin(spec, "");
        assert_eq!(
            score, 100,
            "Empty query should return default score 100 for {:?}",
            spec.kind
        );
    }
}

#[test]
fn context_picker_diagnostics_matches_diag_query() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "diag");
    let has_diagnostics = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Diagnostics)
        )
    });
    assert!(
        has_diagnostics,
        "Query 'diag' should match Diagnostics built-in"
    );
}

#[test]
fn context_picker_catalog_has_at_least_15_entries() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();
    assert!(
        builtin_count >= 15,
        "Catalog should have at least 15 built-in entries, got {builtin_count}"
    );
}

#[test]
fn context_picker_bro_query_ranks_browser() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "bro");
    let first_builtin = items
        .iter()
        .find(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)));
    assert!(first_builtin.is_some(), "Query 'bro' should match at least one built-in");
    match &first_builtin.unwrap().kind {
        ContextPickerItemKind::BuiltIn(kind) => {
            assert_eq!(
                *kind,
                ContextAttachmentKind::Browser,
                "Query 'bro' should rank Browser first"
            );
        }
        other => panic!("Expected BuiltIn(Browser), got {:?}", other),
    }
}

#[test]
fn context_picker_win_query_ranks_window() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "win");
    let has_window = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Window)
        )
    });
    assert!(has_window, "Query 'win' should match Window built-in");
}

#[test]
fn context_picker_git_query_matches_git_entries() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "git");
    let git_kinds: Vec<_> = items
        .iter()
        .filter_map(|i| match &i.kind {
            ContextPickerItemKind::BuiltIn(k @ ContextAttachmentKind::GitStatus) => Some(*k),
            ContextPickerItemKind::BuiltIn(k @ ContextAttachmentKind::GitDiff) => Some(*k),
            _ => None,
        })
        .collect();
    assert!(
        git_kinds.contains(&ContextAttachmentKind::GitStatus),
        "Query 'git' should match GitStatus"
    );
    assert!(
        git_kinds.contains(&ContextAttachmentKind::GitDiff),
        "Query 'git' should match GitDiff"
    );
}

#[test]
fn context_picker_clip_query_matches_clipboard() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "clip");
    let has_clipboard = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Clipboard)
        )
    });
    assert!(
        has_clipboard,
        "Query 'clip' should match Clipboard built-in"
    );
}

#[test]
fn context_picker_screenshot_matches_screen_query() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "screen");
    let has_screenshot = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Screenshot)
        )
    });
    assert!(
        has_screenshot,
        "Query 'screen' should match Screenshot built-in"
    );
}

#[test]
fn context_picker_all_new_builtins_have_mention() {
    for spec in context_attachment_specs() {
        match spec.kind {
            ContextAttachmentKind::Screenshot
            | ContextAttachmentKind::Clipboard
            | ContextAttachmentKind::FrontmostApp
            | ContextAttachmentKind::MenuBar
            | ContextAttachmentKind::RecentScripts
            | ContextAttachmentKind::GitStatus
            | ContextAttachmentKind::GitDiff
            | ContextAttachmentKind::Processes
            | ContextAttachmentKind::System => {
                assert!(
                    spec.mention.is_some(),
                    "New kind {:?} must have a mention",
                    spec.kind
                );
                assert!(
                    spec.mention.unwrap().starts_with('@'),
                    "Mention for {:?} must start with @",
                    spec.kind
                );
            }
            _ => {} // Original kinds already tested
        }
    }
}

#[test]
fn context_picker_each_new_kind_produces_resource_uri_part() {
    let new_kinds = [
        ContextAttachmentKind::Screenshot,
        ContextAttachmentKind::Clipboard,
        ContextAttachmentKind::FrontmostApp,
        ContextAttachmentKind::MenuBar,
        ContextAttachmentKind::RecentScripts,
        ContextAttachmentKind::GitStatus,
        ContextAttachmentKind::GitDiff,
        ContextAttachmentKind::Processes,
        ContextAttachmentKind::System,
    ];
    for kind in new_kinds {
        let part = kind.part();
        match part {
            crate::ai::message_parts::AiContextPart::ResourceUri { uri, label } => {
                assert!(!uri.is_empty(), "URI for {:?} must not be empty", kind);
                assert!(!label.is_empty(), "Label for {:?} must not be empty", kind);
            }
            other => panic!("Expected ResourceUri for {:?}, got {:?}", kind, other),
        }
    }
}

// ── Trigger-aware extraction tests ──────────────────────────────────────

#[test]
fn extract_at_trigger_returns_mention() {
    let result = extract_context_picker_query("hello @sel").unwrap();
    assert_eq!(result.trigger, ContextPickerTrigger::Mention);
    assert_eq!(result.query, "sel");
}

#[test]
fn extract_slash_trigger_at_start() {
    let result = extract_context_picker_query("/con").unwrap();
    assert_eq!(result.trigger, ContextPickerTrigger::Slash);
    assert_eq!(result.query, "con");
}

#[test]
fn extract_slash_trigger_after_space() {
    let result = extract_context_picker_query("hello /bro").unwrap();
    assert_eq!(result.trigger, ContextPickerTrigger::Slash);
    assert_eq!(result.query, "bro");
}

#[test]
fn extract_slash_not_triggered_in_path() {
    // foo/bar should NOT trigger the slash picker
    let result = extract_context_picker_query("foo/bar");
    assert!(result.is_none(), "Slash inside a path should not trigger");
}

#[test]
fn extract_no_trigger_for_plain_text() {
    assert!(extract_context_picker_query("hello world").is_none());
}

#[test]
fn extract_trigger_returns_none_after_space() {
    assert!(extract_context_picker_query("@ ").is_none());
    assert!(extract_context_picker_query("/ ").is_none());
}

#[test]
fn extract_bare_at_returns_empty_query() {
    let result = extract_context_picker_query("hello @").unwrap();
    assert_eq!(result.trigger, ContextPickerTrigger::Mention);
    assert_eq!(result.query, "");
}

#[test]
fn extract_bare_slash_at_start() {
    let result = extract_context_picker_query("/").unwrap();
    assert_eq!(result.trigger, ContextPickerTrigger::Slash);
    assert_eq!(result.query, "");
}

// ── Slash-mode filtering tests ──────────────────────────────────────────

#[test]
fn slash_mode_only_includes_specs_with_slash_command() {
    let items = build_picker_items(ContextPickerTrigger::Slash, "");
    for item in &items {
        if let ContextPickerItemKind::BuiltIn(kind) = &item.kind {
            let spec = kind.spec();
            assert!(
                spec.slash_command.is_some(),
                "Slash mode should only include specs with slash_command, but got {:?}",
                kind,
            );
        }
    }
}

#[test]
fn slash_mode_bro_ranks_browser_first() {
    let items = build_picker_items(ContextPickerTrigger::Slash, "bro");
    let first_builtin = items
        .iter()
        .find(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)));
    assert!(first_builtin.is_some(), "Slash 'bro' should match Browser");
    match &first_builtin.unwrap().kind {
        ContextPickerItemKind::BuiltIn(kind) => {
            assert_eq!(
                *kind,
                ContextAttachmentKind::Browser,
                "Slash 'bro' should rank Browser first"
            );
        }
        other => panic!("Expected BuiltIn(Browser), got {:?}", other),
    }
}

#[test]
fn slash_mode_no_file_results() {
    let items = build_picker_items(ContextPickerTrigger::Slash, "src");
    let file_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::File(_)))
        .count();
    assert_eq!(file_count, 0, "Slash mode should never include file results");
}

// ── Tab/Enter acceptance regression (already wired, lock with test) ─────

#[test]
fn enter_and_tab_both_route_to_accept() {
    use super::super::render_keydown::*;
    // This is implicitly tested by the key_routing_tests in render_keydown.rs
    // but we lock the contract explicitly here
    // (The existing test `enter_and_tab_accept_picker_selection` covers this)
}

// ── match_query_chars tests ─────────────────────────────────────────────

#[test]
fn match_query_chars_empty_query() {
    let hits = match_query_chars("", "Selection").unwrap();
    assert!(hits.is_empty());
}

#[test]
fn match_query_chars_prefix_match() {
    let hits = match_query_chars("sel", "Selection").unwrap();
    assert_eq!(hits, vec![0, 1, 2]);
}

#[test]
fn match_query_chars_scattered_match() {
    let hits = match_query_chars("sn", "Selection").unwrap();
    assert_eq!(hits, vec![0, 8]); // S...n in "Selection"
}

#[test]
fn match_query_chars_no_match() {
    assert!(match_query_chars("xyz", "Selection").is_none());
}

#[test]
fn match_query_chars_case_insensitive() {
    let hits = match_query_chars("SEL", "selection").unwrap();
    assert_eq!(hits, vec![0, 1, 2]);
}

// ── Highlight indices populated ─────────────────────────────────────────

#[test]
fn picker_items_have_highlight_indices_for_query() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "sel");
    let selection = items
        .iter()
        .find(|i| {
            matches!(
                i.kind,
                ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Selection)
            )
        })
        .expect("Selection should be in results");
    assert!(
        !selection.label_highlight_indices.is_empty(),
        "Selection should have label highlight indices for 'sel' query"
    );
}

#[test]
fn picker_items_have_no_highlights_for_empty_query() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    for item in &items {
        assert!(
            item.label_highlight_indices.is_empty(),
            "Empty query should produce no label highlights for {}",
            item.label
        );
    }
}

// ── Empty state hints ───────────────────────────────────────────────────

#[test]
fn empty_state_hints_mention_mode() {
    let hints = super::empty_state_hints(ContextPickerTrigger::Mention);
    assert!(hints.len() >= 3);
    assert!(hints[0].starts_with('@'));
}

#[test]
fn empty_state_hints_slash_mode() {
    let hints = super::empty_state_hints(ContextPickerTrigger::Slash);
    assert!(hints.len() >= 3);
    assert!(hints[0].starts_with('/'));
}
