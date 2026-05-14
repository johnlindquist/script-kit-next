#[test]
fn compare_mode_labels_fixture_data_sources() {
    let src = include_str!("../src/storybook/main_menu_variations/mod.rs");
    let browser = include_str!("../src/storybook/browser.rs");

    if src.contains("compare") || src.contains("Compare") || browser.contains("Compare") {
        assert!(
            src.contains("ComparePanelContract") && src.contains("StorybookDataSource"),
            "main-menu compare mode must declare fixture/data-source contracts"
        );
        assert!(
            !src.contains("registered_primary_catalog: true") || !src.contains("MockDesignOnly"),
            "primary-catalog compare panel must not use MockDesignOnly data"
        );
    }
}
