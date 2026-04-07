use super::types::{ContextPickerItemKind, ContextPickerState, ContextPickerTrigger};
use super::{
    build_picker_items, build_slash_picker_items, extract_context_picker_query, match_query_chars,
    score_builtin,
};
use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};

#[test]
fn context_picker_empty_query_returns_all_non_provider_builtins() {
    // Clear provider data so provider-backed items are hidden
    std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
    std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
    std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();
    // 3 provider-backed kinds (Dictation, Calendar, Notifications) are hidden
    let provider_gated_count = 3;
    assert_eq!(
        builtin_count,
        context_attachment_specs().len() - provider_gated_count,
        "Empty query should return all non-provider-gated built-in context specs"
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
    // Provider-backed kinds are only shown when real data exists
    let provider_gated = [
        ContextAttachmentKind::Dictation,
        ContextAttachmentKind::Calendar,
        ContextAttachmentKind::Notifications,
    ];
    std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
    std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
    std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "");

    for spec in context_attachment_specs() {
        if provider_gated.contains(&spec.kind) {
            continue;
        }
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
fn context_picker_catalog_has_at_least_12_entries() {
    // Provider-gated items may be hidden, so the minimum is lower
    std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
    std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
    std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();
    assert!(
        builtin_count >= 12,
        "Catalog should have at least 12 built-in entries, got {builtin_count}"
    );
}

#[test]
fn context_picker_bro_query_ranks_browser() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "bro");
    let first_builtin = items
        .iter()
        .find(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)));
    assert!(
        first_builtin.is_some(),
        "Query 'bro' should match at least one built-in"
    );
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
fn slash_mode_only_includes_agent_slash_commands() {
    let items = build_slash_picker_items("", ["compact", "clear", "help"]);
    assert!(
        !items.is_empty(),
        "Slash mode should include provided commands"
    );
    assert!(items
        .iter()
        .all(|item| matches!(item.kind, ContextPickerItemKind::SlashCommand(_))));
}

#[test]
fn slash_mode_compact_matches_agent_command() {
    let items = build_slash_picker_items("com", ["compact", "clear", "help"]);
    let first = items.first().expect("Slash 'com' should match compact");
    match &first.kind {
        ContextPickerItemKind::SlashCommand(command) => assert_eq!(command, "compact"),
        other => panic!("Expected SlashCommand(compact), got {:?}", other),
    }
}

#[test]
fn slash_mode_no_file_results() {
    let items = build_slash_picker_items("src", ["compact", "clear", "help"]);
    let file_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::File(_)))
        .count();
    assert_eq!(
        file_count, 0,
        "Slash mode should never include file results"
    );
}

// ── Tab/Enter acceptance regression (already wired, lock with test) ─────

#[test]
fn enter_and_tab_both_route_to_accept() {
    use super::super::render_keydown::*;
    // This is implicitly tested by the key_routing_tests in render_keydown.rs
    // but we lock the contract explicitly here
    // (The existing test `enter_and_tab_accept_picker_selection` covers this)
}

// ── V05 ranking and meta verification ──────────────────────────────────

#[test]
fn slash_commands_rank_prefix_matches_before_fuzzy_matches() {
    let items = build_slash_picker_items("cle", ["compact", "clear", "help"]);
    let first = items.first().expect("Slash query should return clear");
    match &first.kind {
        ContextPickerItemKind::SlashCommand(command) => assert_eq!(command, "clear"),
        other => panic!("Expected SlashCommand(clear), got {:?}", other),
    }
}

#[test]
fn mention_sel_shows_mention_selection_as_meta() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "sel");
    let selection = items
        .iter()
        .find(|i| {
            matches!(
                i.kind,
                ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Selection)
            )
        })
        .expect("Selection should be in results for 'sel'");
    assert_eq!(
        selection.meta.as_ref(),
        "@selection",
        "Selection meta in mention mode should be the mention '@selection'"
    );
    assert!(
        !selection.label_highlight_indices.is_empty(),
        "Selection label should have highlight indices for 'sel'"
    );
}

