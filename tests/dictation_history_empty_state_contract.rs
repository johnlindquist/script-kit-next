const DICTATION_HISTORY: &str = include_str!("../src/render_builtins/dictation_history.rs");

#[test]
fn dictation_history_empty_state_copy_is_modeled() {
    assert!(
        DICTATION_HISTORY.contains("enum DictationHistoryEmptyState")
            && DICTATION_HISTORY.contains("NoSavedDictation")
            && DICTATION_HISTORY.contains("NoFilteredMatches"),
        "Dictation History empty-state copy should use named states"
    );
    assert!(
        DICTATION_HISTORY.contains("fn from_filter(filter: &str) -> Self")
            && DICTATION_HISTORY.contains("fn message(self) -> &'static str"),
        "Dictation History empty states should own filter classification and visible copy"
    );
    assert!(
        DICTATION_HISTORY.contains("DictationHistoryEmptyState::from_filter(&filter).message()"),
        "Dictation History renderer should derive empty-state copy from the model"
    );
    assert!(
        !DICTATION_HISTORY.contains("child(if filter.is_empty()"),
        "Dictation History empty-state copy must not regress to inline filter-empty branching"
    );
}
