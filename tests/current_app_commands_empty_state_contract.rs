const CURRENT_APP_COMMANDS: &str = include_str!("../src/render_builtins/current_app_commands.rs");

#[test]
fn current_app_commands_empty_state_copy_is_modeled() {
    assert!(
        CURRENT_APP_COMMANDS.contains("enum CurrentAppCommandsEmptyState")
            && CURRENT_APP_COMMANDS.contains("NoCommandsReady")
            && CURRENT_APP_COMMANDS.contains("NoFilteredMatches"),
        "Current App Commands empty-state copy should use named states"
    );
    assert!(
        CURRENT_APP_COMMANDS.contains("fn from_filter(filter: &str) -> Self")
            && CURRENT_APP_COMMANDS.contains("fn title(self) -> &'static str")
            && CURRENT_APP_COMMANDS.contains("fn detail(self, filter: &str) -> String"),
        "Current App Commands empty state should own filter classification, title, and detail copy"
    );
    assert!(
        CURRENT_APP_COMMANDS
            .contains("let empty_state = CurrentAppCommandsEmptyState::from_filter(&filter);")
            && CURRENT_APP_COMMANDS.contains("let empty_title = empty_state.title();")
            && CURRENT_APP_COMMANDS.contains("let empty_detail = empty_state.detail(&filter);"),
        "Current App Commands renderer should derive empty-state title/detail from the model"
    );
    assert!(
        !CURRENT_APP_COMMANDS.contains("let empty_title = if filter.trim().is_empty()")
            && !CURRENT_APP_COMMANDS.contains("let empty_detail = if filter.trim().is_empty()"),
        "Current App Commands empty-state copy must not regress to inline filter-empty branching"
    );
}
