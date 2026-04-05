//! Integration tests for the inline context picker.
//!
//! Validates deterministic ranking, built-in spec seeding, query filtering,
//! and the contract between picker items and `AiContextPart` creation.

use script_kit_gpui::ai::{
    build_picker_items, build_slash_picker_items, context_attachment_specs, score_builtin,
    AiContextPart, ContextAttachmentKind, ContextPickerItemKind, ContextPickerState,
    ContextPickerTrigger,
};

static PROVIDER_SLOT_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock_provider_slots() -> std::sync::MutexGuard<'static, ()> {
    PROVIDER_SLOT_TEST_LOCK
        .lock()
        .expect("provider slot test lock must not be poisoned")
}

/// Seed provider JSON slots so provider-backed builtins appear in the picker.
fn seed_provider_slots() {
    script_kit_gpui::mcp_resources::clear_provider_json_slots();
    script_kit_gpui::mcp_resources::publish_dictation_json(r#"{"transcription":"test"}"#);
    script_kit_gpui::mcp_resources::publish_calendar_json(r#"{"events":[]}"#);
    script_kit_gpui::mcp_resources::publish_notifications_json(r#"{"notifications":[]}"#);
}

// ---------- Deterministic picker ranking ----------

#[test]
fn picker_ranking_is_deterministic_across_calls() {
    let _guard = lock_provider_slots();
    // Seed provider slots for stable state — other tests may clear/seed
    // global provider slots concurrently in the same process.
    seed_provider_slots();
    let items_a = build_picker_items(ContextPickerTrigger::Mention, "con");
    let items_b = build_picker_items(ContextPickerTrigger::Mention, "con");

    assert_eq!(
        items_a.len(),
        items_b.len(),
        "Same query must produce same item count"
    );

    for (a, b) in items_a.iter().zip(items_b.iter()) {
        assert_eq!(a.id, b.id, "Same query must produce same item order");
        assert_eq!(a.score, b.score, "Same query must produce same scores");
    }
}

#[test]
fn empty_query_returns_all_builtins_deterministically() {
    let _guard = lock_provider_slots();
    seed_provider_slots();
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let specs = context_attachment_specs();

    let builtin_count = items
        .iter()
        .filter(|i| matches!(i.kind, ContextPickerItemKind::BuiltIn(_)))
        .count();

    assert_eq!(
        builtin_count,
        specs.len(),
        "Empty query should return exactly all built-in context specs"
    );

    // Verify each spec is present
    for spec in specs {
        let found = items.iter().any(|item| match &item.kind {
            ContextPickerItemKind::BuiltIn(kind) => *kind == spec.kind,
            _ => false,
        });
        assert!(found, "Built-in item for {:?} should be present", spec.kind);
    }
}

// ---------- Query filtering ----------

#[test]
fn sel_query_ranks_selection_first() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "sel");
    assert!(!items.is_empty(), "'sel' should match at least Selection");

    match &items[0].kind {
        ContextPickerItemKind::BuiltIn(kind) => {
            assert_eq!(
                *kind,
                ContextAttachmentKind::Selection,
                "'sel' should rank Selection first"
            );
        }
        other => panic!("Expected BuiltIn(Selection), got {:?}", other),
    }
}

#[test]
fn nonexistent_query_returns_no_builtins() {
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
fn diag_query_matches_diagnostics() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "diag");
    let has_diag = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Diagnostics)
        )
    });
    assert!(has_diag, "'diag' should match Diagnostics built-in");
}

#[test]
fn browser_query_matches_browser() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "brow");
    let has_browser = items.iter().any(|i| {
        matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Browser)
        )
    });
    assert!(has_browser, "'brow' should match Browser built-in");
}

// ---------- Grouping: builtins before files ----------

