Updated the AI script-generation prompts and import enforcement to match current Script Kit SDK usage and patterns observed in `~/.scriptkit/kit/main/scripts/*.ts`.

**Changed files**
- `src/ai/script_generation.rs:16`  
  Updated `AI_SCRIPT_GENERATION_SYSTEM_PROMPT` to modern conventions (`import "@scriptkit/sdk";`, top-level `await`, `arg/div/editor/notify/md`, `home/writeFile`, metadata comments).
- `src/ai/script_generation.rs:360`  
  Updated `has_kit_import()` to recognize only `@scriptkit/sdk` imports.
- `src/ai/script_generation.rs:390`  
  Updated `enforce_script_kit_conventions()` to insert `import "@scriptkit/sdk";` via constant.
- `src/ai/script_generation.rs:531`  
  Added tests to assert modern import handling and prompt content (and reject legacy `@johnlindquist/kit`).
- `src/app_impl/prompt_ai.rs:29`  
  Updated duplicate script-generation system prompt to match modern SDK/API patterns.
- `src/app_impl/prompt_ai.rs:826`  
  Added prompt test asserting `@scriptkit/sdk` and no legacy import string.

**How to test**
1. `cargo check -p script-kit-gpui --lib`
2. After the shared tree stabilizes, run targeted tests:
   - `cargo test -p script-kit-gpui --lib test_has_kit_import_accepts_scriptkit_sdk_and_rejects_legacy_kit_import`
   - `cargo test -p script-kit-gpui --lib test_ai_script_generation_system_prompt_uses_modern_sdk_conventions`
   - `cargo test -p script-kit-gpui --lib test_ai_script_generation_system_prompt_references_scriptkit_sdk_not_legacy_kit`

**Verification run**
- Passed: `cargo check -p script-kit-gpui --lib`
- Blocked: `cargo test -p script-kit-gpui --lib ...` due unrelated concurrent-tree errors (e.g. duplicate test module files in `src/protocol/message` and `src/designs/core`, plus unrelated `actions` test compile errors).

**Risks / known gaps**
- Unit tests for these changes could not be executed end-to-end because of unrelated compile breaks from other active agent changes in the shared working tree.