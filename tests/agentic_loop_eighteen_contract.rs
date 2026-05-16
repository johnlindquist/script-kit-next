//! Source-level contract for eighteenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_eighteen_recipes() {
    for name in [
        "empty-error-retry-state-ux-stress",
        "form-validation-inline-recovery-stress",
        "navigation-back-stack-history-stress",
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
        "runEmptyErrorRetryStateUxStressScenario",
        "runFormValidationInlineRecoveryStressScenario",
        "runNavigationBackStackHistoryStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-eighteen function {function_name} must be wired"
        );
    }
}

#[test]
fn empty_error_retry_state_ux_pins_empty_loading_error_retry_and_recovery() {
    for token in [
        "empty-error-retry-state-ux-stress",
        "emptyErrorRetryStateUx",
        "runEmptyErrorRetryStateUxStressScenario",
        "missing_empty_error_retry_state_ux_receipt",
        "ux.emptyErrorRetryState",
        "emptyRetryStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "query",
        "stateReceipt",
        "elementsReceipt",
        "emptyStateSamples",
        "emptyMessageSemanticId",
        "emptyMessageText",
        "emptyMessageVisible",
        "emptyIllustrationVisible",
        "loadingStateSamples",
        "loadingGeneration",
        "loadingMessageText",
        "loadingSpinnerVisible",
        "errorStateSamples",
        "errorGeneration",
        "errorBannerSemanticId",
        "errorMessageText",
        "errorSeverity",
        "retryButtonSemanticId",
        "retryButtonLabel",
        "retryButtonEnabled",
        "retryRequestId",
        "retryStateSamples",
        "retryAttempt",
        "retryStartedAt",
        "retryCompletedAt",
        "recoverySamples",
        "recoveredStateReceipt",
        "recoveredElementsReceipt",
        "recoveryClearsError",
        "selectionStableAcrossEmpty",
        "footerActionsSafeInEmpty",
        "noStaleErrorAfterRecovery",
        "noDisabledRetryTrap",
        "screenshotStateRevalidated",
        "cleanupConfirmed",
        "file_linear:empty_error_retry_state_ux_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "empty/error/retry UX stress must pin {token}"
        );
    }
}

#[test]
fn form_validation_inline_recovery_pins_errors_focus_preservation_and_submit_guard() {
    for token in [
        "form-validation-inline-recovery-stress",
        "formValidationInlineRecovery",
        "runFormValidationInlineRecoveryStressScenario",
        "missing_form_validation_inline_recovery_receipt",
        "ux.formValidationInlineRecovery",
        "formValidationStressId",
        "surface",
        "promptType",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "initialFormReceipt",
        "initialElementsReceipt",
        "fieldSamples",
        "fieldSemanticId",
        "fieldName",
        "fieldLabel",
        "fieldRole",
        "fieldRequired",
        "fieldValueBeforeInvalidSubmit",
        "invalidInputValue",
        "validInputValue",
        "validationRuleId",
        "fieldValidationGeneration",
        "invalidSubmitReceipt",
        "submitPrevented",
        "preventedAccidentalSubmit",
        "firstInvalidFieldSemanticId",
        "focusAfterInvalidSubmit",
        "cursorAfterInvalidSubmit",
        "inlineErrorSamples",
        "errorSemanticId",
        "errorText",
        "errorVisible",
        "errorLinkedFieldSemanticId",
        "errorSeverity",
        "inputPreservedAfterInvalidSubmit",
        "footerSubmitDisabledReason",
        "validEditReceipt",
        "errorsClearedOnValidEdit",
        "fieldValueAfterValidEdit",
        "focusPreservedDuringRecovery",
        "submitRecoveryReceipt",
        "submittedValueReceipt",
        "noStaleInlineErrors",
        "noCrossFieldErrorLeakage",
        "actionsDialogStillSafe",
        "escapeCancelStillSafe",
        "cleanupConfirmed",
        "file_linear:form_validation_inline_recovery_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "form validation inline recovery stress must pin {token}"
        );
    }
}

#[test]
fn navigation_back_stack_history_pins_transition_restore_actions_and_stale_state_guards() {
    for token in [
        "navigation-back-stack-history-stress",
        "navigationBackStackHistory",
        "runNavigationBackStackHistoryStressScenario",
        "missing_navigation_back_stack_history_receipt",
        "ux.navigationBackStackHistory",
        "navigationBackStackStressId",
        "navigationRunId",
        "originSurface",
        "originAutomationWindowId",
        "originSemanticSurface",
        "originStateReceipt",
        "originElementsReceipt",
        "originSelectionSemanticId",
        "originFilterText",
        "originScrollTop",
        "originFooterReceipt",
        "originFocusSemanticId",
        "transitionSamples",
        "transitionSequenceId",
        "transitionKind",
        "fromSurface",
        "toSurface",
        "surfaceStackGeneration",
        "routeStackDepthBefore",
        "routeStackDepthAfter",
        "triggerReceipt",
        "stateReceiptAfterTransition",
        "elementsReceiptAfterTransition",
        "actionsDialogReceipt",
        "actionsDiscoverabilityReceipt",
        "actionRowsVisible",
        "disabledActionSamples",
        "disabledReason",
        "noOpActionSemanticId",
        "noOpAffordanceVisible",
        "noAccidentalExecution",
        "backStackSamples",
        "backAction",
        "escapeReceipt",
        "backReceipt",
        "cmdKCloseReceipt",
        "returnToOriginReceipt",
        "returnedSurface",
        "selectionRestored",
        "filterRestored",
        "scrollRestored",
        "footerRestored",
        "focusRestored",
        "inputCursorRestored",
        "routeStackDrained",
        "noStalePopup",
        "noStaleSurfaceState",
        "wrongSurfaceBackRejected",
        "staleTransitionRejected",
        "cleanupConfirmed",
        "file_linear:navigation_back_stack_history_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "navigation/back-stack history stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_eighteen_boundaries() {
    for token in [
        "empty/error/retry state UX",
        "empty, loading, error, retry, and recovered states",
        "no stale error after recovery",
        "form validation and inline error recovery",
        "focus first invalid field",
        "preserve user input",
        "clear errors on valid edits",
        "prevent accidental submit",
        "navigation/back-stack history",
        "return-to-origin restore selection, filter, scroll, footer, and focus",
        "actions discoverability",
        "no-op affordances",
        "agentic_loop_eighteen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-eighteen docs and skill must teach {token}"
        );
    }
}
