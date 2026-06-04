use script_kit_gpui::designs::MainMenuThemeVariant;
use script_kit_gpui::dev_style_tool::{
    runtime_overrides, StyleValue, LIST_ITEM_HEIGHT_KNOB_ID, ROW_HOVER_FILL_ALPHA_KNOB_ID,
    ROW_INNER_PADDING_X_KNOB_ID, SEARCH_HEIGHT_KNOB_ID, STYLE_KNOBS,
};

#[test]
fn main_menu_def_applies_runtime_search_height_override() {
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
    runtime_overrides::reset_all();
    let variant = MainMenuThemeVariant::InfoBarBase;
    let base = variant.base_def();

    runtime_overrides::set_value(LIST_ITEM_HEIGHT_KNOB_ID, StyleValue::Number(54.0))
        .expect("list item height knob should exist");
    runtime_overrides::set_value(ROW_INNER_PADDING_X_KNOB_ID, StyleValue::Number(22.0))
        .expect("row inner padding knob should exist");
    runtime_overrides::set_value(ROW_HOVER_FILL_ALPHA_KNOB_ID, StyleValue::Number(77.0))
        .expect("row hover alpha knob should exist");

    let def = variant.def();
    assert_eq!(variant.base_def().list.item_height, base.list.item_height);
    assert_eq!(def.list.item_height, 54.0);
    assert_eq!(def.row.inner_padding_x, 22.0);
    assert_eq!(def.row.hover_fill_alpha, 77);

    runtime_overrides::reset_all();
}

#[test]
fn devtools_numeric_setter_accepts_catalog_control_ids() {
    runtime_overrides::reset_all();

    let applied = runtime_overrides::set_number_from_devtools("row.innerPaddingX", "18px")
        .expect("row inner padding should be settable through devtools");

    assert_eq!(applied, "row.innerPaddingX=18");
    assert_eq!(
        runtime_overrides::current_value(ROW_INNER_PADDING_X_KNOB_ID),
        Some(StyleValue::Number(18.0))
    );

    runtime_overrides::reset_all();
}
