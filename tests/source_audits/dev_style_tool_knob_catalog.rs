use std::fs;

#[test]
fn dev_style_tool_catalog_owns_search_height_descriptor() {
    let source =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");

    assert!(
        source.contains("SEARCH_HEIGHT_KNOB_ID")
            && source.contains("\"search.height\"")
            && source.contains("\"Main input height\""),
        "search.height must be a named descriptor in the dev style tool catalog"
    );
    assert!(
        source.contains("LIST_ITEM_HEIGHT_KNOB_ID")
            && source.contains("\"list.itemHeight\"")
            && source.contains("\"Item height\""),
        "list.itemHeight must be a named descriptor in the dev style tool catalog"
    );
    assert!(
        source.contains("ROW_INNER_PADDING_X_KNOB_ID")
            && source.contains("\"row.innerPaddingX\"")
            && source.contains("\"Item inner padding X\""),
        "row padding must be controlled through the dev style tool catalog"
    );
    assert!(
        source.contains("STYLE_KNOBS"),
        "dev style controls must render from the shared STYLE_KNOBS catalog"
    );
}

#[test]
fn main_menu_theme_keeps_override_free_base_def() {
    let source = fs::read_to_string("src/designs/core/main_menu_theme.rs")
        .expect("read main menu theme source");

    assert!(
        source.contains("pub fn base_def(self) -> MainMenuThemeDef")
            && source.contains("apply_to_main_menu_def(self.base_def())"),
        "MainMenuThemeVariant must expose base_def() and apply runtime overrides through def()"
    );
}
