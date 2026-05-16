//! Source-level contract for thirty-second-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_two_recipes() {
    for name in [
        "acp-slash-mention-provider-visibility-stress",
        "acp-composer-token-keyboard-edit-parity-stress",
        "acp-transcript-stream-retry-virtualization-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn acp_slash_mention_provider_visibility_pins_readiness_rows_and_guardrails() {
    for token in [
        "acpSlashMentionProviderVisibilityReceipt",
        "runAcpSlashMentionProviderVisibilityStressScenario",
        "missing_acp_slash_mention_provider_visibility_receipt",
        "ux.acpSlashMentionProviderVisibility",
        "acpSlashMentionProviderVisibilityStressId",
        "providerHintCatalogId",
        "popupFamily",
        "triggerText",
        "queryText",
        "providerReadinessGeneration",
        "providerVisibilityRows",
        "providerHintText",
        "providerUnavailableReason",
        "providerLoadingState",
        "providerErrorRecoveredState",
        "hiddenUntilResourceAvailable",
        "dictationProviderVisibleWhenKitResourceReady",
        "browserHistoryProviderVisibleWhenCacheReady",
        "slashCommandProviderRows",
        "mentionProviderRows",
        "selectedRowSemanticId",
        "focusedRowSemanticId",
        "disabledProviderRowsNotAccepted",
        "staleProviderGenerationRejected",
        "wrongPopupRejected",
        "noRawProviderContentLeak",
        "file_linear:acp_slash_mention_provider_visibility_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn acp_composer_token_edit_parity_pins_atomic_delete_reorder_and_metadata() {
    for token in [
        "acpComposerTokenKeyboardEditParityReceipt",
        "runAcpComposerTokenKeyboardEditParityStressScenario",
        "missing_acp_composer_token_keyboard_edit_parity_receipt",
        "ux.acpComposerTokenKeyboardEditParity",
        "acpComposerTokenKeyboardEditParityStressId",
        "fixtureComposerTokenSetId",
        "hostSurfaceIdentity",
        "composerGeneration",
        "tokenSemanticIds",
        "tokenKinds",
        "tokenAliases",
        "tokenBounds",
        "cursorBeforeToken",
        "cursorAfterToken",
        "backspaceRemovesTokenAtomically",
        "deleteForwardRemovesTokenAtomically",
        "rangeRemoveReceipt",
        "moveTokenLeftReceipt",
        "moveTokenRightReceipt",
        "tokenOrderBefore",
        "tokenOrderAfter",
        "pendingContextPartsPreserved",
        "slashSkillContextPreserved",
        "pastedTokenMetadataPreserved",
        "cursorSelectionPreserved",
        "noPartialTokenTextLeak",
        "staleComposerGenerationRejected",
        "duplicateTokenIdRejected",
        "wrongHostRejected",
        "file_linear:acp_composer_token_keyboard_edit_parity_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn acp_transcript_stream_retry_virtualization_pins_stream_rows_retry_and_redaction() {
    for token in [
        "acpTranscriptStreamRetryVirtualizationReceipt",
        "runAcpTranscriptStreamRetryVirtualizationStressScenario",
        "missing_acp_transcript_stream_retry_virtualization_receipt",
        "ux.acpTranscriptStreamRetryVirtualization",
        "acpTranscriptStreamRetryVirtualizationStressId",
        "fixtureTranscriptId",
        "hostSurfaceIdentity",
        "threadGeneration",
        "transcriptGeneration",
        "virtualizedMessageWindow",
        "visibleMessageIds",
        "messageRowSemanticIds",
        "streamRunId",
        "streamChunkSequence",
        "activeAssistantMessageId",
        "monotonicChunkAppend",
        "scrollAnchorBefore",
        "scrollAnchorAfter",
        "bottomStickinessState",
        "userScrolledAwayPreserved",
        "assistantErrorMessageId",
        "errorKind",
        "errorVisibleText",
        "retryButtonSemanticId",
        "retryDraftFingerprint",
        "retryRequestGeneration",
        "retryRecoveryMessageId",
        "noStaleErrorAfterRecovery",
        "staleStreamChunkRejected",
        "wrongMessageRetryRejected",
        "virtualizedRowIdentityStable",
        "blankRowRejected",
        "noTranscriptBodyLeakInReceipts",
        "noAgentProcessSpawn",
        "noSecurityPrompt",
        "file_linear:acp_transcript_stream_retry_virtualization_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_two_boundaries() {
    for token in [
        "ACP slash/mention provider visibility",
        "ACP composer token keyboard edit parity",
        "ACP transcript stream retry virtualization",
        "agentic_loop_thirty_two_contract",
        "no-native-input",
        "no-native-pointer",
        "no-native-picker",
        "no-quick-look",
        "no-screen-capture",
        "no-security-prompts",
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
