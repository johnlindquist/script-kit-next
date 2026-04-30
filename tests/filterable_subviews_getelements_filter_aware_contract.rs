//! Source-level contract for the Run 2 Pass #38
//! `filterable-subviews-getelements-filter-aware` user story.
//!
//! Pass #37 fixed the `getElements` filter-drift on `ClipboardHistoryView`.
//! Pass #38 extends the same fix to the four sibling subviews that shared
//! the bug — `AppLauncherView`, `WindowSwitcherView`, `ProcessManagerView`,
//! `CurrentAppCommandsView` — each of which mapped its cached dataset to
//! rows without honoring the destructured `filter` field, so
//! `getElements` streamed the full dataset with populated `choice:*`
//! elements while `getState.visibleChoiceCount` correctly narrowed.
//!
//! AURP-05 tightened the CurrentAppCommands arm: it must reuse the same
//! named visible-row helper as the renderer and `getState` path, because
//! menu command queries also match descriptions and keywords.
//!
//! AURP-18 applies that helper-owner shape to AppLauncherView and
//! ProcessManagerView so renderer rows, getState counts, getElements rows, and
//! Tab AI targets all share the same row projection.
//!
//! Live-verified on apps-launcher after rebuild:
//!   - setFilter "zzz_no_app_p38" → list:"0 items", totalCount=2
//!   - setFilter "safari"         → list:"1 items", totalCount=3
//!
//! This contract pins every arm's narrowing shape so a future refactor
//! can't silently regress any of the four surfaces.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");

fn arm_body<'a>(start_pat: &str, end_pat: &str) -> &'a str {
    let start = COLLECT_ELEMENTS
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing arm start: {start_pat}"));
    let end_rel = COLLECT_ELEMENTS[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing arm end: {end_pat}"));
    &COLLECT_ELEMENTS[start..start + end_rel]
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn app_launcher_arm_narrows_by_variant_filter() {
    let body = arm_body(
        "AppView::AppLauncherView {\n                filter,",
        "\n            AppView::WindowSwitcherView",
    );
    assert!(
        body.contains("let rows = self.app_launcher_visible_row_names(filter);"),
        "AppLauncherView arm must ask the shared visible-row helper for rows."
    );
    assert!(
        !body.contains("app.name.to_lowercase().contains(&filter_lower)"),
        "AppLauncherView collect-elements must not bypass the named visible-row helper."
    );
    assert!(
        !body.contains("let filter_lower = filter.to_lowercase();"),
        "AppLauncherView collect-elements must not own raw filter normalization."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn window_switcher_arm_narrows_by_variant_filter() {
    let body = arm_body(
        "AppView::WindowSwitcherView {\n                filter,",
        "\n            AppView::FileSearchView",
    );
    assert!(
        body.contains("if filter.is_empty() {"),
        "WindowSwitcherView arm must fast-path empty filter."
    );
    assert!(
        body.contains("let filter_lower = filter.to_lowercase();"),
        "WindowSwitcherView arm must lowercase the destructured `filter`."
    );
    assert!(
        body.contains(".filter(|row| row.to_lowercase().contains(&filter_lower))"),
        "WindowSwitcherView arm must narrow the formatted `app — title` row by \
         `row.to_lowercase().contains(&filter_lower)` so the filter matches \
         against the same string the user sees."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn process_manager_arm_narrows_by_variant_filter() {
    let body = arm_body(
        "AppView::ProcessManagerView {\n                filter,",
        "\n            AppView::CurrentAppCommandsView",
    );
    assert!(
        body.contains("let rows = self.process_manager_visible_row_names(filter);"),
        "ProcessManagerView arm must ask the shared visible-row helper for rows."
    );
    assert!(
        !body.contains("let filter_lower = filter.to_lowercase();"),
        "ProcessManagerView collect-elements must not own raw filter normalization."
    );
    assert!(
        !body.contains("p.script_path.to_lowercase().contains(&filter_lower)"),
        "ProcessManagerView collect-elements must not bypass the named visible-row helper."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn current_app_commands_arm_narrows_by_variant_filter() {
    let body = arm_body(
        "AppView::CurrentAppCommandsView {\n                filter,",
        "\n            AppView::EmojiPickerView",
    );
    assert!(
        body.contains("let rows = self.current_app_commands_visible_row_names(filter);"),
        "CurrentAppCommandsView arm must ask the shared visible-row helper for rows."
    );
    assert!(
        !body.contains("filter_menu_bar_entries("),
        "CurrentAppCommandsView collect-elements must not call the raw filter directly; \
         renderer, getState, and getElements must share the named helper."
    );
    assert!(
        !body.contains("e.name.to_lowercase().contains(&filter_lower)"),
        "CurrentAppCommandsView arm must not regress to name-only matching; \
         menu command filters also match descriptions and keywords."
    );
}
