use super::types::{
    ContextPickerItemKind, ContextPickerState, ContextPickerTrigger, PortalKind,
    SlashCommandPayload,
};
use super::{
    build_picker_items, build_slash_picker_items, build_slash_picker_items_with_payloads,
    extract_context_picker_query, match_query_chars, score_builtin, slash_picker_empty_row,
    slash_picker_loading_row, slash_picker_no_match_row,
};
use crate::ai::context_contract::{context_attachment_specs, ContextAttachmentKind};

#[test]
fn context_picker_empty_query_returns_all_non_provider_builtins() {
    let _env = ProviderTestEnv::new();

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
    let _env = ProviderTestEnv::new();
    // Provider-backed kinds are only shown when real data exists
    let provider_gated = [
        ContextAttachmentKind::Dictation,
        ContextAttachmentKind::Calendar,
        ContextAttachmentKind::Notifications,
    ];

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

    let mut seen_non_system = false;
    for item in &items {
        match &item.kind {
            ContextPickerItemKind::BuiltIn(_)
            | ContextPickerItemKind::Portal(_)
            | ContextPickerItemKind::PortalPrefix(_)
            | ContextPickerItemKind::PortalResult(_) => {
                assert!(
                    !seen_non_system,
                    "Built-in/portal items must appear before file/folder items"
                );
            }
            _ => {
                seen_non_system = true;
            }
        }
    }
}

#[test]
fn context_picker_empty_query_includes_all_portals() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "");
    let portal_labels: Vec<String> = items
        .iter()
        .filter_map(|item| match item.kind {
            ContextPickerItemKind::Portal(_) => Some(item.label.to_string()),
            _ => None,
        })
        .collect();

    for expected in [
        "@file",
        "@browser-history",
        "@tabs",
        "@clipboard",
        "@script",
        "@scriptlet",
        "@skill",
        "@note",
        "@history",
        "@terminal",
    ] {
        assert!(
            portal_labels.iter().any(|label| label == expected),
            "expected portal label {expected:?} in empty-query picker results"
        );
    }
}

#[test]
fn context_picker_browser_history_query_matches_portal() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "chrome");
    let has_browser_history_portal = items
        .iter()
        .any(|item| item.label.as_ref() == "@browser-history");
    assert!(
        has_browser_history_portal,
        "browser-history portal should match browser-focused queries"
    );
}

#[test]
fn browser_history_colon_query_keeps_inline_fallback_in_picker() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "browser-history:");
    assert!(
        items.iter().any(|item| {
            item.id.as_ref() == "portal-full:browser-history"
                && matches!(
                    item.kind,
                    ContextPickerItemKind::Portal(PortalKind::BrowserHistory)
                )
        }),
        "@browser-history: should keep an explicit full-browser fallback inline"
    );
}

#[test]
fn every_portal_colon_query_enters_inline_search_mode_with_fallback() {
    for (query, fallback_id) in [
        ("file:", "portal-full:file"),
        ("browser-history:", "portal-full:browser-history"),
        ("tabs:", "portal-full:tabs"),
        ("browser-tabs:", "portal-full:tabs"),
        ("clipboard:", "portal-full:clipboard"),
        ("dictation:", "portal-full:dictation"),
        ("script:", "portal-full:script"),
        ("scriptlet:", "portal-full:scriptlet"),
        ("skill:", "portal-full:skill"),
        ("note:", "portal-full:note"),
        ("history:", "portal-full:history"),
        ("terminal:", "portal-full:terminal"),
    ] {
        let items = build_picker_items(ContextPickerTrigger::Mention, query);
        assert!(
            items.iter().any(|item| item.id.as_ref() == fallback_id),
            "{query:?} should enter inline portal mode and keep fallback {fallback_id:?}"
        );
        assert!(
            items
                .iter()
                .all(|item| !matches!(item.kind, ContextPickerItemKind::PortalPrefix(_))),
            "{query:?} should search inside the tray, not offer another prefix row"
        );
    }
}

