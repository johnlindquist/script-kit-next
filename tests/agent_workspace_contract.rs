//! Contract tests for the root agent workspace at ~/.scriptkit.
//!
//! Validates that `ensure_kit_setup()` seeds the correct root-level
//! agent workspace structure, and that `HarnessConfig::default()`
//! points its working directory at the Script Kit root.

use script_kit_gpui::setup::{ensure_kit_setup, get_kit_path, SK_PATH_ENV};
use std::fs;
use tempfile::TempDir;

/// Shared lock for SK_PATH env var mutation.
/// Integration tests run in the same process, so env var changes are global.
static SK_PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Run a test with SK_PATH set to a fresh temp directory.
fn with_temp_sk_path<F: FnOnce(&std::path::Path)>(f: F) {
    let _lock = SK_PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let temp_dir = TempDir::new().expect("create temp dir");
    let kit_root = temp_dir.path().join("scriptkit-test");
    std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());

    f(&kit_root);

    std::env::remove_var(SK_PATH_ENV);
}

/// Fresh setup seeds the full root-level agent workspace.
#[test]
fn test_setup_seeds_root_agent_workspace() {
    with_temp_sk_path(|kit_root| {
        let result = ensure_kit_setup();
        assert!(
            !result.warnings.iter().any(|w| w.contains("Failed")),
            "Setup should complete without failures: {:?}",
            result.warnings
        );

        // Root-level agent docs
        assert!(
            kit_root.join("CLAUDE.md").exists(),
            "Root CLAUDE.md must exist"
        );
        assert!(
            kit_root.join("AGENTS.md").exists(),
            "Root AGENTS.md must exist"
        );
        assert!(
            kit_root.join("GUIDE.md").exists(),
            "Root GUIDE.md must exist"
        );

        // Skills directory with all four skills
        for skill in &[
            "script-authoring",
            "scriptlets",
            "config",
            "troubleshooting",
        ] {
            let skill_path = kit_root.join("skills").join(skill).join("SKILL.md");
            assert!(
                skill_path.exists(),
                "skills/{skill}/SKILL.md must exist at {}",
                skill_path.display()
            );
        }
        assert!(
            kit_root.join("skills").join("README.md").exists(),
            "skills/README.md must exist"
        );

        // Example scripts
        for example in &[
            "hello-world.ts",
            "choose-from-list.ts",
            "clipboard-transform.ts",
            "path-picker.ts",
        ] {
            let example_path = kit_root.join("examples").join("scripts").join(example);
            assert!(
                example_path.exists(),
                "examples/scripts/{example} must exist at {}",
                example_path.display()
            );
        }
        assert!(
            kit_root.join("examples").join("README.md").exists(),
            "examples/README.md must exist"
        );

        // docs/ directory
        assert!(kit_root.join("docs").exists(), "docs/ directory must exist");

        // Kit subtree still exists
        assert!(
            kit_root.join("kit").join("config.ts").exists(),
            "kit/config.ts must exist"
        );
        assert!(
            kit_root.join("kit").join("theme.json").exists(),
            "kit/theme.json must exist"
        );
        assert!(
            kit_root.join("kit").join("package.json").exists(),
            "kit/package.json must exist"
        );
        assert!(
            kit_root.join("kit").join("tsconfig.json").exists(),
            "kit/tsconfig.json must exist"
        );
        assert!(
            kit_root
                .join("kit")
                .join("main")
                .join("scripts")
                .exists(),
            "kit/main/scripts/ must exist"
        );
        assert!(
            kit_root
                .join("kit")
                .join("main")
                .join("extensions")
                .exists(),
            "kit/main/extensions/ must exist"
        );
        assert!(
            kit_root
                .join("kit")
                .join("main")
                .join("agents")
                .exists(),
            "kit/main/agents/ must exist"
        );

        // kit/CLAUDE.md and kit/AGENTS.md are redirect stubs
        let kit_claude = fs::read_to_string(kit_root.join("kit").join("CLAUDE.md"))
            .expect("kit/CLAUDE.md should be readable");
        assert!(
            kit_claude.contains("../CLAUDE.md"),
            "kit/CLAUDE.md should redirect to root"
        );
        let kit_agents = fs::read_to_string(kit_root.join("kit").join("AGENTS.md"))
            .expect("kit/AGENTS.md should be readable");
        assert!(
            kit_agents.contains("../AGENTS.md"),
            "kit/AGENTS.md should redirect to root"
        );

        // SDK
        assert!(
            kit_root.join("sdk").join("kit-sdk.ts").exists(),
            "sdk/kit-sdk.ts must exist"
        );
    });
}

