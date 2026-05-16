//! Source-level contract for fourth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const TARGET_THREAD: &str = include_str!("../scripts/agentic/target-thread.ts");

#[test]
fn index_help_exposes_loop_four_recipes() {
    for name in [
        "drop-prompt-native-drop-privacy-stress",
        "path-prompt-filesystem-edge-stress",
        "screenshot-identity-acp-context-stress",
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
fn drop_prompt_privacy_stress_pins_redacted_native_drop_receipts() {
    for token in [
        "drop-prompt-native-drop-privacy-stress",
        "missing_drop_prompt_native_drop_receipt",
        "dropPrompt",
        "stateResult.drop.files[index,name,size]",
        "list:dropped-files",
        "kind:dropped_file",
        "forbiddenFields",
        "pathLeakDetected",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "DropPrompt privacy stress must pin {token}"
        );
    }
}

#[test]
fn path_prompt_stress_wraps_filesystem_edge_helper() {
    for token in [
        "path-prompt-filesystem-edge-stress",
        "scripts/agentic/path-prompt-fs-edges.ts",
        "pathPrompt",
        "missing",
        "empty",
        "file-start",
        "permission-denied",
        "path_prompt_filesystem_edge_failed",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "PathPrompt filesystem edge stress must pin {token}"
        );
    }
}

#[test]
fn screenshot_identity_stress_pins_context_threading_receipts() {
    for token in [
        "screenshot-identity-acp-context-stress",
        "missing_screenshot_identity_context_receipt",
        "screenshotIdentity",
        "stateResult.screenshotIdentity",
        "bare screenshot filename",
        "acpContextPart",
        "identityMatched",
        "filesystemGrepUsed: false",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "Screenshot identity stress must pin {token}"
        );
    }
}
