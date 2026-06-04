use script_kit_gpui::designs::MainMenuThemeVariant;
use script_kit_gpui::dev_style_tool::{
    export, runtime_overrides, StyleValue, FOOTER_SIDE_INSET_KNOB_ID,
    HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID, HEADER_INFO_VARIATION_BADGE_WIDTH_KNOB_ID,
    LIST_ITEM_HEIGHT_KNOB_ID, LIST_SECTION_GAP_KNOB_ID, LIST_SECTION_PADDING_X_KNOB_ID,
    LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID, METADATA_ALPHA_KNOB_ID,
    METADATA_BADGE_PADDING_X_KNOB_ID, METADATA_BADGE_PADDING_Y_KNOB_ID,
    METADATA_BADGE_RADIUS_KNOB_ID, ROW_HOVER_FILL_ALPHA_KNOB_ID, ROW_INNER_PADDING_X_KNOB_ID,
    SEARCH_HEIGHT_KNOB_ID, STYLE_KNOBS,
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
    runtime_overrides::set_value(
        HEADER_INFO_VARIATION_BADGE_WIDTH_KNOB_ID,
        StyleValue::Number(72.0),
    )
    .expect("header variation badge width knob should exist");
    runtime_overrides::set_value(
        HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID,
        StyleValue::Number(12.0),
    )
    .expect("header context edge outset knob should exist");
    runtime_overrides::set_value(LIST_SECTION_PADDING_X_KNOB_ID, StyleValue::Number(28.0))
        .expect("list section padding x knob should exist");
    runtime_overrides::set_value(LIST_SECTION_GAP_KNOB_ID, StyleValue::Number(10.0))
        .expect("list section gap knob should exist");
    runtime_overrides::set_value(
        LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID,
        StyleValue::Number(44.0),
    )
    .expect("source status row height knob should exist");

    let def = variant.def();
    assert_eq!(variant.base_def().list.item_height, base.list.item_height);
    assert_eq!(def.list.item_height, 54.0);
    assert_eq!(def.row.inner_padding_x, 22.0);
    assert_eq!(def.row.hover_fill_alpha, 77);
    assert_eq!(def.metadata.metadata_alpha, 88);
    assert_eq!(def.metadata.badge_padding_x, 9.0);
    assert_eq!(def.metadata.badge_padding_y, 3.0);
    assert_eq!(def.metadata.badge_radius, 11.0);
    assert_eq!(def.footer.metrics.side_inset_px, 12.0);
    assert_eq!(def.header_info_bar.variation_badge_width_px, 72.0);
    assert_eq!(def.header_info_bar.context_edge_outset_x, 12.0);
    assert_eq!(def.list.section_padding_x, 28.0);
    assert_eq!(def.list.section_gap, 10.0);
    assert_eq!(def.list.source_status_row_height, 44.0);

    let metrics =
        script_kit_gpui::list_item::ListItemMetricsOverride::from_main_menu_theme(variant);
    assert_eq!(metrics.source_status_row_height, 44.0);
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

    let applied = runtime_overrides::set_number_from_devtools("list.sourceStatusRowHeight", "44px")
        .expect("source status row height should be settable through devtools");

    assert_eq!(applied, "list.sourceStatusRowHeight=44");
    assert_eq!(
        runtime_overrides::current_value(LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID),
        Some(StyleValue::Number(44.0))
    );

    runtime_overrides::reset_all();
}

#[test]
fn export_current_settings_includes_agent_readable_overrides_and_effective_values() {
    let _guard = runtime_test_guard();
    runtime_overrides::reset_all();
    runtime_overrides::set_value(LIST_ITEM_HEIGHT_KNOB_ID, StyleValue::Number(57.0))
        .expect("list item height knob should exist");

    let json = export::current_settings_json();
    assert_eq!(json["schema"], "script-kit-main-window-style/v1");
    assert_eq!(json["overrideCount"], 1);
    assert_eq!(json["controls"], STYLE_KNOBS.len());
    assert!(json["agentPrompt"]
        .as_str()
        .expect("agent prompt should be a string")
        .contains("src/dev_style_tool/catalog.rs"));
    assert!(json["overrides"]
        .as_array()
        .expect("overrides should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.itemHeight" && entry["value"] == 57.0));
    assert!(json["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "metadata.badgePaddingX"));
    assert!(json["effective"]
        .as_array()
        .expect("effective should be an array")
        .iter()
        .any(|entry| entry["id"] == "list.sourceStatusRowHeight"));

    let markdown = export::current_settings_markdown();
    assert!(markdown.contains("```json"));
    assert!(markdown.contains("\"list.itemHeight\""));

    runtime_overrides::reset_all();
}