#[test]
fn builtins_grouped_before_files_and_folders() {
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

// ---------- Scoring ----------

#[test]
fn exact_mention_scores_higher_than_prefix() {
    let selection_spec = ContextAttachmentKind::Selection.spec();
    let exact = score_builtin(selection_spec, "selection");
    let prefix = score_builtin(selection_spec, "sel");
    assert!(
        exact > prefix,
        "Exact mention ({}) should score higher than prefix ({})",
        exact,
        prefix
    );
}

#[test]
fn empty_query_gives_default_score_to_all_builtins() {
    for spec in context_attachment_specs() {
        let score = score_builtin(spec, "");
        assert_eq!(
            score, 100,
            "Empty query should give default score 100 for {:?}",
            spec.kind
        );
    }
}

#[test]
fn prefix_on_label_scores_higher_than_substring() {
    // "Current Context" — "cur" is a prefix, "ren" is a substring
    let current_spec = ContextAttachmentKind::Current.spec();
    let prefix_score = score_builtin(current_spec, "cur");
    let substring_score = score_builtin(current_spec, "ren");
    assert!(
        prefix_score > substring_score,
        "Label prefix ({}) should score higher than substring ({})",
        prefix_score,
        substring_score
    );
}

// ---------- Picker item → AiContextPart contract ----------

#[test]
fn builtin_selection_item_produces_correct_context_part() {
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

    match &selection_item.kind {
        ContextPickerItemKind::BuiltIn(kind) => {
            let part = kind.part();
            assert_eq!(part.label(), "Selection");
            assert!(
                part.source().contains("selectedText=1"),
                "Selection URI should contain selectedText=1, got: {}",
                part.source()
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn all_builtin_items_produce_resource_uri_parts() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    for item in &items {
        if let ContextPickerItemKind::BuiltIn(kind) = &item.kind {
            let part = kind.part();
            match part {
                AiContextPart::ResourceUri { uri, label } => {
                    assert!(
                        uri.starts_with("kit://"),
                        "Built-in part URI should start with kit://, got: {}",
                        uri
                    );
                    assert!(!label.is_empty(), "Built-in part label should not be empty");
                }
                _ => panic!(
                    "Built-in {:?} should produce ResourceUri, got FilePath",
                    kind
                ),
            }
        }
    }
}

// ---------- ContextPickerState navigation ----------

#[test]
fn picker_state_navigation_wraps_around() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let count = items.len();
    assert!(count >= 2, "Need at least 2 items for navigation test");

    let mut state = ContextPickerState::new(ContextPickerTrigger::Mention, String::new(), items);
    assert_eq!(state.selected_index, 0);

    // Move to last
    state.selected_index = count - 1;

    // Wrap around
    state.selected_index = (state.selected_index + 1) % state.items.len();
    assert_eq!(state.selected_index, 0, "Should wrap to 0");
}

// ---------- Spec completeness ----------

#[test]
fn every_context_attachment_kind_has_a_spec() {
    // Exhaustive check that each kind can produce a spec and a part
    let kinds = [
        ContextAttachmentKind::Current,
        ContextAttachmentKind::Full,
        ContextAttachmentKind::Selection,
        ContextAttachmentKind::Browser,
        ContextAttachmentKind::Window,
        ContextAttachmentKind::Diagnostics,
        ContextAttachmentKind::Screenshot,
        ContextAttachmentKind::Clipboard,
        ContextAttachmentKind::FrontmostApp,
        ContextAttachmentKind::MenuBar,
        ContextAttachmentKind::RecentScripts,
        ContextAttachmentKind::GitStatus,
        ContextAttachmentKind::GitDiff,
        ContextAttachmentKind::Processes,
        ContextAttachmentKind::System,
        ContextAttachmentKind::Dictation,
        ContextAttachmentKind::Calendar,
        ContextAttachmentKind::Notifications,
    ];

    for kind in &kinds {
        let spec = kind.spec();
        assert!(
            !spec.label.is_empty(),
            "{:?} spec should have a label",
            kind
        );
        assert!(!spec.uri.is_empty(), "{:?} spec should have a URI", kind);
        let part = kind.part();
        assert!(
            !part.label().is_empty(),
            "{:?} part should have a label",
            kind
        );
    }
}

// ---------- New catalog entries ----------

#[test]
fn catalog_has_at_least_15_entries() {
    let specs = context_attachment_specs();
    assert!(
        specs.len() >= 15,
        "Catalog should have at least 15 entries, got {}",
        specs.len()
    );
}

#[test]
fn new_entries_are_queryable() {
    let _guard = lock_provider_slots();
    seed_provider_slots();
    // @dictation
    let items = build_picker_items(ContextPickerTrigger::Mention, "dictation");
    assert!(
        items.iter().any(|i| matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Dictation)
        )),
        "'dictation' should match Dictation built-in"
    );

    // @calendar
    let items = build_picker_items(ContextPickerTrigger::Mention, "calendar");
    assert!(
        items.iter().any(|i| matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Calendar)
        )),
        "'calendar' should match Calendar built-in"
    );

    // @notifications
    let items = build_picker_items(ContextPickerTrigger::Mention, "notif");
    assert!(
        items.iter().any(|i| matches!(
            i.kind,
            ContextPickerItemKind::BuiltIn(ContextAttachmentKind::Notifications)
        )),
        "'notif' should match Notifications built-in"
    );
}

// ---------- Slash-mode ranking and highlight improvements ----------

#[test]
fn slash_mode_stays_command_only() {
    let items = build_slash_picker_items("sel", ["compact", "clear", "help"]);
    assert!(
        items
            .iter()
            .all(|item| matches!(item.kind, ContextPickerItemKind::SlashCommand(_))),
        "Slash mode should only return slash commands"
    );
}

#[test]
fn slash_mode_ranking_is_deterministic() {
    let a = build_slash_picker_items("com", ["compact", "clear", "help"]);
    let b = build_slash_picker_items("com", ["compact", "clear", "help"]);
    assert_eq!(a.len(), b.len());
    for (ia, ib) in a.iter().zip(b.iter()) {
        assert_eq!(ia.id, ib.id);
        assert_eq!(ia.score, ib.score);
        assert_eq!(ia.label_highlight_indices, ib.label_highlight_indices);
        assert_eq!(ia.meta_highlight_indices, ib.meta_highlight_indices);
    }
}

