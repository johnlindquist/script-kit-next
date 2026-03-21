use super::types::{ContextPickerItemKind, ContextPickerState};
use super::{build_picker_items, score_builtin};
use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};

#[test]
fn context_picker_empty_query_returns_all_builtins() {
    let items = build_picker_items("");
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
    let items = build_picker_items("sel");
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
    let items_a = build_picker_items("con");
    let items_b = build_picker_items("con");

    assert_eq!(items_a.len(), items_b.len(), "Same query must produce same count");

    for (a, b) in items_a.iter().zip(items_b.iter()) {
        assert_eq!(a.id, b.id, "Same query must produce same order");
        assert_eq!(a.score, b.score, "Same query must produce same scores");
    }
}

#[test]
fn context_picker_builtins_seeded_from_specs() {
    let items = build_picker_items("");

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
    let items = build_picker_items("selection");
    let selection_item = items
        .iter()
        .find(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Selection)))
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
    let items = build_picker_items("");
    let count = items.len();
    assert!(count >= 2, "Need at least 2 items for navigation test");

    let mut state = ContextPickerState::new(String::new(), items);
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
    let items = build_picker_items("zzzznonexistent");
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
    let items = build_picker_items("con");

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
    let items = build_picker_items("diag");
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
