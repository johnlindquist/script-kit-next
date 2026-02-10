Changed files:
- `src/actions/dialog/part_02/part_03.rs`: removed unused `ActionsDialog::dismiss_on_click_outside()` (no call sites) and its stale comment block.

What I checked:
- `rg -n "dismiss_on_click_outside" src/actions -g '*.rs'` -> no remaining references.
- `cargo check` (failed due unrelated existing errors).
- `cargo clippy --all-targets -- -D warnings` (failed due unrelated existing errors).

How to test:
1. Run `cargo check`.
2. Run `cargo clippy --all-targets -- -D warnings`.

Current blockers in this tree (not from this change):
- Duplicate module files: `src/protocol/message/tests.rs` and `src/protocol/message/tests/mod.rs`; `src/designs/core/tests.rs` and `src/designs/core/tests/mod.rs`.
- `clipboard_history` compile errors (missing `source_app_bundle_id`/`source_app_name`, non-exhaustive `ContentType` matches).
- Many `unused import` clippy errors in `src/actions/tests/...` generated test partitions.

Risks / known gaps:
- Full verification is blocked by the existing workspace errors above.
- This change is low risk: it only removes an orphaned method with no call sites.