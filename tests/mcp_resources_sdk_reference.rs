//! Regression tests proving the published harness example in `kit://sdk-reference`
//! is verification-friendly: it contains the `SK_VERIFY` branch and, when executed
//! with `SK_VERIFY=1`, produces deterministic JSON stdout.

use script_kit_gpui::mcp_resources::{self, SdkReferenceDocument, SDK_REFERENCE_SCHEMA_VERSION};

/// Helper: read and deserialize the SDK reference document.
fn read_sdk_reference() -> SdkReferenceDocument {
    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None)
        .expect("kit://sdk-reference should resolve");
    serde_json::from_str(&content.text).expect("valid JSON document")
}

// -------------------------------------------------------
// Contract assertions (always run, no external deps)
// -------------------------------------------------------

#[test]
fn sdk_reference_harness_workflow_example_is_verification_friendly() {
    let doc = read_sdk_reference();
    let script = &doc.harness_workflow.example_test_script;

    // Must contain the SK_VERIFY environment check
    assert!(
        script.contains(r#"process.env.SK_VERIFY === "1""#),
        "example_test_script must contain the SK_VERIFY branch"
    );

    // Must contain the deterministic JSON stdout path
    assert!(
        script.contains("console.log(JSON.stringify({ ok: true, result }))"),
        "example_test_script must emit JSON verify output"
    );

    // Must contain the interactive fallback (arg prompt)
    assert!(
        script.contains("await arg("),
        "example_test_script must have an interactive fallback using arg()"
    );

    // The verify branch must short-circuit to a known value
    assert!(
        script.contains(r#"? "a""#),
        "verify branch must produce the deterministic value \"a\""
    );
}

#[test]
fn sdk_reference_harness_workflow_example_has_metadata_block() {
    let doc = read_sdk_reference();
    let script = &doc.harness_workflow.example_test_script;

    assert!(
        script.contains("export const metadata"),
        "example_test_script must export metadata"
    );
    assert!(
        script.contains(r#"name: "Harness Test""#),
        "example_test_script metadata must include a name"
    );
}

#[test]
fn sdk_reference_harness_workflow_example_imports_sdk() {
    let doc = read_sdk_reference();
    let script = &doc.harness_workflow.example_test_script;

    assert!(
        script.contains(r#"import "@scriptkit/sdk""#),
        "example_test_script must import @scriptkit/sdk"
    );
}

#[test]
fn sdk_reference_lists_ui_automation_functions() {
    let doc = read_sdk_reference();
    assert_eq!(doc.schema_version, SDK_REFERENCE_SCHEMA_VERSION);

    for (name, signature, category) in [
        (
            "getState",
            "await getState(): Promise<PromptState>",
            "automation",
        ),
        (
            "getElements",
            "await getElements(limit?: number): Promise<ElementsSnapshot>",
            "automation",
        ),
        (
            "waitFor",
            "await waitFor(condition: WaitCondition, options?: WaitForOptions): Promise<WaitForResult>",
            "automation",
        ),
        (
            "batch",
            "await batch(commands: BatchCommand[], options?: BatchOptions): Promise<BatchResult>",
            "automation",
        ),
    ] {
        let entry = doc
            .functions
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("missing sdk function entry: {name}"));
        assert_eq!(entry.signature, signature, "signature mismatch for {name}");
        assert_eq!(entry.category, category, "category mismatch for {name}");
    }
}

// -------------------------------------------------------
// Bun build + execution (requires bun on PATH)
// Gated behind `system-tests` feature flag.
// Run: cargo test --features system-tests
// -------------------------------------------------------

#[cfg(feature = "system-tests")]
mod bun_execution {
    use super::*;
    use std::process::Command;

    /// Verify bun is available before running execution tests.
    fn require_bun() -> String {
        let output = Command::new("bun")
            .arg("--version")
            .output()
            .expect("bun must be on PATH for system-tests");
        assert!(output.status.success(), "bun --version must succeed");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[test]
    fn harness_example_bun_build_succeeds() {
        let _version = require_bun();
        let doc = read_sdk_reference();
        let script = &doc.harness_workflow.example_test_script;

        let dir = tempfile::tempdir().expect("create temp dir");
        let script_path = dir.path().join("harness-test.ts");
        let out_path = dir.path().join("harness-test.verify.mjs");

        std::fs::write(&script_path, script).expect("write temp script");

        let output = Command::new("bun")
            .args([
                "build",
                script_path.to_str().unwrap(),
                "--target=bun",
                "--outfile",
                out_path.to_str().unwrap(),
            ])
            .output()
            .expect("bun build should run");

        assert!(
            output.status.success(),
            "bun build failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert!(out_path.exists(), "build output file must exist");
    }

    #[test]
    fn harness_example_sk_verify_produces_expected_json() {
        let _version = require_bun();
        let doc = read_sdk_reference();
        let script = &doc.harness_workflow.example_test_script;

        let dir = tempfile::tempdir().expect("create temp dir");
        let script_path = dir.path().join("harness-test.ts");
        std::fs::write(&script_path, script).expect("write temp script");

        let output = Command::new("bun")
            .arg(script_path.to_str().unwrap())
            .env("SK_VERIFY", "1")
            .output()
            .expect("bun should run the script");

        assert!(
            output.status.success(),
            "SK_VERIFY=1 bun failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(
            stdout, r#"{"ok":true,"result":"a"}"#,
            "SK_VERIFY=1 must produce exactly {{\"ok\":true,\"result\":\"a\"}}"
        );
    }
}
