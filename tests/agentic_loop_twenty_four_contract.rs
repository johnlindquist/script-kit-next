//! Source-level contract for twenty-fourth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_four_recipes() {
    for name in [
        "clipboard-copy-visual-feedback-stress",
        "portal-cancel-return-state-restoration-stress",
        "tooltip-hover-focus-affordance-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
    for function_name in [
        "runClipboardCopyVisualFeedbackStressScenario",
        "runPortalCancelReturnStateRestorationStressScenario",
        "runTooltipHoverFocusAffordanceStressScenario",
    ] {
        assert!(INDEX.contains(function_name) || SCENARIO.contains(function_name));
    }
}

#[test]
fn clipboard_copy_visual_feedback_pins_fixture_pasteboard_and_visible_feedback_receipts() {
    for token in [
        "clipboard-copy-visual-feedback-stress",
        "clipboardCopyVisualFeedback",
        "runClipboardCopyVisualFeedbackStressScenario",
        "missing_clipboard_copy_visual_feedback_receipt",
        "ux.clipboardCopyVisualFeedback",
        "clipboardCopyFeedbackStressId",
        "hostSamples",
        "host",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "clipboardCopyVisualFeedbackReceipt",
        "copyActionSemanticId",
        "copyActionLabel",
        "copyGeneration",
        "copyButtonStateBefore",
        "copyButtonStateAfter",
        "visibleCopiedState",
        "copiedStateDurationMs",
        "copyToastReceipt",
        "redactedPayloadPreview",
        "payloadFingerprint",
        "pasteboardScope",
        "fixturePasteboardUsed",
        "systemPasteboardUnchanged",
        "originalPasteboardFingerprint",
        "postRunPasteboardFingerprint",
        "noRawClipboardContentLogged",
        "staleCopyGenerationRejected",
        "wrongHostCopyRejected",
        "noAccidentalPaste",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:clipboard_copy_visual_feedback_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-four clipboard copy scenario must pin {token}"
        );
    }
}

#[test]
fn portal_cancel_return_restoration_pins_origin_portal_cancel_and_no_insert_receipts() {
    for token in [
        "portal-cancel-return-state-restoration-stress",
        "portalCancelReturnStateRestoration",
        "runPortalCancelReturnStateRestorationStressScenario",
        "missing_portal_cancel_return_state_restoration_receipt",
        "ux.portalCancelReturnStateRestoration",
        "portalCancelReturnStressId",
        "originSamples",
        "origin",
        "originAutomationWindowId",
        "originGeneration",
        "originSemanticSurface",
        "originStateReceipt",
        "originElementsReceipt",
        "draftTextBeforePortal",
        "cursorBeforePortal",
        "selectionBeforePortal",
        "portalSessionId",
        "portalSurface",
        "portalAutomationWindowId",
        "portalQuery",
        "portalSelectionBeforeCancel",
        "cancelMethod",
        "cancelReceipt",
        "returnTargetIdentity",
        "returnGeneration",
        "focusRestored",
        "draftTextRestored",
        "cursorRestored",
        "selectionRestored",
        "filterRestored",
        "scrollRestored",
        "noContextPartInserted",
        "noPromptSubmit",
        "noSelectionMutationDuringPortal",
        "stalePortalReturnRejected",
        "foreignPortalEventRejected",
        "wrongOriginReturnRejected",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:portal_cancel_return_state_restoration_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-four portal cancel scenario must pin {token}"
        );
    }
}

#[test]
fn tooltip_hover_focus_affordance_pins_keyboard_fallback_placement_and_dismissal_receipts() {
    for token in [
        "tooltip-hover-focus-affordance-stress",
        "tooltipHoverFocusAffordance",
        "runTooltipHoverFocusAffordanceStressScenario",
        "missing_tooltip_hover_focus_affordance_receipt",
        "ux.tooltipHoverFocusAffordance",
        "tooltipHoverFocusStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "tooltipHoverFocusAffordanceReceipt",
        "targetSamples",
        "targetSemanticId",
        "targetRole",
        "triggerMode",
        "hoverGeneration",
        "focusGeneration",
        "tooltipGeneration",
        "tooltipText",
        "tooltipKind",
        "tooltipAnchorBounds",
        "tooltipBounds",
        "tooltipPlacement",
        "hoverDelayMs",
        "hoverDelayRespected",
        "keyboardFocusOpensTooltip",
        "tooltipAccessibleDescriptionMatches",
        "escapeDismissesTooltip",
        "scrollDismissesTooltip",
        "focusLossDismissesTooltip",
        "noFocusSteal",
        "targetFocusPreserved",
        "doesNotCoverTarget",
        "doesNotCoverFooter",
        "doesNotCoverPopupOwner",
        "staleTooltipGenerationRejected",
        "wrongSurfaceTooltipRejected",
        "noAccidentalExecution",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:tooltip_hover_focus_affordance_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-four tooltip scenario must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_four_boundaries() {
    for token in [
        "clipboard copy visual feedback",
        "fixture-scoped pasteboard isolation",
        "unchanged system pasteboard fingerprints",
        "portal cancel/back return restoration",
        "no context insertion",
        "no prompt submit",
        "tooltip hover/focus affordances",
        "protocol-hover and keyboard-focus triggers",
        "accessible description parity",
        "no target/footer/popup-owner coverage",
        "agentic_loop_twenty_four_contract",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
    ] {
        assert!(
            SKILL.contains(token)
                || AUTOMATION.contains(token)
                || VERIFICATION.contains(token)
                || INDEX.contains(token)
                || SCENARIO.contains(token),
            "loop twenty-four docs/skill must teach {token}"
        );
    }
}
