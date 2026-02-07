# Tracing / Logging Audit

Date: 2026-02-07
Agent: `codex-tracing-logging`
Scope: `src/**/*.rs`, `Cargo.toml`

## Executive Summary

Overall status: **Partially effective**.

What works well:
- `tracing` + `tracing-subscriber` + `tracing-appender` are wired and active.
- Correlation IDs are consistently injected into JSON and compact stderr output.
- Several critical subsystems (stdin/protocol/executor) emit structured fields with useful diagnostics.

Main gaps:
- **Span context is effectively dropped by custom formatters**, so most `#[instrument]` usage is not visible in output.
- **No log rotation** for the primary JSONL file (`append(true)` forever).
- **High-volume legacy info logs** dominate output and reduce signal/noise.
- **Hot UI paths (notably chat/markdown render + key handling)** have limited performance instrumentation.

## Dependency/Config Check

From `Cargo.toml`:
- `tracing = "0.1"` (`Cargo.toml:46`)
- `tracing-subscriber = { version = "0.3", features = ["json", "env-filter", "time", "tracing-log"] }` (`Cargo.toml:47`)
- `tracing-appender = "0.2"` (`Cargo.toml:48`)

`perf` feature is disabled by default (`Cargo.toml:139-154`), so some performance helpers are intentionally dormant in default runs.

## Findings

### 1) High: Span context is not emitted by current formatters

Evidence:
- `CompactAiFormatter::format_event` ignores formatter context and only records event fields (`src/logging.rs:426-464`).
- `JsonWithCorrelation::format_event` also ignores formatter context and serializes only event-local fields (`src/logging.rs:534-589`).
- No `with_span_events(...)` / `FmtSpan` configuration is present (`src/logging.rs:891-937`; no other hits).

Impact:
- `#[instrument]` annotations do not provide expected traceability unless events manually repeat the same fields.
- Parent/child span hierarchy is not visible in output JSONL.
- Performance tracing via spans is limited despite broad annotation usage.

Quantitative context (repo scan):
- `#[instrument]` attrs: **104**
- explicit span macros (`info_span!`/`debug_span!`/`span!`): **5**

### 2) High: No log rotation for main JSONL

Evidence:
- Main log file opened with `.append(true)` at `src/logging.rs:863-874`.
- Session file truncates each launch (`src/logging.rs:876-888`) and is healthy for short-term use.
- Uses `tracing_appender::non_blocking(...)` only (`src/logging.rs:891-892`), with no rolling appender.

Impact:
- `~/.scriptkit/logs/script-kit-gpui.jsonl` can grow without bound.
- Long-term disk growth and degraded log tooling ergonomics.

### 3) Medium: Legacy logging dominates and skews levels to INFO

Evidence:
- `logging::log(...)` fallback maps most categories to `Info` unless category is explicitly `ERROR`/`WARN`/`DEBUG`/`TRACE` (`src/logging.rs:246-253`, `src/logging.rs:1027-1086`).
- Repo-wide callsites:
  - `logging::log(...)`: **1786**
  - direct `tracing::{trace,debug,info,warn,error}!`: **267**
- High-frequency lifecycle and event loops in `src/main.rs` are heavily legacy-logged at info level (examples: `src/main.rs:2615-2641`, `src/main.rs:3175-3201`, `src/main.rs:3696-3750`).

Runtime sample:
- `SCRIPT_KIT_AI_LOG=1` + one `{"type":"show"}` command produced **259 lines in ~8s** (`/tmp/tracing-audit-run.log`), mostly info-level noise.

Impact:
- Important warnings/errors are harder to find during incident triage.
- More I/O overhead than needed on hot paths.

### 4) Medium: Hot UI render paths lack explicit tracing/perf markers

Evidence:
- `src/prompts/markdown.rs` has no tracing instrumentation in parse/render entrypoints (`src/prompts/markdown.rs:187-976`).
- `src/prompts/chat.rs` key/render flow is primarily legacy logs; very limited structured tracing (`src/prompts/chat.rs:2881-3010`, plus only a couple of `tracing::warn!` lines around clipboard at `src/prompts/chat.rs:1251-1257`).