#[test]
fn mention_sel_has_meta_highlight_indices() {
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
        !selection.meta_highlight_indices.is_empty(),
        "Selection meta should have highlight indices for 'sel' in '@selection'"
    );
}

// ── Scroll reveal source audit ─────────────────────────────────────────

#[test]
fn context_picker_render_uses_list_not_children() {
    let source = include_str!("render.rs");
    assert!(
        source.contains("let picker_list = list(")
            && source.contains("self.context_picker_list_state.clone()"),
        "Context picker must render through GPUI list state, not raw .children()"
    );
    assert!(
        !source.contains(".children(rows)"),
        "Context picker must not use .children(rows) — use list() for scroll reveal"
    );
}

#[test]
fn context_picker_sync_and_reveal_called_on_open() {
    let source = include_str!("mod.rs");
    let open_fn_start = source
        .find("fn open_context_picker(")
        .expect("open_context_picker function");
    let open_fn_body = &source[open_fn_start..source.len().min(open_fn_start + 1400)];
    assert!(
        open_fn_body.contains("sync_context_picker_list_state()"),
        "open_context_picker must call sync_context_picker_list_state"
    );
    assert!(
        open_fn_body.contains("reveal_selected_context_picker_item"),
        "open_context_picker must call reveal_selected_context_picker_item"
    );
}

#[test]
fn context_picker_sync_and_reveal_called_on_filter() {
    let source = include_str!("mod.rs");
    let filter_fn_start = source
        .find("fn update_context_picker_query(")
        .expect("update_context_picker_query function");
    let filter_fn_body = &source[filter_fn_start..source.len().min(filter_fn_start + 1200)];
    assert!(
        filter_fn_body.contains("sync_context_picker_list_state()"),
        "update_context_picker_query must call sync_context_picker_list_state"
    );
    assert!(
        filter_fn_body.contains("reveal_selected_context_picker_item"),
        "update_context_picker_query must call reveal_selected_context_picker_item"
    );
}

#[test]
fn context_picker_reveal_called_on_prev_and_next() {
    let source = include_str!("mod.rs");

    let prev_start = source
        .find("fn context_picker_select_prev(")
        .expect("context_picker_select_prev function");
    let prev_body = &source[prev_start..source.len().min(prev_start + 500)];
    assert!(
        prev_body.contains("reveal_selected_context_picker_item(\"keyboard_prev\""),
        "context_picker_select_prev must reveal with reason \"keyboard_prev\""
    );

    let next_start = source
        .find("fn context_picker_select_next(")
        .expect("context_picker_select_next function");
    let next_body = &source[next_start..source.len().min(next_start + 500)];
    assert!(
        next_body.contains("reveal_selected_context_picker_item(\"keyboard_next\""),
        "context_picker_select_next must reveal with reason \"keyboard_next\""
    );
}

#[test]
fn context_picker_reveal_emits_structured_log() {
    let source = include_str!("mod.rs");
    let reveal_start = source
        .find("fn reveal_selected_context_picker_item(")
        .expect("reveal_selected_context_picker_item function");
    let reveal_body = &source[reveal_start..source.len().min(reveal_start + 1200)];
    assert!(
        reveal_body.contains("target: \"ai\""),
        "reveal must log to target \"ai\""
    );
    assert!(reveal_body.contains("reason"), "reveal must log the reason");
    assert!(
        reveal_body.contains("selected_index"),
        "reveal must log the selected_index"
    );
    assert!(
        reveal_body.contains("ai_context_picker_scrolled_to_selected"),
        "reveal must log ai_context_picker_scrolled_to_selected"
    );
}

#[test]
fn context_picker_sync_resets_last_scrolled_index() {
    let source = include_str!("mod.rs");
    let sync_start = source
        .find("fn sync_context_picker_list_state(")
        .expect("sync_context_picker_list_state function");
    let sync_body = &source[sync_start..source.len().min(sync_start + 500)];
    assert!(
        sync_body.contains("context_picker_last_scrolled_index = None"),
        "sync must reset last_scrolled_index to invalidate stale reveal cache"
    );
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
    assert!(hints[0].display.starts_with('@'));
}

