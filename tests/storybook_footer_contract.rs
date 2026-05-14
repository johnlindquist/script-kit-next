#[test]
fn main_menu_storybook_covers_required_footer_states() {
    let src = include_str!("../src/storybook/main_menu_variations/mod.rs");

    for required in [
        "populated-results",
        "empty-results",
        "selected-row",
        "bottom-of-list-footer-safe-reveal",
        "frontmost-app-paste",
        "acp-ready-footer",
        "acp-not-ready-footer",
    ] {
        assert!(
            src.contains(required),
            "missing required main-menu footer/story state: {required}"
        );
    }

    assert!(src.contains("StorybookFooterSnapshot"));
    assert!(src.contains("ActiveFooterState") || src.contains("MainWindowFooterConfig"));
    assert!(src.contains("activeFooter"));
    assert!(src.contains("execute_script_by_path"));
    assert!(src.contains("SCRIPT_READY receipt missing"));
}
