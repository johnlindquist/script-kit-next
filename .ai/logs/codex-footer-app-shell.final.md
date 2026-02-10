Footer audit and fix is complete.

**Findings**
1. `src/app_render.rs:1` and `src/app_layout.rs:1` only include split modules; they do not build or paint the footer.
2. Main-window footer path is:
`src/main_sections/render_impl.rs:160` -> `src/render_script_list/part_000.rs:1197` -> `src/components/prompt_footer.rs:348`.
3. Footer background paint happens in `src/components/prompt_footer.rs:364` via `.bg(footer_bg)`, where `footer_bg` comes from `footer_surface_rgba` (`src/components/prompt_footer.rs:90`).
4. Inconsistency found: app-shell footer colors were sourced from `ColorResolver::selection_background()` in `src/render_script_list/part_000.rs` (design-token path can be white for some variants), which could yield a light/white footer.

**Fixes**
1. `src/render_script_list/part_000.rs:1`
Added `app_shell_footer_colors(theme)` and switched footer color sourcing to `PromptFooterColors::from_theme(&self.theme)` (`src/render_script_list/part_000.rs:863`, `src/render_script_list/part_000.rs:1238`), guaranteeing theme token usage (`accent.selected_subtle` background).
2. `src/components/prompt_footer.rs:83`
Changed `PromptFooterColors::default()` to use cached theme tokens (removed hardcoded white fallback).
3. Added tests:
- `src/render_script_list/part_000.rs:1291` (`test_app_shell_footer_colors_use_theme_accent_tokens`)
- `src/components/prompt_footer.rs:468` (`test_prompt_footer_colors_default_uses_cached_theme_tokens`)

**Changed Files**
- `src/render_script_list/part_000.rs`
- `src/components/prompt_footer.rs`

**Verification**
- `cargo test --bin script-kit-gpui components::prompt_footer::tests::test_prompt_footer_colors_default_uses_cached_theme_tokens -- --exact`
- `cargo test --bin script-kit-gpui render_script_list_footer_tests::test_app_shell_footer_colors_use_theme_accent_tokens -- --exact`
- `cargo check --bin script-kit-gpui`

**How To Test**
1. Run the two exact tests above.
2. Launch app in light mode and switch design variants; the main footer should keep theme-token selected-subtle styling instead of turning white.

**Risks / Known Gaps**
1. I ran scoped checks only (parallel-agent workspace); I did not run full `cargo clippy --all-targets -- -D warnings` or full `cargo test`.
2. A broad `cargo test ... footer_colors` filter includes an unrelated pre-existing arg-prompt test failure in this repo.

**Commits**
- `3cb9552` `fix(footer): enforce theme-token app shell footer colors`