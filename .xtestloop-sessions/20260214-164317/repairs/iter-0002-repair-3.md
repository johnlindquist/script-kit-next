Now I have a clear picture. All these tests use `set_var`/`remove_var` on the same env var and need to be serialized. The simplest fix: add a static mutex and acquire it in each test.

Let me now bundle the clipboard_history module and the setup test to send to Oracle for analysis, as the skill requested.

Actually, wait — re-reading the request more carefully. The gate output shows exactly **1 failure**: `setup::tests::test_complete_setup_structure`. The task says to produce "FIX-ONLY swarm tasks to make the gate pass." The clipboard_history target is the xtestloop's reason for the session, but the actual gate failure is in `setup`.

Let me produce the swarm task list based on the actual failure:

---

## SWARM TASK LIST

### Task 1: fix-setup-test-race
**Scope:** `src/setup/mod.rs`
**Task:**

The single gate failure is `setup::tests::test_complete_setup_structure` panicking with `kit/ directory should exist`. This is an env var race condition — 7 tests in `src/setup/mod.rs` call `std::env::set_var(SK_PATH_ENV, ...)` / `std::env::remove_var(SK_PATH_ENV)` concurrently. When tests run in parallel, one test's `remove_var` or `set_var` can clobber another test's env state between set and use.

**Fix:** Add a static `Mutex<()>` (following the existing pattern at `clipboard_history/mod.rs:109` and `frontmost_app_tracker/mod.rs:531`) and acquire the lock at the top of every test that touches `SK_PATH_ENV`:

1. In `src/setup/mod.rs` inside the `#[cfg(test)] mod tests` block (around line 1430), add:
   ```rust
   use std::sync::Mutex;
   static SK_PATH_ENV_LOCK: Mutex<()> = Mutex::new(());
   ```

2. At the start of each of these 7 tests, add `let _guard = SK_PATH_ENV_LOCK.lock().unwrap();`:
   - `test_kit_directory_uses_kit_subdirectory` (line 1437)
   - `test_sample_files_in_kit_subdirectory` (line 1477)
   - `test_get_kit_path_sk_path_override` (around line 1528)
   - `test_get_kit_path_tilde_expansion` (around line 1537)
   - `test_get_kit_path_env_var_expansion` (around line 1548)
   - `test_complete_setup_structure` (line 1580)
   - `test_agents_md_paths_match_setup` (line 1792)

No behavior changes. No new features. Just serializing tests that share mutable global state.

---

NEXT_AREA: After the gate is green, add characterization tests for `src/clipboard_history` (the original target — highest churn, 70 unwrap/expect, no tests.rs yet).
