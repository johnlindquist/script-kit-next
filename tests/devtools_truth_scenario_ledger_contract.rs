const LEDGER: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/scenario-ledger.json"
);
const RECEIPT_SCHEMA: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/receipt.schema.json"
);
const README: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/README.md"
);
const ORACLE_PROMPT: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/devtools-truth-scenarios/oracle-prompt.md"
);

#[test]
fn truth_scenario_ledger_is_oracle_gated_and_runner_free() {
    for required in [
        "\"requiredBeforeExecution\": true",
        "\"slug\": \"new-devtools-scenarios-plan\"",
        "output.log",
        "\"status\": \"accepted\"",
        "\"hasRunner\": false",
    ] {
        assert!(
            LEDGER.contains(required),
            "truth scenario ledger must preserve Oracle gate: {required}"
        );
    }

    for forbidden_executor in [
        "scripts/agentic/index.ts",
        "scripts/agentic/user-story-audit.ts",
        "scripts/agentic/surface-navigator.ts",
        "tests/smoke/*",
    ] {
        assert!(
            LEDGER.contains(forbidden_executor),
            "truth scenario ledger must explicitly forbid executor reuse: {forbidden_executor}"
        );
    }
}

#[test]
fn truth_scenario_ledger_has_exactly_fifty_oracle_accepted_candidate_ids() {
    let ledger: serde_json::Value =
        serde_json::from_str(LEDGER).expect("truth scenario ledger must be valid JSON");
    let scenarios = ledger["scenarios"]
        .as_array()
        .expect("truth scenario ledger must have a scenarios array");
    assert_eq!(
        scenarios.len(),
        50,
        "truth scenario ledger must contain exactly 50 candidate scenarios"
    );

    for id in 1..=50 {
        let prefix = format!("dt-truth-{id:03}-");
        assert!(
            scenarios.iter().any(|scenario| scenario["id"]
                .as_str()
                .is_some_and(|sid| sid.starts_with(&prefix))),
            "missing scenario id prefix {prefix}"
        );
    }

    for scenario in scenarios {
        assert_eq!(
            scenario["oracleStatus"].as_str(),
            Some("accepted"),
            "each scenario must preserve Oracle acceptance; offending scenario: {}",
            scenario["id"]
        );
    }
}

#[test]
fn truth_scenarios_do_not_reuse_known_agentic_recipe_ids() {
    for forbidden in [
        "keyboard-hint-label-parity-stress",
        "footer-status-persistence-stress",
        "actions-command-discoverability-noop-stress",
        "destructive-confirm-modal-safety-stress",
        "file-search-preview-sanitization-stress",
        "current-app-commands-frontmost-stress",
    ] {
        assert!(
            !LEDGER.contains(&format!("\"id\": \"{forbidden}\"")),
            "ledger must not reuse existing scenario id {forbidden}"
        );
    }
}

#[test]
fn truth_scenarios_exclude_destructive_operation_markers() {
    for forbidden in [
        "\"kill\"",
        "\"terminate\"",
        "\"delete\"",
        "\"trash\"",
        "\"purge\"",
        "\"uninstall\"",
        "\"install\"",
        "\"systemPasteboardMutated\": true",
        "\"filesystemMutationOutsideSandbox\": true",
        "\"externalActivation\": true",
    ] {
        assert!(
            !LEDGER.to_lowercase().contains(&forbidden.to_lowercase()),
            "ledger contains forbidden destructive operation marker {forbidden}"
        );
    }
}

#[test]
fn truth_scenarios_require_text_action_and_state_receipts() {
    for required in [
        "visibleLabel",
        "footerIntent",
        "actionId",
        "handlerId",
        "sideEffectClass",
        "disabledReason",
        "focusOwner",
        "routeGeneration",
        "selectedSemanticId",
        "targetSurface",
        "parentSubjectId",
        "layoutGeneration",
    ] {
        assert!(
            LEDGER.contains(required),
            "missing ledger receipt field {required}"
        );
        assert!(
            RECEIPT_SCHEMA.contains(required),
            "missing receipt schema field {required}"
        );
    }
}

#[test]
fn truth_scenario_docs_preserve_oracle_first_non_executor_boundary() {
    for required in [
        "Oracle-reviewed",
        "receipt-shaped, not runner-shaped",
        "does not execute scenarios",
        "new-devtools-scenarios-plan",
    ] {
        assert!(
            README.contains(required),
            "README must document the first-slice boundary: {required}"
        );
    }

    for required in [
        "Return text only",
        "first slice must not introduce a runner",
        "Use Oracle session slug `new-devtools-scenarios-plan`",
    ] {
        assert!(
            ORACLE_PROMPT.contains(required),
            "Oracle prompt must preserve Oracle-first boundary: {required}"
        );
    }
}
