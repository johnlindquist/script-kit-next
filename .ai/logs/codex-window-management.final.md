Implemented and documented the window-management audit/fixes, and released all swarm claims/status.

**Changed Files**
- `plans/window-management.md`
- `src/window_state.rs:391`
- `src/window_state_persistence_tests.rs:285`
- `src/app_impl.rs:6385`
- `src/app_actions.rs:304`

**What Changed**
- Added display-aware main-window persistence helper + typed outcome enum:
  - `MainPositionSaveOutcome`
  - `save_main_position_with_display_detection(...)`
- Fixed close lifecycle path to persist main position per-display (with legacy fallback):
  - `src/app_impl.rs:6385`
- Fixed action hide lifecycle path to persist bounds before hide:
  - `src/app_actions.rs:304`
- Added regression tests for:
  - per-display save
  - legacy fallback save
  - suppressed save

**How To Test**
1. Build/check:
```bash
cargo check
cargo build
```
2. Runtime show/hide smoke via stdin protocol:
```bash
timeout 8 bash -lc 'cat <<EOF | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui > /tmp/window-mgmt-show-hide.log 2>&1
{"type":"show"}
{"type":"hide"}
EOF'
grep -iE "show|hide|restoring|saving position|Saved main position" /tmp/window-mgmt-show-hide.log | tail -n 40
```
3. Full gates (currently blocked by pre-existing unrelated issues):
```bash
cargo clippy --all-targets -- -D warnings
cargo test
```

**Risks / Known Gaps**
- `src/window_resize.rs` still does not clamp resized frames to visible bounds after topology changes.
- Resize math is still raw frame-delta based with no explicit mixed-DPI normalization.
- `main.rs` stdin show/hide path duplicates helper logic, creating drift risk.
- Stdin hide path in `main.rs` still lacks legacy fallback when display detection fails.
- Full `clippy`/`test` remain blocked by pre-existing branch issues (not introduced by this task), including missing `AiApp::message_body_content` in tests.

**Commits**
- No commits were made.