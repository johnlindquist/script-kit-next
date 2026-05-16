//! Source-level contract for seventh-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");

#[test]
fn index_help_exposes_loop_seven_recipes() {
    for name in [
        "settings-theme-hot-reload-stress",
        "file-search-drag-out-identity-stress",
        "scriptlet-bundle-execution-matrix-stress",
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
}

#[test]
fn settings_theme_hot_reload_stress_pins_identity_repaint_and_cleanup_receipts() {
    for token in [
        "settings-theme-hot-reload-stress",
        "missing_settings_theme_hot_reload_receipt",
        "settingsThemeHotReload",
        "configSourceIdentity",
        "configPathFingerprint",
        "beforeThemeTokenFingerprint",
        "afterThemeTokenFingerprint",
        "beforeRendererCacheRevision",
        "afterRendererCacheRevision",
        "staleRendererCacheRejected",
        "activeWindowRepaint",
        "beforePaintRevision",
        "afterPaintRevision",
        "repaintObserved",
        "restoredConfigFingerprint",
        "manualSettingsClicks: false",
        "mutatedUserConfig: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Settings/theme hot-reload stress must pin {token}"
        );
    }
}

#[test]
fn file_search_drag_out_stress_pins_payload_privacy_refusal_and_return_receipts() {
    for token in [
        "file-search-drag-out-identity-stress",
        "missing_file_search_drag_out_identity_receipt",
        "fileSearchDragOut",
        "selectedFileUri",
        "selectedRowFingerprint",
        "visibleRowsRedacted",
        "privatePathLeakDetected",
        "forbiddenVisibleFields",
        "dragPreview",
        "previewFileUri",
        "dragPayloadIdentity",
        "payloadMatchesSelectedFile",
        "hostDropRefusal",
        "wrongHostAccepted",
        "returnSurface",
        "returnedToSourceSurface",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "File Search drag-out identity stress must pin {token}"
        );
    }
}

#[test]
fn scriptlet_bundle_execution_matrix_stress_pins_hash_isolation_output_and_cancel_receipts() {
    for token in [
        "scriptlet-bundle-execution-matrix-stress",
        "missing_scriptlet_bundle_execution_receipt",
        "scriptletBundleExecution",
        "bundleSourceHash",
        "matrixCases",
        "selectedScriptletId",
        "selectedBundleId",
        "argsFingerprint",
        "envFingerprint",
        "argEnvIsolation",
        "executionId",
        "executionOutput",
        "cancellationPath",
        "cancelledBeforeOutputCommit",
        "orphanProcessDetected",
        "crossScriptletStateBleed",
        "mutatedUserKenv: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Scriptlet bundle execution matrix stress must pin {token}"
        );
    }
}
