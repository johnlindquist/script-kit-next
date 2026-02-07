Implemented the actions-popup UX changes in the claimed scope.

**Changed files**
- `src/actions/builders.rs`
  - Added section grouping for script/scriptlet actions (`Actions`, `Edit`, `Share`, `Destructive`) and ensured destructive actions are appended last in both script and scriptlet builders (`src/actions/builders.rs:363`, `src/actions/builders.rs:549`).
  - Added shortcut hint for `reset_ranking` (`⌃⌘R`) and tightened label/description phrasing to verb-first style.
  - Added section on scriptlet-defined actions (`src/actions/builders.rs:321`).
- `src/actions/dialog.rs`
  - Added destructive detection helper and section-separator helper (`src/actions/dialog.rs:56`, `src/actions/dialog.rs:169`).
  - Rendered visual separators at section boundaries in separator mode and applied destructive red styling to title/icon/row/keycaps (`src/actions/dialog.rs:1492`).
  - Added unit tests for destructive detection + separator behavior (`src/actions/dialog.rs:2090`).
- `src/actions/builders_tests.rs`
  - Added tests for section grouping and destructive-last ordering + shortcut hints (`src/actions/builders_tests.rs:539`).

**How to test**
1. Run targeted tests:
   - `cargo test script_context_actions_ -- --nocapture`
   - `cargo test script_context_destructive_actions_are_last_and_marked -- --nocapture`
2. Manual UI check:
   - Open app, focus any script, open actions popup (`⌘K`).
   - Verify:
     - Actions are grouped (Actions/Edit/Share/Destructive).
     - Destructive actions are at the bottom and styled in warning/red.
     - Shortcut hints are visible beside actions.

**Validation run status**
- Passed:
  - `cargo test script_context_actions_ -- --nocapture`
  - `cargo test script_context_destructive_actions_are_last_and_marked -- --nocapture`
- Currently blocked for full gate by parallel-agent changes outside this scope:
  - `cargo check` fails at `src/prompts/select.rs:376` (`E0282`, missing type annotation).
  - `cargo clippy --all-targets -- -D warnings` fails on unrelated pre-existing warnings/errors in other files.

**Risks / known gaps**
- Full repo-wide gate (`cargo check && cargo clippy --all-targets -- -D warnings && cargo test`) is not green due external in-progress changes outside the claimed scope.
- Destructive styling uses ID/title heuristics; custom third-party actions starting with delete/remove/clear will be styled as destructive by design.