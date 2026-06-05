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

#[test]
fn menu_syntax_main_hint_form_field_chrome_is_list_tokenized() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let render_script_list =
        fs::read_to_string("src/render_script_list/mod.rs").expect("read render script list");

    assert!(theme.contains("pub main_hint_form_focused_border_alpha: u32"));
    assert!(theme.contains("pub main_hint_form_border_alpha: u32"));
    assert!(theme.contains("pub main_hint_form_focused_bg_alpha: u32"));
    assert!(theme.contains("pub main_hint_form_bg_alpha: u32"));
    assert!(theme.contains("pub main_hint_form_label_alpha: u32"));
    assert!(theme.contains("pub main_hint_form_value_alpha: u32"));

    assert!(catalog.contains("LIST_MAIN_HINT_FORM_FOCUSED_BORDER_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintFormFocusedBorderAlpha\""));
    assert!(catalog.contains("\"Main hint form focused border alpha\""));
    assert!(catalog.contains("LIST_MAIN_HINT_FORM_BG_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintFormBgAlpha\""));
    assert!(catalog.contains("\"Main hint form background alpha\""));
    assert!(catalog.contains("LIST_MAIN_HINT_FORM_VALUE_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.mainHintFormValueAlpha\""));
    assert!(catalog.contains("\"Main hint form value alpha\""));

    let field_body = function_body(&render_script_list, "render_menu_syntax_form_field");
    assert!(field_body.contains("list_tokens.main_hint_form_focused_border_alpha"));
    assert!(field_body.contains("list_tokens.main_hint_form_border_alpha"));
    assert!(field_body.contains("list_tokens.main_hint_form_focused_bg_alpha"));
    assert!(field_body.contains("list_tokens.main_hint_form_bg_alpha"));
    assert!(field_body.contains("list_tokens.main_hint_form_label_alpha"));
    assert!(field_body.contains("list_tokens.main_hint_form_value_alpha"));
    assert!(!field_body.contains("| 0xF2"));
    assert!(!field_body.contains("| 0x80"));
    assert!(!field_body.contains("| 0x3D"));
    assert!(!field_body.contains("| 0x24"));
    assert!(!field_body.contains("| 0xB3"));
    assert!(!field_body.contains("| 0xFF"));

    let form_body = render_script_list
        .split("fn render_menu_syntax_form(")
        .nth(1)
        .expect("missing render_menu_syntax_form")
        .split("\nfn ")
        .next()
        .expect("render_menu_syntax_form body should be present");
    let compact_form_body: String = form_body.split_whitespace().collect();
    assert!(compact_form_body.contains(
        "render_menu_syntax_form_field(theme,list_tokens,design_variant,field,input)"
    ));
    let main_hint_body = function_body(&render_script_list, "render_menu_syntax_main_hint");
    assert!(main_hint_body.contains("render_menu_syntax_form(\n                theme,\n                list_tokens,"));
}

#[test]
fn inline_calc_row_chrome_is_list_tokenized() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let render_script_list =
        fs::read_to_string("src/render_script_list/mod.rs").expect("read render script list");

    assert!(theme.contains("pub inline_calc_selected_overlay_min_alpha: u32"));
    assert!(theme.contains("pub inline_calc_selected_hint_alpha: u32"));
    assert!(theme.contains("pub inline_calc_hint_alpha: u32"));
    assert!(theme.contains("pub inline_calc_result_font_size: f32"));
    assert!(theme.contains("pub inline_calc_hint_font_size: f32"));

    assert!(catalog.contains("LIST_INLINE_CALC_SELECTED_OVERLAY_MIN_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.inlineCalcSelectedOverlayMinAlpha\""));
    assert!(catalog.contains("\"Inline calc selected overlay minimum alpha\""));
    assert!(catalog.contains("LIST_INLINE_CALC_SELECTED_HINT_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.inlineCalcSelectedHintAlpha\""));
    assert!(catalog.contains("LIST_INLINE_CALC_HINT_ALPHA_KNOB_ID"));
    assert!(catalog.contains("\"list.inlineCalcHintAlpha\""));
    assert!(catalog.contains("LIST_INLINE_CALC_RESULT_FONT_SIZE_KNOB_ID"));
    assert!(catalog.contains("\"list.inlineCalcResultFontSize\""));
    assert!(catalog.contains("LIST_INLINE_CALC_HINT_FONT_SIZE_KNOB_ID"));
    assert!(catalog.contains("\"list.inlineCalcHintFontSize\""));

    let overlay_body =
        function_body(&render_script_list, "inline_calc_list_item_selected_overlay_rgba");
    assert!(overlay_body.contains("list_tokens.inline_calc_selected_overlay_min_alpha"));
    assert!(!overlay_body.contains(".max(0x2E)"));

    let row_body = function_body(&render_script_list, "render_inline_calc_list_item");
    assert!(row_body.contains("list_tokens.inline_calc_selected_hint_alpha"));
    assert!(row_body.contains("list_tokens.inline_calc_hint_alpha"));
    assert!(row_body.contains("list_tokens.inline_calc_result_font_size"));
    assert!(row_body.contains("list_tokens.inline_calc_hint_font_size"));
    assert!(!row_body.contains("typography.font_size_lg"));
    assert!(!row_body.contains("typography.font_size_xs"));
    assert!(!row_body.contains("0xD9"));
    assert!(!row_body.contains("0x8C"));
    assert!(render_script_list.contains(
        "this.current_main_menu_theme.def().list,\n                                            this.current_design,"
    ));
}

