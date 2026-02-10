**Findings (Markdown Audit)**
1. `Medium` `src/components/shortcut_recorder/component.rs:199` + `src/components/shortcut_recorder/types.rs:124`  
   `ShortcutRecorder` records a non-modifier key and sets `is_recording = false` even when no modifiers are held. `is_complete()` requires a modifier, so this can leave keyboard users in a dead-end state (cannot save; must clear/cancel manually).  
2. `Resolved` `src/app_shell/keymap.rs:173` + `src/app_shell/keymap.rs:268`  
   Shell key routing previously depended on exact key strings, so alias/case variants (`arrowup`, `DownArrow`, `Esc`, `return`) could miss bindings. I normalized key aliases/case at bind and lookup/route time.

**Code Changes Made**
- Added key normalization in `src/app_shell/keymap.rs` for:
  - `arrowup/uparrow -> up`
  - `arrowdown/downarrow -> down`
  - `arrowleft/leftarrow -> left`
  - `arrowright/rightarrow -> right`
  - `esc -> escape`
  - `return -> enter`
  - case-insensitive matching
- Added regression tests in `src/app_shell/tests.rs` for alias/case variants and normalized lookup behavior.

**Changed Files**
- `src/app_shell/keymap.rs`
- `src/app_shell/tests.rs`

**How To Test**
1. `cargo test app_shell::tests::keymap_tests -- --nocapture`
2. `cargo test shortcut_recorder::tests -- --nocapture`
3. `cargo test --bin script-kit-gpui app_navigation_selection_tests -- --nocapture`

All passed in this run.

**Risks / Known Gaps**
- The modifierless-shortcut dead-end in `ShortcutRecorder` (finding #1) is not fixed in this patch.
- I ran scoped verification only (parallel-agent safe), not full workspace `cargo check/clippy/test`.

**Commit**
- `29b06d7` `fix(app_shell): normalize key variants in shell keymap`