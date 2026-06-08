use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::Value;

const ITERATIONS_JSON: &str = include_str!("../docs/ai/profile-builder-iterations.json");
const PLAN_DOC: &str = include_str!("../docs/ai/profile-builder-plan.md");
const RUNTIME_RECEIPTS: &str = include_str!("../docs/ai/profile-builder-runtime-receipts.md");
const DEVTOOLS_ACT_SOURCE: &str = include_str!("../scripts/devtools/act.ts");

#[test]
fn profile_builder_iteration_ledger_has_ten_unique_iterations() {
    let json: Value = serde_json::from_str(ITERATIONS_JSON).expect("iteration ledger parses");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(
        json["runtimeContract"]["selectorSurface"],
        "main-menu-search"
    );
    assert_eq!(json["runtimeContract"]["selectorTrigger"], "|");

    let forbidden = json["runtimeContract"]["forbiddenSelectorSurfaces"]
        .as_array()
        .expect("forbidden selector surfaces");
    assert!(forbidden
        .iter()
        .any(|value| value == "agent_chat-profile-popup"));
    assert!(forbidden.iter().any(|value| value == "actions-dialog"));

    let iterations = json["iterations"].as_array().expect("iterations array");
    assert_eq!(iterations.len(), 10);

    let mut ids = HashSet::new();
    for iteration in iterations {
        let id = iteration["id"].as_str().expect("iteration id");
        assert!(ids.insert(id), "duplicate iteration id {id}");
        assert!(
            matches!(
                iteration["status"].as_str(),
                Some("seeded-example") | Some("planned") | Some("validated")
            ),
            "{id} must declare a known status"
        );
        for field in [
            "purpose",
            "artifactPath",
            "oraclePrompt",
            "allowedPrompt",
            "blockedPrompt",
        ] {
            assert!(
                iteration[field]
                    .as_str()
                    .is_some_and(|value| !value.trim().is_empty()),
                "{id} missing {field}"
            );
        }
        assert!(
            iteration["expectedTools"].is_array(),
            "{id} expectedTools must be an array"
        );
    }
}

#[test]
fn seeded_iterations_match_shipped_example_profiles() {
    let json: Value = serde_json::from_str(ITERATIONS_JSON).expect("iteration ledger parses");
    let seeded = json["iterations"]
        .as_array()
        .expect("iterations array")
        .iter()
        .filter(|iteration| iteration["status"] == "seeded-example")
        .map(|iteration| {
            (
                iteration["id"].as_str().expect("seeded id"),
                iteration["artifactPath"].as_str().expect("artifact path"),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        seeded,
        vec![
            ("profile-builder", "kit-init/profiles/profile-builder"),
            ("codebase-scout", "kit-init/profiles/codebase-scout"),
            (
                "plugin-sandbox-builder",
                "kit-init/profiles/plugin-sandbox-builder"
            ),
            ("text-polisher", "kit-init/profiles/text-polisher"),
            ("docs-researcher", "kit-init/profiles/docs-researcher"),
            (
                "project-docs-maintainer",
                "kit-init/profiles/project-docs-maintainer"
            ),
            (
                "package-manager-plan-only",
                "kit-init/profiles/package-manager-plan-only"
            ),
            (
                "legacy-agent-import",
                "kit-init/profiles/legacy-agent-import"
            ),
            (
                "invalid-schema-collision",
                "kit-init/profiles/invalid-schema-collision"
            ),
            (
                "ambient-leakage-stress",
                "kit-init/profiles/ambient-leakage-stress"
            ),
        ]
    );
}

#[test]
fn seeded_iterations_have_complete_matching_profile_artifacts() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json: Value = serde_json::from_str(ITERATIONS_JSON).expect("iteration ledger parses");
    for iteration in json["iterations"]
        .as_array()
        .expect("iterations array")
        .iter()
        .filter(|iteration| iteration["status"] == "seeded-example")
    {
        let id = iteration["id"].as_str().expect("seeded id");
        let artifact_path = iteration["artifactPath"].as_str().expect("artifact path");
        let root = manifest_dir.join(artifact_path);
        assert!(root.is_dir(), "{id} artifact root must exist");

        for relative in [
            "profile.json",
            "PROMPT.md",
            "README.md",
            "examples/smoke.json",
        ] {
            assert!(
                root.join(relative).is_file(),
                "{id} missing artifact file {relative}"
            );
        }

        let profile_json: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("profile.json")).expect("read profile.json"),
        )
        .expect("parse profile.json");
        assert_eq!(profile_json["id"], id);
        assert_eq!(profile_json["schemaVersion"], 1);

        let expected_tools = iteration["expectedTools"]
            .as_array()
            .expect("expectedTools array")
            .iter()
            .map(|value| value.as_str().expect("tool string"))
            .collect::<Vec<_>>();
        let artifact_tools = profile_json["toolPolicy"]["allow"]
            .as_array()
            .expect("toolPolicy.allow array")
            .iter()
            .map(|value| value.as_str().expect("tool string"))
            .collect::<Vec<_>>();
        assert_eq!(artifact_tools, expected_tools, "{id} tool list drifted");

        let readme = std::fs::read_to_string(root.join("README.md")).expect("read readme");
        assert!(
            readme.contains("main Menu Search") && readme.contains("`|`"),
            "{id} README must explain main Menu Search profile selection"
        );

        serde_json::from_str::<Value>(
            &std::fs::read_to_string(root.join("examples/smoke.json")).expect("read smoke"),
        )
        .expect("smoke example JSON parses");
    }
}

