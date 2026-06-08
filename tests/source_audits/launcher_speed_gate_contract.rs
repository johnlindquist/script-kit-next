use super::read_source as read;

fn assert_contains(content: &str, needle: &str, context: &str) {
    assert!(
        content.contains(needle),
        "{context} should contain {needle:?}"
    );
}

fn assert_not_contains(content: &str, needle: &str, context: &str) {
    assert!(
        !content.contains(needle),
        "{context} should not contain {needle:?}"
    );
}

#[test]
fn root_typing_lag_benchmark_is_enforced_launcher_filter_gate() {
    let benchmark = read("scripts/agentic/root-typing-lag-benchmark.ts");

    for needle in [
        "root-typing-lag-benchmark",
        ".test-output\", \"root-typing-lag-benchmark",
        "SCRIPT_KIT_FILTER_PERF_LOG",
        "delete process.env.SCRIPT_KIT_PREFLIGHT_DEEP_LOG",
        "inputMode = argValue(\"--input-mode\", \"setFilter\")",
        "\"setFilter\"",
        "\"printable-key\"",
        "process.argv.includes(\"--enforce\")",
        "writeFileSync(join(outputDir, \"receipt.json\")",
        "transportAckMode = \"stateEcho\"",
        "transportAckMode,",
        "sendMs: Number(sendMs.toFixed(3))",
        "send: stats(events.map((event) => event.sendMs))",
        "protocolResponsesPath: sessionStatus.protocolResponses ?? null",
        "envelope.kind === \"protocolResponse\"",
    ] {
        assert_contains(&benchmark, needle, "root typing-lag benchmark");
    }

    assert_not_contains(
        &benchmark,
        "tail.includes(\"event_type=stdin_command_parsed\")",
        "root typing-lag benchmark transport ack",
    );
    assert_not_contains(
        &benchmark,
        "tail.includes(\"event_type=stdin_parse_failed\")",
        "root typing-lag benchmark transport ack",
    );

    for needle in [
        "summary.typing.inputEcho.p50Ms > 20",
        "summary.typing.inputEcho.p95Ms > 50",
        "summary.typing.inputEcho.maxMs > 150",
        "summary.typing.cadenceOverrunMaxMs > 75",
        "summary.perfLogs.groupDone.p95Ms > 35",
        "summary.perfLogs.searchTotal.p95Ms > 15",
        "summary.perfLogs.passiveSources.all.maxMs > 20",
        "summary.perfLogs.passiveSources.implicit.maxMs > 12",
        "summary.perfLogs.maxLogLineBytes > 2048",
        "summary.perfLogs.preflightDeepLineCount !== 0",
        "inputEchoP50Ms: 20",
        "inputEchoP95Ms: 50",
        "inputEchoMaxMs: 150",
        "cadenceOverrunMaxMs: 75",
        "groupDoneP95Ms: 35",
        "searchTotalP95Ms: 15",
        "passiveSourceMaxMs: 20",
        "implicitPassiveSourceMaxMs: 12",
        "maxLogLineBytes: 2048",
        "deep preflight lines present",
    ] {
        assert_contains(&benchmark, needle, "launcher typing-lag threshold contract");
    }
}

#[test]
fn root_typing_lag_receipt_includes_semantic_preflight_and_perf_summaries() {
    let benchmark = read("scripts/agentic/root-typing-lag-benchmark.ts");

    for needle in [
        "schemaVersion: 1",
        "status: failures.length === 0 ? \"pass\" : \"fail\"",
        "summary",
        "typingReceipts",
        "emptyReceipts",
        "logPath: sessionStatus.log",
        "responsesPath: sessionStatus.responses",
        "computedMismatchCount",
        "preflightFingerprint",
        "visibleResultCount",
    ] {
        assert_contains(&benchmark, needle, "root typing-lag receipt");
    }
}

#[test]
fn root_search_frame_stability_is_the_async_provider_regression_gate() {
    let benchmark = read("scripts/agentic/root-search-frame-stability.ts");

    for needle in [
        "root-search-frame-stability",
        "delayMs: 250",
        "function assertSameFrame",
        "function classifyRootFileBaseline",
        "function hasWarmRootFileCache",
        "sampleUntilRootFileSettled",
        "settled-provider-early-visible-loading",
        "baselineProof",
        "observedAsyncHandoff",
        "visibleLoading !== true",
        "status.visibleResultCount === 0",
        "status.cacheEntryCount",
        "status.cacheResultCount",
        "requiredSettledStableSamples",
        "mainWindowPreflight",
        "schemaVersion: 2",
        "baseline",
        "settled",
        "samples",
    ] {
        assert_contains(&benchmark, needle, "root search frame-stability gate");
    }

    assert_not_contains(
        &benchmark,
        "loading !== true for delayed provider baseline",
        "root search frame-stability gate must not require catching the transient provider-loading tick",
    );
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
