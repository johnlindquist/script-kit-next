//! Source-level regression tests that lock the shared reconciliation path
//! in `src/app_impl/filter_input_updates.rs`.
//!
//! These tests read production source and assert that the single authoritative
//! post-filter reconciliation helper exists and that the query-commit path
//! (`apply_filter_compute_now`, reached from both `queue_filter_compute` and
//! `set_filter_text_immediate`) delegates to it instead of inlining bespoke
//! state sequences.

use std::fs;

fn read_source() -> String {
    fs::read_to_string("src/app_impl/filter_input_updates.rs")
        .expect("Failed to read src/app_impl/filter_input_updates.rs")
}

/// Extract the full Rust function body that starts at `start_marker`.
fn function_body(source: &str, start_marker: &str) -> String {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("marker not found: '{start_marker}'"));
    let body_start = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("body start not found for: '{start_marker}'"));

    let mut depth = 0usize;
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return source[start..=body_start + offset].to_string();
                }
            }
            _ => {}
        }
    }

    panic!("body end not found for: '{start_marker}'");
}

fn assert_in_order(body: &str, needles: &[&str], message: &str) {
    let mut from = 0;
    for needle in needles {
        let at = body[from..]
            .find(needle)
            .unwrap_or_else(|| panic!("{message}: missing `{needle}` after byte {from}"));
        from += at + needle.len();
    }
}

// ---------------------------------------------------------------------------
// 1) Shared reconciliation helper exists with required call sites
// ---------------------------------------------------------------------------

#[test]
fn reconcile_helper_exists() {
    let source = read_source();
    assert!(
        source.contains("fn reconcile_script_list_after_filter_change"),
        "expected shared reconciliation helper to exist"
    );
}

#[test]
fn reconcile_helper_reveals_the_first_row_of_the_new_query() {
    let source = read_source();
    let section = function_body(
        &source,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
    );
    assert_in_order(
        &section,
        &[
            "self.reset_main_menu_selection_intent();",
            "self.reconcile_main_menu_selection_intent();",
            "self.sync_list_state_for_filter_replacement(MainListReplacementPolicy::ResetToTop);",
        ],
        "shared reconciliation must select against the new rows, then reset the viewport to the top",
    );
}

#[test]
fn reconcile_helper_calls_rebuild_preflight() {
    let source = read_source();
    let section = function_body(
        &source,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
    );
    assert!(
        section.contains("self.finish_main_menu_publication(reason, None, cx);"),
        "shared reconciliation must publish through finish_main_menu_publication"
    );
    let publication = function_body(&source, "fn finish_main_menu_publication(");
    assert!(
        publication.contains("self.rebuild_main_window_preflight_if_needed();"),
        "shared reconciliation must rebuild preflight outside render"
    );
}

// ---------------------------------------------------------------------------
// 2) filter compute delegates to the shared helper
// ---------------------------------------------------------------------------

#[test]
fn apply_filter_compute_now_uses_shared_helper() {
    let source = read_source();
    let section = function_body(&source, "fn apply_filter_compute_now");
    assert!(
        section.contains("self.reconcile_script_list_after_filter_change(\"query_commit\", cx);"),
        "filter compute must delegate to reconcile_script_list_after_filter_change"
    );
}

#[test]
fn queue_filter_compute_does_not_bypass_scroll_helper() {
    let source = read_source();
    let section = function_body(&source, "pub(crate) fn queue_filter_compute");
    assert!(
        !section.contains("scroll_to_reveal_item(app.selected_index)"),
        "queue_filter_compute must not bypass scroll_to_selected_if_needed with a direct scroll_to_reveal_item call"
    );
}

// ---------------------------------------------------------------------------
// 3) set_filter_text_immediate delegates to the shared helper
// ---------------------------------------------------------------------------

#[test]
fn set_filter_text_immediate_uses_shared_helper() {
    let source = read_source();
    let section = function_body(&source, "pub(crate) fn set_filter_text_immediate(");
    assert!(
        section.contains("self.set_filter_text_and_cursor_immediate(text, cursor, window, cx);"),
        "set_filter_text_immediate must delegate to the cursor-aware owner"
    );
    let owner = function_body(
        &source,
        "pub(crate) fn set_filter_text_and_cursor_immediate(",
    );
    assert!(
        owner.contains("self.apply_filter_compute_now(text.clone(), cx);"),
        "set_filter_text_immediate must commit through apply_filter_compute_now, which delegates to reconcile_script_list_after_filter_change"
    );
}
