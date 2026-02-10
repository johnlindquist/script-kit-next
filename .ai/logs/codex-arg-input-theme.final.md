Updated the arg prompt input color path to use active theme tokens instead of design defaults, which fixes white text on light backgrounds.

Changed files:
- `src/render_prompts/arg/render.rs:255`
  - `text_primary` now uses `self.theme.colors.text.primary`
  - `text_muted` now uses `self.theme.colors.text.muted`
  - `accent_color` now uses `self.theme.colors.accent.selected`
  - Empty-state muted text also now uses the same `text_muted` token (`src/render_prompts/arg/render.rs:271`)
- `src/render_prompts/arg/tests.rs:135`
  - Added `test_arg_prompt_input_text_uses_theme_tokens_when_rendering` regression check.

How to test:
1. Build-check the target:
   - `cargo check --bin script-kit-gpui`
2. Run app and open an `await arg()` prompt under a light theme; type in the input and confirm text follows theme foreground color (not white on light bg).

Verification run:
- Passed: `cargo check --bin script-kit-gpui`
- Failed due unrelated existing repo issues:
  - `cargo test test_arg_prompt_input_text_uses_theme_tokens_when_rendering -- --nocapture`
  - `cargo test --bin script-kit-gpui test_arg_prompt_input_text_uses_theme_tokens_when_rendering -- --nocapture`
  - `cargo clippy --bin script-kit-gpui -- -D warnings`

Risks / known gaps:
- Existing workspace test/clippy breakage prevents full automated validation of the new test execution path.
- Regression test is source-assertion based, not a runtime UI render assertion.

Commits made:
- `c8180e8f32e7c7848cdefb7de966b8b6ed5b4e46` - `fix(arg-prompt): use theme tokens for input text colors`