const SDK_REFERENCE: &str = include_str!("../src/render_builtins/sdk_reference.rs");

#[test]
fn sdk_reference_empty_state_copy_is_modeled() {
    assert!(
        SDK_REFERENCE.contains("enum SdkReferenceEmptyState")
            && SDK_REFERENCE.contains("NoFunctionsAvailable")
            && SDK_REFERENCE.contains("NoFilteredMatches"),
        "SDK Reference empty-state copy should use named states"
    );
    assert!(
        SDK_REFERENCE.contains("fn from_filter(filter: &str) -> Self")
            && SDK_REFERENCE.contains("fn message(self) -> &'static str"),
        "SDK Reference empty states should own filter classification and visible copy"
    );
    assert!(
        SDK_REFERENCE.contains("SdkReferenceEmptyState::from_filter(filter).message()"),
        "SDK Reference renderer should derive empty-state copy from the model"
    );
    assert!(
        !SDK_REFERENCE.contains("child(if filter.trim().is_empty()"),
        "SDK Reference empty-state copy must not regress to inline filter-empty branching"
    );
}

#[test]
fn sdk_reference_row_description_copy_is_modeled() {
    assert!(
        SDK_REFERENCE.contains("fn sdk_reference_row_description("),
        "SDK Reference row description fallback should have one owner"
    );
    assert!(
        SDK_REFERENCE.contains("Self::sdk_reference_row_description(entry)"),
        "SDK Reference row rendering should use the shared description helper"
    );
    assert!(
        !SDK_REFERENCE.contains("let description = if entry.signature.is_empty()"),
        "SDK Reference row description must not regress to inline signature-empty branching"
    );
}
