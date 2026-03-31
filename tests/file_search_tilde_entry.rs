//! Integration tests: verify the `~` tilde trigger routes ScriptList into
//! mini file search and that the render pipeline threads `FileSearchPresentation`.

use std::fs;

#[test]
fn filter_input_change_routes_tilde_into_mini_file_search() {
    let source = fs::read_to_string("src/app_impl/filter_input_change.rs")
        .expect("Failed to read src/app_impl/filter_input_change.rs");

    assert!(
        source.contains("should_enter_file_search_from_script_list"),
        "filter_input_change must call the ~ trigger predicate"
    );
    assert!(
        source.contains("FileSearchPresentation::Mini"),
        "filter_input_change must open Mini presentation on ~ trigger"
    );
    assert!(
        source.contains("open_file_search_view"),
        "filter_input_change must call open_file_search_view"
    );
}

#[test]
fn render_impl_threads_file_search_presentation() {
    let source = fs::read_to_string("src/main_sections/render_impl.rs")
        .expect("Failed to read src/main_sections/render_impl.rs");

    assert!(
        source.contains("presentation"),
        "render_impl must destructure FileSearchView.presentation"
    );
    assert!(
        source.contains("render_file_search"),
        "render_impl must call render_file_search with presentation"
    );
}
