const BROWSER_HISTORY: &str = include_str!("../src/render_builtins/browser_history.rs");

#[test]
fn browser_history_empty_state_copy_is_modeled() {
    assert!(
        BROWSER_HISTORY.contains("enum BrowserHistoryEmptyState")
            && BROWSER_HISTORY.contains("NoHistoryFound")
            && BROWSER_HISTORY.contains("NoFilteredMatches"),
        "Browser History empty-state copy should use named states"
    );
    assert!(
        BROWSER_HISTORY.contains("fn from_filter(filter: &str) -> Self")
            && BROWSER_HISTORY.contains("fn message(self) -> &'static str"),
        "Browser History empty states should own filter classification and visible copy"
    );
    assert!(
        BROWSER_HISTORY.contains("BrowserHistoryEmptyState::from_filter(&filter).message()"),
        "Browser History renderer should derive empty-state copy from the model"
    );
    assert!(
        !BROWSER_HISTORY.contains("child(if filter.is_empty()"),
        "Browser History empty-state copy must not regress to inline filter-empty branching"
    );
}