#[test]
fn main_menu_font_sizes_are_design_tool_controls() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let render_script_list =
        fs::read_to_string("src/render_script_list/mod.rs").expect("read render script list");

    for required in [
        "SEARCH_FONT_SIZE_KNOB_ID",
        "TYPOGRAPHY_NAME_FONT_SIZE_KNOB_ID",
        "TYPOGRAPHY_DESC_FONT_SIZE_KNOB_ID",
        "TYPOGRAPHY_SECTION_FONT_SIZE_KNOB_ID",
        "METADATA_SOURCE_FONT_SIZE_KNOB_ID",
        "METADATA_BADGE_FONT_SIZE_KNOB_ID",
        "METADATA_KEYCAP_FONT_SIZE_KNOB_ID",
        "FOOTER_LABEL_FONT_SIZE_KNOB_ID",
        "FOOTER_KEYCAP_FONT_SIZE_KNOB_ID",
        "HEADER_INFO_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_CHIP_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_ROW_LABEL_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_ROW_VALUE_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_FRAGMENT_ROLE_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_FRAGMENT_VALUE_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_TITLE_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_BODY_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_EXAMPLE_LABEL_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_FORM_LABEL_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_FORM_INPUT_FONT_SIZE_KNOB_ID",
        "LIST_MAIN_HINT_FORM_VALUE_FONT_SIZE_KNOB_ID",
    ] {
        assert!(catalog.contains(required), "missing font-size knob {required}");
    }

    for id in [
        "\"search.fontSize\"",
        "\"typography.nameFontSize\"",
        "\"typography.descFontSize\"",
        "\"typography.sectionFontSize\"",
        "\"metadata.sourceFontSize\"",
        "\"metadata.badgeFontSize\"",
        "\"metadata.keycapFontSize\"",
        "\"footer.labelFontSize\"",
        "\"footer.keycapFontSize\"",
        "\"headerInfo.fontSize\"",
        "\"list.mainHintTitleFontSize\"",
        "\"list.mainHintBodyFontSize\"",
        "\"list.mainHintExampleLabelFontSize\"",
        "\"list.mainHintFormLabelFontSize\"",
        "\"list.mainHintFormInputFontSize\"",
        "\"list.mainHintFormValueFontSize\"",
    ] {
        assert!(catalog.contains(id), "missing font-size control id {id}");
    }

    assert!(theme.contains("pub main_hint_title_font_size: f32"));
    assert!(theme.contains("pub main_hint_body_font_size: f32"));
    assert!(theme.contains("pub main_hint_example_label_font_size: f32"));
    assert!(theme.contains("pub main_hint_form_label_font_size: f32"));
    assert!(theme.contains("pub main_hint_form_input_font_size: f32"));
    assert!(theme.contains("pub main_hint_form_value_font_size: f32"));

    let main_hint_body = function_body(&render_script_list, "render_menu_syntax_main_hint");
    assert!(main_hint_body.contains("list_tokens.main_hint_title_font_size"));
    assert!(main_hint_body.contains("list_tokens.main_hint_body_font_size"));
    assert!(main_hint_body.contains("list_tokens.main_hint_example_label_font_size"));
    assert!(!main_hint_body.contains("metrics.body_size - 1.0"));
    assert!(!main_hint_body.contains("metrics.body_line - 2.0"));

    let form_body = function_body(&render_script_list, "render_menu_syntax_form_field");
    assert!(form_body.contains("list_tokens.main_hint_form_label_font_size"));
    assert!(form_body.contains("list_tokens.main_hint_form_input_font_size"));
    assert!(form_body.contains("list_tokens.main_hint_form_value_font_size"));
    assert!(!form_body.contains("field_metrics.label_font_size"));
    assert!(!form_body.contains("field_metrics.input_font_size"));
}

