const CLIPBOARD_HISTORY: &str = include_str!("../src/render_builtins/clipboard.rs");

#[test]
fn clipboard_history_empty_state_copy_is_modeled() {
    assert!(
        CLIPBOARD_HISTORY.contains("enum ClipboardHistoryEmptyState")
            && CLIPBOARD_HISTORY.contains("NoHistory")
            && CLIPBOARD_HISTORY.contains("NoFilteredMatches"),
        "Clipboard History empty-state copy should use named states"
    );
    assert!(
        CLIPBOARD_HISTORY.contains("fn from_filter(filter: &str) -> Self")
            && CLIPBOARD_HISTORY.contains("fn message(self) -> &'static str"),
        "Clipboard History empty states should own filter classification and visible copy"
    );
    assert!(
        CLIPBOARD_HISTORY.contains("ClipboardHistoryEmptyState::from_filter(&filter).message()"),
        "Clipboard History renderer should derive empty-state copy from the model"
    );
    assert!(
        !CLIPBOARD_HISTORY.contains("child(if filter.is_empty()"),
        "Clipboard History empty-state copy must not regress to inline filter-empty branching"
    );
}
