Implemented the scoped exclusion list change.

**Changed Files**
- `src/clipboard_history/exclusions.rs:1`  
  Added:
  - `EXCLUDED_BUNDLE_IDS: &[&str]` with the 6 password-manager bundle IDs
  - `should_exclude_clipboard(source_bundle_id: &str) -> bool` using prefix matching via `starts_with`
  - Unit tests for exact match, prefix match, and non-match cases
- `src/clipboard_history/mod.rs:30`  
  Added module wiring and public re-exports:
  - `mod exclusions;`
  - `pub use exclusions::{should_exclude_clipboard, EXCLUDED_BUNDLE_IDS};`

**How To Test**
1. `cargo check --lib`
2. `rustc --edition=2021 --test src/clipboard_history/exclusions.rs -o /tmp/exclusions_test && /tmp/exclusions_test`
3. Optional once unrelated test churn settles: `cargo test --lib clipboard_history::exclusions::tests`

**Verification Run**
- `cargo check --lib` passed.
- Isolated unit tests passed: 3/3.
- `cargo test --lib clipboard_history::exclusions::tests` is currently blocked by unrelated concurrent changes elsewhere (duplicate `tests` modules/import errors outside clipboard history).

**Risks / Known Gaps**
- Matching is case-sensitive prefix matching.
- Per scope, this does not wire exclusion into monitor/runtime behavior.

**Commit**
- `8749408` `feat(clipboard_history): add password manager exclusions`