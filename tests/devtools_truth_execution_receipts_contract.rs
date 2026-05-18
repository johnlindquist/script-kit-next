use serde_json::Value;

const MANIFEST: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-binding-v1/slice.manifest.json"
);
const TARGET_LIFECYCLE_MANIFEST: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-target-lifecycle-v1/slice.manifest.json"
);
const TARGET_LIFECYCLE_V2_MANIFEST: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-target-lifecycle-v2/slice.manifest.json"
);
const DT_011: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-binding-v1/dt-truth-011-actions-parent-filter-mutates-while-open/scenario.receipt.json"
);
const DT_013: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-binding-v1/dt-truth-013-actions-section-heading-not-action-target/scenario.receipt.json"
);
const DT_015: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-binding-v1/dt-truth-015-actions-dialog-parent-focus-return-truth/scenario.receipt.json"
);
const DT_013_TARGET_LIFECYCLE: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-target-lifecycle-v1/dt-truth-013-actions-section-heading-not-action-target/scenario.receipt.json"
);
const DT_013_TARGET_LIFECYCLE_V2: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipts/direct-actions-target-lifecycle-v2/dt-truth-013-actions-section-heading-not-action-target/scenario.receipt.json"
);

const ALLOWED_PRIMITIVES: &[&str] = &[
    "scripts/devtools/targets.ts",
    "scripts/devtools/elements.ts",
    "scripts/devtools/focus.ts",
    "scripts/devtools/text.ts",
    "scripts/devtools/layout.ts",
    "scripts/devtools/act.ts",
    "scripts/devtools/actions.ts",
];

const FORBIDDEN_EXECUTORS: &[&str] = &[
    "scripts/agentic/index.ts",
    "scripts/agentic/user-story-audit.ts",
    "scripts/agentic/surface-navigator.ts",
    "tests/smoke/",
    "stress recipe",
    "recipe catalog",
];

const REQUIRED_TRUTH_MODEL_FIELDS: &[&str] = &[
    "routeGeneration",
    "focusOwner",
    "selectedSemanticId",
    "visibleLabel",
    "footerIntent",
    "actionId",
    "handlerId",
    "sideEffectClass",
    "disabledReason",
    "targetSurface",
    "parentSubjectId",
    "layoutGeneration",
];

fn parse(json: &str) -> Value {
    serde_json::from_str(json).expect("receipt JSON must parse")
}

fn command_script(argv: &Value) -> Option<&str> {
    argv.as_array()?
        .iter()
        .filter_map(Value::as_str)
        .find(|part| {
            part.starts_with("scripts/devtools/")
                || part.starts_with("scripts/agentic/")
                || part.starts_with("tests/smoke/")
        })
}

#[test]
fn direct_actions_binding_slice_has_exact_scenarios_and_no_runner() {
    let manifest = parse(MANIFEST);
    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["sliceId"], "direct-actions-binding-v1");
    assert_eq!(manifest["oracleSession"], "devtools-truth-execution");
    assert_eq!(
        manifest["scenarioLedgerOracleSession"],
        "new-devtools-scenarios-plan"
    );
    assert_eq!(manifest["executor"], "direct-devtools-primitives");
    assert_eq!(manifest["hasRunner"], false);
    assert_eq!(manifest["forbiddenExecutorsUsed"], false);

    let scenario_ids = manifest["scenarioIds"]
        .as_array()
        .expect("scenarioIds must be an array")
        .iter()
        .map(|value| value.as_str().expect("scenario id must be string"))
        .collect::<Vec<_>>();
    assert_eq!(
        scenario_ids,
        vec![
            "dt-truth-011-actions-parent-filter-mutates-while-open",
            "dt-truth-013-actions-section-heading-not-action-target",
            "dt-truth-015-actions-dialog-parent-focus-return-truth",
        ],
        "first direct execution slice must stay small and exact"
    );

    for forbidden in FORBIDDEN_EXECUTORS {
        assert!(
            !MANIFEST.contains(forbidden),
            "manifest must not reference forbidden executor {forbidden}"
        );
    }
}

