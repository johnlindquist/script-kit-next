//! Source-level contract for the Run 9 Pass #11 Extend of
//! `actions-cmdk-builtin-browser-tabs` — adds a
//! `AppView::BrowserTabsView` match arm to `collect_visible_elements`
//! in `src/app_layout/collect_elements.rs` so `getElements` against a
//! live BrowserTabs-hosted main window returns full
//! `input + list + row*` semantics instead of the
//! `panel:current-view` + `collector_used_current_view_fallback`
//! shape observed in Run 9 Pass #10.
//!
//! Pre-fix: four of the 13 `BuiltinList` host views
//! (`src/app_impl/actions_dialog.rs:31-43`) had collect arms
//! (`WindowSwitcherView`, `ProcessManagerView`, `BrowseKitsView`,
//! `ThemeChooserView`) — the remaining 9 (including
//! `BrowserTabsView`) fell through to the `_ =>` catch-all,
//! yielding only `panel:current-view`. This contract pins the
//! BrowserTabsView arm specifically so a refactor that consolidates
//! the `BuiltinList` arms (or removes this one while refactoring
//! a sibling) cannot silently re-open the fallback. Sibling views
//! get their own contracts in later passes as each is exercised
//! live.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

#[test]
fn collect_visible_elements_has_browser_tabs_view_arm() {
    assert!(
        COLLECT_ELEMENTS.contains("AppView::BrowserTabsView {"),
        "src/app_layout/collect_elements.rs must contain an \
         `AppView::BrowserTabsView {{` match arm in \
         `collect_visible_elements`. Without it, `getElements` with \
         no target (or `target=Main`) under a BrowserTabs host falls \
         through to the `_ =>` catch-all and returns only \
         `panel:current-view` with `collector_used_current_view_fallback` \
         — the Run 9 Pass #10 tool-gap shape."
    );
}

#[test]
fn browser_tabs_arm_calls_collect_named_rows_with_browser_tabs_list_name() {
    assert!(
        COLLECT_ELEMENTS.contains("\"browser-tabs-filter\","),
        "BrowserTabsView arm must pass `\"browser-tabs-filter\"` as the \
         input name to `collect_named_rows`. This keeps the input \
         semanticId stable across future edits so agentic callers \
         reading `focusedSemanticId` don't drift."
    );
    assert!(
        COLLECT_ELEMENTS.contains("\"browser-tabs\","),
        "BrowserTabsView arm must pass `\"browser-tabs\"` as the \
         list name to `collect_named_rows`. This keeps the list \
         semanticId stable for `focusedSemanticId` / \
         `selectedSemanticId` receipts."
    );
}

#[test]
fn browser_tabs_arm_uses_fuzzy_search_not_raw_contains() {
    // BrowserTabsView filters with `fuzzy_search_browser_tabs` and labels
    // rows with `display_title()`, both inside the renderer-owned
    // `browser_tabs_visible_row_labels` helper. The collect arm MUST use
    // that helper, NOT its own `.to_lowercase().contains()` predicate,
    // which would drift the collector's row count away from the
    // renderer's visible list and skew `visibleChoiceCount` receipts.
    let start = COLLECT_ELEMENTS
        .find("AppView::BrowserTabsView {")
        .expect("BrowserTabsView arm must exist (see sibling contract)");
    let end_rel = COLLECT_ELEMENTS[start..]
        .find("\n            AppView::")
        .or_else(|| COLLECT_ELEMENTS[start..].find("\n            _ =>"))
        .expect(
            "BrowserTabsView arm must be followed by a sibling `AppView::` \
             variant or the `_ =>` catch-all — anchor end.",
        );
    let arm = &COLLECT_ELEMENTS[start..start + end_rel];
    assert!(
        arm.contains("self.browser_tabs_visible_row_labels(filter)"),
        "BrowserTabsView arm must take its rows from the renderer's \
         `browser_tabs_visible_row_labels` helper, which filters with \
         `crate::browser_tabs::fuzzy_search_browser_tabs` and labels rows \
         with `tab.display_title()`. Any other predicate or field would \
         skew row count or text vs the renderer. Arm body was:\n{}",
        arm
    );
    assert!(
        !arm.contains(".to_lowercase().contains("),
        "BrowserTabsView arm must not filter with a raw substring match. \
         Arm body was:\n{}",
        arm
    );
}
