Implemented the dialog render consistency fix in `src/actions/dialog/part_02/part_03.rs`.

**Changed files**
- `src/actions/dialog/part_02/part_03.rs:1`
  - Added shared alpha/opacity constants + helper functions so search/container styling follows one consistent path.
- `src/actions/dialog/part_02/part_03.rs:133`
  - `get_search_colors()` now derives alpha via shared helpers (no duplicated inline scaling logic).
- `src/actions/dialog/part_02/part_03.rs:167`
  - `get_container_colors()` now uses theme opacity + shared helpers for dialog/background/border alpha, replacing the hardcoded border alpha path.
- `src/actions/dialog/part_02/part_03.rs:197`
  - Added unit tests for opacity scaling/floor behavior consistency.

No additional edits were needed in `src/actions/dialog/part_02/part_01.rs` or `src/actions/dialog/part_02/part_02.rs` for the requested render-token/style consistency checks.

**Validation run**
- `cargo check -p script-kit-gpui`  
  - Fails due unrelated pre-existing errors (not in dialog part_02 files), e.g. missing `ClipboardEntry` fields and non-exhaustive `ContentType` matches in `src/clipboard_history/*`.
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`  
  - Fails due unrelated pre-existing workspace issues (duplicate `mod tests`, widespread unused imports/tests wiring errors, plus same clipboard/content-type failures).
- `cargo test -p script-kit-gpui actions_dialog_opacity_consistency_tests`  
  - Blocked by the same unrelated compile errors before test execution.

**How to test**
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `cargo test -p script-kit-gpui actions_dialog_opacity_consistency_tests`

**Risks / known gaps**
- Full verification is currently blocked by unrelated repo-wide compile/lint failures.
- There is an unrelated existing diff hunk in `src/actions/dialog/part_02/part_03.rs` (outside this consistency work) that I did not revert.

No commits were made.