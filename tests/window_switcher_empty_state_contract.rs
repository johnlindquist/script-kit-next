const WINDOW_SWITCHER: &str = include_str!("../src/render_builtins/window_switcher.rs");

#[test]
fn window_switcher_empty_state_copy_is_modeled() {
    assert!(
        WINDOW_SWITCHER.contains("enum WindowSwitcherEmptyState")
            && WINDOW_SWITCHER.contains("NoWindowsFound")
            && WINDOW_SWITCHER.contains("NoFilteredMatches"),
        "Window Switcher empty-state copy should use named states"
    );
    assert!(
        WINDOW_SWITCHER.contains("fn from_filter(filter: &str) -> Self")
            && WINDOW_SWITCHER.contains("fn message(self) -> &'static str"),
        "Window Switcher empty states should own filter classification and visible copy"
    );
    assert!(
        WINDOW_SWITCHER.contains("WindowSwitcherEmptyState::from_filter(&filter).message()"),
        "Window Switcher renderer should derive empty-state copy from the model"
    );
    assert!(
        !WINDOW_SWITCHER.contains("child(if filter.is_empty()"),
        "Window Switcher empty-state copy must not regress to inline filter-empty branching"
    );
}
