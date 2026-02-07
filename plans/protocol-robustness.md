# Protocol Robustness Audit

Date: 2026-02-07
Agent: `codex-protocol-robustness`
Scope: `src/**/*.rs`, `docs/PROTOCOL.md`

## Executive Summary

The protocol layer has strong foundations (typed `Message` enum, graceful parse pipeline, bounded inter-thread channels, parse issue correlation IDs), but there are high-impact reliability gaps:

1. Unbounded JSON line reads allow memory amplification from oversized or unterminated lines.
2. Reader/writer backpressure can deadlock or stall when the script stops reading stdin.
3. UI-originated responses are intentionally dropped on full channels, breaking request/response guarantees.
4. Multiple documented/declared message types are not implemented in runtime dispatch and fall back to generic unhandled toasts.
5. `docs/PROTOCOL.md` is materially out of sync with implementation (parser API, message counts, stdin commands, examples).

No classic memory-unsafe buffer overflow was found in Rust code paths reviewed, but there are clear memory pressure / DoS vectors from unbounded input sizes.

## What I Reviewed

- Protocol parsing and streaming:
  - `src/protocol/io.rs`
  - `src/protocol/message.rs`
  - `src/protocol/mod.rs`
- Script session runtime wiring:
  - `src/execute_script.rs`
  - `src/executor/runner.rs`
  - `src/executor/selected_text.rs`
  - `src/ai/sdk_handlers.rs`
  - `src/prompt_handler.rs`
  - `src/app_impl.rs`
- External stdin control protocol:
  - `src/stdin_commands.rs`
  - `src/main.rs`
- Documentation:
  - `docs/PROTOCOL.md`

## Strengths (Current Good Behavior)

- Parse pipeline classifies malformed input without crashing (`MissingType`, `UnknownType`, `InvalidPayload`, `ParseError`) in `src/protocol/io.rs`.
- Parse logs use truncated previews (`MAX_RAW_LOG_PREVIEW`) to avoid dumping large payloads in protocol logs (`src/protocol/io.rs:15`, `src/logging.rs:1208`).
- Protocol parse issues include generated `correlation_id` (`src/protocol/io.rs:105-124`) and can propagate to user-visible toast path (`src/execute_script.rs:307-383`, `src/prompt_handler.rs:612-644`).
- Response/prompt channels are bounded, preventing unbounded queue growth (`src/execute_script.rs:131`, `src/execute_script.rs:186`, `src/stdin_commands.rs:171`).

## Findings (Severity-Ordered)

### PRB-001 (High): Unbounded line size in JSONL readers enables memory amplification

Evidence:
- `JsonlReader` uses `BufRead::read_line` into a reusable `String` with no max length guard (`src/protocol/io.rs:251`, `src/protocol/io.rs:299`).
- External stdin listener also reads unbounded lines with no size limit (`src/stdin_commands.rs:179-196`).

Impact:
- A script can emit a very large line (or never-ending line before newline), forcing large allocation growth.
- This is a practical memory pressure / DoS vector for both script protocol and external stdin command channel.

Recommendation:
- Add explicit per-line byte limits for both readers (for example `MAX_PROTOCOL_LINE_BYTES`, `MAX_STDIN_COMMAND_BYTES`).
- If exceeded: emit structured parse issue, truncate logging, drain until newline, continue safely.
- Add tests for over-limit behavior and recovery on subsequent valid lines.

---

### PRB-002 (High): Reader/writer backpressure can block stdout draining (deadlock risk)

Evidence:
- Response channel is bounded sync channel: `mpsc::sync_channel::<Message>(100)` (`src/execute_script.rs:186`).
- Reader thread sends many direct responses via blocking `reader_response_tx.send(...)` (`src/execute_script.rs:388`, `536`, `712`, `764`, `873`, `907`, `958`, `1026`, `1100`, `1118`, `1207`).
- Writer thread is the only consumer and can block on writing/flushing child stdin if child is not reading (`src/execute_script.rs:223-283`).

Impact:
- If child stops reading stdin and continues writing stdout, the writer blocks, response channel fills, reader blocks on `send`, and stdout is no longer drained.
- This can deadlock/stall bidirectional communication.

Recommendation:
- Convert reader direct-response path to non-blocking send with timeout/drop policy + explicit error response strategy.
- Consider separate high-priority channel for protocol-critical responses.
- Add watchdog/health metrics for queue depth and writer blocked duration.
- Add integration test that simulates script flooding requests without reading stdin.

