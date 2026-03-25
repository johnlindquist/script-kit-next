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
// 1) queue_filter_compute must perform the full post-filter pipeline
// ---------------------------------------------------------------------------

#[test]
fn queue_filter_compute_performs_full_post_filter_pipeline() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let body = section_between(
        &source,
        "pub(crate) fn queue_filter_compute",
        "pub(crate) fn set_filter_text_immediate",
    );

    let required_steps: &[(&str, &str)] = &[
        (
            "sync_list_state()",
            "queue_filter_compute must sync list state after recompute",
        ),
        (
            "selected_index = 0",
            "queue_filter_compute must reset selection to zero",
        ),
        (
            "validate_selection_bounds(cx)",
            "queue_filter_compute must validate selection bounds",
        ),
        (
            "scroll_to_reveal_item(app.selected_index)",
            "queue_filter_compute must reveal the selected item",
        ),
        (
            "last_scrolled_index = Some(app.selected_index)",
            "queue_filter_compute must update scroll tracking",
        ),
        (
            "rebuild_main_window_preflight_if_needed()",
            "queue_filter_compute must rebuild preflight",
        ),
        (
            "update_window_size()",
            "queue_filter_compute must trigger window resize",
        ),
        (
            "cx.notify()",
            "queue_filter_compute must notify after mutations",
        ),
    ];

    for (needle, msg) in required_steps {
        assert!(body.contains(needle), "{msg}");
    }
}

#[test]
fn queue_filter_compute_ordering_sync_before_select_before_reveal_before_resize() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let body = section_between(
        &source,
        "pub(crate) fn queue_filter_compute",
        "pub(crate) fn set_filter_text_immediate",
    );

    // Verify ordering: each step must appear after the previous one
    let ordered_markers = [
        "sync_list_state()",
        "selected_index = 0",
        "validate_selection_bounds",
        "scroll_to_reveal_item",
        "last_scrolled_index = Some",
        "rebuild_main_window_preflight_if_needed()",
        "update_window_size()",
        "cx.notify()",
    ];

    let mut last_pos = 0usize;
    for marker in ordered_markers {
        let pos = body
            .find(marker)
            .unwrap_or_else(|| panic!("marker not found in queue_filter_compute: '{marker}'"));
        assert!(
            pos >= last_pos,
            "ordering violation: '{marker}' appeared before the previous step in queue_filter_compute"
        );
        last_pos = pos;
    }
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

    let has_reveal =
        body.contains("scroll_to_reveal_item(self.selected_index)");

    assert!(
        has_reveal,
        "sync_list_state must restore reveal for the current selected_index \
         after syncing item_count"
    );
}
