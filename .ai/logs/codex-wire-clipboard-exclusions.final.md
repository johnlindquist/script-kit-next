Implemented the clipboard exclusion wiring in the monitor path so excluded password-manager sources are not recorded.

**Changed files**
- `src/clipboard_history/monitor.rs:204`
- `src/clipboard_history/monitor.rs:325`
- `src/clipboard_history/monitor.rs:494`
- `src/clipboard_history/mod.rs:51`

**What changed**
1. In `capture_clipboard_content`, I now read the source app bundle ID via `crate::frontmost_app_tracker::get_last_real_app_bundle_id()`.
2. Before any `add_entry(...)` call, I gate capture with `should_exclude_clipboard(...)`; when matched, the function returns early and skips recording entirely.
3. Added `should_skip_clipboard_capture(...)` helper to keep this decision logic centralized.
4. Added unit tests for excluded and non-excluded/missing bundle ID scenarios.
5. Removed `#[allow(unused_imports)]` on the exclusions re-export in `src/clipboard_history/mod.rs` since it is now used.

**Verification run**
- `cargo check --lib`
  - Passed once earlier in this session, then later failed due unrelated concurrent edits outside scope (`BuiltInFeature::Quicklinks` mismatch in builtins).
- `cargo clippy --lib --tests -- -D warnings`
  - Failed due many unrelated existing/concurrent errors outside `src/clipboard_history/**` (duplicate test module paths, unresolved symbols in action tests, etc.).
- `cargo test --lib clipboard_history::monitor::tests::test_should_skip_clipboard_capture_returns_true_when_source_bundle_id_is_excluded -- --exact`
  - Failed to build due unrelated concurrent errors outside this scope.

**How to test (when tree is stable)**
1. `cargo check --lib`
2. `cargo clippy --lib --tests -- -D warnings`
3. `cargo test --lib clipboard_history::monitor::tests::test_should_skip_clipboard_capture_returns_true_when_source_bundle_id_is_excluded -- --exact`
4. `cargo test --lib clipboard_history::monitor::tests::test_should_skip_clipboard_capture_returns_false_when_source_bundle_id_is_not_excluded_or_missing -- --exact`

**Risks / known gaps**
- Source app detection currently depends on `frontmost_app_tracker`; if it has no bundle ID at capture time, capture proceeds (no exclusion).
- Frontmost app tracking is a proxy for clipboard source and may be stale in edge timing cases.

Commits made: none.