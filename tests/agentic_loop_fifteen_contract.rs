//! Source-level contract for fifteenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_fifteen_recipes() {
    for name in [
        "stream-progress-cancel-visual-stability-stress",
        "dictation-media-permission-readiness-churn-stress",
        "animation-frame-capture-determinism-stress",
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
        "runStreamProgressCancelVisualStabilityStressScenario",
        "runDictationMediaPermissionReadinessChurnStressScenario",
        "runAnimationFrameCaptureDeterminismStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-fifteen function {function_name} must be wired"
        );
    }
}

#[test]
fn stream_progress_cancel_pins_monotonic_cancel_and_stale_chunk_guards() {
    for token in [
        "stream-progress-cancel-visual-stability-stress",
        "streamProgressCancelVisualStability",
        "missing_stream_progress_cancel_visual_stability_receipt",
        "stream.progressCancelVisualStability",
        "streamRunId",
        "originSurface",
        "streamGenerationBefore",
        "streamGenerationAfterCancel",
        "progressSamples",
        "progressSequenceMonotonic",
        "visibleProgressMonotonic",
        "visibleTextSamples",
        "cancelRequestId",
        "cancelRequestedAtMs",
        "cancelAcknowledgedAtMs",
        "cancelStateVisible",
        "lastPaintedChunkSequence",
        "staleChunkAfterCancelRejected",
        "staleChunkIdsRejected",
        "staleChunkRepaintDetected",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "cursorRangeBefore",
        "cursorRangeAfter",
        "submitCountBefore",
        "submitCountAfter",
        "layoutShiftPxMax",
        "screenshotStateRevalidated",
        "cleanupConfirmed",
        "file_linear:stream_progress_cancel_visual_stability_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "stream progress cancel stress must pin {token}"
        );
    }
}

#[test]
fn dictation_media_permission_readiness_churn_pins_passive_setup_and_target_guards() {
    for token in [
        "dictation-media-permission-readiness-churn-stress",
        "dictationMediaPermissionReadinessChurn",
        "missing_dictation_media_permission_readiness_churn_receipt",
        "media.dictationPermissionReadinessChurn",
        "dictationSessionId",
        "targetSurface",
        "targetAutomationWindowId",
        "targetOsWindowId",
        "targetSemanticSurface",
        "targetFingerprint",
        "setupMode",
        "passiveSetupConfirmed",
        "microphonePermissionBefore",
        "microphonePermissionAfter",
        "microphonePermissionGenerationBefore",
        "microphonePermissionGenerationAfter",
        "modelReadinessBefore",
        "modelReadinessAfter",
        "modelReadinessGenerationBefore",
        "modelReadinessGenerationAfter",
        "readinessChurnEvents",
        "transcriptGenerationId",
        "transcriptTargetFingerprint",
        "transcriptInsertedRange",
        "transcriptPreviewRedacted",
        "transcriptDeliveredToTarget",
        "wrongTargetDeliveryRejected",
        "autoSubmitPrevented",
        "submitCountBefore",
        "submitCountAfter",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "cursorRangeBefore",
        "cursorRangeAfter",
        "noSystemSettingsOpened",
        "noTccMutationAttempted",
        "cleanupConfirmed",
        "file_linear:dictation_media_permission_readiness_churn_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "dictation/media readiness churn stress must pin {token}"
        );
    }
}

#[test]
fn animation_frame_capture_pins_deterministic_sampling_and_occlusion_guards() {
    for token in [
        "animation-frame-capture-determinism-stress",
        "animationFrameCaptureDeterminism",
        "missing_animation_frame_capture_determinism_receipt",
        "visual.animationFrameCaptureDeterminism",
        "animationStressId",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "capturePlanId",
        "animationGenerationBefore",
        "animationGenerationAfter",
        "frameSampleCount",
        "frameIntervalMs",
        "animationClockSource",
        "frameSamples",
        "captureSequence",
        "frameId",
        "animationFrameId",
        "stateReceipt",
        "elementsReceipt",
        "screenshotReceipt",
        "visibleTextFingerprint",
        "layoutFingerprint",
        "occlusionPairs",
        "spinnerSemanticId",
        "skeletonSemanticIds",
        "frameIdsStrictlyIncreasing",
        "captureFrameIdsStable",
        "stateBeforeScreenshot",
        "screenshotTargetMatched",
        "screenshotStateRevalidated",
        "blankFrameRejected",
        "motionOcclusionDetected",
        "visibleTextNotOccluded",
        "layoutFingerprintStable",
        "staleFrameRejected",
        "wrongWindowFrameRejected",
        "cleanupConfirmed",
        "file_linear:animation_frame_capture_determinism_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "animation frame capture stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_fifteen_boundaries() {
    for token in [
        "streaming progress cancellation",
        "monotonic progress",
        "stale post-cancel chunk",
        "dictation/media permission readiness churn",
        "passive microphone",
        "model readiness generation",
        "animation frame capture determinism",
        "stable frame",
        "motion occlusion",
        "agentic_loop_fifteen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-fifteen docs and skill must teach {token}"
        );
    }
}
