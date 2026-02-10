Implemented the dead-code cleanup in the assigned prompt files.

**Changed files**
- `src/prompts/mod.rs:16`  
  Removed module-level `#![allow(dead_code)]`.
- `src/prompts/prelude.rs:3`  
  Removed `#![allow(unused_imports)]` and trimmed exports to test-only items actually used by `prelude_tests`.
- `src/prompts/drop.rs:103`  
  Removed dead `handle_drop(...)`.
- `src/prompts/markdown/inline_render.rs:152`  
  Removed dead `render_code_block(...)`.
- `src/prompts/context.rs:179`  
  Removed dead `resolve_mention(...)`.
- `src/prompts/template/prompt.rs:301`  
  Removed dead `filled_template(...)`.
- `src/prompts/path/prompt.rs:49`  
  Removed dead builder/mutator methods: `with_show_actions`, `set_show_actions`, `with_close_actions`.

**How to test**
1. `cargo check`
2. `cargo test --lib prompts::prelude_tests::test_prompts_prelude_exports_core_prompt_types`

**Risks / known gaps**
- After removing blanket dead-code allow, `cargo check` still reports existing dead-code warnings in other prompt modules (pre-existing, outside this task’s explicit removals).
- There are also two remaining unused-import warnings outside the assigned scope:
  - `src/prompts/markdown/mod.rs` (`CodeSpan`)
  - `src/prompts/path/mod.rs` (`CloseActionsCallback`)
- `src/prompts/prelude.rs` is now effectively test-only; if any non-test callsite starts importing from `prompts::prelude`, it will need explicit exports restored.