// ============================================
// FRECENCY CACHE INVALIDATION TESTS
// ============================================
//
// BUG: When frecency_store.record_use() is called in main.rs:1904,
// the grouped_results cache is NOT invalidated. This means when
// the window is re-shown, stale cached results are returned instead
// of results reflecting the updated frecency scores.
//
// These tests verify the expected behavior of get_grouped_results()
// with frecency - the pure function works correctly. The cache
// invalidation bug is in main.rs::get_grouped_results_cached().

/// Helper to create a test Script with a given path
fn test_script_with_path(name: &str, path: &str) -> Script {
    Script {
        name: name.to_string(),
        path: PathBuf::from(path),
        extension: "ts".to_string(),
        icon: None,
        description: None,
        alias: None,
        shortcut: None,
        ..Default::default()
    }
}

#[test]
fn test_get_grouped_results_respects_frecency_ordering() {
    use crate::frecency::FrecencyStore;
    use tempfile::NamedTempFile;

    // Create a frecency store with temp file
    let temp_file = NamedTempFile::new().unwrap();
    let mut frecency_store = FrecencyStore::with_path(temp_file.path().to_path_buf());

    // Create test scripts
    let scripts = wrap_scripts(vec![
        test_script_with_path("Alpha Script", "/test/alpha.ts"),
        test_script_with_path("Beta Script", "/test/beta.ts"),
        test_script_with_path("Gamma Script", "/test/gamma.ts"),
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<crate::app_launcher::AppInfo> = vec![];

    // Initially no frecency - should return alphabetical order in MAIN section
    let (grouped1, _results1) = get_grouped_results(
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

    // Should have MAIN section header + 3 items
    assert!(grouped1.len() >= 3);

    // Record use for Gamma (should become "recent")
    frecency_store.record_use("/test/gamma.ts");

    // Now get_grouped_results should show Gamma in SUGGESTED section
    let (grouped2, results2) = get_grouped_results(
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

    // Should now have SUGGESTED header, at least one recent item, MAIN header, remaining items
    // The first section header should be "SUGGESTED"
    let first_header = grouped2.iter().find_map(|item| match item {
        GroupedListItem::SectionHeader(s, _) => Some(s.clone()),
        _ => None,
    });
    assert!(
        first_header
            .as_ref()
            .is_some_and(|s| s.starts_with("SUGGESTED")),
        "After recording use, SUGGESTED section should appear"
    );

    // Find the first item after the SUGGESTED header - it should be Gamma
    let mut found_recent_header = false;
    let mut first_recent_item: Option<&SearchResult> = None;
    for item in grouped2.iter() {
        match item {
            GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED") => {
                found_recent_header = true;
            }
            GroupedListItem::Item(idx) if found_recent_header && first_recent_item.is_none() => {
                first_recent_item = results2.get(*idx);
                break;
            }
            _ => {}
        }
    }

    assert!(
        first_recent_item.is_some(),
        "Should have at least one item in SUGGESTED section"
    );
    assert_eq!(
        first_recent_item.unwrap().name(),
        "Gamma Script",
        "The most recently used script should appear first in SUGGESTED section"
    );
}

#[test]
fn test_get_grouped_results_updates_after_frecency_change() {
    use crate::frecency::FrecencyStore;
    use tempfile::NamedTempFile;

    // Create a frecency store with temp file
    let temp_file = NamedTempFile::new().unwrap();
    let mut frecency_store = FrecencyStore::with_path(temp_file.path().to_path_buf());

    // Create test scripts
    let scripts = wrap_scripts(vec![
        test_script_with_path("First Script", "/test/first.ts"),
        test_script_with_path("Second Script", "/test/second.ts"),
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<crate::app_launcher::AppInfo> = vec![];

    // Record initial use for First
    frecency_store.record_use("/test/first.ts");

    // Get initial results
    let (grouped1, results1) = get_grouped_results(
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

    // Find the first recent item - should be "First Script"
    let first_recent_1 = grouped1
        .iter()
        .filter_map(|item| match item {
            GroupedListItem::Item(idx) => results1.get(*idx),
            _ => None,
        })
        .next();
    assert_eq!(first_recent_1.map(|r| r.name()), Some("First Script"));

    // Now record use for Second (multiple times to ensure higher frecency)
    frecency_store.record_use("/test/second.ts");
    frecency_store.record_use("/test/second.ts");
    frecency_store.record_use("/test/second.ts");

    // Get updated results - THIS IS WHERE THE BUG MANIFESTS IN MAIN.RS
    // The pure function correctly returns updated results,
    // but get_grouped_results_cached() would return stale cached results
    // because invalidate_grouped_cache() is not called after record_use()
    let (grouped2, results2) = get_grouped_results(
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

    // Find items in SUGGESTED section
    let mut in_recent_section = false;
    let mut recent_items: Vec<&str> = vec![];
    for item in grouped2.iter() {
        match item {
            GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED") => {
                in_recent_section = true;
            }
            GroupedListItem::SectionHeader(..) => {
                in_recent_section = false;
            }
            GroupedListItem::Item(idx) if in_recent_section => {
                if let Some(result) = results2.get(*idx) {
                    recent_items.push(result.name());
                }
            }
            _ => {}
        }
    }

    // Second Script should now be first in SUGGESTED (higher frecency score)
    assert!(!recent_items.is_empty(), "Should have recent items");
    assert_eq!(
        recent_items[0], "Second Script",
        "Script with higher frecency (more uses) should appear first in SUGGESTED"
    );
}

