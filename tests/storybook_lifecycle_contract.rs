#[test]
fn storybook_lifecycle_uses_counted_window_registry() {
    let browser = include_str!("../src/storybook/browser.rs");
    let module = include_str!("../src/storybook/mod.rs");

    assert!(
        browser.contains("StorybookWindowRegistry") || module.contains("StorybookWindowRegistry"),
        "storybook lifecycle should count Storybook-owned primary and child windows"
    );
    assert!(
        browser.contains("should_quit_after_close") || module.contains("should_quit_after_close"),
        "quit-on-last-window must be guarded by the Storybook registry state"
    );
    for token in [
        "register_primary",
        "register_child",
        "unregister_primary",
        "unregister_child",
    ] {
        assert!(module.contains(token), "registry missing {token}");
    }
}
