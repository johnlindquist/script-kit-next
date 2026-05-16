//! Source-level contract for thirty-first-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_one_recipes() {
    for name in [
        "acp-footer-activity-indicator-stress",
        "acp-model-history-popover-visual-state-stress",
        "acp-context-insertion-preview-parity-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn acp_footer_activity_pins_dot_transitions_footer_owner_and_no_security_prompt() {
    for token in [
        "acpFooterActivityIndicatorReceipt",
        "runAcpFooterActivityIndicatorStressScenario",
        "missing_acp_footer_activity_indicator_receipt",
        "ux.acpFooterActivityIndicator",
        "acpFooterActivityIndicatorStressId",
        "fixtureAgentEventStreamId",
        "hostSurfaceIdentity",
        "footerOwner",
        "nativeFooterSurfaceId",
        "gpuiFooterDotStatus",
        "nativeFooterDotStatus",
        "activityStatusTransitions",
        "contextCapturePendingStatus",
        "toolCallStatus",
        "planUpdateStatus",
        "permissionWaitStatus",
        "cancelRestoresIdle",
        "footerRepaintGeneration",
        "dotPulseTokenStable",
        "modelLabelPreserved",
        "noGlobalAiFooterButton",
        "noAgentProcessSpawn",
        "noSecurityPrompt",
        "staleActivityRejected",
        "wrongHostRejected",
        "file_linear:acp_footer_activity_indicator_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn acp_model_history_popover_pins_visual_states_badges_and_redacted_previews() {
    for token in [
        "acpModelHistoryPopoverVisualStateReceipt",
        "runAcpModelHistoryPopoverVisualStateStressScenario",
        "missing_acp_model_history_popover_visual_state_receipt",
        "ux.acpModelHistoryPopoverVisualState",
        "acpModelHistoryPopoverVisualStateStressId",
        "fixturePopoverCatalogId",
        "popupFamily",
        "popupAutomationId",
        "promptPopupKind",
        "anchorBounds",
        "popupBounds",
        "selectedRowSemanticId",
        "focusedRowSemanticId",
        "rowVisualStateTokens",
        "currentModelBadge",
        "historyRecencyBadge",
        "historyPreviewRedactedFingerprint",
        "emptyFilteredState",
        "loadingRefreshState",
        "errorRecoveredState",
        "synopsisBounds",
        "selectionPreservedAfterFilter",
        "noTranscriptBodyLeak",
        "stalePopupSnapshotRejected",
        "wrongPopupRejected",
        "file_linear:acp_model_history_popover_visual_state_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn acp_context_insertion_preview_parity_pins_row_preview_context_token_and_stale_guards() {
    for token in [
        "acpContextInsertionPreviewParityReceipt",
        "runAcpContextInsertionPreviewParityStressScenario",
        "missing_acp_context_insertion_preview_parity_receipt",
        "ux.acpContextInsertionPreviewParity",
        "acpContextInsertionPreviewParityStressId",
        "sourceSurfaceIdentity",
        "destinationComposerIdentity",
        "portalSessionId",
        "sourceSelectionGeneration",
        "selectedRowSemanticId",
        "selectedRowPreviewFingerprint",
        "selectedRowPreviewTitle",
        "selectedRowPreviewKind",
        "previewGeneration",
        "acceptedContextPartUri",
        "insertedTokenAlias",
        "insertedTokenPreviewFingerprint",
        "composerGeneration",
        "replacementRange",
        "rowPreviewMatchesInsertedContext",
        "selectionPreservedAfterInsert",
        "selectionDriftRejected",
        "stalePreviewRejected",
        "wrongDestinationRejected",
        "noRawContentLeak",
        "file_linear:acp_context_insertion_preview_parity_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_one_boundaries() {
    for token in [
        "ACP footer activity indicators",
        "ACP model/history popover visual state",
        "ACP context insertion preview parity",
        "agentic_loop_thirty_one_contract",
        "no-native-input",
        "no-native-pointer",
        "no-security-prompts",
        "no-quick-look",
        "no-network",
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