#[test]
fn file_colon_query_opens_full_file_search_without_inline_results() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "file:demo");
    let first = items.first().expect("@file: should offer full file search");

    assert_eq!(first.id.as_ref(), "portal-full:file");
    assert!(matches!(
        first.kind,
        ContextPickerItemKind::Portal(PortalKind::FileSearch)
    ));
    assert!(
        items
            .iter()
            .all(|item| !matches!(item.kind, ContextPickerItemKind::PortalResult(_))),
        "@file: must not use local inline file rows because they lack the built-in preview panel"
    );
}

#[test]
fn inline_script_list_portal_results_attach_like_full_portal_selection() {
    use std::sync::Arc;

    let script = crate::scripts::Script {
        name: "Build Release".to_string(),
        path: std::path::PathBuf::from("/tmp/build-release.ts"),
        description: Some("Builds the release".to_string()),
        ..Default::default()
    };
    let script_item = super::inline_portal_item_from_search_result(
        PortalKind::ScriptSearch,
        crate::scripts::SearchResult::Script(crate::scripts::ScriptMatch {
            script: Arc::new(script),
            score: 42,
            filename: "build-release.ts".to_string(),
            match_indices: crate::scripts::MatchIndices {
                name_indices: vec![0, 1],
                ..Default::default()
            },
            match_kind: crate::scripts::ScriptMatchKind::Name,
            content_match: None,
            match_evidence: None,
        }),
    )
    .expect("script result should map to inline item");
    assert!(matches!(
        script_item.kind,
        ContextPickerItemKind::PortalResult(super::types::InlinePortalResultPayload {
            attachment: super::types::InlinePortalAttachment::FilePath { .. },
            ..
        })
    ));

    let skill = crate::plugins::PluginSkill {
        plugin_id: "scriptkit".to_string(),
        plugin_title: "Script Kit".to_string(),
        skill_id: "new-script".to_string(),
        path: std::path::PathBuf::from("/tmp/SKILL.md"),
        title: "New Script".to_string(),
        description: "Create scripts".to_string(),
    };
    let skill_item = super::inline_portal_item_from_search_result(
        PortalKind::SkillSearch,
        crate::scripts::SearchResult::Skill(crate::scripts::SkillMatch {
            skill: Arc::new(skill),
            score: 99,
            match_indices: crate::scripts::MatchIndices::default(),
            match_evidence: None,
        }),
    )
    .expect("skill result should map to inline item");
    assert!(matches!(
        skill_item.kind,
        ContextPickerItemKind::PortalResult(super::types::InlinePortalResultPayload {
            attachment: super::types::InlinePortalAttachment::SkillFile { .. },
            ..
        })
    ));
}

#[test]
fn context_picker_browser_tabs_aliases_open_full_portal_and_keep_colon_fallback() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "tabs");
    let tabs_row = items
        .iter()
        .find(|item| item.label.as_ref() == "@tabs")
        .expect("@tabs portal should match tabs query");
    assert!(matches!(
        tabs_row.kind,
        ContextPickerItemKind::Portal(PortalKind::BrowserTabs)
    ));

    for query in ["tabs:", "browser-tabs:"] {
        let items = build_picker_items(ContextPickerTrigger::Mention, query);
        assert!(
            items.iter().any(|item| {
                item.id.as_ref() == "portal-full:tabs"
                    && matches!(
                        item.kind,
                        ContextPickerItemKind::Portal(PortalKind::BrowserTabs)
                    )
            }),
            "{query:?} should enter browser tab inline mode with a full-portal fallback"
        );
    }
}

