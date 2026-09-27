const IMPL_SCROLL: &str = include_str!("../src/app_navigation/impl_scroll.rs");
const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source.find(start).expect("start marker missing");
    let rest = &source[start_ix..];
    let end_ix = rest.find(end).expect("end marker missing");
    &rest[..end_ix]
}

fn position(body: &str, needle: &str) -> usize {
    body.find(needle)
        .unwrap_or_else(|| panic!("filter replacement is missing `{needle}`"))
}

fn sync_body() -> &'static str {
    section_between(
        IMPL_SCROLL,
        "pub fn sync_list_state_for_filter_replacement(",
        "let restored_offset = match policy {",
    )
}

#[test]
fn filter_list_state_replacement_splices_count_changes() {
    let sync_body = sync_body();
    let count_branch_ix = position(sync_body, "if old_list_count != item_count {");
    let splice_ix = position(
        sync_body,
        "self.main_list_state.splice(0..old_list_count, item_count);",
    );
    let empty_branch_ix = position(sync_body, "} else if item_count == 0 {");
    let new_ix = position(sync_body, "self.main_list_state = ListState::new(");

    assert!(
        count_branch_ix < splice_ix && splice_ix < empty_branch_ix && empty_branch_ix < new_ix,
        "count-changing filter replacement must splice GPUI ListState in its own branch, before the same-count ListState replacement"
    );
}

#[test]
fn filter_list_state_replacement_skips_empty_same_count_rebuilds() {
    let sync_body = sync_body();
    let empty_branch_ix = position(sync_body, "} else if item_count == 0 {");
    let same_count_branch_ix = empty_branch_ix
        + sync_body[empty_branch_ix..]
            .find("} else {")
            .expect("same-count replacement branch missing");
    let new_ix = position(sync_body, "self.main_list_state = ListState::new(");

    assert!(
        !sync_body[empty_branch_ix..same_count_branch_ix].contains("ListState::new("),
        "empty same-count replacement must not rebuild ListState"
    );
    assert!(
        same_count_branch_ix < new_ix,
        "only the non-empty same-count branch replaces ListState"
    );
}

#[test]
fn script_list_filter_reconciliation_still_validates_and_reveals() {
    let reconcile_body = section_between(
        FILTER_INPUT_UPDATES,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
        "pub(crate) fn commit_main_menu_results_refresh(",
    );

    let reconcile_selection_ix = position(
        reconcile_body,
        "self.reconcile_main_menu_selection_intent();",
    );
    let sync_ix = position(
        reconcile_body,
        "self.sync_list_state_for_filter_replacement(MainListReplacementPolicy::ResetToTop);",
    );
    assert!(
        reconcile_selection_ix < sync_ix,
        "script-list filter reconciliation must bound the selection to the new rows before list-state sync resets the viewport to the top"
    );
}
