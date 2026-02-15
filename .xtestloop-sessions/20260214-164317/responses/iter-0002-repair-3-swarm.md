All green. Here's the swarm summary:

## Swarm Results

**Worker:** `fix-setup-test-race` - **Completed successfully**

**Commit:** `273c5415` — `fix(setup-tests): serialize SK_PATH env var mutation`

**What changed:** Added a `static SK_PATH_ENV_LOCK: Mutex<()>` in `src/setup/mod.rs` tests and acquired the lock in all 7 tests that mutate `SK_PATH_ENV`. This serializes the env var access and eliminates the race condition.

**Verification gate:**
- `cargo check` — passed
- `cargo clippy --all-targets -- -D warnings` — passed
- `cargo test` — **16 passed, 0 failed** (57 ignored are doc tests / system tests)

The gate is green. The worker also noted that `test_get_kit_path_default` also calls `remove_var(SK_PATH_ENV)` without the mutex (wasn't in the requested 7), so there's a residual theoretical race there — worth noting for a future pass.
