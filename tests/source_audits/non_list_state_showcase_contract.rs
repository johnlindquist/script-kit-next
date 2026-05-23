const COMPONENTS_MOD: &str = include_str!("../../src/components/mod.rs");
const ABOUT_RENDER: &str = include_str!("../../src/about/render.rs");
const AI_WELCOME_RENDER: &str = include_str!("../../src/ai/window/render_welcome.rs");
const BUILTIN_EXECUTION: &str = include_str!("../../src/app_execute/builtin_execution.rs");
const BUILTINS_MOD: &str = include_str!("../../src/builtins/mod.rs");
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

#[test]
fn non_list_state_design_command_routes_to_showcase_surface() {
    assert!(
        BUILTINS_MOD.contains("builtin/design-non-list-states"),
        "builtins should register a stable command id for the non-list design showcase"
    );
    assert!(
        BUILTINS_MOD.contains("Design: Non-List States"),
        "builtins should expose a searchable command label for the non-list design showcase"
    );
    assert!(
        BUILTINS_MOD.contains("BuiltInFeature::DesignNonListStates"),
        "builtins command should use a dedicated non-list design feature"
    );
    assert!(
        BUILTIN_EXECUTION.contains("OpenNonListStates"),
        "execution should define the non-list design explorer action"
    );
    assert!(
        BUILTIN_EXECUTION
            .contains("configure_for_design_explorer(Some(\n                        script_kit_gpui::storybook::StorySurface::NonListState"),
        "non-list design command should route directly to the NonListState surface"
    );
    assert!(
        BUILTIN_EXECUTION.contains("configure_for_design_explorer(Some(\n                        script_kit_gpui::storybook::StorySurface::MainMenu")
            && BUILTIN_EXECUTION.contains("select_variant_id(\"current-main-menu\")"),
        "existing Design Explorer command should keep its MainMenu startup behavior"
    );
}

#[test]
fn about_surface_consumes_non_list_state_language() {
    assert!(
        ABOUT_RENDER.contains("non_list_palette(&theme)"),
        "About surface should route visual colors through NonListPalette"
    );
    assert!(
        ABOUT_RENDER.contains("non_list_metrics(NonListDensity::Comfortable)"),
        "About surface should use comfortable non-list density"
    );
    assert!(
        ABOUT_RENDER.contains("non_list_content_stack(\"about-non-list-content\""),
        "About surface body should use the shared non-list content stack"
    );
    assert!(
        ABOUT_RENDER.contains("non_list_card(\"about-update-card\""),
        "About update card should use the shared non-list card primitive"
    );
    assert!(
        ABOUT_RENDER.contains("non_list_card(\"about-acknowledgements\""),
        "About acknowledgements should use the shared non-list card primitive"
    );
    assert!(
        ABOUT_RENDER.contains("non_list_action_row(vec!["),
        "About quick actions should use the shared non-list action row"
    );
    assert!(
        ABOUT_RENDER.contains("non_list_footer_note("),
        "About footer copy should use the shared non-list footer-note primitive"
    );
    assert!(
        ABOUT_RENDER.contains("palette: NonListPalette"),
        "About action buttons should receive NonListPalette instead of AppChromeColors"
    );
}

#[test]
fn ai_empty_chat_welcome_consumes_non_list_state_language() {
    assert!(
        AI_WELCOME_RENDER.contains("non_list_palette(&theme)"),
        "AI empty welcome should route colors through NonListPalette"
    );
    assert!(
        AI_WELCOME_RENDER.contains("non_list_metrics(NonListDensity::Compact)"),
        "Mini AI welcome should use compact non-list density"
    );
    assert!(
        AI_WELCOME_RENDER.contains("non_list_metrics(NonListDensity::Comfortable)"),
        "Full AI welcome should use comfortable non-list density"
    );
    assert!(
        AI_WELCOME_RENDER.contains("non_list_centered_shell(\"ai-welcome-non-list\""),
        "Full AI welcome should use the shared centered non-list shell"
    );
    assert!(
        AI_WELCOME_RENDER.contains(
            "non_list_content_stack(\n                    \"ai-mini-welcome-suggestions\""
        ) || AI_WELCOME_RENDER.contains("non_list_content_stack(\"ai-mini-welcome-suggestions\""),
        "Mini AI welcome should use the shared non-list content stack"
    );
    assert!(
        AI_WELCOME_RENDER.contains("non_list_card(\"ai-welcome-suggestions\""),
        "Full AI welcome suggestion group should use the shared non-list card primitive"
    );
    assert!(
        AI_WELCOME_RENDER.contains("non_list_intro("),
        "AI empty welcome should use shared non-list intro hierarchy"
    );
    assert!(
        AI_WELCOME_RENDER
            .matches("return self.render_setup_card(cx).into_any_element();")
            .count()
            >= 2,
        "Mini and Full welcome must preserve setup-card branching"
    );
    assert!(
        !AI_WELCOME_RENDER.contains("crate::theme::opacity"),
        "AI empty welcome should not hand-roll design colors through theme opacity constants"
    );
    assert!(
        !AI_WELCOME_RENDER.contains("const SUGGESTION_MAX_W"),
        "Full AI welcome should use non-list metrics instead of local full-mode suggestion width"
    );
}
