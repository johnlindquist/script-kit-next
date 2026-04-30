//! Source-level contract for the Run 9 Pass #14 Extend of
//! `actions-cmdk-builtin-browser-history` — adds a
//! `AppView::BrowserHistoryView` match arm to `collect_visible_elements`
//! in `src/app_layout/collect_elements.rs` so `getElements` against a
//! live BrowserHistory-hosted main window returns full
//! `input + list + row*` semantics instead of the
//! `panel:current-view` + `collector_used_current_view_fallback`
//! shape that Pass #10 documented for sibling BuiltinList views.
//!
//! Follows the Pass #11 BrowserTabs template exactly — same
//! `input-filter` + `list` naming shape, same fuzzy-search predicate
//! mirror against the renderer. BrowserHistory is the closest sibling
//! to BrowserTabs (runtime-fetched entries, `fuzzy_search_*` +
//! `display_title()` pipeline) so a refactor that consolidates the
//! tabs arm would naturally touch this one too; pinning both arms
//! independently prevents either side of that refactor from silently
//! re-opening the fallback.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn collect_visible_elements_has_browser_history_view_arm() {
    assert!(
        COLLECT_ELEMENTS.contains("AppView::BrowserHistoryView {"),
        "src/app_layout/collect_elements.rs must contain an \
         `AppView::BrowserHistoryView {{` match arm in \
         `collect_visible_elements`. Without it, `getElements` with \
         no target (or `target=Main`) under a BrowserHistory host \
         falls through to the `_ =>` catch-all and returns only \
         `panel:current-view` with `collector_used_current_view_fallback` \
         — the Pass #10 BuiltinList tool-gap shape."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn browser_history_arm_calls_collect_named_rows_with_browser_history_list_name() {
    assert!(
        COLLECT_ELEMENTS.contains("\"browser-history-filter\","),
        "BrowserHistoryView arm must pass `\"browser-history-filter\"` \
         as the input name to `collect_named_rows`. This keeps the \
         input semanticId stable across future edits so agentic \
         callers reading `focusedSemanticId` don't drift."
    );
    assert!(
        COLLECT_ELEMENTS.contains("\"browser-history\","),
        "BrowserHistoryView arm must pass `\"browser-history\"` as the \
         list name to `collect_named_rows`. This keeps the list \
         semanticId stable for `focusedSemanticId` / \
         `selectedSemanticId` receipts."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn browser_history_arm_uses_fuzzy_search_not_raw_contains() {
    // BrowserHistoryView uses `fuzzy_search_browser_history` for its
    // filter predicate (see `src/render_builtins/browser_history.rs:62`
    // and `src/prompt_handler/mod.rs:2382`). The collect arm MUST
    // match that predicate, NOT fall back to `.to_lowercase().contains()`
    // which would drift the collector's row count away from the
    // renderer's visible list and skew `visibleChoiceCount` receipts.
    let start = COLLECT_ELEMENTS
        .find("AppView::BrowserHistoryView {")
        .expect("BrowserHistoryView arm must exist (see sibling contract)");
    let end_rel = COLLECT_ELEMENTS[start..]
        .find("\n            AppView::")
        .or_else(|| COLLECT_ELEMENTS[start..].find("\n            _ =>"))
        .expect(
            "BrowserHistoryView arm must be followed by a sibling `AppView::` \
             variant or the `_ =>` catch-all — anchor end.",
        );
    let arm = &COLLECT_ELEMENTS[start..start + end_rel];
    assert!(
        arm.contains("fuzzy_search_browser_history"),
        "BrowserHistoryView arm must call \
         `crate::browser_history::fuzzy_search_browser_history` for its \
         non-empty-filter branch, mirroring the renderer at \
         `src/render_builtins/browser_history.rs:62` and the \
         `collect_state` arm at `src/prompt_handler/mod.rs:2382`. Any \
         other predicate would skew row count vs renderer. Arm body was:\n{}",
        arm
    );
    assert!(
        arm.contains("display_title()"),
        "BrowserHistoryView arm must derive row strings from \
         `entry.display_title()`, consistent with the renderer's \
         display. Any other field (e.g., `entry.url`) would render \
         different text in `getElements` vs the visible list. Arm \
         body was:\n{}",
        arm
    );
}
