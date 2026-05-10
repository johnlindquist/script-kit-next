//! Regression tests for the root launcher file-search boundary.
//!
//! Root search may append bounded file rows, but the unified matcher itself
//! must stay a pure in-memory ranker. Spotlight process ownership remains in
//! the file-search module, and the dedicated File Search view keeps its richer
//! directory-browser behavior.

#[cfg(test)]
mod tests {
    use std::fs;

    fn production_source(source: &str) -> &str {
        source.split("#[cfg(test)]").next().unwrap_or(source)
    }

    #[test]
    fn unified_search_module_does_not_call_file_search_processes() {
        let source = fs::read_to_string("src/scripts/search/unified.rs")
            .expect("read src/scripts/search/unified.rs");
        let production = production_source(&source);

        for forbidden in ["mdfind", "search_files(", "search_files_streaming"] {
            assert!(
                !production.contains(forbidden),
                "unified search should not call file search process APIs directly: {forbidden}"
            );
        }
    }

    #[test]
    fn dedicated_file_search_still_owns_file_search_view_navigation() {
        let list_source = fs::read_to_string("src/render_builtins/file_search_list.rs")
            .expect("read src/render_builtins/file_search_list.rs");
        let view_source = fs::read_to_string("src/render_builtins/file_search.rs")
            .expect("read src/render_builtins/file_search.rs");

        assert!(
            list_source.contains("AppView::FileSearchView"),
            "dedicated File Search view should remain a distinct browser surface"
        );
        assert!(
            view_source.contains("Double-click: browse directory inline or open file")
                && view_source.contains("Tab/Shift+Tab handled by intercept_keystrokes"),
            "dedicated File Search should keep directory browsing and parent navigation"
        );
    }

    #[test]
    fn root_streaming_search_disables_filesystem_fallback() {
        let source = fs::read_to_string("src/file_search/mdfind.rs")
            .expect("read src/file_search/mdfind.rs");
        let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("pub fn root_search() -> Self { Self { skip_metadata: true, allow_filesystem_fallback: false"),
            "root search options should skip metadata and disable filesystem fallback"
        );
        assert!(
            normalized
                .contains("SearchFilesStreamingOptions::dedicated_file_search(skip_metadata)"),
            "existing streaming entry point should preserve dedicated File Search defaults"
        );
    }
}
