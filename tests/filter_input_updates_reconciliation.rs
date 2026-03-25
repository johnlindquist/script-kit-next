//! Source-level regression tests that lock the shared reconciliation path
//! in `src/app_impl/filter_input_updates.rs`.
//!
//! These tests read production source and assert that the single authoritative
//! post-filter reconciliation helper exists and that both mutation sites
//! (`queue_filter_compute` and `set_filter_text_immediate`) delegate to it
//! instead of inlining bespoke state sequences.

use std::fs;

fn read_source() -> String {
    fs::read_to_string("src/app_impl/filter_input_updates.rs")
        .expect("Failed to read src/app_impl/filter_input_updates.rs")
}

/// Extract a bounded slice of `source` starting at the first occurrence of
/// `start_marker`, extending up to `max_len` characters (clamped to EOF).
fn slice_from(source: &str, start_marker: &str, max_len: usize) -> String {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("marker not found: '{start_marker}'"));
    let end = (start + max_len).min(source.len());
    source[start..end].to_string()
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
fn reconcile_helper_calls_scroll_to_selected_if_needed() {
    let source = read_source();
    let section = slice_from(
        &source,
        "fn reconcile_script_list_after_filter_change",
        2000,
    );
    assert!(
        section.contains("scroll_to_selected_if_needed(reason)"),
        "shared reconciliation must reveal the final selected row via scroll_to_selected_if_needed"
    );
}

#[test]
fn reconcile_helper_calls_rebuild_preflight() {
    let source = read_source();
    let section = slice_from(
        &source,
        "fn reconcile_script_list_after_filter_change",
        2000,
    );
    assert!(
        section.contains("rebuild_main_window_preflight_if_needed();"),
        "shared reconciliation must rebuild preflight outside render"
    );
}

// ---------------------------------------------------------------------------
// 2) queue_filter_compute delegates to the shared helper
// ---------------------------------------------------------------------------

#[test]
fn queue_filter_compute_uses_shared_helper() {
    let source = read_source();
    let section = slice_from(&source, "pub(crate) fn queue_filter_compute", 2600);
    assert!(
        section
            .contains("app.reconcile_script_list_after_filter_change(\"filter_coalesced\", cx);"),
        "queue_filter_compute must delegate to reconcile_script_list_after_filter_change"
    );
}

#[test]
fn queue_filter_compute_does_not_bypass_scroll_helper() {
    let source = read_source();
    let section = slice_from(&source, "pub(crate) fn queue_filter_compute", 2600);
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
    let section = slice_from(&source, "pub(crate) fn set_filter_text_immediate", 2600);
    assert!(
        section.contains(
            "self.reconcile_script_list_after_filter_change(\"set_filter_text_immediate\", cx);"
        ),
        "set_filter_text_immediate must delegate to reconcile_script_list_after_filter_change"
    );
}
