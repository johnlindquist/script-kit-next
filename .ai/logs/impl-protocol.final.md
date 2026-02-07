Implemented protocol robustness hardening for malformed/oversized JSONL and stdin command lines.

**Changed Files**
- `src/protocol/io.rs:16`
- `src/protocol/io.rs:254`
- `src/protocol/io.rs:327`
- `src/protocol/io.rs:390`
- `src/protocol/io.rs:735`
- `src/stdin_commands.rs:43`
- `src/stdin_commands.rs:52`
- `src/stdin_commands.rs:286`
- `src/stdin_commands.rs:414`

**What Changed**
- Added bounded line-size handling for protocol JSONL reads (`64 KiB` max) with drain-and-recover behavior for oversized lines.
- Added explicit `ParseIssueKind::LineTooLong` and graceful issue reporting/logging with `correlation_id`.
- Switched JSONL newline handling from broad `trim()` to newline-only trimming (`\r\n`), while still skipping whitespace-only lines.
- Added bounded line-size handling for external stdin commands (`16 KiB` max), including oversize warning logs and continued processing of subsequent commands.
- Added regression tests for oversized-line behavior and recovery in both protocol and stdin readers.

**Validation**
- Passed: `CARGO_TARGET_DIR=target-impl-protocol cargo check --lib`
- Failed (unrelated pre-existing/in-flight workspace errors):
  - `cargo check` fails in `src/render_builtins.rs` (`focus_main_filter` missing).
  - `cargo test oversized_line -- --nocapture` fails in `src/ai/window.rs` (`AiApp::message_body_content` missing).

**How To Test**
1. `cargo check --lib`
2. `cargo test oversized_line -- --nocapture` (currently blocked by unrelated compile errors above)
3. Manual runtime check:
   - Send a very large JSONL line from script stdout and confirm it is skipped and next valid message is processed.
   - Send oversized stdin command JSON and confirm warning log + listener continues.

**Risks / Known Gaps**
- Full test suite and full `cargo check` are currently blocked by unrelated compile breaks in other active-agent areas.
- I did not change unknown-type classification logic in `parse_message_graceful` (still uses serde error text matching).

**Commits**
- No commits made.