//! Source-level contract for the AURP-05 Current App Commands visible-row owner.
//!
//! The surface has three observable paths that can drift independently:
//! renderer rows, `getState.visibleChoiceCount`, and `getElements` rows.
//! This contract keeps all three routed through the same named helper family.

const CURRENT_APP_COMMANDS_RENDERER: &str =
    include_str!("../src/render_builtins/current_app_commands.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

#[test]
fn helper_family_keeps_filter_menu_bar_entries_as_the_single_filter_owner() {
    assert!(
        CURRENT_APP_COMMANDS_RENDERER.contains("fn current_app_commands_filtered_entries"),
        "Current App Commands must expose a named filtered-entry helper."
    );
    assert!(
        CURRENT_APP_COMMANDS_RENDERER
            .contains("builtins::filter_menu_bar_entries(entries, filter)"),
        "The filtered-entry helper must delegate to the menu-bar matcher so name, \
         description, keyword, and multi-term semantics stay intact."
    );
    assert!(
        CURRENT_APP_COMMANDS_RENDERER.contains("fn current_app_commands_visible_row_names"),
        "getElements needs a named row projection rather than remapping entries inline."
    );
    assert!(
        CURRENT_APP_COMMANDS_RENDERER
            .contains("fn current_app_commands_dataset_and_visible_counts"),
        "getState needs a named count helper that returns dataset count and visible count together."
    );
    assert!(
        CURRENT_APP_COMMANDS_RENDERER.contains("fn current_app_commands_selected_visible_row_name"),
        "getState selectedValue must come from the same visible filtered row set."
    );
    assert_eq!(
        CURRENT_APP_COMMANDS_RENDERER
            .matches("filter_menu_bar_entries(")
            .count(),
        1,
        "The renderer module should call the raw menu-bar filter only inside the helper."
    );
}

#[test]
fn renderer_routes_all_visible_lists_through_filtered_entry_helper() {
    assert!(
        CURRENT_APP_COMMANDS_RENDERER.contains(
            "Self::current_app_commands_filtered_entries(&self.cached_current_app_entries, &filter)"
        ),
        "The initial render pass must use the shared filtered-entry helper."
    );
    assert_eq!(
        CURRENT_APP_COMMANDS_RENDERER
            .matches("&this.cached_current_app_entries")
            .count(),
        2,
        "Keyboard and wheel handlers must recompute rows through the shared helper."
    );
}

#[test]
fn get_state_routes_counts_and_selected_value_through_visible_row_helpers() {
    let body = source_between(
        PROMPT_HANDLER,
        "AppView::CurrentAppCommandsView {\n                        filter,",
        "\n                    AppView::SearchAiPresetsView",
    );
    assert!(
        body.contains("self.current_app_commands_dataset_and_visible_counts(filter)"),
        "getState must derive choiceCount and visibleChoiceCount from the shared helper."
    );
    assert!(
        body.contains("current_app_commands_selected_visible_row_name(")
            && body.contains("filter,\n                            *selected_index,"),
        "getState selectedValue must come from the shared visible filtered row set."
    );
    assert!(
        !body.contains("let filter_lower = filter.to_lowercase();"),
        "getState must not reintroduce an inline filter."
    );
    assert!(
        !body.contains("e.name.to_lowercase().contains(&filter_lower)"),
        "getState must not regress to name-only matching."
    );
}

#[test]
fn get_elements_routes_rows_through_visible_row_helper() {
    let body = source_between(
        COLLECT_ELEMENTS,
        "AppView::CurrentAppCommandsView {\n                filter,",
        "\n            AppView::EmojiPickerView",
    );
    assert!(
        body.contains("let rows = self.current_app_commands_visible_row_names(filter);"),
        "getElements must get rows from the shared visible-row helper."
    );
    assert!(
        !body.contains("filter_menu_bar_entries("),
        "getElements must not bypass the named visible-row helper."
    );
    assert!(
        !body.contains("let filter_lower = filter.to_lowercase();"),
        "getElements must not reintroduce an inline filter."
    );
}
