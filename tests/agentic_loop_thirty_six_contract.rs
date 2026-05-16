//! Source-level contract for thirty-sixth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const SKILL: &str = include_str!("../.agents/skills/agentic-testing/SKILL.md");
const AUTOMATION: &str = include_str!("../lat.md/automation.md");
const VERIFICATION: &str = include_str!("../lat.md/verification.md");

#[test]
fn index_help_exposes_loop_thirty_six_recipes() {
    for name in [
        "app-launcher-icon-grid-keyboard-navigation-stress",
        "browser-history-time-grouped-privacy-stress",
        "settings-preferences-search-reset-preview-stress",
    ] {
        assert!(INDEX.contains(&format!("name: \"{name}\"")));
        assert!(INDEX.contains(&format!("case \"{name}\"")));
        assert!(INDEX.contains(&format!("bun scripts/agentic/index.ts {name}")));
    }
}

#[test]
fn app_launcher_icon_grid_pins_keyboard_layout_and_no_launch() {
    for token in [
        "appLauncherIconGridKeyboardNavigationReceipt",
        "runAppLauncherIconGridKeyboardNavigationStressScenario",
        "missing_app_launcher_icon_grid_keyboard_navigation_receipt",
        "ux.appLauncherIconGridKeyboardNavigation",
        "appLauncherIconGridKeyboardNavigationStressId",
        "fixtureAppCatalogId",
        "iconGridGeneration",
        "visibleAppIds",
        "visibleIconBounds",
        "iconImageFingerprints",
        "selectedAppIdBefore",
        "selectedAppIdAfter",
        "selectedCellBounds",
        "selectedCellVisible",
        "keyboardNeighborMap",
        "rowColumnCount",
        "filterGeneration",
        "emptyStateBounds",
        "previewPanelBounds",
        "tooltipForTruncatedName",
        "noIconTextOverlap",
        "noPreviewFooterCollision",
        "enterLaunchRefused",
        "noNativeAppLaunch",
        "staleCatalogGenerationRejected",
        "wrongAppActivationRejected",
        "file_linear:app_launcher_icon_grid_keyboard_navigation_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn browser_history_time_grouped_pins_privacy_and_no_activation() {
    for token in [
        "browserHistoryTimeGroupedPrivacyReceipt",
        "runBrowserHistoryTimeGroupedPrivacyStressScenario",
        "missing_browser_history_time_grouped_privacy_receipt",
        "ux.browserHistoryTimeGroupedPrivacy",
        "browserHistoryTimeGroupedPrivacyStressId",
        "fixtureHistoryProviderId",
        "historyGeneration",
        "timeBucketIds",
        "stickyTimeHeaderBounds",
        "visibleVisitIds",
        "visitRowFingerprints",
        "faviconFallbackIds",
        "redactedUrlFingerprints",
        "selectedVisitBefore",
        "selectedVisitAfter",
        "selectedVisitVisible",
        "duplicateVisitCollapsed",
        "noRawPrivateUrlLeak",
        "noFaviconNetworkRequest",
        "openInBrowserRefused",
        "noBrowserActivationReceipt",
        "staleHistoryGenerationRejected",
        "wrongVisitActivationRejected",
        "file_linear:browser_history_time_grouped_privacy_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn settings_preferences_pins_search_reset_preview_and_no_write() {
    for token in [
        "settingsPreferencesSearchResetPreviewReceipt",
        "runSettingsPreferencesSearchResetPreviewStressScenario",
        "missing_settings_preferences_search_reset_preview_receipt",
        "ux.settingsPreferencesSearchResetPreview",
        "settingsPreferencesSearchResetPreviewStressId",
        "sandboxConfigId",
        "preferenceSectionIds",
        "visiblePreferenceIds",
        "controlBounds",
        "controlAccessibleNames",
        "valueBeforeByPreference",
        "previewValueByPreference",
        "dirtyPreferenceIds",
        "searchHighlightRanges",
        "resetPreviewReceipt",
        "cancelResetRestoresValues",
        "disabledControlRefusal",
        "noConfigFileWrite",
        "noSecretValueLeak",
        "stalePreferenceGenerationRejected",
        "wrongPreferenceMutationRejected",
        "file_linear:settings_preferences_search_reset_preview_receipts_missing",
    ] {
        assert!(INDEX.contains(token) || SCENARIO.contains(token) || AUTOMATION.contains(token));
    }
}

#[test]
fn docs_and_skill_teach_loop_thirty_six_boundaries() {
    for token in [
        "App Launcher icon-grid keyboard navigation",
        "Browser History time-grouped privacy",
        "Settings preferences search/reset preview",
        "agentic_loop_thirty_six_contract",
        "no-native-input",
        "no-native-pointer",
        "no-app-launch",
        "no-browser-activation",
        "no-config-write",
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
