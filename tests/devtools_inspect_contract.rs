use std::fs;

const INSPECT: &str = include_str!("../scripts/devtools/inspect.ts");
const DEVTOOLS_SKILL: &str = include_str!("../.agents/skills/script-kit-devtools/SKILL.md");
const DEVTOOLS_AUDIT: &str =
    include_str!("../.agents/skills/script-kit-devtools/references/devtools-coverage-audit.md");

#[test]
fn inspect_composes_existing_protocol_primitives() {
    for needle in [
        "listAutomationWindows",
        "inspectAutomationWindow",
        "getState",
        "getElements",
        "getLayoutInfo",
        "scripts/agentic/session.sh",
    ] {
        assert!(
            INSPECT.contains(needle),
            "devtools inspect must compose the existing protocol primitive {needle}"
        );
    }

    assert!(
        !INSPECT.contains("scripts/agentic/index.ts"),
        "devtools inspect must not route through the recipe catalog"
    );
}

#[test]
fn inspect_reports_capabilities_gaps_and_next_steps() {
    for needle in [
        "script-kit-devtools.inspect",
        "schemaVersion",
        "capabilities",
        "missingFields",
        "recommendedNext",
        "errors",
        "status: errors.length === 0 ? \"ok\" : \"degraded\"",
    ] {
        assert!(
            INSPECT.contains(needle),
            "devtools inspect report is missing contract field or fail-closed behavior: {needle}"
        );
    }

    for gap in [
        "target_state",
        "semantic_elements",
        "full_semantic_elements",
        "target_layout_info",
        "screenshot_metadata",
    ] {
        assert!(
            INSPECT.contains(gap),
            "devtools inspect must name missing coverage gap {gap}"
        );
    }
}

#[test]
fn inspect_supports_agent_target_selection() {
    for needle in [
        "--target-id",
        "--target-kind",
        "--target-index",
        "--target-title",
        "--focused",
        "--main",
        "--hi-dpi",
        "--start",
        "--show",
    ] {
        assert!(
            INSPECT.contains(needle),
            "devtools inspect must support target selector {needle}"
        );
    }
}

#[test]
fn devtools_skill_keeps_recipes_as_regression_wrappers() {
    for needle in [
        "Think Chrome DevTools for Script Kit, not a script catalog.",
        "Use recipes only when they match the bug directly or as regression proof",
        "Produce at least one direct primitive receipt",
        "devtools.inspect",
    ] {
        assert!(
            DEVTOOLS_SKILL.contains(needle),
            "script-kit-devtools skill must preserve the DevTools-first boundary: {needle}"
        );
    }

    assert!(
        DEVTOOLS_AUDIT.contains("Recipes should be rebuilt as thin smoke/regression wrappers"),
        "coverage audit must keep recipe usage bounded to smoke/regression wrappers"
    );
}

#[test]
fn devtools_docs_are_checked_in() {
    for path in [
        ".agents/skills/script-kit-devtools/SKILL.md",
        ".agents/skills/script-kit-devtools/references/devtools-coverage-audit.md",
        "scripts/devtools/inspect.ts",
    ] {
        assert!(
            fs::metadata(path).is_ok(),
            "expected checked-in DevTools artifact at {path}"
        );
    }
}
