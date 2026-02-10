Implemented the section-header count fix so it matches rendered header behavior.

Changed files:
- `src/actions/window/part_01.rs:31`
  - `count_section_headers()` now tracks only the previous non-`None` section (`Option<&str>`), so unsectioned rows do not reset section runs.
- `src/actions/window/part_01.rs:508`
  - Added regression tests:
  - `test_count_section_headers_does_not_reset_on_unsectioned_rows`
  - `test_count_section_headers_counts_new_section_after_unsectioned_row`

How to test:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. Optional focused test: `cargo test window_lifecycle_tests::test_count_section_headers -- --nocapture`

Verification run results:
- `cargo check`: failed due existing unrelated workspace errors (not in this file), e.g. missing `ClipboardEntry` fields in `src/clipboard_history/*` and non-exhaustive `ContentType` matches.
- `cargo clippy --all-targets -- -D warnings`: failed due existing unrelated workspace issues, including duplicate test module files and many pre-existing warnings escalated to errors.
- Focused test command also could not complete for the same existing workspace compile issues.

Risks / known gaps:
- I could not get a green verification run because of pre-existing repository-wide build/clippy failures outside `src/actions/window/part_01.rs`.
- There are existing tests elsewhere that may encode old counting expectations; those were out of scope for this task and were not modified.