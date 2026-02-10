Implemented the footer shortcut keycap styling in `src/components/prompt_footer.rs`.

**Changed files**
- `src/components/prompt_footer.rs:54`
- `src/components/prompt_footer.rs:348`
- `src/components/prompt_footer.rs:369`
- `src/components/prompt_footer.rs:694`

What changed:
- Updated `PROMPT_FOOTER_PADDING_X_PX` from `12.0` to `14.0`.
- In `render_button()`, shortcut text is now wrapped in a keycap pill:
  - `.px(px(6.0)).py(px(1.0)).rounded(px(4.0))`
  - `.bg(self.colors.border.rgba8(0x20))`
  - `.border_1().border_color(self.colors.border.rgba8(0x40))`
  - `.font_family(crate::list_item::FONT_MONO)`
- Kept theme-token-based coloring only (no hardcoded rgb values for this change).

**Verification run**
- `cargo check -p script-kit-gpui` ✅ passed
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` ❌ fails due unrelated pre-existing workspace issues (e.g. duplicate `tests` module files, unresolved imports in `actions`/`scripts`, etc.)
- `cargo test -p script-kit-gpui prompt_footer` ❌ fails due unrelated pre-existing workspace test compile issues

**How to test**
1. Run:
   - `cargo check -p script-kit-gpui`
   - `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
   - `cargo test -p script-kit-gpui prompt_footer`
2. Manual UI check:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
   - Confirm footer shortcuts render as pill keycaps with subtle border/background.

**Risks / known gaps**
- Workspace currently has many unrelated compile/lint/test failures, so clippy/test cannot be fully green from this branch state.
- I did not capture a UI screenshot in this run.

**Commits**
- `afa13dc` `feat(prompt-footer): style shortcut hints as keycaps`

Push status:
- `git push` ✅ succeeded (`main -> main`).