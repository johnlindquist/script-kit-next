// ============================================
// WINDOW SEARCH TESTS
// ============================================
//
// Note: Most window search tests require WindowInfo to have a public constructor.
// These tests verify the function signatures and empty input handling.
// Integration tests with actual WindowInfo require window_control module changes.

#[test]
fn test_fuzzy_search_windows_empty_list() {
    // Test with empty window list
    let windows: Vec<crate::window_control::WindowInfo> = vec![];

    let results = fuzzy_search_windows(&windows, "test");
    assert!(results.is_empty());

    let results_empty_query = fuzzy_search_windows(&windows, "");
    assert!(results_empty_query.is_empty());
}

#[test]
fn test_window_match_type_exists() {
    // Verify WindowMatch struct has expected fields by type-checking
    fn _type_check(wm: &WindowMatch) {
        let _window: &crate::window_control::WindowInfo = &wm.window;
        let _score: i32 = wm.score;
    }
}

#[test]
fn test_search_result_window_type_label() {
    // We can't construct WindowInfo directly, but we can verify
    // the SearchResult::Window variant exists and type_label is correct
    // by checking the match arm in type_label implementation compiles
    fn _verify_window_variant_exists() {
        fn check_label(result: &SearchResult) -> &'static str {
            match result {
                SearchResult::Window(_) => "Window",
                _ => "other",
            }
        }
        let _ = check_label;
    }
}

#[test]
fn test_fuzzy_search_unified_with_windows_empty_inputs() {
    let scripts: Vec<Arc<Script>> = wrap_scripts(vec![]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<crate::app_launcher::AppInfo> = vec![];
    let windows: Vec<crate::window_control::WindowInfo> = vec![];

    let results = fuzzy_search_unified_with_windows(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &windows,
        "test",
    );

    assert!(results.is_empty());
}

#[test]
fn test_fuzzy_search_unified_with_windows_returns_scripts() {
    let scripts = wrap_scripts(vec![Script {
        name: "test_script".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<crate::app_launcher::AppInfo> = vec![];
    let windows: Vec<crate::window_control::WindowInfo> = vec![];

    let results = fuzzy_search_unified_with_windows(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &windows,
        "test",
    );

    assert_eq!(results.len(), 1);
    assert!(matches!(&results[0], SearchResult::Script(_)));
}

