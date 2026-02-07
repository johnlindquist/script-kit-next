Implemented the webcam capture task updates in scoped files and verified behavior via tests/builds.

**Changed files**
- `src/app_impl.rs:4174`
  - `webcam_photo_directory()` now prefers `~/Desktop` and falls back to temp (`std::env::temp_dir()`, then `/tmp`).
- `src/app_impl.rs:4358`
  - Webcam action label changed from `Capture` to `Capture Photo`.
- `src/render_prompts/other.rs:335`
  - Webcam footer primary label changed from `Capture` to `Capture Photo`.
- `src/webcam_actions_consistency_tests.rs:75`
  - Added regression checks for `Capture Photo` label in actions + footer.

**Validation run**
- `cargo test webcam_actions_consistency_tests -- --nocapture` ✅
- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ (fails on pre-existing, out-of-scope warnings/errors in `src/actions/dialog_builtin_action_validation_tests_*` and `src/ai/window.rs:5724`)
- `cargo test` ✅ (9558 passed, 0 failed, 35 ignored)
- Runtime stdin protocol sanity:
  - `printf "%s\n" "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui` parsed and processed `Show` command successfully.

**How to test**
1. Build and launch:
   - `cargo build`
   - `printf "%s\n" "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`
2. Open the Webcam prompt in app UI.
3. Trigger capture via footer primary button or actions dialog.
4. Verify:
   - Label reads `Capture Photo`.
   - HUD shows `Photo saved to <path>`.
   - File is written to `~/Desktop` (or temp fallback if Desktop is unavailable).
   - Finder reveal behavior occurs after save.

**Risks / known gaps**
- End-to-end webcam frame capture wasn’t fully automatable in this headless run (camera/device interaction), so runtime verification focused on protocol startup and regression tests.
- Clippy gate is currently blocked by unrelated existing lint debt outside this task’s scope.