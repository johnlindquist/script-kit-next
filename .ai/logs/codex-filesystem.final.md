Implemented and documented the filesystem crate audit.

**Changed Files**
- `src/setup.rs:790`  
  Added `shellexpand::full(&sk_path)` in `get_kit_path()` so `SK_PATH` now expands env vars and `~` (falls back to `tilde` if full expansion fails).
- `src/setup.rs:2131`  
  Added `test_get_kit_path_with_env_var_expansion`.
- `crates-audits/filesystem.md`  
  Added full audit report for `notify`, `dirs`, `shellexpand`, `glob`, `filetime`, `tempfile`, `which` with concrete callsites and recommendations.

**How To Test**
1. `cargo check`
2. `cargo test test_get_kit_path_with_env_var_expansion -- --nocapture`

**Validation Run Results**
- `cargo check` passed.
- `cargo test test_get_kit_path_with_env_var_expansion -- --nocapture` is currently blocked by an unrelated concurrent-tree compile error:
  - `error[E0425]: cannot find function find_next_occurrence_utc_in_timezone`
  - `src/scheduler.rs:647`

**Risks / Known Gaps**
- `notify` is currently using macOS FSEvents (default), not `macos_kqueue`.
- `dirs` is mostly limited to `home_dir()`; platform-specific dir APIs are not broadly used.
- `shellexpand` env expansion is now fixed for `SK_PATH`, but many other callsites are still tilde-only.
- Some temp-file flows intentionally persist files (`.keep()`) or use manual temp paths without cleanup policy.

**Commits**
- None made.