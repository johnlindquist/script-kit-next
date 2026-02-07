Implemented the HTTP/scheduling audit fixes and wrote the report.

**Changed files**
- `src/ai/providers.rs`: hardened `ureq` setup and request handling.
- `src/scheduler.rs`: improved cron/timezone handling and scheduler behavior with tests.
- `crates-audits/http-scheduling.md`: full audit report with findings, actions, validation, and gaps.

Key code points:
- `src/ai/providers.rs:160` adds `send_json_with_retry(...)` with bounded retries/backoff.
- `src/ai/providers.rs:222` updates `create_agent()` to `https_only(true)`, `http_status_as_error(false)`, and full request-phase timeouts.
- `src/scheduler.rs:134` and `src/scheduler.rs:269` compute next runs with local-time semantics via `find_next_occurrence_utc_in_timezone(...)`.
- `src/scheduler.rs:377` adds timezone-aware UTC conversion helper for cron evaluation.
- `src/scheduler.rs:421`, `src/scheduler.rs:505`, `src/scheduler.rs:684` add cron/english-to-cron/timezone regression tests.

**Validation run**
- `cargo test --lib test_create_agent_disables_status_errors_and_enforces_https` passed.
- `cargo test --lib scheduler::tests` passed (20/20).
- `cargo test --lib ai::providers::tests` passed (42 passed, 3 ignored).
- `cargo check` passed.
- `cargo clippy --lib -- -D warnings` failed due pre-existing unrelated issues in other files (not part of this scope), e.g. `src/theme/types.rs`, `src/watcher.rs`, `src/actions/dialog.rs`, `src/prompts/div.rs`, `src/prompts/markdown.rs`, `src/agents/parser.rs`.

**How to test**
1. Run provider+scheduler tests:
```bash
cargo test --lib ai::providers::tests
cargo test --lib scheduler::tests
```
2. Re-run compile gate:
```bash
cargo check
```
3. Read audit report:
```bash
cat crates-audits/http-scheduling.md
```

**Risks / known gaps**
- Retries do not yet honor `Retry-After`.
- Retries apply to POST calls, which can duplicate billable operations under ambiguous network failures.
- Backoff has no jitter.
- DST edge cases (nonexistent/ambiguous local times) are not yet explicitly regression-tested.
- `english-to-cron` remains heuristic; explicit cron is safer for critical schedules.

Commits made: none.