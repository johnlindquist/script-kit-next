//! Source-level contract for second-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");

#[test]
fn index_help_exposes_loop_two_recipes() {
    for name in [
        "file-portal-origin-roundtrip",
        "permission-privacy-preflight",
        "shortcut-recorder-focus-capture",
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
fn portal_round_trip_fails_closed_until_origin_receipts_exist() {
    for token in [
        "file-portal-origin-roundtrip",
        "missing_portal_round_trip_origin_receipt",
        "portalSessionId",
        "returnTarget",
        "contextPart.uri",
        "mutatedUserData: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "portal stress receipt must pin {token}"
        );
    }
}

#[test]
fn permission_preflight_is_read_only_and_non_prompting() {
    for token in [
        "permission-privacy-preflight",
        "macos-input.ts\", \"check",
        "window.ts\", \"status",
        "openedSystemSettings: false",
        "mutatedTcc: false",
        "clickedSettings: false",
        "notPrompted",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "permission preflight must pin read-only token {token}"
        );
    }
}

#[test]
fn shortcut_recorder_stress_fails_closed_without_config_writes() {
    for token in [
        "shortcut-recorder-focus-capture",
        "missing_shortcut_recorder_capture_receipt",
        "recorderSurface",
        "focusedAutomationWindowId",
        "capturedShortcut",
        "leakedGlobalHotkey",
        "mutatedUserData: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "shortcut recorder stress must pin {token}"
        );
    }
}
