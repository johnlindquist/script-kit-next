// ============================================
// ASCII CASE-FOLDING HELPER TESTS
// ============================================

#[test]
fn test_contains_ignore_ascii_case_basic() {
    // Note: needle_lower must already be lowercase
    assert!(contains_ignore_ascii_case("OpenFile", "open"));
    assert!(contains_ignore_ascii_case("OPENFILE", "open"));
    assert!(contains_ignore_ascii_case("openfile", "open"));
    assert!(contains_ignore_ascii_case("MyOpenFile", "open"));
}

#[test]
fn test_contains_ignore_ascii_case_not_found() {
    assert!(!contains_ignore_ascii_case("OpenFile", "save"));
    assert!(!contains_ignore_ascii_case("test", "testing"));
}

#[test]
fn test_contains_ignore_ascii_case_empty_needle() {
    assert!(contains_ignore_ascii_case("OpenFile", ""));
    assert!(contains_ignore_ascii_case("", ""));
}

#[test]
fn test_contains_ignore_ascii_case_needle_longer() {
    assert!(!contains_ignore_ascii_case("ab", "abc"));
}

#[test]
fn test_find_ignore_ascii_case_at_start() {
    assert_eq!(find_ignore_ascii_case("OpenFile", "open"), Some(0));
    assert_eq!(find_ignore_ascii_case("OPENFILE", "open"), Some(0));
}

#[test]
fn test_find_ignore_ascii_case_in_middle() {
    assert_eq!(find_ignore_ascii_case("MyOpenFile", "open"), Some(2));
}

#[test]
fn test_find_ignore_ascii_case_not_found() {
    assert_eq!(find_ignore_ascii_case("OpenFile", "save"), None);
}

#[test]
fn test_find_ignore_ascii_case_empty_needle() {
    assert_eq!(find_ignore_ascii_case("OpenFile", ""), Some(0));
}

#[test]
fn test_fuzzy_match_with_indices_ascii_basic() {
    let (matched, indices) = fuzzy_match_with_indices_ascii("OpenFile", "of");
    assert!(matched);
    assert_eq!(indices, vec![0, 4]); // 'O' at 0, 'F' at 4
}

#[test]
fn test_fuzzy_match_with_indices_ascii_case_insensitive() {
    // Note: pattern_lower must already be lowercase
    let (matched, indices) = fuzzy_match_with_indices_ascii("OpenFile", "of");
    assert!(matched);
    assert_eq!(indices, vec![0, 4]);
}

#[test]
fn test_fuzzy_match_with_indices_ascii_no_match() {
    let (matched, indices) = fuzzy_match_with_indices_ascii("test", "xyz");
    assert!(!matched);
    assert!(indices.is_empty());
}

#[test]
fn test_fuzzy_match_with_indices_ascii_empty_pattern() {
    let (matched, indices) = fuzzy_match_with_indices_ascii("test", "");
    assert!(matched);
    assert!(indices.is_empty());
}

#[test]
fn test_compute_match_indices_for_script_result() {
    let script_match = ScriptMatch {
        script: Arc::new(Script {
            name: "OpenFile".to_string(),
            path: PathBuf::from("/openfile.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        }),
        score: 100,
        filename: "openfile.ts".to_string(),
        match_indices: MatchIndices::default(),
    };
    let result = SearchResult::Script(script_match);

    let indices = compute_match_indices_for_result(&result, "of");
    assert!(!indices.name_indices.is_empty());
    assert_eq!(indices.name_indices, vec![0, 4]); // 'O' at 0, 'F' at 4
}

#[test]
fn test_compute_match_indices_for_scriptlet_result() {
    let scriptlet_match = ScriptletMatch {
        scriptlet: Arc::new(test_scriptlet("Copy Text", "ts", "copy()")),
        score: 100,
        display_file_path: Some("copy.md#copy-text".to_string()),
        match_indices: MatchIndices::default(),
    };
    let result = SearchResult::Scriptlet(scriptlet_match);

    let indices = compute_match_indices_for_result(&result, "ct");
    assert!(!indices.name_indices.is_empty());
    assert_eq!(indices.name_indices, vec![0, 5]); // 'C' at 0, 'T' at 5
}

#[test]
fn test_compute_match_indices_empty_query() {
    let script_match = ScriptMatch {
        script: Arc::new(Script {
            name: "Test".to_string(),
            path: PathBuf::from("/test.ts"),
            extension: "ts".to_string(),
            icon: None,
            description: None,
            alias: None,
            shortcut: None,
            ..Default::default()
        }),
        score: 0,
        filename: "test.ts".to_string(),
        match_indices: MatchIndices::default(),
    };
    let result = SearchResult::Script(script_match);

    let indices = compute_match_indices_for_result(&result, "");
    assert!(indices.name_indices.is_empty());
    assert!(indices.filename_indices.is_empty());
}

#[test]
fn test_scriptlet_code_search_gated_by_length() {
    // Code search only happens when query >= 4 chars and score == 0
    // Use a name that doesn't contain any of the search characters
    let scriptlets = wrap_scriptlets(vec![test_scriptlet(
        "Utility",
        "ts",
        "contains_xyz_function()",
    )]);

    // Short query - should NOT search code (even if it would match)
    let results = fuzzy_search_scriptlets(&scriptlets, "xyz");
    assert!(results.is_empty()); // No match because "xyz" only in code, and query < 4 chars

    // Long query >= 4 chars should search code when name doesn't match
    let results = fuzzy_search_scriptlets(&scriptlets, "xyz_f");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 5); // Only code match score
}

#[test]
fn test_scriptlet_code_search_skipped_when_name_matches() {
    // If name matches, code search is skipped (score > 0)
    let scriptlets = wrap_scriptlets(vec![test_scriptlet(
        "special_snippet",
        "ts",
        "unrelated_code()",
    )]);

    // Should match on name, not search code
    let results = fuzzy_search_scriptlets(&scriptlets, "special");
    assert_eq!(results.len(), 1);
    // Score should be from name match, not code match
    assert!(results[0].score > 5);
}

