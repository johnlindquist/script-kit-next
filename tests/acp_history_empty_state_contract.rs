const ACP_HISTORY: &str = include_str!("../src/render_builtins/acp_history.rs");

#[test]
fn acp_history_empty_state_copy_is_modeled() {
    assert!(
        ACP_HISTORY.contains("enum AcpHistoryEmptyState")
            && ACP_HISTORY.contains("NoConversationHistory")
            && ACP_HISTORY.contains("NoFilteredMatches"),
        "ACP History empty-state copy should use named states"
    );
    assert!(
        ACP_HISTORY.contains("fn from_filter(filter: &str) -> Self")
            && ACP_HISTORY.contains("fn message(self) -> &'static str"),
        "ACP History empty states should own filter classification and visible copy"
    );
    assert!(
        ACP_HISTORY.contains("AcpHistoryEmptyState::from_filter(&filter).message()"),
        "ACP History renderer should derive empty-state copy from the model"
    );
    assert!(
        !ACP_HISTORY.contains("child(if filter.is_empty()"),
        "ACP History empty-state copy must not regress to inline filter-empty branching"
    );
}
