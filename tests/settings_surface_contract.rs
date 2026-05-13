//! Source-level contracts for the Settings built-in route.

const SETTINGS_SOURCE: &str = include_str!("../src/render_builtins/settings.rs");

#[test]
fn settings_uses_shared_chrome_text_tokens() {
    assert!(
        SETTINGS_SOURCE.contains("AppChromeColors::from_theme(&self.theme)"),
        "Settings should resolve route chrome through shared AppChromeColors"
    );
    for needle in [
        "self.theme.colors.text.dimmed",
        "self.theme.colors.text.muted",
        "let text_dimmed",
        "let text_muted",
    ] {
        assert!(
            !SETTINGS_SOURCE.contains(needle),
            "Settings should not use pre-dimmed local text colors: {needle}"
        );
    }
    assert!(SETTINGS_SOURCE.contains("chrome.text_primary_hex"));
    assert!(SETTINGS_SOURCE.contains("chrome.text_hint_rgba"));
    assert!(SETTINGS_SOURCE.contains("chrome.text_muted_rgba"));
}

#[test]
fn settings_exposes_visible_rows_to_state_and_elements() {
    let collect_elements = include_str!("../src/app_layout/collect_elements.rs");
    let prompt_handler = include_str!("../src/prompt_handler/mod.rs");

    for needle in [
        "fn settings_visible_row_names(&self, filter: &str) -> Vec<String>",
        "fn settings_dataset_and_visible_counts(&self, filter: &str) -> (usize, usize)",
        "fn settings_selected_visible_row_name(",
    ] {
        assert!(
            SETTINGS_SOURCE.contains(needle),
            "Settings should declare shared visible-row helper: {needle}"
        );
    }

    assert!(
        collect_elements.contains("let rows = self.settings_visible_row_names(filter);"),
        "getElements should collect Settings rows through the shared helper"
    );
    assert!(
        prompt_handler.contains("self.settings_dataset_and_visible_counts(filter)"),
        "getState should report Settings total and visible counts"
    );
    assert!(
        prompt_handler.contains("self.settings_selected_visible_row_name(filter, *selected_index)"),
        "getState should report the selected Settings row value"
    );
}

#[test]
fn settings_routes_actions_before_local_keys_and_consumes_handled_routes() {
    let route = SETTINGS_SOURCE
        .find("route_key_to_actions_dialog")
        .expect("Settings should route actions popup keys");
    let escape = SETTINGS_SOURCE
        .find("if is_key_escape(key)")
        .expect("Settings should handle Escape");
    let cmd_w = SETTINGS_SOURCE
        .find("key.eq_ignore_ascii_case(\"w\")")
        .expect("Settings should handle Cmd+W");

    assert!(
        route < escape && route < cmd_w,
        "Settings should route popup-owned keys before local Escape/Cmd+W"
    );
    for needle in [
        "ActionsRoute::Handled =>",
        "ActionsRoute::Execute {\n                        action_id,\n                        should_close,\n                    } =>",
        "cx.stop_propagation();\n                        return;",
    ] {
        assert!(
            SETTINGS_SOURCE.contains(needle),
            "Settings actions-popup route should consume handled branches: {needle}"
        );
    }
}

#[test]
fn settings_row_clicks_follow_launcher_activation_contract() {
    for needle in [
        "event.click_count()",
        "should_submit_selected_row_click",
        "*selected_index = ix;",
        "cx.stop_propagation();",
    ] {
        assert!(
            SETTINGS_SOURCE.contains(needle),
            "Settings row clicks should select first and submit through launcher helper: {needle}"
        );
    }
}

#[test]
fn settings_header_uses_bounded_flex_for_filter_and_count() {
    for needle in [
        ".flex_1()",
        ".min_w(px(0.))",
        ".flex_none()",
        ".whitespace_nowrap()",
        "chrome.text_hint_rgba",
    ] {
        assert!(
            SETTINGS_SOURCE.contains(needle),
            "Settings header should keep filter/count layout bounded: {needle}"
        );
    }
}

#[test]
fn settings_keeps_listitem_hover_disclosure_without_local_hover_paint() {
    assert!(
        SETTINGS_SOURCE.contains(".hovered(is_hovered)"),
        "Settings may keep ListItem hover disclosure state"
    );
    for needle in [
        "opacity.hover",
        "opacity.selected",
        "hover_row_bg",
        "selection_rgba",
    ] {
        assert!(
            !SETTINGS_SOURCE.contains(needle),
            "Settings should not locally paint row hover/selection chrome: {needle}"
        );
    }
}
