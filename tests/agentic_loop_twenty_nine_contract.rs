//! Source-level contract for twenty-ninth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_twenty_nine_recipes() {
    for name in [
        "command-palette-breadcrumb-route-stack-stress",
        "root-source-chip-action-semantics-stress",
        "recent-history-dedupe-root-grouping-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn command_palette_breadcrumb_pins_route_stack_back_and_restore_receipts() {
    for token in [
        "commandPaletteBreadcrumbRouteStackReceipt",
        "runCommandPaletteBreadcrumbRouteStackStressScenario",
        "missing_command_palette_breadcrumb_route_stack_receipt",
        "ux.commandPaletteBreadcrumbRouteStack",
        "commandPaletteBreadcrumbRouteStackStressId",
        "actionsDialogHost",
        "routeStackDepth",
        "breadcrumbTrailLabels",
        "breadcrumbSemanticIds",
        "activeRouteId",
        "parentRouteSnapshot",
        "childRouteSnapshot",
        "drillDownActionId",
        "drillDownPushedReceipt",
        "breadcrumbBackReceipt",
        "escapeBackReceipt",
        "searchTextPreserved",
        "selectionRestoredToParent",
        "scrollAnchorRestored",
        "noOnSelectBeforeDrillDown",
        "noAccidentalExecution",
        "topmostOwnerBeforeKey",
        "staleRouteRejected",
        "wrongHostRejected",
        "file_linear:command_palette_breadcrumb_route_stack_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn root_source_chip_actions_pin_chip_decorations_status_refusal_and_stale_guards() {
    for token in [
        "rootSourceChipActionSemanticsReceipt",
        "runRootSourceChipActionSemanticsStressScenario",
        "missing_root_source_chip_action_semantics_receipt",
        "ux.rootSourceChipActionSemantics",
        "rootSourceChipActionSemanticsStressId",
        "fixtureSourceCatalogId",
        "inputRenderedText",
        "strippedSearchText",
        "sourceFilterSet",
        "sourceChipSemanticIds",
        "sourceChipRoles",
        "chipRemoveReceipt",
        "chipClearAllReceipt",
        "chipToggleExcludeReceipt",
        "filterInputDecorationsGeneration",
        "preflightFilterIndicators",
        "statusChipNonSelectable",
        "groupedRowsSuppressDisallowedSources",
        "inputHistoryRecallBlocked",
        "selectionPreservedAfterChipAction",
        "noStatusAsActionSubject",
        "noAccidentalExecution",
        "staleChipActionRejected",
        "wrongSurfaceRejected",
        "usedSystemPasteboard",
        "file_linear:root_source_chip_action_semantics_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn recent_history_dedupe_pins_grouping_metadata_and_stale_passive_receipts() {
    for token in [
        "recentHistoryDedupeRootGroupingReceipt",
        "runRecentHistoryDedupeRootGroupingStressScenario",
        "missing_recent_history_dedupe_root_grouping_receipt",
        "ux.recentHistoryDedupeRootGrouping",
        "recentHistoryDedupeRootGroupingStressId",
        "fixtureHistorySnapshotId",
        "sourceCatalogGeneration",
        "queryFrameKey",
        "rootFileFrameKey",
        "passiveFrameKey",
        "visibleResultsRoles",
        "groupSectionOrder",
        "filesSectionContiguous",
        "searchFilesContinuationStable",
        "dedupeKeys",
        "duplicateKeyCollisionsRejected",
        "recentFileSeedPoolFingerprint",
        "historyRowsMetadataOnly",
        "noFullTranscriptOrNoteBodyLeak",
        "stableSelectionKey",
        "rowFingerprintBeforeAfterCycles",
        "fallbackRowsSuppressedWhenSourceRowsPresent",
        "stalePassivePublishRejected",
        "localFixtureOnly",
        "file_linear:recent_history_dedupe_root_grouping_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_twenty_nine_boundaries() {
    for token in [
        "command palette breadcrumb route-stack",
        "root source-chip action semantics",
        "recent/history dedupe root grouping",
        "agentic_loop_twenty_nine_contract",
        "no-native-input",
        "no-native-pointer",
        "no-system-pasteboard",
        "no-network",
        "no-submit",
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
