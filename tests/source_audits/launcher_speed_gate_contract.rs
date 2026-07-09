use super::read_source as read;

fn assert_contains(content: &str, needle: &str, context: &str) {
    assert!(
        content.contains(needle),
        "{context} should contain {needle:?}"
    );
}

fn workflow_step<'a>(workflow: &'a str, name: &str) -> &'a str {
    let marker = format!("      - name: {name}");
    let (_, after_marker) = workflow
        .split_once(&marker)
        .unwrap_or_else(|| panic!("workflow should contain step {name:?}"));
    after_marker
        .split("\n      - name:")
        .next()
        .expect("workflow step should have a body")
}

/// GitHub Actions wiring cannot be enforced by the compiler or by the probes themselves.
/// Keep this contract limited to proving that CI syntax-checks both probes and executes the
/// deterministic frame-identity runtime gate; probe behavior belongs in the runtime proof.
#[test]
fn deterministic_perf_workflow_runs_the_root_frame_gate() {
    let workflow = read(".github/workflows/perf-gates.yml");
    let syntax_step = workflow_step(&workflow, "Syntax-check performance probes");
    let runtime_step = workflow_step(&workflow, "Run semantic frame-identity gate");

    for probe in [
        "scripts/agentic/root-typing-lag-benchmark.ts",
        "scripts/agentic/root-search-frame-stability.ts",
    ] {
        assert_contains(syntax_step, probe, "performance-probe syntax check");
    }

    for needle in [
        "bun scripts/agentic/root-search-frame-stability.ts",
        "--binary target-agent/artifacts/root-frame-gate/script-kit-gpui",
        "--receipt .test-output/perf-gates/root-search-frame-stability.json",
    ] {
        assert_contains(runtime_step, needle, "root frame-identity runtime gate");
    }
}

#[test]
fn history_render_prep_static_gate_has_thresholds_and_row_identity_proof() {
    let benchmark = read("scripts/bench-main-menu-history-render.mjs");

    for needle in [
        "SAMPLES = 240",
        "WARMUP = 30",
        "VISIBLE_ROWS = 22",
        "filterReplacement.includes(\".measure_all()\")",
        "main_list_row_generation",
        "script-item-gen-${rowGeneration}",
        "section-header-gen-${rowGeneration}",
        "script-item-gen-{row_generation}",
        "section-header-gen-{row_generation}",
        "report.totalP95Ms > 8",
        "report.visibleRowsP95Ms > 2.5",
    ] {
        assert_contains(&benchmark, needle, "history render-prep speed gate");
    }
}

#[test]
fn passive_source_speed_contract_stays_linked_to_runtime_log_parser() {
    let filtering = read("src/app_impl/filtering_cache.rs");
    let typing_benchmark = read("scripts/agentic/root-typing-lag-benchmark.ts");
    let passive_contract = read("tests/source_audits/root_unified_passive_source_perf_contract.rs");

    assert_contains(
        &filtering,
        "[PASSIVE_SOURCE_DONE]",
        "passive-source timing logs",
    );
    assert_contains(
        &typing_benchmark,
        "PASSIVE_SOURCE_DONE",
        "typing benchmark passive-source parser",
    );
    assert_contains(
        &passive_contract,
        "root_passive_frame_times_every_passive_source",
        "passive-source source audit",
    );
    assert_contains(
        &passive_contract,
        "timed_root_passive_source",
        "passive-source source audit",
    );
}
