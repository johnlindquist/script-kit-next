Implemented the accessibility updates from `plans/accessibility-audit.md` in my scoped files and released all swarm claims.

**Changed Files**
- `src/components/button.rs:259`  
  Added runtime focus resolution (`FocusHandle::is_focused`) so visual focus state tracks actual keyboard focus, and kept keyboard activation handling for Enter/Return/Space (`on_key_down` at `src/components/button.rs:433`).
- `src/components/footer_button.rs:188`  
  Added keyboard activation parity (Enter/Return/Space) for footer buttons, with interactive-state checks.
- `src/theme/types.rs:127`  
  Raised default selection/hover opacity values for better non-text contrast and visibility; adjusted unfocused opacity behavior to preserve selected/hover strength (`src/theme/types.rs:1137`).
- `src/theme/theme_tests.rs:140`  
  Added tests to enforce focus-opacity behavior and contrast thresholds for selected/hover states (`src/theme/theme_tests.rs:151`).
- `src/render_script_list.rs:1096`  
  Updated Ask AI hint styling to a non-clickable visual hint (`cursor_default`) to match keyboard-only behavior.
- `src/actions_button_visibility_tests.rs:58`  
  Added regression test ensuring Ask AI hint does not use clickable cursor semantics.

**Verification Run**
- `cargo check` passed.
- Runtime launch via stdin JSON protocol passed (app started and handled stdin command path):
  - `printf '{"type":"show"}\n' > /tmp/skcmd.jsonl && timeout 8 bash -c 'cat /tmp/skcmd.jsonl | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
- Focused test run attempt:
  - `cargo test test_ask_ai_hint_is_non_clickable_visual_hint`
  - blocked by unrelated cross-agent compile errors in `src/camera.rs`, `src/window_control.rs`, `src/menu_executor_tests.rs`, and `src/ai/window.rs`.

**How To Test**
1. `cargo check`
2. `printf '{"type":"show"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. After other agent compile breakages are fixed: `cargo test`

**Risks / Known Gaps**
- Full test suite is currently blocked by unrelated compile failures in other agents’ files.
- Higher default selected/hover opacity may change visuals for users expecting prior lower-contrast defaults.
- If product intent later makes Ask AI hint clickable, current non-clickable cursor semantics will need to be revisited.

Commits made: none.