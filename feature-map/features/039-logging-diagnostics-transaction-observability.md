# 039 Logging, Diagnostics, and Transaction Observability

Logging, diagnostics, and transaction observability turn runtime debugging into bounded, privacy-safe, machine-readable evidence for humans and agents.


## Executive Summary

Feature 039 covers the observability layer used to understand Script Kit GPUI behavior without guessing from screenshots or ad hoc grep. It includes `./dev.sh`, compact AI logs, structured JSONL logs, privacy-safe value previews, debug trace markers, protocol stats, transaction traces, transaction replay/idempotency, MCP trace resources, and AI preflight audit logs.

This feature is an operator/developer capability. It does not own UI semantics. UI behavior claims still belong to the relevant surface skill, while this feature supplies the logs, traces, receipts, resources, and tests that make those claims inspectable.

## What Users Can Do

- Start `./dev.sh` and inspect compact, low-token runtime logs.
- Switch to verbose `RUST_LOG` output when compact logs are insufficient.
- Tail recent JSONL session logs for post-failure diagnosis.
- Capture deterministic `waitFor` and `batch` transaction traces.
- Read the latest transaction trace through MCP resources instead of manually tailing files.
- Replay or dedupe transaction requests safely using request id plus command fingerprint.
- Inspect AI preflight audit records for context resolution, image state, pending parts, decisions, and correlation ids.
- Verify log privacy by checking safe previews, byte caps, truncation fields, and rate-limit suppression.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Compact logs | Low-token stderr format for dev and agent loops. | `dev.sh`, `src/logging/` |
| Structured JSONL logs | Persistent machine-readable session and audit logs. | `src/logging/`, `~/.scriptkit/logs/` |
| Safe user-value preview | Byte-capped, UTF-8-safe preview of untrusted values. | `src/logging/` |
| Log rate limiter | Time-window suppression keyed without retaining raw untrusted strings. | `src/logging/` |
| Debug markers | Stable strings for focused reproduction filtering. | `removed-docs`, source trace targets |
| Protocol stats | Live counters and health thresholds for protocol boundary failures. | `src/protocol_stats.rs` |
| Transaction executor | Deterministic `waitFor`/`batch` execution with receipts and traces. | `src/protocol/transaction_executor.rs` |
| Transaction trace | Schema-versioned JSONL execution trace with snapshots, observations, timings, and errors. | `src/protocol/transaction_trace.rs` |
| AI preflight audit | Bounded JSONL audit for ACP/AI context preparation and submit decisions. | `src/ai/preflight_audit.rs` |

## Entry Points

| Entry point | User intent | Expected target |
|---|---|---|
| `./dev.sh` | Run dev loop with compact observability | App process plus compact logs |
| `SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui` | Direct compact runtime inspection | App stderr |
| `RUST_LOG=debug ./dev.sh` | Full verbose tracing | Pretty/full tracing output |
| `~/.scriptkit/logs/latest-session.jsonl` | Inspect latest session records | JSONL log file |
| Stable markers | Filter a live reproduction | `DO_IN_TRACE`, `SCROLL_TRACE`, input history targets |
| `waitFor` / `batch` with trace mode | Capture deterministic transaction proof | Transaction trace records |
| AI preflight audit log | Inspect context submit decisions | `ai-preflight-audits.jsonl` |

## User Workflows

### Dev Loop Triage


```bash
./dev.sh
```


```bash
SCRIPT_KIT_AI_LOG=0 ./dev.sh
RUST_LOG=debug ./dev.sh
```

### Safe Log Review

When a log line includes user-controlled values such as stdin text, queries, titles, trigger names, dictation text, or ACP command display strings, it must route through safe preview and rate-limit helpers. The review checks for preview text, raw byte count, safe byte count, truncation flag, and suppression flag.

### Trace A UI Reproduction


- `DO_IN_TRACE` for current-app command normalization, intent resolution, and built-in execution routing.
- `SCROLL_TRACE` for wheel ownership, scroll metrics, and reanchor decisions.

User-entered text in these traces still uses safe previews.

### Inspect Protocol Health


Zero-tolerance counters such as parse failures and unsupported protocol versions fail health immediately. Expected/noisy counters such as unknown triggerBuiltin typos have higher thresholds.

### Capture A Transaction Trace


### Read Latest Transaction Resource


### Inspect AI Preflight Decisions

