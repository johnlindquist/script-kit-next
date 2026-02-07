Implemented the scoped security fixes from `plans/security-audit.md` for input sanitization, path traversal prevention, and insecure deserialization.

**Changed files**
- `src/stdin_commands.rs:45`  
  - Added bounded stdin command size handling and capture path policy types/helpers.
  - Added `CaptureWindow` path validator (`validate_capture_window_output_path`) with allowlisted roots, `.png` enforcement, path normalization, and symlink-component rejection.
  - Hardened deserialization with `#[serde(..., deny_unknown_fields)]` at `src/stdin_commands.rs:309`.
  - Added regression tests for unknown fields and capture path policy at `src/stdin_commands.rs:754` and `src/stdin_commands.rs:939`.
- `src/main.rs:3635`  
  - `captureWindow` now validates output path before writing, creates parent dirs safely, and emits structured rejection logs (`stdin_capture_window_path_rejected`).
- `src/utils/applescript.rs:1` (new file)  
  - Added shared `escape_applescript_string` helper + unit tests.
- `src/utils/mod.rs:9`  
  - Wired and re-exported `escape_applescript_string`.
- `src/app_impl.rs:5664`  
  - `open_in_terminal` now escapes path and uses `quoted form of` in AppleScript.
- `src/app_impl.rs:5786`  
  - `move_to_trash` now escapes AppleScript path input.
- `src/file_search.rs:912` and `src/file_search.rs:942`  
  - Replaced ad-hoc escaping with shared helper.
- `src/app_actions.rs:106`  
  - Uses shared AppleScript escape helper for trash action.

**Verification run**
- `cargo check` ✅ passed.
- Runtime stdin smoke test ✅ confirmed policy enforcement:  
  - Command with `/tmp/security-test.png` was rejected with `event_type=stdin_capture_window_path_rejected` and allowed-roots reason.
- Focused tests/clippy could not complete due unrelated existing tree failures:
  - `src/window_control.rs:1642` (`as_ptr` missing)
  - `src/menu_executor_tests.rs:88` (`OwnedAxElement` unresolved)
  - `src/ai/window.rs:8729` (`AiApp::message_body_content` missing)
  - plus unrelated clippy/test issues in `src/camera.rs`, `src/actions/dialog.rs`, `src/prompts/div.rs`.

**How to test**
1. `cargo check`
2. Rejection case (should log path rejection):
   - `printf "%s\n" "{\"type\":\"captureWindow\",\"title\":\"Script Kit\",\"path\":\"/tmp/security-test.png\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. Unknown-field deserialization rejection (should log parse failure):
   - `printf "%s\n" "{\"type\":\"show\",\"unexpected\":\"field\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
4. Allowed-root case (policy should pass path validation):
   - `mkdir -p .test-screenshots`
   - `printf "%s\n" "{\"type\":\"captureWindow\",\"title\":\"Script Kit\",\"path\":\".test-screenshots/shot.png\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Risks / known gaps**
- Stdin auth/capability gating from SA-MED-003 is not included in this scoped patch.
- Predictable temp-file hardening from SA-LOW-006 is not included in this scoped patch.
- There is still a potential TOCTOU window between path validation and file write.
- Full `cargo test` / `cargo clippy --all-targets -- -D warnings` is currently blocked by unrelated pre-existing compile issues listed above.

**Commits**
- None made.