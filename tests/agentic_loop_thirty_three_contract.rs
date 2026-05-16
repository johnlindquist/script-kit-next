//! Source-level contract for thirty-third-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_three_recipes() {
    for name in [
        "acp-plugin-skill-entry-thread-affinity-stress",
        "notes-cart-acp-handoff-dedupe-stress",
        "root-file-source-filter-pagination-footer-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn acp_plugin_skill_entry_thread_affinity_pins_target_thread_and_no_spawn() {
    for token in [
        "acpPluginSkillEntryThreadAffinityReceipt",
        "runAcpPluginSkillEntryThreadAffinityStressScenario",
        "missing_acp_plugin_skill_entry_thread_affinity_receipt",
        "ux.acpPluginSkillEntryThreadAffinity",
        "acpPluginSkillEntryThreadAffinityStressId",
        "fixtureSkillCatalogId",
        "entryPath",
        "hostSurfaceIdentity",
        "resolvedAcpTarget",
        "targetThreadId",
        "detachedThreadReused",
        "embeddedThreadReused",
        "selectedSkillId",
        "selectedSkillFileFingerprint",
        "slashTokenText",
        "slashTokenRange",
        "pendingSkillContextPartUri",
        "skillContextBoundToTargetThread",
        "composerGeneration",
        "returnOriginSnapshot",
        "noAutoSubmit",
        "noAgentProcessSpawn",
        "noSecurityPrompt",
        "staleLauncherSelectionRejected",
        "staleDetachedThreadRejected",
        "wrongHostRejected",
        "file_linear:acp_plugin_skill_entry_thread_affinity_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn notes_cart_acp_handoff_dedupe_pins_sandbox_store_dedupe_and_no_note_leak() {
    for token in [
        "notesCartAcpHandoffDedupeReceipt",
        "runNotesCartAcpHandoffDedupeStressScenario",
        "missing_notes_cart_acp_handoff_dedupe_receipt",
        "ux.notesCartAcpHandoffDedupe",
        "notesCartAcpHandoffDedupeStressId",
        "sandboxNotesStoreId",
        "fixtureNoteIds",
        "activeNoteId",
        "cartSnapshotGeneration",
        "cartItemIds",
        "cartDedupeKeys",
        "dedupedCartItemIds",
        "duplicateCartItemsRejected",
        "handoffSessionId",
        "destinationHostIdentity",
        "destinationAcpGeneration",
        "stagedContextPartUris",
        "inlineTokenAliases",
        "redactedPreviewFingerprints",
        "consumeRequestGeneration",
        "consumeIsDryRunOnly",
        "cancelRestoresCartSnapshot",
        "switchNoteClearsPreviousNoteContext",
        "wrongNoteConsumeRejected",
        "staleCartGenerationRejected",
        "noRawNoteBodyLeak",
        "noUserNotesMutation",
        "file_linear:notes_cart_acp_handoff_dedupe_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn root_file_source_filter_pagination_footer_pins_selection_scroll_and_provider_stability() {
    for token in [
        "rootFileSourceFilterPaginationFooterReceipt",
        "runRootFileSourceFilterPaginationFooterStressScenario",
        "missing_root_file_source_filter_pagination_footer_receipt",
        "ux.rootFileSourceFilterPaginationFooter",
        "rootFileSourceFilterPaginationFooterStressId",
        "fixtureFileProviderId",
        "sourceFilterSet",
        "renderedInputText",
        "strippedSearchText",
        "rootFrameKey",
        "providerGeneration",
        "pageGeneration",
        "pageSize",
        "visibleFileRowIds",
        "fileRowFingerprints",
        "searchFilesContinuationRowId",
        "selectedStableKeyBefore",
        "selectedStableKeyAfter",
        "selectedRowVisible",
        "selectedRowAboveFooter",
        "mainListScroll",
        "viewportHeight",
        "contentHeight",
        "footerHeight",
        "maxScrollTop",
        "nearBottomPageRequest",
        "pageAppendDoesNotChangeSelectedKey",
        "providerPublishDoesNotReplaceFrame",
        "duplicateFileKeyRejected",
        "fallbackSuppressedWhileSourceFilterActive",
        "statusChipsNonSelectable",
        "quickLookRefused",
        "stalePageGenerationRejected",
        "file_linear:root_file_source_filter_pagination_footer_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_three_boundaries() {
    for token in [
        "ACP plugin skill entry thread affinity",
        "Notes cart ACP handoff dedupe",
        "root Files source-filter pagination footer",
        "agentic_loop_thirty_three_contract",
        "no-native-input",
        "no-native-pointer",
        "no-security-prompts",
        "no-screen-capture",
        "sandbox-notes-store",
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
