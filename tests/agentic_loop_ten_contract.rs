//! Source-level contract for tenth-loop agentic-testing visual diagnostics.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_ten_visual_recipes() {
    for name in [
        "visible-text-clipping-overlap-stress",
        "layout-measurement-regression-stress",
        "div-container-scroll-overflow-stress",
        "main-menu-dynamic-choice-resize-stress",
        "notes-window-resize-stress",
        "screenshot-semantics-visual-consistency-stress",
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
        "runVisibleTextClippingOverlapStressScenario",
        "runLayoutMeasurementRegressionStressScenario",
        "runDivContainerScrollOverflowStressScenario",
        "runMainMenuDynamicChoiceResizeStressScenario",
        "runNotesWindowResizeStressScenario",
        "runScreenshotSemanticsVisualConsistencyStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-ten function {function_name} must be wired"
        );
    }
}

#[test]
fn visible_text_clipping_overlap_stress_pins_text_bounds_overlap_and_truncation_receipts() {
    for token in [
        "visible-text-clipping-overlap-stress",
        "runVisibleTextClippingOverlapStressScenario",
        "visibleTextAudit",
        "missing_visible_text_measurement_receipt",
        "visibleTextLayoutAudit",
        "textMeasurementSource",
        "appkit_text_width_plus_getLayoutInfo",
        "textBounds",
        "renderedTextBounds",
        "containerBounds",
        "availableWidthPx",
        "measuredWidthPx",
        "clipIntent",
        "tooltipOrAccessibleFullText",
        "overlapPairs",
        "screenshot_only",
        "ocr_only",
        "estimated_width_only",
        "surface_not_yet_measured",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "Visible text visual audit must pin {token}"
        );
    }
}

#[test]
fn main_menu_dynamic_choice_resize_pins_count_bounds_and_cleanup_receipts() {
    for token in [
        "main-menu-dynamic-choice-resize-stress",
        "runMainMenuDynamicChoiceResizeStressScenario",
        "mainMenuDynamicChoiceResize",
        "missing_main_menu_dynamic_choice_resize_receipt",
        "ux.mainMenuDynamicChoiceResize",
        "mainMenuDynamicChoiceResizeStressId",
        "smallCount",
        "largeCount",
        "smallBounds",
        "largeBounds",
        "heightDeltaPx",
        "visibleChoiceCountTracksFixture",
        "heightGrewWithChoices",
        "widthStable",
        "escapeCleanupReceipt",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "Main menu dynamic choice resize audit must pin {token}"
        );
    }
}

#[test]
fn notes_window_resize_stress_pins_sandboxed_bounds_and_cleanup_receipts() {
    for token in [
        "notes-window-resize-stress",
        "runNotesWindowResizeStressScenario",
        "notesWindowResize",
        "missing_notes_window_resize_receipt",
        "ux.notesWindowResize",
        "notesWindowResizeStressId",
        "SCRIPT_KIT_TEST_NOTES_DB_PATH",
        "sandboxNotesStore",
        "beforeBounds",
        "afterGrowBounds",
        "afterShrinkBounds",
        "growDeltaPx",
        "shrinkDeltaPx",
        "heightGrewForTallContent",
        "heightShrankForShortContent",
        "widthStable",
        "cleanupConfirmed",
    ] {
        assert!(
            INDEX.contains(token)
                || SCENARIO.contains(token)
                || SKILL.contains(token)
                || AUTOMATION.contains(token),
            "Notes window resize audit must pin {token}"
        );
    }
}

#[test]
fn div_container_scroll_overflow_stress_pins_div_bounds_overflow_and_cleanup_receipts() {
    for token in [
        "div-container-scroll-overflow-stress",
        "runDivContainerScrollOverflowStressScenario",
        "divContainerScrollOverflow",
        "missing_div_container_scroll_overflow_receipt",
        "ux.divContainerScrollOverflow",
        "divContainerScrollOverflowStressId",
        "DivContent",
        "divContentBounds",
        "estimatedContentHeightPx",
        "divViewportHeightPx",
        "scrollRequired",
        "noLauncherListOrPreviewComponents",
        "scrollContainerSemanticId",
        "endMarkerPresentInFixture",
        "escapeCleanupReceipt",
        "scroll_position_not_yet_exposed_for_div_prompt",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "Div container overflow visual audit must pin {token}"
        );
    }
}

#[test]
fn layout_measurement_regression_stress_pins_rem_bounds_ownership_and_shift_receipts() {
    for token in [
        "layout-measurement-regression-stress",
        "runLayoutMeasurementRegressionStressScenario",
        "layoutMeasurement",
        "missing_layout_measurement_receipt",
        "layoutMeasurementRegression",
        "remPx",
        "scaleFactor",
        "contentBounds",
        "containerBounds",
        "scrollContainer",
        "footerOwnership",
        "inputOwnership",
        "layoutShiftAfterFilter",
        "layoutShiftAfterResize",
        "mainSurface",
        "attachedPopupSurface",
        "detachedAcpSurface",
        "window_bounds_only",
        "surface_not_yet_measured",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "Layout measurement visual audit must pin {token}"
        );
    }
}

#[test]
fn screenshot_semantics_consistency_stress_pins_pixel_semantic_alignment_receipts() {
    for token in [
        "screenshot-semantics-visual-consistency-stress",
        "runScreenshotSemanticsVisualConsistencyStressScenario",
        "visualConsistency",
        "strictWindow",
        "contentAudit",
        "blankLike",
        "semanticSurfaceMatched",
        "stateElementsSurfaceAgreement",
        "captureTargetMatched",
        "capture_target_mismatch",
        "screenshotCropAgreesWithElements",
        "targetBoundsInScreenshot",
        "selectedRowMatched",
        "focusReceiptMatched",
        "footerActionsMatched",
        "visibleTextMode",
        "semanticElements",
        "screenshot_semantics_consistency_failed",
        "screenshotSemanticsConsistency",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "screenshotCropWindowId",
        "selectedSemanticId",
        "selectedRowText",
        "focusRingElementId",
        "footerActions",
        "visibleTextFingerprint",
        "contentAudit",
        "selectedRowPixelBounds",
        "focusRingPixelBounds",
        "footerPixelBounds",
        "screenshotMatchesSemanticSurface",
        "visibleTextMatchesElements",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "Screenshot semantics visual audit must pin {token}"
        );
    }
}

#[test]
fn canonical_skill_teaches_visual_diagnostics_before_screenshot_trust() {
    for token in [
        "Visual Diagnostics",
        "visible text",
        "layout measurement",
        "screenshot-to-semantics",
        "Do not treat pixels alone as proof",
        "Do not claim text fits from a screenshot alone",
        "visibleTextMode:\"semanticElements\"",
        "not OCR",
        "not clipping proof",
        "getElements`, `getLayoutInfo`, and AppKit text measurement",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "agentic-testing skill must teach visual diagnostic rule: {token}"
        );
    }
}
