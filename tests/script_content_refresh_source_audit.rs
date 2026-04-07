//! Source-level regression lock for watcher-driven content index refresh.
//!
//! These tests read source files and assert that key strings required for
//! the content-search refresh pipeline remain in place. They catch silent
//! regressions where someone removes cache invalidation, body indexing,
//! or watcher file-type checks without updating the content-search feature.

use std::fs;

#[test]
fn script_refresh_still_invalidates_content_search_caches() {
    let source = fs::read_to_string("src/app_impl/refresh_scriptlets.rs")
        .expect("should read src/app_impl/refresh_scriptlets.rs");

    for needle in [
        "self.invalidate_filter_cache();",
        "self.invalidate_grouped_cache();",
        "self.invalidate_preview_cache();",
        "script_content_index_refresh:",
        "s.body.is_some()",
    ] {
        assert!(
            source.contains(needle),
            "refresh path must keep `{needle}` so content-search state is rebuilt after script changes"
        );
    }
}

#[test]
fn script_watcher_still_treats_ts_and_js_as_relevant_script_files() {
    let source = fs::read_to_string("src/watcher/mod.rs").expect("should read src/watcher/mod.rs");

    assert!(
        source.contains("Some(\"ts\") | Some(\"js\") | Some(\"md\")"),
        "script watcher must continue emitting reloads for ts/js/md changes"
    );
}

#[test]
fn script_loader_still_reads_bodies_for_full_text_indexing() {
    let source =
        fs::read_to_string("src/scripts/loader.rs").expect("should read src/scripts/loader.rs");

    assert!(
        source.contains("std::fs::read_to_string(&path)"),
        "loader must keep reading script files into memory for content search"
    );
    assert!(
        source.contains("Failed to read script body for content indexing"),
        "loader should keep the content-indexing diagnostic"
    );
}
