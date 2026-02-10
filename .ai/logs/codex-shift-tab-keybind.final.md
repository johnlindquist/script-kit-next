Implemented `Shift+Tab` script-generation flow in parallel with existing `Tab` chat flow.

Changed files:
- `src/app_impl/startup.rs:558`  
  - Tab key matcher now explicitly supports both variants (`"tab" | "Tab"`).
  - In ScriptList, `Shift+Tab` now routes to `generate_script_from_ai_prompt(...)`; plain `Tab` still routes to `show_inline_ai_chat(...)`.
- `src/app_impl/startup_new_tab.rs:9`  
  - Mirrored the same Tab/Shift+Tab interceptor behavior in the new-tab startup path.
- `src/app_impl/prompt_ai.rs:29`  
  - Added AI script-generation prompt/instructions for Script Kit best practices.
  - Added generation pipeline helpers (model selection, fenced-code extraction, script-name derivation, backend call).
  - Added `generate_script_from_ai_prompt(...)` (`src/app_impl/prompt_ai.rs:469`) to:
    - use current input as prompt,
    - call AI provider backend,
    - create/write a new script file,
    - open it in the editor,
    - emit structured logs/toasts.
- `src/app_impl/tests.rs:52`  
  - Added tests asserting:
    - Shift+Tab routes to script generation,
    - Tab still routes to inline AI chat,
    - interceptor matches both key variants (`"tab" | "Tab"`).

Verification run:
- Attempted scoped tests:
  - `cargo test --lib app_impl::prompt_ai::tests::test_build_ai_script_generation_user_prompt_includes_description`
  - `cargo test --lib app_impl::tests::app_impl_state_sync_tests::test_shift_tab_routes_to_ai_script_generation_in_script_list_tab_interceptor`
- Result: blocked by unrelated shared-tree compile failures (duplicate module files and unrelated `src/actions/tests/**` errors).
- Patch hygiene check passed:
  - `git diff --check -- src/app_impl/startup.rs src/app_impl/startup_new_tab.rs src/app_impl/prompt_ai.rs src/app_impl/tests.rs`

How to test:
1. Run app with logs:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
2. In main ScriptList input, type a script request.
3. Press `Shift+Tab`.
4. Verify `AI_SCRIPT_GEN` logs appear, a new script is created, and editor opens it.
5. Press `Tab` with input text and verify inline AI chat still opens.

Risks / known gaps:
- Automated Rust verification for this change is currently blocked by unrelated compile issues in the shared tree.
- Input text is cleared before generation starts; on failure, the prompt is not restored automatically.

Commits:
- None made.