Related perf utility status:
- `KeyEventPerfGuard` exists but appears unused (definition in `src/perf.rs:513-543`; no external callsites found).
- `ScrollPerfGuard` is used in list scrolling (`src/app_navigation.rs:311-318`).

Impact:
- Hard to diagnose jank/regressions in markdown streaming and chat interaction from logs alone.

### 5) Medium: Non-blocking writers use defaults; drop behavior not explicitly controlled

Evidence:
- `tracing_appender::non_blocking(file)` defaults are used (`src/logging.rs:891-892`), without explicit builder settings.

Impact:
- Under log bursts, behavior/tuning is implicit rather than policy-driven.
- No explicit project-level decision in code for lossy/backpressure mode and buffer sizing.

### 6) Low: `tracing-log` feature appears unused in this crate

Evidence:
- No `log::...` macros or `tracing_log::LogTracer` usage found in `src/` and `Cargo.toml` references.

Impact:
- Potentially unnecessary feature surface unless retained for future dependency bridging.

### 7) Positive: Correlation ID design is strong and broadly enforced

Evidence:
- Thread-local + guard API (`src/logging.rs:77-113`).
- Mandatory correlation injection in both formatters (`src/logging.rs:453-463`, `src/logging.rs:554-574`).
- Ingress propagation in stdin/protocol/hotkey paths (`src/stdin_commands.rs:525-633`, `src/protocol/io.rs:409-540`, `src/hotkeys.rs:881-935`, `src/main.rs:2617-2689`, `src/main.rs:3198`).

Note:
- Fallback correlation ID is session-global when no contextual ID is set (`src/logging.rs:104-113`), which guarantees presence but can blur unrelated events.

## Direct Answers To Audit Questions

- Are we using structured logging effectively?
  - **Partially.** Several subsystems use typed fields well, but legacy message logging still dominates.
- Are spans used for performance tracing?
  - **Partially / weakly.** Spans exist in code, but current formatters do not emit span context, reducing practical value.
- Is env-filter configured properly?
  - **Mostly yes.** `RUST_LOG` override + sane fallback are present (`src/logging.rs:900-904`), but parse-fallback is silent and no runtime reload exists.
- Are we using tracing-appender for log rotation?
  - **No.** Non-blocking file writers are used, but rolling appenders are not.
- Any missing instrumentation on hot paths?
  - **Yes.** Chat/markdown render and key flows need structured timing/spans.
- Are log levels appropriate?
  - **Not consistently.** Many hot-path operational logs are emitted at info via legacy wrapper.

## Recommended Remediation Plan (Priority Ordered)

1. **Emit span context in JSON/compact formatters**
- Include current span scope/fields in `JsonWithCorrelation` and compact output.
- Optionally enable `FmtSpan::CLOSE` (or equivalent) for duration-at-close style telemetry.

2. **Add rolling log files for long-lived JSONL**
- Keep `latest-session.jsonl` behavior.
- Replace append-forever main writer with daily/hourly rotation + retention policy.

3. **Reduce info-level noise from legacy callsites**
- Reclassify noisy categories (`HOTKEY`, `VISIBILITY`, verbose `STDIN` simulation traces) to debug/trace.
- Reserve info for lifecycle milestones and actionable operator events.

4. **Instrument chat/markdown hot paths with explicit timings**
- Add structured timing for markdown parse/build/cache-hit rate and chat key/render loops.
- Emit `duration_ms`, payload sizes, and cache stats at debug (warn on threshold exceed).

5. **Make `non_blocking` policy explicit**
- Use builder API to set queue size and lossy/backpressure mode intentionally.
- Document rationale near initialization.

6. **Reassess `tracing-log` feature**
- If not bridging `log` records, drop feature to simplify configuration.
- If needed, wire `LogTracer` explicitly and test it.

## Verification Commands Used

- Static scan:
  - `rg` queries across `src/**/*.rs` and `Cargo.toml`
- Runtime sample:
  - `timeout 8 bash -lc 'printf "{\"type\":\"show\"}\n" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1' > /tmp/tracing-audit-run.log`
  - `tail -n 80 ~/.scriptkit/logs/latest-session.jsonl`
