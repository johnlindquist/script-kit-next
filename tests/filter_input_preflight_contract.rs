const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source.find(start).expect("start marker missing");
    let rest = &source[start_ix..];
    let end_ix = rest.find(end).expect("end marker missing");
    &rest[..end_ix]
}

#[test]
fn immediate_script_list_reconciliation_defers_preflight_to_final_state_callers() {
    let reconcile_body = section_between(
        FILTER_INPUT_UPDATES,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
        "pub(crate) fn queue_filter_compute(",
    );

    assert!(
        reconcile_body.contains("\"filter_immediate\"")
            && reconcile_body.contains("\"set_filter_text_immediate\""),
        "reconciliation must defer preflight for both immediate filter paths whose callers rebuild after final state"
    );
    assert!(
        !reconcile_body.contains("if reason != \"filter_immediate\""),
        "reconciliation must not only special-case filter_immediate; set_filter_text_immediate also owns the final preflight rebuild"
    );
    assert!(
        reconcile_body.contains("self.refresh_ghost_with_input(cx);"),
        "ghost refresh must stay in reconciliation even when preflight is deferred"
    );
}

#[test]
fn set_filter_text_immediate_rebuilds_preflight_once_after_fallback_updates() {
    let body = section_between(
        FILTER_INPUT_UPDATES,
        "pub(crate) fn set_filter_text_immediate(",
        "pub(crate) fn handle_script_list_printable_simulate_key(",
    );

    assert_eq!(
        body.matches("self.rebuild_main_window_preflight_if_needed();")
            .count(),
        1,
        "set_filter_text_immediate should own exactly one final preflight rebuild"
    );

    let reconcile_ix = body
        .find("self.reconcile_script_list_after_filter_change(\"set_filter_text_immediate\", cx);")
        .expect("set_filter_text_immediate reconciliation call missing");
    let fallback_ix = body
        .find("self.main_menu_fallback_state.replace_items(fallbacks);")
        .expect("fallback replacement block missing");
    let rebuild_ix = body
        .find("self.rebuild_main_window_preflight_if_needed();")
        .expect("final preflight rebuild missing");

    assert!(
        reconcile_ix < fallback_ix && fallback_ix < rebuild_ix,
        "final preflight rebuild must stay after reconciliation and fallback updates"
    );
}
