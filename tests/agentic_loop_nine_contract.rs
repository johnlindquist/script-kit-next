//! Source-level contract for ninth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");

#[test]
fn index_help_exposes_loop_nine_recipes() {
    for name in [
        "clipboard-share-trust-install-stress",
        "clipboard-share-watcher-stale-replay-stress",
        "permission-share-cross-prompt-focus-stress",
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
        "runClipboardShareTrustInstallStressScenario",
        "runClipboardShareWatcherStaleReplayStressScenario",
        "runPermissionShareCrossPromptFocusStressScenario",
    ] {
        assert!(
            INDEX.contains(function_name) || SCENARIO.contains(function_name),
            "loop-nine function {function_name} must be wired"
        );
    }
}

#[test]
fn clipboard_share_trust_install_stress_pins_prompt_package_install_and_restore_receipts() {
    for token in [
        "clipboard-share-trust-install-stress",
        "missing_clipboard_share_trust_install_receipt",
        "clipboardShareTrust",
        "scriptkit-share://v1",
        "decodedPackageFingerprint",
        "promptKind: \"shareTrust\"",
        "parentWindowId",
        "promptWindowId",
        "shownBeforeInstall",
        "noInstallBeforeTrust",
        "installAttemptBeforeAccept: false",
        "explicitAcceptRequired: true",
        "explicitRefuseRequired: true",
        "refusePath",
        "acceptPath",
        "clipboardRestored",
        "mutatedClipboard: false",
        "mutatedUserPlugins: false",
        "file_linear:clipboard_share_trust_install_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Clipboard share trust install stress must pin {token}"
        );
    }
}

#[test]
fn clipboard_watcher_stale_replay_stress_pins_generation_replacement_replay_and_cleanup_receipts() {
    for token in [
        "clipboard-share-watcher-stale-replay-stress",
        "missing_clipboard_share_watcher_replay_receipt",
        "clipboardShareReplay",
        "requestedUriCount",
        "burstMs",
        "burstChangeCounts",
        "observedGenerations",
        "latestGeneration",
        "generationOrderingStrict",
        "staleUriRejected",
        "staleRejectionReceipts",
        "promptLifecycle",
        "replacedPromptGenerations",
        "cancelledPromptGenerations",
        "promptReplacementReceipt",
        "promptCancelReceipt",
        "installDedupeKey",
        "duplicateInstallRejected",
        "noDuplicateInstalls",
        "mutatedClipboard: false",
        "mutatedUserPlugins: false",
        "file_linear:clipboard_share_watcher_replay_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Clipboard watcher stale/replay stress must pin {token}"
        );
    }
}

#[test]
fn permission_share_cross_prompt_focus_stress_pins_priority_focus_no_activation_and_cleanup_receipts(
) {
    for token in [
        "permission-share-cross-prompt-focus-stress",
        "missing_permission_share_cross_prompt_focus_receipt",
        "permissionShareCrossPrompt",
        "PassiveOverlayPanel",
        "shareTrustPrompt",
        "promptPriority",
        "targetWindowIdentity",
        "sharePromptDidNotStealPermissionDrag",
        "permissionPanelDidNotAcceptShare",
        "systemSettingsActivated: false",
        "settingsActivationLeak: false",
        "accidentalShareAccepted: false",
        "openedSystemSettings: false",
        "clickedSettings: false",
        "performedDrag: false",
        "mutatedTcc: false",
        "wroteTccDb: false",
        "mutatedUserPlugins: false",
        "activationPolicyRestored",
        "file_linear:permission_share_cross_prompt_focus_receipts_missing",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Permission/share cross-prompt focus stress must pin {token}"
        );
    }
}