/// HarnessConfig::default() must resolve working_directory to the Script Kit root.
#[test]
fn test_default_harness_working_directory_is_scriptkit_root() {
    // The harness source must set working_directory from get_kit_path() in Default impl.
    let harness_source = include_str!("../src/ai/harness/mod.rs");

    // Verify the Default impl sets working_directory from get_kit_path
    assert!(
        harness_source.contains("crate::setup::get_kit_path()"),
        "HarnessConfig::default() must call crate::setup::get_kit_path() \
         to resolve working_directory"
    );
    assert!(
        harness_source.contains("Some(crate::setup::get_kit_path()"),
        "HarnessConfig::default() must set working_directory to Some(...)"
    );
}

/// Re-running setup must not overwrite user-authored workspace content.
#[test]
fn test_setup_idempotent_preserves_user_content() {
    with_temp_sk_path(|kit_root| {
        // First run — creates everything
        let _ = ensure_kit_setup();

        // Simulate user editing a file in kit/main/scripts
        let user_script = kit_root
            .join("kit")
            .join("main")
            .join("scripts")
            .join("my-custom.ts");
        fs::write(&user_script, "import \"@scriptkit/sdk\";\n// custom").unwrap();

        // Also write custom config
        let config_path = kit_root.join("kit").join("config.ts");
        let original_config = fs::read_to_string(&config_path).unwrap();
        let custom_config = format!("{original_config}\n// user customization");
        fs::write(&config_path, &custom_config).unwrap();

        // Second run
        let result = ensure_kit_setup();
        assert!(
            !result.warnings.iter().any(|w| w.contains("Failed")),
            "Rerun should not fail: {:?}",
            result.warnings
        );

        // User script preserved
        assert!(
            user_script.exists(),
            "User script should survive rerun"
        );
        let user_content = fs::read_to_string(&user_script).unwrap();
        assert!(
            user_content.contains("// custom"),
            "User script content should be preserved"
        );

        // User config preserved (write_string_if_missing won't overwrite)
        let reread_config = fs::read_to_string(&config_path).unwrap();
        assert!(
            reread_config.contains("// user customization"),
            "User config customization should be preserved"
        );
    });
}

/// No seeded skill or example references legacy v1 paths or packages.
#[test]
fn test_seeded_skills_do_not_reference_legacy_v1_contract() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();

        let legacy_patterns = [
            "@johnlindquist/kit",
            "~/.kenv",
            "~/.scriptkit/scripts",
            "// Name:",
            "require(",
        ];

        // Check all skill files
        for skill in &[
            "script-authoring",
            "scriptlets",
            "config",
            "troubleshooting",
        ] {
            let skill_path = kit_root.join("skills").join(skill).join("SKILL.md");
            let content = fs::read_to_string(&skill_path)
                .unwrap_or_else(|_| panic!("Should read {}", skill_path.display()));
            for legacy in &legacy_patterns {
                assert!(
                    !content.contains(legacy),
                    "skills/{skill}/SKILL.md must not contain legacy pattern '{legacy}'"
                );
            }
        }

        // Check all example scripts
        for example in &[
            "hello-world.ts",
            "choose-from-list.ts",
            "clipboard-transform.ts",
            "path-picker.ts",
        ] {
            let example_path = kit_root.join("examples").join("scripts").join(example);
            let content = fs::read_to_string(&example_path)
                .unwrap_or_else(|_| panic!("Should read {}", example_path.display()));
            for legacy in &legacy_patterns {
                assert!(
                    !content.contains(legacy),
                    "examples/scripts/{example} must not contain legacy pattern '{legacy}'"
                );
            }
        }

        // Check root CLAUDE.md and AGENTS.md
        let root_claude = fs::read_to_string(kit_root.join("CLAUDE.md")).unwrap();
        assert!(
            !root_claude.contains("@johnlindquist/kit"),
            "Root CLAUDE.md must not reference @johnlindquist/kit"
        );
        let root_agents = fs::read_to_string(kit_root.join("AGENTS.md")).unwrap();
        assert!(
            !root_agents.contains("@johnlindquist/kit"),
            "Root AGENTS.md must not reference @johnlindquist/kit"
        );
    });
}

