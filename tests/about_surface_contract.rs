//! Source-level contracts for the launcher-native About route.

const ABOUT_RENDER_SOURCE: &str = include_str!("../src/about/render.rs");
const ABOUT_ROUTE_SOURCE: &str = include_str!("../src/app_impl/about_route.rs");
const BUTTON_SOURCE: &str = include_str!("../src/components/button/component.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");

#[test]
fn about_surface_uses_shared_chrome_tokens_not_local_alpha_packing() {
    assert!(
        ABOUT_RENDER_SOURCE.contains("AppChromeColors::from_theme"),
        "About should resolve chrome through shared AppChromeColors"
    );
    for needle in [
        "<< 8",
        "text_secondary_hex",
        "text_muted_hex",
        "text_dimmed_hex",
    ] {
        assert!(
            !ABOUT_RENDER_SOURCE.contains(needle),
            "About should not use local packed or pre-dimmed text chrome: {needle}"
        );
    }
    for needle in ["text_muted_rgba", "text_hint_rgba", "text_icon_rgba"] {
        assert!(
            ABOUT_RENDER_SOURCE.contains(needle),
            "About should use semantic text RGBA ladder token: {needle}"
        );
    }
}

#[test]
fn about_surface_controls_own_keyboard_activation() {
    assert!(
        ABOUT_RENDER_SOURCE.contains("fn is_about_activation_key"),
        "About custom controls should share a local activation-key helper"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains("ClickEvent::default()"),
        "About keyboard activation should invoke the same click handlers"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains("cx.stop_propagation();"),
        "About keyboard activation should consume handled keys"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains(".on_key_down(move |event: &KeyDownEvent, window, cx|"),
        "About custom controls should own Enter/Space activation"
    );
}

#[test]
fn about_surface_does_not_render_instructional_hint_strip() {
    assert!(
        !ABOUT_RENDER_SOURCE.contains("HintStrip"),
        "About should not render persistent instructional chrome"
    );
}

#[test]
fn about_surface_controls_keep_disabled_items_out_of_tab_order() {
    let action_button_start = ABOUT_RENDER_SOURCE
        .find("fn action_button(")
        .expect("action_button should exist");
    let action_button = &ABOUT_RENDER_SOURCE[action_button_start..];

    assert!(
        action_button.contains("if enabled {\n        button = button\n            .tab_index(0)"),
        "About action buttons should only enter tab order when enabled"
    );
    assert!(
        action_button.contains("} else {\n        button = button.cursor_default();\n    }"),
        "disabled About action buttons should render as default-cursor dead controls"
    );
}

#[test]
fn about_surface_text_is_contained_in_fixed_rows() {
    for needle in [
        ".w_full()\n        .max_w(px(500.0))",
        ".min_w(px(0.0))",
        ".overflow_hidden()",
        ".text_ellipsis()",
        ".whitespace_nowrap()",
    ] {
        assert!(
            ABOUT_RENDER_SOURCE.contains(needle),
            "About fixed rows should guard text overflow: {needle}"
        );
    }
}

#[test]
fn about_surface_body_scrolls_when_content_exceeds_container() {
    let body_start = ABOUT_RENDER_SOURCE
        .find(".id(\"about-content-scroll\")")
        .expect("About should expose a stable scroll container id");
    let body_block = &ABOUT_RENDER_SOURCE[body_start
        ..ABOUT_RENDER_SOURCE[body_start..]
            .find(".child(\n                    div()")
            .map(|offset| body_start + offset)
            .expect("About scroll container should wrap the content column")];

    assert!(
        body_block.contains(".flex_1()") && body_block.contains(".overflow_y_scrollbar()"),
        "About body should consume remaining window height and become scrollable when content overflows"
    );
    assert!(
        !body_block.contains(".overflow_hidden()"),
        "About body should not clip overflowing vertical content"
    );
}

#[test]
fn about_route_takes_launcher_input_out_of_focus() {
    assert!(ABOUT_ROUTE_SOURCE.contains("self.focused_input = FocusedInput::None"));
    assert!(ABOUT_ROUTE_SOURCE.contains("self.pending_focus = Some(FocusTarget::AppRoot)"));
    assert!(
        ABOUT_ROUTE_SOURCE.contains("transition_current_view_and_rekey_main_automation_surface")
    );
}

#[test]
fn about_escape_is_consumed_by_the_about_key_handler() {
    assert!(RENDER_IMPL_SOURCE.contains("this.dismiss_about(cx)"));
    assert!(RENDER_IMPL_SOURCE.contains("cx.stop_propagation();"));
}

#[test]
fn shared_button_keyboard_activation_stops_propagation() {
    let activation_start = BUTTON_SOURCE
        .find("if Button::can_activate_from_key")
        .expect("shared Button keyboard activation should exist");
    let activation_block = &BUTTON_SOURCE[activation_start
        ..BUTTON_SOURCE[activation_start..]
            .find("});")
            .map(|offset| activation_start + offset)
            .expect("Button key handler should close")];

    assert!(
        activation_block.contains("callback(&click_event, window, cx);")
            && activation_block.contains("cx.stop_propagation();")
            && activation_block.contains("cx.propagate();"),
        "shared Button should consume handled activation keys and propagate unhandled keys"
    );
}
