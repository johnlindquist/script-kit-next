**Changed files**
- `src/scriptlets.rs`
- `src/extension_types.rs`
- `src/scripts/scriptlet_loader.rs`
- `src/scriptlet_tests.rs`

**What I changed**
- Switched `resolve_scriptlet_icon` to return `Cow<str>` and borrow existing icon strings instead of cloning.
- Replaced duplicate-input checks that did `contains(&trimmed.to_string())` with borrowed comparisons (`iter().any(...)`) in:
  - `src/scriptlets.rs`
  - `src/extension_types.rs`
- Changed companion actions path generation to `md_path.with_extension("actions.md")` in `src/scriptlets.rs` (preserves non-UTF8 stems).
- Removed unnecessary clones when consuming `codefence_result.code` by moving `language` and `content` in `src/scriptlets.rs`.
- Delayed language allocation in `extract_code_block` in `src/scripts/scriptlet_loader.rs` by keeping it as `&str` until return.
- Added regression test `test_get_actions_file_path_preserves_non_utf8_stem_bytes` in `src/scriptlet_tests.rs` (`#[cfg(unix)]`).

**Verification run**
- `cargo check` ✅ passed.
- `cargo clippy --all-targets -- -D warnings` ❌ failed due unrelated in-flight workspace errors (e.g. missing `src/prompts/prelude.rs`, config/hotkey mismatches in other files).
- `cargo test` ❌ failed due unrelated in-flight workspace errors (watcher/hotkeys/builtins compile issues outside this change set).

**How to test**
1. Run `cargo check`.
2. When the shared tree is stabilized, run:
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
3. Specifically validate the new regression test with:
   - `cargo test test_get_actions_file_path_preserves_non_utf8_stem_bytes`

**Risks / known gaps**
- Full clippy/test verification is currently blocked by unrelated parallel-agent changes in other modules.
- The non-UTF8 regression test is Unix-only by design (`#[cfg(unix)]`).

**Commits**
- No commit was created (full verification gate is currently red due unrelated shared-tree failures).