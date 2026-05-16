//! Source-level contract for twenty-seventh-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_seven_recipes() {
    for name in [
        "warning-banner-action-dismiss-semantics-stress",
        "select-prompt-multiselect-keyboard-state-stress",
        "file-search-preview-sanitization-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn warning_banner_action_dismiss_pins_action_dismiss_contrast_and_obstruction_receipts() {
    for token in [
        "warningBannerActionDismissSemanticsReceipt",
        "runWarningBannerActionDismissSemanticsStressScenario",
        "missing_warning_banner_action_dismiss_semantics_receipt",
        "ux.warningBannerActionDismissSemantics",
        "warningBannerActionDismissSemanticsStressId",
        "bannerSamples",
        "bannerGeneration",
        "bannerSemanticId",
        "bannerKind",
        "bannerVisibleText",
        "bannerBounds",
        "bannerTextBounds",
        "bannerActionSemanticId",
        "bannerDismissSemanticId",
        "hoverStateReceipt",
        "focusStateReceipt",
        "dismissClickReceipt",
        "actionClickReceipt",
        "actionExecutionPreventedForDismiss",
        "dismissDoesNotTriggerAction",
        "actionDoesNotDismissUnlessConfigured",
        "nonColorStateCue",
        "contrastRatio",
        "footerNotObscured",
        "inputNotObscured",
        "staleBannerGenerationRejected",
        "wrongSurfaceRejected",
        "file_linear:warning_banner_action_dismiss_semantics_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn select_prompt_multiselect_pins_keyboard_state_filter_and_no_submit_receipts() {
    for token in [
        "selectPromptMultiselectKeyboardStateReceipt",
        "runSelectPromptMultiselectKeyboardStateStressScenario",
        "missing_select_prompt_multiselect_keyboard_state_receipt",
        "ux.selectPromptMultiselectKeyboardState",
        "selectPromptMultiselectKeyboardStateStressId",
        "multiSelectSamples",
        "choiceCount",
        "selectionStep",
        "promptType",
        "selectMode",
        "focusedChoiceSemanticId",
        "selectedChoiceSemanticIds",
        "checkedRowSemanticIds",
        "visibleChoiceSemanticIds",
        "selectionCountLabel",
        "footerSubmitLabel",
        "footerSubmitDisabledReason",
        "filterTextBefore",
        "filterTextAfter",
        "filterGeneration",
        "selectionGeneration",
        "cmdAReceipt",
        "spaceToggleReceipt",
        "rangeToggleReceipt",
        "filterPreservesSelectedSet",
        "clearFilterRestoresCheckedRows",
        "checkedRowsMatchState",
        "visibleRowsMatchElements",
        "noSubmitReceipt",
        "noActivationReceipt",
        "file_linear:select_prompt_multiselect_keyboard_state_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn file_search_preview_sanitization_pins_redaction_fallback_and_no_external_handoff_receipts() {
    for token in [
        "fileSearchPreviewSanitizationReceipt",
        "runFileSearchPreviewSanitizationStressScenario",
        "missing_file_search_preview_sanitization_receipt",
        "ux.fileSearchPreviewSanitization",
        "fileSearchPreviewSanitizationStressId",
        "previewSamples",
        "previewFixtureKind",
        "selectedRowSemanticId",
        "selectedFileUri",
        "selectedFileFingerprint",
        "previewGeneration",
        "previewSourceIdentity",
        "previewRenderKind",
        "previewTitle",
        "previewVisibleText",
        "previewBounds",
        "previewTextBounds",
        "previewByteLimit",
        "previewTruncated",
        "binaryPreviewFallback",
        "missingFileFallback",
        "unsupportedPreviewFallback",
        "privatePathRedacted",
        "redactedPathFingerprint",
        "noRawPathLeak",
        "noNetworkFetch",
        "noExternalServiceContacted",
        "noQuickLookOpened",
        "noNativePickerOpened",
        "noSystemPasteboardMutation",
        "stalePreviewGenerationRejected",
        "wrongRowPreviewRejected",
        "file_linear:file_search_preview_sanitization_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_seven_boundaries() {
    for token in [
        "warning banner action/dismiss semantics",
        "SelectPrompt keyboard multi-selection state parity",
        "File Search safe preview sanitization",
        "agentic_loop_twenty_seven_contract",
        "usedNativeInput",
        "usedNativePointer",
        "systemPasteboardMutated",
        "destructiveOperationRequested",
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
