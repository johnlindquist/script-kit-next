#[test]
fn main_menu_registered_stories_declare_non_mock_representation() {
    let adoption = include_str!("../src/storybook/adoption.rs");
    let variations = include_str!("../src/storybook/main_menu_variations/mod.rs");
    let wrapper = include_str!("../src/stories/main_menu_variations.rs");

    assert!(adoption.contains("MainMenuStoryContract"));
    assert!(adoption.contains("StorybookRepresentation"));
    assert!(adoption.contains("StorybookDataSource"));
    assert!(adoption.contains("FooterHintSource"));
    assert!(wrapper.contains("main_menu_story_variants"));
    assert!(variations.contains("LiveSurface") || variations.contains("PresenterFixture"));
    assert!(
        !variations.contains("MockDesignOnly"),
        "primary main-menu variants must not carry mock-only data or footer sources"
    );
    assert!(
        !variations.contains("runtimeFixture"),
        "primary main-menu variants must not use runtimeFixture"
    );
}
