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
        source.contains("SHELL_DIVIDER_ALPHA_KNOB_ID")
            && source.contains("LIST_AVERAGE_SCROLL_HEIGHT_KNOB_ID")
            && source.contains("ROW_SELECTED_BORDER_ALPHA_KNOB_ID")
            && source.contains("ICON_TILE_BORDER_ALPHA_KNOB_ID"),
        "main window divider, list, row, and icon numeric tokens must be cataloged"
    );
    assert!(
        source.contains("METADATA_ALPHA_KNOB_ID")
            && source.contains("\"metadata.typeAccessorySize\"")
            && source.contains("METADATA_BADGE_PADDING_X_KNOB_ID")
            && source.contains("\"metadata.badgePaddingX\"")
            && source.contains("METADATA_BADGE_PADDING_Y_KNOB_ID")
            && source.contains("\"metadata.badgePaddingY\"")
            && source.contains("METADATA_BADGE_RADIUS_KNOB_ID")
            && source.contains("\"metadata.badgeRadius\"")
            && source.contains("FOOTER_SIDE_INSET_KNOB_ID")
            && source.contains("FOOTER_BUTTON_BORDER_ALPHA_KNOB_ID"),
        "metadata and footer numeric tokens must be cataloged"
    );
    assert!(
        source.contains("HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID")
            && source.contains("\"headerInfo.contextEdgeOutsetX\"")
            && source.contains("HEADER_INFO_VARIATION_BADGE_WIDTH_KNOB_ID")
            && source.contains("\"headerInfo.variationBadgeWidth\""),
        "header info context geometry must be controlled through the dev style tool catalog"
    );
    assert!(
        source.contains("LIST_SECTION_PADDING_X_KNOB_ID")
            && source.contains("\"list.sectionPaddingX\"")
            && source.contains("LIST_SECTION_PADDING_TOP_KNOB_ID")
            && source.contains("\"list.sectionPaddingTop\"")
            && source.contains("LIST_SECTION_PADDING_BOTTOM_KNOB_ID")
            && source.contains("\"list.sectionPaddingBottom\"")
            && source.contains("LIST_SECTION_GAP_KNOB_ID")
            && source.contains("\"list.sectionGap\"")
            && source.contains("LIST_SECTION_ICON_SIZE_KNOB_ID")
            && source.contains("\"list.sectionIconSize\""),
        "section header geometry must be controlled through the dev style tool catalog"
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

#[test]
fn list_item_metadata_badges_use_main_menu_metadata_metrics() {
    let source = fs::read_to_string("src/list_item/mod.rs").expect("read list item source");

    assert!(source.contains("source_font_size: def.metadata.source_font_size"));
    assert!(source.contains("badge_font_size: def.metadata.badge_font_size"));
    assert!(source.contains("badge_padding_x: def.metadata.badge_padding_x"));
    assert!(source.contains("badge_padding_y: def.metadata.badge_padding_y"));
    assert!(source.contains("badge_radius: def.metadata.badge_radius"));
    assert!(source.contains("text_size(px(metrics.source_font_size))"));
    assert!(source.contains("text_size(px(metrics.badge_font_size))"));
    assert!(source.contains(".px(px(metrics.badge_padding_x))"));
    assert!(source.contains(".py(px(metrics.badge_padding_y))"));
    assert!(source.contains(".rounded(px(metrics.badge_radius))"));
}

#[test]
fn list_item_does_not_retain_dead_left_block_accent_branch() {
    let source = fs::read_to_string("src/list_item/mod.rs").expect("read list item source");

    assert!(
        !source.contains("let left_block = false"),
        "dead left_block row-accent gate should not remain"
    );
    assert!(
        !source.contains("if left_block"),
        "dead left_block row-accent branch should not remain"
    );
    assert!(
        !source.contains("block_alpha"),
        "dead left_block accent alpha ladder should not remain"
    );
}
