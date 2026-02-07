**Changed Files**
- `AGENTS.md`

**What I changed**
- Replaced and streamlined `AGENTS.md` into a 28.5k-character agent guide.
- Added a full top-level module ownership map for `src/` (94 modules), including:
  - what each module owns,
  - key public types/APIs,
  - primary files to edit.
- Added file-scope boundaries (low-coupling vs tightly-coupled areas).
- Added a dependency graph section with:
  - top inbound dependency hubs,
  - full top-level adjacency list (`module -> internal dependencies`) derived from `use crate::...` / `crate::...`.
- Added common implementation flows:
  - `BuiltInFeature` -> `get_builtin_entries` -> `execute_builtin` -> `AppView`,
  - prompt render layering (`render_prompts/*.rs` wrapping `prompts/*.rs`),
  - stdin/stdout JSONL protocol flow.
- Added testing guidance:
  - full verification gate,
  - scoped Rust test patterns,
  - stdin JSONL smoke-test patterns,
  - test inventory map.

**How to test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Optional UI protocol smoke:
   - `echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-main-menu.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

**Verification results**
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due existing repo-wide lint debt (not introduced by this docs-only change).
- `cargo test`: failed due existing test failures unrelated to `AGENTS.md` (12 failing tests in current tree).

**Risks / Known gaps**
- Dependency adjacency is static-analysis-based (crate import references) and may not capture dynamic behavior through runtime dispatch or macro-generated paths.
- Key public API names in the ownership map are derived from module exports and may omit secondary symbols.

**Commits**
- No commits made (verification gate is currently red in this working tree).