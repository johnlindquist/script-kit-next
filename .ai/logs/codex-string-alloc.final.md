Implemented the string-allocation pass and wrote the requested report at `plans/string-allocation-optimization.md`.

**Changed Files**
- `src/scriptlets.rs:143`
  - `resolve_scriptlet_icon` now returns `Cow<'a, str>` to avoid unnecessary icon `String` clones.
  - Removed transient `trimmed.to_string()` allocation in `extract_named_inputs`.
  - `get_actions_file_path` now uses `with_extension("actions.md")` (no lossy stem conversion).
  - Removed `String` clones when consuming parsed codefence data in scriptlet parsing paths.
- `src/scriptlet_tests.rs:2619`
  - Added `test_get_actions_file_path_preserves_non_utf8_stem_bytes` (unix-only regression test).
- `src/scripts/scriptlet_loader.rs:55`
  - Delayed language allocation in `extract_code_block` until return, avoiding early allocations on skipped fences.
- `src/extension_types.rs:419`
  - Removed transient `trimmed.to_string()` allocation in placeholder dedupe check.
- `plans/string-allocation-optimization.md:1`
  - Full optimization report with implemented changes and deferred follow-ups.

**Verification Run**
- `cargo check` ✅ pass
- `cargo build` ✅ pass
- Stdin protocol smoke launch:
  - `printf '{"type":"show"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui` (with timeout) ✅ command parsed and processed (`Parsed command: Show { ... }`)
- `cargo clippy --all-targets -- -D warnings` ❌ fails due unrelated pre-existing issues (e.g. `src/window_control_enhanced/*` unused/dead code from other in-flight edits)
- `cargo test` ❌ blocked by unrelated pre-existing compile failure: `src/ai/window.rs:8549` (`AiApp::message_body_content` missing)

**How To Test**
1. `cargo check`
2. `cargo build`
3. `timeout 8 bash -lc 'printf "{\"type\":\"show\"}\n" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
4. `cargo clippy --all-targets -- -D warnings` (currently expected to fail due unrelated tree state)
5. `cargo test` (currently expected to fail due unrelated tree state)

**Risks / Known Gaps**
- Full `clippy`/`test` gate is currently blocked by unrelated parallel-agent changes outside this task’s files.
- The new non-UTF8 behavior path has targeted regression coverage, but I could not execute that single test independently because test compilation currently fails on unrelated code.

**Commits**
- No commits were made.