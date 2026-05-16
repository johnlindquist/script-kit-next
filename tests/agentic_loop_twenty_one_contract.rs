//! Source-level contract for twenty-first-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_one_recipes() {
    for name in [
        "icon-image-fallback-redaction-stress",
        "footer-status-persistence-stress",
        "keyboard-hint-label-parity-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
    for function_name in [
        "runIconImageFallbackRedactionStressScenario",
        "runFooterStatusPersistenceStressScenario",
        "runKeyboardHintLabelParityStressScenario",
    ] {
        assert!(INDEX.contains(function_name) || SCENARIO.contains(function_name));
    }
}

#[test]
fn icon_image_fallback_redaction_pins_fallback_redaction_and_accessible_label_receipts() {
    for token in [
        "icon-image-fallback-redaction-stress",
        "iconImageFallbackRedaction",
        "runIconImageFallbackRedactionStressScenario",
        "missing_icon_image_fallback_redaction_receipt",
        "ux.iconImageFallbackRedaction",
        "iconImageStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "imageFallbackReceipt",
        "assetFixtureReceipt",
        "fixtureKind",
        "requestedImageSourceKind",
        "requestedImageFingerprint",
        "rawSourceRedacted",
        "displayedImageKind",
        "fallbackIconKind",
        "fallbackReason",
        "imageLoadGeneration",
        "cacheKeyFingerprint",
        "redactedPreview",
        "noRawPath",
        "noRawUrl",
        "noFileContents",
        "brokenImageRejected",
        "unsupportedSchemeRejected",
        "staleImageGenerationRejected",
        "defaultIconRendered",
        "accessibleLabelPreserved",
        "rowIdentityPreserved",
        "footerStatePreserved",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:icon_image_fallback_redaction_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-one icon scenario must pin {token}"
        );
    }
}

#[test]
fn footer_status_persistence_pins_owner_generation_and_no_duplicate_footer_receipts() {
    for token in [
        "footer-status-persistence-stress",
        "footerStatusPersistence",
        "runFooterStatusPersistenceStressScenario",
        "missing_footer_status_persistence_receipt",
        "ux.footerStatusPersistence",
        "footerStatusStressId",
        "surfaceSamples",
        "surface",
        "hostAutomationWindowId",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateBefore",
        "elementsBefore",
        "transitionSamples",
        "transitionGeneration",
        "routeStackDepth",
        "filterTextBefore",
        "selectedSemanticIdBefore",
        "footerReceipt",
        "footerOwner",
        "nativeFooterSurfaceId",
        "gpuiFallbackVisible",
        "renderedButtons",
        "buttonSemanticIds",
        "buttonLabel",
        "buttonShortcutHint",
        "disabledReason",
        "statusBarReceipt",
        "statusText",
        "statusKind",
        "statusGeneration",
        "persistedAcrossFilter",
        "persistedAcrossSelection",
        "persistedAcrossActionsOpenClose",
        "persistedAcrossPopupClose",
        "noDuplicateFooterRows",
        "noStaleStatusAfterRecovery",
        "footerSafeSelection",
        "inputCollisionFree",
        "wrongSurfaceFooterRejected",
        "staleFooterGenerationRejected",
        "stateAfter",
        "elementsAfter",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:footer_status_persistence_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-one footer scenario must pin {token}"
        );
    }
}

#[test]
fn keyboard_hint_label_parity_pins_cross_surface_shortcut_hint_receipts() {
    for token in [
        "keyboard-hint-label-parity-stress",
        "keyboardHintLabelParity",
        "runKeyboardHintLabelParityStressScenario",
        "missing_keyboard_hint_label_parity_receipt",
        "ux.keyboardHintLabelParity",
        "keyboardHintStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "actionCatalogReceipt",
        "footerHintReceipt",
        "rowHintSamples",
        "tooltipHintReceipt",
        "semanticId",
        "actionId",
        "hintOwner",
        "visibleLabel",
        "accessibleLabel",
        "footerLabel",
        "rowAccessoryLabel",
        "tooltipLabel",
        "tooltipNotRequiredReason",
        "platformShortcutLabel",
        "shortcutTokens",
        "normalizedShortcut",
        "glyphTokens",
        "labelParityMatched",
        "noMismatchedKeyGlyphs",
        "noDuplicateShortcutHints",
        "disabledStateParity",
        "activationOwner",
        "safeKeyboardActivation",
        "noAccidentalExecution",
        "hintGeneration",
        "staleHintRejected",
        "wrongSurfaceHintRejected",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:keyboard_hint_label_parity_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-one hint scenario must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_one_boundaries() {
    for token in [
        "icon/image fallback redaction",
        "fallback icon kind",
        "no raw path/URL/content leakage",
        "footer/status persistence",
        "duplicate-footer rejection",
        "stale-status rejection",
        "keyboard hint label parity",
        "normalized shortcut tokens",
        "no accidental execution",
        "agentic_loop_twenty_one_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "docs and canonical skill must teach loop twenty-one boundary token {token}"
        );
    }
}