/// kit://sdk-reference must use @scriptkit/sdk, new paths, and export const metadata.
#[test]
fn test_sdk_reference_uses_scriptkit_sdk_and_new_paths() {
    use script_kit_gpui::mcp_resources::{self, SdkReferenceDocument};

    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None)
        .expect("kit://sdk-reference should resolve");
    let doc: SdkReferenceDocument =
        serde_json::from_str(&content.text).expect("valid JSON document");

    // Must use @scriptkit/sdk — not @johnlindquist/kit
    assert_eq!(
        doc.sdk_package, "@scriptkit/sdk",
        "sdk_package must be @scriptkit/sdk"
    );

    // Script directory must point to the new kit/main/scripts path
    assert!(
        doc.script_directory.contains("kit/main/scripts"),
        "script_directory must use ~/.scriptkit/kit/main/scripts/, got: {}",
        doc.script_directory
    );

    // Metadata format must use export const metadata
    assert!(
        doc.metadata_format.contains("export const metadata"),
        "metadata_format must use export const metadata, got: {}",
        doc.metadata_format
    );

    // Example test script must import @scriptkit/sdk, not @johnlindquist/kit
    assert!(
        doc.harness_workflow
            .example_test_script
            .contains("@scriptkit/sdk"),
        "example_test_script must import @scriptkit/sdk"
    );
    assert!(
        !doc.harness_workflow
            .example_test_script
            .contains("@johnlindquist/kit"),
        "example_test_script must not reference @johnlindquist/kit"
    );

    // Example test script must not use legacy comment-header metadata
    assert!(
        !doc.harness_workflow
            .example_test_script
            .contains("// Name:"),
        "example_test_script must not use legacy // Name: comment header"
    );
}

/// The create-script action must open the canonical scripts_dir(), not a hardcoded legacy path.
#[test]
fn test_create_script_action_opens_current_scripts_dir() {
    let source = include_str!("../src/app_actions/handle_action/scripts.rs");

    // Must NOT contain the legacy hardcoded path
    assert!(
        !source.contains("~/.scriptkit/scripts"),
        "create_script action must not hardcode ~/.scriptkit/scripts"
    );

    // Must use the canonical scripts_dir() source of truth
    assert!(
        source.contains("crate::script_creation::scripts_dir()"),
        "create_script action must use crate::script_creation::scripts_dir()"
    );
}

