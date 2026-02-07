Changed files:
- `crates-audits/tracing-logging.md`

What I delivered:
- Wrote a full audit report covering `tracing`, `tracing-subscriber`, and `tracing-appender` usage, including:
  - structured logging quality
  - span/perf tracing effectiveness
  - env-filter configuration
  - log rotation status
  - missing hot-path instrumentation
  - log level appropriateness
  - prioritized remediation plan

Verification run:
- Attempted full gate:
  - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- It failed due pre-existing compile issues outside this audit change, including type mismatches in `src/hotkeys.rs` (e.g. around `src/hotkeys.rs:367`, `src/hotkeys.rs:374`, `src/hotkeys.rs:1169`) and existing warnings (`src/theme/types.rs`, `src/watcher.rs`).

How to test:
1. Review the audit: `crates-audits/tracing-logging.md`
2. Re-run verification once branch compile issues are resolved:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Risks / known gaps:
- This task produced an audit document only; no runtime logging behavior was changed.
- Full verification is currently blocked by unrelated branch compile errors.
- The repo is actively modified by parallel agents, so code-line references can drift.

Commits made:
- None.