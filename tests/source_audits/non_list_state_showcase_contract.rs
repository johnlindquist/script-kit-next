const COMPONENTS_MOD: &str = include_str!("../../src/components/mod.rs");
const NON_LIST_STATE: &str = include_str!("../../src/components/non_list_state.rs");
const STORYBOOK_MOD: &str = include_str!("../../src/storybook/mod.rs");
const STORYBOOK_SHOWCASE: &str = include_str!("../../src/storybook/non_list_state_showcase.rs");
const STORYBOOK_STORY: &str = include_str!("../../src/stories/non_list_state_showcase.rs");
const STORIES_MOD: &str = include_str!("../../src/stories/mod.rs");

#[test]
fn non_list_state_helpers_are_exported_from_components() {
    assert!(
        COMPONENTS_MOD.contains("pub(crate) mod non_list_state;"),
        "components module should own the non-list state helper module"
    );
    for symbol in [
        "non_list_centered_shell",
        "non_list_content_stack",
        "non_list_intro",
        "non_list_card",
        "non_list_callout",
        "non_list_requirement_row",
        "NonListDensity",
        "NonListPalette",
    ] {
        assert!(
            COMPONENTS_MOD.contains(symbol),
            "components module should re-export {symbol}"
        );
    }
}

#[test]
fn non_list_state_palette_routes_through_app_chrome() {
    assert!(
        NON_LIST_STATE.contains("AppChromeColors::from_theme(theme)"),
        "non-list palette should route through shared AppChromeColors"
    );
    for token in [
        "chrome.text_primary_hex",
        "chrome.text_muted_rgba",
        "chrome.text_hint_rgba",
        "chrome.placeholder_text_rgba",
        "chrome.panel_surface_rgba",
        "chrome.input_surface_rgba",
        "chrome.selection_rgba",
    ] {
        assert!(
            NON_LIST_STATE.contains(token),
            "non-list palette should use {token}"
        );
    }
}

#[test]
fn storybook_showcase_registers_required_layouts() {
    for required in [
        "Empty",
        "Help",
        "Form",
        "Setup",
        "Permission",
        "Recovery",
        "About",
        "Density",
    ] {
        assert!(
            STORYBOOK_SHOWCASE.contains(required),
            "showcase should include {required} layout"
        );
    }

    assert!(
        STORYBOOK_SHOWCASE.contains(".with_prop(\"surface\", \"nonListState\")"),
        "showcase variants should identify the non-list state surface"
    );
    assert!(
        STORYBOOK_SHOWCASE.contains(".with_prop(\"representation\", \"presenterFixture\")"),
        "showcase variants should be presenter fixtures"
    );
}

#[test]
fn storybook_registry_exposes_non_list_state_story() {
    assert!(
        STORYBOOK_MOD.contains("pub mod non_list_state_showcase;"),
        "storybook module should expose the non-list showcase module"
    );
    assert!(
        STORYBOOK_MOD.contains("render_non_list_state_showcase_preview"),
        "storybook module should re-export showcase render helpers"
    );
    assert!(
        STORYBOOK_STORY.contains("StorySurface::NonListState"),
        "story wrapper should use the NonListState surface"
    );
    assert!(
        STORIES_MOD.contains("mod non_list_state_showcase;")
            && STORIES_MOD.contains("NonListStateShowcaseStory")
            && STORIES_MOD.contains("StoryEntry::new(Box::new(NonListStateShowcaseStory))"),
        "stories registry should include the non-list showcase story"
    );
}
