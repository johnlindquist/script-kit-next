**Findings**
1. High (fixed): `src/prompts/chat/tests.rs:219` was effectively always-pass (`revealed = content`), so it did not validate streaming boundary behavior at all.
2. Medium (fixed): `src/prompts/chat/tests.rs:255` only checked monotonic offsets and length bound, not reconstruction integrity; now it verifies UTF-8 boundary safety and exact content reconstruction.
3. Medium (fixed): `src/prompts/chat/tests.rs:350` only asserted “no panic” for large scroll indices; now it also asserts the expected saturated behavior result.
4. Medium (fixed): `src/prompts/select/tests.rs:26` only asserted positive matches; now it also asserts a clear no-match path to catch “always returns Some” regressions.
5. Medium (fixed): `src/prompts/select/tests.rs:93` previously only checked a single positive char (`' '`); now it also checks control-char rejection (`'\n'`, `'\0'`).
6. Medium (fixed): `src/prompts/env/tests.rs:76` key action coverage missed canonical/cased aliases; now includes `Enter`, `escape`, and `ESC`.
7. Medium (fixed): `src/prompts/template/tests.rs:147` had missing validation branch coverage; now covers required-empty rejection, optional-empty acceptance, and invalid slug hyphen patterns.

**Changed Files**
- `src/prompts/chat/tests.rs`
- `src/prompts/select/tests.rs`
- `src/prompts/env/tests.rs`
- `src/prompts/template/tests.rs`

**How To Test**
1. `cargo test --lib prompts::chat::tests`
2. `cargo test --lib prompts::select::tests`
3. `cargo test --lib prompts::env::tests`
4. `cargo test --lib prompts::template::tests`

(All passed.)

**Risks / Known Gaps**
- `src/prompts/chat/tests.rs:290` still does not validate all `build_conversation_turns` metadata fields (`model`, `streaming`, `error`, image-cache attachment, assistant ID precedence).
- `src/prompts/select/tests.rs:10` metadata parsing is still mostly happy-path; malformed descriptor cases are not covered.
- `src/prompts/select/tests.rs:85` UTF-8 range conversion still lacks explicit out-of-range/unsorted-index behavior tests.

**Commits**
- `ac2eb93` `test(prompts): harden weak prompt helper assertions`
- `f2cf84d` `test(prompts): add env and template edge-case coverage`