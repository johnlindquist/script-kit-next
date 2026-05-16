//! Source-level contract for thirty-seventh-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_seven_recipes() {
    for name in [
        "settings-preferences-readonly-detail-panel-stress",
        "design-picker-preview-restore-visual-stress",
        "dictation-history-transcript-preview-redaction-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn settings_readonly_detail_panel_pins_no_write_and_detail_bounds() {
    for token in [
        "settingsPreferencesReadonlyDetailPanelReceipt",
        "runSettingsPreferencesReadonlyDetailPanelStressScenario",
        "missing_settings_preferences_readonly_detail_panel_receipt",
        "ux.settingsPreferencesReadonlyDetailPanel",
        "settingsPreferencesReadonlyDetailPanelStressId",
        "fixtureCatalogId",
        "settingsSurfaceId",
        "selectedSectionBefore",
        "selectedSectionAfter",
        "detailPanelGeneration",
        "visibleRowLabels",
        "visibleTextBounds",
        "detailBodyBounds",
        "detailFooterBounds",
        "emptyStateCopy",
        "disabledApplySaveReason",
        "configFingerprintBefore",
        "configFingerprintAfter",
        "noSetupOrSecurityPrompt",
        "staleDetailGenerationRejected",
        "wrongSectionMutationRejected",
        "file_linear:settings_preferences_readonly_detail_panel_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn design_picker_preview_restore_pins_visual_tokens_and_no_design_write() {
    for token in [
        "designPickerPreviewRestoreVisualReceipt",
        "runDesignPickerPreviewRestoreVisualStressScenario",
        "missing_design_picker_preview_restore_visual_receipt",
        "ux.designPickerPreviewRestoreVisual",
        "designPickerPreviewRestoreVisualStressId",
        "fixtureDesignCatalogId",
        "activeDesignIdBeforePreview",
        "previewDesignId",
        "previewGeneration",
        "themeTokenFingerprintsBefore",
        "themeTokenFingerprintsPreview",
        "themeTokenFingerprintsRestored",
        "visiblePickerRowIds",
        "visiblePickerRowLabels",
        "selectedPreviewRowVisible",
        "screenshotSemanticTargetIdentity",
        "escapeRestoresPreviewState",
        "cmdWRestoresPreviewState",
        "persistedDesignFingerprintBefore",
        "persistedDesignFingerprintAfter",
        "stalePreviewGenerationRejected",
        "wrongSurfacePreviewRejected",
        "file_linear:design_picker_preview_restore_visual_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn dictation_history_preview_pins_redaction_and_no_media_capture() {
    for token in [
        "dictationHistoryTranscriptPreviewRedactionReceipt",
        "runDictationHistoryTranscriptPreviewRedactionStressScenario",
        "missing_dictation_history_transcript_preview_redaction_receipt",
        "ux.dictationHistoryTranscriptPreviewRedaction",
        "dictationHistoryTranscriptPreviewRedactionStressId",
        "fixtureDictationStoreId",
        "transcriptRowIds",
        "transcriptGeneration",
        "queryGeneration",
        "selectedTranscriptBefore",
        "selectedTranscriptAfter",
        "previewGeneration",
        "previewSourceId",
        "previewRenderKind",
        "visiblePreviewTextBounds",
        "redactedTranscriptFingerprint",
        "missingAudioFallbackCopy",
        "emojiGraphemeBounds",
        "footerInputNonOverlapping",
        "noRawTranscriptLeak",
        "noRawAudioPathLeak",
        "noMicrophonePermissionRequest",
        "noMediaCaptureRequest",
        "staleTranscriptGenerationRejected",
        "wrongRowPreviewRejected",
        "file_linear:dictation_history_transcript_preview_redaction_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_seven_boundaries() {
    for token in [
        "Settings read-only detail panel",
        "Design Picker preview restore",
        "Dictation History transcript preview redaction",
        "agentic_loop_thirty_seven_contract",
        "no-security-prompts",
        "no-system-settings",
        "no-tcc-mutation",
        "no-design-write",
        "no-microphone",
        "no-media-capture",
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