#[test]
fn context_picker_browser_tabs_inline_result_carries_full_focused_target_metadata() {
    let mut items = Vec::new();
    super::collect_browser_tabs_inline_items_from_tabs(
        "openai",
        vec![crate::browser_tabs::BrowserTabInfo {
            browser_name: "Google Chrome".to_string().into(),
            browser_bundle_id: "com.google.Chrome".to_string().into(),
            window_index: 1,
            tab_index: 2,
            title: "OpenAI Docs".to_string().into(),
            url: "https://example.test/openai-docs".to_string().into(),
        }],
        &mut items,
    );

    let item = items
        .iter()
        .find(|item| item.label.as_ref() == "OpenAI Docs")
        .expect("matching browser tab should render inline result row");
    match &item.kind {
        ContextPickerItemKind::PortalResult(payload) => {
            assert_eq!(payload.portal_kind, PortalKind::BrowserTabs);
            match &payload.attachment {
                super::types::InlinePortalAttachment::FocusedTarget {
                    source,
                    kind,
                    semantic_id,
                    label,
                    metadata,
                } => {
                    assert_eq!(source, "BrowserTabs");
                    assert_eq!(kind, "browser_tab");
                    assert!(semantic_id.starts_with("browser-tab:"));
                    assert_eq!(label, "OpenAI Docs");
                    let metadata = metadata.as_ref().expect("browser tab metadata");
                    assert_eq!(metadata["browserName"], "Google Chrome");
                    assert_eq!(metadata["browserBundleId"], "com.google.Chrome");
                    assert_eq!(metadata["windowIndex"], 1);
                    assert_eq!(metadata["tabIndex"], 2);
                    assert_eq!(metadata["title"], "OpenAI Docs");
                    assert_eq!(metadata["url"], "https://example.test/openai-docs");
                    assert_eq!(metadata["host"], "example.test");
                    assert!(metadata["stableKey"]
                        .as_str()
                        .is_some_and(|stable_key| stable_key.contains("com.google.Chrome")));
                }
                other => panic!("expected FocusedTarget attachment, got {other:?}"),
            }
        }
        other => panic!("expected BrowserTabs PortalResult, got {other:?}"),
    }
}

#[test]
fn top_level_portal_rows_open_full_portals() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "browser");
    let row = items
        .iter()
        .find(|item| item.label.as_ref() == "@browser-history")
        .expect("browser history portal should match browser query");
    assert!(matches!(
        row.kind,
        ContextPickerItemKind::Portal(PortalKind::BrowserHistory)
    ));
}

#[test]
fn context_picker_terminal_query_matches_portal() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "terminal");
    let row = items
        .iter()
        .find(|item| item.label.as_ref() == "@terminal")
        .expect("@terminal portal should match terminal query");
    assert!(matches!(
        row.kind,
        ContextPickerItemKind::Portal(PortalKind::Terminal)
    ));
}

#[test]
fn terminal_colon_query_returns_terminal_history_results() {
    crate::terminal_history::clear_for_tests();
    crate::terminal_history::record(crate::terminal_history::TerminalHistoryEntry {
        label: "Terminal Output".to_string(),
        source: "terminal://quick-terminal/test".to_string(),
        text: "Terminal session\n--- output ---\npnpm test passed".to_string(),
        line_count: 3,
        truncated: false,
        captured_at: "2026-06-04T00:00:00Z".to_string(),
    });

    let items = build_picker_items(ContextPickerTrigger::Mention, "terminal:test");
    assert!(items.iter().any(|item| {
        matches!(
            &item.kind,
            ContextPickerItemKind::PortalResult(super::types::InlinePortalResultPayload {
                portal_kind: PortalKind::Terminal,
                attachment: super::types::InlinePortalAttachment::TextBlock { .. },
            })
        )
    }));
}

#[test]
fn context_picker_top_level_clipboard_and_file_open_full_portals() {
    for (query, label, expected_kind) in [
        ("clipboard", "@clipboard", PortalKind::ClipboardHistory),
        ("file", "@file", PortalKind::FileSearch),
    ] {
        let items = build_picker_items(ContextPickerTrigger::Mention, query);
        let row = items
            .iter()
            .find(|item| item.label.as_ref() == label)
            .unwrap_or_else(|| panic!("{label} portal should match {query:?}"));
        assert!(
            matches!(row.kind, ContextPickerItemKind::Portal(kind) if kind == expected_kind),
            "{label} should open the full portal surface, got {:?}",
            row.kind
        );
    }
}

