//! Source-level contracts for the launcher-native About route.

const ABOUT_RENDER_SOURCE: &str = include_str!("../src/about/render.rs");
const ABOUT_ROUTE_SOURCE: &str = include_str!("../src/app_impl/about_route.rs");
const LAYOUT_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");
const BUTTON_SOURCE: &str = include_str!("../src/components/button/component.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");

#[test]
fn about_surface_uses_shared_chrome_tokens_not_local_alpha_packing() {
    assert!(
        ABOUT_RENDER_SOURCE.contains("AppChromeColors::from_theme"),
        "About should resolve chrome through shared AppChromeColors"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains("non_list_palette(&theme)"),
        "About body should route product-surface colors through the shared non-list palette"
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
    for needle in ["palette.body", "palette.hint", "palette.title"] {
        assert!(
            ABOUT_RENDER_SOURCE.contains(needle),
            "About should use semantic non-list text token: {needle}"
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
        ".max_w(px(metrics.max_width))",
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
        body_block.contains(".flex_1()")
            && body_block.contains(".min_h(px(0.0))")
            && body_block.contains(".overflow_y_scrollbar()"),
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
fn about_root_key_handler_propagates_non_escape_keys() {
    let about_start = RENDER_IMPL_SOURCE
        .find("AppView::About")
        .expect("About render arm should exist");
    let key_handler_start = RENDER_IMPL_SOURCE[about_start..]
        .find("key_down: std::rc::Rc::new")
        .map(|offset| about_start + offset)
        .expect("About render arm should wire root key handler");
    let key_handler = &RENDER_IMPL_SOURCE[key_handler_start
        ..RENDER_IMPL_SOURCE[key_handler_start..]
            .find("}),\n                };")
            .map(|offset| key_handler_start + offset)
            .expect("About root key handler should close before actions struct")];

    assert!(
        key_handler.contains("is_key_escape(event.keystroke.key.as_str())"),
        "About root key handler should only own Escape"
    );
    assert!(
        key_handler.contains("cx.stop_propagation();")
            && key_handler.contains(
                "} else {\n                            cx.propagate();\n                        }"
            ),
        "About root key handler should propagate Enter/Space to focused child controls"
    );
}

#[test]
fn about_quick_actions_wrap_before_overflowing_narrow_widths() {
    let quick_actions_start = ABOUT_RENDER_SOURCE
        .find("fn render_quick_actions")
        .expect("quick action row should exist");
    let quick_actions = &ABOUT_RENDER_SOURCE[quick_actions_start
        ..ABOUT_RENDER_SOURCE[quick_actions_start..]
            .find("fn render_update_card")
            .map(|offset| quick_actions_start + offset)
            .expect("quick action row should precede update card")];

    assert!(
        quick_actions.contains("non_list_action_row(vec![")
            && quick_actions.contains(".max_w(px(metrics.max_width))")
            && quick_actions.contains(".flex_wrap()"),
        "About quick actions should use the shared non-list action row and wrap within the content column"
    );
    assert!(
        quick_actions.contains("128.0"),
        "About quick action buttons should use compact minimum widths separate from the update button"
    );
    assert!(
        ABOUT_RENDER_SOURCE.contains("action_button_with_min_width"),
        "About should keep button sizing explicit per layout context"
    );
}

#[test]
fn about_layout_info_uses_about_specific_window_geometry() {
    let about_start = LAYOUT_SOURCE
        .find("if matches!(self.current_view, AppView::About")
        .expect("About layout branch should exist");
    let about_layout = &LAYOUT_SOURCE[about_start
        ..LAYOUT_SOURCE[about_start..]
            .find("if matches!(self.current_view, AppView::CreationFeedback")
            .map(|offset| about_start + offset)
            .expect("About layout branch should return before CreationFeedback")];

    for needle in [
        "AboutHeader",
        "AboutCloseButton",
        "AboutScrollContainer",
        "AboutContentStack",
        "AboutQuickActions",
        "AboutUpdateCard",
        "AboutAcknowledgementsCard",
    ] {
        assert!(
            about_layout.contains(needle),
            "About layout receipt should expose About-specific component: {needle}"
        );
    }
    assert!(
        about_layout.contains("LIQUID_GLASS_CONTROL_RADIUS_PX")
            && about_layout.contains("LIQUID_GLASS_COMPACT_RADIUS_PX"),
        "About layout receipt should report Liquid Glass control and compact radii"
    );
    assert!(
        about_layout.contains("let about_header_y = content_top;")
            && about_layout.contains("let about_scroll_y = content_top + ABOUT_HEADER_HEIGHT;")
            && about_layout.contains(".with_parent(\"MainViewMain\")"),
        "About detail receipts should live below and inside MainViewMain"
    );
    for node in [
        "AboutScrollContainer",
        "AboutContentStack",
        "AboutTitle",
        "AboutTagline",
        "AboutCreatorRow",
        "AboutQuickActions",
    ] {
        let node_start = about_layout
            .find(node)
            .unwrap_or_else(|| panic!("About layout should include {node}"));
        let node_source = &about_layout[node_start..];
        let token_end = node_source
            .find(".with_visual_token")
            .expect("About visual nodes should declare visual tokens");
        assert!(
            node_source[..token_end].contains("Some(chrome_tokens::LIQUID_GLASS_"),
            "About visual node {node} should expose a positive Liquid Glass radius before its visual token"
        );
    }
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
