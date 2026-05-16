//! Source-level contract for nineteenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_nineteen_recipes() {
    for name in [
        "long-text-wrap-resize-surface-stress",
        "actions-command-discoverability-noop-stress",
        "dense-list-detail-preview-readability-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
        assert!(
            INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")),
            "plain help examples must advertise {name}"
        );
    }
    for function_name in [
        "runLongTextWrapResizeSurfaceStressScenario",
        "runActionsCommandDiscoverabilityNoopStressScenario",
        "runDenseListDetailPreviewReadabilityStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-nineteen function {function_name} must be wired"
        );
    }
}

#[test]
fn long_text_wrap_resize_pins_text_bounds_accessible_full_text_and_footer_collision_guards() {
    for token in [
        "long-text-wrap-resize-surface-stress",
        "longTextWrapResizeSurface",
        "runLongTextWrapResizeSurfaceStressScenario",
        "missing_long_text_wrap_resize_surface_receipt",
        "ux.longTextWrapResizeSurface",
        "longTextStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "widthSamples",
        "widthMode",
        "resizeGeneration",
        "windowBounds",
        "contentBounds",
        "inputBounds",
        "listBounds",
        "footerBounds",
        "fixtureSamples",
        "fixtureId",
        "longNameFixture",
        "longPathFixture",
        "longDescriptionFixture",
        "multilineSnippetFixture",
        "semanticId",
        "role",
        "fullText",
        "visibleText",
        "textBounds",
        "renderedTextBounds",
        "elementBounds",
        "availableWidth",
        "measuredWidth",
        "wrapLineCount",
        "clippingState",
        "truncationIntent",
        "tooltipOrAccessibleFullText",
        "accessibleFullText",
        "overlapPairs",
        "footerCollision",
        "inputCollision",
        "lostAccessibleText",
        "resizeTransitionSamples",
        "fromWidthMode",
        "toWidthMode",
        "selectionPreserved",
        "focusPreserved",
        "noLayoutShiftBeyondContainer",
        "noFooterCollision",
        "screenshotStateRevalidated",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "cleanupConfirmed",
        "file_linear:long_text_wrap_resize_surface_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "long text wrapping/resizing UX stress must pin {token}"
        );
    }
}

#[test]
fn actions_command_discoverability_noop_pins_disabled_rows_keyboard_guards_and_no_execution() {
    for token in [
        "actions-command-discoverability-noop-stress",
        "actionsCommandDiscoverabilityNoop",
        "runActionsCommandDiscoverabilityNoopStressScenario",
        "missing_actions_command_discoverability_noop_receipt",
        "ux.actionsCommandDiscoverabilityNoop",
        "actionsNoopStressId",
        "hostSamples",
        "hostSurface",
        "hostAutomationWindowId",
        "hostSemanticSurface",
        "hostStateBefore",
        "hostElementsBefore",
        "actionsDialogReceipt",
        "parentAutomationWindowId",
        "routeStackDepth",
        "actionsVisible",
        "filterText",
        "focusedSemanticId",
        "actionRowSamples",
        "rowSemanticId",
        "actionId",
        "label",
        "section",
        "rowKind",
        "actionable",
        "disabled",
        "no-op",
        "enabled",
        "disabledReason",
        "noOpReason",
        "keyboardSelectable",
        "keyboardSkipOrExplainReceipt",
        "enterWouldExecute",
        "keyboardSelectionSamples",
        "fromSemanticId",
        "toSemanticId",
        "skippedSemanticIds",
        "skipReasons",
        "activationGuardSamples",
        "attemptedSemanticId",
        "attemptedActionId",
        "activationPrevented",
        "preventedReason",
        "noAccidentalExecution",
        "hostMutationCountBefore",
        "hostMutationCountAfter",
        "hostStateAfter",
        "hostMutationReceipt",
        "selectionUnchanged",
        "filterUnchanged",
        "scrollUnchanged",
        "footerUnchanged",
        "focusRestored",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "cleanupConfirmed",
        "state_not_yet_measured",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "actions discoverability no-op stress must pin {token}"
        );
    }
}

#[test]
fn dense_list_detail_preview_readability_pins_identity_preview_and_resize_guards() {
    for token in [
        "dense-list-detail-preview-readability-stress",
        "denseListDetailPreviewReadability",
        "runDenseListDetailPreviewReadabilityStressScenario",
        "missing_dense_list_detail_preview_readability_receipt",
        "ux.denseListDetailPreviewReadability",
        "densePreviewStressId",
        "surfaceSamples",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "query",
        "stateReceipt",
        "elementsReceipt",
        "listPaneSamples",
        "listPaneBounds",
        "visibleRowCount",
        "selectedRowSemanticId",
        "selectedStableKey",
        "selectedRowBounds",
        "selectedRowTextBounds",
        "selectedRowVisible",
        "selectedRowAboveFooter",
        "rowIdentityVisible",
        "previewPaneSamples",
        "previewPaneBounds",
        "previewSourceStableKey",
        "previewMatchesSelectedStableKey",
        "previewTitleSemanticId",
        "previewTitleText",
        "previewTitleBounds",
        "previewBodySemanticId",
        "previewBodyVisibleLineCount",
        "previewBodyBounds",
        "previewMetadataChips",
        "chipSemanticId",
        "chipLabel",
        "chipBounds",
        "chipReadable",
        "chipOverlaps",
        "previewFooterCollision",
        "previewListOverlap",
        "selectionChangeSamples",
        "selectionGeneration",
        "fromStableKey",
        "toStableKey",
        "previewGenerationBefore",
        "previewGenerationAfter",
        "previewUpdated",
        "noPreviewStaleAfterSelection",
        "focusPreserved",
        "filterGenerationSamples",
        "filterGeneration",
        "filterText",
        "rowFingerprintBefore",
        "rowFingerprintAfter",
        "previewStaleRejected",
        "selectedRowReanchored",
        "resizeSamples",
        "resizeGeneration",
        "widthMode",
        "noColumnOverlap",
        "previewReadable",
        "metadataChipsReadable",
        "footerActionsReadable",
        "footerActionSamples",
        "footerActionSemanticId",
        "label",
        "enabled",
        "overlapsPreview",
        "overlapsSelectedRow",
        "rowPreviewIdentityMatches",
        "usedNativeInput",
        "usedScreenshot",
        "openedSystemSettings",
        "mutatedTcc",
        "installedAgents",
        "triggeredSecurityPrompt",
        "cleanupConfirmed",
        "file_linear:dense_list_detail_preview_readability_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "dense list/detail preview readability stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_nineteen_boundaries() {
    for token in [
        "long text wrapping/resizing UX stress",
        "long names, paths, descriptions, and multi-line snippets",
        "full accessible text",
        "footer collisions",
        "actions/command discoverability no-op stress",
        "Cmd-K action popup row measurement",
        "keyboard selection skips or explains disabled actions",
        "no-op rows cannot accidentally execute",
        "dense list/detail preview readability stress",
        "row identity, preview text, metadata chips, keyboard focus, and footer actions",
        "filtering, selection changes, and resize",
        "agentic_loop_nineteen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "docs and canonical skill must teach loop nineteen boundary token {token}"
        );
    }
}
