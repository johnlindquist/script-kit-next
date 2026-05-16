//! Source-level contract for twenty-fifth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_five_recipes() {
    for name in [
        "shortcut-recorder-cancel-layering-stress",
        "inline-popover-anchor-resize-stress",
        "disabled-footer-hit-target-refusal-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
    for function_name in [
        "runShortcutRecorderCancelLayeringStressScenario",
        "runInlinePopoverAnchorResizeStressScenario",
        "runDisabledFooterHitTargetRefusalStressScenario",
    ] {
        assert!(INDEX.contains(function_name) || SCENARIO.contains(function_name));
    }
}

#[test]
fn shortcut_recorder_cancel_layering_pins_modal_cancel_config_and_restore_receipts() {
    for token in [
        "shortcut-recorder-cancel-layering-stress",
        "shortcutRecorderCancelLayeringReceipt",
        "runShortcutRecorderCancelLayeringStressScenario",
        "missing_shortcut_recorder_cancel_layering_receipt",
        "ux.shortcutRecorderCancelLayering",
        "shortcutRecorderCancelLayeringStressId",
        "surface",
        "action",
        "parentAutomationWindowId",
        "recorderAutomationWindowId",
        "parentSemanticSurface",
        "modalLayerReceipt",
        "parentBounds",
        "recorderBounds",
        "shellNarrowerThanParent",
        "titleText",
        "pressKeysPlaceholderVisible",
        "footerAbsent",
        "visibleCancelButton",
        "cancelMethod",
        "escapeCancels",
        "cmdWCancels",
        "backdropClickCancels",
        "parentClickCancels",
        "chordNotCapturedOnCancel",
        "configFingerprintBefore",
        "configFingerprintAfter",
        "configUnchanged",
        "globalHotkeyNotRegistered",
        "parentFocusRestored",
        "parentSelectionRestored",
        "staleRecorderRejected",
        "wrongParentRejected",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:shortcut_recorder_cancel_layering_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-five shortcut recorder scenario must pin {token}"
        );
    }
}

#[test]
fn inline_popover_anchor_resize_pins_anchor_clipping_z_order_and_keyboard_receipts() {
    for token in [
        "inline-popover-anchor-resize-stress",
        "inlinePopoverAnchorResizeReceipt",
        "runInlinePopoverAnchorResizeStressScenario",
        "missing_inline_popover_anchor_resize_receipt",
        "ux.inlinePopoverAnchorResize",
        "inlinePopoverAnchorResizeStressId",
        "familySamples",
        "family",
        "originAutomationWindowId",
        "popupAutomationWindowId",
        "parentSemanticSurface",
        "triggerText",
        "triggerRange",
        "anchorBoundsBeforeResize",
        "anchorBoundsAfterResize",
        "popupBoundsBeforeResize",
        "popupBoundsAfterResize",
        "resizeGeneration",
        "widthMode",
        "visibleRangeBeforeResize",
        "visibleRangeAfterResize",
        "selectedRowVisible",
        "selectedRowIdentityPreserved",
        "synopsisBounds",
        "footerRowBounds",
        "noSynopsisFooterOverlap",
        "noParentClipping",
        "noViewportOverflow",
        "zOrderAboveParent",
        "noFocusSteal",
        "keyboardSelectionPreserved",
        "keyboardFallbackAccepted",
        "screenshotToSemanticsAlignment",
        "strictCaptureTarget",
        "blankScreenshotRejected",
        "staleResizeGenerationRejected",
        "wrongPopupRejected",
        "usedNativeInput",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:inline_popover_anchor_resize_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-five inline popover scenario must pin {token}"
        );
    }
}

#[test]
fn disabled_footer_hit_target_refusal_pins_disabled_state_refusals_and_no_submit_receipts() {
    for token in [
        "disabled-footer-hit-target-refusal-stress",
        "disabledFooterHitTargetRefusalReceipt",
        "runDisabledFooterHitTargetRefusalStressScenario",
        "missing_disabled_footer_hit_target_refusal_receipt",
        "ux.disabledFooterHitTargetRefusal",
        "disabledFooterHitTargetRefusalStressId",
        "surfaceSamples",
        "surface",
        "fixture",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "activeFooter",
        "nativeFooterSurfaceId",
        "footerButtonSemanticId",
        "footerButtonLabel",
        "actionDisabled",
        "disabledReason",
        "disabledVisualState",
        "disabledAccessibleState",
        "keyboardEnterRefused",
        "footerShortcutRefused",
        "protocolFooterClickRefused",
        "cmdKActionsStillAvailable",
        "noSubmitReceipt",
        "submitAttemptGeneration",
        "sideEffectCountsBefore",
        "sideEffectCountsAfter",
        "stateFingerprintBefore",
        "stateFingerprintAfter",
        "focusPreserved",
        "selectionPreserved",
        "filterPreserved",
        "staleFooterGenerationRejected",
        "wrongSurfaceFooterRejected",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:disabled_footer_hit_target_refusal_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-five disabled footer scenario must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_five_boundaries() {
    for token in [
        "shortcut recorder cancel/layering",
        "unchanged config fingerprints",
        "no global hotkey registration",
        "inline popover anchor/resize",
        "no parent clipping or viewport overflow",
        "strict capture target",
        "disabled footer hit-target refusal",
        "no submit receipt",
        "unchanged side-effect counts",
        "agentic_loop_twenty_five_contract",
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
            "loop twenty-five docs/skill must teach {token}"
        );
    }
}
