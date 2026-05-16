//! Source-level contract for thirteenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirteen_recipes() {
    for name in [
        "display-migration-visual-bounds-stress",
        "native-picker-external-return-focus-stress",
        "drag-cancel-payload-scope-stress",
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
        "runDisplayMigrationVisualBoundsStressScenario",
        "runNativePickerExternalReturnFocusStressScenario",
        "runDragCancelPayloadScopeStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-thirteen function {function_name} must be wired"
        );
    }
}

#[test]
fn display_migration_visual_bounds_pins_display_text_focus_and_capture_identity() {
    for token in [
        "display-migration-visual-bounds-stress",
        "displayMigrationVisualBounds",
        "missing_display_migration_visual_bounds_receipt",
        "window.displayMigrationVisualBounds",
        "migrationGeneration",
        "sourceDisplayId",
        "targetDisplayId",
        "sourceDisplayBoundsPx",
        "targetDisplayBoundsPx",
        "displayScaleFactorBefore",
        "displayScaleFactorAfter",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "windowBoundsBefore",
        "windowBoundsAfter",
        "contentBoundsBefore",
        "contentBoundsAfter",
        "visibleTextBoundsBefore",
        "visibleTextBoundsAfter",
        "textClipState",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "selectedSemanticIdBefore",
        "selectedSemanticIdAfter",
        "screenshotSemanticAlignment",
        "wrongDisplayCaptureRejected",
        "staleDisplayMigrationRejected",
        "popupMainClobbered",
        "file_linear:display_migration_visual_bounds_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "display migration visual bounds stress must pin {token}"
        );
    }
}

#[test]
fn native_picker_external_return_pins_origin_focus_selection_and_foreign_event_guards() {
    for token in [
        "native-picker-external-return-focus-stress",
        "nativePickerExternalReturnFocus",
        "missing_native_picker_external_return_focus_receipt",
        "handoff.returnFocus",
        "originSurface",
        "originAutomationWindowId",
        "originOsWindowId",
        "originSemanticSurface",
        "originSelectionSemanticId",
        "originCursorRange",
        "originSurfaceGeneration",
        "handoffRequestId",
        "nativePickerWindowId",
        "externalBundleId",
        "externalWindowId",
        "returnGeneration",
        "returnTargetAutomationWindowId",
        "returnTargetOsWindowId",
        "focusRestoredToOrigin",
        "selectionRestoredToOrigin",
        "cursorRangeRestored",
        "staleWindowEventRejected",
        "foreignWindowEventRejected",
        "foreignWindowEventDelivered",
        "staleReturnTargetUsed",
        "selectionMutatedDuringHandoff",
        "actionSubmittedDuringHandoff",
        "file_linear:native_picker_external_return_focus_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "native picker external return focus stress must pin {token}"
        );
    }
}

#[test]
fn drag_cancel_payload_scope_pins_payload_hover_cleanup_and_side_effect_guards() {
    for token in [
        "drag-cancel-payload-scope-stress",
        "dragCancelPayloadScope",
        "missing_drag_cancel_payload_scope_receipt",
        "drag.payloadScope",
        "dragSessionId",
        "originSurface",
        "originAutomationWindowId",
        "originOsWindowId",
        "originSelectedSemanticId",
        "originFocusSemanticId",
        "originSurfaceGeneration",
        "payloadFingerprint",
        "redactedPayloadPreview",
        "payloadKind",
        "dragPreviewIdentity",
        "payloadScopedToDragSession",
        "hoverTargetBeforeCancel",
        "dropTargetBeforeCancel",
        "cancelMethod",
        "escapeDuringDragCancelled",
        "dragSessionClosed",
        "originStateRestored",
        "hoverTargetsCleared",
        "dropTargetsCleared",
        "clipboardChangeCountBefore",
        "clipboardChangeCountAfter",
        "fileMutationCount",
        "temporaryFileCount",
        "partialPayloadDelivered",
        "attachmentInsertedDuringCancel",
        "promptSubmittedDuringCancel",
        "foreignDropRejected",
        "staleDragSessionRejected",
        "file_linear:drag_cancel_payload_scope_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "drag cancel payload scope stress must pin {token}"
        );
    }
}

#[test]
fn canonical_skill_and_verification_docs_teach_loop_thirteen_boundaries() {
    for token in [
        "display migration visual bounds",
        "native picker",
        "external app return",
        "drag cancellation",
        "wrong-display capture rejection",
        "stale or foreign window event",
        "hover/drop target cleanup",
        "agentic_loop_thirteen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-thirteen docs and skill must teach {token}"
        );
    }
}
