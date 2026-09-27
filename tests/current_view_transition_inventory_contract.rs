//! Source-level contract for the generated current-view transition inventory.
//!
//! The inventory is not a replacement for named transition owners. It is the
//! agent-readable map of remaining direct mutation sites while those owners are
//! introduced in smaller behavior slices.

use std::process::Command;

const GENERATOR: &str = include_str!("../scripts/generate-current-view-transitions.ts");
const INVENTORY_JSON: &str = include_str!("../docs/ai/contracts/current-view-transitions.json");

fn inventory_entries() -> Vec<serde_json::Value> {
    let parsed: serde_json::Value = serde_json::from_str(INVENTORY_JSON)
        .expect("current-view transition inventory must be valid JSON");
    assert_eq!(parsed["schemaVersion"], 1);
    assert_eq!(
        parsed["inventory"],
        "ScriptListApp/AppView current_view transition sites"
    );
    parsed["entries"]
        .as_array()
        .expect("inventory entries must be an array")
        .clone()
}

#[test]
fn generated_current_view_transition_inventory_is_not_stale() {
    let output = Command::new("bun")
        .arg("scripts/generate-current-view-transitions.ts")
        .arg("--check")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("bun must run current-view transition generator");
    assert!(
        output.status.success(),
        "current-view transition inventory is stale:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn inventory_entries_expose_agent_needed_transition_fields() {
    let entries = inventory_entries();
    assert!(
        entries.len() >= 20,
        "inventory should expose the broad remaining current_view transition surface"
    );
    for entry in entries {
        for key in [
            "file",
            "owner",
            "receiver",
            "operation",
            "expression",
            "inferredTarget",
        ] {
            assert!(
                entry[key].as_str().is_some(),
                "inventory entry must expose `{key}` as a string: {entry:?}"
            );
        }
        assert!(
            entry["line"].as_u64().is_some(),
            "inventory entry must expose line as an integer: {entry:?}"
        );
        assert!(
            entry["requiresManualReview"].as_bool().is_some(),
            "inventory entry must expose requiresManualReview as a boolean: {entry:?}"
        );
        if entry["operation"] == "transitionHelper" {
            assert!(
                entry["helper"].as_str().is_some(),
                "transitionHelper entries must expose the helper name: {entry:?}"
            );
            assert!(
                entry["transitionContract"].as_object().is_some(),
                "transitionHelper entries must expose transitionContract metadata: {entry:?}"
            );
        }
    }
}

#[test]
fn generator_scans_source_instead_of_hardcoding_transition_entries() {
    for expected in [
        "SOURCE_DIRS",
        "src/app_actions",
        "src/app_execute",
        "src/app_impl",
        "src/prompt_handler",
        "src/main_sections",
        "src/main_entry",
        "assignmentRegex",
        "std::mem::replace",
        "TRANSITION_HELPERS",
        "enter_embedded_agent_chat_surface",
        "restore_current_view_with_focus",
        "show_script_list_with_main_filter_focus",
        "--check",
        "--write",
    ] {
        assert!(
            GENERATOR.contains(expected),
            "generator must include source-backed inventory marker `{expected}`"
        );
    }
}
