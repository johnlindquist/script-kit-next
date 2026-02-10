Verified and committed the existing prompt-header changes.

Changed files:
- `src/components/prompt_header/component.rs`
- `src/components/prompt_header/tests.rs`

What was verified:
- In `src/components/prompt_header/component.rs`, both “Ask AI” and “Tab” hint buttons have:
  - `.cursor_pointer()`
  - `.bg(rgba(transparent_bg))`
  - `.hover(move |style| { ... })` (hover feedback)
- `src/components/prompt_header/tests.rs` includes assertions for transparent background, pointer cursor, hover feedback, and ghost spacing tokens.

How to test:
1. `cargo check --lib`
2. `cargo clippy --lib -- -D warnings`
3. Optional scoped test attempt: `cargo test --lib prompt_header::tests`

Verification results:
- `cargo check --lib`: pass
- `cargo clippy --lib -- -D warnings`: pass
- `cargo test --lib prompt_header::tests`: blocked by unrelated compile error `E0753` in `src/app_impl/clipboard_actions_focus_routing_tests.rs` (outside this task’s scope)

Commit made:
- `fd4dde4` `fix(prompt-header): enforce ghost hover UX for Ask AI hints`

Risks / known gaps:
- Prompt-header test execution is currently blocked by unrelated lib-test compilation errors outside the assigned files.