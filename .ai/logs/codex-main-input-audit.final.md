Implemented an input-color audit hardening pass in scoped component files by adding regression tests (no runtime behavior change was needed because component input text already uses theme tokens).

**Changed files**
- `src/components/prompt_input.rs:497`
- `src/components/prompt_header/tests.rs:15`

**What I verified in audit**
- `PromptInput` input/placeholder text uses theme-derived tokens (`colors.text_primary` / `colors.text_muted`) in `src/components/prompt_input.rs:515`.
- `PromptHeader` input area uses theme-derived tokens in `src/components/prompt_header/component.rs` (guarded by new tests in `src/components/prompt_header/tests.rs:136`).
- Main select input path uses theme tokens in `src/prompts/select/render.rs:62` and `src/prompts/select/render.rs:63`.
- Arg prompt input path uses theme tokens in `src/render_prompts/arg/render.rs:255` and `src/render_prompts/arg/render.rs:256`.
- Shared component exists (`PromptInput`), but main select and arg prompt render paths are still separate (not yet both routed through shared `PromptInput`).

**How to test**
1. `rustfmt src/components/prompt_input.rs src/components/prompt_header/tests.rs`
2. `cargo check --lib`
3. Intended scoped tests:
   - `cargo test prompt_header::tests:: -- --nocapture`
   - `cargo test prompt_input::tests:: -- --nocapture`

**Risks / known gaps**
- Scoped `cargo test` is currently blocked by unrelated workspace conflicts (duplicate test module paths and unrelated action-test compile errors outside this task’s scope), so new tests were added but could not be executed end-to-end in this shared in-progress tree.
- Added tests are source-level guardrails (string assertions), not rendered UI snapshot tests.

**Commit**
- `7a6c0cb` — `test(components): add input color token audit coverage`