---

### PRB-003 (High): UI path drops protocol responses when channel is full

Evidence:
- `submit_prompt_response` uses `try_send`; full channel drops response (`src/app_impl.rs:7113-7123`).
- Cancellation `Exit` may also be dropped when channel full (`src/app_impl.rs:6289-6296`).

Impact:
- Request/response semantics become lossy under pressure.
- Script may hang waiting for missing submit/exit, or diverge from expected state.

Recommendation:
- Define protocol delivery semantics explicitly:
  - either guaranteed delivery (block with bounded timeout + user feedback),
  - or explicit `responseDropped` event back to script.
- At minimum, surface dropped response to protocol/UI as structured error with correlation ID.

---

### PRB-004 (Medium): Unknown-type classification depends on serde error string matching

Evidence:
- `parse_message_graceful` determines `UnknownType` via `error_str.contains("unknown variant")` (`src/protocol/io.rs:187-196`).

Impact:
- Fragile to serde error wording changes.
- Misclassification can route errors to wrong handling path (unknown type vs invalid payload).

Recommendation:
- Use explicit type-name registry or two-step parse strategy:
  - extract `type` string,
  - check against known type set,
  - then parse full payload.
- Add tests guarding classification independent of exact serde error text.

---

### PRB-005 (Medium): Malformed JSON and missing-type parse issues are not surfaced to UI

Evidence:
- Parse issue callback in reader thread reports only `InvalidPayload` and `UnknownType` (`src/execute_script.rs:309-313`).
- `MissingType` and `ParseError` are skipped from user-facing `PromptMessage::ProtocolError` despite being captured in parser (`src/protocol/io.rs:328-414`).

Impact:
- Script authors may not see actionable feedback for malformed lines unless inspecting logs.

Recommendation:
- Make surfacing policy explicit.
- If suppressing noise is desired, add rate-limited summary toast for repeated parse errors.

---

### PRB-006 (Medium): Protocol handshake exists in type system but is not implemented in runtime dispatch

Evidence:
- `Hello`/`HelloAck` variants and constructors exist (`src/protocol/message.rs:32-56`, `1535-1575`).
- Parse tests exist (`src/protocol/io.rs:796-909`).
- Runtime dispatch in `execute_script` has no `Message::Hello` handling path; unmatched variants fall to generic unhandled toast (`src/execute_script.rs:1231-1452`, especially fallback at `src/execute_script.rs:1445`).

Impact:
- Capabilities negotiation is effectively non-operational at runtime.
- Forward-compat claims rely on parser behavior but not negotiated runtime behavior.

Recommendation:
- Implement `Hello -> HelloAck` exchange early in session.
- Gate optional features by negotiated capability flags.

---

### PRB-007 (Medium): Declared protocol variants with request IDs are not implemented in runtime handler

Evidence:
- Message enum includes request/response families with constructors/accessors (for example `GetElements`, `RunScriptlet`, `GetScriptlets`, `SimulateClick`, `GetMenuBar`, `ExecuteMenuAction`) in `src/protocol/message.rs:860-1112`.
- No corresponding handling paths found in `src/execute_script.rs` dispatch; unmatched variants reach generic `UnhandledMessage` fallback (`src/execute_script.rs:1445`).
- AI SDK path explicitly returns `None` for `AiAppendMessage`, `AiSendMessage`, `AiSetSystemPrompt`, `AiSubscribe`, `AiUnsubscribe` (`src/ai/sdk_handlers.rs:326-330`), while `execute_script` only explicitly forwards `AiStartChat` and `AiFocus` (`src/execute_script.rs:1412-1431`).

Impact:
- SDK can emit validly-typed messages that parse successfully but are non-functional at runtime.
- Behavior degrades to warning/toast rather than deterministic protocol response.

Recommendation:
- Publish an explicit implementation matrix (supported / parsed-only / reserved).
- For parsed-but-unimplemented request messages, return structured protocol error response (with `requestId`) instead of only local toast.

---

### PRB-008 (Medium): External stdin control protocol logs full raw commands (including user input)

Evidence:
- Raw command line logged: `logging::log("STDIN", &format!("Received: {}", line))` (`src/stdin_commands.rs:182`).
- Main dispatch logs user-provided command content, including fallback input text (`src/main.rs:3664-3668`).

