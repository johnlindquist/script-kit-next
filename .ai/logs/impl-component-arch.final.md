Implemented the component-architecture task by centralizing duplicated input defaults and improving component reuse surfaces in `src/components/**`.

**Changed files**
- `src/components/input_tokens.rs` (new): shared input placeholder/font-size constants.
- `src/components/mod.rs`: added/exports for `input_tokens` and `script_kit_input` (`ScriptKitInput`, `ScriptKitInputConfig`, shared constants).
- `src/components/prompt_input.rs`: replaced duplicated placeholder literals with shared constants.
- `src/components/script_kit_input.rs`: replaced duplicated placeholders/font sizes with shared constants; added parity tests to enforce consistency with `PromptInputConfig`.

**How to test**
1. `cargo test --lib script_kit_input::tests:: -- --nocapture`
2. `cargo check`
3. Full gate (optional for current repo state): `cargo clippy --all-targets -- -D warnings && cargo test`

**Verification run**
- `cargo check`: passed.
- `cargo test --lib script_kit_input::tests:: -- --nocapture`: passed.
- Full `cargo clippy` and full `cargo test`: failing due existing unrelated issues outside this scope.

**Risks / known gaps**
- Broader migration to use `ScriptKitInput` in other non-component areas was not in this scoped change.
- Existing repo-wide lint/test failures remain and should be resolved separately for green full-gate CI.

**Commits**
- None.