#[test]
fn slash_mode_highlights_align_with_meta_text() {
    let items = build_slash_picker_items("com", ["compact", "clear", "help"]);
    let current = items
        .iter()
        .find(|i| matches!(&i.kind, ContextPickerItemKind::SlashCommand(command) if command == "compact"))
        .expect("/compact should match 'com' in slash mode");

    let meta_bare = current.meta.trim_start_matches('/');
    for &idx in &current.meta_highlight_indices {
        assert!(
            idx < meta_bare.len(),
            "meta highlight index {} out of range for '{}' (len {})",
            idx,
            meta_bare,
            meta_bare.len()
        );
    }
    assert!(
        !current.meta_highlight_indices.is_empty(),
        "Slash mode should produce meta highlights for 'com'"
    );
}

#[test]
fn mention_mode_highlights_align_with_meta_text() {
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

    let meta_bare = selection.meta.trim_start_matches('@');
    for &idx in &selection.meta_highlight_indices {
        assert!(
            idx < meta_bare.len(),
            "meta highlight index {} out of range for '{}' (len {})",
            idx,
            meta_bare,
            meta_bare.len()
        );
    }
}

// ---------- Provider-gated picker visibility ----------

#[test]
fn provider_backed_items_absent_when_no_provider_data() {
    let _guard = lock_provider_slots();
    // Clear any leftover provider slot data from other tests
    script_kit_gpui::mcp_resources::clear_provider_json_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let provider_kinds = [
        ContextAttachmentKind::Dictation,
        ContextAttachmentKind::Calendar,
        ContextAttachmentKind::Notifications,
    ];

    for kind in &provider_kinds {
        let found = items.iter().any(|i| matches!(&i.kind, ContextPickerItemKind::BuiltIn(k) if k == kind));
        assert!(
            !found,
            "{kind:?} should NOT appear in picker when no provider data exists"
        );
    }
}

#[test]
fn provider_backed_items_appear_when_provider_data_seeded() {
    let _guard = lock_provider_slots();
    seed_provider_slots();

    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let provider_kinds = [
        ContextAttachmentKind::Dictation,
        ContextAttachmentKind::Calendar,
        ContextAttachmentKind::Notifications,
    ];

    for kind in &provider_kinds {
        let found = items.iter().any(|i| matches!(&i.kind, ContextPickerItemKind::BuiltIn(k) if k == kind));
        assert!(
            found,
            "{kind:?} should appear in picker when provider data is seeded"
        );
    }
}

// ---------- Inline mention provider gating ----------

use script_kit_gpui::ai::{mention_range_at_cursor, parse_inline_context_mentions};

#[test]
fn inline_mention_token_skipped_when_no_provider_data() {
    let _guard = lock_provider_slots();
    script_kit_gpui::mcp_resources::clear_provider_json_slots();

    let mentions = parse_inline_context_mentions("Check @calendar please");
    let has_calendar = mentions.iter().any(|m| m.canonical_token == "@calendar");
    assert!(
        !has_calendar,
        "@calendar inline mention should not resolve when no provider data exists"
    );
}

#[test]
fn inline_mention_token_resolves_when_provider_data_seeded() {
    let _guard = lock_provider_slots();
    seed_provider_slots();

    let mentions = parse_inline_context_mentions("Check @calendar please");
    let has_calendar = mentions.iter().any(|m| m.canonical_token == "@calendar");
    assert!(
        has_calendar,
        "@calendar inline mention should resolve when provider data is seeded"
    );
}

// ---------- Atomic delete edge cases ----------

#[test]
fn mention_range_at_cursor_backspace_trailing_edge() {
    let _guard = lock_provider_slots();
    seed_provider_slots();
    let text = "Fix @browser now";
    // @browser spans chars 4..12
    // Backspace at trailing edge (cursor=12) should match
    let range = mention_range_at_cursor(text, 12);
    assert_eq!(range, Some(4..12), "Backspace at trailing edge should match");
}

#[test]
fn mention_range_at_cursor_backspace_inside() {
    let _guard = lock_provider_slots();
    seed_provider_slots();
    let text = "Fix @browser now";
    // Cursor inside the token (e.g. cursor=8)
    let range = mention_range_at_cursor(text, 8);
    assert_eq!(range, Some(4..12), "Backspace inside token should match");
}

#[test]
fn mention_range_at_cursor_does_not_match_leading_edge() {
    let _guard = lock_provider_slots();
    seed_provider_slots();
    let text = "Fix @browser now";
    // cursor=4 is the leading edge (on the @); mention_range_at_cursor
    // requires cursor > start, so this should NOT match
    let range = mention_range_at_cursor(text, 4);
    assert!(range.is_none(), "mention_range_at_cursor should not match at leading edge (cursor == start)");
}

#[test]
fn mention_range_at_cursor_leading_edge_delete_via_shift() {
    let _guard = lock_provider_slots();
    seed_provider_slots();
    let text = "Fix @browser now";
    // For leading-edge delete, the view helper shifts cursor+1 and checks
    // start match. We verify that cursor+1=5 lands inside the token.
    let range = mention_range_at_cursor(text, 5);
    assert_eq!(range, Some(4..12), "cursor+1 should land inside @browser token");
}