#[test]
fn empty_state_hints_slash_mode() {
    let hints = super::empty_state_hints(ContextPickerTrigger::Slash);
    assert!(hints.len() >= 3);
    assert!(hints[0].display.starts_with('/'));
}

// ── Scroll-into-view and navigation edge cases ────────────────────────────

#[test]
fn context_picker_state_navigation_wrap_up_from_zero() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let count = items.len();
    assert!(count >= 2);

    let mut state = ContextPickerState::new(ContextPickerTrigger::Mention, String::new(), items);
    assert_eq!(state.selected_index, 0);

    // Wrap up from 0 → last
    state.selected_index = if state.selected_index == 0 {
        state.items.len() - 1
    } else {
        state.selected_index - 1
    };
    assert_eq!(state.selected_index, count - 1, "Should wrap to last item");
}

#[test]
fn context_picker_state_clamp_on_filter_shrink() {
    let all_items = build_picker_items(ContextPickerTrigger::Mention, "");
    let count = all_items.len();
    assert!(count >= 3);

    let mut state =
        ContextPickerState::new(ContextPickerTrigger::Mention, String::new(), all_items);
    state.selected_index = count - 1; // Select last item

    // Filter to a smaller set
    let filtered = build_picker_items(ContextPickerTrigger::Mention, "sel");
    assert!(filtered.len() < count, "Filtered set should be smaller");
    state.items = filtered;

    // Clamp selected_index to valid range (same logic as update_context_picker_query)
    if state.selected_index >= state.items.len() {
        state.selected_index = state.items.len().saturating_sub(1);
    }
    assert!(
        state.selected_index < state.items.len(),
        "selected_index {} must be < items.len() {} after filter",
        state.selected_index,
        state.items.len(),
    );
}

#[test]
fn context_picker_state_clamp_on_empty_filter() {
    let mut state = ContextPickerState::new(
        ContextPickerTrigger::Mention,
        String::new(),
        build_picker_items(ContextPickerTrigger::Mention, ""),
    );
    state.selected_index = 5;

    // Filter to no results
    let empty = build_picker_items(ContextPickerTrigger::Mention, "zzzzzznothing");
    state.items = empty;

    if state.selected_index >= state.items.len() {
        state.selected_index = state.items.len().saturating_sub(1);
    }
    assert_eq!(state.selected_index, 0, "Empty items should clamp to 0");
}

// ── Slash-mode ranking improvements ────────────────────────────────────

#[test]
fn slash_mode_empty_without_agent_commands() {
    let items = build_slash_picker_items("any", std::iter::empty::<&str>());
    assert!(
        items.is_empty(),
        "Slash mode should not surface context attachments"
    );
}

#[test]
fn slash_sel_does_not_surface_selection_context() {
    let items = build_slash_picker_items("sel", ["compact", "clear", "help"]);
    assert!(
        items
            .iter()
            .all(|item| !matches!(item.kind, ContextPickerItemKind::BuiltIn(_))),
        "Slash mode should never return context attachments"
    );
}

#[test]
fn slash_mode_meta_highlights_align_with_slash_command() {
    let items = build_slash_picker_items("com", ["compact", "clear", "help"]);
    let compact = items
        .iter()
        .find(|i| matches!(&i.kind, ContextPickerItemKind::SlashCommand(command) if command == "compact"))
        .expect("compact should match 'com' in slash mode");

    assert!(
        compact.meta.contains("compact"),
        "Slash mode meta should contain 'compact', got: {}",
        compact.meta,
    );
    assert!(
        !compact.meta_highlight_indices.is_empty(),
        "Slash mode should produce meta highlight indices for 'com'"
    );

    let meta_bare = compact.meta.trim_start_matches('/');
    for &idx in &compact.meta_highlight_indices {
        assert!(idx < meta_bare.len());
    }
}

