// ============================================
// COMPREHENSIVE RANKING & RELEVANCE TESTS
// ============================================

#[test]
fn test_exact_substring_at_start_highest_score() {
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
            name: "reopen".to_string(),
            path: PathBuf::from("/reopen.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "open");
    // "open" starts with "open" (score 100 + fuzzy 50 = 150)
    // "reopen" has "open" but not at start (score 75 + fuzzy 50 = 125)
    assert_eq!(results[0].script.name, "open");
    assert!(results[0].score > results[1].score);
}

#[test]
fn test_description_match_lower_priority_than_name() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "test".to_string(),
            path: PathBuf::from("/test.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "other".to_string(),
            path: PathBuf::from("/other.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("test description".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "test");
    // Name match should rank higher than description match
    assert_eq!(results[0].script.name, "test");
}

#[test]
fn test_path_match_lowest_priority() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "test".to_string(),
            path: PathBuf::from("/test.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "other".to_string(),
            path: PathBuf::from("/test/other.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "test");
    // Name match should rank higher than path match
    assert_eq!(results[0].script.name, "test");
}

#[test]
fn test_scriptlet_code_match_lower_than_description() {
    let mut snippet = test_scriptlet("Snippet", "ts", "paste()");
    snippet.description = Some("copy text".to_string());

    let other = test_scriptlet("Other", "ts", "copy()");

    let scriptlets = wrap_scriptlets(vec![snippet, other]);

    let results = fuzzy_search_scriptlets(&scriptlets, "copy");
    // Description match should score higher than code match
    assert_eq!(results[0].scriptlet.name, "Snippet");
}

#[test]
fn test_tool_type_bonus_in_scoring() {
    let scriptlets = wrap_scriptlets(vec![
        test_scriptlet("Script1", "bash", "code"),
        test_scriptlet("Script2", "ts", "code"),
    ]);

    let results = fuzzy_search_scriptlets(&scriptlets, "bash");
    // "bash" matches tool type in Script1
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].scriptlet.name, "Script1");
}

#[test]
fn test_longer_exact_match_ties_with_fuzzy() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "open".to_string(),
            path: PathBuf::from("/open.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("Open a file".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "openfile".to_string(),
            path: PathBuf::from("/openfile.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "open");
    // Both have name matches at start (100 points) and fuzzy match (50 points)
    // When tied, should sort by name alphabetically
    assert_eq!(results[0].script.name, "open");
    assert_eq!(results[1].script.name, "openfile");
}

#[test]
fn test_case_insensitive_matching() {
    let scripts = wrap_scripts(vec![Script {
        name: "OpenFile".to_string(),
        path: PathBuf::from("/openfile.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let results = fuzzy_search_scripts(&scripts, "OPEN");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].script.name, "OpenFile");
}

#[test]
fn test_ranking_preserves_relative_order_on_score_tie() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "aaa".to_string(),
            path: PathBuf::from("/aaa.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("test".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "bbb".to_string(),
            path: PathBuf::from("/bbb.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("test".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "test");
    // Same score, should sort by name
    assert_eq!(results[0].script.name, "aaa");
    assert_eq!(results[1].script.name, "bbb");
}

#[test]
fn test_scriptlet_name_match_bonus_points() {
    let scriptlets = wrap_scriptlets(vec![
        test_scriptlet("copy", "ts", "copy()"),
        test_scriptlet("paste", "ts", "copy()"),
    ]);

    let results = fuzzy_search_scriptlets(&scriptlets, "copy");
    // "copy" name has higher bonus than "paste" code match
    assert_eq!(results[0].scriptlet.name, "copy");
    assert!(results[0].score > 0);
}

#[test]
fn test_unified_search_ties_scripts_first() {
    let scripts = wrap_scripts(vec![Script {
        name: "test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("Test script".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let scriptlets = wrap_scriptlets(vec![test_scriptlet_with_desc(
        "test",
        "ts",
        "test()",
        "Test snippet",
    )]);

    let results = fuzzy_search_unified(&scripts, &scriptlets, "test");
    // Same score, scripts should come before scriptlets
    assert_eq!(results.len(), 2);
    match &results[0] {
        SearchResult::Script(_) => {}
        SearchResult::Scriptlet(_) => panic!("Expected Script first"),
        SearchResult::BuiltIn(_) => panic!("Expected Script first"),
        SearchResult::App(_) => panic!("Expected Script first"),
        SearchResult::Window(_) => panic!("Expected Script first"),
        SearchResult::Agent(_) => panic!("Expected Script first"),
        SearchResult::Fallback(_) => panic!("Expected Script first"),
    }
}

#[test]
fn test_partial_match_scores_appropriately() {
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

    let results = fuzzy_search_scripts(&scripts, "es");
    // "es" is fuzzy match in "test" but not a substring match
    assert_eq!(results.len(), 1);
    assert!(results[0].score > 0);
}

#[test]
fn test_multiple_word_query() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "open file".to_string(),
            path: PathBuf::from("/openfile.ts"),
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

    // Query with space - will be treated as literal string
    let results = fuzzy_search_scripts(&scripts, "open file");
    assert!(!results.is_empty());
}

#[test]
fn test_all_search_types_contribute_to_score() {
    // Test that all scoring categories work
    let scripts = wrap_scripts(vec![Script {
        name: "database".to_string(),
        path: PathBuf::from("/database.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("database connection".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let results = fuzzy_search_scripts(&scripts, "database");
    // Should match on name (100 + 50 = 150) + description (25) = 175
    assert!(results[0].score > 100);
}

#[test]
fn test_search_quality_metrics() {
    // Ensure search returns meaningful results
    let scripts = wrap_scripts(vec![
        Script {
            name: "zzzFile".to_string(),
            path: PathBuf::from("/home/user/.scriptkit/scripts/zzzFile.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("Opens a file dialog".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "someScript".to_string(),
            path: PathBuf::from("/home/user/.scriptkit/scripts/someScript.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("Does something".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "saveData".to_string(),
            path: PathBuf::from("/home/user/.scriptkit/scripts/saveData.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("Saves data to file".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "file");
    // Two should match (zzzFile name and saveData description)
    assert_eq!(results.len(), 2);
    // Name match (zzzFile) should rank higher than description match (saveData)
    assert_eq!(results[0].script.name, "zzzFile");
    assert_eq!(results[1].script.name, "saveData");
}

#[test]
fn test_relevance_ranking_realistic_scenario() {
    let scripts = wrap_scripts(vec![
        Script {
            name: "grep".to_string(),
            path: PathBuf::from("/grep.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("Search files with grep".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "find".to_string(),
            path: PathBuf::from("/grep-utils.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: Some("Find files".to_string()),
            alias: None,
            shortcut: None,
            ..Default::default()
        },
        Script {
            name: "search".to_string(),
            path: PathBuf::from("/search.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        },
    ]);

    let results = fuzzy_search_scripts(&scripts, "grep");
    // "grep" name should rank highest
    assert_eq!(results[0].script.name, "grep");
    // "find" with grep in path should rank second
    assert!(results.len() >= 2);
}

#[test]
fn test_mixed_content_search() {
    // Combine scripts and scriptlets in unified search
    let scripts = wrap_scripts(vec![Script {
        name: "copyClipboard".to_string(),
        path: PathBuf::from("/copy.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: Some("Copy to clipboard".to_string()),
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);

    let mut quick_copy = test_scriptlet_with_desc("Quick Copy", "ts", "copy()", "Copy selection");
    quick_copy.shortcut = Some("cmd c".to_string());
    let scriptlets = wrap_scriptlets(vec![quick_copy]);

    let results = fuzzy_search_unified(&scripts, &scriptlets, "copy");
    assert_eq!(results.len(), 2);
    // Verify both types are present
    let has_script = results.iter().any(|r| matches!(r, SearchResult::Script(_)));
    let has_scriptlet = results
        .iter()
        .any(|r| matches!(r, SearchResult::Scriptlet(_)));
    assert!(has_script);
    assert!(has_scriptlet);
}

