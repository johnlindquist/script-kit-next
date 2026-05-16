//! Source-level contract for twenty-sixth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_six_recipes() {
    for name in [
        "mini-full-transition-layout-continuity-stress",
        "filter-input-decoration-chip-layout-stress",
        "focus-ring-viewport-integrity-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
    for function_name in [
        "runMiniFullTransitionLayoutContinuityStressScenario",
        "runFilterInputDecorationChipLayoutStressScenario",
        "runFocusRingViewportIntegrityStressScenario",
    ] {
        assert!(INDEX.contains(function_name) || SCENARIO.contains(function_name));
    }
}

#[test]
fn mini_full_transition_layout_continuity_pins_bounds_rem_footer_and_capture_receipts() {
    for token in [
        "mini-full-transition-layout-continuity-stress",
        "miniFullTransitionLayoutContinuityReceipt",
        "runMiniFullTransitionLayoutContinuityStressScenario",
        "missing_mini_full_transition_layout_continuity_receipt",
        "ux.miniFullTransitionLayoutContinuity",
        "miniFullTransitionLayoutContinuityStressId",
        "modeSamples",
        "transition",
        "fixture",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "modeBefore",
        "modeAfter",
        "viewTypeBefore",
        "viewTypeAfter",
        "transitionGeneration",
        "remSizeBefore",
        "remSizeAfter",
        "scaleFactor",
        "windowBoundsBefore",
        "windowBoundsAfter",
        "contentBoundsBefore",
        "contentBoundsAfter",
        "inputBoundsBefore",
        "inputBoundsAfter",
        "listViewportBoundsBefore",
        "listViewportBoundsAfter",
        "footerBoundsBefore",
        "footerBoundsAfter",
        "nativeFooterSurfaceId",
        "focusRingBounds",
        "selectedRowVisible",
        "selectedRowAboveFooter",
        "noInputFooterOverlap",
        "noContentClip",
        "noFooterClip",
        "noPopupMainClobbering",
        "screenshotToSemanticsAlignment",
        "strictCaptureTarget",
        "blankScreenshotRejected",
        "staleModeGenerationRejected",
        "wrongSurfaceRejected",
        "usedNativeInput",
        "usedNativePointer",
        "openedSystemSettings",
        "mutatedTcc",
        "systemPasteboardMutated",
        "setupInstallFlowEntered",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "destructiveOperationRequested",
        "cleanupConfirmed",
        "file_linear:mini_full_transition_layout_continuity_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-six mini/full scenario must pin {token}"
        );
    }
}

#[test]
fn filter_input_decoration_chip_layout_pins_chip_bounds_overlap_and_stale_clearing_receipts() {
    for token in [
        "filter-input-decoration-chip-layout-stress",
        "filterInputDecorationChipLayoutReceipt",
        "runFilterInputDecorationChipLayoutStressScenario",
        "missing_filter_input_decoration_chip_layout_receipt",
        "ux.filterInputDecorationChipLayout",
        "filterInputDecorationChipLayoutStressId",
        "inputDecorationSamples",
        "query",
        "widthMode",
        "scaleFactor",
        "remSize",
        "stateReceipt",
        "elementsReceipt",
        "filterInputDecorations",
        "renderedInputText",
        "strippedSearchText",
        "chipRanges",
        "chipRoles",
        "chipBounds",
        "textBounds",
        "renderedTextBounds",
        "cursorBounds",
        "placeholderBounds",
        "measuredWidth",
        "availableWidth",
        "visibleText",
        "decorationGeneration",
        "inputGeneration",
        "sourceHeadCleared",
        "staleDecorationCleared",
        "noChipTextOverlap",
        "noChipCursorOverlap",
        "noPlaceholderOverlap",
        "noInputFooterOverlap",
        "noHorizontalClip",
        "tooltipOrAccessibleFullText",
        "screenshotToSemanticsAlignment",
        "strictCaptureTarget",
        "blankScreenshotRejected",
        "staleDecorationGenerationRejected",
        "wrongSurfaceRejected",
        "configUnchanged",
        "file_linear:filter_input_decoration_chip_layout_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-six filter input scenario must pin {token}"
        );
    }
}

#[test]
fn focus_ring_viewport_integrity_pins_ring_bounds_occlusion_tab_order_and_no_submit_receipts() {
    for token in [
        "focus-ring-viewport-integrity-stress",
        "focusRingViewportIntegrityReceipt",
        "runFocusRingViewportIntegrityStressScenario",
        "missing_focus_ring_viewport_integrity_receipt",
        "ux.focusRingViewportIntegrity",
        "focusRingViewportIntegrityStressId",
        "focusSamples",
        "focusStep",
        "inputMode",
        "focusGeneration",
        "focusedSemanticId",
        "focusOwner",
        "semanticFocusMatchesState",
        "focusRingBounds",
        "focusedElementBounds",
        "viewportBounds",
        "scrollViewportBounds",
        "contentBounds",
        "footerBounds",
        "popupBounds",
        "ringVisible",
        "ringNotClipped",
        "ringWithinViewport",
        "ringAboveFooter",
        "ringNotObscuredByFooter",
        "ringNotCoveredByPopup",
        "tabOrderIndex",
        "tabOrderStable",
        "selectionPreserved",
        "scrollAnchorPreserved",
        "focusRestoredAfterEscape",
        "noActivationReceipt",
        "noSubmitReceipt",
        "staleFocusGenerationRejected",
        "wrongSurfaceFocusRejected",
        "destructiveOperationRequested",
        "file_linear:focus_ring_viewport_integrity_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-six focus ring scenario must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_six_boundaries() {
    for token in [
        "mini/full transition layout continuity",
        "no popup/main clobbering",
        "filter input decoration chip layout",
        "stale decoration clearing",
        "focus ring viewport integrity",
        "no footer/popup occlusion",
        "agentic_loop_twenty_six_contract",
        "usedNativeInput",
        "usedNativePointer",
        "openedSystemSettings",
        "mutatedTcc",
        "systemPasteboardMutated",
        "setupInstallFlowEntered",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "destructiveOperationRequested",
        "cleanupConfirmed",
    ] {
        assert!(
            SKILL.contains(token)
                || AUTOMATION.contains(token)
                || VERIFICATION.contains(token)
                || INDEX.contains(token)
                || SCENARIO.contains(token),
            "loop twenty-six docs/skill must teach {token}"
        );
    }
}
