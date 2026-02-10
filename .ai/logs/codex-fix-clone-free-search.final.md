Implemented the refactor to avoid cloning full app/window structs during match collection and sorting by using indices into the original slices.

Changed files:
- `src/scripts/search/apps.rs`
- `src/scripts/search/windows.rs`

What changed:
- `fuzzy_search_apps` now stores matches as `Vec<(usize, i32)>`, sorts using `apps[index]`, then materializes `AppMatch` at the end.
- `fuzzy_search_windows` now does the same for non-empty queries, and for empty queries sorts `Vec<usize>` then materializes `WindowMatch`.

How to test:
1. `cargo check`
2. `cargo test --lib scripts::search`

Verification run:
- `cargo check`: passed
- `cargo test --lib scripts::search`: failed due unrelated in-progress watcher changes (`compute_backoff` signature mismatch in `src/watcher/part_004_tests/tests_part_001.rs`)

Risks / known gaps:
- Final result types (`AppMatch`/`WindowMatch`) still require cloning when materializing output; eliminating those final clones would require API/type changes outside this task’s scope.

Commit made:
- `80964d6` `refactor(search): use index-based app/window match buffers`