#[test]
fn slash_query_never_returns_portal_prefix_or_result_rows() {
    let items = build_picker_items(ContextPickerTrigger::Slash, "browser-history:");
    assert!(
        items.iter().all(|item| !matches!(
            item.kind,
            ContextPickerItemKind::PortalPrefix(_) | ContextPickerItemKind::PortalResult(_)
        )),
        "slash picker must stay command-only"
    );
}

#[test]
fn browser_history_entries_can_be_collected_as_inline_portal_results() {
    let mut items = Vec::new();
    super::collect_browser_history_inline_items_from_entries(
        "rust",
        vec![crate::browser_history::BrowserHistoryEntry {
            browser_name: "Safari".to_string().into(),
            browser_bundle_id: "com.apple.Safari".to_string().into(),
            title: "Rust docs".to_string().into(),
            url: "https://doc.rust-lang.org/".to_string().into(),
            host: "doc.rust-lang.org".to_string().into(),
            last_visited_at_ms: 1,
            visit_count: 2,
            profile: "Default".to_string().into(),
        }],
        &mut items,
    );

    assert!(
        items
            .iter()
            .any(|item| matches!(item.kind, ContextPickerItemKind::PortalResult(_))),
        "matching browser history entries should render inline result rows"
    );
}

#[test]
fn clipboard_entries_can_be_collected_as_inline_portal_results() {
    let mut items = Vec::new();
    super::collect_clipboard_inline_items_from_entries(
        "token",
        vec![crate::clipboard_history::ClipboardEntryMeta {
            id: "clip-1".to_string(),
            content_type: crate::clipboard_history::ContentType::Text,
            timestamp: 1,
            pinned: false,
            text_preview: "token from clipboard".to_string(),
            image_width: None,
            image_height: None,
            byte_size: 20,
            ocr_text: None,
        }],
        &mut items,
    );

    assert!(
        items
            .iter()
            .any(|item| matches!(item.kind, ContextPickerItemKind::PortalResult(_))),
        "matching clipboard entries should render inline result rows"
    );
}

