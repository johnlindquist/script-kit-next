//! Source-level contract for the Run 2 Pass #37
//! `clipboard-history-getelements-filter-aware` user story.
//!
//! Pass #14 recorded `empty-clipboard-state [!]` with four sub-gaps.
//! Passes #33/#34/#36 closed sub-gaps 2+3 (the `getState` / `setFilter`
//! stdin path). Sub-gap 4 — `getElements` on `ClipboardHistoryView`
//! emitting `choice:*` elements that do NOT respect the variant's
//! `filter` field — remained live-broken even after those passes:
//! `setFilter "zzz_no_match_p37"` left `list:"100 items"` and eight
//! populated `choice:N:*` elements.
//!
//! Pass #37 originally narrowed the `rows` Vec in the `ClipboardHistoryView`
//! arm of `collect_visible_elements` (src/app_layout/collect_elements.rs).
//! That behavior now lives in the shared
//! `clipboard_history_visible_rows_for_entries` helper so rendering,
//! `getState`, and `getElements` cannot drift apart. Text semantics still
//! include case-insensitive `contains` on `text_preview.to_lowercase()`.
//! Live receipt after the fix:
//!
//!   - setFilter ""                   → list:"100 items"
//!   - setFilter "pnpm"               → list:"1 items"
//!   - setFilter "zzz_no_match_p37"   → list:"0 items", zero choice:* elements
//!
//! This contract pins the shared-helper shape so a future refactor cannot
//! silently regress `getElements` back to emitting the full dataset while
//! the filter narrows only the renderer or `getState` receipt.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const CLIPBOARD: &str = include_str!("../src/render_builtins/clipboard.rs");

// doc-anchor-removed: [[removed-docs and introspection]]
#[test]
fn clipboard_history_collect_elements_narrows_rows_by_variant_filter() {
    let arm_pos = COLLECT_ELEMENTS
        .find("AppView::ClipboardHistoryView {\n                filter,")
        .expect(
            "src/app_layout/collect_elements.rs must contain a \
             `ClipboardHistoryView` arm destructuring `filter,` on the \
             line after the opening brace. Any other shape (binding via \
             `..` or reading `self.filter_text`) would reintroduce the \
             Pass #14 sub-gap 4: `getElements` would drift off the \
             variant filter and silently emit the full dataset.",
        );
    let body_end_rel = COLLECT_ELEMENTS[arm_pos..]
        .find("\n            AppView::AppLauncherView")
        .expect(
            "`ClipboardHistoryView` arm must be immediately followed by \
             `AppView::AppLauncherView` — if that ordering changes, \
             amend this contract.",
        );
    let body = &COLLECT_ELEMENTS[arm_pos..arm_pos + body_end_rel];

    assert!(
        body.contains("let rows = self.clipboard_history_visible_row_labels(filter);"),
        "getElements must derive clipboard rows from \
         `clipboard_history_visible_row_labels(filter)` so it shares the \
         same visible-row owner as render and getState."
    );
    assert!(
        body.contains("\"clipboard-filter\"") && body.contains("\"clipboard-history\""),
        "arm must still call `collect_named_rows` with the \
         `clipboard-filter` and `clipboard-history` semantic ids — \
         changing these would break every existing agentic-testing \
         story keyed on those ids."
    );

    let helper_pos = CLIPBOARD
        .find("pub(crate) fn clipboard_history_visible_rows_for_entries")
        .expect("clipboard history must expose the shared visible-row helper");
    let helper_end_rel = CLIPBOARD[helper_pos..]
        .find("\n    fn clipboard_history_visible_rows(")
        .expect("visible-row helper must be followed by its instance wrapper");
    let helper = &CLIPBOARD[helper_pos..helper_pos + helper_end_rel];

    assert!(
        helper.contains("let query = ClipboardHistoryFilterQuery::parse(filter);"),
        "shared helper must parse the destructured variant filter, not \
         `self.filter_text`, before deriving visible rows."
    );
    assert!(
        helper.contains("if query.is_empty() {")
            && helper.contains("entries.iter().cloned().enumerate().collect()"),
        "shared helper must fast-path an empty query so an unfiltered \
         view returns the full dataset without per-entry match work."
    );
    assert!(
        helper.contains(".filter(|(_, entry)| query.matches(entry))"),
        "shared helper must filter rows with `query.matches(entry)` so \
         getElements, getState, and rendering use the same filter logic."
    );
    assert!(
        CLIPBOARD.contains("entry.text_preview.to_lowercase().contains(filter_lower)"),
        "clipboard filter matching must still include case-insensitive \
         `text_preview` contains semantics for plain text filters."
    );
}
