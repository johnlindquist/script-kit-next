Implemented the config visibility fix and resolved the hotkey type mismatches that were blocking compile.

**Changed Files**
- `src/config/mod.rs:14` changed `mod defaults;` to `pub mod defaults;` so `config::defaults::*` is externally visible.
- `src/config/mod.rs:23` added `#[allow(unused_imports)]` on type re-exports to keep intentional public API exports from warning.
- `src/hotkeys.rs:366` and `src/hotkeys.rs:374` updated AI/logs hotkey update paths to handle `Option<HotkeyConfig>`.
- `src/hotkeys.rs:1171` and `src/hotkeys.rs:1174` updated startup registration to only register AI/logs hotkeys when configured/enabled.

**How To Test**
1. `cargo check`  
   Result: passes.
2. `cargo clippy --all-targets -- -D warnings`  
   Result: fails due many pre-existing workspace lint errors outside this task (e.g. `src/theme/types.rs`, `src/watcher.rs`, `src/actions/dialog.rs`, `src/prompts/div.rs`, etc.).
3. `cargo test`  
   Result: fails with existing suite failures (14 failing tests, including `config::loader::tests::test_config_loader_preserves_valid_fields_when_one_field_invalid`, `prompts::markdown::tests::nested_lists_preserve_parent_child_structure`, and others).
4. Focused verification for this change: `cargo test --lib hotkeys::`  
   Result: passes (19 passed, 0 failed).

**Risks / Known Gaps**
- Repo is currently red from unrelated clippy/test failures, so full gate is not green.
- No behavior risk expected from `pub mod defaults`; hotkey changes now correctly skip AI/logs registration when those hotkeys are disabled/unset.

**Commits**
- None.