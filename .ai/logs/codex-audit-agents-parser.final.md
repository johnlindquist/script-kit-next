**Findings**
1. High (fixed): frontmatter delimiter parsing was too permissive and could mis-detect malformed delimiters (`---not-frontmatter`, `---not-a-delimiter`), causing incorrect parse/skip behavior. Hardened in `src/agents/parser.rs:55`.
2. Medium (fixed): BOM/line-ending robustness was incomplete; UTF-8 BOM and CRLF delimiter edge cases now parse correctly via normalized start and line-splitting logic in `src/agents/parser.rs:56` and `src/agents/parser.rs:117`.
3. Medium (fixed): non-string YAML keys could invalidate frontmatter handling; parser now ignores non-string keys and keeps valid string keys with warning in `src/agents/parser.rs:94`.
4. Medium (fixed): `_interactive` previously treated any non-bool value as `true` (including `"false"`/`0`); now coerces bool/null/string/number safely in `src/agents/parser.rs:172` and `src/agents/parser.rs:304`.
5. Panic audit: no direct panic paths found in parser runtime code; malformed frontmatter paths return `Err`/`None` and are recovered by `parse_agent`.

**Changed Files**
- `src/agents/parser.rs`

**How To Test**
1. `cargo test agents::parser -- --nocapture`
2. `cargo test --test agents_parser_consistency -- --nocapture`
3. `cargo check --lib`
4. `cargo clippy --lib --tests -- -D warnings`

**Risks / Known Gaps**
- `src/agents/parser.rs:387`: non-UTF8 filenames are still skipped (`to_str()?`).
- `src/agents/parser.rs:348`
- `src/agents/parser.rs:356`
Detection for shell inlines/remote imports is substring-based and can still false-positive in some markdown contexts.

**Commits Made**
- `643b41a` `fix(agents): parse interactive flags from strings and numbers`

