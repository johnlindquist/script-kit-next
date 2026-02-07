use script_kit_gpui::components::FooterButton;

#[test]
fn footer_button_hover_bg_uses_15_percent_alpha() {
    assert_eq!(FooterButton::hover_bg(0x123456), 0x12345626);
}
