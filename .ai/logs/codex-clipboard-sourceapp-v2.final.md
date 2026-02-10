Changed files:
- `src/clipboard_history/types.rs:98`
  - Added `source_app_name: Option<String>` and `source_app_bundle_id: Option<String>` to `ClipboardEntry`.
  - Kept derives as `#[derive(Debug, Clone)]` (unchanged).
- `src/clipboard_history/types.rs:425`
  - Updated in-file test `ClipboardEntry` literals to include the new optional fields.

How to test:
1. `rustfmt --check src/clipboard_history/types.rs`
2. `cargo test clipboard_history::types`

Verification run:
- `rustfmt --check src/clipboard_history/types.rs` passed.
- `cargo test clipboard_history::types` failed due existing workspace issues outside this task (duplicate `tests` module files and multiple unrelated compile errors), plus expected downstream `ClipboardEntry` initializer updates needed in other files not in this scope.

Risks / known gaps:
- This struct-only change introduces compile breakages in other modules that construct `ClipboardEntry` until they add the two new fields.
- No DB/schema/monitor/UI changes were made, per scope.