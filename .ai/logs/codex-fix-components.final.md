Implemented the component-scope compile fixes and cleared all `src/components` errors from `cargo check` output.

**Changed files**
- `src/components/alias_input/render.rs`
  - Fixed module path import: `super::super::types`
  - Restored GPUI extension traits via `prelude::*` import
- `src/components/form_fields/text_field/render.rs`
  - Fixed helper import path to `super::super::helpers`
- `src/components/form_fields/text_area/mod.rs`
  - Removed misplaced `ScrollableElement` import
- `src/components/form_fields/text_area/render.rs`
  - Added `gpui_component::scroll::ScrollableElement` where used

**Verification run**
- `cargo check 2>&1 | grep src/components`  
  - Result: no matches (zero component errors)
- `cargo check`  
  - Fails outside component scope (`src/prompts/*` visibility/private field errors)
- `cargo clippy --all-targets -- -D warnings`  
  - Fails outside component scope (`src/actions/*`, `src/prompts/*`)
- `cargo test`  
  - Fails outside component scope (`src/actions/*`, `src/prompts/*`)

**How to test**
1. Run: `cargo check 2>&1 | grep src/components`
2. Confirm it prints nothing.

**Risks / known gaps**
- Workspace-wide check/clippy/test still fail due unrelated in-flight changes outside `src/components`.
- These files are part of the ongoing split and may be concurrently touched by other agents.

**Commit made**
- `37fc9d0` `fix(components): repair split module imports in component renders`