When an ACP/AI submit appears to use stale or missing context, inspect the AI preflight audit log. Key fields include correlation id, generation, draft fingerprint, chat id, message id, decision, context receipt, pending image state, context parts state, and final user content length.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Start compact dev loop | Terminal | App launching | `./dev.sh` | dev loop/logging setup | Compact logs stream | Startup log mode line |
| Escalate verbosity | Terminal | App launching | `RUST_LOG=debug` | tracing subscriber config | Full logs stream | Verbose trace output |
| Filter reproduction | Terminal/log tool | Running app | marker grep | stable trace targets | Narrow event stream | Marker lines with safe previews |
| Inspect protocol health | MCP resource | Any app state | resource read | protocol stats resource | JSON health report | `protocol_stats_report_contract` |
| Capture transaction | Protocol automation | Flow under test | `waitFor`/`batch` | transaction executor | Receipts and trace | transaction trace tests/resource |
| Read latest trace | MCP resource | Trace file present | resource read | transaction resources | Latest trace returned | `transaction_trace_resources` |
| Recover malformed trace log | Resource/read path | Corrupt JSONL line | resource read | streaming reader | Valid traces still readable | malformed-line tests |
| Inspect AI submit decision | Audit log | ACP/AI submit | log read | preflight audit append/read | Correlation record returned | `ai_preflight_persistent_audit_contract` |
| Verify safe logging | Source/test audit | Log site | code review/test | safe preview/rate limiter | Bounded private output | structured logging audits |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Compact logging active | `SCRIPT_KIT_AI_LOG=1` or `./dev.sh` default | Verbose/full logging | Compact format remains low-token and parseable. |
| Verbose tracing active | `RUST_LOG` override | Compact logging | Used only when compact logs are insufficient. |
| Safe value logged | Untrusted value reaches log site | Suppressed or emitted | Byte cap, UTF-8 boundary, metadata, rate limit. |
| Protocol stats healthy | No threshold exceeded | Not healthy | Health flags walk counters in stable order. |
| Transaction running | `waitFor`/`batch` request | Success/failure trace | Request id and fingerprint identify payload. |
| Trace persisted | Trace policy includes trace | Compacted/retained/read | File capped and malformed lines skipped. |
| Replay request seen | Duplicate request id | Idempotent return or reject | Same fingerprint may reuse; different payload rejects. |
| AI preflight audited | ACP/AI preflight decision | Compacted/read/deduped | Schema version, correlation id, malformed-line recovery. |

## Visual And Focus States


- Compact stderr lines for live development.
- JSONL session or audit records.
- MCP resource payloads for protocol stats and transaction traces.
- Runtime receipts attached to `waitFor`, `batch`, `getState`, or agentic scripts.

Screenshots are not the right first proof for this feature. Use screenshots only when the observed failure is visual; use logs, resources, and receipts for observability behavior.

## Keystrokes And Commands

| Command | Context | Behavior |
|---|---|---|
| `./dev.sh` | Repo root | Starts dev loop with compact AI logs by default. |
| `SCRIPT_KIT_AI_LOG=0 ./dev.sh` | Repo root | Disables compact mode for fuller output. |
| `RUST_LOG=debug ./dev.sh` | Repo root | Enables full debug tracing. |
| `SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui` | Direct runtime | Runs app with compact logs. |
| `tail -50 ~/.scriptkit/logs/latest-session.jsonl` | Shell | Reads recent structured session records. |

## Actions And Menus


- Main menu actions should emit domain-owned state and actions receipts.
- ACP context actions should emit context/preflight/audit receipts.
- Protocol-driven actions should emit transaction trace records when trace mode is enabled.

## Automation And Protocol Surface

| Surface | Target/proof | Notes |
|---|---|---|
| `waitFor` | Transaction executor | Poll observations, timeout/failure suggestions, typed errors. |
| `batch` | Transaction executor | Per-command snapshots, failure index, total elapsed time. |
| AI preflight audit | `ai-preflight-audits.jsonl` | Correlation-level decision ledger for ACP/AI submits. |
| Compact logs | `SCRIPT_KIT_AI_LOG=1` | Low-token live operator stream. |
| JSONL logs | `~/.scriptkit/logs/` | Durable structured records with bounded retention. |

## Data, Storage, And Privacy Boundaries

- Raw untrusted user values must not be logged directly.
- Safe previews are byte-capped, UTF-8-safe, and include metadata for raw bytes, safe bytes, truncation, and suppression.
- Rate limiter keys must not retain raw untrusted strings.
- Transaction trace logs are bounded to about 10 MiB and retain up to 2,000 compacted valid entries.
- AI preflight audit logs are bounded to about 5 MiB and retain up to 2,000 records.
- Missing files should read as empty state; malformed lines should be skipped, warned, and not block valid records.
- Draft fingerprints and preflight records should carry lengths and correlation data, not raw private content.

## Error, Empty, Loading, And Disabled States

