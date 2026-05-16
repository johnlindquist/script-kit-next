//! Source-level contract for sixteenth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_sixteen_recipes() {
    for name in [
        "accessibility-tree-semantic-parity-stress",
        "rtl-bidi-emoji-text-rendering-stress",
        "high-volume-virtualized-list-stability-stress",
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
        "runAccessibilityTreeSemanticParityStressScenario",
        "runRtlBidiEmojiTextRenderingStressScenario",
        "runHighVolumeVirtualizedListStabilityStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-sixteen function {function_name} must be wired"
        );
    }
}

#[test]
fn accessibility_tree_semantic_parity_pins_roles_labels_focus_and_activation() {
    for token in [
        "accessibility-tree-semantic-parity-stress",
        "accessibilityTreeSemanticParity",
        "runAccessibilityTreeSemanticParityStressScenario",
        "missing_accessibility_tree_semantic_parity_receipt",
        "accessibility.treeSemanticParity",
        "accessibilityAuditId",
        "surfaceSamples",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "stateReceipt",
        "elementsReceipt",
        "axTreeReceipt",
        "screenshotReceipt",
        "visibleControlIds",
        "automationElementIds",
        "axNodeIds",
        "roleParity",
        "labelParity",
        "focusOrder",
        "tabOrder",
        "disabledStateParity",
        "keyboardActivationParity",
        "activationPlan",
        "sideEffectSafe",
        "activationMethod",
        "activatedSemanticId",
        "activationResult",
        "disabledActivationPrevented",
        "focusSemanticIdBefore",
        "focusSemanticIdAfter",
        "hitTargetBounds",
        "screenshotSemanticAlignment",
        "missingAxNodes",
        "extraAxNodes",
        "staleAxTreeRejected",
        "wrongWindowAxRejected",
        "accessibilityPermissionBefore",
        "noSystemSettingsOpened",
        "noTccMutationAttempted",
        "cleanupConfirmed",
        "file_linear:accessibility_tree_semantic_parity_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "accessibility tree semantic parity stress must pin {token}"
        );
    }
}

#[test]
fn rtl_bidi_emoji_text_rendering_pins_graphemes_cursor_selection_and_filtering() {
    for token in [
        "rtl-bidi-emoji-text-rendering-stress",
        "rtlBidiEmojiTextRendering",
        "runRtlBidiEmojiTextRenderingStressScenario",
        "missing_rtl_bidi_emoji_text_rendering_receipt",
        "text.rtlBidiEmojiTextRendering",
        "bidiStressId",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "inputSemanticId",
        "rawText",
        "normalizedText",
        "textDirectionBase",
        "directionRuns",
        "bidiEmbeddingLevels",
        "graphemeClusters",
        "clusterBoundaries",
        "emojiZwJSequences",
        "combiningMarkSequences",
        "cursorSamples",
        "cursorLogicalIndex",
        "cursorUtf16Index",
        "cursorVisualRect",
        "cursorInVisibleWindow",
        "selectionSamples",
        "selectionLogicalRange",
        "selectionUtf16Range",
        "selectionVisualRects",
        "visibleTextBounds",
        "renderedTextBounds",
        "availableWidth",
        "measuredWidth",
        "truncationState",
        "accessibleFullText",
        "searchFilterSamples",
        "normalizedQuery",
        "matchingSemanticIds",
        "filterResultFingerprint",
        "backspaceClusterAtomicity",
        "selectionPreservedAcrossFilter",
        "cursorRangeBefore",
        "cursorRangeAfter",
        "screenshotStateRevalidated",
        "staleTextLayoutRejected",
        "wrongSurfaceMutationRejected",
        "noAccidentalSubmit",
        "cleanupConfirmed",
        "file_linear:rtl_bidi_emoji_text_rendering_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "RTL/bidi/emoji text rendering stress must pin {token}"
        );
    }
}

#[test]
fn high_volume_virtualized_list_stability_pins_identity_reanchor_and_semantics() {
    for token in [
        "high-volume-virtualized-list-stability-stress",
        "highVolumeVirtualizedListStability",
        "runHighVolumeVirtualizedListStabilityStressScenario",
        "missing_high_volume_virtualized_list_stability_receipt",
        "list.highVolumeVirtualizedListStability",
        "virtualizedListStressId",
        "surface",
        "automationWindowId",
        "osWindowId",
        "semanticSurface",
        "datasetId",
        "fixtureItemCount",
        "totalItemCount",
        "visibleWindowSize",
        "virtualizationGenerationBefore",
        "virtualizationGenerationAfter",
        "rowSamples",
        "semanticId",
        "stableRowKey",
        "dataIndex",
        "visibleIndex",
        "rowBounds",
        "textBounds",
        "renderedTextBounds",
        "selectedSemanticIdBefore",
        "selectedSemanticIdAfter",
        "selectedStableKeyBefore",
        "selectedStableKeyAfter",
        "selectionReanchored",
        "scrollAnchorKey",
        "scrollTopBefore",
        "scrollTopAfter",
        "viewportBounds",
        "contentHeight",
        "filterCycles",
        "expectedCount",
        "actualCount",
        "firstVisibleKey",
        "selectedKey",
        "rowFingerprintBefore",
        "rowFingerprintAfter",
        "elementsFingerprint",
        "rapidFilterTransitions",
        "filterGeneration",
        "staleFilterResultsRejected",
        "screenshotReceipt",
        "screenshotStateRevalidated",
        "semanticVisibleTextMatchesRows",
        "duplicateRowKeysRejected",
        "rowReuseIdentityPreserved",
        "blankRowsRejected",
        "footerSafeSelectedRow",
        "cleanupConfirmed",
        "file_linear:high_volume_virtualized_list_stability_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token),
            "high-volume virtualized list stability stress must pin {token}"
        );
    }
}

#[test]
fn docs_and_skill_teach_loop_sixteen_boundaries() {
    for token in [
        "accessibility tree semantic parity",
        "role parity",
        "label parity",
        "keyboard activation semantics",
        "RTL/bidirectional/emoji text rendering",
        "grapheme clusters",
        "cursor visual positions",
        "high-volume virtualized list stability",
        "row identity",
        "selection reanchor",
        "screenshot-to-semantics consistency",
        "agentic_loop_sixteen_contract",
    ] {
        assert!(
            SKILL.contains(token) || AUTOMATION.contains(token) || VERIFICATION.contains(token),
            "loop-sixteen docs and skill must teach {token}"
        );
    }
}
