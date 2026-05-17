const PROCESS_MANAGER: &str = include_str!("../src/render_builtins/process_manager.rs");

#[test]
fn process_manager_empty_state_copy_is_modeled() {
    assert!(
        PROCESS_MANAGER.contains("enum ProcessManagerEmptyState")
            && PROCESS_MANAGER.contains("NoRunningScripts")
            && PROCESS_MANAGER.contains("NoFilteredMatches"),
        "Process Manager empty-state copy should use named states"
    );
    assert!(
        PROCESS_MANAGER.contains("fn from_filter(filter: &str) -> Self")
            && PROCESS_MANAGER.contains("fn message(self) -> &'static str"),
        "Process Manager empty states should own filter classification and visible copy"
    );
    assert!(
        PROCESS_MANAGER.contains("ProcessManagerEmptyState::from_filter(&filter).message()"),
        "Process Manager renderer should derive empty-state copy from the model"
    );
    assert!(
        !PROCESS_MANAGER.contains("child(if filter.is_empty()"),
        "Process Manager empty-state copy must not regress to inline filter-empty branching"
    );
}

#[test]
fn process_manager_count_label_copy_is_modeled() {
    assert!(
        PROCESS_MANAGER.contains("fn process_manager_count_label("),
        "Process Manager header count copy should live in a named helper"
    );
    assert!(
        PROCESS_MANAGER.contains("let suffix = if total_count == 1 { \"\" } else { \"es\" };")
            && PROCESS_MANAGER.contains("format!(\"{} process{}\", total_count, suffix)"),
        "Process Manager count helper should avoid '1 processes'"
    );
    assert!(
        PROCESS_MANAGER.contains("Self::process_manager_count_label(total_count)"),
        "Process Manager renderer should use the count label helper"
    );
}