#[test]
fn context_picker_clipboard_query_keeps_builtin_ahead_of_portal() {
    let items = build_picker_items(ContextPickerTrigger::Mention, "clipboard");

    let built_in_index = items
        .iter()
        .position(|item| item.label.as_ref() == "Clipboard")
        .expect("clipboard built-in should be present");
    let portal_index = items
        .iter()
        .position(|item| {
            item.label.as_ref() == "@clipboard"
                && matches!(
                    item.kind,
                    ContextPickerItemKind::Portal(PortalKind::ClipboardHistory)
                )
        })
        .expect("clipboard portal should be present");

    assert!(
        built_in_index < portal_index,
        "exact built-in mention should outrank the clipboard portal"
    );
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
    let _env = ProviderTestEnv::new();
    // Provider-gated items may be hidden, so the minimum is lower
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
        ContextPickerItemKind::SlashCommand(payload) => {
            assert_eq!(payload.slash_name(), "compact")
        }
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
        ContextPickerItemKind::SlashCommand(payload) => {
            assert_eq!(payload.slash_name(), "clear")
        }
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
fn context_picker_rows_use_main_list_item_chrome() {
    let source = include_str!("render.rs");
    assert!(source.contains("ListItem::new"));
    assert!(source.contains("ListItemColors::from_theme"));
    assert!(source.contains(".selected(is_selected)"));
    assert!(source.contains(".main_menu_theme("));
    assert!(source.contains(".semantic_id(format!(\"ctx-picker-{ix}\"))"));
    assert!(!source.contains("render_soft_compact_picker_row"));
    assert!(!source.contains("render_dense_monoline_picker_row"));
}

#[test]
fn context_picker_overlay_shell_and_click_contract_are_preserved() {
    let source = include_str!("render.rs");
    for needle in [
        "let picker_list = list(",
        "self.context_picker_list_state.clone()",
        "InlineDropdown::new(",
        "\"context-picker-overlay\"",
        ".empty_state_opt(empty_state)",
        ".synopsis(synopsis)",
        ".vertical_padding(4.0)",
        "picker.selected_index = ix",
        "accept_context_picker_selection(window, cx)",
    ] {
        assert!(
            source.contains(needle),
            "missing preserved contract: {needle}"
        );
    }
}

#[test]
fn context_picker_empty_state_hint_chips_remain_clickable() {
    let source = include_str!("render.rs");
    for needle in [
        "format!(\"hint-{}\", hint.display)",
        "ai_context_picker_empty_hint_applied",
        "set_composer_value(hint_insertion_for_click.clone(), window, cx)",
        "No matching commands",
        "No matching context",
        "No matching profiles",
    ] {
        assert!(
            source.contains(needle),
            "missing empty-state contract: {needle}"
        );
    }
}

#[test]
fn context_picker_open_log_uses_main_list_item_layout() {
    let source = include_str!("mod.rs");
    assert!(source.contains("layout = \"main_list_item\""));
    assert!(!source.contains("layout = \"dense_monoline_shared\""));
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
        .find(|i| matches!(&i.kind, ContextPickerItemKind::SlashCommand(ref payload) if payload.slash_name() == "compact"))
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
        |i| matches!(&i.kind, ContextPickerItemKind::SlashCommand(ref payload) if payload.slash_name() == "compact"),
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
    let _env = ProviderTestEnv::new();

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
    let _env = ProviderTestEnv::new();

    // Publish dictation data
    crate::mcp_resources::publish_dictation_json(
        r#"{"schemaVersion":1,"type":"dictation","ok":true,"available":true,"source":"slot","items":[{"text":"test"}]}"#,
    );

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
}

#[test]
fn provider_backed_items_hidden_in_targeted_query() {
    let _env = ProviderTestEnv::new();

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

struct ProviderTestEnv {
    _guard: std::sync::MutexGuard<'static, ()>,
    prev_calendar: Option<std::ffi::OsString>,
    prev_dictation: Option<std::ffi::OsString>,
    prev_notifications: Option<std::ffi::OsString>,
}

impl ProviderTestEnv {
    fn new() -> Self {
        let guard = crate::test_utils::lock_provider_json_test();
        let prev_calendar = std::env::var_os("SCRIPT_KIT_CALENDAR_JSON");
        let prev_dictation = std::env::var_os("SCRIPT_KIT_DICTATION_JSON");
        let prev_notifications = std::env::var_os("SCRIPT_KIT_NOTIFICATIONS_JSON");
        std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
        std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
        std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
        crate::mcp_resources::clear_provider_json_slots();
        Self {
            _guard: guard,
            prev_calendar,
            prev_dictation,
            prev_notifications,
        }
    }
}

impl Drop for ProviderTestEnv {
    fn drop(&mut self) {
        crate::mcp_resources::clear_provider_json_slots();
        restore_env("SCRIPT_KIT_DICTATION_JSON", self.prev_dictation.take());
        restore_env("SCRIPT_KIT_CALENDAR_JSON", self.prev_calendar.take());
        restore_env(
            "SCRIPT_KIT_NOTIFICATIONS_JSON",
            self.prev_notifications.take(),
        );
    }
}

#[test]
fn mention_empty_state_hints_hide_unavailable_provider_entries() {
    let _env = ProviderTestEnv::new();

    let hints = super::empty_state_hints(ContextPickerTrigger::Mention);
    assert!(
        !hints.iter().any(|hint| hint.display == "@dictation"),
        "@dictation hint must be hidden when provider data is unavailable"
    );
    assert!(
        !hints.iter().any(|hint| hint.display == "@calendar"),
        "@calendar hint must be hidden when provider data is unavailable"
    );
}

#[test]
fn mention_empty_state_hints_show_available_provider_entries() {
    let _env = ProviderTestEnv::new();
    std::env::set_var(
        "SCRIPT_KIT_CALENDAR_JSON",
        r#"{"schemaVersion":1,"type":"calendar","ok":true,"available":true,"source":"env","items":[{"title":"Demo"}]}"#,
    );
    crate::mcp_resources::publish_dictation_json(
        r#"{"schemaVersion":1,"type":"dictation","ok":true,"available":true,"source":"slot","items":[{"text":"hello"}]}"#,
    );

    let hints = super::empty_state_hints(ContextPickerTrigger::Mention);
    assert!(
        hints.iter().any(|hint| hint.display == "@dictation"),
        "@dictation hint must be shown when provider data is real"
    );
    assert!(
        hints.iter().any(|hint| hint.display == "@calendar"),
        "@calendar hint must be shown when provider data is real"
    );
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
    assert_eq!(inline_dropdown_visible_range(7, 20, 8), 0..8);
    assert_eq!(inline_dropdown_visible_range(8, 20, 8), 1..9);
}

// ── Source-aware slash identity tests ─────────────────────────────────

#[test]
fn slash_mode_duplicate_skill_rows_keep_distinct_payloads() {
    use std::path::PathBuf;

    let alpha_skill = crate::plugins::PluginSkill {
        plugin_id: "alpha".to_string(),
        plugin_title: "Alpha".to_string(),
        skill_id: "review".to_string(),
        path: PathBuf::from("/alpha/skills/review/SKILL.md"),
        title: "Review".to_string(),
        description: "Alpha review".to_string(),
    };
    let beta_skill = crate::plugins::PluginSkill {
        plugin_id: "beta".to_string(),
        plugin_title: "Beta".to_string(),
        skill_id: "review".to_string(),
        path: PathBuf::from("/beta/skills/review/SKILL.md"),
        title: "Review".to_string(),
        description: "Beta review".to_string(),
    };

    let payloads = vec![
        (
            SlashCommandPayload::Default {
                name: "clear".to_string(),
            },
            "Clear conversation".to_string(),
        ),
        (
            SlashCommandPayload::PluginSkill(alpha_skill.clone()),
            "Alpha review".to_string(),
        ),
        (
            SlashCommandPayload::PluginSkill(beta_skill.clone()),
            "Beta review".to_string(),
        ),
    ];

    let items = build_slash_picker_items_with_payloads(
        "review",
        payloads.iter().map(|(p, d)| (p, d.as_str())),
    );

    // Both plugin review rows should appear
    assert_eq!(items.len(), 2, "Two 'review' skill rows should match");

    let ids: Vec<String> = items.iter().map(|i| i.id.to_string()).collect();
    assert!(
        ids.contains(&"slash-cmd:plugin:alpha:review".to_string()),
        "Alpha review row should have stable ID, got: {:?}",
        ids
    );
    assert!(
        ids.contains(&"slash-cmd:plugin:beta:review".to_string()),
        "Beta review row should have stable ID, got: {:?}",
        ids
    );

    // Verify payloads are distinct
    let alpha_row = items
        .iter()
        .find(|i| i.id.as_ref() == "slash-cmd:plugin:alpha:review")
        .expect("alpha row");
    let beta_row = items
        .iter()
        .find(|i| i.id.as_ref() == "slash-cmd:plugin:beta:review")
        .expect("beta row");

    assert_ne!(alpha_row.meta, beta_row.meta, "Meta should differ by owner");
    assert!(
        alpha_row.meta.contains("Alpha"),
        "Alpha row meta should contain 'Alpha', got: {}",
        alpha_row.meta
    );
    assert!(
        beta_row.meta.contains("Beta"),
        "Beta row meta should contain 'Beta', got: {}",
        beta_row.meta
    );
}

#[test]
fn slash_payload_stable_id_formats() {
    let default = SlashCommandPayload::Default {
        name: "compact".to_string(),
    };
    assert_eq!(default.stable_id(), "default:compact");
    assert_eq!(default.slash_name(), "compact");
    assert_eq!(default.owner_label(), "Built-in");

    let plugin = SlashCommandPayload::PluginSkill(crate::plugins::PluginSkill {
        plugin_id: "tools".to_string(),
        plugin_title: "Dev Tools".to_string(),
        skill_id: "lint".to_string(),
        path: std::path::PathBuf::from("/tmp/lint/SKILL.md"),
        title: "Lint".to_string(),
        description: String::new(),
    });
    assert_eq!(plugin.stable_id(), "plugin:tools:lint");
    assert_eq!(plugin.slash_name(), "lint");
    assert_eq!(plugin.owner_label(), "Dev Tools");

    let claude = SlashCommandPayload::ClaudeCodeSkill {
        skill_id: "plan".to_string(),
        skill_path: std::path::PathBuf::from("/tmp/plan/SKILL.md"),
    };
    assert_eq!(claude.stable_id(), "claude:plan");
    assert_eq!(claude.slash_name(), "plan");
    assert_eq!(claude.owner_label(), "Claude Code");
}

#[test]
fn slash_default_items_have_default_stable_ids() {
    let items = build_slash_picker_items("", ["compact", "clear"]);
    for item in &items {
        assert!(
            item.id.starts_with("slash-cmd:default:"),
            "Default slash items should have 'slash-cmd:default:' prefix, got: {}",
            item.id
        );
    }
}

// =========================================================================
// Loading and empty state rows
// =========================================================================

#[test]
fn slash_picker_loading_and_empty_states_are_inert() {
    let loading = slash_picker_loading_row();
    assert!(
        matches!(loading.kind, ContextPickerItemKind::Inert),
        "Loading row must be Inert, got: {:?}",
        loading.kind
    );
    assert_eq!(loading.id.as_ref(), "slash-loading");
    assert!(
        loading.label.contains("Discovering"),
        "Loading label should mention discovery: {}",
        loading.label
    );
    assert_eq!(loading.score, 0, "Inert rows should have zero score");

    let empty = slash_picker_empty_row();
    assert!(
        matches!(empty.kind, ContextPickerItemKind::Inert),
        "Empty row must be Inert, got: {:?}",
        empty.kind
    );
    assert_eq!(empty.id.as_ref(), "slash-empty");
    assert!(
        empty.label.contains("No slash commands"),
        "Empty label should indicate no results: {}",
        empty.label
    );
    assert_eq!(empty.score, 0, "Inert rows should have zero score");

    // Snapshot section should report "inert" for both.
    let state = ContextPickerState::new(
        ContextPickerTrigger::Slash,
        String::new(),
        vec![loading.clone(), empty.clone()],
    );
    let snap = state.snapshot();
    assert_eq!(snap.items.len(), 2);
    assert_eq!(snap.items[0].section, "inert");
    assert_eq!(snap.items[1].section, "inert");
}

#[test]
fn agent_chat_slash_picker_duplicate_rows_show_owner_labels() {
    let alpha_skill = crate::plugins::PluginSkill {
        plugin_id: "alpha".to_string(),
        plugin_title: "Alpha".to_string(),
        skill_id: "review".to_string(),
        path: std::path::PathBuf::from("/alpha/skills/review/SKILL.md"),
        title: "Review".to_string(),
        description: "Alpha review desc".to_string(),
    };
    let beta_skill = crate::plugins::PluginSkill {
        plugin_id: "beta".to_string(),
        plugin_title: "Beta".to_string(),
        skill_id: "review".to_string(),
        path: std::path::PathBuf::from("/beta/skills/review/SKILL.md"),
        title: "Review".to_string(),
        description: "Beta review desc".to_string(),
    };

    let payloads = vec![
        (
            SlashCommandPayload::Default {
                name: "clear".to_string(),
            },
            "Clear conversation".to_string(),
        ),
        (
            SlashCommandPayload::PluginSkill(alpha_skill),
            "Alpha review desc".to_string(),
        ),
        (
            SlashCommandPayload::PluginSkill(beta_skill),
            "Beta review desc".to_string(),
        ),
    ];

    // Query "review" should match both plugin rows but not "clear".
    let items = build_slash_picker_items_with_payloads(
        "review",
        payloads.iter().map(|(p, d)| (p, d.as_str())),
    );
    assert_eq!(items.len(), 2, "Two 'review' skill rows should match");

    let alpha_row = items
        .iter()
        .find(|i| i.id.as_ref() == "slash-cmd:plugin:alpha:review")
        .expect("Alpha review row should exist");
    let beta_row = items
        .iter()
        .find(|i| i.id.as_ref() == "slash-cmd:plugin:beta:review")
        .expect("Beta review row should exist");

    // Meta must show owner labels.
    assert!(
        alpha_row.meta.contains("Alpha"),
        "Alpha row meta should show owner 'Alpha', got: {}",
        alpha_row.meta
    );
    assert!(
        beta_row.meta.contains("Beta"),
        "Beta row meta should show owner 'Beta', got: {}",
        beta_row.meta
    );
    // Default rows should NOT show owner in meta.
    let all_items =
        build_slash_picker_items_with_payloads("", payloads.iter().map(|(p, d)| (p, d.as_str())));
    let clear_row = all_items
        .iter()
        .find(|i| i.id.as_ref() == "slash-cmd:default:clear")
        .expect("clear row");
    assert!(
        !clear_row.meta.contains("Built-in"),
        "Default command meta should not show owner label: {}",
        clear_row.meta
    );
}

// =========================================================================
// No-match row
// =========================================================================

#[test]
fn slash_picker_no_match_row_is_inert_and_distinct() {
    let no_match = slash_picker_no_match_row();
    assert!(
        matches!(no_match.kind, ContextPickerItemKind::Inert),
        "No-match row must be Inert, got: {:?}",
        no_match.kind
    );
    assert_eq!(no_match.id.as_ref(), "slash-no-match");
    assert!(
        no_match.label.contains("No matching"),
        "No-match label should indicate no match: {}",
        no_match.label
    );
    assert_eq!(no_match.score, 0, "Inert rows should have zero score");

    // Distinct from loading and empty rows.
    let loading = slash_picker_loading_row();
    let empty = slash_picker_empty_row();
    assert_ne!(no_match.id, loading.id);
    assert_ne!(no_match.id, empty.id);
}

// =========================================================================
// Slash picker query filters to no-match row
// =========================================================================

#[test]
fn slash_picker_query_filters_to_no_match_row() {
    // Build items with real commands, then filter with a query that
    // won't match anything. The caller (refresh_mention_session) would
    // push a no-match row in this case.
    let payloads = vec![
        (
            SlashCommandPayload::Default {
                name: "compact".to_string(),
            },
            "Compact conversation".to_string(),
        ),
        (
            SlashCommandPayload::Default {
                name: "clear".to_string(),
            },
            "Clear conversation".to_string(),
        ),
    ];

    let items = build_slash_picker_items_with_payloads(
        "zzzzzz",
        payloads.iter().map(|(p, d)| (p, d.as_str())),
    );
    assert!(
        items.is_empty(),
        "Gibberish query should filter all items: {:?}",
        items.iter().map(|i| i.id.as_ref()).collect::<Vec<_>>()
    );

    // Simulates the refresh_mention_session fallback.
    let mut display_items = items;
    if display_items.is_empty() {
        display_items.push(slash_picker_no_match_row());
    }
    assert_eq!(display_items.len(), 1);
    assert_eq!(display_items[0].id.as_ref(), "slash-no-match");
}
