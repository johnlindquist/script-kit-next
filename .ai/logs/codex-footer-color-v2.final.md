Implemented the footer light-mode color regression fix while preserving newer footer structure improvements.

**Changed Files**
- `src/components/prompt_footer.rs`
  - Reverted footer color token mapping:
    - `PromptFooterColors::from_theme()` now uses `theme.colors.accent.selected_subtle` (not `background.search_box`).
    - `PromptFooterColors::from_design()` now uses `colors.background_selected` (not `background_secondary`).
    - `PromptFooterColors::default().background` restored to `0xffffff`.
  - Updated `footer_surface_rgba()` to legacy-safe behavior:
    - Light mode: fixed `0xf2f1f1ff`.
    - Dark mode: `(background << 8) | 0x33`.
  - Kept structural/footer UX improvements (width caps, disabled button flags, etc.) intact.
- `src/render_prompts/arg.rs`
  - `prompt_footer_colors_for_prompt()` now uses `design_colors.background_selected` for footer surface.
  - Updated associated unit expectation accordingly.
- `tests/prompt_footer.rs`
  - Updated/added integration assertions to lock in:
    - legacy footer surface RGBA behavior,
    - `from_theme()` selected-subtle mapping,
    - `from_design()` selected-background mapping,
    - default background remains white.

**Verification Run**
- `cargo test --test prompt_footer` (passes, 9 tests)
- `cargo check` (passes)
- Runtime stdin smoke test:
  - `printf '{"type":"show"}\n' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui`
  - Log confirmed `Processing external command type=show` and window show flow.

**How To Test**
1. `cargo check`
2. `cargo test --test prompt_footer`
3. `printf "%s\n" "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Risks / Known Gaps**
- I did not run full `cargo clippy --all-targets -- -D warnings` or full `cargo test` in this turn.
- `src/prompts/select.rs` and `src/render_script_list.rs` were already dirty in the working tree; I did not modify them for this fix.

**Commits**
- No commits made.