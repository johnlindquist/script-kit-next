const GEOMETRY: &str = include_str!("../../src/platform/accessibility/geometry.rs");
const AX: &str = include_str!("../../src/platform/accessibility/ax.rs");

#[test]
fn geometry_prefers_caret_then_selection_then_field_then_window() {
    let caret = GEOMETRY.find("caret_bounds").expect("caret");
    let selection = GEOMETRY[caret..]
        .find("selection_bounds")
        .expect("selection")
        + caret;
    let field = GEOMETRY[selection..].find("field_bounds").expect("field") + selection;
    let window = GEOMETRY[field..].find("window_bounds").expect("window") + field;
    assert!(caret < selection && selection < field && field < window);
}

#[test]
fn native_geometry_uses_ax_bounds_position_size_and_window_fallback() {
    for needle in [
        "AXBoundsForRange",
        "AXSelectedTextRange",
        "AXPosition",
        "AXSize",
        "AXWindow",
        "focused_geometry",
    ] {
        assert!(
            AX.contains(needle),
            "missing native geometry needle {needle}"
        );
    }
}
