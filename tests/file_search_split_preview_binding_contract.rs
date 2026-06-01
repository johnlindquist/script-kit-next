const FILE_SEARCH: &str = include_str!("../src/render_builtins/file_search.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const FILE_ACTIONS: &str = include_str!("../src/app_actions/handle_action/files.rs");
const UTILITY_VIEWS: &str = include_str!("../src/app_execute/utility_views.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let rest = &source[start..];
    let body_start = rest
        .find('{')
        .unwrap_or_else(|| panic!("missing function body: {signature}"));
    let mut depth = 0usize;
    for (idx, ch) in rest[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[..body_start + idx + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

#[test]
fn file_search_state_uses_display_projection_for_selected_value() {
    let arm_pos = PROMPT_HANDLER
        .find("AppView::FileSearchView {\n                        ref query,")
        .expect("FileSearchView getState arm must exist");
    let body_end = PROMPT_HANDLER[arm_pos..]
        .find("\n                    AppView::ThemeChooserView")
        .expect("FileSearchView arm should precede ThemeChooserView");
    let body = &PROMPT_HANDLER[arm_pos..arm_pos + body_end];

    assert!(body.contains("let selection = self.file_search_selection_binding(*selected_index);"));
    assert!(body.contains("self.cached_file_results.len(),"));
    assert!(body.contains("self.file_search_display_indices.len(),"));
    assert!(body.contains("selection.file.as_ref().map(|file| file.name.clone())"));
    assert!(
        !body.contains(
            "self.cached_file_results\n                            .get(*selected_index)"
        ),
        "getState must not report selectedValue from raw cached_file_results"
    );
}

#[test]
fn file_search_render_and_keys_use_same_selection_binding() {
    assert!(
        UTILITY_VIEWS.contains("pub(crate) fn file_search_selection_binding(")
            && UTILITY_VIEWS.contains("resolve_file_search_selection_projection("),
        "File Search must expose one selection-binding projection owner"
    );

    let render_body = function_body(FILE_SEARCH, "pub(crate) fn render_file_search(");
    assert!(
        render_body.contains("let selection = self.file_search_selection_binding(selected_index);")
            && render_body.contains("let selected_file = selection.file.clone();")
            && render_body.contains("projection.display_index")
            && render_body.contains("let get_selected_file = || this.file_search_selection_binding(sel_idx).file;"),
        "render, preview, and key handlers must resolve File Search selection from the same binding"
    );
}

#[test]
fn file_search_actions_resolve_path_from_same_selection_binding() {
    let body = function_body(FILE_ACTIONS, "fn resolve_file_search_path_info(&self)");
    assert!(
        body.contains("let binding = self.file_search_selection_binding(*selected_index);"),
        "file actions should resolve selected path through the shared File Search binding"
    );
    assert!(
        !body.contains("self.selected_file_search_result(*selected_index)?"),
        "file actions must not use a different selected-row resolver than preview/state"
    );
}
