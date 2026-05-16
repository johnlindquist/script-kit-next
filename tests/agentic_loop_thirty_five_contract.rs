//! Source-level contract for thirty-fifth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_five_recipes() {
    for name in [
        "notes-markdown-preview-scroll-sync-stress",
        "quick-terminal-ansi-scrollback-search-stress",
        "script-output-inspector-folding-recovery-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn notes_markdown_preview_scroll_sync_pins_editor_preview_and_focus() {
    for token in [
        "notesMarkdownPreviewScrollSyncReceipt",
        "runNotesMarkdownPreviewScrollSyncStressScenario",
        "missing_notes_markdown_preview_scroll_sync_receipt",
        "ux.notesMarkdownPreviewScrollSync",
        "notesMarkdownPreviewScrollSyncStressId",
        "sandboxNotesStoreId",
        "fixtureNoteIds",
        "activeNoteIdBefore",
        "activeNoteIdAfter",
        "markdownFixtureIds",
        "editorGeneration",
        "previewGeneration",
        "renderedMarkdownBlockIds",
        "previewBlockFingerprints",
        "editorCursorBefore",
        "editorCursorAfter",
        "editorSelectionRange",
        "editorScrollAnchor",
        "previewScrollAnchor",
        "scrollSyncDeltaPx",
        "splitPaneBounds",
        "previewToggleReceipt",
        "switchNoteCleanupReceipt",
        "focusRestoredToEditor",
        "noUserNotesMutation",
        "noRawNoteBodyLeak",
        "stalePreviewGenerationRejected",
        "wrongNoteMutationRejected",
        "file_linear:notes_markdown_preview_scroll_sync_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn quick_terminal_ansi_scrollback_pins_ansi_wide_search_and_no_shell() {
    for token in [
        "quickTerminalAnsiScrollbackSearchReceipt",
        "runQuickTerminalAnsiScrollbackSearchStressScenario",
        "missing_quick_terminal_ansi_scrollback_search_receipt",
        "ux.quickTerminalAnsiScrollbackSearch",
        "quickTerminalAnsiScrollbackSearchStressId",
        "fixtureTerminalTranscriptId",
        "terminalSurfaceId",
        "transcriptGeneration",
        "ansiRunIds",
        "sgrTokenRuns",
        "wideCellGraphemeIds",
        "combiningMarkCellIds",
        "hyperlinkSpanIds",
        "redactedHrefFingerprints",
        "stderrBlockIds",
        "promptContinuationRows",
        "scrollbackViewportRows",
        "viewportRowRange",
        "searchQueryGeneration",
        "searchHitIds",
        "highlightedCellRanges",
        "selectedSearchHitVisible",
        "wrapContinuationMarkers",
        "cursorCellBounds",
        "promptLineBounds",
        "footerInputNonOverlapping",
        "staleTranscriptGenerationRejected",
        "noShellCommandSpawned",
        "noRawHyperlinkLeak",
        "file_linear:quick_terminal_ansi_scrollback_search_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn script_output_inspector_pins_folding_retry_and_no_spawn() {
    for token in [
        "scriptOutputInspectorFoldingRecoveryReceipt",
        "runScriptOutputInspectorFoldingRecoveryStressScenario",
        "missing_script_output_inspector_folding_recovery_receipt",
        "ux.scriptOutputInspectorFoldingRecovery",
        "scriptOutputInspectorFoldingRecoveryStressId",
        "fixtureScriptRunId",
        "outputFixtureIds",
        "outputStreamGeneration",
        "stdoutBlockIds",
        "stderrBlockIds",
        "ansiStackFrameIds",
        "jsonLineIds",
        "progressRewriteGeneration",
        "exitBadgeKind",
        "exitBadgeBounds",
        "filterText",
        "highlightedOutputRanges",
        "stderrFoldStateBefore",
        "stderrFoldStateAfter",
        "stackTraceExpandedState",
        "clearFilterRestoresOutput",
        "retryDryRunReceipt",
        "retryDoesNotSpawnHandler",
        "selectionScrollAnchorRestored",
        "noOutputInterleaveDrift",
        "staleOutputGenerationRejected",
        "wrongRunMutationRejected",
        "noHandlerSpawn",
        "noProcessKill",
        "file_linear:script_output_inspector_folding_recovery_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_five_boundaries() {
    for token in [
        "Notes markdown preview scroll sync",
        "Quick Terminal ANSI scrollback search",
        "script output inspector folding recovery",
        "agentic_loop_thirty_five_contract",
        "no-native-input",
        "no-native-pointer",
        "no-external-services",
        "no-shell-command",
        "no-handler-spawn",
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
