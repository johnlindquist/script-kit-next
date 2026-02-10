Implemented window sixth tiling positions in `src/window_control/**` and committed as:

- `7568e58` `feat(window_control): add sixth tile positions`

Changed files:

- `src/window_control/part_000.rs:163`
  - Added six new `TilePosition` variants:
  - `TopLeftSixth`, `TopCenterSixth`, `TopRightSixth`
  - `BottomLeftSixth`, `BottomCenterSixth`, `BottomRightSixth`
- `src/window_control/part_003.rs:63`
  - Added `calculate_tile_bounds` match arms for all six sixth positions (top/bottom half x left/center/right third).
- `src/window_control/part_003.rs:258`
  - Added tests:
  - `test_calculate_tile_bounds_top_center_sixth`
  - `test_calculate_tile_bounds_bottom_right_sixth`

How to test:

1. `cargo test sixth -- --nocapture`
2. Or individually:
   - `cargo test test_calculate_tile_bounds_top_center_sixth -- --nocapture`
   - `cargo test test_calculate_tile_bounds_bottom_right_sixth -- --nocapture`

Risks / known gaps:

- Protocol/SDK registration for new sixth positions was **not** updated in this change (this commit stayed within `src/window_control/**`).
- Current workspace had unrelated concurrent compile breakages outside this scope (`src/clipboard_history/*`, `src/actions/*`), which blocked end-to-end scoped test execution in the latest run.