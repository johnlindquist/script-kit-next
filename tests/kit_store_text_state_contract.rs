const KIT_STORE: &str = include_str!("../src/render_builtins/kit_store.rs");

#[test]
fn kit_store_browse_empty_copy_is_modeled_by_query_state() {
    assert!(
        KIT_STORE.contains("enum KitStoreBrowseEmptyState"),
        "Kit Store browse empty copy should be modeled as named states"
    );
    assert!(
        KIT_STORE.contains("KitStoreBrowseEmptyState::from_query(&query_owned)"),
        "browse renderer should derive empty copy from query state"
    );
    assert!(
        KIT_STORE.contains("Self::NoFeaturedKits => \"No kits available\"")
            && KIT_STORE
                .contains("Self::NoFeaturedKits => \"Check your network connection or try again\""),
        "empty browse query should not tell the user to try a different search query"
    );
    assert!(
        KIT_STORE.contains("Self::NoSearchResults => \"No kits found\"")
            && KIT_STORE.contains("Self::NoSearchResults => \"Try a different search query\""),
        "non-empty browse query should keep search-specific empty copy"
    );
}

#[test]
fn kit_store_installed_empty_copy_is_modeled() {
    assert!(
        KIT_STORE.contains("enum KitStoreInstalledEmptyState"),
        "installed kits empty copy should be modeled as a named state"
    );
    assert!(
        KIT_STORE.contains("let empty_state = KitStoreInstalledEmptyState::Empty;"),
        "installed renderer should use the modeled empty state"
    );
    assert!(
        KIT_STORE.contains("Self::Empty => \"No installed kits\"")
            && KIT_STORE.contains("Self::Empty => \"Use \\\"Browse Kit Store\\\" to install one\""),
        "installed empty state should retain accurate browse handoff copy"
    );
}

#[test]
fn kit_store_row_detail_copy_is_modeled() {
    assert!(
        KIT_STORE.contains("fn kit_store_browse_row_description("),
        "browse result description fallback should live in a named helper"
    );
    assert!(
        KIT_STORE
            .matches("Self::kit_store_browse_row_description")
            .count()
            >= 1,
        "browse rows should render description copy through the helper"
    );
    assert!(
        KIT_STORE.contains("fn kit_store_installed_row_commit_label("),
        "installed row commit copy should live in a named helper"
    );
    assert!(
        KIT_STORE
            .matches("Self::kit_store_installed_row_commit_label")
            .count()
            >= 1,
        "installed rows should render commit copy through the helper"
    );
}

#[test]
fn kit_store_input_and_count_copy_are_modeled() {
    assert!(
        KIT_STORE.contains("fn kit_store_browse_input_display("),
        "browse input placeholder/value copy should live in a named helper"
    );
    assert!(
        KIT_STORE.contains("Self::kit_store_browse_input_display(&query_owned)"),
        "browse renderer should use the input display helper"
    );
    assert!(
        KIT_STORE.contains("fn kit_store_browse_count_label(")
            && KIT_STORE.contains("format!(\"{} kit{}\", total_results, suffix)"),
        "browse count helper should avoid '1 kits'"
    );
    assert!(
        KIT_STORE.contains("Self::kit_store_browse_count_label(total_results)"),
        "browse renderer should use the count label helper"
    );
    assert!(
        KIT_STORE.contains("fn kit_store_installed_count_label(")
            && KIT_STORE.contains("format!(\"{} installed kit{}\", total_kits, suffix)"),
        "installed count helper should avoid ambiguous or unpluralized installed counts"
    );
    assert!(
        KIT_STORE.contains("Self::kit_store_installed_count_label(total_kits)"),
        "installed renderer should use the installed count label helper"
    );
}
