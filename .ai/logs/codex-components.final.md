Implemented the `src/components` split so all previously oversized component files are now under 500 lines, including an extra split of `src/components/alias_input/component.rs` (was 504 after first pass).

**Changed files**
- `src/components/alias_input.rs`
- `src/components/alias_input/component.rs`
- `src/components/alias_input/render.rs`
- `src/components/alias_input/tests.rs`
- `src/components/alias_input/types.rs`
- `src/components/button.rs`
- `src/components/button/component.rs`
- `src/components/button/tests.rs`
- `src/components/button/types.rs`
- `src/components/form_fields.rs`
- `src/components/form_fields/checkbox.rs`
- `src/components/form_fields/colors.rs`
- `src/components/form_fields/helpers.rs`
- `src/components/form_fields/state.rs`
- `src/components/form_fields/text_area/mod.rs`
- `src/components/form_fields/text_area/render.rs`
- `src/components/form_fields/text_field/mod.rs`
- `src/components/form_fields/text_field/render.rs`
- `src/components/form_fields_tests.rs`
- `src/components/prompt_header.rs`
- `src/components/prompt_header/component.rs`
- `src/components/prompt_header/tests.rs`
- `src/components/prompt_header/types.rs`
- `src/components/shortcut_recorder.rs`
- `src/components/shortcut_recorder/component.rs`
- `src/components/shortcut_recorder/render.rs`
- `src/components/shortcut_recorder/render_helpers.rs`
- `src/components/shortcut_recorder/tests.rs`
- `src/components/shortcut_recorder/types.rs`
- `src/components/text_input.rs`
- `src/components/text_input/core.rs`
- `src/components/text_input/tests.rs`
- `src/components/toast.rs`
- `src/components/toast/constructors.rs`
- `src/components/toast/model.rs`
- `src/components/toast/render.rs`
- `src/components/toast/tests.rs`
- `src/components/toast/types.rs`

**How to test**
- Line-count check (all component files now <= 500):  
  `find src/components -maxdepth 3 -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -n 60`
- Formatting check on touched files:  
  `rustfmt --edition 2021 --check <touched component files>`
- Verification gate attempted:
  - `cargo check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

**Verification result**
- `cargo check`, `cargo clippy`, and `cargo test` are currently failing due repo-wide errors outside `src/components` (examples: `src/camera/part_000.rs`, `src/debug_grid/part_000.rs`, `src/editor/part_001.rs`, missing files under `src/setup/...`).
- No failure lines referenced the component files I changed when filtering compiler output for `src/components/(alias_input|button|form_fields|prompt_header|shortcut_recorder|text_input|toast)`.

**Risks / known gaps**
- Full compile/test verification of this split is blocked until upstream non-component errors are fixed.
- `src/components/mod.rs` did not need API changes because top-level module names/exports stayed stable.

**Commits**
- None. I did not commit unverified work because the required gate is currently blocked by unrelated workspace errors.