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
        source.contains("LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID")
            && source.contains("\"list.sourceStatusRowHeight\"")
            && source.contains("\"Source status row height\""),
        "source status row height must be controlled through the dev style tool catalog"
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

#[test]
fn source_status_row_height_is_theme_tokenized() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let list_item = fs::read_to_string("src/list_item/mod.rs").expect("read list item source");
    let render_script_list =
        fs::read_to_string("src/render_script_list/mod.rs").expect("read render script list");

    assert!(theme.contains("pub source_status_row_height: f32"));
    assert!(theme.contains("source_status_row_height: 32.0"));
    assert!(list_item.contains("source_status_row_height: def.list.source_status_row_height"));
    assert!(list_item.contains("effective_source_status_row_height_for_theme"));
    assert!(
        !list_item
            .split("pub fn effective_source_status_row_height()")
            .nth(1)
            .and_then(|body| body.split("#[inline]").next())
            .unwrap_or_default()
            .contains("SOURCE_STATUS_ROW_HEIGHT"),
        "effective_source_status_row_height must not return the raw hard-coded constant"
    );
    assert!(render_script_list
        .contains("effective_source_status_row_height_for_theme(\n                                            current_main_menu_theme"));
}

#[test]
fn selected_name_underline_is_row_tokenized_and_cataloged() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let list_item = fs::read_to_string("src/list_item/mod.rs").expect("read list item source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");

    assert!(theme.contains("pub selected_name_underline_width: f32"));
    assert!(theme.contains("pub selected_name_underline_padding_bottom: f32"));
    assert!(theme.contains("selected_name_underline_width = 2.0"));
    assert!(theme.contains("selected_name_underline_padding_bottom = 1.0"));
    assert!(list_item
        .contains("row_selected_name_underline_width: def.row.selected_name_underline_width"));
    assert!(list_item.contains(".selected_name_underline_padding_bottom"));
    assert!(list_item.contains(".border_b(px(metrics.row_selected_name_underline_width))"));
    assert!(list_item.contains(".pb(px(metrics.row_selected_name_underline_padding_bottom))"));
    assert!(!list_item.contains(".border_b(px(2.0))"));
    assert!(!list_item.contains(".pb(px(1.0))"));
    assert!(
        !list_item.contains(
            "let name_underline_bold = matches!(row_kind, MainMenuRowKind::CarbonNeon) && selected"
        ),
        "renderer should not own the CarbonNeon underline gate"
    );
    assert!(catalog.contains("ROW_SELECTED_NAME_UNDERLINE_WIDTH_KNOB_ID"));
    assert!(catalog.contains("\"row.selectedNameUnderlineWidth\""));
    assert!(catalog.contains("\"Selected name underline width\""));
    assert!(catalog.contains("ROW_SELECTED_NAME_UNDERLINE_PADDING_BOTTOM_KNOB_ID"));
    assert!(catalog.contains("\"row.selectedNameUnderlinePaddingBottom\""));
    assert!(catalog.contains("\"Selected name underline padding bottom\""));
}

fn function_body<'a>(source: &'a str, fn_name: &str) -> &'a str {
    source
        .split(&format!("fn {fn_name}"))
        .nth(1)
        .unwrap_or_else(|| panic!("missing function {fn_name}"))
        .split("\nfn ")
        .next()
        .expect("function body should be present")
}

