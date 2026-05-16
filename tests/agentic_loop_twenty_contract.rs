//! Source-level contract for twentieth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_recipes() {
    for name in [
        "toast-notification-queue-lifecycle-stress",
        "destructive-confirm-modal-safety-stress",
        "loading-skeleton-progress-restoration-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
        assert!(
            INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")),
            "plain help examples must advertise {name}"
        );
    }
    for function_name in [
        "runToastNotificationQueueLifecycleStressScenario",
        "runDestructiveConfirmModalSafetyStressScenario",
        "runLoadingSkeletonProgressRestorationStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-twenty function {function_name} must be wired"
        );
    }
}

#[test]
fn toast_notification_queue_lifecycle_pins_queue_bridge_bounds_and_no_action_receipts() {
    for token in [
        "toast-notification-queue-lifecycle-stress",
        "toastNotificationQueueLifecycle",
        "runToastNotificationQueueLifecycleStressScenario",
        "missing_toast_notification_queue_lifecycle_receipt",
        "ux.toastNotificationQueueLifecycle",
        "toastStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "toastQueueReceipt",
        "queueGeneration",
        "notificationBridgeGeneration",
        "toastSamples",
        "toastId",
        "message",
        "variant",
        "persistent",
        "autoHideMs",
        "duplicateCount",
        "visible",
        "visibleText",
        "createdAtMs",
        "expiresAtMs",
        "dismissedAtMs",
        "dismissReason",
        "autohideObserved",
        "manualDismissObserved",
        "duplicateCollapsed",
        "orderingPreserved",
        "maxVisibleCount",
        "toastBounds",
        "overlapPairs",
        "doesNotBlockInput",
        "doesNotCoverFooter",
        "staleToastRejected",
        "noActionExecutionFromToast",
        "networkAccessed",
        "externalServiceContacted",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "cleanupConfirmed",
        "file_linear:toast_notification_queue_lifecycle_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "toast notification lifecycle stress must pin {token}"
        );
    }
}

#[test]
fn destructive_confirm_modal_safety_pins_dry_run_focus_restore_and_no_system_command() {
    for token in [
        "destructive-confirm-modal-safety-stress",
        "destructiveConfirmModalSafety",
        "runDestructiveConfirmModalSafetyStressScenario",
        "missing_destructive_confirm_modal_safety_receipt",
        "ux.destructiveConfirmModalSafety",
        "confirmSafetyStressId",
        "hostSurface",
        "hostAutomationWindowId",
        "hostSemanticSurface",
        "stateBefore",
        "elementsBefore",
        "confirmReceipt",
        "confirmPromptId",
        "confirmRouteGeneration",
        "confirmSurfaceKind",
        "parentAutomationWindowId",
        "previousViewIdentity",
        "dangerActionId",
        "dangerActionLabel",
        "dangerLevel",
        "dryRunOnly",
        "destructiveActionFixture",
        "confirmButtonSemanticId",
        "cancelButtonSemanticId",
        "focusedButtonBefore",
        "tabFocusSamples",
        "enterResolutionSamples",
        "escapeCancelReceipt",
        "cancelResolvedFalse",
        "confirmResolvedTrue",
        "actionMutationCountBefore",
        "actionMutationCountAfter",
        "noMutationBeforeConfirm",
        "noMutationAfterCancel",
        "noExecutionWithoutConfirm",
        "destructiveCommandExecuted",
        "systemCommandRequested",
        "quitRequested",
        "trashMutationRequested",
        "restartRequested",
        "shutdownRequested",
        "staleConfirmRejected",
        "wrongSurfaceConfirmRejected",
        "focusRestored",
        "selectionRestored",
        "filterRestored",
        "routeStackRestored",
        "footerActionsSafe",
        "networkAccessed",
        "externalServiceContacted",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "cleanupConfirmed",
        "file_linear:destructive_confirm_modal_safety_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "destructive confirm modal safety stress must pin {token}"
        );
    }
}

#[test]
fn loading_skeleton_progress_restoration_pins_generations_activation_blocking_and_restore() {
    for token in [
        "loading-skeleton-progress-restoration-stress",
        "loadingSkeletonProgressRestoration",
        "runLoadingSkeletonProgressRestorationStressScenario",
        "missing_loading_skeleton_progress_restoration_receipt",
        "ux.loadingSkeletonProgressRestoration",
        "loadingSkeletonStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateBefore",
        "elementsBefore",
        "requestGeneration",
        "loadingReceipt",
        "loadingState",
        "skeletonVisible",
        "skeletonRows",
        "skeletonRowSemanticIds",
        "skeletonBounds",
        "progressReceipt",
        "progressText",
        "progressPercent",
        "progressMonotonic",
        "resultGeneration",
        "resultsReadyReceipt",
        "stateAfter",
        "elementsAfter",
        "realRowsVisible",
        "skeletonCleared",
        "noSkeletonAfterResults",
        "selectedSemanticIdBefore",
        "selectedSemanticIdAfter",
        "selectionRestored",
        "focusRestored",
        "filterTextPreserved",
        "scrollAnchorPreserved",
        "footerActionStateDuringLoading",
        "activationBlockedWhileLoading",
        "noSubmitDuringLoading",
        "staleLoadingGenerationRejected",
        "staleProgressRejected",
        "staleResultRejected",
        "noBlankFrame",
        "localFixtureOnly",
        "networkAccessed",
        "externalServiceContacted",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "cleanupConfirmed",
        "file_linear:loading_skeleton_progress_restoration_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "loading skeleton/progress restoration stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_boundaries() {
    for token in [
        "toast notification lifecycle",
        "duplicate collapse",
        "autohide and manual dismiss ordering",
        "destructive confirm safety",
        "dry-run-only fixture identity",
        "no real system command request",
        "loading skeleton/progress restoration",
        "activation blocking while loading",
        "stale loading/progress/result rejection",
        "agentic_loop_twenty_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "docs and canonical skill must teach loop twenty boundary token {token}"
        );
    }
}
