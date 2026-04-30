//! Source-level contract for AURP-03 surface vocabulary.
//!
//! This pins the shared words used by the exhaustive surface registry. The
//! registry itself is covered by `app_view_policy_contract.rs`.

const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
const SURFACES_DOC: &str = include_str!("../lat.md/surfaces.md");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start.find(end).unwrap_or(after_start.len());
    &after_start[..end_index]
}

// @lat: [[lat.md/surfaces#Surfaces#Surface Contract Vocabulary]]
#[test]
fn launcher_surface_vocabulary_names_behavior_dimensions() {
    let vocabulary_block = source_between(
        APP_VIEW_STATE,
        "pub(crate) enum LauncherSurfaceFamily",
        "/// The user/system action",
    );

    for expected in [
        "pub(crate) enum LauncherSurfaceFamily",
        "MainMenu",
        "ScriptPrompt",
        "FilterableLauncherList",
        "UtilityWorkspace",
        "AttachmentPortal",
        "AssistantWorkspace",
        "FeedbackSurface",
        "pub(crate) enum LauncherSurfaceInputOwnership",
        "LauncherFilter",
        "PromptEntity",
        "ChildView",
        "NoEditableInput",
        "pub(crate) enum LauncherSurfacePreviewRole",
        "NoPersistentPreview",
        "OptionalInfoPanel",
        "RequiredSplitPreview",
        "ContentPane",
        "FeedbackPanel",
    ] {
        assert!(
            vocabulary_block.contains(expected),
            "surface vocabulary must include behavior name `{expected}`"
        );
    }
}

#[test]
fn launcher_surface_vocabulary_is_not_a_stringly_typed_placeholder() {
    let vocabulary_block = source_between(
        APP_VIEW_STATE,
        "pub(crate) enum LauncherSurfaceFamily",
        "/// The user/system action",
    );

    assert!(
        vocabulary_block.contains("pub(crate) struct LauncherSurfaceContractVocabulary"),
        "surface dimensions should be grouped by a named struct for the future registry"
    );
    assert!(
        vocabulary_block.contains("pub(crate) const fn new("),
        "the vocabulary tuple should be constructible without ad hoc string maps"
    );
    assert!(
        !vocabulary_block.contains("String,") && !vocabulary_block.contains("&'static str"),
        "AURP-03 vocabulary should use enums, not stringly typed surface names"
    );
}

#[test]
fn surfaces_lattice_explains_the_same_vocabulary() {
    assert!(
        SURFACES_DOC.contains("## Surface Contract Vocabulary"),
        "surfaces.md must describe the shared vocabulary for agents"
    );

    for expected in [
        "LauncherSurfaceFamily",
        "LauncherSurfaceInputOwnership",
        "LauncherSurfacePreviewRole",
        "LauncherSurfaceContractVocabulary",
        "filterable launcher list with launcher-owned input",
        "child-view content pane",
    ] {
        assert!(
            SURFACES_DOC.contains(expected),
            "surfaces.md must include `{expected}`"
        );
    }
}
