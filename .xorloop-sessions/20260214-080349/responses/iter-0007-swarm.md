All 4 swarm workers confirmed complete. Recap:

| Worker | Target Files | Result |
|--------|-------------|--------|
| `dedup-positioning-bounds` | `positioning.rs` | Deduplicated with `MouseDisplayPlacement` enum + shared pipeline |
| `dedup-vibrancy-config` | `secondary_window_config.rs` | Extracted `configure_window_vibrancy_common` |
| `dedup-screenshot-encode` | `screenshots_window_open.rs` | Extracted `capture_and_encode_png` helper |
| `name-appkit-constants` | `app_window_management.rs`, `vibrancy_swizzle_materials.rs`, `secondary_window_config.rs` | Named constants replacing raw literals |

Build passes (`cargo check` + `cargo clippy`). All changes are unstaged in the working tree.
