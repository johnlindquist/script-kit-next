use std::fs;

const INSPECT: &str = include_str!("../scripts/devtools/inspect.ts");
const TARGETS: &str = include_str!("../scripts/devtools/targets.ts");
const SURFACE: &str = include_str!("../scripts/devtools/surface.ts");
const AUTOMATION_INSPECT: &str = include_str!("../src/protocol/types/automation_inspect.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const APP_VIEW_STATE: &str = include_str!("../src/main_sections/app_view_state.rs");
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
fn automation_inspect_snapshot_exposes_runtime_surface_identity() {
    for needle in [
        "pub surface_kind: Option<String>",
        "pub app_view_variant: Option<String>",
        "pub native_footer_surface: Option<String>",
    ] {
        assert!(
            AUTOMATION_INSPECT.contains(needle),
            "AutomationInspectSnapshot must expose runtime surface identity field: {needle}"
        );
    }

    for needle in [
        "fn app_view_variant(&self) -> &'static str",
        "AppView::ScriptList => \"ScriptList\"",
        "AppView::DivPrompt { .. } => \"DivPrompt\"",
        "AppView::AcpChatView { .. } => \"AcpChatView\"",
    ] {
        assert!(
            APP_VIEW_STATE.contains(needle),
            "AppView must expose stable variant identity for DevTools: {needle}"
        );
    }

    for needle in [
        "surface_kind: (resolved.kind == protocol::AutomationWindowKind::Main)",
        "app_view_variant: (resolved.kind == protocol::AutomationWindowKind::Main)",
        "native_footer_surface: (resolved.kind == protocol::AutomationWindowKind::Main)",
    ] {
        assert!(
            PROMPT_HANDLER.contains(needle),
            "inspectAutomationWindow must populate runtime surface identity: {needle}"
        );
    }
}

#[test]
fn targets_cli_promotes_strict_target_identity_to_first_class_receipt() {
    for needle in [
        "script-kit-devtools.targets",
        "targets.list",
        "targets.inspect",
        "listAutomationWindows",
        "inspectAutomationWindow",
        "--target-id",
        "--target-kind",
        "--target-index",
        "--target-title",
        "--focused",
        "--main",
        "--surface",
        "--strict",
        "requestedTarget",
        "resolvedTarget",
        "stableTargetId",
        "strictTargetMatch",
        "blocked-by-target-ambiguity",
        "blocked-by-timeout",
        "screenshotIdentity",
    ] {
        assert!(
            TARGETS.contains(needle),
            "targets CLI must expose strict target identity field: {needle}"
        );
    }

    assert!(
        DEVTOOLS_SKILL.contains("Targets CLI"),
        "DevTools skill should route strict target identity through targets CLI"
    );
}

#[test]
fn surface_cli_combines_strict_target_identity_with_surface_contract() {
    for needle in [
        "script-kit-devtools.surface",
        "surface.inspect",
        "scripts/devtools/targets.ts",
        "docs/ai/contracts/surface-contracts.json",
        "--surface",
        "requestedSurfaceKind",
        "targetReceipt",
        "dismissPolicy",
        "getState",
        "stateResult",
        "activeFooterSurface",
        "surfaceContract",
        "activeFooter",
        "focusedSemanticId",
        "selectedSemanticId",
        "rowCountVisible",
        "visibleChoiceCount",
        "choiceCount",
        "blocked-by-unknown-surface",
        "blocked-by-missing-primitive",
        "strictTargetIdentity",
        "surfaceContract",
    ] {
        assert!(
            SURFACE.contains(needle),
            "surface CLI must expose target plus contract field: {needle}"
        );
    }

    assert!(
        DEVTOOLS_SKILL.contains("Surface CLI"),
        "DevTools skill should route surface inspection through surface CLI"
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
        "scripts/devtools/targets.ts",
        "scripts/devtools/surface.ts",
    ] {
        assert!(
            fs::metadata(path).is_ok(),
            "expected checked-in DevTools artifact at {path}"
        );
    }
}