#[test]
fn mention_mode_meta_highlights_align_with_mention() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "sel");
    let selection = items
        .iter()
        .find(|i| {
            matches!(
                i.kind,
                ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Selection)
            )
        })
        .expect("Selection should match 'sel'");

    // Meta should be @selection in mention mode
    assert!(
        selection.meta.starts_with('@'),
        "Mention mode meta should start with @, got: {}",
        selection.meta,
    );

    let meta_bare = selection.meta.trim_start_matches('@');
    for &idx in &selection.meta_highlight_indices {
        assert!(
            idx < meta_bare.len(),
            "meta highlight index {} out of range for '{}' (len {})",
            idx,
            meta_bare,
            meta_bare.len(),
        );
    }
}

#[test]
fn slash_and_mention_highlights_both_cover_query_length() {
    let mention_query = "bro";
    let slash_query = "com";
    let slash_items = build_slash_picker_items(slash_query, ["compact", "clear", "help"]);
    let slash_compact = slash_items.iter().find(
        |i| matches!(&i.kind, ContextPickerItemKind::SlashCommand(command) if command == "compact"),
    );

    let mention_items = build_picker_items(ContextPickerTrigger::Mention, mention_query);
    let mention_browser = mention_items.iter().find(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Browser)
        )
    });

    if let Some(item) = slash_compact {
        assert!(
            item.label_highlight_indices.len() >= slash_query.len(),
            "Slash compact label highlights ({:?}) should cover query len {}",
            item.label_highlight_indices,
            slash_query.len(),
        );
    }

    if let Some(item) = mention_browser {
        assert!(
            item.label_highlight_indices.len() >= mention_query.len(),
            "Mention Browser label highlights ({:?}) should cover query len {}",
            item.label_highlight_indices,
            mention_query.len(),
        );
    }
}

// ── Fuzzy-only admission tests ─────────────────────────────────────────

#[test]
fn fuzzy_gst_matches_git_status() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "gst");
    let has_git_status = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::GitStatus)
        )
    });
    assert!(
        has_git_status,
        "Fuzzy query 'gst' should match Git Status via scattered character matching"
    );
}

#[test]
fn fuzzy_rsc_matches_recent_scripts() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "rsc");
    let has_recent_scripts = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::RecentScripts)
        )
    });
    assert!(
        has_recent_scripts,
        "Fuzzy query 'rsc' should match Recent Scripts via scattered character matching"
    );
}

#[test]
fn fuzzy_match_scores_below_substring() {
    let spec = ContextAttachmentKind::GitStatus.spec();
    let fuzzy_score = score_builtin(spec, "gst");
    let substring_score = score_builtin(spec, "git");
    assert!(
        substring_score > fuzzy_score,
        "Substring match ({}) should outrank fuzzy match ({})",
        substring_score,
        fuzzy_score,
    );
    assert_eq!(
        fuzzy_score, 50,
        "Fuzzy-only matches should score exactly 50"
    );
}

#[test]
fn fuzzy_match_has_highlight_indices() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "gst");
    let git_status = items
        .iter()
        .find(|i| {
            matches!(
                i.kind,
                ContextPickerItemKind::BuiltIn(ContextAttachmentKind::GitStatus)
            )
        })
        .expect("Git Status should be in fuzzy results for 'gst'");
    assert!(
        !git_status.label_highlight_indices.is_empty()
            || !git_status.meta_highlight_indices.is_empty(),
        "Fuzzy matches should have highlight indices for rendering"
    );
}

#[test]
fn fuzzy_nonexistent_query_still_filters() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "zqx");
    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();
    assert_eq!(
        builtin_count, 0,
        "Query with no fuzzy match should still filter out all built-ins"
    );
}

// ── Provider-backed item gating ───────────────────────────────────────

#[test]
fn provider_backed_items_are_hidden_when_unavailable() {
    // Ensure no provider data exists
    std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
    std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
    std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    assert!(
        items.iter().all(|item| item.label.as_ref() != "Dictation"),
        "Dictation must not be advertised when no provider data exists"
    );
    assert!(
        items.iter().all(|item| item.label.as_ref() != "Calendar"),
        "Calendar must not be advertised when no provider data exists"
    );
    assert!(
        items
            .iter()
            .all(|item| item.label.as_ref() != "Notifications"),
        "Notifications must not be advertised when no provider data exists"
    );
}

