//! Source-level contract for twenty-second-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_two_recipes() {
    for name in [
        "row-state-parity-without-pointer-stress",
        "quiet-chrome-card-nesting-stress",
        "scroll-shadow-sticky-header-density-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
    for function_name in [
        "runRowStateParityWithoutPointerStressScenario",
        "runQuietChromeCardNestingStressScenario",
        "runScrollShadowStickyHeaderDensityStressScenario",
    ] {
        assert!(INDEX.contains(function_name) || SCENARIO.contains(function_name));
    }
}

#[test]
fn row_state_parity_without_pointer_pins_paint_precedence_and_safety_receipts() {
    for token in [
        "row-state-parity-without-pointer-stress",
        "rowStateParityWithoutPointer",
        "runRowStateParityWithoutPointerStressScenario",
        "missing_row_state_parity_without_pointer_receipt",
        "ux.rowStateParityWithoutPointer",
        "rowStateParityStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "rowStateParityWithoutPointerReceipt",
        "rowStateSamples",
        "semanticId",
        "rowRole",
        "rowIndex",
        "rowLabel",
        "modality",
        "selectedSemanticId",
        "focusedSemanticId",
        "hoverSemanticId",
        "keyboardFocusRingVisible",
        "selectionPaintVisible",
        "hoverPaintVisible",
        "focusPaintVisible",
        "selectedFillToken",
        "hoverFillToken",
        "focusRingToken",
        "textOpacityToken",
        "iconOpacityToken",
        "selectedPrecedenceOverHover",
        "hoverDoesNotOverrideSelection",
        "focusDoesNotStealSelection",
        "focusedRowMatchesElements",
        "selectedRowMatchesState",
        "hoverReceiptSyntheticOnly",
        "noNativePointerRequired",
        "staleRowStateRejected",
        "wrongSurfaceRowStateRejected",
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
        "file_linear:row_state_parity_without_pointer_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-two row-state scenario must pin {token}"
        );
    }
}

#[test]
fn quiet_chrome_card_nesting_pins_layer_tokens_depth_and_rejection_receipts() {
    for token in [
        "quiet-chrome-card-nesting-stress",
        "quietChromeCardNesting",
        "runQuietChromeCardNestingStressScenario",
        "missing_quiet_chrome_card_nesting_receipt",
        "ux.quietChromeCardNesting",
        "quietChromeStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "quietChromeCardNestingReceipt",
        "chromeLayerSamples",
        "shellLayer",
        "contentLayer",
        "rowLayer",
        "popupLayer",
        "footerLayer",
        "borderToken",
        "fillToken",
        "shadowToken",
        "vibrancyMaterial",
        "cornerRadius",
        "insetPx",
        "gapPx",
        "cardDepth",
        "nestedCardCount",
        "maxAllowedCardDepth",
        "duplicateBorderRejected",
        "opaqueFillRejected",
        "heavyShadowRejected",
        "doubleCardNestingRejected",
        "footerChromeSeparated",
        "inputChromeSeparated",
        "popupMaterialPreserved",
        "quietChromeBudgetMatched",
        "themeTokenFingerprint",
        "staleChromeTokenRejected",
        "wrongSurfaceChromeRejected",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:quiet_chrome_card_nesting_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-two chrome scenario must pin {token}"
        );
    }
}

#[test]
fn scroll_shadow_sticky_header_density_pins_scroll_bounds_tokens_and_density_receipts() {
    for token in [
        "scroll-shadow-sticky-header-density-stress",
        "scrollShadowStickyHeaderDensity",
        "runScrollShadowStickyHeaderDensityStressScenario",
        "missing_scroll_shadow_sticky_header_density_receipt",
        "ux.scrollShadowStickyHeaderDensity",
        "scrollChromeDensityStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "scrollShadowStickyHeaderDensityReceipt",
        "scrollSamples",
        "scrollPosition",
        "scrollTop",
        "scrollViewportBounds",
        "scrollContentBounds",
        "scrollContentHeight",
        "scrollViewportHeight",
        "stickyHeaderReceipt",
        "headerSemanticId",
        "headerBounds",
        "headerPinned",
        "headerZIndex",
        "headerDoesNotOverlapRows",
        "headerDoesNotOverlapInput",
        "headerDoesNotOverlapFooter",
        "scrollShadowReceipt",
        "topShadowVisible",
        "bottomShadowVisible",
        "topShadowOpacityToken",
        "bottomShadowOpacityToken",
        "shadowGradientToken",
        "shadowMatchesScrollPosition",
        "densityReceipt",
        "densityMode",
        "rowHeightPx",
        "sectionHeaderHeightPx",
        "inputHeightPx",
        "footerHeightPx",
        "verticalGapPx",
        "horizontalInsetPx",
        "remSize",
        "scaleFactor",
        "densityTokenFingerprint",
        "rowRhythmStable",
        "footerSafeViewport",
        "selectedRowVisibleAboveFooter",
        "staleScrollGenerationRejected",
        "wrongSurfaceScrollRejected",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:scroll_shadow_sticky_header_density_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-two scroll/density scenario must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_two_boundaries() {
    for token in [
        "row visual-state parity without native pointer input",
        "selected, focused, hovered, and selected-hovered",
        "quiet chrome/card nesting",
        "duplicate-border rejection",
        "opaque-fill rejection",
        "scroll shadows, sticky headers, and density drift",
        "sticky header bounds/z-index",
        "footer-safe viewport",
        "agentic_loop_twenty_two_contract",
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
                || SCENARIO.contains(token),
            "docs and canonical skill must teach loop twenty-two boundary token {token}"
        );
    }
}
