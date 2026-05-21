const FILE_SEARCH_RENDERER: &str = include_str!("../src/render_builtins/file_search.rs");
const FILE_SEARCH_CORE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const FILE_SEARCH_UTILITY: &str = include_str!("../src/app_execute/utility_views.rs");
const ROOT_FILE_SEARCH: &str = include_str!("../src/app_impl/root_file_search.rs");

#[test]
fn file_search_empty_state_copy_is_modeled() {
    assert!(
        FILE_SEARCH_RENDERER.contains("enum FileSearchEmptyState")
            && FILE_SEARCH_RENDERER.contains("NoFilesFound"),
        "file search empty-state copy should use the remaining no-results state"
    );
    assert!(
        FILE_SEARCH_RENDERER.contains("fn from_query(_query: &str) -> Self")
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
    assert!(
        !FILE_SEARCH_RENDERER.contains("TypeToSearch")
            && !FILE_SEARCH_RENDERER.contains("Type to search files")
            && !FILE_SEARCH_RENDERER.contains("empty_idle"),
        "default File Search should not retain the old type-to-search empty state"
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

#[test]
fn file_search_default_state_seeds_recent_files_before_first_paint() {
    assert!(
        FILE_SEARCH_CORE.contains("fn seed_file_search_default_results_for_first_paint")
            && FILE_SEARCH_CORE.contains("recent_file_results_from_frecency")
            && FILE_SEARCH_CORE.contains("list_directory_with_options(\n                \"~/\"")
            && FILE_SEARCH_CORE.contains("presentation == FileSearchPresentation::Full")
            && FILE_SEARCH_CORE.contains("query.trim().is_empty()"),
        "fresh full File Search opens must seed frecency-backed recent files, falling back to the existing ~/ directory listing pattern"
    );
    assert!(
        FILE_SEARCH_CORE.contains("if seeded_default_results")
            && FILE_SEARCH_CORE.contains("return;")
            && FILE_SEARCH_CORE.contains("begin_file_search_session"),
        "seeded default File Search should not immediately clear recents with an empty Spotlight search"
    );
}

#[test]
fn file_search_and_root_launcher_share_recent_file_source() {
    assert!(
        FILE_SEARCH_UTILITY.contains("pub(crate) fn recent_file_results_from_frecency")
            && FILE_SEARCH_UTILITY.contains("top_file_paths")
            && FILE_SEARCH_UTILITY.contains("file_result_from_existing_path"),
        "File Search recents should reuse the app's frecency-backed recent file source"
    );
    assert!(
        ROOT_FILE_SEARCH
            .contains("self.recent_file_results_from_frecency(crate::file_search::ROOT_FILE_RECENT_SEED_LIMIT)")
            && !ROOT_FILE_SEARCH.contains("top_file_paths(crate::file_search::ROOT_FILE_RECENT_HYDRATE_LIMIT)"),
        "root launcher and full File Search should not maintain divergent recent-file hydration logic"
    );
}
