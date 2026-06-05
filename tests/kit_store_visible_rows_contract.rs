//! Source-level contract for Kit Store visible-row ownership.

const KIT_STORE: &str = include_str!("../src/render_builtins/kit_store.rs");
const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn kit_store_defines_visible_row_helper_family() {
    for helper in [
        "kit_store_browse_visible_rows",
        "kit_store_installed_visible_rows",
        "kit_store_browse_selected_visible_result",
        "kit_store_installed_selected_visible_kit",
        "kit_store_browse_dataset_and_visible_counts",
        "kit_store_installed_dataset_and_visible_counts",
        "kit_store_browse_visible_row_labels",
        "kit_store_installed_visible_row_labels",
    ] {
        assert!(KIT_STORE.contains(helper), "missing helper {helper}");
    }
}

#[test]
fn kit_store_current_selection_paths_use_visible_helpers() {
    assert!(KIT_STORE
        .contains("Self::kit_store_browse_selected_visible_result(results, *selected_index)"));
    assert!(KIT_STORE
        .contains("Self::kit_store_installed_selected_visible_kit(kits, filter, *selected_index)"));
    assert!(
        !KIT_STORE.contains("results.get(*selected_index)")
            && !KIT_STORE.contains("kits.get(*selected_index)"),
        "selected-index reads must go through Kit Store visible-row helpers"
    );
}

#[test]
fn kit_store_get_state_and_get_elements_use_visible_helpers() {
    assert!(PROMPT_HANDLER.contains("kit_store_browse_dataset_and_visible_counts"));
    assert!(PROMPT_HANDLER.contains("kit_store_installed_dataset_and_visible_counts(kits, filter)"));
    assert!(COLLECT_ELEMENTS.contains("kit_store_browse_visible_row_labels(results)"));
    assert!(COLLECT_ELEMENTS.contains("kit_store_installed_visible_row_labels(kits, filter)"));
}