#[test]
fn direct_actions_binding_receipts_have_required_truth_schema_and_safety() {
    for (expected_id, raw) in [
        (
            "dt-truth-011-actions-parent-filter-mutates-while-open",
            DT_011,
        ),
        (
            "dt-truth-013-actions-section-heading-not-action-target",
            DT_013,
        ),
        (
            "dt-truth-015-actions-dialog-parent-focus-return-truth",
            DT_015,
        ),
    ] {
        let receipt = parse(raw);
        assert_eq!(receipt["schemaVersion"], 1);
        assert_eq!(receipt["scenarioId"], expected_id);
        assert_eq!(receipt["oracleSession"], "devtools-truth-execution");
        assert_eq!(
            receipt["scenarioLedgerOracleSession"],
            "new-devtools-scenarios-plan"
        );
        assert_eq!(receipt["executor"], "direct-devtools-primitives");

        let result = receipt["result"].as_str().expect("result must be string");
        assert!(
            matches!(
                result,
                "pass"
                    | "fail"
                    | "blocked-by-missing-primitive"
                    | "blocked-by-unsafe-operation"
                    | "needs-oracle-review"
            ),
            "unexpected scenario result {result}"
        );

        let truth_model = receipt["truthModel"]
            .as_object()
            .expect("truthModel must be object");
        for field in REQUIRED_TRUTH_MODEL_FIELDS {
            assert!(
                truth_model.contains_key(*field),
                "{expected_id} missing truthModel.{field}"
            );
        }

        let user_path = receipt["userPath"]
            .as_array()
            .expect("userPath must be array");
        assert!(
            !user_path.is_empty(),
            "{expected_id} must record user path steps"
        );
        for step in user_path {
            assert_eq!(
                step["controlChannel"], "direct-primitive",
                "{expected_id} must use direct primitives only"
            );
            assert!(
                step["receiptRef"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(".json")),
                "{expected_id} userPath step must reference a JSON primitive receipt"
            );
        }

        for safety_field in [
            "destructiveOperationObserved",
            "systemPasteboardChanged",
            "filesystemMutationOutsideSandbox",
            "externalActivation",
        ] {
            assert_eq!(
                receipt["safety"][safety_field], false,
                "{expected_id} must preserve non-destructive safety field {safety_field}"
            );
        }
    }
}

#[test]
fn direct_actions_binding_receipts_only_reference_allowed_devtools_primitives() {
    for (expected_id, raw) in [
        (
            "dt-truth-011-actions-parent-filter-mutates-while-open",
            DT_011,
        ),
        (
            "dt-truth-013-actions-section-heading-not-action-target",
            DT_013,
        ),
        (
            "dt-truth-015-actions-dialog-parent-focus-return-truth",
            DT_015,
        ),
    ] {
        for forbidden in FORBIDDEN_EXECUTORS {
            assert!(
                !raw.contains(forbidden),
                "{expected_id} must not reference forbidden executor {forbidden}"
            );
        }

        let receipt = parse(raw);
        let commands = receipt["executorProvenance"]["topLevelCommands"]
            .as_array()
            .expect("executorProvenance.topLevelCommands must be array");
        assert!(
            !commands.is_empty(),
            "{expected_id} must record top-level primitive commands"
        );
        for command in commands {
            let script = command_script(&command["argv"])
                .expect("each command argv must include a script path");
            assert!(
                ALLOWED_PRIMITIVES.contains(&script),
                "{expected_id} used non-allowed command path {script}"
            );
        }
    }
}

#[test]
fn direct_actions_binding_slice_has_expected_primitive_coverage_by_scenario() {
    let required = [
        (
            "dt-truth-011-actions-parent-filter-mutates-while-open",
            parse(DT_011),
            vec![
                "targets.inspect",
                "elements.snapshot",
                "focus.inspect",
                "text.measure",
                "act.open-actions",
                "actions.inspect",
                "act.set-input",
                "layout.measure",
            ],
        ),
        (
            "dt-truth-013-actions-section-heading-not-action-target",
            parse(DT_013),
            vec![
                "targets.inspect",
                "act.open-actions",
                "actions.inspect",
                "act.set-input",
                "elements.snapshot",
                "focus.inspect",
                "act.key",
                "layout.measure",
            ],
        ),
        (
            "dt-truth-015-actions-dialog-parent-focus-return-truth",
            parse(DT_015),
            vec![
                "targets.inspect",
                "focus.inspect",
                "text.measure",
                "act.open-actions",
                "actions.inspect",
                "act.key",
                "act.set-input",
                "elements.snapshot",
                "layout.measure",
            ],
        ),
    ];

    for (scenario_id, receipt, expected_commands) in required {
        let primitive_commands = receipt["primitiveReceipts"]
            .as_array()
            .expect("primitiveReceipts must be array")
            .iter()
            .filter_map(|entry| entry["command"].as_str())
            .collect::<Vec<_>>();
        for expected in expected_commands {
            assert!(
                primitive_commands.contains(&expected),
                "{scenario_id} missing primitive command coverage {expected}; got {primitive_commands:?}"
            );
        }
    }
}

