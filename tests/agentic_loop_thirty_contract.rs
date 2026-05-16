//! Source-level contract for thirtieth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_recipes() {
    for name in [
        "inline-attachment-preview-chip-stability-stress",
        "window-title-status-semantics-stress",
        "menu-syntax-capture-validation-chip-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn inline_attachment_preview_pins_chip_redaction_and_no_external_receipts() {
    for token in [
        "inlineAttachmentPreviewChipStabilityReceipt",
        "runInlineAttachmentPreviewChipStabilityStressScenario",
        "missing_inline_attachment_preview_chip_stability_receipt",
        "ux.inlineAttachmentPreviewChipStability",
        "inlineAttachmentPreviewChipStabilityStressId",
        "fixtureAttachmentSetId",
        "hostSurfaceIdentity",
        "composerGeneration",
        "attachmentChipSemanticIds",
        "chipKinds",
        "chipLabels",
        "chipBounds",
        "previewRedactedFingerprint",
        "overflowChipReceipt",
        "focusChipReceipt",
        "removeChipReceipt",
        "reorderChipReceipt",
        "cursorSelectionPreserved",
        "noRawPathOrContentLeak",
        "noSystemPasteboard",
        "noNativePicker",
        "noScreenCapture",
        "noNetwork",
        "staleAttachmentRejected",
        "wrongHostRejected",
        "file_linear:inline_attachment_preview_chip_stability_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn window_title_status_pins_title_status_parity_and_stale_rejection_receipts() {
    for token in [
        "windowTitleStatusSemanticsReceipt",
        "runWindowTitleStatusSemanticsStressScenario",
        "missing_window_title_status_semantics_receipt",
        "ux.windowTitleStatusSemantics",
        "windowTitleStatusSemanticsStressId",
        "resolvedTarget",
        "automationWindowTitle",
        "nativeWindowTitle",
        "semanticSurfaceTitle",
        "visibleStatusText",
        "titleGeneration",
        "statusGeneration",
        "transitionReceipts",
        "detachedWindowParity",
        "attachedPopupParentTitleUnaffected",
        "statusErrorRecovery",
        "staleTitleRejected",
        "staleStatusRejected",
        "wrongSurfaceRejected",
        "noFocusSteal",
        "file_linear:window_title_status_semantics_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn menu_syntax_capture_validation_pins_chip_labels_and_no_submit_receipts() {
    for token in [
        "menuSyntaxCaptureValidationChipReceipt",
        "runMenuSyntaxCaptureValidationChipStressScenario",
        "missing_menu_syntax_capture_validation_chip_receipt",
        "ux.menuSyntaxCaptureValidationChip",
        "menuSyntaxCaptureValidationChipStressId",
        "fixtureMenuSyntaxCatalogId",
        "filterInputText",
        "menuSyntaxMainHintSnapshot",
        "captureValidationStatus",
        "statusChipLabels",
        "missingFieldLabels",
        "malformedFieldLabel",
        "malformedReason",
        "unresolvedDates",
        "fragmentPreviewRows",
        "priorityChoicesRow",
        "canSubmitFalsePreventsEnter",
        "noPayloadWrite",
        "noHandlerSpawn",
        "staleValidationRejected",
        "wrongSurfaceRejected",
        "file_linear:menu_syntax_capture_validation_chip_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_boundaries() {
    for token in [
        "inline attachment preview chip stability",
        "window title/status semantics",
        "menu syntax capture validation chips",
        "agentic_loop_thirty_contract",
        "no-native-input",
        "no-native-pointer",
        "no-native-picker",
        "no-screen-capture",
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
