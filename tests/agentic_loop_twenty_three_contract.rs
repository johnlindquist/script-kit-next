//! Source-level contract for twenty-third-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_three_recipes() {
    for name in [
        "popup-focus-keycap-visual-semantics-stress",
        "reduced-motion-animation-disable-stress",
        "command-search-highlighting-accessory-badges-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
    for function_name in [
        "runPopupFocusKeycapVisualSemanticsStressScenario",
        "runReducedMotionAnimationDisableStressScenario",
        "runCommandSearchHighlightingAccessoryBadgesStressScenario",
    ] {
        assert!(INDEX.contains(function_name) || SCENARIO.contains(function_name));
    }
}

#[test]
fn popup_focus_keycap_visual_semantics_pins_focus_keycap_and_parent_receipts() {
    for token in [
        "popup-focus-keycap-visual-semantics-stress",
        "popupFocusKeycapVisualSemantics",
        "runPopupFocusKeycapVisualSemanticsStressScenario",
        "missing_popup_focus_keycap_visual_semantics_receipt",
        "ux.popupFocusKeycapVisualSemantics",
        "popupKeycapStressId",
        "surfaceSamples",
        "surface",
        "popupKind",
        "automationWindowId",
        "osWindowId",
        "parentAutomationWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "popupFocusKeycapVisualSemanticsReceipt",
        "keycapSamples",
        "keycapRole",
        "keycapLabel",
        "shortcutLabel",
        "normalizedShortcutTokens",
        "platformGlyph",
        "focused",
        "focusOwnerSemanticId",
        "focusedButtonSemanticId",
        "focusedKeycapMatchesFocusedButton",
        "escapeKeycapAvailable",
        "enterKeycapAvailable",
        "keycapFillToken",
        "keycapGlyphToken",
        "keycapTextToken",
        "focusRingToken",
        "dangerSemanticOnLabelNotKeycap",
        "disabledKeycapMuted",
        "shortcutGlyphNormalized",
        "popupIsTopmostOwner",
        "parentFocusUnchanged",
        "parentSelectionUnchanged",
        "staleFocusReceiptRejected",
        "wrongSurfaceKeycapRejected",
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
        "file_linear:popup_focus_keycap_visual_semantics_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-three popup keycap scenario must pin {token}"
        );
    }
}

#[test]
fn reduced_motion_animation_disable_pins_fixture_policy_stable_frames_and_no_tcc() {
    for token in [
        "reduced-motion-animation-disable-stress",
        "reducedMotionAnimationDisable",
        "runReducedMotionAnimationDisableStressScenario",
        "missing_reduced_motion_animation_disable_receipt",
        "ux.reducedMotionAnimationDisable",
        "reducedMotionStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "reducedMotionAnimationDisableReceipt",
        "motionPolicyReceipt",
        "motionPreferenceSource",
        "fixtureOnlyReducedMotion",
        "systemPreferenceNotRead",
        "systemPreferenceNotMutated",
        "animationSamples",
        "animationName",
        "transitionGeneration",
        "frameId",
        "frameClockPaused",
        "motionDurationMs",
        "effectiveDurationMs",
        "animatedOpacityStable",
        "animatedTransformStable",
        "spinnerHiddenOrStatic",
        "shimmerDisabled",
        "loadingPulseDisabled",
        "autoFocusPreserved",
        "selectedRowPreserved",
        "cursorPositionPreserved",
        "noLayoutShiftDuringMotionDisable",
        "staleMotionGenerationRejected",
        "wrongSurfaceMotionRejected",
        "noNativeInputRequired",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "networkAccessed",
        "externalServiceContacted",
        "cleanupConfirmed",
        "file_linear:reduced_motion_animation_disable_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-three reduced-motion scenario must pin {token}"
        );
    }
}

#[test]
fn command_search_highlighting_accessory_badges_pins_query_highlight_badge_and_catalog_receipts() {
    for token in [
        "command-search-highlighting-accessory-badges-stress",
        "commandSearchHighlightingAccessoryBadges",
        "runCommandSearchHighlightingAccessoryBadgesStressScenario",
        "missing_command_search_highlighting_accessory_badges_receipt",
        "ux.commandSearchHighlightAccessoryBadges",
        "commandHighlightBadgeStressId",
        "hostSamples",
        "host",
        "popupKind",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "commandSearchHighlightingAccessoryBadgesReceipt",
        "querySamples",
        "query",
        "searchGeneration",
        "commandRows",
        "semanticId",
        "commandId",
        "commandLabel",
        "sectionLabel",
        "highlightedRanges",
        "highlightText",
        "matchedQuery",
        "highlightMatchesFilter",
        "highlightDoesNotMutateLabel",
        "accessoryBadges",
        "badgeKind",
        "badgeLabel",
        "badgeTooltip",
        "shortcutBadge",
        "disabledBadge",
        "noOpBadge",
        "loadingBadge",
        "accessoryOrderStable",
        "badgesMatchActionCatalog",
        "disabledReasonVisible",
        "loadingReasonVisible",
        "staleBadgeRejected",
        "staleHighlightRejected",
        "wrongHostCommandRejected",
        "footerActionsStable",
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
        "file_linear:command_search_highlighting_accessory_badges_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loop twenty-three command highlight scenario must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_three_boundaries() {
    for token in [
        "popup focus/keycap visual semantics",
        "focused button/keycap parity",
        "danger semantics on labels rather than keycaps",
        "reduced-motion animation disable behavior",
        "fixture-only motion policy",
        "no System Settings or TCC mutation",
        "command search highlighting/accessory badges",
        "highlighted ranges",
        "accessory badge order",
        "agentic_loop_twenty_three_contract",
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
            "docs and canonical skill must teach loop twenty-three boundary token {token}"
        );
    }
}