- Missing transaction trace file returns an explicit empty resource payload.
- Malformed transaction JSONL lines are skipped so the latest valid trace can still be read.
- Unsupported trace schema versions are skipped or rejected according to the owner contract.
- Replay with same request id and different fingerprint is rejected.
- Protocol health flips when zero-tolerance counters exceed thresholds.
- AI preflight audit reading skips malformed or unsupported schema records.
- Log sites that rely only on occurrence-count gates are below the current privacy/budget standard.

## Code Ownership

| Area | Primary files | Notes |
|---|---|---|
| Dev loop | `dev.sh` | Compact log mode, watch loop, operator startup output. |
| Logging helpers | `src/logging/` | Safe previews, rate limiting, structured logging. |
| Protocol stats | `src/protocol_stats.rs` | Counters, thresholds, health report. |
| Transaction executor | `src/protocol/transaction_executor.rs` | `waitFor`/`batch` execution and receipts. |
| Transaction traces | `src/protocol/transaction_trace.rs` | Append/read/compact/replay identity. |
| AI preflight audit | `src/ai/preflight_audit.rs`, `src/ai/acp/preflight.rs`, `src/ai/window/context_preflight.rs` | Decision records, dedupe, schema handling. |
| Main-window preflight | `src/main_window_preflight/` | Runtime preflight receipts. |
| Source audits | `tests/source_audits/structured_logging.rs`, `tests/source_audits/trace_propagation.rs` | Privacy and trace propagation contracts. |
| Transaction tests | `tests/transaction_trace_contract.rs`, `tests/transaction_trace_resources.rs`, `tests/tx_trace_replay_idempotency_contract.rs`, `tests/tx_trace_wait_for_runtime_contract.rs` | Trace and replay behavior. |
| Preflight tests | `tests/ai_preflight_persistent_audit_contract.rs`, `tests/context_preflight_source_audits.rs` | Audit persistence and source contracts. |

## Invariants And Regression Risks

- Never log raw untrusted values.
- Safe previews are byte-capped, not character-capped.
- Safe preview truncation preserves UTF-8 boundaries.
- Safe log fields include preview, raw bytes, safe bytes, truncation, and suppression.
- Rate limiting must not retain raw user strings as keys.
- Compact logs remain default for dev/agent loops.
- Protocol stats should be machine-readable through MCP.
- Transaction traces are schema-versioned and bounded.
- Transaction trace readers recover from malformed JSONL.
- Repeated request ids are safe only when the command fingerprint matches.
- AI preflight audits dedupe by correlation id and tolerate schema drift.
- Observability does not replace domain-specific UI verification.

## Verification Recipes


```bash
cargo check --lib
source checks
```


```bash
cargo test --test source_audits structured_logging -- --nocapture
cargo test --test source_audits trace_propagation -- --nocapture
```


```bash
cargo test --test protocol_stats_report_contract -- --nocapture
```


```bash
cargo test --test transaction_trace_contract -- --nocapture
cargo test --test transaction_trace_resources -- --nocapture
cargo test --test tx_trace_replay_idempotency_contract -- --nocapture
cargo test --test tx_trace_wait_for_runtime_contract -- --nocapture
```


```bash
cargo test --test ai_preflight_persistent_audit_contract -- --nocapture
cargo test --test context_preflight_source_audits -- --nocapture
```


```bash
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui
```


```bash
npm run build # from feature_explorer/
source checks
jq empty feature-map/receipts/oracle-sessions.json
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md feature_explorer removed-docs removed-docs
```

## Agent Notes

- Do not use this feature as a substitute for UI behavior ownership; load the adjacent domain skill for the surface being debugged.
- To verify a log privacy fix, inspect both safe preview metadata and rate-limit suppression.
- If an AI submit has missing/stale context, inspect the AI preflight audit by correlation id and generation.
- If a JSONL file has malformed lines, the correct behavior is recovery with warnings, not total read failure.
- Screenshots are only useful after logs/receipts establish that the failing behavior is visual.

## Related Features

- 004 MCP Context Resources / SDK / Protocol Automation.
- 012 Root Unified Source Filters / Source Chips / Lazy Paging.
- 030 ACP Chat SDK APIs.
- 031 Legacy `chat()` Prompt.
- 038 Agent Skills and AI Context Catalog.
- 040 Main Window Sizing and Surface Contracts.

## Open Questions And Gaps

- Oracle noted that the focused bundle was filtered; exact definitions for some safe logging helpers should be confirmed in the live working tree before editing them.
- The ingress observer records unsupported protocol versions, but Oracle flagged that parse-message behavior may not be fully wired to that observer path; treat health reporting and live dispatch effects separately.
- Runtime compact-log proof requires launching the app and should be recorded separately when behavior, not docs, changes.
- Some source-audit command names may need exact test filtering confirmed locally before depending on them in automation.
