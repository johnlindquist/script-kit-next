//! Source-level contract for Notes window snap/bounds independence.

const WINDOW_STATE_SOURCE: &str = include_str!("../src/window_state/mod.rs");
const NOTES_WINDOW_OPS_SOURCE: &str = include_str!("../src/notes/window/window_ops.rs");
const NOTES_KEYBOARD_SOURCE: &str = include_str!("../src/notes/window/keyboard.rs");
const NOTES_INIT_SOURCE: &str = include_str!("../src/notes/window/init.rs");

// doc-anchor-removed: [[tests/notes-acp#Notes multi-display snap session#Notes bounds use the Notes window role]]
#[test]
fn notes_window_persistence_uses_notes_role_not_main_role() {
    for source in [
        NOTES_WINDOW_OPS_SOURCE,
        NOTES_KEYBOARD_SOURCE,
        NOTES_INIT_SOURCE,
    ] {
        assert!(
            source.contains("WindowRole::Notes"),
            "Notes window paths must persist bounds with WindowRole::Notes"
        );
    }
    let combined =
        format!("{NOTES_WINDOW_OPS_SOURCE}\n{NOTES_KEYBOARD_SOURCE}\n{NOTES_INIT_SOURCE}");
    assert!(
        !combined.contains("WindowRole::Main"),
        "Notes window persistence paths must not alias to the main window role"
    );
}

// doc-anchor-removed: [[tests/notes-acp#Notes multi-display snap session#Restored Notes bounds are clamped to live displays]]
#[test]
fn window_state_exposes_display_visibility_and_clamp_helpers_for_restored_bounds() {
    assert!(WINDOW_STATE_SOURCE.contains("pub fn is_bounds_visible("));
    assert!(WINDOW_STATE_SOURCE.contains("pub fn clamp_bounds_to_displays("));
    assert!(WINDOW_STATE_SOURCE.contains("if is_bounds_visible(&saved, displays)"));
    assert!(WINDOW_STATE_SOURCE.contains("clamp_bounds_to_displays(&saved, displays)"));
}
