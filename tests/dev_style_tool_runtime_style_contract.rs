use gpui::FontWeight;
use script_kit_gpui::designs::MainMenuThemeVariant;
use script_kit_gpui::dev_style_tool::theme_catalog::{
    format_theme_color_hex, THEME_COLOR_KNOBS, THEME_TEXT_PRIMARY_KNOB_ID, THEME_UI_BORDER_KNOB_ID,
};
use script_kit_gpui::dev_style_tool::{
    base_agent_chat_style, base_confirm_modal_style, export, runtime_overrides, StyleValue,
    ACTIONS_POPUP_KNOBS, CONFIRM_MODAL_ACTIONS_EDGE_PADDING_X_KNOB_ID,
    CONFIRM_MODAL_ACTIONS_PADDING_X_KNOB_ID, CONFIRM_MODAL_KNOBS, CONFIRM_MODAL_PADDING_X_KNOB_ID,
    COPY_CONTROLS, FOOTER_ACTIONS_SLOT_WIDTH_KNOB_ID, FOOTER_AI_SLOT_WIDTH_KNOB_ID,
    FOOTER_BUTTON_HOVER_BG_ALPHA_KNOB_ID, FOOTER_BUTTON_HOVER_BORDER_ALPHA_KNOB_ID,
    FOOTER_BUTTON_HOVER_GLYPH_ALPHA_KNOB_ID, FOOTER_BUTTON_HOVER_TEXT_ALPHA_KNOB_ID,
    FOOTER_FONT_WEIGHT_KNOB_ID, FOOTER_HEIGHT_KNOB_ID, FOOTER_KEYCAP_HEIGHT_KNOB_ID,
    FOOTER_KEY_GLYPH_NUDGE_Y_KNOB_ID, FOOTER_PASTE_RESPONSE_SLOT_WIDTH_KNOB_ID,
    FOOTER_RETURN_GLYPH_NUDGE_Y_KNOB_ID, FOOTER_RUN_SLOT_MAX_WIDTH_KNOB_ID,
    FOOTER_RUN_SLOT_MIN_WIDTH_KNOB_ID, FOOTER_SEMICOLON_GLYPH_NUDGE_Y_KNOB_ID,
    FOOTER_SIDE_INSET_KNOB_ID, HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID,
    HEADER_INFO_PILL_HOVER_BG_ALPHA_KNOB_ID, HEADER_INFO_PILL_HOVER_BORDER_ALPHA_KNOB_ID,
    HEADER_INFO_PILL_HOVER_KEY_ALPHA_KNOB_ID, HEADER_INFO_PILL_HOVER_TEXT_ALPHA_KNOB_ID,
    LIST_INLINE_CALC_HINT_ALPHA_KNOB_ID, LIST_INLINE_CALC_HINT_FONT_SIZE_KNOB_ID,
    LIST_INLINE_CALC_RESULT_FONT_SIZE_KNOB_ID, LIST_INLINE_CALC_SELECTED_HINT_ALPHA_KNOB_ID,
    LIST_INLINE_CALC_SELECTED_OVERLAY_MIN_ALPHA_KNOB_ID, LIST_ITEM_HEIGHT_KNOB_ID,
    LIST_ITEM_INNER_PADDING_Y_KNOB_ID, LIST_ITEM_OUTER_PADDING_Y_KNOB_ID,
    LIST_MAIN_HINT_BODY_FONT_SIZE_KNOB_ID, LIST_MAIN_HINT_CHIP_BORDER_ALPHA_KNOB_ID,
    LIST_MAIN_HINT_CHIP_PADDING_X_KNOB_ID, LIST_MAIN_HINT_DIVIDER_HEIGHT_KNOB_ID,
    LIST_MAIN_HINT_EXAMPLES_GROUP_GAP_KNOB_ID, LIST_MAIN_HINT_EXAMPLE_LABEL_FONT_SIZE_KNOB_ID,
    LIST_MAIN_HINT_EXAMPLE_ROW_GAP_KNOB_ID, LIST_MAIN_HINT_FORM_BG_ALPHA_KNOB_ID,
    LIST_MAIN_HINT_FORM_BORDER_ALPHA_KNOB_ID, LIST_MAIN_HINT_FORM_FOCUSED_BG_ALPHA_KNOB_ID,
    LIST_MAIN_HINT_FORM_FOCUSED_BORDER_ALPHA_KNOB_ID, LIST_MAIN_HINT_FORM_INPUT_FONT_SIZE_KNOB_ID,
    LIST_MAIN_HINT_FORM_LABEL_ALPHA_KNOB_ID, LIST_MAIN_HINT_FORM_LABEL_FONT_SIZE_KNOB_ID,
    LIST_MAIN_HINT_FORM_VALUE_ALPHA_KNOB_ID, LIST_MAIN_HINT_FORM_VALUE_FONT_SIZE_KNOB_ID,
    LIST_MAIN_HINT_FRAGMENT_ROLE_BG_ALPHA_KNOB_ID, LIST_MAIN_HINT_FRAGMENT_ROLE_WIDTH_KNOB_ID,
    LIST_MAIN_HINT_FRAGMENT_ROWS_GAP_KNOB_ID, LIST_MAIN_HINT_ROWS_GAP_KNOB_ID,
    LIST_MAIN_HINT_ROW_LABEL_WIDTH_KNOB_ID, LIST_MAIN_HINT_STATUS_CHIP_GAP_KNOB_ID,
    LIST_MAIN_HINT_TITLE_FONT_SIZE_KNOB_ID, LIST_MAIN_HINT_WARNING_BG_ALPHA_KNOB_ID,
    LIST_MAIN_HINT_WARNING_BORDER_ALPHA_KNOB_ID, LIST_SCROLLBAR_WIDTH_KNOB_ID,
    LIST_SECTION_GAP_KNOB_ID, LIST_SECTION_PADDING_X_KNOB_ID,
    LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID, METADATA_ALPHA_KNOB_ID,
    METADATA_BADGE_PADDING_X_KNOB_ID, METADATA_BADGE_PADDING_Y_KNOB_ID,
    METADATA_BADGE_RADIUS_KNOB_ID, ROW_HOVER_FILL_ALPHA_KNOB_ID, ROW_INNER_PADDING_X_KNOB_ID,
    ROW_SELECTED_NAME_UNDERLINE_PADDING_BOTTOM_KNOB_ID, ROW_SELECTED_NAME_UNDERLINE_WIDTH_KNOB_ID,
    SEARCH_FONT_SIZE_KNOB_ID, SEARCH_HEIGHT_KNOB_ID, STYLE_KNOBS,
};
use std::sync::{Mutex, OnceLock};

fn runtime_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("runtime style test mutex should not be poisoned")
}

