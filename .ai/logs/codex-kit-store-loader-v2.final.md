**Changed Files**
- `src/kit_store/discover.rs`
  - Added `discover_installed_kits() -> Vec<PathBuf>` to read `~/.scriptkit/kits` and return direct subdirectories.
  - Added `kit_scripts_dir(kit_path: &Path) -> PathBuf` (`kit_path/scripts`).
  - Added `kit_scriptlets_dir(kit_path: &Path) -> PathBuf` (`kit_path/scriptlets`).
  - Added unit tests for missing root, directory filtering, and both path helpers.
- `src/kit_store/mod.rs`
  - Added `pub mod discover;`.

**How To Test**
1. Run scoped tests:
   - `cargo test --lib kit_store::discover::tests`
2. (Optional compile check)  
   - `cargo check --lib`

Verification note: both commands are currently blocked by unrelated workspace errors outside this task (e.g. duplicate `mod tests` declarations and `clipboard_history` enum/struct updates in other files). I did not modify those files.

**Risks / Known Gaps**
- New unit tests were added but could not be executed to completion due unrelated parallel changes in the repo.
- `discover_installed_kits()` is intentionally best-effort and returns an empty list if home directory or `~/.scriptkit/kits` cannot be read.

Commits made: none.