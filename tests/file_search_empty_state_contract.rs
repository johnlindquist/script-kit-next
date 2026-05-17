const FILE_SEARCH_RENDERER: &str = include_str!("../src/render_builtins/file_search.rs");

#[test]
fn file_search_empty_state_copy_is_modeled() {
    assert!(
        FILE_SEARCH_RENDERER.contains("enum FileSearchEmptyState")
            && FILE_SEARCH_RENDERER.contains("TypeToSearch")
            && FILE_SEARCH_RENDERER.contains("NoFilesFound"),
        "file search empty-state copy should use named states"
    );
    assert!(
        FILE_SEARCH_RENDERER.contains("fn from_query(query: &str) -> Self")
            && FILE_SEARCH_RENDERER.contains("fn audit_state(self) -> &'static str")
            && FILE_SEARCH_RENDERER.contains("fn render_state(self) -> &'static str")
            && FILE_SEARCH_RENDERER.contains("fn title(self) -> &'static str"),
        "file search empty states should own renderer, audit, and title strings"
    );
}

#[test]
fn file_search_empty_state_render_paths_use_model() {
    assert!(
        FILE_SEARCH_RENDERER.contains("FileSearchEmptyState::from_query(query).audit_state()"),
        "file search state audit should derive empty-state copy from the model"
    );
    assert!(
        FILE_SEARCH_RENDERER.contains("FileSearchEmptyState::from_query(query).title()"),
        "inline list empty title should derive from the model"
    );
    assert!(
        FILE_SEARCH_RENDERER.contains("state = empty_state.render_state()"),
        "full file search empty telemetry should derive from the model"
    );
    assert!(
        !FILE_SEARCH_RENDERER.contains("if query.is_empty() { \"Type to search files\""),
        "visible empty-state copy must not regress to inline query-empty branching"
    );
    assert!(
        !FILE_SEARCH_RENDERER.contains("if query.is_empty() { \"empty_idle\""),
        "empty-state telemetry must not regress to inline query-empty branching"
    );
}