#[test]
fn main_menu_def_applies_runtime_search_height_override() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let variant = MainMenuThemeVariant::InfoBarBase;
    let base = variant.base_def();
    let requested = base.search.height + 11.0;

    let change = runtime_overrides::set_value(SEARCH_HEIGHT_KNOB_ID, StyleValue::Number(requested))
        .expect("search height knob should exist");

    assert_eq!(change.applied, StyleValue::Number(requested));
    assert_eq!(variant.base_def().search.height, base.search.height);
    assert_eq!(variant.def().search.height, requested);

    runtime_overrides::reset_all();
    assert_eq!(variant.def().search.height, base.search.height);
}

#[test]
fn saved_main_window_style_values_are_project_defaults() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let base = MainMenuThemeVariant::InfoBarBase.base_def();

    assert_eq!(base.shell.header_padding_x, 2.0);
    assert_eq!(base.shell.header_padding_y, 4.0);
    assert_eq!(base.shell.header_gap, 2.0);
    assert_eq!(base.search.height, 26.0);
    assert_eq!(base.search.text_inset_x, 16.0);
    assert_eq!(base.list.item_height, 44.0);
    assert_eq!(base.list.section_header_height, 28.0);
    assert_eq!(base.list.first_section_header_height, 28.0);
    assert_eq!(base.list.footer_reveal_clearance_height, 0.0);
    assert_eq!(base.footer.metrics.height_px, 32.0);
    assert_eq!(base.footer.metrics.item_gap_px, 2.0);
    assert_eq!(base.footer.metrics.content_gap, 4.0);
    assert_eq!(base.footer.metrics.run_button_padding_x, 12.0);
    assert_eq!(base.footer.metrics.button_radius, 6.0);
    assert_eq!(base.footer.metrics.keycap_padding_x, 0.0);
    assert_eq!(base.footer.metrics.keycap_padding_y, 0.0);
    assert_eq!(base.footer.metrics.label_font_size, 13.0);
    assert_eq!(base.footer.metrics.font_weight, FontWeight(400.0));
    assert_eq!(base.footer.metrics.keycap_font_size, 11.0);
    assert_eq!(base.footer.metrics.keycap_height, 20.0);
    assert_eq!(base.footer.divider_alpha, 20);
    assert_eq!(base.footer.button.border_alpha, 50);
    assert_eq!(base.footer.button.hover, 0x10);
    assert_eq!(base.footer.button.hover_border_alpha, 0x57);
    assert_eq!(base.footer.button.hover_text_alpha, 0xff);
    assert_eq!(base.footer.button.hover_glyph_alpha, 0xff);
    assert_eq!(base.row.outer_padding_y, 0.0);
    assert_eq!(base.row.radius, 14.0);
    assert_eq!(base.typography.name_font_size, 14.0);
    assert_eq!(base.typography.name_line_height, 16.0);
    assert_eq!(base.header_info_bar.key_opacity, 0.5);
    assert_eq!(base.header_info_bar.pill_padding_x, 6.0);
    assert_eq!(base.header_info_bar.pill_padding_y, 0.0);
    // Header context chips share the footer action buttons' hover-pill
    // radius and hovered keycap-border alpha (canonical hover button style).
    assert_eq!(
        base.header_info_bar.pill_radius,
        base.footer.metrics.button_radius
    );
    assert_eq!(base.header_info_bar.pill_radius, 6.0);
    assert_eq!(base.header_info_bar.pill_hover_bg_alpha, 0x10);
    assert_eq!(
        base.header_info_bar.pill_hover_border_alpha,
        base.footer.button.hover_border_alpha
    );
    assert_eq!(base.header_info_bar.pill_hover_border_alpha, 0x57);
    assert_eq!(base.header_info_bar.pill_hover_text_alpha, 0xff);
    assert_eq!(base.header_info_bar.pill_hover_key_alpha, 0xff);
    assert_eq!(base.header_info_bar.context_edge_outset_x, 8.0);
    assert_eq!(base.icon.container_size, 20.0);
}

#[test]
fn saved_agent_chat_style_values_are_project_defaults() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let base = base_agent_chat_style();

    assert_eq!(base.transcript.row_padding_x, 16.0);
    assert_eq!(base.collapsible.thought_header_opacity, 0.75);
    assert_eq!(base.collapsible.tool_header_opacity, 0.75);
    assert_eq!(base.collapsible.status_opacity, 0.50);
    assert_eq!(base.collapsible.thought_border_alpha, 127.0);
    assert_eq!(base.collapsible.tool_border_alpha, 127.0);
    assert_eq!(base.error.bg_alpha, 50.0);

    assert_eq!(base.markdown.code_block_bg_alpha, 160.0);
    assert_eq!(base.markdown.code_block_border_alpha, 64.0);
    assert_eq!(base.markdown.blockquote_padding_x, 12.0);
    assert_eq!(base.markdown.blockquote_padding_y, 6.0);
    assert_eq!(base.markdown.blockquote_radius, 5.0);
    assert_eq!(base.markdown.blockquote_bg_alpha, 16.0);
    assert_eq!(base.markdown.blockquote_border_alpha, 64.0);
}

#[test]
fn saved_confirm_modal_style_values_are_project_defaults() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let base = base_confirm_modal_style();

    assert_eq!(base.shell.padding_x, 16.0);
    assert_eq!(base.shell.padding_y, 16.0);
    assert_eq!(base.shell.gap, 10.0);
    assert_eq!(base.shell.radius, 8.0);
    assert_eq!(base.header.accent_width, 2.0);
    assert_eq!(base.header.accent_height, 14.0);
    assert_eq!(base.header.gap, 8.0);
    assert_eq!(base.actions.button_radius, 6.0);
    assert_eq!(base.actions.padding_x, 4.0);
    assert_eq!(base.actions.edge_padding_x, 10.0);
    assert_eq!(base.actions.padding_y, 2.0);
    assert_eq!(base.actions.content_gap, 4.0);
}

#[test]
fn runtime_search_height_override_is_clamped_and_generation_counted() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let before_generation = runtime_overrides::generation();

    let change = runtime_overrides::set_value(SEARCH_HEIGHT_KNOB_ID, StyleValue::Number(10_000.0))
        .expect("search height knob should exist");

    let knob = STYLE_KNOBS
        .iter()
        .find(|knob| knob.id == SEARCH_HEIGHT_KNOB_ID)
        .expect("search height knob should be cataloged");
    assert_eq!(change.applied, StyleValue::Number(knob.max));
    assert!(change.generation > before_generation);
    assert_eq!(
        runtime_overrides::current_value(SEARCH_HEIGHT_KNOB_ID),
        Some(StyleValue::Number(knob.max))
    );

    runtime_overrides::reset_all();
}