/// Every seeded example script must use `@scriptkit/sdk` and `export const metadata`,
/// use Bun-first patterns (no Node-only APIs), and live under examples/scripts/.
#[test]
fn test_seeded_examples_are_sdk_based_and_bun_first() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();

        let examples = [
            "hello-world.ts",
            "choose-from-list.ts",
            "clipboard-transform.ts",
            "path-picker.ts",
        ];

        let examples_dir = kit_root.join("examples").join("scripts");

        for example in &examples {
            let path = examples_dir.join(example);
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Should read {}", path.display()));

            // Must import @scriptkit/sdk
            assert!(
                content.contains("import \"@scriptkit/sdk\""),
                "{example} must import @scriptkit/sdk"
            );

            // Must export metadata
            assert!(
                content.contains("export const metadata"),
                "{example} must use export const metadata"
            );

            // Must not use Node-only patterns
            let node_only_patterns = [
                "require(",
                "fs.readFileSync",
                "fs.writeFileSync",
                "process.argv",
                "__dirname",
                "__filename",
            ];
            for pattern in &node_only_patterns {
                assert!(
                    !content.contains(pattern),
                    "{example} must not use Node-only pattern '{pattern}'"
                );
            }
        }

        // Examples must NOT exist in kit/main/scripts/ (no pollution)
        let main_scripts = kit_root.join("kit").join("main").join("scripts");
        for example in &examples {
            // The example filenames should not appear as auto-installed entries
            // (hello-world.ts is a special case — the fresh-install sample script
            // uses that name, which is acceptable since it's user-facing)
            if *example != "hello-world.ts" {
                assert!(
                    !main_scripts.join(example).exists(),
                    "{example} must not be auto-installed into kit/main/scripts/"
                );
            }
        }

        // README must exist alongside the scripts
        assert!(
            kit_root.join("examples").join("README.md").exists(),
            "examples/README.md must exist"
        );
    });
}

/// Fresh setup must create the harness temp directories advertised by kit://sdk-reference.
#[test]
fn test_setup_seeds_harness_workflow_temp_dirs() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();

        let test_scripts = kit_root.join("tmp").join("test-scripts");
        let test_scriptlets = kit_root.join("tmp").join("test-scriptlets");

        assert!(
            test_scripts.exists(),
            "tmp/test-scripts must exist at {}",
            test_scripts.display()
        );
        assert!(
            test_scriptlets.exists(),
            "tmp/test-scriptlets must exist at {}",
            test_scriptlets.display()
        );
    });
}

/// kit://sdk-reference must publish a harness run_command that works from ~/.scriptkit,
/// not only from the repository root.
#[test]
fn test_sdk_reference_run_command_is_cwd_safe() {
    use script_kit_gpui::mcp_resources::{self, SdkReferenceDocument, SDK_REFERENCE_SCHEMA_VERSION};

    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None)
        .expect("kit://sdk-reference should resolve");
    let doc: SdkReferenceDocument =
        serde_json::from_str(&content.text).expect("valid JSON document");

    assert_eq!(
        doc.schema_version, SDK_REFERENCE_SCHEMA_VERSION,
        "schema_version must match SDK_REFERENCE_SCHEMA_VERSION"
    );

    let run_command = &doc.harness_workflow.run_command;

    assert!(
        !run_command.contains("./target/"),
        "run_command must not be repo-relative because harnesses cwd into ~/.scriptkit: {run_command}"
    );
    assert!(
        run_command.contains("\"type\":\"run\""),
        "run_command must send the stdin bridge run payload: {run_command}"
    );
    assert!(
        run_command.contains("{path}"),
        "run_command must preserve the {{path}} placeholder: {run_command}"
    );
}

/// Root agent docs are the canonical agent entrypoint and must stay free of legacy v1 tokens.
#[test]
fn test_root_agent_docs_do_not_reference_legacy_v1_contract() {
    with_temp_sk_path(|kit_root| {
        let _ = ensure_kit_setup();

        let legacy_patterns = [
            "@johnlindquist/kit",
            "~/.kenv",
            "~/.scriptkit/scripts",
            "// Name:",
            "require(",
        ];

        for doc in &["CLAUDE.md", "AGENTS.md"] {
            let path = kit_root.join(doc);
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Should read {}", path.display()));
            for legacy in &legacy_patterns {
                assert!(
                    !content.contains(legacy),
                    "{doc} must not contain legacy pattern '{legacy}'"
                );
            }
        }
    });
}
