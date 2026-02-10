**Changed Files**
- `src/components/prompt_footer.rs`

**What I changed**
- Added a pressed-state token and resolver for footer buttons:
  - `PROMPT_FOOTER_BUTTON_ACTIVE_OPACITY` (`src/components/prompt_footer.rs:48`)
  - `footer_button_active_rgba(...)` (`src/components/prompt_footer.rs:162`)
- Updated clickable footer buttons to include:
  - `.cursor_pointer()`
  - `.hover(|s| s.bg(...))`
  - `.active(|s| s.bg(...))`
  - in `render_button` (`src/components/prompt_footer.rs:368`)
- Confirmed disabled buttons stay non-clickable affordance:
  - `.opacity(0.5).cursor_default()` (`src/components/prompt_footer.rs:373`)
- Added tests for active-state token behavior and constant coverage:
  - `test_footer_button_active_rgba_uses_background_token_with_pressed_opacity` (`src/components/prompt_footer.rs:644`)
  - layout token assertion includes active opacity (`src/components/prompt_footer.rs:678`)

**How to test**
1. `cargo check --lib`
2. `cargo clippy --lib -- -D warnings`
3. `cargo test prompt_footer::tests -- --nocapture`

**Verification run results**
- `cargo check --lib`: passed
- `cargo clippy --lib -- -D warnings`: passed
- `cargo test prompt_footer::tests -- --nocapture`: blocked by unrelated existing compile errors (`E0753`) in other test modules, so footer tests did not execute.

**Risks / Known Gaps**
- Scoped footer tests could not run due unrelated pre-existing test compile failures outside this file.
- No runtime screenshot validation was performed in this pass.