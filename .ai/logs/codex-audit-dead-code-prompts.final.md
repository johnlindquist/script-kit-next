**Findings (src/prompts audit)**

1. **Stale chat actions-menu implementation is mostly unreachable**
- `src/prompts/chat/render_core.rs:133` says actions navigation is handled by parent `ActionsDialog`.
- `src/prompts/chat/actions.rs:164`, `src/prompts/chat/actions.rs:171`, `src/prompts/chat/actions.rs:179`, `src/prompts/chat/actions.rs:245` (`actions_menu_up/down/select`, `render_actions_menu`) have no call sites outside their own definitions/callback internals.
- `src/prompts/chat/prompt.rs:25` (`actions_menu_open`) is initialized but never read.
- `src/prompts/chat/types.rs:43` (`ChatAction`) appears tied to this stale overlay path.

2. **Confirmed dead private helper in DropPrompt**
- `src/prompts/drop.rs:105` `handle_drop` is marked `#[allow(dead_code)]` and has no call sites.

3. **Confirmed dead markdown helper kept only by lint suppression**
- `src/prompts/markdown/inline_render.rs:163` `render_code_block` is marked `#[allow(dead_code)]` and has no call sites (new path uses `build_code_block_element`).

4. **Unused + placeholder context resolver**
- `src/prompts/context.rs:180` `resolve_mention` has no call sites.
- It also returns `None` for clipboard/selection/terminal branches (`src/prompts/context.rs:182`, `src/prompts/context.rs:187`, `src/prompts/context.rs:195`), indicating partial placeholder behavior.

5. **Unused TemplatePrompt API**
- `src/prompts/template/prompt.rs:302` `filled_template` has no call sites.

6. **Likely stale PathPrompt callback API**
- `src/prompts/path/prompt.rs:50`, `src/prompts/path/prompt.rs:56`, `src/prompts/path/prompt.rs:61` (`with_show_actions`, `set_show_actions`, `with_close_actions`) have no repo call sites; event-emitter path appears to be the active mechanism.

7. **Lint suppressions masking dead code/imports**
- `src/prompts/mod.rs:16` has `#![allow(dead_code)]`.
- `src/prompts/prelude.rs:2` has `#![allow(unused_imports)]`.
- Multiple re-exports in `src/prompts/mod.rs:42` onward are guarded by `#[allow(unused_imports)]`.

8. **Stale module docs**
- `src/prompts/mod.rs:10` through `src/prompts/mod.rs:14` still label several prompts as “(skeleton)” although those modules now contain substantial implementations/tests.

**TODO/FIXME/HACK scan**
- No `TODO`, `FIXME`, or `HACK` markers found in `src/prompts/**/*.rs` via grep scan.

**Changed files**
- None (audit/report only).

**Verification run**
- `cargo test prompts::`
- Result: 105 passed, 1 failed.
- Failure: `prompts::markdown::tests::nested_lists_preserve_parent_child_structure` in `src/prompts/markdown/tests.rs:35` (existing behavior mismatch in expected ordered-list parent text).

**Risks / known gaps**
- Dead-code findings are based on whole-repo symbol search plus call-path review; module-level `allow(...)` suppressions reduce compiler visibility, so additional dead code may still be hidden.