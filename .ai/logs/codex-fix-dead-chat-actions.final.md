Removed the dead/stale chat actions-menu code paths and cleaned associated type exports/imports.

Changed files:
- `src/prompts/chat/actions.rs:136`  
  Removed unreachable menu-only methods: `actions_menu_up`, `actions_menu_down`, `actions_menu_select`, `render_actions_menu`, and related dead helpers (`close_actions_menu`, `get_actions`, `get_menu_item_count`, click-selection helpers). Kept `toggle_actions_menu` callback path.
- `src/prompts/chat/prompt.rs:23`  
  Removed obsolete state fields `actions_menu_open` and `actions_menu_selected` plus constructor initialization.
- `src/prompts/chat/types.rs:41`  
  Removed unused `ChatAction` type and impl.
- `src/prompts/chat/mod.rs:56`  
  Removed `ChatAction` from public re-exports (associated import cleanup).
- `src/prompts/chat/render_core.rs:368`  
  Removed stale `.relative()` usage tied to the old in-prompt menu overlay positioning.

How to test:
1. `cargo check`
2. `cargo test --lib prompts::chat`

Verification run:
- `cargo check` passed.
- `cargo test --lib prompts::chat` passed (21 tests, 0 failures).

Risks / known gaps:
- Scoped verification was run per assignment; full workspace `clippy`/full test sweep was not run in this agent task.

Commit made:
- `870bb5d` `fix(chat): remove dead actions menu code`