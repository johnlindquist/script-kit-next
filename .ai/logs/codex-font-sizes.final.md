Implemented a typography normalization pass focused on shared UI components and prompt header input sizing, using theme/design typography tokens instead of mixed hardcoded text classes.

**Changed files**
- `src/components/form_fields.rs:93`  
  Added `input_font_size` / `label_font_size` tokens to `FormFieldColors` and switched text field, textarea, checkbox label/checkmark rendering to tokenized `text_size(...)` calls (`src/components/form_fields.rs:656`, `src/components/form_fields.rs:1219`, `src/components/form_fields.rs:1450`).
- `src/components/form_fields_tests.rs:172`  
  Added guard tests to enforce tokenized form field typography and arg-header token usage (`src/components/form_fields_tests.rs:195`).
- `src/components/prompt_header.rs:41`  
  Added header typography tokens (`input/supporting/caption`) and replaced hardcoded `text_lg/sm/xs` with explicit tokenized sizes (`src/components/prompt_header.rs:289`, `src/components/prompt_header.rs:454`, `src/components/prompt_header.rs:535`).
- `src/components/prompt_footer.rs:268`  
  Footer info/helper text now uses tokenized sizes derived from theme font tokens (instead of hardcoded `text_xs/text_sm`).
- `src/components/prompt_container.rs:30`  
  Added `hint_font_size` token and applied it to hint rendering (`src/components/prompt_container.rs:223`, `src/components/prompt_container.rs:291`).
- `src/components/footer_button.rs:67`  
  Footer button label/shortcut text now uses tokenized size from theme font tokens.
- `src/render_prompts/arg.rs:413`  
  Normalized arg header input from hardcoded `text_xl()` to `text_size(px(design_typography.font_size_lg))`.

**Validation run**
- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ (pre-existing repo-wide failures in action validation test files and one clippy issue in `src/ai/window.rs`)
- `cargo test` ✅
- Targeted tests:
  - `cargo test --lib test_form_fields_use_theme_token_font_sizes -- --nocapture` ✅
  - `cargo test --lib test_arg_prompt_header_uses_design_token_large_input_size -- --nocapture` ✅
- UI/runtime protocol check:
  - Ran stdin JSON protocol with:
    - `{"type":"show"}` and
    - `{"type":"run","path":".../tests/smoke/test-footer-button.ts"}`
  - Logs confirmed valid parse/processing of JSON stdin commands.

**How to test**
1. `cargo check`
2. `cargo test --lib test_form_fields_use_theme_token_font_sizes -- --nocapture`
3. `cargo test --lib test_arg_prompt_header_uses_design_token_large_input_size -- --nocapture`
4. `cargo test`
5. `timeout 15 bash -c 'printf "%s\n" "{\"type\":\"run\",\"path\":\"'"$(pwd)"'/tests/smoke/test-footer-button.ts\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`

**Risks / known gaps**
- `clippy -D warnings` is still blocked by unrelated existing issues outside this change set.
- Footer helper/info typography is derived from current theme font config; if a design variant intentionally diverges typography from theme fonts, footer text may be slightly less variant-specific than other tokenized surfaces.