#[test]
fn menu_syntax_main_hint_helper_chrome_is_list_tokenized() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let render_script_list =
        fs::read_to_string("src/render_script_list/mod.rs").expect("read render script list");

    assert!(theme.contains("pub main_hint_chip_padding_x: f32"));
    assert!(theme.contains("pub main_hint_chip_border_alpha: u32"));
    assert!(theme.contains("pub main_hint_row_label_width: f32"));
    assert!(theme.contains("pub main_hint_fragment_role_width: f32"));
    assert!(theme.contains("pub main_hint_fragment_role_bg_alpha: u32"));

    assert!(catalog.contains("LIST_MAIN_HINT_CHIP_PADDING_X_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintChipPaddingX\""));
    assert!(catalog.contains("\"Main hint chip padding X\""));
    assert!(catalog.contains("LIST_MAIN_HINT_ROW_LABEL_WIDTH_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintRowLabelWidth\""));
    assert!(catalog.contains("\"Main hint label width\""));
    assert!(catalog.contains("LIST_MAIN_HINT_FRAGMENT_ROLE_WIDTH_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintFragmentRoleWidth\""));
    assert!(catalog.contains("\"Main hint fragment role width\""));

    let chip_body = function_body(&render_script_list, "render_menu_syntax_hint_chip");
    assert!(chip_body.contains("list_tokens.main_hint_chip_padding_x"));
    assert!(chip_body.contains("list_tokens.main_hint_chip_border_alpha"));
    assert!(!chip_body.contains(".px(px(8.0))"));
    assert!(!chip_body.contains("| 0x66"));
    assert!(!chip_body.contains("| 0x18"));

    let row_body = function_body(&render_script_list, "render_menu_syntax_hint_row");
    assert!(row_body.contains("list_tokens.main_hint_row_gap"));
    assert!(row_body.contains("list_tokens.main_hint_row_label_width"));
    assert!(row_body.contains("list_tokens.main_hint_row_value_alpha"));
    assert!(!row_body.contains(".gap(px(12.0))"));
    assert!(!row_body.contains(".w(px(76.0))"));
    assert!(!row_body.contains("| 0xCC"));
    assert!(!row_body.contains("| 0xE6"));

    let fragment_body = function_body(&render_script_list, "render_menu_syntax_fragment_preview_row");
    assert!(fragment_body.contains("list_tokens.main_hint_fragment_row_gap"));
    assert!(fragment_body.contains("list_tokens.main_hint_fragment_role_width"));
    assert!(fragment_body.contains("list_tokens.main_hint_fragment_role_bg_alpha"));
    assert!(fragment_body.contains("list_tokens.main_hint_fragment_value_alpha"));
    assert!(!fragment_body.contains(".gap(px(10.0))"));
    assert!(!fragment_body.contains(".w(px(82.0))"));
    assert!(!fragment_body.contains("| 0x55"));
    assert!(!fragment_body.contains("| 0x14"));
}

#[test]
fn menu_syntax_main_hint_container_chrome_is_list_tokenized() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let render_script_list =
        fs::read_to_string("src/render_script_list/mod.rs").expect("read render script list");

    assert!(theme.contains("pub main_hint_status_chip_gap: f32"));
    assert!(theme.contains("pub main_hint_rows_gap: f32"));
    assert!(theme.contains("pub main_hint_fragment_rows_gap: f32"));
    assert!(theme.contains("pub main_hint_warning_border_alpha: u32"));
    assert!(theme.contains("pub main_hint_warning_bg_alpha: u32"));
    assert!(theme.contains("pub main_hint_divider_height: f32"));
    assert!(theme.contains("pub main_hint_examples_group_gap: f32"));
    assert!(theme.contains("pub main_hint_example_row_gap: f32"));

    assert!(catalog.contains("LIST_MAIN_HINT_STATUS_CHIP_GAP_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintStatusChipGap\""));
    assert!(catalog.contains("\"Main hint status chip gap\""));
    assert!(catalog.contains("LIST_MAIN_HINT_WARNING_BG_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintWarningBgAlpha\""));
    assert!(catalog.contains("\"Main hint warning background alpha\""));
    assert!(catalog.contains("LIST_MAIN_HINT_DIVIDER_HEIGHT_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintDividerHeight\""));
    assert!(catalog.contains("\"Main hint divider height\""));
    assert!(catalog.contains("LIST_MAIN_HINT_EXAMPLE_ROW_GAP_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintExampleRowGap\""));
    assert!(catalog.contains("\"Main hint example row gap\""));

    let body = function_body(&render_script_list, "render_menu_syntax_main_hint");
    assert!(body.contains("list_tokens.main_hint_status_chip_gap"));
    assert!(body.contains("list_tokens.main_hint_rows_gap"));
    assert!(body.contains("list_tokens.main_hint_fragment_rows_gap"));
    assert!(body.contains("list_tokens.main_hint_warning_border_alpha"));
    assert!(body.contains("list_tokens.main_hint_warning_bg_alpha"));
    assert!(body.contains("list_tokens.main_hint_divider_height"));
    assert!(body.contains("list_tokens.main_hint_examples_group_gap"));
    assert!(body.contains("list_tokens.main_hint_example_row_gap"));
    assert!(body.contains("\"menu-syntax-main-hint-divider\""));
    assert!(body.contains("\"menu-syntax-main-hint-examples-group\""));
    assert!(!body.contains(".gap(px(6.0))"));
    assert!(!body.contains(".gap(px(7.0))"));
    assert!(!body.contains("| 0x66"));
    assert!(!body.contains("| 0x14"));
    assert!(!body.contains(".h(px(1.0))"));
    assert!(!body.contains(".gap(px(5.0))"));
    assert!(!body.contains(".gap(px(3.0))"));
}