#[test]
fn direct_actions_binding_records_first_slice_as_blocked_not_green() {
    let manifest = parse(MANIFEST);
    assert_eq!(manifest["summary"]["pass"], 0);
    assert_eq!(manifest["summary"]["blockedByMissingPrimitive"], 3);

    for (expected_id, raw) in [
        (
            "dt-truth-011-actions-parent-filter-mutates-while-open",
            DT_011,
        ),
        (
            "dt-truth-013-actions-section-heading-not-action-target",
            DT_013,
        ),
        (
            "dt-truth-015-actions-dialog-parent-focus-return-truth",
            DT_015,
        ),
    ] {
        let receipt = parse(raw);
        assert_eq!(
            receipt["result"], "blocked-by-missing-primitive",
            "{expected_id} should preserve the direct DevTools blocker instead of claiming green"
        );
        let checks = receipt["truthChecks"]
            .as_array()
            .expect("truthChecks must be an array");
        assert!(
            checks.iter().any(|check| check["status"] == "blocked"),
            "{expected_id} must name the blocked truth check"
        );
    }
}

#[test]
fn direct_actions_target_lifecycle_slice_records_heading_truth_and_submit_failure() {
    let manifest = parse(TARGET_LIFECYCLE_MANIFEST);
    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["sliceId"], "direct-actions-target-lifecycle-v1");
    assert_eq!(manifest["oracleSession"], "actions-target-session-fix");
    assert_eq!(manifest["executor"], "direct-devtools-primitives");
    assert_eq!(manifest["hasRunner"], false);
    assert_eq!(manifest["summary"]["fail"], 1);
    assert_eq!(manifest["summary"]["pass"], 0);
    assert_eq!(
        manifest["scenarioIds"]
            .as_array()
            .expect("scenarioIds must be an array")
            .iter()
            .map(|value| value.as_str().expect("scenario id must be string"))
            .collect::<Vec<_>>(),
        vec!["dt-truth-013-actions-section-heading-not-action-target"]
    );

    let receipt = parse(DT_013_TARGET_LIFECYCLE);
    assert_eq!(receipt["schemaVersion"], 1);
    assert_eq!(
        receipt["scenarioId"],
        "dt-truth-013-actions-section-heading-not-action-target"
    );
    assert_eq!(receipt["result"], "fail");
    assert_eq!(
        receipt["truthModel"]["selectedSemanticId"],
        "choice:2:toggle_info"
    );
    assert_eq!(receipt["truthModel"]["actionId"], "toggle_info");
    assert_eq!(receipt["safety"]["submitAttempted"], false);

    for field in REQUIRED_TRUTH_MODEL_FIELDS {
        assert!(
            receipt["truthModel"]
                .as_object()
                .expect("truthModel must be object")
                .contains_key(*field),
            "target lifecycle receipt missing truthModel.{field}"
        );
    }

    let checks = receipt["truthChecks"]
        .as_array()
        .expect("truthChecks must be array");
    for expected_pass in [
        "actionsDialogTargetIsExplicit",
        "sectionRowsAreNonExecutable",
        "selectedSemanticIdExcludesHeadings",
        "keyboardNavigationSkipsHeadings",
    ] {
        assert!(
            checks
                .iter()
                .any(|check| check["name"] == expected_pass && check["status"] == "pass"),
            "expected pass check {expected_pass}"
        );
    }
    assert!(
        checks
            .iter()
            .any(|check| check["name"] == "safeSubmitRequiresLiveTarget"
                && check["status"] == "fail"
                && check["actual"]
                    .as_str()
                    .is_some_and(|actual| actual.contains("session_dead"))),
        "receipt must preserve the live-target submit failure"
    );
}

