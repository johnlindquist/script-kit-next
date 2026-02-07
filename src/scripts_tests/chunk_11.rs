// ============================================
// GROUPED RESULTS (FRECENCY) TESTS
// ============================================

#[test]
fn test_get_grouped_results_search_mode_flat_list() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "open".to_string(),
            path: PathBuf::from("/open.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "save".to_string(),
            path: PathBuf::from("/save.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<AppInfo> = vec![];
    let frecency_store = FrecencyStore::new();

    // Search mode: non-empty filter should return flat list
    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "open",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Should be a flat list with fallback section at the end
    // Non-fallback items should be Item entries, followed by a SectionHeader and fallback Items
    assert!(!grouped.is_empty());

    // Find first section header (should be the "Use with..." section)
    let first_header_idx = grouped
        .iter()
        .position(|item| matches!(item, GroupedListItem::SectionHeader(..)));

    // Items before the header should all be Item entries
    if let Some(idx) = first_header_idx {
        for item in grouped.iter().take(idx) {
            assert!(matches!(item, GroupedListItem::Item(_)));
        }
    }

    // At least one match for "open"
    assert!(!results.is_empty());
}

#[test]
fn test_get_grouped_results_empty_filter_grouped_view() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "alpha".to_string(),
            path: PathBuf::from("/alpha.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "beta".to_string(),
            path: PathBuf::from("/beta.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<AppInfo> = vec![];
    let frecency_store = FrecencyStore::new();

    // Empty filter should return grouped view
    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Results should contain all items
    assert_eq!(results.len(), 2);

    // Grouped should have SCRIPTS section (no SUGGESTED since frecency is empty)
    assert!(!grouped.is_empty());

    // First item should be MAIN section header (scripts without kit_name default to "main" kit)
    assert!(matches!(&grouped[0], GroupedListItem::SectionHeader(s, _) if s.starts_with("MAIN")));
}

#[test]
fn test_get_grouped_results_with_frecency() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "alpha".to_string(),
            path: PathBuf::from("/alpha.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "beta".to_string(),
            path: PathBuf::from("/beta.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "gamma".to_string(),
            path: PathBuf::from("/gamma.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<AppInfo> = vec![];

    // Create frecency store and record usage for one script
    let mut frecency_store = FrecencyStore::new();
    frecency_store.record_use("/beta.ts");

    // Empty filter should return grouped view with SUGGESTED section
    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Results should contain all items
    assert_eq!(results.len(), 3);

    // Grouped should have both SUGGESTED and SCRIPTS sections
    let section_headers: Vec<&str> = grouped
        .iter()
        .filter_map(|item| match item {
            GroupedListItem::SectionHeader(s, _) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        section_headers.iter().any(|s| s.starts_with("SUGGESTED")),
        "Expected a SUGGESTED section header"
    );
    // Scripts without kit_name default to "main" kit, so we get "MAIN" section instead of "SCRIPTS"
    assert!(
        section_headers.iter().any(|s| s.starts_with("MAIN")),
        "Expected a MAIN section header"
    );
}

#[test]
fn test_get_grouped_results_frecency_script_appears_before_builtins() {
    // This test verifies the fix for: Clipboard History appearing first
    // regardless of frecency scores.
    //
    // Expected behavior: When a script has frecency > 0, it should appear
    // in the SUGGESTED section BEFORE builtins in MAIN.
    //
    // Bug scenario: User frequently uses "test-script", but Clipboard History
    // still appears as the first choice when opening Script Kit.

    let scripts = wrap_scripts(vec![
        Script {
            name: "test-script".to_string(),
            path: PathBuf::from("/test-script.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("A frequently used script".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "another-script".to_string(),
            path: PathBuf::from("/another-script.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins = create_test_builtins(); // Includes Clipboard History and App Launcher
    let apps: Vec<AppInfo> = vec![];

    // Record usage for test-script to give it frecency
    let mut frecency_store = FrecencyStore::new();
    frecency_store.record_use("/test-script.ts");

    // Get grouped results with empty filter (default view)
    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Verify structure:
    // grouped[0] = SectionHeader("SUGGESTED")
    // grouped[1] = Item(idx) where results[idx] is the frecency script
    // Then type-based sections: SCRIPTS, COMMANDS, etc.

    // First should be SUGGESTED header
    assert!(
        matches!(&grouped[0], GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED")),
        "First item should be SUGGESTED section header, got {:?}",
        grouped[0]
    );

    // Second should be the frecency script (test-script)
    assert!(
        matches!(&grouped[1], GroupedListItem::Item(idx) if {
            let result = &results[*idx];
            matches!(result, SearchResult::Script(sm) if sm.script.name == "test-script")
        }),
        "Second item should be the frecency script 'test-script', got {:?}",
        grouped.get(1).map(|g| {
            if let GroupedListItem::Item(idx) = g {
                format!("Item({}) = {}", idx, results[*idx].name())
            } else {
                format!("{:?}", g)
            }
        })
    );

    // Collect all section headers and items
    let section_headers: Vec<&str> = grouped
        .iter()
        .filter_map(|item| match item {
            GroupedListItem::SectionHeader(s, _) => Some(s.as_str()),
            _ => None,
        })
        .collect();

    // Should have SUGGESTED, MAIN (kit-based section), and COMMANDS sections
    // Scripts without kit_name default to "main" kit, so we get "MAIN" section instead of "SCRIPTS"
    assert!(
        section_headers.iter().any(|s| s.starts_with("MAIN")),
        "Should have MAIN section for non-recent script. Headers: {:?}",
        section_headers
    );
    assert!(
        section_headers.iter().any(|s| s.starts_with("COMMANDS")),
        "Should have COMMANDS section for builtins. Headers: {:?}",
        section_headers
    );

    // Find builtins in COMMANDS section
    let commands_items: Vec<&str> = grouped
        .iter()
        .filter_map(|item| {
            if let GroupedListItem::Item(idx) = item {
                let result = &results[*idx];
                if matches!(result, SearchResult::BuiltIn(_)) {
                    Some(result.name())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Builtins should be in COMMANDS, not SUGGESTED
    assert!(
        commands_items.contains(&"Clipboard History"),
        "Clipboard History should be in COMMANDS section, not SUGGESTED. COMMANDS items: {:?}",
        commands_items
    );
    assert!(
        commands_items.contains(&"App Launcher"),
        "App Launcher should be in COMMANDS section. COMMANDS items: {:?}",
        commands_items
    );
}

#[test]
fn test_get_grouped_results_builtin_with_frecency_vs_script_frecency() {
    // This test captures a more nuanced bug scenario:
    // When BOTH a builtin (Clipboard History) AND a script have frecency,
    // the script with higher frecency should appear first in SUGGESTED.
    //
    // Bug: Clipboard History appears first even when user scripts have
    // higher/more recent frecency scores.

    let scripts = wrap_scripts(vec![Script {
        name: "my-frequent-script".to_string(),
        path: PathBuf::from("/my-frequent-script.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("User's frequently used script".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins = create_test_builtins(); // Clipboard History, App Launcher
    let apps: Vec<AppInfo> = vec![];

    let mut frecency_store = FrecencyStore::new();

    // Record builtin usage once (older)
    frecency_store.record_use("builtin:Clipboard History");

    // Record script usage multiple times (more frequent, should have higher score)
    frecency_store.record_use("/my-frequent-script.ts");
    frecency_store.record_use("/my-frequent-script.ts");
    frecency_store.record_use("/my-frequent-script.ts");

    let (grouped, results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        "",
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Both should be in SUGGESTED, but script should come FIRST (higher frecency)
    assert!(
        matches!(&grouped[0], GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED")),
        "First item should be SUGGESTED header"
    );

    // The first ITEM in SUGGESTED should be the user script (higher frecency)
    assert!(
        matches!(&grouped[1], GroupedListItem::Item(idx) if {
            let result = &results[*idx];
            matches!(result, SearchResult::Script(sm) if sm.script.name == "my-frequent-script")
        }),
        "First item in SUGGESTED should be 'my-frequent-script' (highest frecency), got: {}",
        if let GroupedListItem::Item(idx) = &grouped[1] {
            results[*idx].name().to_string()
        } else {
            format!("{:?}", grouped[1])
        }
    );

    // Clipboard History should be second in SUGGESTED (lower frecency)
    assert!(
        matches!(&grouped[2], GroupedListItem::Item(idx) if {
            results[*idx].name() == "Clipboard History"
        }),
        "Second item in SUGGESTED should be 'Clipboard History' (lower frecency), got: {}",
        if let GroupedListItem::Item(idx) = &grouped[2] {
            results[*idx].name().to_string()
        } else {
            format!("{:?}", grouped[2])
        }
    );
}

