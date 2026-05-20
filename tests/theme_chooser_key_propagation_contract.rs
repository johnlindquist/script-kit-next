//! Source contract for theme chooser input handling.
//!
//! The theme chooser has its own Enter/Escape/navigation and click semantics
//! while the main filter input may still be focused. Handled chooser input must
//! stop propagation or the same event can fall through to the main menu after
//! the chooser returns, executing whatever launcher item is selected there.

const THEME_CHOOSER_SOURCE: &str = include_str!("../src/render_builtins/theme_chooser.rs");
const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");

fn theme_chooser_key_handler() -> &'static str {
    THEME_CHOOSER_SOURCE
        .split("// ── Keyboard handler")
        .nth(1)
        .and_then(|section| {
            section
                .split("// ── Pre-compute data for list closure")
                .next()
        })
        .expect("theme chooser keyboard handler should be present")
}

fn assert_stop_before_marker(source: &str, marker: &str) {
    let marker_start = source
        .find(marker)
        .unwrap_or_else(|| panic!("expected marker `{marker}` in theme chooser source"));
    let search_start = marker_start.saturating_sub(2000);
    let preceding = &source[search_start..marker_start];
    assert!(
        preceding.contains("cx.stop_propagation();"),
        "theme chooser click handler for `{marker}` must stop propagation before applying the theme mutation"
    );
}

#[test]
fn handled_theme_chooser_keys_stop_propagation() {
    let handler = theme_chooser_key_handler();

    for marker in [
        "ActionsRoute::Handled =>",
        "ActionsRoute::Execute {\n                        action_id,\n                        should_close,\n                    } =>",
        "if is_key_escape(key) && !this.show_actions_popup",
        "if has_cmd && key.eq_ignore_ascii_case(\"w\")",
        "\"theme_chooser_keyboard_preview\"",
    ] {
        let branch = handler
            .split(marker)
            .nth(1)
            .and_then(|section| section.split("return;").next())
            .expect("expected handled theme chooser key branch");
        assert!(
            branch.contains("cx.stop_propagation();"),
            "handled theme chooser key branch `{marker}` must stop propagation so Enter/Escape cannot fall through into the main filter handler"
        );
    }
}

#[test]
fn handled_theme_chooser_clicks_stop_propagation() {
    for marker in [
        "\"theme_chooser_mouse_click\"",
        "\"theme_chooser_accent_click\"",
        "\"theme_chooser_opacity_click\"",
        "\"theme_chooser_vibrancy_click\"",
        "\"theme_chooser_material_click\"",
        "\"theme_chooser_font_size_click\"",
        "\"theme_chooser_reset_click\"",
        "\"theme_chooser_save_as_click\"",
        "\"theme_chooser_save_copy_click\"",
        "\"theme_chooser_update_user_theme_click\"",
        "\"theme_chooser_delete_user_theme_click\"",
        "\"theme_chooser_restore_deleted_user_theme_click\"",
        "\"theme_chooser_gradient_click\"",
        "\"theme_chooser_surprise_me\"",
    ] {
        assert_stop_before_marker(THEME_CHOOSER_SOURCE, marker);
    }
}

#[test]
fn theme_chooser_enter_is_owned_by_shared_input_press_enter() {
    let press_enter_branch = STARTUP_SOURCE
        .split("InputEvent::PressEnter { .. } => {")
        .nth(1)
        .and_then(|section| section.split("// Handle Enter for mini/arg prompts").next())
        .expect("expected main input PressEnter branch");
    assert!(
        press_enter_branch.contains("AppView::ThemeChooserView { .. }")
            && press_enter_branch.contains("this.submit_theme_chooser_from_input_enter(window, cx)")
            && press_enter_branch.contains("return;"),
        "InputEvent::PressEnter must route Enter to ThemeChooser while ThemeChooser owns the shared input"
    );

    assert!(
        THEME_CHOOSER_SOURCE.contains("submit_theme_chooser_from_input_enter")
            && THEME_CHOOSER_SOURCE.contains("\"theme_chooser_done\""),
        "ThemeChooser submit behavior should live behind the shared input PressEnter owner branch"
    );

    let removed_bridge_guard = ["suppress", "_next_main_menu_press_enter"].concat();
    assert!(
        !APP_STATE_SOURCE.contains(&removed_bridge_guard)
            && !STARTUP_SOURCE.contains(&removed_bridge_guard)
            && !THEME_CHOOSER_SOURCE.contains(&removed_bridge_guard),
        "ThemeChooser Enter must be modeled as input ownership, not one-shot bridge state"
    );
}
