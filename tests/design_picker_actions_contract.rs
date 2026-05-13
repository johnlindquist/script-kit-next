//! Phase 3 — pin the Design Picker Cmd+K actions catalog.
//!
//! The spec at `.goals/design-variants-overhaul.md` requires the picker's
//! actions dialog to expose a 12-row catalog plus the Done/Revert pair.
//! This test asserts every action id is wired through
//! `Self::design_picker_actions_for_dialog()` so the dispatcher and the UI
//! cannot silently drift apart.

const ACTIONS_SRC: &str = include_str!("../src/render_builtins/actions.rs");
const TOGGLE_SRC: &str = include_str!("../src/app_impl/actions_toggle.rs");

#[test]
fn design_picker_actions_dialog_is_wired_into_dispatcher() {
    assert!(
        TOGGLE_SRC.contains("AppView::DesignPickerView { .. }")
            && TOGGLE_SRC.contains("toggle_design_picker_actions"),
        "AppView::DesignPickerView branch must call toggle_design_picker_actions in dispatch_actions_toggle_for_current_view"
    );
}

#[test]
fn design_picker_actions_catalog_declares_spec_rows() {
    let required_ids = [
        "design_picker_done",
        "design_picker_revert",
        "design_picker_cycle",
        "design_picker_surprise",
        "design_picker_set_default",
        "design_picker_reset_one",
        "design_picker_reset_all",
        "design_picker_toggle_density",
        "design_picker_toggle_vibrancy",
        "design_picker_cycle_accent",
        "design_picker_cycle_font",
        "design_picker_open_storybook",
        "design_picker_copy_snippet",
    ];
    for id in required_ids {
        assert!(
            ACTIONS_SRC.contains(&format!("\"{}\"", id)),
            "design_picker actions catalog missing required id `{}`",
            id
        );
    }
}

#[test]
fn design_picker_actions_use_actions_dialog_host() {
    assert!(
        ACTIONS_SRC.contains("ActionsDialogHost::DesignPicker"),
        "toggle_design_picker_actions must route through ActionsDialogHost::DesignPicker"
    );
    // Activation and close callbacks both must be wired to the same host.
    let occurrences = ACTIONS_SRC
        .matches("ActionsDialogHost::DesignPicker")
        .count();
    assert!(
        occurrences >= 2,
        "ActionsDialogHost::DesignPicker must appear in both activation and close callbacks (saw {})",
        occurrences
    );
}