#[test]
fn profile_builder_iteration_points_to_runtime_receipt_doc() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json: Value = serde_json::from_str(ITERATIONS_JSON).expect("iteration ledger parses");
    let profile_builder = json["iterations"]
        .as_array()
        .expect("iterations array")
        .iter()
        .find(|iteration| iteration["id"] == "profile-builder")
        .expect("profile-builder iteration");
    let receipt_path = profile_builder["receiptPath"]
        .as_str()
        .expect("profile-builder receiptPath");

    assert!(manifest_dir.join(receipt_path).is_file());
    for required in [
        "profile-builder-ledger-proof",
        "profile-builder-prompt-transcript-profile-builder",
        "classification: ok",
        "Profile Builder ✓",
        "Current Agent Chat profile · Plugin · Pi",
        "profile-switch",
        "agent-chat-route",
        "postIntentTargetProof.classification: ok",
        "Allowed validation only",
        "Blocked validation only",
        "agent_chat_export_markdown",
        "read-only profile for `~/dev/demo`",
        "this profile can only create profile artifacts under plugins/main/profiles",
        "ActionsDialog",
    ] {
        assert!(
            RUNTIME_RECEIPTS.contains(required),
            "missing receipt text {required}"
        );
    }
}

#[test]
fn iteration_plan_uses_main_menu_search_devtools_not_actions_popup() {
    assert!(PLAN_DOC.contains("main Menu Search"));
    assert!(PLAN_DOC.contains("set-input --session profile-builder-main-menu --main"));
    assert!(PLAN_DOC.contains("--value '|'"));
    assert!(PLAN_DOC.contains("--submit-intent profile-switch"));

    for forbidden in ["open-actions", "actionsDialog"] {
        assert!(
            !PLAN_DOC.contains(forbidden),
            "profile validation plan must not route profile selection through {forbidden}"
        );
        assert!(
            !ITERATIONS_JSON.contains(forbidden),
            "iteration ledger must not route profile selection through {forbidden}"
        );
    }
}

#[test]
fn devtools_allows_scoped_main_menu_profile_switch_submit() {
    assert!(DEVTOOLS_ACT_SOURCE.contains("submitIntent !== \"profile-switch\""));
    assert!(DEVTOOLS_ACT_SOURCE.contains("selected?.kind === \"profile\""));
    assert!(DEVTOOLS_ACT_SOURCE.contains("selected?.kind === \"hint\""));
    assert!(DEVTOOLS_ACT_SOURCE.contains("choice:0:ready-to-send"));
    assert!(DEVTOOLS_ACT_SOURCE.contains("\\|plugin:[a-z0-9-]+"));
    assert!(DEVTOOLS_ACT_SOURCE.contains("selected?.sourceName === \"Spine\""));
    assert!(DEVTOOLS_ACT_SOURCE.contains("allowedBy: \"submitIntent:profile-switch\""));
}
