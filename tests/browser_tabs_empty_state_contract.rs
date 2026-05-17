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
        BROWSER_TABS.contains("BrowserTabsEmptyState::from_filter(&filter).message()"),
        "Browser Tabs renderer should derive empty-state copy from the model"
    );
    assert!(
        !BROWSER_TABS.contains("child(if filter.is_empty()"),
        "Browser Tabs empty-state copy must not regress to inline filter-empty branching"
    );
}