Impact:
- Potential log leakage of sensitive command contents.
- Log amplification if very large input lines are injected.

Recommendation:
- Reuse payload summarization/truncation strategy used by protocol send logs.
- Redact sensitive fields by command type where appropriate.

---

### PRB-009 (Low): `trim()` on incoming JSONL lines changes edge-case payload semantics

Evidence:
- Reader uses `let trimmed = self.line_buffer.trim();` before parse (`src/protocol/io.rs:258`, `305`).

Impact:
- Leading/trailing whitespace is removed before parsing. Usually harmless for object JSON, but mutates raw payload representation and could hide formatting bugs.

Recommendation:
- Prefer newline-only stripping (`trim_end_matches(['\r', '\n'])`) to preserve payload fidelity.

## Documentation Drift: `docs/PROTOCOL.md` vs Implementation

### Critical mismatches

1. Parser API and result variants are outdated
- Doc claims `ParseResult` includes `MalformedJson` and only three variants (`docs/PROTOCOL.md:113-121`, `1577-1583`).
- Actual code has five variants: `Ok`, `MissingType`, `UnknownType`, `InvalidPayload`, `ParseError` (`src/protocol/io.rs:67-92`).

2. Referenced API names do not match current implementation
- Doc references `write_message()`, `read_message()`, `Message::parse(...)` usage (`docs/PROTOCOL.md:71`, `81`, `103-109`).
- Actual API centers on `serialize_message`, `JsonlReader::next_message*`, `parse_message*` (`src/protocol/io.rs`, `src/executor/runner.rs:494-513`).

3. Message count is stale by large margin
- Doc total says `59` (`docs/PROTOCOL.md:1666-1688`).
- Current `Message` enum has ~126 variants (counted from `src/protocol/message.rs`).

4. Stdin command documentation is incomplete
- Doc section highlights only `run/show/hide/setFilter` (`docs/PROTOCOL.md:134-170`).
- Actual `ExternalCommand` includes many additional commands (`openNotes`, `openAi`, `showGrid`, `hideGrid`, `simulateAiKey`, `captureWindow`, `setAiInput`, `executeFallback`, `showShortcutRecorder`, etc.) in `src/stdin_commands.rs:48-149` and dispatched in `src/main.rs:3583-3668`.

5. Module responsibility table is stale
- Doc says `message.rs` includes `ParseResult` and `io.rs` exposes `write_message/read_message` (`docs/PROTOCOL.md:69-81`).
- Actual `ParseResult` is in `io.rs`, and streaming uses `JsonlReader`.

### Additional drift inside Rust module docs

- `src/protocol/mod.rs` still advertises "59+ variants" and references message families not aligned with current implementation comments (`src/protocol/mod.rs:42-46`).

## Suggested Remediation Plan

### Phase 1 (Safety and reliability)

1. Add hard line-size limits for protocol/stdin readers.
2. Remove blocking `reader_response_tx.send` on hot paths or wrap with timeout strategy.
3. Add structured handling for dropped UI responses (not log-only).
4. Harden unknown-type classification to avoid serde error-string dependency.

### Phase 2 (Protocol completeness)

1. Implement runtime hello/helloAck negotiation.
2. Add deterministic error responses for parsed-but-unimplemented request messages with `requestId`.
3. Publish and enforce a supported-message matrix in code/docs.

### Phase 3 (Documentation parity)

1. Rewrite `docs/PROTOCOL.md` API examples to current function names and parse model.
2. Regenerate message counts/categories from `Message` enum.
3. Expand stdin command docs to reflect `ExternalCommand` reality.
4. Align `src/protocol/mod.rs` module-level docs with current architecture.

## Test Gaps to Add

1. Oversized protocol line: parser should reject, emit issue, and recover for next valid line.
2. Oversized external stdin command line: reject safely and continue loop.
3. Backpressure scenario: script not reading stdin while sending stdout requests should not deadlock reader thread.
4. UI submit channel full: verify deterministic protocol behavior (either retries, explicit drop response, or timeout error).
5. Handshake integration: `hello` from script should produce `helloAck` and capability state.
6. Unsupported request message (with `requestId`): verify app returns structured `setError`/error response, not only local toast.

## Notes on Buffer Overflow Risk

- No traditional memory-unsafe buffer overflow pattern was identified in reviewed Rust protocol code.
- Primary risk class is memory amplification (large-line allocation growth), not unsafe write overflow.