#[test]
fn direct_actions_target_lifecycle_uses_only_direct_devtools_primitives() {
    for forbidden in FORBIDDEN_EXECUTORS {
        assert!(
            !TARGET_LIFECYCLE_MANIFEST.contains(forbidden),
            "manifest must not reference forbidden executor {forbidden}"
        );
        assert!(
            !DT_013_TARGET_LIFECYCLE.contains(forbidden),
            "receipt must not reference forbidden executor {forbidden}"
        );
    }

    let receipt = parse(DT_013_TARGET_LIFECYCLE);
    let commands = receipt["executorProvenance"]["topLevelCommands"]
        .as_array()
        .expect("executorProvenance.topLevelCommands must be array");
    assert_eq!(commands.len(), 16);
    for command in commands {
        let script =
            command_script(&command["argv"]).expect("each command argv must include a script path");
        assert!(
            ALLOWED_PRIMITIVES.contains(&script),
            "target lifecycle slice used non-allowed command path {script}"
        );
    }

    let primitive_commands = receipt["primitiveReceipts"]
        .as_array()
        .expect("primitiveReceipts must be array")
        .iter()
        .filter_map(|entry| entry["command"].as_str())
        .collect::<Vec<_>>();
    for expected in [
        "actions.inspect",
        "targets.list",
        "act.set-input",
        "elements.snapshot",
        "focus.inspect",
        "act.key",
        "layout.measure",
        "targets.inspect",
    ] {
        assert!(
            primitive_commands.contains(&expected),
            "target lifecycle slice missing primitive command coverage {expected}; got {primitive_commands:?}"
        );
    }
}

#[test]
fn direct_actions_target_lifecycle_v2_records_submit_lifecycle_green() {
    let manifest = parse(TARGET_LIFECYCLE_V2_MANIFEST);
    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["sliceId"], "direct-actions-target-lifecycle-v2");
    assert_eq!(manifest["oracleSession"], "actions-submit-lifecycle");
    assert_eq!(manifest["executor"], "direct-devtools-primitives");
    assert_eq!(manifest["hasRunner"], false);
    assert_eq!(manifest["summary"]["pass"], 1);
    assert_eq!(manifest["summary"]["fail"], 0);

    let receipt = parse(DT_013_TARGET_LIFECYCLE_V2);
    assert_eq!(receipt["result"], "pass");
    assert_eq!(
        receipt["truthModel"]["selectedSemanticId"],
        "choice:2:toggle_info"
    );
    assert_eq!(receipt["truthModel"]["actionId"], "toggle_info");
    assert_eq!(receipt["safety"]["submitAttempted"], true);
    assert_eq!(
        receipt["safety"]["submitPreflightSelectedSemanticId"],
        "choice:2:toggle_info"
    );
    assert_eq!(
        receipt["safety"]["submitLifecycleState"],
        "source-closed-parent-live"
    );

    for field in REQUIRED_TRUTH_MODEL_FIELDS {
        assert!(
            receipt["truthModel"]
                .as_object()
                .expect("truthModel must be object")
                .contains_key(*field),
            "target lifecycle v2 receipt missing truthModel.{field}"
        );
    }

    let checks = receipt["truthChecks"]
        .as_array()
        .expect("truthChecks must be array");
    for expected_pass in [
        "selectedSemanticIdExcludesHeadings",
        "safeSubmitLeavesParentInspectable",
        "postSubmitMainTargetLive",
    ] {
        assert!(
            checks
                .iter()
                .any(|check| check["name"] == expected_pass && check["status"] == "pass"),
            "expected v2 pass check {expected_pass}"
        );
    }

    let commands = receipt["executorProvenance"]["topLevelCommands"]
        .as_array()
        .expect("executorProvenance.topLevelCommands must be array");
    assert_eq!(commands.len(), 16);
    for command in commands {
        let script =
            command_script(&command["argv"]).expect("each command argv must include a script path");
        assert!(
            ALLOWED_PRIMITIVES.contains(&script),
            "target lifecycle v2 used non-allowed command path {script}"
        );
    }
}
