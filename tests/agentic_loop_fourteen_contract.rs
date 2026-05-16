//! Source-level contract for fourteenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_fourteen_recipes() {
    for name in [
        "runtime-appearance-churn-focused-input-stress",
        "power-resume-window-generation-stress",
        "menu-tray-notification-modal-interruption-stress",
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
        "runRuntimeAppearanceChurnFocusedInputStressScenario",
        "runPowerResumeWindowGenerationStressScenario",
        "runMenuTrayNotificationModalInterruptionStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-fourteen function {function_name} must be wired"
        );
    }
}

#[test]
fn runtime_appearance_churn_pins_focused_input_layout_and_stale_repaint_guards() {
    for token in [
        "runtime-appearance-churn-focused-input-stress",
        "runtimeAppearanceChurnFocusedInput",
        "missing_runtime_appearance_churn_focused_input_receipt",
        "ui.appearanceChurnFocusedInput",
        "appearanceChurnId",
        "surfaceGenerationBefore",
        "surfaceGenerationAfter",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "inputTextBefore",
        "inputTextAfter",
        "visibleTextBefore",
        "visibleTextAfter",
        "cursorRangeBefore",
        "cursorRangeAfter",
        "selectionRangeBefore",
        "selectionRangeAfter",
        "inputLayoutBefore",
        "inputLayoutAfter",
        "remPx",
        "fontFamily",
        "fontSizePx",
        "scaleFactor",
        "themeTokenFingerprintBefore",
        "themeTokenFingerprintAfter",
        "rendererTokenGenerationBefore",
        "rendererTokenGenerationAfter",
        "staleTokenRepaintDetected",
        "layoutShiftPxMax",
        "visibleTextPreserved",
        "cursorRangePreserved",
        "selectionRangePreserved",
        "focusPreserved",
        "wrongSurfaceMutationRejected",
        "file_linear:runtime_appearance_churn_focused_input_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "runtime appearance churn stress must pin {token}"
        );
    }
}

#[test]
fn power_resume_window_generation_pins_epoch_stale_target_and_revalidation_guards() {
    for token in [
        "power-resume-window-generation-stress",
        "powerResumeWindowGeneration",
        "missing_power_resume_window_generation_receipt",
        "window.powerResumeGeneration",
        "resumeEventId",
        "sessionEpochBefore",
        "sessionEpochAfter",
        "appGenerationBefore",
        "appGenerationAfter",
        "powerStateBefore",
        "powerStateAfter",
        "sleepObservedAtMs",
        "wakeObservedAtMs",
        "preSleepTarget",
        "postWakeTarget",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "windowGeneration",
        "targetFingerprint",
        "preSleepTargetRejectedBeforeInput",
        "nativeInputDeliveryBlockedForStaleTarget",
        "batchDeliveryBlockedForStaleTarget",
        "gpuiEventDeliveryBlockedForStaleTarget",
        "screenshotDeliveryBlockedForStaleTarget",
        "targetReResolvedAfterWake",
        "stateReceiptAfterWake",
        "elementsReceiptAfterWake",
        "screenshotReceiptAfterWake",
        "screenshotStateRevalidatedAfterWake",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "selectedSemanticIdBefore",
        "selectedSemanticIdAfter",
        "staleScreenshotRejected",
        "wrongGenerationStateRejected",
        "cleanupConfirmed",
        "file_linear:power_resume_window_generation_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "power resume window generation stress must pin {token}"
        );
    }
}

#[test]
fn menu_tray_notification_interruption_pins_modal_focus_and_wrong_surface_guards() {
    for token in [
        "menu-tray-notification-modal-interruption-stress",
        "menuTrayNotificationModalInterruption",
        "missing_menu_tray_notification_modal_interruption_receipt",
        "platform.modalInterruptionFocus",
        "interruptionStressId",
        "hostSurface",
        "activeSurface",
        "modalStackGenerationBefore",
        "modalStackGenerationAfter",
        "activeModalIdBefore",
        "activeModalIdAfter",
        "parentAutomationWindowId",
        "parentOsWindowId",
        "modalAutomationWindowId",
        "modalOsWindowId",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "selectedSemanticIdBefore",
        "selectedSemanticIdAfter",
        "inputTextBefore",
        "inputTextAfter",
        "cursorRangeBefore",
        "cursorRangeAfter",
        "interruptions",
        "tray-menu",
        "app-menu",
        "notification",
        "interruptionId",
        "menuItemId",
        "notificationId",
        "notificationActionId",
        "actionTargetSurface",
        "wrongSurfaceActionRejected",
        "modalRemainedTopmost",
        "focusStolen",
        "selectionMutated",
        "submitCountBefore",
        "submitCountAfter",
        "modalClosedDuringInterruption",
        "parentSelectionMutated",
        "promptSubmittedDuringInterruption",
        "notificationActionDeliveredToWrongSurface",
        "trayActionDeliveredToWrongSurface",
        "appMenuActionDeliveredToWrongSurface",
        "focusRestoredToActiveModal",
        "file_linear:menu_tray_notification_modal_interruption_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "menu/tray/notification interruption stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_fourteen_boundaries() {
    for token in [
        "runtime appearance churn",
        "focused input",
        "stale token repaint",
        "power resume",
        "pre-sleep target",
        "post-wake target",
        "menu/tray/notification interruption",
        "wrong-surface action rejection",
        "topmost modal preservation",
        "agentic_loop_fourteen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-fourteen docs and skill must teach {token}"
        );
    }
}
