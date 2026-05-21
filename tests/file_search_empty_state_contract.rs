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
        FILE_SEARCH_RENDERER.contains("let state = FileSearchEmptyState::from_query(query);")
            && FILE_SEARCH_RENDERER.contains("EmptyState::new(state.title()"),
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

#[test]
fn file_search_empty_state_uses_handler_text_scale_without_accent_pill() {
    let empty_state_renderer = FILE_SEARCH_RENDERER
        .split("let render_empty_list_state = || {")
        .nth(1)
        .and_then(|source| source.split("};\n\n        // Footer").next())
        .expect("file search empty-state renderer closure should be present");

    assert!(
        empty_state_renderer.contains(".text_size(px(19.0))")
            && empty_state_renderer.contains(".line_height(px(24.0))")
            && empty_state_renderer.contains(".text_size(px(13.0))")
            && empty_state_renderer.contains(".line_height(px(19.0))"),
        "file search empty-state text should track the handler-form title/body scale"
    );
    assert!(
        !empty_state_renderer.contains(".bg(rgb(accent_color))")
            && !empty_state_renderer.contains(".w(px(56.0))")
            && !empty_state_renderer.contains(".h(px(10.0))"),
        "file search empty state must not render the old accent-colored pill"
    );
}
