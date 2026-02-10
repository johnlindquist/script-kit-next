// ============================================
// BUILT-IN SEARCH TESTS
// ============================================

fn create_test_builtins() -> Vec<BuiltInEntry> {
    use crate::builtins::{BuiltInFeature, BuiltInGroup};
    vec![
        BuiltInEntry {
            id: "builtin-clipboard-history".to_string(),
            name: "Clipboard History".to_string(),
            description: "View and manage your clipboard history".to_string(),
            keywords: vec![
                "clipboard".to_string(),
                "history".to_string(),
                "paste".to_string(),
                "copy".to_string(),
            ],
            feature: BuiltInFeature::ClipboardHistory,
            icon: Some("📋".to_string()),
            group: BuiltInGroup::Core,
        },
        BuiltInEntry {
            id: "builtin-app-launcher".to_string(),
            name: "App Launcher".to_string(),
            description: "Search and launch installed applications".to_string(),
            keywords: vec![
                "app".to_string(),
                "launch".to_string(),
                "open".to_string(),
                "application".to_string(),
            ],
            feature: BuiltInFeature::AppLauncher,
            icon: Some("🚀".to_string()),
            group: BuiltInGroup::Core,
        },
    ]
}

#[test]
fn test_fuzzy_search_builtins_by_name() {
    let builtins = create_test_builtins();
    let results = fuzzy_search_builtins(&builtins, "clipboard");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "Clipboard History");
    assert!(results[0].score > 0);
}

#[test]
fn test_fuzzy_search_builtins_by_keyword() {
    let builtins = create_test_builtins();

    // "paste" is a keyword for clipboard history
    let results = fuzzy_search_builtins(&builtins, "paste");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "Clipboard History");

    // "launch" is a keyword for app launcher
    let results = fuzzy_search_builtins(&builtins, "launch");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "App Launcher");
}

#[test]
fn test_fuzzy_search_builtins_partial_keyword() {
    let builtins = create_test_builtins();

    // "clip" should match "clipboard" keyword
    let results = fuzzy_search_builtins(&builtins, "clip");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "Clipboard History");

    // "app" should match "app" keyword in App Launcher
    let results = fuzzy_search_builtins(&builtins, "app");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "App Launcher");
}

#[test]
fn test_fuzzy_search_builtins_by_description() {
    let builtins = create_test_builtins();

    // "manage" is in clipboard history description
    let results = fuzzy_search_builtins(&builtins, "manage");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "Clipboard History");

    // "installed" is in app launcher description
    let results = fuzzy_search_builtins(&builtins, "installed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "App Launcher");
}

#[test]
fn test_fuzzy_search_builtins_empty_query() {
    let builtins = create_test_builtins();
    let results = fuzzy_search_builtins(&builtins, "");

    assert_eq!(results.len(), 2);
    // Both should have score 0
    assert_eq!(results[0].score, 0);
    assert_eq!(results[1].score, 0);
}

#[test]
fn test_fuzzy_search_builtins_no_match() {
    let builtins = create_test_builtins();
    let results = fuzzy_search_builtins(&builtins, "nonexistent");

    assert!(results.is_empty());
}

/// Test that name matches are prioritized over keyword matches
/// This is critical: when searching "scr", "Scratch Pad" (name starts with "Scr")
/// should rank higher than "Lock Screen" (keyword "screen" contains "scr")
#[test]
fn test_fuzzy_search_builtins_name_priority_over_keywords() {
    use crate::builtins::{BuiltInFeature, BuiltInGroup, UtilityCommandType};

    let builtins = vec![
        BuiltInEntry {
            id: "builtin-lock-screen".to_string(),
            name: "Lock Screen".to_string(),
            description: "Lock the screen".to_string(),
            keywords: vec![
                "lock".to_string(),
                "screen".to_string(),
                "security".to_string(),
            ],
            feature: BuiltInFeature::ClipboardHistory, // Feature doesn't matter for this test
            icon: Some("🔒".to_string()),
            group: BuiltInGroup::Core,
        },
        BuiltInEntry {
            id: "builtin-scratch-pad".to_string(),
            name: "Scratch Pad".to_string(),
            description: "Quick editor for notes".to_string(),
            keywords: vec![
                "scratch".to_string(),
                "pad".to_string(),
                "notes".to_string(),
            ],
            feature: BuiltInFeature::UtilityCommand(UtilityCommandType::ScratchPad),
            icon: Some("📝".to_string()),
            group: BuiltInGroup::Core,
        },
    ];

    // When searching "scr", Scratch Pad should rank first because its name starts with "Scr"
    // Lock Screen has "screen" as a keyword which contains "scr", but name matches should win
    let results = fuzzy_search_builtins(&builtins, "scr");

    assert!(results.len() >= 2, "Both items should match 'scr'");
    assert_eq!(
        results[0].entry.name, "Scratch Pad",
        "Scratch Pad (name starts with 'Scr') should rank higher than Lock Screen (keyword 'screen')"
    );
    assert!(
        results[0].score > results[1].score,
        "Scratch Pad score ({}) should be higher than Lock Screen score ({})",
        results[0].score,
        results[1].score
    );
}

