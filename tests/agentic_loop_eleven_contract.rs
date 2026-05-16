//! Source-level contract for eleventh-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_eleven_recipes() {
    for name in [
        "modal-stack-arbitration-stress",
        "cross-surface-export-provenance-stress",
        "dev-session-recovery-stale-target-stress",
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
        "runModalStackArbitrationStressScenario",
        "runCrossSurfaceExportProvenanceStressScenario",
        "runDevSessionRecoveryStaleTargetStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-eleven function {function_name} must be wired"
        );
    }
}

#[test]
fn modal_stack_arbitration_pins_topmost_owner_and_parent_restore_receipts() {
    for token in [
        "modal-stack-arbitration-stress",
        "modalStackArbitration",
        "missing_modal_stack_arbitration_receipt",
        "requestedStack",
        "actionsDialog",
        "confirmPopup",
        "promptPopup",
        "stackGeneration",
        "topmostOwnerOnly",
        "keyDispatches",
        "beforeTopOwner",
        "handledBy",
        "afterTopOwner",
        "lowerOwnersMutated",
        "escape",
        "cmd+w",
        "enter",
        "parentSelectionFingerprintBefore",
        "parentSelectionFingerprintAfter",
        "parentFocusBefore",
        "parentFocusAfter",
        "parentSelectionRestored",
        "parentFocusRestored",
        "actionsDialogRouteRestored",
        "promptInputRestored",
        "file_linear:modal_stack_arbitration_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "modal stack arbitration stress must pin {token}"
        );
    }
}

#[test]
fn cross_surface_provenance_pins_payload_redaction_stale_source_and_cleanup_receipts() {
    for token in [
        "cross-surface-export-provenance-stress",
        "crossSurfaceExport",
        "missing_cross_surface_export_provenance_receipt",
        "selectionSemanticId",
        "selectionGeneration",
        "visibleRowFingerprint",
        "filterGenerationBefore",
        "filterGenerationAfter",
        "redactedVisibleRows",
        "forbiddenVisibleFields",
        "payloadUri",
        "payloadFingerprint",
        "provenanceChain",
        "sourceGenerationMatched",
        "destination",
        "acp-composer",
        "notes",
        "composerGeneration",
        "notesRevision",
        "insertionRange",
        "acceptedContextPartUri",
        "insertedPayloadFingerprint",
        "staleSourceGenerationRejected",
        "wrongPayloadAccepted",
        "sourceSnapshotRecheckedBeforeInsert",
        "visibleRowsLeakedPrivatePath",
        "rawClipboardTextLogged",
        "payloadContentLogged",
        "file_linear:cross_surface_export_provenance_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "cross-surface provenance stress must pin {token}"
        );
    }
}

#[test]
fn stale_window_recovery_pins_rejection_reresolution_no_stale_input_and_cleanup_receipts() {
    for token in [
        "dev-session-recovery-stale-target-stress",
        "runDevSessionRecoveryStaleTargetStressScenario",
        "sessionRecovery",
        "computeAgenticSessionEpoch",
        "targetSessionEpoch",
        "initialSession",
        "initialTarget",
        "restartMode",
        "stop-start",
        "epochChanged",
        "staleTargetProbe",
        "session_epoch_mismatch",
        "inputGate",
        "blockedBeforeDelivery",
        "inputNotSentToStaleWindow",
        "attemptedNativeInput",
        "attemptedBatchOnStaleTarget",
        "attemptedGpuiEventOnStaleTarget",
        "reResolvedTarget",
        "usedReResolvedTarget",
        "finalProbe",
        "restartedSession",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "stale window recovery stress must pin {token}"
        );
    }
    let start = SCENARIO
        .find("runDevSessionRecoveryStaleTargetStressScenario")
        .expect("recovery function must exist");
    let body = &SCENARIO[start
        ..SCENARIO[start..]
            .find("// ---------------------------------------------------------------------------")
            .unwrap()
            + start];
    for forbidden in ["macos-input.ts", "captureWindow", "verify-shot.ts"] {
        assert!(
            !body.contains(forbidden),
            "recovery function must not use mutating or screenshot-only helper {forbidden}"
        );
    }
}

#[test]
fn canonical_skill_and_verification_docs_teach_loop_eleven_hard_scenario_boundaries() {
    for token in [
        "modal stack",
        "topmost owner",
        "cross-surface export",
        "session epoch",
        "stale-target",
        "agentic_loop_eleven_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-eleven docs and skill must teach {token}"
        );
    }
}
