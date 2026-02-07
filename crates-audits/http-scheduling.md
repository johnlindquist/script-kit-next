# HTTP + Scheduling Crate Audit

Date: 2026-02-07  
Agent: `codex-http-scheduling`

## Scope
- `ureq` (`3`, features: `json`, `rustls`)
- `croner` (`3.0`)
- `english-to-cron` (`0.1`)
- `chrono` (`0.4`, feature: `serde`)

## Executive Summary
- `ureq` was missing a few hardening controls and retry behavior for transient failures. I added explicit HTTPS-only transport, explicit timeout coverage for request phases, disabled automatic status-as-error to preserve API error body parsing, and added bounded retries for transient failures.
- Scheduler cron execution was UTC-centric for next-run computation, which can violate user expectation for local time phrases like "every day at 9am". I switched scheduling to timezone-aware local-time cron semantics and normalize stored execution timestamps back to UTC.
- `croner` support is broad and can parse standard variants used in this project configuration.
- `english-to-cron` is usable but heuristic; I added compatibility tests ensuring generated cron strings parse under `croner`.

## Findings and Actions

### 1) `ureq`: timeouts, TLS, HTTP errors, retries

#### What was found
- Agent configuration only set connect/read timeouts; non-2xx handling relied on custom response parsing, but default `ureq` status handling can interfere.
- No retry policy existed for transient API failures.

#### What changed
- `src/ai/providers.rs`
  - `create_agent()` now sets:
    - `http_status_as_error(false)`
    - `https_only(true)`
    - `timeout_global`, `timeout_connect`, `timeout_send_request`, `timeout_send_body`, `timeout_recv_response`, `timeout_recv_body`
  - Added bounded retry helper: `send_json_with_retry(...)`
    - Retryable HTTP status: `408`, `429`, `5xx`
    - Retryable transport errors: timeout/io/host-not-found/connection-failed/protocol/body-stalled
    - Max attempts: `3`
    - Exponential backoff base: `250ms`
  - Applied retry wrapper to OpenAI, Anthropic, and Vercel non-streaming + streaming request paths.
  - Retry logs include `correlation_id` + structured fields (`provider`, `operation`, `attempt`, `status/error`, `retry_in_ms`).

#### Validation
- Added tests in `src/ai/providers.rs`:
  - `test_create_agent_disables_status_errors_and_enforces_https`
  - `test_should_retry_http_status_when_transient`
  - `test_should_not_retry_http_status_when_permanent_client_error`
  - `test_should_retry_transport_error_timeout`
  - `test_should_not_retry_transport_error_bad_uri`
- Ran: `cargo test --lib ai::providers::tests` (pass)

#### Residual risk / gap
- Retries currently do not honor provider `Retry-After` headers.
- Retries are applied to POST requests; depending on provider semantics this may duplicate billable operations when network ambiguity occurs.
- No jitter in backoff (synchronized retry storms are still possible under load).

### 2) `croner`: standard cron expression support

#### What was found
- Parser is configured with `with_seconds_optional()` and `with_dom_and_dow()` which covers common 5-field and optional-seconds formats.

#### What changed
- Added compatibility coverage in `src/scheduler.rs`:
  - `test_parse_cron_supports_standard_variants`
  - Includes ranges/steps, month names, DOW aliases, numeric Sunday, and 6-field seconds example.

#### Validation
- Ran: `cargo test --lib scheduler::tests` (pass)

#### Residual risk / gap
- Quartz-only syntax (e.g. `?`, `L`, `W`, `#`) is not explicitly guaranteed by our tests and should be treated as unsupported unless validated.

### 3) `english-to-cron`: reliability

#### What was found
- `english-to-cron` provides convenient natural-language conversion but is not a deterministic NL parser for all phrasing variants.

#### What changed
- Added `test_natural_to_cron_output_is_parseable_by_croner` in `src/scheduler.rs` to ensure NL output is consumable by the runtime parser.

#### Residual risk / gap
- Ambiguous phrases and locale/timezone nuances may still convert unexpectedly.
- Recommended behavior: continue treating natural language as convenience input; for mission-critical schedules, prefer explicit cron.

### 4) `chrono`: timezone handling correctness

#### What was found
- Next-run computation previously used UTC directly, causing local-time schedules to drift from user expectation.

#### What changed
- `src/scheduler.rs`
  - Added `find_next_occurrence_utc_in_timezone(...)`
  - Scheduler now computes next occurrence in local timezone semantics (`Local`), then converts back to UTC for storage/comparison.
  - Added test `test_find_next_occurrence_utc_in_timezone_keeps_local_hour` using a fixed UTC-8 offset to verify 9am local remains 9am local.

#### Validation
- Ran: `cargo test --lib scheduler::tests` (pass)

#### Residual risk / gap
- DST transitions with nonexistent/ambiguous local times are not explicitly regression-tested yet.

## Verification Commands Executed
- `cargo test --lib test_create_agent_disables_status_errors_and_enforces_https`
- `cargo test --lib scheduler::tests`
- `cargo test --lib ai::providers::tests`
- `cargo check`
- `cargo clippy --lib -- -D warnings` (fails due unrelated pre-existing issues outside this scope)

## Clippy Failure Context (Unrelated)
`cargo clippy --lib -- -D warnings` failed due existing warnings/errors in files not touched by this audit (for example `src/theme/types.rs`, `src/watcher.rs`, `src/actions/dialog.rs`, `src/prompts/div.rs`, `src/prompts/markdown.rs`, `src/agents/parser.rs`).