#[test]
fn footer_keycap_and_slot_sizes_are_design_tool_controls() {
    let theme =
        fs::read_to_string("src/designs/core/main_menu_theme.rs").expect("read theme source");
    let catalog =
        fs::read_to_string("src/dev_style_tool/catalog.rs").expect("read dev style catalog");
    let footer =
        fs::read_to_string("src/components/footer_chrome.rs").expect("read footer chrome source");

    for required in [
        "pub keycap_height: f32",
        "pub key_glyph_nudge_y: f32",
        "pub return_glyph_nudge_y: f32",
        "pub semicolon_glyph_nudge_y: f32",
        "pub run_slot_min_width: f32",
        "pub run_slot_max_width: f32",
        "pub actions_slot_width: f32",
        "pub ai_slot_width: f32",
        "pub apply_slot_width: f32",
        "pub close_slot_width: f32",
        "pub stop_slot_width: f32",
        "pub paste_response_slot_width: f32",
    ] {
        assert!(theme.contains(required), "missing footer metric {required}");
    }

    for required in [
        "FOOTER_KEYCAP_HEIGHT_KNOB_ID",
        "\"footer.keycapHeight\"",
        "FOOTER_KEY_GLYPH_NUDGE_Y_KNOB_ID",
        "\"footer.keyGlyphNudgeY\"",
        "FOOTER_RETURN_GLYPH_NUDGE_Y_KNOB_ID",
        "\"footer.returnGlyphNudgeY\"",
        "FOOTER_SEMICOLON_GLYPH_NUDGE_Y_KNOB_ID",
        "\"footer.semicolonGlyphNudgeY\"",
        "FOOTER_RUN_SLOT_MIN_WIDTH_KNOB_ID",
        "\"footer.runSlotMinWidth\"",
        "FOOTER_RUN_SLOT_MAX_WIDTH_KNOB_ID",
        "\"footer.runSlotMaxWidth\"",
        "FOOTER_ACTIONS_SLOT_WIDTH_KNOB_ID",
        "\"footer.actionsSlotWidth\"",
        "FOOTER_AI_SLOT_WIDTH_KNOB_ID",
        "\"footer.aiSlotWidth\"",
        "FOOTER_APPLY_SLOT_WIDTH_KNOB_ID",
        "\"footer.applySlotWidth\"",
        "FOOTER_CLOSE_SLOT_WIDTH_KNOB_ID",
        "\"footer.closeSlotWidth\"",
        "FOOTER_STOP_SLOT_WIDTH_KNOB_ID",
        "\"footer.stopSlotWidth\"",
        "FOOTER_PASTE_RESPONSE_SLOT_WIDTH_KNOB_ID",
        "\"footer.pasteResponseSlotWidth\"",
    ] {
        assert!(catalog.contains(required), "missing footer control {required}");
    }

    let slot_body = function_body(&footer, "footer_action_slot_width_for_metrics");
    assert!(slot_body.contains("metrics.run_slot_min_width"));
    assert!(slot_body.contains("metrics.actions_slot_width"));
    assert!(slot_body.contains("metrics.ai_slot_width"));
    assert!(slot_body.contains("metrics.apply_slot_width"));
    assert!(slot_body.contains("metrics.close_slot_width"));
    assert!(slot_body.contains("metrics.stop_slot_width"));
    assert!(slot_body.contains("metrics.paste_response_slot_width"));
    assert!(!slot_body.contains("FOOTER_RUN_SLOT_MIN_WIDTH_PX"));
    assert!(!slot_body.contains("FOOTER_ACTIONS_SLOT_WIDTH_PX"));

    let keycap_body = function_body(&footer, "render_footer_keycap_with_metrics");
    assert!(keycap_body.contains("keycap_height_px.unwrap_or(metrics.keycap_height)"));
    assert!(!keycap_body.contains("keycap_height_px.unwrap_or(FOOTER_KEYCAP_HEIGHT_PX)"));

    let labelcap_body = function_body(&footer, "render_footer_labelcap_constrained");
    assert!(labelcap_body.contains(".min_h(px(metrics.keycap_height))"));
    assert!(labelcap_body.contains(".line_height(px(metrics.keycap_height))"));
    assert!(!labelcap_body.contains("FOOTER_KEYCAP_HEIGHT_PX"));

    let nudge_body = function_body(&footer, "footer_key_glyph_nudge_y");
    assert!(nudge_body.contains("metrics.key_glyph_nudge_y"));
    assert!(nudge_body.contains("metrics.return_glyph_nudge_y"));
    assert!(nudge_body.contains("metrics.semicolon_glyph_nudge_y"));
}
