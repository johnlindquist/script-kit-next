Implemented the assigned scoped fixes in the two requested files.

**Changed files**
- `src/actions/types/action_model.rs:44`
  - Added `PartialEq` derive to `Action`.
- `src/actions/types/action_model.rs:157`
  - Feature-gated unused `ActionCategory` variants with `#[cfg(test)]`:
  - `ScriptOps` at `src/actions/types/action_model.rs:164`
  - `GlobalOps` at `src/actions/types/action_model.rs:168`
  - Kept `Terminal` unchanged for runtime usage.
- `src/actions/builders/notes.rs:323`
  - Preset actions now include descriptions via `Some(new_chat_preset_description(preset))` instead of `None`.
- `src/actions/builders/notes.rs:351`
  - Added `new_chat_preset_description()` helper (`"Use {name} preset"`).
- `src/actions/builders/notes.rs:276`
  - Added test `test_get_new_chat_actions_includes_descriptions_for_presets`.

**Verification run**
- Ran: `cargo check -p script-kit-gpui`
  - Failed due pre-existing unrelated errors (e.g. missing `ClipboardEntry` fields and non-exhaustive `ContentType` matches in `src/clipboard_history/*`).
- Ran: `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
  - Failed due pre-existing unrelated workspace issues (including duplicate test module files and broad existing warnings/errors outside scoped files).
- Attempted targeted test run for the new preset-description test, but compilation is currently blocked by unrelated existing errors.

**How to test (once workspace compiles cleanly)**
1. `cargo test -p script-kit-gpui --lib test_get_new_chat_actions_includes_descriptions_for_presets`
2. `cargo check -p script-kit-gpui`
3. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

**Risks / known gaps**
- Full verification is blocked by existing non-scope compile/lint failures in other modules.
- There are existing tests in other files that previously asserted preset descriptions were `None`; once those suites are runnable, they may need updates to align with the new preset-description behavior.