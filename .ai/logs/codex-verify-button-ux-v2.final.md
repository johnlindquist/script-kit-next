Validated and committed the existing scoped changes without rewriting them.

Changed files:
- `src/components/button/component.rs`
- `src/components/button/tests.rs`
- `src/components/alias_input/component.rs`
- `src/components/alias_input/render.rs`
- `src/components/alias_input/tests.rs`

Verification run:
- `cargo check --lib`
- `cargo clippy --lib -- -D warnings`
- `cargo test --lib components::button::tests::`
- `cargo test --lib components::alias_input::tests::`

All passed.

Commit made:
- `60011f1` `fix(components): enforce cursor affordance in button and alias input`

Known gaps / risks:
- No runtime visual screenshot validation was performed in this pass; verification was compile/lint + scoped unit tests only.