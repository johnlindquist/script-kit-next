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
//! Pass #37 narrows the `rows` Vec in the `ClipboardHistoryView` arm of
//! `collect_visible_elements` (src/app_layout/collect_elements.rs) by
//! the destructured `filter` field before passing into
//! `collect_named_rows`. Semantics mirror the `collect_state` arm
//! (src/prompt_handler/mod.rs): case-insensitive `contains` on
//! `text_preview.to_lowercase()`. Live receipt after the fix:
//!
//!   - setFilter ""                   → list:"100 items"
//!   - setFilter "pnpm"               → list:"1 items"
//!   - setFilter "zzz_no_match_p37"   → list:"0 items", zero choice:* elements
//!
//! This contract pins the arm shape so a future refactor cannot
//! silently regress `getElements` back to emitting the full dataset
//! while the filter narrows only the `getState` receipt.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
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
        body.contains("if filter.is_empty() {"),
        "arm must fast-path `filter.is_empty()` so an unfiltered \
         view returns the full dataset without paying for the \
         lowercase/contains work."
    );
    assert!(
        body.contains("let filter_lower = filter.to_lowercase();"),
        "arm must lowercase the destructured `filter` before matching \
         (`let filter_lower = filter.to_lowercase();`). Using \
         `self.filter_text` would reintroduce Pass #14 sub-gap 2; a \
         case-sensitive match would drift from `handle_filter_input_change`."
    );
    assert!(
        body.contains("e.text_preview.to_lowercase().contains(&filter_lower)"),
        "arm must filter via `e.text_preview.to_lowercase().contains(&filter_lower)` \
         — matching on `full_text` or `app_name` here would drift the \
         visible-elements contract away from what the UI renders and \
         what the `collect_state` arm (src/prompt_handler/mod.rs) uses."
    );
    assert!(
        body.contains("self.cached_clipboard_entries"),
        "arm must source rows from `self.cached_clipboard_entries` — \
         this is the single shared dataset feeding both the renderer \
         and the `collect_state` arm."
    );
    assert!(
        body.contains("\"clipboard-filter\"") && body.contains("\"clipboard-history\""),
        "arm must still call `collect_named_rows` with the \
         `clipboard-filter` and `clipboard-history` semantic ids — \
         changing these would break every existing agentic-testing \
         story keyed on those ids."
    );
}
