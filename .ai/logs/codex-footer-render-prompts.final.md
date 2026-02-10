Implemented the footer audit/fix in the requested scope.

**Changed files**
- `src/render_prompts/arg/render.rs:414`  
- `src/render_prompts/arg/render_prompt.rs:330`  
- `src/render_prompts/editor.rs:272`  
- `src/render_prompts/form/render.rs:212`  
- `src/render_prompts/div.rs:121`  
- `src/render_prompts/other.rs:214`  
- `src/render_prompts/arg/helpers.rs:15`  
- `src/render_prompts/arg/tests.rs:35`  

**What changed**
- Replaced prompt footer color wiring from `prompt_footer_colors_for_prompt(...)` to `PromptFooterColors::from_theme(&self.theme)` in prompt renderers that build a shared `PromptFooter` (arg, form, editor, div, webcam).
- Removed the obsolete `prompt_footer_colors_for_prompt` helper from `src/render_prompts/arg/helpers.rs`.
- Updated arg footer color test to validate `PromptFooterColors::from_theme(...)` token mapping.

**How to test**
- `cargo test --bin script-kit-gpui prompt_footer_colors_from_theme_match_cached_theme_tokens`
- `cargo test --bin script-kit-gpui test_editor_footer_displays_cmd_enter_and_cmd_s_submit_hints`
- `cargo test --bin script-kit-gpui form_footer_status_text_mentions_cmd_enter_for_textarea`
- `cargo test --bin script-kit-gpui simple_prompt_wrappers_skip_unused_shell_allocations`
- Optional audit grep:
  - `rg -n "prompt_footer_colors_for_prompt" src/render_prompts src/render_builtins` (should return no matches)
  - `rg -n "PromptFooterColors::from_theme\\(&self.theme\\)" src/render_prompts src/render_builtins`

**Risks / known gaps**
- I ran scoped verification only (per parallel-agent constraints), not full workspace `cargo check/clippy/test`.
- `select/env/template/chat/path/markdown` wrappers in `render_prompts` do not construct footers directly; their footer behavior is handled in their prompt entities outside this assignment’s scope.
- `src/render_builtins/theme_chooser.rs` and `src/render_builtins/theme_chooser_footer_return.rs` still intentionally use custom built-in footer divs (not prompt-specific `PromptFooter`).

Commits made: none.