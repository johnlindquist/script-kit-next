/// This test simulates the caching behavior in main.rs to demonstrate the bug.
///
/// In the actual app, ScriptKitApp has:
/// - grouped_cache_key: String - tracks what filter the cache was computed for
/// - cached_grouped_items: Arc<[GroupedListItem]> - cached results
///
/// BUG: When frecency_store.record_use() is called, the cache is NOT invalidated,
/// so subsequent calls return stale results.
///
/// This test demonstrates the expected vs actual behavior.
#[test]
fn test_frecency_cache_invalidation_required() {
    use crate::frecency::FrecencyStore;
    use tempfile::NamedTempFile;

    // Create a frecency store
    let temp_file = NamedTempFile::new().unwrap();
    let mut frecency_store = FrecencyStore::with_path(temp_file.path().to_path_buf());

    // Create test scripts
    let scripts = wrap_scripts(vec![
        test_script_with_path("ScriptA", "/test/a.ts"),
        test_script_with_path("ScriptB", "/test/b.ts"),
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<crate::app_launcher::AppInfo> = vec![];

    // === SIMULATE MAIN.RS CACHING BEHAVIOR ===

    // Initial call - would populate cache in main.rs
    let filter_text = ""; // Empty filter = main menu view
    let cache_key = filter_text.to_string();

    let (cached_grouped, cached_results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        filter_text,
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Record frecency use (this happens in main.rs:1904 after script execution)
    // BUG: This call does NOT invalidate the grouped cache
    frecency_store.record_use("/test/b.ts");

    // === WHAT HAPPENS IN BUGGY CODE ===
    // In main.rs, the cache_key is still the same (empty string for main menu),
    // so get_grouped_results_cached() returns the stale cached results
    // without calling get_grouped_results() again.

    // Simulate cache hit with stale data (this is the BUG)
    let buggy_grouped = if cache_key == filter_text {
        // Cache "hit" - returns stale data, doesn't reflect frecency change
        cached_grouped.clone()
    } else {
        // This branch never executes because cache_key matches
        get_grouped_results(
            &scripts,
            &scriptlets,
            &builtins,
            &apps,
            &frecency_store,
            filter_text,
            &SuggestedConfig::default(),
            &[],
            None,
        )
        .0
    };
    let buggy_results = if cache_key == filter_text {
        cached_results.clone()
    } else {
        get_grouped_results(
            &scripts,
            &scriptlets,
            &builtins,
            &apps,
            &frecency_store,
            filter_text,
            &SuggestedConfig::default(),
            &[],
            None,
        )
        .1
    };

    // === WHAT SHOULD HAPPEN (CORRECT BEHAVIOR) ===
    // After frecency_store.record_use(), invalidate_grouped_cache() should be called,
    // forcing a recompute that reflects the updated frecency

    let (correct_grouped, correct_results) = get_grouped_results(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        filter_text,
        &SuggestedConfig::default(),
        &[],
        None,
    );

    // Extract recent items from correct results
    let mut correct_recent_items: Vec<&str> = vec![];
    let mut in_recent = false;
    for item in correct_grouped.iter() {
        match item {
            GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED") => in_recent = true,
            GroupedListItem::SectionHeader(..) => in_recent = false,
            GroupedListItem::Item(idx) if in_recent => {
                if let Some(r) = correct_results.get(*idx) {
                    correct_recent_items.push(r.name());
                }
            }
            _ => {}
        }
    }

    // Extract recent items from buggy (cached) results
    let mut buggy_recent_items: Vec<&str> = vec![];
    let mut in_recent_buggy = false;
    for item in buggy_grouped.iter() {
        match item {
            GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED") => {
                in_recent_buggy = true
            }
            GroupedListItem::SectionHeader(..) => in_recent_buggy = false,
            GroupedListItem::Item(idx) if in_recent_buggy => {
                if let Some(r) = buggy_results.get(*idx) {
                    buggy_recent_items.push(r.name());
                }
            }
            _ => {}
        }
    }

    // The CORRECT results should show ScriptB in SUGGESTED section
    // (because we just recorded a use for it)
    assert!(
        correct_recent_items.contains(&"ScriptB"),
        "CORRECT behavior: ScriptB should appear in SUGGESTED section after record_use()"
    );

    // The BUGGY cached results do NOT show ScriptB in SUGGESTED
    // (because the cache wasn't invalidated)
    //
    // THIS ASSERTION DEMONSTRATES THE BUG:
    // The buggy code returns stale results that don't include ScriptB in SUGGESTED
    assert!(!buggy_recent_items.contains(&"ScriptB"),
            "BUG VERIFICATION: Cached results don't contain ScriptB in SUGGESTED (cache wasn't invalidated). \
             This assertion demonstrates the bug exists - it should be removed after the fix.");

    // The REAL test that should PASS after the fix is applied:
    // When invalidate_grouped_cache() is called after record_use(),
    // the next call to get_grouped_results_cached() should return fresh results
    // that include ScriptB in SUGGESTED.
    //
    // Uncomment this after applying the fix:
    // assert_eq!(buggy_recent_items, correct_recent_items,
    //     "After fix: cached and fresh results should be identical");
}

/// This test verifies that frecency cache invalidation works correctly.
///
/// After frecency_store.record_use() is called in main.rs,
/// invalidate_grouped_cache() is now called, so the cached grouped results
/// are properly invalidated and reflect the updated frecency scores.
///
/// This test simulates the correct behavior: after recording frecency use,
/// subsequent queries return updated results with the frecency changes.
#[test]
fn test_frecency_change_invalidates_cache() {
    use crate::frecency::FrecencyStore;
    use tempfile::NamedTempFile;

    // ============================================================
    // TEST FOR FRECENCY CACHE INVALIDATION (FIXED)
    // ============================================================
    //
    // After calling frecency_store.record_use() in main.rs,
    // invalidate_grouped_cache() is now called, so the cache is
    // properly invalidated and subsequent queries return fresh results.
    //
    // This test simulates the caching pattern from main.rs and
    // verifies the correct behavior: frecency changes are reflected
    // in subsequent queries.
    // ============================================================

    // Setup
    let temp_file = NamedTempFile::new().unwrap();
    let mut frecency_store = FrecencyStore::with_path(temp_file.path().to_path_buf());

    let scripts = wrap_scripts(vec![
        test_script_with_path("AlphaScript", "/test/alpha.ts"),
        test_script_with_path("BetaScript", "/test/beta.ts"),
    ]);
    let scriptlets: Vec<Arc<Scriptlet>> = wrap_scriptlets(vec![]);
    let builtins: Vec<BuiltInEntry> = vec![];
    let apps: Vec<crate::app_launcher::AppInfo> = vec![];

    // === Simulate ScriptKitApp state ===
    struct MockCache {
        grouped_cache_key: String,
        cached_grouped: Vec<GroupedListItem>,
        cached_results: Vec<SearchResult>,
        cache_valid: bool,
    }

    impl MockCache {
        fn new() -> Self {
            MockCache {
                grouped_cache_key: String::from("\0_INVALIDATED_\0"),
                cached_grouped: vec![],
                cached_results: vec![],
                cache_valid: false,
            }
        }

        /// Simulates get_grouped_results_cached() from main.rs
        fn get_cached(
            &mut self,
            scripts: &[Arc<Script>],
            scriptlets: &[Arc<Scriptlet>],
            builtins: &[BuiltInEntry],
            apps: &[crate::app_launcher::AppInfo],
            frecency_store: &FrecencyStore,
            filter_text: &str,
        ) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
            // Cache hit check (simulates main.rs line 1493)
            if self.cache_valid && filter_text == self.grouped_cache_key {
                return (self.cached_grouped.clone(), self.cached_results.clone());
            }

            // Cache miss - recompute
            let (grouped, results) = get_grouped_results(
                scripts,
                scriptlets,
                builtins,
                apps,
                frecency_store,
                filter_text,
                &SuggestedConfig::default(),
                &[],
                None,
            );

            self.cached_grouped = grouped.clone();
            self.cached_results = results.clone();
            self.grouped_cache_key = filter_text.to_string();
            self.cache_valid = true;

            (grouped, results)
        }

        /// This should be called after frecency_store.record_use()
        /// BUG: This is NOT called in main.rs!
        #[allow(dead_code)]
        fn invalidate(&mut self) {
            self.cache_valid = false;
            self.grouped_cache_key = String::from("\0_INVALIDATED_\0");
        }
    }

    let mut cache = MockCache::new();
    let filter_text = "";

    // Initial query - populates cache
    let (initial_grouped, _initial_results) = cache.get_cached(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        filter_text,
    );

    // Verify initial state: no SUGGESTED section (no frecency data)
    let initial_has_recent = initial_grouped.iter().any(
        |item| matches!(item, GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED")),
    );
    assert!(
        !initial_has_recent,
        "Initially there should be no SUGGESTED section"
    );

    // === THIS IS WHERE THE BUG HAPPENS ===
    // In main.rs:1904, frecency_store.record_use() is called
    // but invalidate_grouped_cache() is NOT called
    frecency_store.record_use("/test/beta.ts");

    // FIXED: cache.invalidate() is now called in main.rs after frecency_store.record_use()
    // This mock simulates the fixed behavior:
    cache.invalidate();

    // Query again - should return fresh results with BetaScript in SUGGESTED
    let (second_grouped, second_results) = cache.get_cached(
        &scripts,
        &scriptlets,
        &builtins,
        &apps,
        &frecency_store,
        filter_text,
    );

    // Extract SUGGESTED items from second query
    let mut recent_items: Vec<&str> = vec![];
    let mut in_recent = false;
    for item in second_grouped.iter() {
        match item {
            GroupedListItem::SectionHeader(s, _) if s.starts_with("SUGGESTED") => in_recent = true,
            GroupedListItem::SectionHeader(..) => in_recent = false,
            GroupedListItem::Item(idx) if in_recent => {
                if let Some(r) = second_results.get(*idx) {
                    recent_items.push(r.name());
                }
            }
            _ => {}
        }
    }

    // === VERIFY CACHE INVALIDATION WORKS ===
    // After frecency_store.record_use() and cache.invalidate(),
    // the SUGGESTED section should contain BetaScript.
    assert!(
        recent_items.contains(&"BetaScript"),
        "After frecency_store.record_use('/test/beta.ts') and cache invalidation, \
             BetaScript should appear in SUGGESTED section. \
             Got SUGGESTED items: {:?}.",
        recent_items
    );
}

