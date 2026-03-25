//! Source-contract tests for the mini launcher's filter → reveal → resize pipeline.
//!
//! These tests read production source code and assert that critical mutation
//! sequences remain intact. They catch regressions where someone "cleans up"
//! filter handling and accidentally breaks the ordering contract.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"))
}

/// Extract the substring from `start` up to (but not including) `end`.
fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source
        .find(start)
        .unwrap_or_else(|| panic!("start marker not found: '{start}'"));
    let tail = &source[start_ix..];
    let end_rel = tail.find(end).unwrap_or(tail.len());
    &tail[..end_rel]
}

// ---------------------------------------------------------------------------
// 1) Shared reconciliation helper exists with the correct pipeline
// ---------------------------------------------------------------------------

#[test]
fn shared_script_list_reconciliation_helper_exists() {
    let source = read("src/app_impl/filter_input_updates.rs");
    assert!(
        source.contains("fn reconcile_script_list_after_filter_change"),
        "expected shared script-list filter reconciliation helper"
    );
    assert!(
        source.contains("self.scroll_to_selected_if_needed(reason);"),
        "shared reconciliation must reveal the final selected row"
    );
    assert!(
        source.contains("self.rebuild_main_window_preflight_if_needed();"),
        "shared reconciliation must rebuild preflight outside render"
    );
}

#[test]
fn reconciliation_helper_ordering_sync_before_select_before_reveal_before_preflight() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let body = section_between(
        &source,
        "fn reconcile_script_list_after_filter_change",
        "pub(crate) fn queue_filter_compute",
    );

    // Verify ordering: each step must appear after the previous one
    let ordered_markers = [
        "sync_list_state()",
        "selected_index = 0",
        "validate_selection_bounds",
        "scroll_to_selected_if_needed(reason)",
        "rebuild_main_window_preflight_if_needed()",
    ];

    let mut last_pos = 0usize;
    for marker in ordered_markers {
        let pos = body
            .find(marker)
            .unwrap_or_else(|| panic!("marker not found in reconciliation helper: '{marker}'"));
        assert!(
            pos >= last_pos,
            "ordering violation: '{marker}' appeared before the previous step in reconciliation helper"
        );
        last_pos = pos;
    }
}

// ---------------------------------------------------------------------------
// 1b) Reconciliation helper must clear scroll dedup guard before reveal
// ---------------------------------------------------------------------------

#[test]
fn reconciliation_helper_clears_last_scrolled_index_before_reveal() {
    // scroll_to_selected_if_needed has a dedup guard:
    //   if self.last_scrolled_index == Some(target) { return; }
    // After a filter change resets selected_index to 0, last_scrolled_index
    // could already be Some(0) from the previous cycle. Without clearing it,
    // the scroll would be silently skipped even though list content changed.
    let source = read("src/app_impl/filter_input_updates.rs");
    let body = section_between(
        &source,
        "fn reconcile_script_list_after_filter_change",
        "pub(crate) fn queue_filter_compute",
    );

    assert!(
        body.contains("self.last_scrolled_index = None;"),
        "reconciliation helper must clear last_scrolled_index to bypass \
         scroll_to_selected_if_needed's dedup guard after filter changes"
    );

    // The clear must happen BEFORE the reveal call
    let clear_pos = body
        .find("self.last_scrolled_index = None;")
        .expect("last_scrolled_index = None not found");
    let reveal_pos = body
        .find("self.scroll_to_selected_if_needed(reason);")
        .expect("scroll_to_selected_if_needed not found");
    assert!(
        clear_pos < reveal_pos,
        "last_scrolled_index must be cleared BEFORE scroll_to_selected_if_needed \
         to ensure the dedup guard doesn't skip the reveal"
    );
}

