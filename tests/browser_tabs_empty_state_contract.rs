const BROWSER_TABS: &str = include_str!("../src/render_builtins/browser_tabs.rs");

#[test]
fn browser_tabs_empty_state_copy_is_modeled() {
    assert!(
        BROWSER_TABS.contains("enum BrowserTabsEmptyState")
            && BROWSER_TABS.contains("NoOpenTabs")
            && BROWSER_TABS.contains("NoFilteredMatches"),
        "Browser Tabs empty-state copy should use named states"
    );
    assert!(
        BROWSER_TABS.contains("fn from_filter(filter: &str) -> Self")
            && BROWSER_TABS.contains("fn message(self) -> &'static str"),
        "Browser Tabs empty states should own filter classification and visible copy"
    );
    assert!(
        BROWSER_TABS.contains("BrowserTabsEmptyState::from_filter")
            && (BROWSER_TABS.contains(".message()") || BROWSER_TABS.contains("state.message()")),
        "Browser Tabs renderer should derive empty-state copy from the model"
    );
    assert!(
        !BROWSER_TABS.contains("child(if filter.is_empty()"),
        "Browser Tabs empty-state copy must not regress to inline filter-empty branching"
    );
}

#[test]
fn browser_tabs_count_label_copy_is_modeled() {
    assert!(
        BROWSER_TABS.contains("fn browser_tabs_count_label("),
        "Browser Tabs header count copy should live in a named helper"
    );
    assert!(
        BROWSER_TABS.contains("let suffix = if total_count == 1 { \"\" } else { \"s\" };")
            && BROWSER_TABS.contains("format!(\"{} tab{}\", total_count, suffix)"),
        "Browser Tabs count helper should avoid '1 tabs'"
    );
    assert!(
        BROWSER_TABS.contains("Self::browser_tabs_count_label")
            && BROWSER_TABS.contains("total_count"),
        "Browser Tabs renderer should use the count label helper"
    );
}
