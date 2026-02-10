Implemented: `Generate Script with AI` now shares the same query-driven execution path as Shift+Tab, and fallback execution now forwards input text into built-in execution without immediately closing the window.

Changed files:
- `src/app_impl/prompt_ai.rs:641`
  - Added shared helper `dispatch_ai_script_generation_from_query(...)` (trim/empty guard + open script-generation chat from query).
- `src/app_execute/builtin_execution.rs:125`
  - Added `execute_builtin_with_query(...)`; `execute_builtin(...)` now delegates to it.
- `src/app_execute/builtin_execution.rs:1036`
  - `AiCommandType::GenerateScript` now routes through `dispatch_ai_script_generation_from_query(...)` using forwarded query when provided.
- `src/app_impl/startup.rs:734`
  - Shift+Tab now calls `dispatch_ai_script_generation_from_query(...)`.
- `src/app_impl/startup_new_tab.rs:185`
  - Same Shift+Tab routing update (kept in sync with interceptor reference file).
- `src/app_impl/selection_fallback.rs:14`
  - Added `fallback_keeps_window_open(...)` and treated `builtin-generate-script-with-ai` as stay-open.
- `src/app_impl/selection_fallback.rs:399`
  - `FallbackResult::ExecuteBuiltin` now calls `execute_builtin_with_query(&entry, Some(input), ...)`.
- `src/main_sections/fallbacks.rs:163`
  - Same fallback query-forwarding update for legacy fallback execution path.
- `src/app_impl/tests.rs:52`
  - Updated routing assertions to shared helper; added assertion that built-in Generate Script also routes to shared helper.
- `src/app_impl/selection_fallback.rs:452`
  - Added tests for keep-window-open behavior for generate-script fallback.

Verification run:
- `cargo check -p script-kit-gpui` ✅ passes.
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` ❌ blocked by unrelated existing repo issues (for example duplicate test module files and many pre-existing lint/test errors outside this scope).
- Scoped test attempts (`cargo test -p script-kit-gpui app_impl_state_sync_tests::` and `cargo test -p script-kit-gpui --lib app_impl_state_sync_tests::`) ❌ blocked by unrelated existing compile failures in other test modules.

How to test manually:
1. Launch app and type prompt text in ScriptList (for example: `build a weather script`).
2. Press Shift+Tab.
3. Confirm script-generation chat opens using that prompt.
4. Return to ScriptList, type prompt text again, select fallback `Generate Script with AI`, press Enter.
5. Confirm behavior matches Shift+Tab (same flow, query preserved, window not immediately closed).

Risks / known gaps:
- Full clippy/test gate is currently not runnable in this shared tree due unrelated parallel-agent breakages; only scoped compile verification (`cargo check -p script-kit-gpui`) completed successfully.