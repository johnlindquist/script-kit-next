Implemented the scoped multi-display stub in `window_control`.

Changed files:
- `src/window_control/part_000.rs:214`  
  Added `TilePosition::NextDisplay` and `TilePosition::PreviousDisplay`.
- `src/window_control/part_003.rs:195`  
  Updated `calculate_tile_bounds` match so `NextDisplay`/`PreviousDisplay` currently return `*display` (same stub behavior as requested).
- `src/window_control/part_003.rs:291`  
  Added `test_calculate_tile_bounds_display_navigation_stubs_return_display`.

Verification run:
- `cargo test --lib window_control::part_003::tests::test_calculate_tile_bounds_display_navigation_stubs_return_display`  
  Failed due unrelated branch errors in other modules (`protocol/actions/clipboard_history`), not in `window_control`.
- `cargo check --lib`  
  Failed due unrelated branch errors in `clipboard_history` (missing fields/non-exhaustive matches).

How to test (once shared-branch compile blockers are resolved):
1. `cargo test --lib window_control::part_003::tests::test_calculate_tile_bounds_display_navigation_stubs_return_display`
2. `cargo test --lib window_control::part_003::tests::test_calculate_tile_bounds_fullscreen`

Risks / known gaps:
- `NextDisplay` and `PreviousDisplay` are stubs only; no actual display enumeration/routing is implemented yet.
- Commit includes unrelated `.ai` log file due pre-staged shared-tree state and policy-blocked history cleanup.

Commits:
- `2db8f20` `feat(window_control): add display navigation tile positions`