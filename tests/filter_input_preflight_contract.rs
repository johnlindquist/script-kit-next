const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source.find(start).expect("start marker missing");
    let rest = &source[start_ix..];
    let end_ix = rest.find(end).expect("end marker missing");
    &rest[..end_ix]
}

fn position(body: &str, needle: &str, what: &str) -> usize {
    body.find(needle)
        .unwrap_or_else(|| panic!("{what} missing: `{needle}`"))
}

#[test]
fn script_list_reconciliation_rebuilds_preflight_and_ghost_from_the_published_state() {
    let reconcile_body = section_between(
        FILTER_INPUT_UPDATES,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
        "pub(crate) fn commit_main_menu_results_refresh(",
    );
    assert!(
        reconcile_body.contains("self.finish_main_menu_publication(reason, None, cx);"),
        "reconciliation must publish through finish_main_menu_publication"
    );

    let publication_body = section_between(
        FILTER_INPUT_UPDATES,
        "fn finish_main_menu_publication(",
        "fn apply_filter_compute_now(",
    );
    let invalidate_ix = position(
        publication_body,
        "self.invalidate_main_window_preflight();",
        "preflight invalidation",
    );
    let rebuild_ix = position(
        publication_body,
        "self.rebuild_main_window_preflight_if_needed();",
        "preflight rebuild",
    );
    let ghost_ix = position(
        publication_body,
        "self.refresh_ghost_with_input(cx);",
        "ghost refresh",
    );
    assert!(
        invalidate_ix < rebuild_ix && rebuild_ix < ghost_ix,
        "publication must invalidate, then rebuild preflight, then refresh the ghost from the published rows"
    );
}

#[test]
fn set_filter_text_immediate_rebuilds_preflight_once_after_fallback_updates() {
    let body = section_between(
        FILTER_INPUT_UPDATES,
        "pub(crate) fn set_filter_text_and_cursor_immediate(",
        "pub(crate) fn handle_script_list_printable_simulate_key(",
    );

    assert_eq!(
        body.matches("self.rebuild_main_window_preflight_if_needed();")
            .count(),
        1,
        "set_filter_text_immediate should own exactly one final preflight rebuild"
    );

    let fallback_ix = position(
        body,
        "self.main_menu_fallback_state.clear();",
        "fallback update",
    );
    let commit_ix = position(
        body,
        "self.apply_filter_compute_now(text.clone(), cx);",
        "query commit",
    );
    let rebuild_ix = position(
        body,
        "self.rebuild_main_window_preflight_if_needed();",
        "final preflight rebuild",
    );

    assert!(
        fallback_ix < commit_ix && commit_ix < rebuild_ix,
        "final preflight rebuild must stay after fallback updates and the query commit"
    );
}
