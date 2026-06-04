use script_kit_gpui::designs::MainMenuThemeVariant;
use script_kit_gpui::dev_style_tool::{
    runtime_overrides, StyleValue, SEARCH_HEIGHT_KNOB_ID, STYLE_KNOBS,
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

    let change =
        runtime_overrides::set_value(SEARCH_HEIGHT_KNOB_ID, StyleValue::Number(10_000.0))
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
