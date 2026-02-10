Implemented a focused footer-parity hardening in `src/panel.rs` and completed the audit.

**What Changed**
- Added canonical panel footer color resolver:
  - `src/panel.rs:90` `panel_footer_colors(theme) -> PromptFooterColors`
  - Delegates directly to `PromptFooterColors::from_theme(...)`.
- Added parity tests:
  - `src/panel.rs:373` verifies light/dark token mapping matches `PromptFooter`.
  - `src/panel.rs:399` verifies panel footer surface alpha behavior (`0xff` light, `0x33` dark) via `footer_surface_rgba`.
- Kept clippy-clean test assertion pattern in `src/panel.rs:368`.

**Audit Results**
1. `footer_surface_rgba()` is token-based, not hardcoded:
   - `src/components/prompt_footer.rs:90` uses `(colors.background << 8) | alpha`
   - `src/components/prompt_footer.rs:63` background comes from `theme.colors.accent.selected_subtle`
2. `PromptFooterColors` construction:
   - Active consumers use `from_theme()`/`from_design()` paths.
   - One struct-literal remains in `src/render_script_list/part_000_body/body_002.rs:336`, but that file appears unreferenced (no include/reference found).
3. Panel/footer parity:
   - Now explicitly enforced in `src/panel.rs` via `panel_footer_colors()` and tests.

**Changed Files**
- `src/panel.rs`

**How To Test**
- `cargo test panel::tests::test_panel_footer_ -- --nocapture`
- `cargo test prompt_footer::tests::test_footer_surface_rgba_ -- --nocapture`
- `cargo test prompt_footer::tests::test_prompt_footer_colors_from_design_uses_cached_theme_tokens -- --nocapture`
- `cargo clippy --lib --tests -- -D warnings`

**Risks / Known Gaps**
- The noncompliant struct-literal in `src/render_script_list/part_000_body/body_002.rs:336` was not changed in this task; it appears to be dead/unreferenced code.
- I ran scoped verification only (parallel-agent safety), not full-workspace commands.

**Commit**
- `0f52cdb` `test(panel): enforce prompt footer color parity`