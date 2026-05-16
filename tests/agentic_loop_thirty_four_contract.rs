//! Source-level contract for thirty-fourth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_four_recipes() {
    for name in [
        "file-search-directory-breadcrumb-restoration-stress",
        "emoji-picker-skin-tone-category-ux-stress",
        "root-window-source-filter-activation-refusal-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn file_search_breadcrumb_restoration_pins_redacted_navigation_and_reanchor() {
    for token in [
        "fileSearchDirectoryBreadcrumbRestorationReceipt",
        "runFileSearchDirectoryBreadcrumbRestorationStressScenario",
        "missing_file_search_directory_breadcrumb_restoration_receipt",
        "ux.fileSearchDirectoryBreadcrumbRestoration",
        "fileSearchDirectoryBreadcrumbRestorationStressId",
        "fixtureDirectoryTreeId",
        "rootFolderFingerprint",
        "breadcrumbSegmentIds",
        "redactedBreadcrumbLabels",
        "onlyInFilterChipId",
        "renderedInputText",
        "strippedSearchText",
        "visibleFileRowIds",
        "directoryRowsBefore",
        "directoryRowsAfter",
        "selectedFileIdBefore",
        "selectedFileIdAfter",
        "selectionReanchoredAfterBreadcrumbClick",
        "filterPreservedAfterDirectoryChange",
        "backForwardStackDepth",
        "scrollAnchorRestored",
        "previewGeneration",
        "noRawPathLeak",
        "nativePickerRefused",
        "quickLookRefused",
        "staleDirectoryGenerationRejected",
        "wrongOriginRejected",
        "file_linear:file_search_directory_breadcrumb_restoration_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn emoji_picker_skin_tone_category_pins_palette_graphemes_and_no_insert() {
    for token in [
        "emojiPickerSkinToneCategoryUxReceipt",
        "runEmojiPickerSkinToneCategoryUxStressScenario",
        "missing_emoji_picker_skin_tone_category_ux_receipt",
        "ux.emojiPickerSkinToneCategoryUx",
        "emojiPickerSkinToneCategoryUxStressId",
        "fixtureEmojiCatalogId",
        "categoryTabIds",
        "selectedCategoryId",
        "stickyCategoryHeaderBounds",
        "skinTonePaletteId",
        "skinTonePaletteBounds",
        "skinToneVariantIds",
        "selectedSkinToneToken",
        "emojiRowIds",
        "zwjSequenceIds",
        "graphemeClusterFingerprints",
        "searchGeneration",
        "highlightedRanges",
        "accessibleLabelParity",
        "previewGlyphBounds",
        "paletteDismissalReceipt",
        "selectionPreservedAcrossCategorySwitch",
        "noSystemPasteboardMutation",
        "noEmojiInsert",
        "stalePaletteGenerationRejected",
        "wrongCategoryMutationRejected",
        "file_linear:emoji_picker_skin_tone_category_ux_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn root_window_source_filter_activation_refusal_pins_no_focus_steal() {
    for token in [
        "rootWindowSourceFilterActivationRefusalReceipt",
        "runRootWindowSourceFilterActivationRefusalStressScenario",
        "missing_root_window_source_filter_activation_refusal_receipt",
        "ux.rootWindowSourceFilterActivationRefusal",
        "rootWindowSourceFilterActivationRefusalStressId",
        "fixtureWindowProviderId",
        "sourceFilterSet",
        "renderedInputText",
        "strippedSearchText",
        "rootFrameKey",
        "windowSnapshotGeneration",
        "zOrderGeneration",
        "visibleWindowRowIds",
        "windowRowFingerprints",
        "selectedStableKeyBefore",
        "selectedStableKeyAfter",
        "selectedRowVisible",
        "actionsSubjectStableKey",
        "activationDryRunReceipt",
        "enterActivationRefused",
        "noNativeWindowActivation",
        "noFocusSteal",
        "duplicateWindowKeyRejected",
        "staleWindowSnapshotRejected",
        "statusChipsNonSelectable",
        "file_linear:root_window_source_filter_activation_refusal_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_four_boundaries() {
    for token in [
        "File Search directory breadcrumb restoration",
        "Emoji Picker skin-tone/category UX",
        "root Windows source-filter activation refusal",
        "agentic_loop_thirty_four_contract",
        "no-native-input",
        "no-native-pointer",
        "no-native-picker",
        "no-window-activation",
        "no-quick-look",
        "no-network",
        "cleanupConfirmed",
    ] {
        assert!(
            SKILL.contains(token)
                || AUTOMATION.contains(token)
                || VERIFICATION.contains(token)
                || INDEX.contains(token)
                || SCENARIO.contains(token)
        );
    }
}
