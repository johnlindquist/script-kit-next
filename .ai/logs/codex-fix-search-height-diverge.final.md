Implemented the fix in `src/actions/dialog/part_04/body_part_02.rs:475`:

- `search_box_height` now uses the same visibility predicate as rendering:
  - `!matches!(self.config.search_position, SearchPosition::Hidden) && !self.hide_search`
- This removes the divergence where height could include search space even when search was not rendered.

**Changed files**
- `src/actions/dialog/part_04/body_part_02.rs`

**How to test**
1. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
2. Manual UI validation:
   - Open actions dialog with `SearchPosition::Hidden` and `hide_search = false`; confirm no extra vertical space is reserved for search.
   - Open with visible search (`Top`/`Bottom`, `hide_search = false`); confirm search height is reserved and layout is stable.

**Verification result**
- `cargo check` failed due unrelated existing workspace errors (not in this file), including `clipboard_history` enum exhaustiveness and missing fields in `ClipboardEntry` initializers.
- `cargo clippy --all-targets -- -D warnings` failed due unrelated existing workspace issues (duplicate test module paths, many unrelated unused imports/missing symbols, plus the same clipboard_history errors).

**Risks / known gaps**
- Full workspace verification is currently blocked by unrelated in-flight changes, so this fix is code-complete but not validated in a clean green build yet.