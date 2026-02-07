Implemented animation/transition improvements in these files:

- `src/window_resize.rs:26`
- `src/actions/window.rs:768`
- `src/components/alias_input.rs:44`
- `src/components/shortcut_recorder.rs:42`

What changed:

- `src/window_resize.rs:396` now animates main window height changes (instead of hard jumps), adds resize gating via `should_animate_resize`, and logs `ANIM_RESIZE_START/END` with `correlation_id`.
- `src/actions/window.rs:768` and `src/actions/window.rs:926` now use animated `setFrame(... animate:true)` for actions window resizing.
- `src/components/alias_input.rs:56` adds overlay enter transition (backdrop fade + modal slide/fade), with per-frame tick scheduling and tests at `src/components/alias_input.rs:750`.
- `src/components/shortcut_recorder.rs:54` adds the same overlay enter transition pattern, with tests at `src/components/shortcut_recorder.rs:971`.

How to test:

1. Compile gate:
```bash
cargo check
```
2. Targeted animation tests (once shared tree compiles cleanly):
```bash
cargo test test_compute_overlay_appear_style
cargo test test_should_animate_resize_true_when_height_changes
cargo test test_should_animate_resize_false_when_height_is_effectively_unchanged
```
3. Stdin protocol smoke run:
```bash
timeout 8 bash -lc "printf '%s\n' '{\"type\":\"show\"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui >/tmp/impl-animations-stdin-show.log 2>&1"
tail -n 80 /tmp/impl-animations-stdin-show.log
```
Expected log signals include parsed/processed `show` command (`stdin_command_parsed`, `Processing external command type=show`).

Risks / known gaps:

- Current shared branch is unstable from parallel edits: latest `cargo check` fails in `src/app_actions.rs` (unrelated to these files), so full-project verification is currently blocked.
- Overlay work is enter-animation only; exit transitions are still hard-close.
- Audit items for full AppView crossfade and list diff transitions were not implemented in this slice.

Commits made: none.