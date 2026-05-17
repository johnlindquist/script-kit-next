const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");

#[test]
fn kit_store_update_all_result_uses_named_outcome_state() {
    assert!(
        BUILTIN_EXECUTION.contains("struct KitStoreUpdateAllResult"),
        "Kit Store update-all should keep an explicit result model"
    );
    assert!(
        BUILTIN_EXECUTION.contains("enum KitStoreUpdateAllOutcome")
            && BUILTIN_EXECUTION.contains("Complete")
            && BUILTIN_EXECUTION.contains("PartialFailure"),
        "Kit Store update-all should classify counts through named outcome states"
    );
    assert!(
        BUILTIN_EXECUTION.contains("fn outcome(self) -> KitStoreUpdateAllOutcome")
            && BUILTIN_EXECUTION.contains("match self.failed")
            && BUILTIN_EXECUTION.contains("0 => KitStoreUpdateAllOutcome::Complete")
            && BUILTIN_EXECUTION.contains("_ => KitStoreUpdateAllOutcome::PartialFailure"),
        "Kit Store update-all outcome should be derived in one testable state transition"
    );
}

#[test]
fn kit_store_update_all_message_and_toast_use_outcome_state() {
    assert!(
        BUILTIN_EXECUTION.contains("match self.outcome()")
            && BUILTIN_EXECUTION.contains("KitStoreUpdateAllOutcome::Complete")
            && BUILTIN_EXECUTION.contains("Updated {updated} kit(s) successfully")
            && BUILTIN_EXECUTION.contains("KitStoreUpdateAllOutcome::PartialFailure")
            && BUILTIN_EXECUTION.contains("Updated {updated} kit(s), {failed} failed"),
        "Kit Store update-all copy should be selected by named outcome state"
    );
    assert!(
        BUILTIN_EXECUTION
            .contains("matches!(self.outcome(), KitStoreUpdateAllOutcome::PartialFailure)"),
        "Kit Store update-all toast severity should use the named partial-failure state"
    );
    assert!(
        !BUILTIN_EXECUTION.contains("if self.failed == 0"),
        "Kit Store update-all feedback must not regress to ad hoc failed-count branching"
    );
}
