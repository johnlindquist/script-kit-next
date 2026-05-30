use std::fs;

const SURFACES: &str = include_str!("../scripts/devtools/surfaces.ts");
const INVESTIGATE: &str = include_str!("../scripts/devtools/investigate.ts");
const SCHEMA: &str = include_str!("../scripts/devtools/schema.ts");
const DEVTOOLS_SKILL: &str = include_str!("../.agents/skills/script-kit-devtools/SKILL.md");
const SURFACE_INVENTORY: &str =
    include_str!("../.agents/skills/script-kit-devtools/references/devtools-surface-inventory.md");
const ORACLE_BUILDOUT: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-oracle-buildout-plan.md"
);
const SURFACE_CONTRACTS: &str = include_str!("../docs/ai/contracts/surface-contracts.json");
const FEATURE_MAP: &str = include_str!("../feature-map/index.md");

#[test]
fn surfaces_cli_is_checked_in_and_source_backed() {
    for path in [
        "scripts/devtools/surfaces.ts",
        "scripts/devtools/investigate.ts",
        "scripts/devtools/schema.ts",
        "docs/ai/contracts/surface-contracts.json",
        "feature-map/index.md",
        ".agents/skills/script-kit-devtools/references/devtools-surface-inventory.md",
        ".agents/skills/script-kit-devtools/references/devtools-oracle-buildout-plan.md",
    ] {
        assert!(
            fs::metadata(path).is_ok(),
            "expected DevTools surface inventory artifact at {path}"
        );
    }

    for needle in [
        "script-kit-devtools.surfaces",
        "docs/ai/contracts/surface-contracts.json",
        "feature-map/index.md",
        "scripts/devtools/coverage.ts",
        "sourceArtifacts",
        "surfaceContracts",
        "featureMap",
        "existingDevToolsCoverage",
        "coverageSurfaceAliases",
        "countsAsCoverage: false",
        "dismissPolicy",
        "uncoveredContractSurfaceKinds",
        "recommendedOracleBatches",
    ] {
        assert!(
            SURFACES.contains(needle),
            "surface inventory CLI must expose source-backed field: {needle}"
        );
    }
}

#[test]
fn schema_cli_pins_shared_receipt_envelope_and_acceptance_bar() {
    for needle in [
        "script-kit-devtools.schema",
        "receiptEnvelopeFields",
        "invocationId",
        "sessionId",
        "target",
        "classification",
        "blocked-by-stale-generation",
        "blocked-by-native-escalation-required",
        "targetIdentityFields",
        "resolvedTarget.strictTargetMatch",
        "devtools.surface.inspect",
        "contract.dismissPolicy",
        "devtools.layout.measure",
        "resizePressure.pressureScore",
        "devtools.act",
        "targetBefore",
        "targetAfter",
        "devtools.compare.redgreen",
        "samePrimitiveStack",
        "devtools.investigate",
        "acceptanceBar",
    ] {
        assert!(
            SCHEMA.contains(needle),
            "schema CLI must pin shared receipt field: {needle}"
        );
    }

    assert!(
        DEVTOOLS_SKILL.contains("Schema CLI"),
        "DevTools skill should expose the shared receipt schema CLI"
    );
}

#[test]
fn oracle_buildout_plan_preserves_primitive_first_acceptance_bar() {
    for needle in [
        "devtools-all-surfaces-buildout-plan-2",
        "Coverage aliases must be routing hints only.",
        "dismissPolicy",
        "Shared Receipt Envelope",
        "blocked-by-target-ambiguity",
        "devtools.targets.inspect",
        "devtools.elements.snapshot",
        "devtools.layout.measure",
        "devtools.act.*",
        "devtools.media.inspect",
        "devtools.investigate.*",
        "The minimum acceptance bar for any surface",
        "Actions popup bugs require route stack",
        "Dynamic main menu resizing bugs require before/after window rects",
        "Oversized prompt container bugs require",
        "Notes resize bugs require active note identity",
        "Dictation bugs require passive permission status",
    ] {
        assert!(
            ORACLE_BUILDOUT.contains(needle),
            "Oracle buildout plan must preserve primitive-first requirement: {needle}"
        );
    }

    assert!(
        DEVTOOLS_SKILL.contains("devtools-oracle-buildout-plan.md"),
        "DevTools skill should point agents at the Oracle-reviewed buildout plan"
    );
}

#[test]
fn investigate_cli_turns_user_bugs_into_fail_closed_proof_plans() {
    for needle in [
        "script-kit-devtools.investigate",
        "--surface",
        "--bug",
        "--screenshot",
        "needs-red-proof",
        "ready-for-green-proof",
        "blocked-by-missing-primitive",
        "blocked-by-unknown-surface",
        "target identity: listAutomationWindows + inspectAutomationWindow",
        "semantic state: getState/getElements",
        "layout state: getLayoutInfo",
        "visual proof: strict target screenshot",
        "Do not call a recipe pass a green investigation.",
        "actions popup route stack",
        "layout box model, scroll extent, overflow, resize pressure",
        "passive media readiness",
        "portal origin, return target",
    ] {
        assert!(
            INVESTIGATE.contains(needle),
            "investigation CLI must preserve bug-proof contract: {needle}"
        );
    }

    assert!(
        DEVTOOLS_SKILL.contains("Investigation CLI"),
        "DevTools skill should route user bug reports through the investigation CLI"
    );
}