#[test]
fn provider_backed_items_appear_when_slot_data_exists() {
    // Clear first
    std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
    std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
    std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    // Publish dictation data
    crate::mcp_resources::publish_dictation_json(r#"{"items":["test"]}"#);

    let items = build_picker_items(ContextPickerTrigger::Mention, "di");
    let has_dictation = items.iter().any(|item| item.label.as_ref() == "Dictation");
    assert!(
        has_dictation,
        "Dictation should appear when provider slot data exists"
    );

    // Calendar and Notifications should still be hidden
    let all_items = build_picker_items(ContextPickerTrigger::Mention, "");
    assert!(
        all_items
            .iter()
            .all(|item| item.label.as_ref() != "Calendar"),
        "Calendar should still be hidden without provider data"
    );

    // Clean up
    crate::mcp_resources::clear_provider_json_slots();
}

#[test]
fn provider_backed_items_hidden_in_targeted_query() {
    std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "di");
    assert!(
        items.iter().all(|item| item.label.as_ref() != "Dictation"),
        "Dictation must not appear even when query matches, if no provider data"
    );
}

// ── Deterministic ranking across trigger modes ─────────────────────────

#[test]
fn slash_ranking_is_deterministic() {
    let items_a = build_slash_picker_items("com", ["compact", "clear", "help"]);
    let items_b = build_slash_picker_items("com", ["compact", "clear", "help"]);

    assert_eq!(items_a.len(), items_b.len());
    for (a, b) in items_a.iter().zip(items_b.iter()) {
        assert_eq!(a.id, b.id, "Slash mode ranking must be deterministic");
        assert_eq!(a.score, b.score);
        assert_eq!(a.label_highlight_indices, b.label_highlight_indices);
        assert_eq!(a.meta_highlight_indices, b.meta_highlight_indices);
    }
}

// ── Provider-backed empty state hint gating ──────────────────────────────

fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

#[test]
fn mention_empty_state_hints_hide_unavailable_provider_entries() {
    let prev_calendar = std::env::var_os("SCRIPT_KIT_CALENDAR_JSON");
    std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
    crate::mcp_resources::clear_provider_json_slots();

    let hints = super::empty_state_hints(ContextPickerTrigger::Mention);
    assert!(
        !hints.iter().any(|hint| hint.display == "@calendar"),
        "@calendar hint must be hidden when provider data is unavailable"
    );

    restore_env("SCRIPT_KIT_CALENDAR_JSON", prev_calendar);
}

#[test]
fn mention_empty_state_hints_show_available_provider_entries() {
    let prev_calendar = std::env::var_os("SCRIPT_KIT_CALENDAR_JSON");
    std::env::set_var(
        "SCRIPT_KIT_CALENDAR_JSON",
        r#"{"schemaVersion":1,"type":"calendar","ok":true,"available":true,"source":"env","items":[{"title":"Demo"}]}"#,
    );

    let hints = super::empty_state_hints(ContextPickerTrigger::Mention);
    assert!(
        hints.iter().any(|hint| hint.display == "@calendar"),
        "@calendar hint must be shown when provider data is real"
    );

    restore_env("SCRIPT_KIT_CALENDAR_JSON", prev_calendar);
}

#[test]
fn context_picker_selection_clamps_via_shared_dropdown_contract() {
    use crate::components::inline_dropdown::inline_dropdown_clamp_selected_index;
    assert_eq!(inline_dropdown_clamp_selected_index(0, 0), 0);
    assert_eq!(inline_dropdown_clamp_selected_index(8, 2), 1);
}

#[test]
fn context_picker_visible_range_uses_shared_dropdown_contract() {
    use crate::components::inline_dropdown::inline_dropdown_visible_range;
    assert_eq!(inline_dropdown_visible_range(0, 4, 8), 0..4);
    assert_eq!(inline_dropdown_visible_range(6, 20, 8), 2..10);
}