#[test]
fn test_builtin_match_struct() {
    use crate::builtins::{BuiltInFeature, BuiltInGroup};

    let entry = BuiltInEntry {
        id: "test".to_string(),
        name: "Test Entry".to_string(),
        description: "Test description".to_string(),
        keywords: vec!["test".to_string()],
        feature: BuiltInFeature::ClipboardHistory,
        icon: None,
        group: BuiltInGroup::Core,
    };

    let builtin_match = BuiltInMatch {
        entry: entry.clone(),
        score: 100,
    };

    assert_eq!(builtin_match.entry.name, "Test Entry");
    assert_eq!(builtin_match.score, 100);
}

#[test]
fn test_search_result_builtin_variant() {
    use crate::builtins::{BuiltInFeature, BuiltInGroup};

    let entry = BuiltInEntry {
        id: "test".to_string(),
        name: "Test Built-in".to_string(),
        description: "Test built-in description".to_string(),
        keywords: vec!["test".to_string()],
        feature: BuiltInFeature::AppLauncher,
        icon: Some("🚀".to_string()),
        group: BuiltInGroup::Core,
    };

    let result = SearchResult::BuiltIn(BuiltInMatch { entry, score: 75 });

    assert_eq!(result.name(), "Test Built-in");
    assert_eq!(result.description(), Some("Test built-in description"));
    assert_eq!(result.score(), 75);
    assert_eq!(result.type_label(), "Built-in");
}

#[test]
fn test_unified_search_with_builtins() {
    let scripts = wrap_scripts(vec![Script {
        name: "my-clipboard".to_string(),
        path: PathBuf::from("/clipboard.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("My clipboard script".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let scriptlets = wrap_scriptlets(vec![test_scriptlet_with_desc(
        "Clipboard Helper",
        "ts",
        "clipboard()",
        "Helper for clipboard",
    )]);

    let builtins = create_test_builtins();

    let results = fuzzy_search_unified_with_builtins(&scripts, &scriptlets, &builtins, "clipboard");

    // All three should match
    assert_eq!(results.len(), 3);

    // Verify all types are present
    let has_builtin = results
        .iter()
        .any(|r| matches!(r, SearchResult::BuiltIn(_)));
    let has_script = results.iter().any(|r| matches!(r, SearchResult::Script(_)));
    let has_scriptlet = results
        .iter()
        .any(|r| matches!(r, SearchResult::Scriptlet(_)));

    assert!(has_builtin);
    assert!(has_script);
    assert!(has_scriptlet);
}

#[test]
fn test_unified_search_builtins_appear_at_top() {
    let scripts = wrap_scripts(vec![Script {
        name: "history".to_string(),
        path: PathBuf::from("/history.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let builtins = create_test_builtins();

    let results = fuzzy_search_unified_with_builtins(&scripts, &[], &builtins, "history");

    // Both should match (Clipboard History builtin and history script)
    assert!(results.len() >= 2);

    // When scores are equal, built-ins should appear first
    // Check that the first result is a built-in if scores are equal
    if results.len() >= 2 && results[0].score() == results[1].score() {
        match &results[0] {
            SearchResult::BuiltIn(_) => {} // Expected
            _ => panic!("Built-in should appear before script when scores are equal"),
        }
    }
}

#[test]
fn test_unified_search_backward_compatible() {
    // Ensure the original fuzzy_search_unified still works without builtins
    let scripts = wrap_scripts(vec![Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let scriptlets = wrap_scriptlets(vec![test_scriptlet("Test Snippet", "ts", "test()")]);

    let results = fuzzy_search_unified(&scripts, &scriptlets, "test");

    // Should still work without builtins
    assert_eq!(results.len(), 2);
}

#[test]
fn test_builtin_keyword_matching_priority() {
    let builtins = create_test_builtins();

    // "copy" matches keyword in clipboard history
    let results = fuzzy_search_builtins(&builtins, "copy");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.name, "Clipboard History");
    // Keyword match gives 40 points (reduced from 75 to prioritize name matches)
    // Plus nucleo fuzzy score on name/description if they also match
    assert!(results[0].score >= 40);
}

#[test]
fn test_builtin_fuzzy_keyword_matching() {
    let builtins = create_test_builtins();

    // "hist" should fuzzy match "history" keyword
    let results = fuzzy_search_builtins(&builtins, "hist");
    assert!(!results.is_empty());
    assert_eq!(results[0].entry.name, "Clipboard History");
}

