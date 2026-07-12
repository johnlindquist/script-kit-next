const LAYOUT_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");

fn node_source(name: &str) -> &'static str {
    let start = LAYOUT_SOURCE
        .find(&format!("LayoutComponentInfo::new(\"{name}\""))
        .unwrap_or_else(|| panic!("{name} layout node should exist"));
    let node_source = &LAYOUT_SOURCE[start..];
    let end = node_source
        .find(".with_visual_token")
        .unwrap_or_else(|| panic!("{name} should declare visual metadata"));
    &node_source[..end]
}

#[test]
fn kit_store_custom_lists_use_liquid_glass_panel_radius() {
    for node in ["KitStoreBrowseList", "KitStoreInstalledList"] {
        let source = node_source(node);
        assert!(
            source.contains("LayoutComponentType::List"),
            "{node} must remain a List node"
        );
        assert!(
            source.contains("Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX)"),
            "{node} must use the shared Liquid Glass panel radius token"
        );
        assert!(
            !source.contains("Some(0.0)") && !source.contains("None"),
            "{node} must not satisfy guideline proof with a zero or missing radius"
        );
    }
}

#[test]
fn kit_store_browse_layout_receipt_uses_shared_chrome_and_content_nodes() {
    for needle in [
        "resolved_main_view_header_input_policy",
        "main_view_header_metrics(menu_def, input_height)",
        "MainViewHeader",
        "MainViewContextZone",
        "MainViewInput",
        "MainViewMain",
        "KitStoreBrowseSurface",
        "KitStoreBrowseCount",
        "KitStoreBrowseList",
        "KitStoreBrowseRow",
        "KitStoreBrowseInstallButton",
        "KitStoreBrowseFooter",
        "KIT_STORE_ROW_HEIGHT: f32 = 72.0",
        "canonical MainViewInput trailing slot",
    ] {
        assert!(
            LAYOUT_SOURCE.contains(needle),
            "KitStoreBrowse layout receipt is missing `{needle}`"
        );
    }

    for forbidden in [
        "KitStoreBrowseHeader",
        "KitStoreBrowseSearch",
        "KitStoreBrowseDivider",
        "KIT_STORE_HEADER_HEIGHT",
        "KIT_STORE_INPUT_HEIGHT",
    ] {
        assert!(
            !LAYOUT_SOURCE.contains(forbidden),
            "KitStoreBrowse must not keep stale custom header geometry: {forbidden}"
        );
    }
}

#[test]
fn kit_store_installed_layout_receipt_uses_shared_chrome_and_content_nodes() {
    for needle in [
        "MainViewHeader",
        "MainViewContextZone",
        "MainViewInput",
        "MainViewMain",
        "KitStoreInstalledSurface",
        "KitStoreInstalledCount",
        "KitStoreInstalledList",
        "KitStoreInstalledRow",
        "KitStoreInstalledFooter",
        "crate::list_item::LIST_ITEM_HEIGHT",
        "canonical MainViewInput trailing slot",
        "shared ListItem chrome",
    ] {
        assert!(
            LAYOUT_SOURCE.contains(needle),
            "KitStoreInstalled layout receipt is missing `{needle}`"
        );
    }

    for forbidden in [
        "KitStoreInstalledHeader",
        "KitStoreInstalledSearch",
        "KitStoreInstalledDivider",
        "KitStoreInstalledTitle",
        "KitStoreInstalledUpdateButton",
        "KitStoreInstalledRemoveButton",
        "kitStoreInstalled.actionButton",
        "Installed kit rows are 72px tall",
        "KIT_STORE_HEADER_HEIGHT",
        "KIT_STORE_INPUT_HEIGHT",
    ] {
        assert!(
            !LAYOUT_SOURCE.contains(forbidden),
            "KitStoreInstalled layout receipt must not keep stale custom row chrome: {forbidden}"
        );
    }
}
