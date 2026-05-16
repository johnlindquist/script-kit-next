//! Source-level contract for twenty-eighth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_eight_recipes() {
    for name in [
        "hotkey-prompt-transient-capture-cancel-stress",
        "process-manager-sort-detail-panel-stability-stress",
        "env-prompt-redacted-status-error-recovery-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn hotkey_prompt_transient_capture_pins_capture_cancel_and_no_registration_receipts() {
    for token in [
        "hotkeyPromptTransientCaptureCancelReceipt",
        "runHotkeyPromptTransientCaptureCancelStressScenario",
        "missing_hotkey_prompt_transient_capture_cancel_receipt",
        "ux.hotkeyPromptTransientCaptureCancel",
        "hotkeyPromptTransientCaptureCancelStressId",
        "promptType",
        "hotkeyPromptSurfaceId",
        "capturePanelSemanticId",
        "shortcutInputSemanticId",
        "placeholderVisibleText",
        "capturedChordTokens",
        "capturedHotkeyInfo",
        "simulateKeyCaptureReceipt",
        "escapeCancelReceipt",
        "cmdWCancelReceipt",
        "noConfigFingerprintChange",
        "noGlobalHotkeyRegistration",
        "noShortcutRecorderRoute",
        "cancelSubmitsNull",
        "focusRestoredToParent",
        "staleHotkeyCaptureRejected",
        "wrongSurfaceRejected",
        "usedNativeInput",
        "usedNativePointer",
        "file_linear:hotkey_prompt_transient_capture_cancel_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn process_manager_sort_detail_pins_header_detail_and_no_kill_receipts() {
    for token in [
        "processManagerSortDetailPanelStabilityReceipt",
        "runProcessManagerSortDetailPanelStabilityStressScenario",
        "missing_process_manager_sort_detail_panel_stability_receipt",
        "ux.processManagerSortDetailPanelStability",
        "processManagerSortDetailPanelStabilityStressId",
        "processFixtureIdentity",
        "tableHeaderSemanticIds",
        "sortKey",
        "sortDirection",
        "sortGeneration",
        "sectionHeaderRows",
        "sectionHeaderSelectableFalse",
        "selectedProcessSemanticId",
        "selectedPid",
        "detailPanelGeneration",
        "detailSourceIdentity",
        "detailTitle",
        "detailMetricRows",
        "cpuMemoryPidParity",
        "filterGeneration",
        "rowReanchorAfterSort",
        "visibleRowsMatchElements",
        "headerAriaSortLabel",
        "killActionDisabled",
        "noProcessSignalRequested",
        "staleSortGenerationRejected",
        "staleDetailRejected",
        "file_linear:process_manager_sort_detail_panel_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn env_prompt_redacted_status_pins_redaction_recovery_and_no_secret_write_receipts() {
    for token in [
        "envPromptRedactedStatusErrorRecoveryReceipt",
        "runEnvPromptRedactedStatusErrorRecoveryStressScenario",
        "missing_env_prompt_redacted_status_error_recovery_receipt",
        "ux.envPromptRedactedStatusErrorRecovery",
        "envPromptRedactedStatusErrorRecoveryStressId",
        "envFixtureIdentity",
        "statusGeneration",
        "statusKind",
        "statusVisibleText",
        "statusSemanticId",
        "inlineErrorSemanticId",
        "firstInvalidFieldSemanticId",
        "maskedValueVisible",
        "secretValueRedacted",
        "redactedSecretFingerprint",
        "noRawSecretLeak",
        "noSecretWrite",
        "noConfigFingerprintChange",
        "validEditClearsErrors",
        "submitDisabledReason",
        "footerSubmitDisabled",
        "focusPreservedAfterError",
        "staleStatusRejected",
        "wrongFieldErrorRejected",
        "visibleRowsMatchElements",
        "file_linear:env_prompt_redacted_status_error_recovery_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_eight_boundaries() {
    for token in [
        "HotkeyPrompt transient capture/cancel",
        "Process Manager sort/detail panel stability",
        "EnvPrompt redacted status/error recovery",
        "agentic_loop_twenty_eight_contract",
        "no-native-input",
        "no-native-pointer",
        "no-config-write",
        "no-global-hotkey-registration",
        "no-process-kill",
        "no-secret-write",
        "cleanupConfirmed",
    ] {
        assert!(
            SKILL.contains(token)
                || AUTOMATION.contains(token)
                || VERIFICATION.contains(token)
                || INDEX.contains(token)
                || SCENARIO.contains(token)
        );
    }
}
