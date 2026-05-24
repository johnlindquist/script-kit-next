const AGY_DEVTOOLS_SH: &str = include_str!("../scripts/agentic/agy-devtools.sh");
const AGY_SKILL: &str = include_str!("../.agents/skills/agy-script-kit-devtools/SKILL.md");

#[test]
fn agy_skill_has_frontmatter_and_routes_to_wrapper() {
    assert!(AGY_SKILL.contains("name: agy-script-kit-devtools"));
    assert!(AGY_SKILL.contains("scripts/agentic/agy-devtools.sh"));
    assert!(AGY_SKILL.contains("$script-kit-devtools"));
    assert!(AGY_SKILL.contains("inference.json"));
    assert!(AGY_SKILL.contains("compact.md"));
}

#[test]
fn wrapper_exposes_required_subcommands() {
    for subcommand in ["run", "infer", "prompt", "compact", "cleanup", "help"] {
        assert!(
            AGY_DEVTOOLS_SH.contains(&format!("{subcommand})")),
            "missing subcommand arm for {subcommand}"
        );
    }
}

#[test]
fn wrapper_defaults_are_safe_and_logs_are_persisted() {
    for needle in [
        "ALLOW_SUBMIT=0",
        "ALLOW_NATIVE=0",
        "ALLOW_MIC=0",
        "ALLOW_REAL_DATA=0",
        "TRUST_REPO=0",
        "--dangerously-skip-permissions",
        "--log-file",
        "--print-timeout",
        "--fast",
        "agy.stdout.md",
        "agy.stderr.log",
        "result.json",
        "receipts",
    ] {
        assert!(AGY_DEVTOOLS_SH.contains(needle), "missing {needle}");
    }
}

#[test]
fn wrapper_inference_knows_core_surfaces() {
    for needle in [
        "actions-dialog",
        "ActionsDialog",
        "ScriptList",
        "Notes",
        "Dictation",
        "PromptEntity",
        "blockedByDefault",
        "unavailablePrimitives",
        "Known minimal Agent Chat path",
        "Known minimal Theme Designer path",
        "builtin/choose-theme",
        "accent-color-hex",
    ] {
        assert!(AGY_DEVTOOLS_SH.contains(needle), "missing {needle}");
    }
}
