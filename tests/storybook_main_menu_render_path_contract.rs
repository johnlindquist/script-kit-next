#[test]
fn main_menu_variations_do_not_hand_build_mock_rows() {
    let src = include_str!("../src/storybook/main_menu_variations/mod.rs");
    let wrapper = include_str!("../src/stories/main_menu_variations.rs");
    let forbidden = [
        "sample_rows",
        "mock_rows",
        "fake_rows",
        "render_mock_main_menu",
        "MainMenuMock",
    ];

    for token in forbidden {
        assert!(
            !src.contains(token) && !wrapper.contains(token),
            "main-menu Storybook still contains mock-only row token: {token}"
        );
    }

    for token in [
        "ProductionMainMenuFixture",
        "render_inputs",
        "render_script_list::render_main_menu_from_inputs",
    ] {
        assert!(
            src.contains(token),
            "main-menu Storybook must route through production-backed fixture helper: {token}"
        );
    }
}
