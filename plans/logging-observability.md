# Logging and Observability Audit

Date: 2026-02-07
Agent: `codex-logging-observability`
Scope: `src/**/*.rs`

## Executive Summary

Logging is functionally rich but inconsistent across three dimensions: severity fidelity, correlation propagation, and structured payload discipline. The biggest gaps are:

1. Legacy `logging::log(...)` flattens severity to `INFO`, even when callers pass categories like `"ERROR"` and `"WARN"`.
2. Correlation IDs are not propagated through several async/channel boundaries, so multi-step flows are hard to reconstruct from logs.
3. Sensitive/high-volume payloads are still logged verbatim in key paths (stdin, protocol, MCP), despite existing truncation helpers.

Compact AI log mode (`SCRIPT_KIT_AI_LOG=1`) is implemented, but coverage is partial because many categories map to `'-'` and many module targets are not covered by target inference.

## Quantitative Snapshot

Collected with `rg` over `src/`:

- `logging::log*` calls: **1784**
- `tracing::{info,warn,error,debug,trace}!` calls: **1183**
- `logging::log*` calls using `format!`: **437**
- Interpolated tracing messages (`"... {}", x`): **18**
- `set_correlation_id(...)` call sites: **12**
- Explicit `correlation_id = ...` fields in log calls: **19**
- `#[instrument]` attributes: **98**
- `#[instrument]` attributes that explicitly include `correlation_id` fields: **0**

Compact coverage heuristic:

- Tracing macro lines estimated to resolve to `'-'` category under current `infer_category_from_target(...)`: **620 / 1183** (~52%)
- This is a heuristic based on file-module naming and current target-matching rules in `src/logging.rs:225`.

## Findings

### 1) Severity Flattening in Legacy Logger (High)

`logging::log(...)` always emits `tracing::info!`, regardless of category.

- Implementation: `src/logging.rs:971` and `src/logging.rs:988`
- Example callsites that intend non-info semantics:
  - `src/main.rs:3231` (`"ERROR"`)
  - `src/prompt_handler.rs:647` (`"WARN"`)
  - Many more (`logging::log("ERROR", ...)` appears ~105 times; `"WARN"` appears ~7 times)

Impact:

- Alerting/noise control by level becomes unreliable.
- Production triage misses real failures hidden at `INFO`.
- Compact mode keeps lowercase level char (`i`) for these events, which can mislead AI/debug operators.

Recommendation:

- Replace `logging::log` with explicit level APIs (`log_info`, `log_warn`, `log_error`) or migrate callsites to direct tracing macros.
- Keep backward-compat wrapper only for transitional callsites and mark with deprecation lint.

### 2) Correlation IDs Not Propagated Across Async Boundaries (High)

Correlation is set in some producer paths, but often not preserved when work is consumed on other threads/tasks.

Evidence:

- Protocol reader sets correlation guard then returns immediately: `src/protocol/io.rs:322` to `src/protocol/io.rs:326`
- Stdin handler processes commands later without setting correlation from `request_id`: `src/main.rs:3024` to `src/main.rs:3039`
- `ExternalCommand` docs claim requestId correlation, but implementation is partial: `src/stdin_commands.rs:43`
- Hotkey producer sets correlation: `src/hotkeys.rs:1281`
- Hotkey consumer logs later without restoring that correlation context: `src/main.rs:2452`

Impact:

- End-to-end traces for `stdin/hotkey -> window action -> script execution` split across unrelated correlation IDs.
- Debugging race conditions and user-reported incidents is slower.

Recommendation:

- Carry correlation in channel payloads (e.g., include `correlation_id` in hotkey/stdin event structs).
- Set a scoped guard in consumer tasks for the full handling lifetime.
- Add regression tests asserting same `correlation_id` appears from ingress to terminal action.

### 3) Raw Payload Logging in Sensitive Paths (High)

Some paths still log full unredacted payloads.

Evidence:

- Full stdin line logged: `src/stdin_commands.rs:182`
- Parsed command `Debug` logged: `src/stdin_commands.rs:185`
- Full protocol message logged via `{:?}`: `src/execute_script.rs:382` and `src/execute_script.rs:387`
- Full MCP RPC body logged: `src/mcp_server.rs:415`

Context:

- Redaction/summarization helper already exists: `src/logging.rs:1192` (`summarize_payload(...)`)

Impact:

- Potential leakage of user input, clipboard contents, base64 image data, tokens, or long prompts.
- Excessive log volume and slower incident analysis.

Recommendation:

- Use summary fields (`type`, `len`, `request_id`) instead of raw payloads.
- Add explicit redaction for known secret-bearing keys (`token`, `authorization`, `apiKey`, etc.).
- Add tests for payload truncation/redaction in stdin and MCP paths.

### 4) Compact AI Mode Category Coverage Gaps (Medium)

Compact formatter exists and includes `cid=...`, but many categories are unmapped or inferred as unknown.

Evidence:

- Category mapping table: `src/logging.rs:185`
- Unknown fallback emits `'-'`: `src/logging.rs:210`
- Target inference is limited: `src/logging.rs:225`
- Common categories not currently mapped include examples such as:
  - `"CHAT"`: `src/prompt_handler.rs:1614`
  - `"AI"`: `src/prompt_handler.rs:2028`
  - `"ACTIONS"`: `src/app_impl.rs:3714`
  - `"WINDOW_STATE"`: `src/window_state.rs:37`

Additional drift:

- Code maps `SCRIPT -> 'G'` and reserves `B` for `BENCH` (`src/logging.rs:204`, `src/logging.rs:209`), while AGENTS.md’s compact legend documents `B` as script.

Impact:

- Compact logs contain many `|-|` category markers, reducing scanability and tooling value.
- Documentation/code mismatch causes operator confusion.

Recommendation:

- Define a single canonical category enum shared by logger + docs.
- Map all currently used legacy categories or normalize them to the canonical set.
- Add tests that fail if unmapped literal categories are introduced.

### 5) Structured Context Is Inconsistent (Medium)

Many callsites still use interpolated strings instead of typed fields.

Evidence:

- Interpolated tracing examples: `src/app_impl.rs:431`, `src/mcp_server.rs:244`, `src/mcp_server.rs:253`
- Large `logging::log(format!(...))` usage footprint (437 callsites)

Impact:

- Harder machine filtering (`error`, `path`, `request_id`, `duration_ms` become embedded text).
- Cross-run comparisons require brittle string parsing.

Recommendation:

- Favor `tracing` field form (`error = %e`, `path = %path`, etc.).
- Keep human message concise, put diagnostics in fields.
- Use `#[instrument(...)]` strategically on ingress/egress points with stable IDs.

## Priority Remediation Plan

1. **P0**: Fix severity flattening (`logging::log` API split or migration shim).
2. **P0**: Propagate correlation through stdin/hotkey/protocol channels end-to-end.
3. **P0**: Redact/summarize stdin/protocol/MCP payload logs.
4. **P1**: Normalize compact category taxonomy and update AGENTS legend to match code.
5. **P1**: Add lint/test guardrails for new logs:
   - no raw payload logging in designated sensitive modules
   - no unmapped compact categories
   - no `"... {}"` interpolation where structured fields are appropriate

## Suggested Verification Work (for follow-up implementation)

- Unit tests in `logging.rs`:
  - category mapping completeness
  - redaction helpers
  - level mapping for legacy wrappers
- Integration tests:
  - stdin command with `requestId` keeps one `correlation_id` across all major steps
  - hotkey trigger keeps one `correlation_id` across producer + consumer logs
- Runtime smoke:
  - `SCRIPT_KIT_AI_LOG=1` run and assert no `|-|` categories for critical paths (`STDIN`, `HOTKEY`, `EXEC`, `UI`)
