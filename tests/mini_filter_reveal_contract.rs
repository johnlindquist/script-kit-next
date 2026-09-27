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

/// Extract a function from its signature through its closing brace.
fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("function not found: '{signature}'"));
    let tail = &source[start..];
    let open = tail.find('{').expect("function should have a body");
    let mut depth = 0usize;
    for (offset, ch) in tail[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &tail[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated body for '{signature}'")
}

fn position(body: &str, marker: &str, owner: &str) -> usize {
    body.find(marker)
        .unwrap_or_else(|| panic!("marker not found in {owner}: '{marker}'"))
}

// ---------------------------------------------------------------------------
// 1) Shared reconciliation helper exists with the correct pipeline
// ---------------------------------------------------------------------------

#[test]
fn shared_script_list_reconciliation_helper_exists() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let helper = fn_body(
        &source,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
    );
    assert!(
        helper.contains(
            "self.sync_list_state_for_filter_replacement(MainListReplacementPolicy::ResetToTop);"
        ),
        "shared reconciliation must reveal the final selected row by resetting the replaced list to its top"
    );
    let publication = fn_body(&source, "fn finish_main_menu_publication(");
    assert!(
        helper.contains("self.finish_main_menu_publication(reason, None, cx)")
            && publication.contains("self.rebuild_main_window_preflight_if_needed();"),
        "shared reconciliation must rebuild preflight outside render"
    );
}

#[test]
fn reconciliation_helper_orders_rows_then_selection_then_viewport_then_preflight() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let body = fn_body(
        &source,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
    );

    // Verify ordering: each step must appear after the previous one
    let ordered_markers = [
        "self.rebuild_main_menu_results_cache(cx)",
        "self.reset_main_menu_selection_intent();",
        "self.reconcile_main_menu_selection_intent();",
        "self.sync_list_state_for_filter_replacement(",
        "self.finish_main_menu_publication(reason, None, cx)",
    ];

    let mut last_pos = 0usize;
    for marker in ordered_markers {
        let pos = position(body, marker, "reconciliation helper");
        assert!(
            pos >= last_pos,
            "ordering violation: '{marker}' appeared before the previous step in reconciliation helper"
        );
        last_pos = pos;
    }
}

// ---------------------------------------------------------------------------
// 1b) List replacement must clear scroll dedup guard before reveal
// ---------------------------------------------------------------------------

#[test]
fn filter_replacement_clears_last_scrolled_index_before_reveal() {
    // scroll_to_selected_if_needed has a dedup guard:
    //   if self.last_scrolled_index == Some(target) { return; }
    // After a filter change resets the selection to the first row,
    // last_scrolled_index could already be Some(0) from the previous cycle.
    // Without clearing it, a later reveal would be silently skipped even
    // though list content changed.
    let source = read("src/app_navigation/impl_scroll.rs");
    let body = fn_body(&source, "pub fn sync_list_state_for_filter_replacement(");

    let clear_pos = position(
        body,
        "self.last_scrolled_index = None;",
        "filter replacement",
    );
    let reveal_pos = position(
        body,
        "self.main_list_state.scroll_to(restored_offset);",
        "filter replacement",
    );
    assert!(
        clear_pos < reveal_pos,
        "last_scrolled_index must be cleared BEFORE the replaced list scrolls \
         to its restored offset so the dedup guard doesn't skip a later reveal"
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
         filter_replacement_clears_last_scrolled_index_before_reveal depends on"
    );
}

// ---------------------------------------------------------------------------
// 2) Both callsites delegate to the shared helper
// ---------------------------------------------------------------------------

#[test]
fn queue_filter_compute_uses_shared_reconciliation_helper() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let queue = fn_body(&source, "pub(crate) fn queue_filter_compute(");
    let apply = fn_body(&source, "fn apply_filter_compute_now(");

    assert!(
        queue.contains("self.apply_filter_compute_now(value, cx);")
            && queue.contains("app.apply_filter_compute_now(latest, cx);")
            && apply
                .contains("self.reconcile_script_list_after_filter_change(\"query_commit\", cx);"),
        "queue_filter_compute should use the shared reconciliation helper"
    );
    assert!(
        !queue.contains("scroll_to_reveal_item(") && !apply.contains("scroll_to_reveal_item("),
        "queue_filter_compute should not bypass the shared list replacement"
    );
}

#[test]
fn set_filter_text_immediate_uses_shared_reconciliation_helper() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let immediate = fn_body(&source, "pub(crate) fn set_filter_text_immediate(");
    let with_cursor = fn_body(
        &source,
        "pub(crate) fn set_filter_text_and_cursor_immediate(",
    );

    assert!(
        immediate.contains("self.set_filter_text_and_cursor_immediate(text, cursor, window, cx);")
            && with_cursor.contains("self.apply_filter_compute_now(text.clone(), cx);"),
        "set_filter_text_immediate should use the shared reconciliation helper"
    );
}

#[test]
fn queue_filter_compute_still_resizes_and_notifies() {
    let source = read("src/app_impl/filter_input_updates.rs");
    let apply = fn_body(&source, "fn apply_filter_compute_now(");
    let helper = fn_body(
        &source,
        "pub(crate) fn reconcile_script_list_after_filter_change(",
    );

    let reconcile_pos = position(
        apply,
        "self.reconcile_script_list_after_filter_change(\"query_commit\", cx);",
        "apply_filter_compute_now",
    );
    assert!(
        apply[reconcile_pos..].contains("self.update_window_size();"),
        "queue_filter_compute must trigger window resize after reconciliation"
    );
    assert!(
        helper.contains("cx.notify_with_owned_cause(\"mainSearchPublication\", sequence);"),
        "queue_filter_compute must notify after mutations"
    );
}

// ---------------------------------------------------------------------------
// 3) sync_list_state must always invalidate reveal cache
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
