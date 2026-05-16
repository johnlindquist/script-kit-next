//! Source-level contract for seventeenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_seventeen_recipes() {
    for name in [
        "input-modality-transition-ownership-stress",
        "multi-context-attachment-dedupe-provenance-stress",
        "visual-contrast-readable-state-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
    }
    for function_name in [
        "runInputModalityTransitionOwnershipStressScenario",
        "runMultiContextAttachmentDedupeProvenanceStressScenario",
        "runVisualContrastReadableStateStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-seventeen function {function_name} must be wired"
        );
    }
}

#[test]
fn input_modality_transition_ownership_pins_hover_focus_scroll_and_activation() {
    for token in [
        "input-modality-transition-ownership-stress",
        "inputModalityTransitionOwnership",
        "runInputModalityTransitionOwnershipStressScenario",
        "missing_input_modality_transition_ownership_receipt",
        "modality.inputTransitionOwnership",
        "modalityStressId",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "initialStateReceipt",
        "initialElementsReceipt",
        "modalitySequence",
        "eventSequenceId",
        "inputDevice",
        "pointerDeviceKind",
        "modalityGeneration",
        "hoverSemanticId",
        "hoverBounds",
        "focusSemanticId",
        "focusRingVisible",
        "selectedSemanticId",
        "scrollInputKind",
        "scrollTopBefore",
        "scrollTopAfter",
        "scrollAnchorKey",
        "shortcutCommandId",
        "activationOwnerSemanticId",
        "activationMethod",
        "hoverFocusParity",
        "selectionPreservedAcrossModality",
        "activationOwnershipPreserved",
        "shortcutDidNotStealHoverOwner",
        "wheelDidNotMutateFocus",
        "staleModalityEventRejected",
        "wrongSurfaceInputRejected",
        "noAccidentalSubmit",
        "screenshotStateRevalidated",
        "cleanupConfirmed",
        "file_linear:input_modality_transition_ownership_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "input modality transition ownership stress must pin {token}"
        );
    }
}

#[test]
fn multi_context_attachment_dedupe_provenance_pins_identity_privacy_and_ordering() {
    for token in [
        "multi-context-attachment-dedupe-provenance-stress",
        "multiContextAttachmentDedupeProvenance",
        "runMultiContextAttachmentDedupeProvenanceStressScenario",
        "missing_multi_context_attachment_dedupe_provenance_receipt",
        "context.multiContextAttachmentDedupeProvenance",
        "attachmentStressId",
        "contextRunId",
        "hostSamples",
        "destinationSurface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "destinationGeneration",
        "stateReceipt",
        "elementsReceipt",
        "originSamples",
        "sourceKind",
        "originSurface",
        "originGeneration",
        "sourceUri",
        "resourceProfile",
        "mcpResourceUri",
        "scriptResourceIdentity",
        "screenshotIdentity",
        "selectedTextCaptureGeneration",
        "clipboardGeneration",
        "redactedPreview",
        "privacyClass",
        "attachmentSamples",
        "attachmentId",
        "dedupeKey",
        "provenanceId",
        "provenanceFingerprint",
        "acceptedContextPartUri",
        "insertIndex",
        "removeReceipt",
        "reorderReceipt",
        "insertedAttachmentIds",
        "removedAttachmentIds",
        "reorderedAttachmentIds",
        "duplicateAttachmentIdsRejected",
        "dedupeCollisionRejected",
        "staleProvenanceRejected",
        "wrongDestinationRejected",
        "orphanAttachmentRejected",
        "noCrossHostLeakage",
        "rawPathNotLogged",
        "rawTextNotLogged",
        "cleanupConfirmed",
        "file_linear:multi_context_attachment_dedupe_provenance_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "multi-context attachment dedupe/provenance stress must pin {token}"
        );
    }
}

#[test]
fn visual_contrast_readable_state_pins_theme_state_text_and_screenshot_receipts() {
    for token in [
        "visual-contrast-readable-state-stress",
        "visualContrastReadableState",
        "runVisualContrastReadableStateStressScenario",
        "missing_visual_contrast_readable_state_receipt",
        "visual.contrastReadableState",
        "AGENTIC_THEME_CONTRAST_RECEIPT",
        "visualContrastStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "themeId",
        "themeMode",
        "themeTokenFingerprint",
        "appearanceGeneration",
        "scaleFactor",
        "remSize",
        "stateSamples",
        "stateKind",
        "semanticId",
        "role",
        "label",
        "visibleText",
        "fontSizePx",
        "fontWeight",
        "elementBounds",
        "textBounds",
        "foregroundColor",
        "backgroundColor",
        "contrastRatio",
        "minimumContrastRatio",
        "contrastPass",
        "readabilityPass",
        "focusIndicatorBounds",
        "focusIndicatorContrastRatio",
        "disabledStateVisible",
        "errorStateVisible",
        "loadingStateVisible",
        "nonColorStateCue",
        "activeInactiveDifferentiator",
        "screenshotReceipt",
        "screenshotStateRevalidated",
        "semanticVisibleTextMatchesReceipt",
        "staleThemeTokenRejected",
        "wrongSurfaceContrastRejected",
        "blankScreenshotRejected",
        "cleanupConfirmed",
        "usedCargoThemeContrastAudit",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "visual contrast readable-state stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_seventeen_boundaries() {
    for token in [
        "input-device modality transitions",
        "hover/focus/selection affordances",
        "activation ownership",
        "multi-context attachment dedupe",
        "attachment provenance",
        "redacted preview",
        "privacy leaks",
        "visual contrast readable state",
        "active inactive disabled focused error loading",
        "non-color state cue",
        "agentic_loop_seventeen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-seventeen docs and skill must teach {token}"
        );
    }
}