#[test]
fn runtime_catalog_overrides_representative_main_window_geometry() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let variant = MainMenuThemeVariant::InfoBarBase;
    let base = variant.base_def();

    runtime_overrides::set_value(LIST_ITEM_HEIGHT_KNOB_ID, StyleValue::Number(54.0))
        .expect("list item height knob should exist");
    runtime_overrides::set_value(LIST_ITEM_OUTER_PADDING_Y_KNOB_ID, StyleValue::Number(0.0))
        .expect("list item outer padding y knob should exist");
    runtime_overrides::set_value(LIST_ITEM_INNER_PADDING_Y_KNOB_ID, StyleValue::Number(1.0))
        .expect("list item inner padding y knob should exist");
    runtime_overrides::set_value(SEARCH_FONT_SIZE_KNOB_ID, StyleValue::Number(21.0))
        .expect("search font size knob should exist");
    runtime_overrides::set_value(LIST_SCROLLBAR_WIDTH_KNOB_ID, StyleValue::Number(18.0))
        .expect("list scrollbar width knob should exist");
    runtime_overrides::set_value(ROW_INNER_PADDING_X_KNOB_ID, StyleValue::Number(22.0))
        .expect("row inner padding knob should exist");
    runtime_overrides::set_value(ROW_HOVER_FILL_ALPHA_KNOB_ID, StyleValue::Number(77.0))
        .expect("row hover alpha knob should exist");
    runtime_overrides::set_value(METADATA_ALPHA_KNOB_ID, StyleValue::Number(88.0))
        .expect("metadata alpha knob should exist");
    runtime_overrides::set_value(METADATA_BADGE_PADDING_X_KNOB_ID, StyleValue::Number(9.0))
        .expect("metadata badge padding x knob should exist");
    runtime_overrides::set_value(METADATA_BADGE_PADDING_Y_KNOB_ID, StyleValue::Number(3.0))
        .expect("metadata badge padding y knob should exist");
    runtime_overrides::set_value(METADATA_BADGE_RADIUS_KNOB_ID, StyleValue::Number(11.0))
        .expect("metadata badge radius knob should exist");
    runtime_overrides::set_value(FOOTER_SIDE_INSET_KNOB_ID, StyleValue::Number(12.0))
        .expect("footer side inset knob should exist");
    runtime_overrides::set_value(FOOTER_HEIGHT_KNOB_ID, StyleValue::Number(40.0))
        .expect("footer height knob should exist");
    runtime_overrides::set_value(
        FOOTER_BUTTON_HOVER_BG_ALPHA_KNOB_ID,
        StyleValue::Number(35.0),
    )
    .expect("footer hover bg alpha knob should exist");
    runtime_overrides::set_value(
        FOOTER_BUTTON_HOVER_BORDER_ALPHA_KNOB_ID,
        StyleValue::Number(99.0),
    )
    .expect("footer hover border alpha knob should exist");
    runtime_overrides::set_value(
        FOOTER_BUTTON_HOVER_TEXT_ALPHA_KNOB_ID,
        StyleValue::Number(188.0),
    )
    .expect("footer hover text alpha knob should exist");
    runtime_overrides::set_value(
        FOOTER_BUTTON_HOVER_GLYPH_ALPHA_KNOB_ID,
        StyleValue::Number(177.0),
    )
    .expect("footer hover glyph alpha knob should exist");
    runtime_overrides::set_value(FOOTER_FONT_WEIGHT_KNOB_ID, StyleValue::Number(475.0))
        .expect("footer font weight knob should exist");
    runtime_overrides::set_value(FOOTER_KEYCAP_HEIGHT_KNOB_ID, StyleValue::Number(24.0))
        .expect("footer keycap height knob should exist");
    runtime_overrides::set_value(FOOTER_KEY_GLYPH_NUDGE_Y_KNOB_ID, StyleValue::Number(1.5))
        .expect("footer key glyph nudge knob should exist");
    runtime_overrides::set_value(FOOTER_RETURN_GLYPH_NUDGE_Y_KNOB_ID, StyleValue::Number(2.0))
        .expect("footer return glyph nudge knob should exist");
    runtime_overrides::set_value(
        FOOTER_SEMICOLON_GLYPH_NUDGE_Y_KNOB_ID,
        StyleValue::Number(-1.5),
    )
    .expect("footer semicolon glyph nudge knob should exist");
    runtime_overrides::set_value(FOOTER_RUN_SLOT_MIN_WIDTH_KNOB_ID, StyleValue::Number(104.0))
        .expect("footer run slot min width knob should exist");
    runtime_overrides::set_value(FOOTER_RUN_SLOT_MAX_WIDTH_KNOB_ID, StyleValue::Number(260.0))
        .expect("footer run slot max width knob should exist");
    runtime_overrides::set_value(FOOTER_ACTIONS_SLOT_WIDTH_KNOB_ID, StyleValue::Number(106.0))
        .expect("footer actions slot width knob should exist");
    runtime_overrides::set_value(FOOTER_AI_SLOT_WIDTH_KNOB_ID, StyleValue::Number(64.0))
        .expect("footer ai slot width knob should exist");
    runtime_overrides::set_value(
        FOOTER_PASTE_RESPONSE_SLOT_WIDTH_KNOB_ID,
        StyleValue::Number(156.0),
    )
    .expect("footer paste response slot width knob should exist");
    runtime_overrides::set_value(
        HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID,
        StyleValue::Number(12.0),
    )
    .expect("header context edge outset knob should exist");
    runtime_overrides::set_value(
        HEADER_INFO_PILL_HOVER_BG_ALPHA_KNOB_ID,
        StyleValue::Number(42.0),
    )
    .expect("header hover bg alpha knob should exist");
    runtime_overrides::set_value(
        HEADER_INFO_PILL_HOVER_BORDER_ALPHA_KNOB_ID,
        StyleValue::Number(77.0),
    )
    .expect("header hover border alpha knob should exist");
    runtime_overrides::set_value(
        HEADER_INFO_PILL_HOVER_TEXT_ALPHA_KNOB_ID,
        StyleValue::Number(202.0),
    )
    .expect("header hover text alpha knob should exist");
    runtime_overrides::set_value(
        HEADER_INFO_PILL_HOVER_KEY_ALPHA_KNOB_ID,
        StyleValue::Number(166.0),
    )
    .expect("header hover key alpha knob should exist");
    runtime_overrides::set_value(LIST_SECTION_PADDING_X_KNOB_ID, StyleValue::Number(28.0))
        .expect("list section padding x knob should exist");
    runtime_overrides::set_value(LIST_SECTION_GAP_KNOB_ID, StyleValue::Number(10.0))
        .expect("list section gap knob should exist");
    runtime_overrides::set_value(
        LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID,
        StyleValue::Number(44.0),
    )
    .expect("source status row height knob should exist");
    runtime_overrides::set_value(
        ROW_SELECTED_NAME_UNDERLINE_WIDTH_KNOB_ID,
        StyleValue::Number(3.0),
    )
    .expect("selected name underline width knob should exist");
    runtime_overrides::set_value(
        ROW_SELECTED_NAME_UNDERLINE_PADDING_BOTTOM_KNOB_ID,
        StyleValue::Number(2.0),
    )
    .expect("selected name underline padding bottom knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_CHIP_PADDING_X_KNOB_ID,
        StyleValue::Number(13.0),
    )
    .expect("main hint chip padding x knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_ROW_LABEL_WIDTH_KNOB_ID,
        StyleValue::Number(96.0),
    )
    .expect("main hint row label width knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FRAGMENT_ROLE_WIDTH_KNOB_ID,
        StyleValue::Number(112.0),
    )
    .expect("main hint fragment role width knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_CHIP_BORDER_ALPHA_KNOB_ID,
        StyleValue::Number(101.0),
    )
    .expect("main hint chip border alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FRAGMENT_ROLE_BG_ALPHA_KNOB_ID,
        StyleValue::Number(19.0),
    )
    .expect("main hint fragment role bg alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_STATUS_CHIP_GAP_KNOB_ID,
        StyleValue::Number(11.0),
    )
    .expect("main hint status chip gap knob should exist");
    runtime_overrides::set_value(LIST_MAIN_HINT_ROWS_GAP_KNOB_ID, StyleValue::Number(12.0))
        .expect("main hint rows gap knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FRAGMENT_ROWS_GAP_KNOB_ID,
        StyleValue::Number(13.0),
    )
    .expect("main hint fragment rows gap knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_WARNING_BORDER_ALPHA_KNOB_ID,
        StyleValue::Number(102.0),
    )
    .expect("main hint warning border alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_WARNING_BG_ALPHA_KNOB_ID,
        StyleValue::Number(96.0),
    )
    .expect("main hint warning bg alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_DIVIDER_HEIGHT_KNOB_ID,
        StyleValue::Number(4.0),
    )
    .expect("main hint divider height knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_EXAMPLES_GROUP_GAP_KNOB_ID,
        StyleValue::Number(10.0),
    )
    .expect("main hint examples group gap knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_EXAMPLE_ROW_GAP_KNOB_ID,
        StyleValue::Number(9.0),
    )
    .expect("main hint example row gap knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_FOCUSED_BORDER_ALPHA_KNOB_ID,
        StyleValue::Number(210.0),
    )
    .expect("main hint form focused border alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_BORDER_ALPHA_KNOB_ID,
        StyleValue::Number(111.0),
    )
    .expect("main hint form border alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_FOCUSED_BG_ALPHA_KNOB_ID,
        StyleValue::Number(74.0),
    )
    .expect("main hint form focused bg alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_BG_ALPHA_KNOB_ID,
        StyleValue::Number(37.0),
    )
    .expect("main hint form bg alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_LABEL_ALPHA_KNOB_ID,
        StyleValue::Number(143.0),
    )
    .expect("main hint form label alpha knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_VALUE_ALPHA_KNOB_ID,
        StyleValue::Number(244.0),
    )
    .expect("main hint form value alpha knob should exist");
    runtime_overrides::set_value(
        LIST_INLINE_CALC_SELECTED_OVERLAY_MIN_ALPHA_KNOB_ID,
        StyleValue::Number(51.0),
    )
    .expect("inline calc selected overlay min alpha knob should exist");
    runtime_overrides::set_value(
        LIST_INLINE_CALC_SELECTED_HINT_ALPHA_KNOB_ID,
        StyleValue::Number(201.0),
    )
    .expect("inline calc selected hint alpha knob should exist");
    runtime_overrides::set_value(
        LIST_INLINE_CALC_HINT_ALPHA_KNOB_ID,
        StyleValue::Number(122.0),
    )
    .expect("inline calc hint alpha knob should exist");
    runtime_overrides::set_value(
        LIST_INLINE_CALC_RESULT_FONT_SIZE_KNOB_ID,
        StyleValue::Number(18.0),
    )
    .expect("inline calc result font size knob should exist");
    runtime_overrides::set_value(
        LIST_INLINE_CALC_HINT_FONT_SIZE_KNOB_ID,
        StyleValue::Number(11.0),
    )
    .expect("inline calc hint font size knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_TITLE_FONT_SIZE_KNOB_ID,
        StyleValue::Number(20.0),
    )
    .expect("main hint title font size knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_BODY_FONT_SIZE_KNOB_ID,
        StyleValue::Number(14.0),
    )
    .expect("main hint body font size knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_EXAMPLE_LABEL_FONT_SIZE_KNOB_ID,
        StyleValue::Number(13.0),
    )
    .expect("main hint example label font size knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_LABEL_FONT_SIZE_KNOB_ID,
        StyleValue::Number(13.0),
    )
    .expect("main hint form label font size knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_INPUT_FONT_SIZE_KNOB_ID,
        StyleValue::Number(17.0),
    )
    .expect("main hint form input font size knob should exist");
    runtime_overrides::set_value(
        LIST_MAIN_HINT_FORM_VALUE_FONT_SIZE_KNOB_ID,
        StyleValue::Number(15.0),
    )
    .expect("main hint form value font size knob should exist");

    let def = variant.def();
    assert_eq!(variant.base_def().list.item_height, base.list.item_height);
    assert_eq!(def.list.item_height, 54.0);
    assert_eq!(def.row.outer_padding_y, 0.0);
    assert_eq!(def.row.inner_padding_y, 1.0);
    assert_eq!(def.search.font_size, 21.0);
    assert_eq!(def.list.scrollbar_width, 18.0);
    assert_eq!(def.row.inner_padding_x, 22.0);
    assert_eq!(def.row.hover_fill_alpha, 77);
    assert_eq!(def.metadata.metadata_alpha, 88);
    assert_eq!(def.metadata.badge_padding_x, 9.0);
    assert_eq!(def.metadata.badge_padding_y, 3.0);
    assert_eq!(def.metadata.badge_radius, 11.0);
    assert_eq!(def.footer.metrics.side_inset_px, 12.0);
    assert_eq!(def.footer.metrics.height_px, 40.0);
    assert_eq!(def.footer.button.hover, 35);
    assert_eq!(def.footer.button.hover_border_alpha, 99);
    assert_eq!(def.footer.button.hover_text_alpha, 188);
    assert_eq!(def.footer.button.hover_glyph_alpha, 177);
    assert_eq!(def.footer.metrics.font_weight, FontWeight(475.0));
    assert_eq!(def.footer.metrics.keycap_height, 24.0);
    assert_eq!(def.footer.metrics.key_glyph_nudge_y, 1.5);
    assert_eq!(def.footer.metrics.return_glyph_nudge_y, 2.0);
    assert_eq!(def.footer.metrics.semicolon_glyph_nudge_y, -1.5);
    assert_eq!(def.footer.metrics.run_slot_min_width, 104.0);
    assert_eq!(def.footer.metrics.run_slot_max_width, 260.0);
    assert_eq!(def.footer.metrics.actions_slot_width, 106.0);
    assert_eq!(def.footer.metrics.ai_slot_width, 64.0);
    assert_eq!(def.footer.metrics.paste_response_slot_width, 156.0);
    assert_eq!(def.header_info_bar.context_edge_outset_x, 12.0);
    assert_eq!(def.header_info_bar.pill_hover_bg_alpha, 42);
    assert_eq!(def.header_info_bar.pill_hover_border_alpha, 77);
    assert_eq!(def.header_info_bar.pill_hover_text_alpha, 202);
    assert_eq!(def.header_info_bar.pill_hover_key_alpha, 166);
    assert_eq!(def.list.section_padding_x, 28.0);
    assert_eq!(def.list.section_gap, 10.0);
    assert_eq!(def.list.source_status_row_height, 44.0);
    assert_eq!(def.list.main_hint_chip_padding_x, 13.0);
    assert_eq!(def.list.main_hint_row_label_width, 96.0);
    assert_eq!(def.list.main_hint_fragment_role_width, 112.0);
    assert_eq!(def.list.main_hint_chip_border_alpha, 101);
    assert_eq!(def.list.main_hint_fragment_role_bg_alpha, 19);
    assert_eq!(def.list.main_hint_status_chip_gap, 11.0);
    assert_eq!(def.list.main_hint_rows_gap, 12.0);
    assert_eq!(def.list.main_hint_fragment_rows_gap, 13.0);
    assert_eq!(def.list.main_hint_warning_border_alpha, 102);
    assert_eq!(def.list.main_hint_warning_bg_alpha, 96);
    assert_eq!(def.list.main_hint_divider_height, 4.0);
    assert_eq!(def.list.main_hint_examples_group_gap, 10.0);
    assert_eq!(def.list.main_hint_example_row_gap, 9.0);
    assert_eq!(def.list.main_hint_form_focused_border_alpha, 210);
    assert_eq!(def.list.main_hint_form_border_alpha, 111);
    assert_eq!(def.list.main_hint_form_focused_bg_alpha, 74);
    assert_eq!(def.list.main_hint_form_bg_alpha, 37);
    assert_eq!(def.list.main_hint_form_label_alpha, 143);
    assert_eq!(def.list.main_hint_form_value_alpha, 244);
    assert_eq!(def.list.inline_calc_selected_overlay_min_alpha, 51);
    assert_eq!(def.list.inline_calc_selected_hint_alpha, 201);
    assert_eq!(def.list.inline_calc_hint_alpha, 122);
    assert_eq!(def.list.inline_calc_result_font_size, 18.0);
    assert_eq!(def.list.inline_calc_hint_font_size, 11.0);
    assert_eq!(def.list.main_hint_title_font_size, 20.0);
    assert_eq!(def.list.main_hint_body_font_size, 14.0);
    assert_eq!(def.list.main_hint_example_label_font_size, 13.0);
    assert_eq!(def.list.main_hint_form_label_font_size, 13.0);
    assert_eq!(def.list.main_hint_form_input_font_size, 17.0);
    assert_eq!(def.list.main_hint_form_value_font_size, 15.0);
    assert_eq!(def.row.selected_name_underline_width, 3.0);
    assert_eq!(def.row.selected_name_underline_padding_bottom, 2.0);

    let metrics =
        script_kit_gpui::list_item::ListItemMetricsOverride::from_main_menu_theme(variant);
    assert_eq!(metrics.source_status_row_height, 44.0);
    assert_eq!(metrics.row_selected_name_underline_width, 3.0);
    assert_eq!(metrics.row_selected_name_underline_padding_bottom, 2.0);
    assert_eq!(metrics.badge_padding_x, 9.0);
    assert_eq!(metrics.badge_padding_y, 3.0);
    assert_eq!(metrics.badge_radius, 11.0);
    assert_eq!(metrics.source_font_size, def.metadata.source_font_size);
    assert_eq!(metrics.badge_font_size, def.metadata.badge_font_size);

    runtime_overrides::reset_all();
}

