//! Source contract for design picker input handling.
//!
//! The design picker owns preview navigation, revert, commit, and row-click
//! semantics while the shared launcher filter may still be focused. Handled
//! picker input must stop propagation so it cannot fall through to the main
//! launcher after the picker closes.

const DESIGN_PICKER_SOURCE: &str = include_str!("../src/render_builtins/design_picker.rs");
const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");

fn design_picker_key_handler() -> &'static str {
    DESIGN_PICKER_SOURCE
        .split("// ── Keyboard handler")
        .nth(1)
        .and_then(|section| section.split("let selected = selected_index;").next())
        .expect("design picker keyboard handler should be present")
}

fn assert_stop_before_marker(source: &str, marker: &str) {
    let marker_start = source
        .find(marker)
        .unwrap_or_else(|| panic!("expected marker `{marker}` in design picker source"));
    let search_start = marker_start.saturating_sub(1200);
    let preceding = &source[search_start..marker_start];
    assert!(
        preceding.contains("cx.stop_propagation();"),
        "design picker click handler for `{marker}` must stop propagation before applying the design mutation"
    );
}

#[test]
fn handled_design_picker_keys_stop_propagation() {
    let handler = design_picker_key_handler();

    for marker in [
        "ActionsRoute::Handled =>",
        "ActionsRoute::Execute { action_id } =>",
        "if is_key_escape(key) && !this.show_actions_popup",
        "if has_cmd && key.eq_ignore_ascii_case(\"w\")",
        "\"design_picker_keyboard_preview\"",
    ] {
        let branch = handler
            .split(marker)
            .nth(1)
            .and_then(|section| section.split("return;").next())
            .expect("expected handled design picker key branch");
        assert!(
            branch.contains("cx.stop_propagation();"),
            "handled design picker key branch `{marker}` must stop propagation so Enter/Escape cannot fall through into the main filter handler"
        );
    }
}

#[test]
fn handled_design_picker_clicks_stop_propagation() {
    for marker in ["\"design_picker_mouse_click\""] {
        assert_stop_before_marker(DESIGN_PICKER_SOURCE, marker);
    }
}

#[test]
fn design_picker_enter_is_owned_by_shared_input_press_enter() {
    let press_enter_branch = STARTUP_SOURCE
        .split("InputEvent::PressEnter { .. } => {")
        .nth(1)
        .and_then(|section| section.split("// Handle Enter for mini/arg prompts").next())
        .expect("expected main input PressEnter branch");
    assert!(
        press_enter_branch.contains("AppView::DesignPickerView { .. }")
            && press_enter_branch
                .contains("this.submit_design_picker_from_input_enter(window, cx)")
            && press_enter_branch.contains("return;"),
        "InputEvent::PressEnter must route Enter to DesignPicker while DesignPicker owns the shared input"
    );

    assert!(
        DESIGN_PICKER_SOURCE.contains("submit_design_picker_from_input_enter")
            && DESIGN_PICKER_SOURCE.contains("\"design_picker_done\""),
        "DesignPicker submit behavior should live behind the shared input PressEnter owner branch"
    );

    let removed_bridge_guard = ["suppress", "_next_main_menu_press_enter"].concat();
    assert!(
        !APP_STATE_SOURCE.contains(&removed_bridge_guard)
            && !STARTUP_SOURCE.contains(&removed_bridge_guard)
            && !DESIGN_PICKER_SOURCE.contains(&removed_bridge_guard),
        "DesignPicker Enter must be modeled as input ownership, not one-shot bridge state"
    );
}