// Also verify scroll_to_selected_if_needed actually HAS the dedup guard
// (if someone removes it, clearing last_scrolled_index becomes dead code)
#[test]
fn scroll_helper_has_dedup_guard_on_last_scrolled_index() {
    let source = read("src/app_navigation/impl_scroll.rs");
    let body = section_between(
        &source,
        "fn scroll_to_selected_if_needed",
        "fn trigger_scroll_activity",
    );

    assert!(
        body.contains("if self.last_scrolled_index == Some(target)"),
        "scroll_to_selected_if_needed must have the dedup guard that \
         reconciliation_helper_clears_last_scrolled_index_before_reveal depends on"
    );
}

// ---------------------------------------------------------------------------
// 2) Both callsites delegate to the shared helper
// ---------------------------------------------------------------------------

#[test]
fn queue_filter_compute_uses_shared_reconciliation_helper() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let start = source
        .find("pub(crate) fn queue_filter_compute")
        .expect("queue_filter_compute not found");
    let section = &source[start..(start + 2600).min(source.len())];

    assert!(
        section
            .contains("app.reconcile_script_list_after_filter_change(\"filter_coalesced\", cx);"),
        "queue_filter_compute should use the shared reconciliation helper"
    );
    assert!(
        !section.contains("scroll_to_reveal_item(app.selected_index)"),
        "queue_filter_compute should not bypass scroll_to_selected_if_needed"
    );
}

#[test]
fn set_filter_text_immediate_uses_shared_reconciliation_helper() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let start = source
        .find("pub(crate) fn set_filter_text_immediate")
        .expect("set_filter_text_immediate not found");
    let section = &source[start..(start + 2600).min(source.len())];

    assert!(
        section.contains(
            "self.reconcile_script_list_after_filter_change(\"set_filter_text_immediate\", cx);"
        ),
        "set_filter_text_immediate should use the shared reconciliation helper"
    );
}

#[test]
fn queue_filter_compute_still_resizes_and_notifies() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let body = section_between(
        &source,
        "pub(crate) fn queue_filter_compute",
        "pub(crate) fn set_filter_text_immediate",
    );

    assert!(
        body.contains("update_window_size()"),
        "queue_filter_compute must trigger window resize after reconciliation"
    );
    assert!(
        body.contains("cx.notify()"),
        "queue_filter_compute must notify after mutations"
    );
}

// ---------------------------------------------------------------------------
// 2) sync_list_state must always invalidate reveal cache and restore reveal
// ---------------------------------------------------------------------------

#[test]
fn sync_list_state_always_invalidates_reveal_cache() {
    let source = read("src/app_navigation/impl_scroll.rs");
    let body = section_between(
        &source,
        "pub fn sync_list_state(&mut self)",
        "pub fn validate_selection_bounds",
    );

    // Must unconditionally clear last_scrolled_index (outside any if-block
    // that checks item_count equality)
    assert!(
        body.contains("self.last_scrolled_index = None;"),
        "sync_list_state must always clear last_scrolled_index so same-count \
         filter replacements can still re-reveal the selected row"
    );

    // The cache-clear must be OUTSIDE the `if old_list_count != item_count` block.
    // Verify by checking it appears after the closing brace of that block.
    let splice_block_end = body
        .find("self.main_list_state.splice(")
        .and_then(|pos| body[pos..].find('}').map(|rel| pos + rel + 1))
        .expect("splice block not found");

    let invalidation_pos = body
        .find("self.last_scrolled_index = None;")
        .expect("invalidation not found");

    assert!(
        invalidation_pos > splice_block_end,
        "last_scrolled_index = None must be OUTSIDE the item_count-changed branch \
         so it fires even when the count stays the same"
    );
}

#[test]
fn sync_list_state_restores_reveal_for_current_selection() {
    let source = read("src/app_navigation/impl_scroll.rs");
    let body = section_between(
        &source,
        "pub fn sync_list_state(&mut self)",
        "pub fn validate_selection_bounds",
    );

    let has_reveal = body.contains("scroll_to_reveal_item(self.selected_index)");

    assert!(
        has_reveal,
        "sync_list_state must restore reveal for the current selected_index \
         after syncing item_count"
    );
}