#[test]
fn surfaces_cli_covers_generated_surface_contract_names() {
    for surface_kind in [
        "ScriptList",
        "ActionsDialog",
        "PromptEntity",
        "PromptChildContent",
        "ExplicitPromptEntity",
        "Webcam",
        "ClipboardHistory",
        "AppLauncher",
        "WindowSwitcher",
        "BrowserTabs",
        "GenericFilterableList",
        "Settings",
        "KitStoreBrowse",
        "KitStoreInstalled",
        "ProcessManager",
        "CurrentAppCommands",
        "DesignGallery",
        "DesignExplorer",
        "UtilityChildContent",
        "FileSearchMini",
        "FileSearchFull",
        "ThemeChooser",
        "EmojiPicker",
        "Feedback",
        "SdkReference",
        "ScriptTemplateCatalog",
        "AcpHistory",
        "AttachmentPortalBrowser",
        "AcpChat",
        "ConfirmPrompt",
    ] {
        assert!(
            SURFACE_CONTRACTS.contains(surface_kind),
            "generated surface contract should include {surface_kind}"
        );
        assert!(
            SURFACES.contains(surface_kind),
            "surface inventory CLI must preserve generated SurfaceKind {surface_kind}"
        );
    }
}

#[test]
fn surfaces_cli_indexes_all_feature_map_rows_and_owner_skills() {
    for id in 1..=47 {
        let id = format!("{id:03}");
        assert!(
            FEATURE_MAP.contains(&format!("| {id} |")),
            "feature map should include row {id}"
        );
    }

    for needle in [
        "featureMapCount",
        "ownerSkillCount",
        "main-menu-search-selection",
        "actions-popups",
        "keyboard-focus-routing",
        "file-search-portals",
        "acp-context-composer",
        "acp-chat-core",
        "mcp-context-resources",
        "sdk-script-execution",
        "protocol-automation",
        "builtin-filterable-surfaces",
        "notes-window",
        "dictation-media",
        "quick-terminal-pty",
        "prompt-runtime",
        "platform-windowing-macos",
        "storage-cache-security",
        "theme-config-preferences",
        "storybook-design",
        "dev-loop-observability",
        "window-resizing",
        "launcher-surface-contracts",
        "testing-quality-gates",
    ] {
        assert!(
            SURFACES.contains(needle) || FEATURE_MAP.contains(needle),
            "surface inventory must retain owner skill or feature-map field: {needle}"
        );
    }
}

#[test]
fn surfaces_cli_groups_oracle_buildout_batches() {
    for batch in [
        "launcher-main-actions",
        "prompt-runtime-family",
        "builtins-filterable",
        "portals-resources-context",
        "acp-chat-ai",
        "notes-dictation-media",
        "platform-windowing-permissions",
        "observability-security-storage",
    ] {
        assert!(
            SURFACES.contains(batch),
            "surface inventory must include Oracle buildout batch: {batch}"
        );
    }

    let platform_index = SURFACES
        .find("platform-windowing-permissions")
        .expect("platform/windowing batch should be present");
    let launcher_index = SURFACES
        .find("launcher-main-actions")
        .expect("launcher batch should be present");
    assert!(
        platform_index < launcher_index,
        "surface inventory should prioritize outside-in platform/window proof before inner launcher controls"
    );
    assert!(
        !SURFACES.contains("storybook-design-theme"),
        "outdated Storybook/design-lab surfaces must not be active Liquid Glass Oracle batches"
    );
    assert!(
        SURFACES.contains("liquidGlassAuditExclusions")
            && SURFACES.contains("DesignGallery")
            && SURFACES.contains("DesignExplorer"),
        "surface inventory should keep outdated design-lab surfaces as explicit audit exclusions"
    );

    for primitive in [
        "devtools.targets.watch",
        "devtools.act",
        "devtools.measure.layout",
        "devtools.prompt.inspect",
        "devtools.resources.inspect",
        "devtools.acp.inspect",
        "devtools.media.inspect",
        "devtools.permissions.inspect",
        "devtools.visual.compare",
        "devtools.events.tail",
        "devtools.investigate",
    ] {
        assert!(
            SURFACES.contains(primitive),
            "surface inventory must recommend DevTools primitive: {primitive}"
        );
    }
}

#[test]
fn skill_docs_route_broad_planning_to_surface_inventory_not_recipes() {
    for needle in [
        "Surface Inventory CLI",
        "references/devtools-surface-inventory.md",
        "protocol/MCP/CLI DevTools primitives",
        "scripted recipes remain regression packs",
        "Run `bun scripts/devtools/surfaces.ts`",
    ] {
        assert!(
            DEVTOOLS_SKILL.contains(needle) || SURFACE_INVENTORY.contains(needle),
            "DevTools docs must route broad planning through surface inventory: {needle}"
        );
    }
}