#[test]
fn devtools_numeric_setter_accepts_catalog_control_ids() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();

    let applied = runtime_overrides::set_number_from_devtools("row.innerPaddingX", "18px")
        .expect("row inner padding should be settable through devtools");

    assert_eq!(applied, "row.innerPaddingX=18");
    assert_eq!(
        runtime_overrides::current_value(ROW_INNER_PADDING_X_KNOB_ID),
        Some(StyleValue::Number(18.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.itemOuterPaddingY", "0px")
        .expect("list item outer padding y should be settable through devtools");

    assert_eq!(applied, "list.itemOuterPaddingY=0");
    assert_eq!(
        runtime_overrides::current_value(LIST_ITEM_OUTER_PADDING_Y_KNOB_ID),
        Some(StyleValue::Number(0.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.itemInnerPaddingY", "1px")
        .expect("list item inner padding y should be settable through devtools");

    assert_eq!(applied, "list.itemInnerPaddingY=1");
    assert_eq!(
        runtime_overrides::current_value(LIST_ITEM_INNER_PADDING_Y_KNOB_ID),
        Some(StyleValue::Number(1.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.sourceStatusRowHeight", "44px")
        .expect("source status row height should be settable through devtools");

    assert_eq!(applied, "list.sourceStatusRowHeight=44");
    assert_eq!(
        runtime_overrides::current_value(LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID),
        Some(StyleValue::Number(44.0))
    );

    let applied =
        runtime_overrides::set_number_from_devtools("row.selectedNameUnderlineWidth", "3px")
            .expect("selected name underline width should be settable through devtools");

    assert_eq!(applied, "row.selectedNameUnderlineWidth=3");
    assert_eq!(
        runtime_overrides::current_value(ROW_SELECTED_NAME_UNDERLINE_WIDTH_KNOB_ID),
        Some(StyleValue::Number(3.0))
    );

    let applied = runtime_overrides::set_number_from_devtools(
        "row.selectedNameUnderlinePaddingBottom",
        "2px",
    )
    .expect("selected name underline padding bottom should be settable through devtools");

    assert_eq!(applied, "row.selectedNameUnderlinePaddingBottom=2");
    assert_eq!(
        runtime_overrides::current_value(ROW_SELECTED_NAME_UNDERLINE_PADDING_BOTTOM_KNOB_ID),
        Some(StyleValue::Number(2.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("search.fontSize", "21px")
        .expect("search font size should be settable through devtools");

    assert_eq!(applied, "search.fontSize=21");
    assert_eq!(
        runtime_overrides::current_value(SEARCH_FONT_SIZE_KNOB_ID),
        Some(StyleValue::Number(21.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.scrollbarWidth", "18px")
        .expect("list scrollbar width should be settable through devtools");

    assert_eq!(applied, "list.scrollbarWidth=18");
    assert_eq!(
        runtime_overrides::current_value(LIST_SCROLLBAR_WIDTH_KNOB_ID),
        Some(StyleValue::Number(18.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.mainHintChipPaddingX", "13px")
        .expect("main hint chip padding x should be settable through devtools");

    assert_eq!(applied, "list.mainHintChipPaddingX=13");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_CHIP_PADDING_X_KNOB_ID),
        Some(StyleValue::Number(13.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.mainHintDividerHeight", "4px")
        .expect("main hint divider height should be settable through devtools");

    assert_eq!(applied, "list.mainHintDividerHeight=4");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_DIVIDER_HEIGHT_KNOB_ID),
        Some(StyleValue::Number(4.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.mainHintWarningBgAlpha", "96")
        .expect("main hint warning bg alpha should be settable through devtools");

    assert_eq!(applied, "list.mainHintWarningBgAlpha=96");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_WARNING_BG_ALPHA_KNOB_ID),
        Some(StyleValue::Number(96.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.mainHintExampleRowGap", "9px")
        .expect("main hint example row gap should be settable through devtools");

    assert_eq!(applied, "list.mainHintExampleRowGap=9");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_EXAMPLE_ROW_GAP_KNOB_ID),
        Some(StyleValue::Number(9.0))
    );

    let applied =
        runtime_overrides::set_number_from_devtools("list.mainHintFormFocusedBorderAlpha", "210")
            .expect("main hint form focused border alpha should be settable through devtools");

    assert_eq!(applied, "list.mainHintFormFocusedBorderAlpha=210");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_FORM_FOCUSED_BORDER_ALPHA_KNOB_ID),
        Some(StyleValue::Number(210.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.mainHintFormBgAlpha", "37")
        .expect("main hint form bg alpha should be settable through devtools");

    assert_eq!(applied, "list.mainHintFormBgAlpha=37");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_FORM_BG_ALPHA_KNOB_ID),
        Some(StyleValue::Number(37.0))
    );

    let applied =
        runtime_overrides::set_number_from_devtools("list.inlineCalcSelectedOverlayMinAlpha", "51")
            .expect("inline calc overlay alpha should be settable through devtools");

    assert_eq!(applied, "list.inlineCalcSelectedOverlayMinAlpha=51");
    assert_eq!(
        runtime_overrides::current_value(LIST_INLINE_CALC_SELECTED_OVERLAY_MIN_ALPHA_KNOB_ID),
        Some(StyleValue::Number(51.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("list.mainHintTitleFontSize", "20px")
        .expect("main hint title font size should be settable through devtools");

    assert_eq!(applied, "list.mainHintTitleFontSize=20");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_TITLE_FONT_SIZE_KNOB_ID),
        Some(StyleValue::Number(20.0))
    );

    let applied =
        runtime_overrides::set_number_from_devtools("list.mainHintFormInputFontSize", "17px")
            .expect("main hint form input font size should be settable through devtools");

    assert_eq!(applied, "list.mainHintFormInputFontSize=17");
    assert_eq!(
        runtime_overrides::current_value(LIST_MAIN_HINT_FORM_INPUT_FONT_SIZE_KNOB_ID),
        Some(StyleValue::Number(17.0))
    );

    let applied =
        runtime_overrides::set_number_from_devtools("list.inlineCalcResultFontSize", "18px")
            .expect("inline calc result font size should be settable through devtools");

    assert_eq!(applied, "list.inlineCalcResultFontSize=18");
    assert_eq!(
        runtime_overrides::current_value(LIST_INLINE_CALC_RESULT_FONT_SIZE_KNOB_ID),
        Some(StyleValue::Number(18.0))
    );

    let applied =
        runtime_overrides::set_number_from_devtools("list.inlineCalcHintFontSize", "11px")
            .expect("inline calc hint font size should be settable through devtools");

    assert_eq!(applied, "list.inlineCalcHintFontSize=11");
    assert_eq!(
        runtime_overrides::current_value(LIST_INLINE_CALC_HINT_FONT_SIZE_KNOB_ID),
        Some(StyleValue::Number(11.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("footer.keycapHeight", "24px")
        .expect("footer keycap height should be settable through devtools");

    assert_eq!(applied, "footer.keycapHeight=24");
    assert_eq!(
        runtime_overrides::current_value(FOOTER_KEYCAP_HEIGHT_KNOB_ID),
        Some(StyleValue::Number(24.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("footer.height", "40px")
        .expect("footer height should be settable through devtools");

    assert_eq!(applied, "footer.height=40");
    assert_eq!(
        runtime_overrides::current_value(FOOTER_HEIGHT_KNOB_ID),
        Some(StyleValue::Number(40.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("footer.fontWeight", "475")
        .expect("footer font weight should be settable through devtools");

    assert_eq!(applied, "footer.fontWeight=475");
    assert_eq!(
        runtime_overrides::current_value(FOOTER_FONT_WEIGHT_KNOB_ID),
        Some(StyleValue::Number(475.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("footer.runSlotMinWidth", "104px")
        .expect("footer run slot min width should be settable through devtools");

    assert_eq!(applied, "footer.runSlotMinWidth=104");
    assert_eq!(
        runtime_overrides::current_value(FOOTER_RUN_SLOT_MIN_WIDTH_KNOB_ID),
        Some(StyleValue::Number(104.0))
    );

    let applied = runtime_overrides::set_number_from_devtools("footer.aiSlotWidth", "64px")
        .expect("footer ai slot width should be settable through devtools");

    assert_eq!(applied, "footer.aiSlotWidth=64");
    assert_eq!(
        runtime_overrides::current_value(FOOTER_AI_SLOT_WIDTH_KNOB_ID),
        Some(StyleValue::Number(64.0))
    );

    runtime_overrides::reset_all();
}

#[test]
fn devtools_copy_setter_updates_effective_main_input_placeholder() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();

    let applied =
        runtime_overrides::set_copy_from_devtools("main.input.placeholder", "Tune live copy")
            .expect("main input placeholder should be settable through devtools");

    assert_eq!(applied, "main.input.placeholder=Tune live copy");
    assert_eq!(
        runtime_overrides::effective_main_input_placeholder(),
        "Tune live copy"
    );

    runtime_overrides::reset_all();
}

#[test]
fn devtools_actions_setter_updates_current_actions_popup_theme() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let base = script_kit_gpui::designs::base_actions_popup_theme();

    let applied =
        runtime_overrides::set_actions_number_from_devtools("actions.list.rowHeight", "64px")
            .expect("actions row height should be settable through devtools");

    assert_eq!(applied, "actions.list.rowHeight=64");
    assert_eq!(
        script_kit_gpui::designs::current_actions_popup_theme()
            .list
            .row_height,
        64.0
    );
    assert_eq!(
        script_kit_gpui::designs::base_actions_popup_theme()
            .list
            .row_height,
        base.list.row_height
    );

    let applied =
        runtime_overrides::set_actions_number_from_devtools("actions.search.paddingX", "18px")
            .expect("actions search padding should be settable through devtools");
    assert_eq!(applied, "actions.search.paddingX=18");
    assert_eq!(
        script_kit_gpui::designs::current_actions_popup_theme()
            .search
            .padding_x,
        18.0
    );

    let applied =
        runtime_overrides::set_actions_number_from_devtools("actions.list.paddingTop", "11px")
            .expect("actions list top padding should be settable through devtools");
    assert_eq!(applied, "actions.list.paddingTop=11");
    assert_eq!(
        script_kit_gpui::designs::current_actions_popup_theme()
            .list
            .padding_top,
        11.0
    );

    let applied =
        runtime_overrides::set_actions_number_from_devtools("actions.section.fontWeight", "700")
            .expect("actions section font weight should be settable through devtools");
    assert_eq!(applied, "actions.section.fontWeight=700");
    assert_eq!(
        script_kit_gpui::designs::current_actions_popup_theme()
            .section
            .font_weight
            .0,
        700.0
    );

    runtime_overrides::reset_all();
}

#[test]
fn devtools_agent_chat_setter_updates_markdown_code_and_blockquote_styles() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();

    let applied = runtime_overrides::set_agent_chat_number_from_devtools(
        "agentChat.markdown.codeBlockBgAlpha",
        "190",
    )
    .expect("code block background alpha should be settable through devtools");
    assert_eq!(applied, "agentChat.markdown.codeBlockBgAlpha=190");

    let applied = runtime_overrides::set_agent_chat_number_from_devtools(
        "agentChat.markdown.blockquoteBgAlpha",
        "42",
    )
    .expect("blockquote background alpha should be settable through devtools");
    assert_eq!(applied, "agentChat.markdown.blockquoteBgAlpha=42");

    let effective = runtime_overrides::effective_agent_chat_style();
    assert_eq!(effective.markdown.code_block_bg_alpha, 190.0);
    assert_eq!(effective.markdown.blockquote_bg_alpha, 42.0);

    runtime_overrides::reset_all();
}

#[test]
fn devtools_confirm_modal_setter_updates_effective_shell_style() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();

    let applied = runtime_overrides::set_confirm_modal_number_from_devtools(
        "confirmModal.shell.paddingX",
        "22",
    )
    .expect("confirm modal shell padding should be settable through devtools");
    assert_eq!(applied, "confirmModal.shell.paddingX=22");

    let applied = runtime_overrides::set_confirm_modal_number_from_devtools(
        "slider:dev-style-tool-confirm-modal:confirmModal.shell.radius",
        "13",
    )
    .expect("confirm modal shell radius should be settable by semantic control id");
    assert_eq!(applied, "confirmModal.shell.radius=13");

    let applied = runtime_overrides::set_confirm_modal_number_from_devtools(
        "input:dev-style-tool-confirm-modal:confirmModal.actions.paddingX",
        "10",
    )
    .expect("confirm modal action padding should be settable by semantic control id");
    assert_eq!(applied, "confirmModal.actions.paddingX=10");

    let applied = runtime_overrides::set_confirm_modal_number_from_devtools(
        "input:dev-style-tool-confirm-modal:confirmModal.actions.edgePaddingX",
        "2",
    )
    .expect("confirm modal action edge padding should be settable by semantic control id");
    assert_eq!(applied, "confirmModal.actions.edgePaddingX=2");

    let effective = runtime_overrides::effective_confirm_modal_style();
    assert_eq!(effective.shell.padding_x, 22.0);
    assert_eq!(effective.shell.radius, 13.0);
    assert_eq!(effective.actions.padding_x, 10.0);
    assert_eq!(effective.actions.edge_padding_x, 2.0);

    let change = runtime_overrides::reset_confirm_modal_value(CONFIRM_MODAL_PADDING_X_KNOB_ID)
        .expect("confirm modal padding should reset");
    assert_eq!(change.applied, StyleValue::Number(16.0));
    let change =
        runtime_overrides::reset_confirm_modal_value(CONFIRM_MODAL_ACTIONS_PADDING_X_KNOB_ID)
            .expect("confirm modal action padding should reset");
    assert_eq!(change.applied, StyleValue::Number(4.0));
    let change =
        runtime_overrides::reset_confirm_modal_value(CONFIRM_MODAL_ACTIONS_EDGE_PADDING_X_KNOB_ID)
            .expect("confirm modal action edge padding should reset");
    assert_eq!(change.applied, StyleValue::Number(10.0));

    runtime_overrides::reset_all();
}

#[test]
fn export_current_settings_includes_agent_readable_overrides_and_effective_values() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    runtime_overrides::set_value(LIST_ITEM_HEIGHT_KNOB_ID, StyleValue::Number(57.0))
        .expect("list item height knob should exist");

    let json = export::current_settings_json();
    assert_eq!(json["schema"], "script-kit-dev-style/v3");
    assert_eq!(json["overrideCount"], 1);
    assert_eq!(json["controls"]["mainWindowStyle"], STYLE_KNOBS.len());
    assert_eq!(json["controls"]["mainWindowCopy"], COPY_CONTROLS.len());
    assert_eq!(
        json["controls"]["actionsPopupStyle"],
        ACTIONS_POPUP_KNOBS.len()
    );
    assert_eq!(
        json["controls"]["confirmModalStyle"],
        CONFIRM_MODAL_KNOBS.len()
    );
    assert!(json["agentPrompt"]
        .as_str()
        .expect("agent prompt should be a string")
        .contains("src/dev_style_tool/catalog.rs"));
    assert!(json["agentPrompt"]
        .as_str()
        .expect("agent prompt should be a string")
        .contains("src/dev_style_tool/actions_popup_catalog.rs"));
    assert!(json["agentPrompt"]
        .as_str()
        .expect("agent prompt should be a string")
        .contains("src/dev_style_tool/confirm_modal_catalog.rs"));
    assert!(json["agentPrompt"]
        .as_str()
        .expect("agent prompt should be a string")
        .contains("src/dev_style_tool/copy_catalog.rs"));
    let main_window_style = &json["surfaces"]["mainWindow"]["style"];
    let main_window_copy = &json["surfaces"]["mainWindow"]["copy"];
    let actions_popup_style = &json["surfaces"]["actionsPopup"]["style"];
    let confirm_modal_style = &json["surfaces"]["confirmModal"]["style"];
    assert!(main_window_style["overrides"]
        .as_array()
        .expect("overrides should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.itemHeight" && entry["value"] == 57.0));
    assert!(main_window_copy["effective"]
        .as_array()
        .expect("copy effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "main.input.placeholder"));
    assert!(actions_popup_style["effective"]
        .as_array()
        .expect("actions popup effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "actions.list.rowHeight"));
    assert!(actions_popup_style["effective"]
        .as_array()
        .expect("actions popup effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "actions.search.paddingX"));
    assert!(actions_popup_style["effective"]
        .as_array()
        .expect("actions popup effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "actions.list.paddingTop"));
    assert!(actions_popup_style["effective"]
        .as_array()
        .expect("actions popup effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "actions.contextHeader.paddingBottom"));
    assert!(confirm_modal_style["effective"]
        .as_array()
        .expect("confirm modal effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "confirmModal.shell.radius"));
    assert!(confirm_modal_style["effective"]
        .as_array()
        .expect("confirm modal effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "confirmModal.actions.paddingX"));
    assert!(confirm_modal_style["effective"]
        .as_array()
        .expect("confirm modal effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "confirmModal.actions.edgePaddingX"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "metadata.badgePaddingX"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.sourceStatusRowHeight"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "row.selectedNameUnderlineWidth"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "row.selectedNameUnderlinePaddingBottom"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintChipPaddingX"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintDividerHeight"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "footer.height"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "footer.fontWeight"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintWarningBgAlpha"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintExampleRowGap"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintFormFocusedBorderAlpha"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintFormBgAlpha"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.inlineCalcSelectedOverlayMinAlpha"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintTitleFontSize"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.mainHintFormInputFontSize"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.inlineCalcResultFontSize"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.inlineCalcHintFontSize"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "footer.keycapHeight"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "footer.runSlotMinWidth"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "footer.aiSlotWidth"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "search.fontSize"));
    assert!(main_window_style["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.scrollbarWidth"));

    assert_eq!(json["controls"]["themeColors"], THEME_COLOR_KNOBS.len());
    assert!(json["agentPrompt"]
        .as_str()
        .expect("agent prompt should be a string")
        .contains("src/dev_style_tool/theme_catalog.rs"));
    let theme_colors = &json["surfaces"]["theme"]["colors"];
    assert!(theme_colors["effective"]
        .as_array()
        .expect("theme colors effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "theme.colors.text.primary"));
    assert!(theme_colors["effective"]
        .as_array()
        .expect("theme colors effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "theme.colors.accent.selected"));

    let markdown = export::current_settings_markdown();
    assert!(markdown.contains("```json"));
    assert!(markdown.contains("\"list.itemHeight\""));

    runtime_overrides::reset_all();
}

#[test]
fn theme_color_override_round_trips_with_undo_redo_and_generation() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    let before_generation = runtime_overrides::generation();

    let change = runtime_overrides::set_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID, 0x112233)
        .expect("theme text primary knob should exist");
    assert_eq!(change.applied, 0x112233);
    assert!(change.generation > before_generation);
    assert_eq!(
        runtime_overrides::current_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID),
        Some(0x112233)
    );
    assert!(runtime_overrides::has_theme_color_overrides());
    assert_eq!(runtime_overrides::theme_color_override_count(), 1);
    assert_eq!(runtime_overrides::history_state().override_count, 1);

    let undo = runtime_overrides::undo_last().expect("theme color set should be undoable");
    assert!(undo.contains("theme.colors.text.primary"));
    assert_eq!(
        runtime_overrides::current_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID),
        None
    );

    let redo = runtime_overrides::redo_last().expect("theme color set should be redoable");
    assert!(redo.contains("theme.colors.text.primary"));
    assert_eq!(
        runtime_overrides::current_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID),
        Some(0x112233)
    );

    let reset = runtime_overrides::reset_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID)
        .expect("theme text primary knob should reset");
    assert_eq!(reset.previous, Some(0x112233));
    assert_eq!(
        runtime_overrides::current_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID),
        None
    );

    runtime_overrides::reset_all();
}

#[test]
fn apply_to_theme_layers_theme_color_overrides_onto_a_theme() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();

    let base = script_kit_gpui::theme::Theme::default();
    let untouched = runtime_overrides::apply_to_theme(base.clone());
    assert_eq!(untouched.colors.ui.border, base.colors.ui.border);

    runtime_overrides::set_theme_color_value(THEME_UI_BORDER_KNOB_ID, 0xAB_CD_EF)
        .expect("theme ui border knob should exist");
    let themed = runtime_overrides::apply_to_theme(base.clone());
    assert_eq!(themed.colors.ui.border, 0xAB_CD_EF);
    assert_eq!(format_theme_color_hex(themed.colors.ui.border), "#ABCDEF");

    runtime_overrides::reset_all();
    let cleared = runtime_overrides::apply_to_theme(base.clone());
    assert_eq!(cleared.colors.ui.border, base.colors.ui.border);
}

#[test]
fn devtools_theme_color_setter_accepts_hex_and_rejects_garbage() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();

    let applied =
        runtime_overrides::set_theme_color_from_devtools("theme.colors.text.primary", "#FBBF24")
            .expect("theme text primary should be settable through devtools");
    assert_eq!(applied, "theme.colors.text.primary=#FBBF24");
    assert_eq!(
        runtime_overrides::current_theme_color_value(THEME_TEXT_PRIMARY_KNOB_ID),
        Some(0xFBBF24)
    );

    let applied = runtime_overrides::set_theme_color_from_devtools(
        "input:dev-style-tool-theme:theme.colors.ui.border",
        "rgb(1, 2, 3)",
    )
    .expect("theme ui border should be settable by semantic control id");
    assert_eq!(applied, "theme.colors.ui.border=#010203");

    assert!(runtime_overrides::set_theme_color_from_devtools(
        "theme.colors.text.primary",
        "not-a-hex"
    )
    .is_err());
    assert!(
        runtime_overrides::set_theme_color_from_devtools("theme.colors.nope", "#112233").is_err()
    );

    runtime_overrides::clear_theme_color_values();
    assert!(!runtime_overrides::has_theme_color_overrides());

    runtime_overrides::reset_all();
}
