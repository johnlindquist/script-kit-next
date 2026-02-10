Changed files:
- `src/actions/builders/scriptlet.rs:6`
- `src/actions/builders/scriptlet.rs:53`
- `src/actions/builders/scriptlet.rs:268`

What changed:
- Added scriptlet action ID dedup logic so repeated H3 command IDs become unique with a counter suffix (`__2`, `__3`, etc.).
- Added malformed action ID parsing guard and `tracing::warn!` when parsing fails, then skip that malformed action.
- Added unit tests in the same file for:
  1. duplicate command IDs get unique suffixed IDs,
  2. suffix collision handling (`copy` + existing `copy__2`),
  3. malformed empty-command action is dropped.

How to test:
1. `cargo test test_get_scriptlet_defined_actions_`
2. `cargo check`
3. `cargo clippy --all-targets -- -D warnings`

Verification results:
- `cargo test test_get_scriptlet_defined_actions_` did not complete due unrelated branch-wide compile errors (module duplication and unrelated action/clipboard test/module issues).
- `cargo check` failed on unrelated existing errors (e.g. missing `ClipboardEntry` fields and non-exhaustive `ContentType` matches outside this file).
- `cargo clippy --all-targets -- -D warnings` failed on unrelated pre-existing repo-wide warnings/errors and module duplication.

Risks / known gaps:
- The new suffixed IDs (`scriptlet_action:<command>__N`) may require downstream execution parsing updates to map back to the intended command when actions are executed.
