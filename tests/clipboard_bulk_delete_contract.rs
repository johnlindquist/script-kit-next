const CLIPBOARD_ACTIONS: &str = include_str!("../src/app_actions/handle_action/clipboard.rs");

#[test]
fn clipboard_bulk_delete_counts_use_named_result_outcome() {
    assert!(
        CLIPBOARD_ACTIONS.contains("struct ClipboardBulkDeleteResult")
            && CLIPBOARD_ACTIONS.contains("deleted: usize")
            && CLIPBOARD_ACTIONS.contains("failed: usize"),
        "clipboard bulk delete should keep a typed aggregate result"
    );
    assert!(
        CLIPBOARD_ACTIONS.contains("enum ClipboardBulkDeleteOutcome")
            && CLIPBOARD_ACTIONS.contains("Complete")
            && CLIPBOARD_ACTIONS.contains("PartialFailure"),
        "clipboard bulk delete should classify aggregate counts with named outcomes"
    );
    assert!(
        CLIPBOARD_ACTIONS.contains("fn outcome(self) -> ClipboardBulkDeleteOutcome")
            && CLIPBOARD_ACTIONS.contains("0 => ClipboardBulkDeleteOutcome::Complete")
            && CLIPBOARD_ACTIONS.contains("_ => ClipboardBulkDeleteOutcome::PartialFailure"),
        "clipboard bulk delete result should derive outcome in one testable transition"
    );
}

#[test]
fn clipboard_bulk_delete_feedback_uses_named_result_outcome() {
    assert!(
        CLIPBOARD_ACTIONS.contains("fn show_feedback(")
            && CLIPBOARD_ACTIONS.contains("match self.outcome()")
            && CLIPBOARD_ACTIONS.contains("ClipboardBulkDeleteOutcome::Complete")
            && CLIPBOARD_ACTIONS.contains("action.success_hud(self.deleted)")
            && CLIPBOARD_ACTIONS.contains("ClipboardBulkDeleteOutcome::PartialFailure")
            && CLIPBOARD_ACTIONS
                .contains("action.partial_failure_message(self.deleted, self.failed)"),
        "clipboard bulk delete feedback should route through the named result outcome"
    );
    assert!(
        CLIPBOARD_ACTIONS
            .contains("ClipboardBulkDeleteResult { deleted, failed }\n                            .show_feedback(bulk_delete_action, this, cx)"),
        "clipboard bulk delete execution should hand aggregate counts to the result model"
    );
    assert!(
        !CLIPBOARD_ACTIONS.contains("if failed == 0"),
        "clipboard bulk delete feedback must not regress to direct failed-count branching"
    );
}

#[test]
fn clipboard_delete_all_uses_named_unpinned_availability() {
    assert!(
        CLIPBOARD_ACTIONS.contains("enum ClipboardUnpinnedDeleteAvailability")
            && CLIPBOARD_ACTIONS.contains("Empty")
            && CLIPBOARD_ACTIONS.contains("Available { unpinned_count: usize }"),
        "clipboard delete-all should classify the unpinned-entry guard with a named state"
    );
    assert!(
        CLIPBOARD_ACTIONS.contains("fn from_count(unpinned_count: usize) -> Self")
            && CLIPBOARD_ACTIONS.contains("0 => Self::Empty")
            && CLIPBOARD_ACTIONS.contains("unpinned_count => Self::Available { unpinned_count }")
            && CLIPBOARD_ACTIONS.contains("fn count(self) -> Option<usize>")
            && CLIPBOARD_ACTIONS.contains("Self::Empty => None")
            && CLIPBOARD_ACTIONS
                .contains("Self::Available { unpinned_count } => Some(unpinned_count)"),
        "clipboard delete-all availability should own count classification and extraction"
    );
    assert!(
        CLIPBOARD_ACTIONS.contains("fn unpinned_availability(self, unpinned_count: usize) -> ClipboardUnpinnedDeleteAvailability")
            && CLIPBOARD_ACTIONS.contains(".unpinned_availability(unpinned_count)")
            && CLIPBOARD_ACTIONS.contains(".count()")
            && CLIPBOARD_ACTIONS.contains("bulk_delete_action.no_unpinned_message()"),
        "clipboard delete-all should route its empty guard through the named availability state"
    );
    assert!(
        !CLIPBOARD_ACTIONS.contains("if unpinned_count == 0"),
        "clipboard delete-all must not regress to direct unpinned-count branching"
    );
}
