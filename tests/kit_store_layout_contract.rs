const LAYOUT_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");

#[test]
fn kit_store_browse_layout_receipt_uses_custom_surface_nodes() {
    let browse_branch = LAYOUT_SOURCE
        .find("if let AppView::BrowseKitsView")
        .expect("build_layout_info must have a KitStoreBrowse-specific layout branch");
    let generic_script_list = LAYOUT_SOURCE
        .find("LayoutComponentInfo::new(\"ScriptList\"")
        .expect("build_layout_info must retain the generic ScriptList layout branch");

    assert!(
        browse_branch < generic_script_list,
        "KitStoreBrowse must be measured before the generic ScriptList branch so receipts do not report 40px launcher rows or a preview panel"
    );

    for needle in [
        "KitStoreBrowseHeader",
        "KitStoreBrowseSearch",
        "KitStoreBrowseCount",
        "KitStoreBrowseList",
        "KitStoreBrowseRow",
        "KitStoreBrowseInstallButton",
        "KitStoreBrowseFooter",
        "KIT_STORE_ROW_HEIGHT: f32 = 72.0",
        "instead of the generic launcher split shell",
    ] {
        assert!(
            LAYOUT_SOURCE.contains(needle),
            "KitStoreBrowse layout receipt is missing `{needle}`"
        );
    }
}
