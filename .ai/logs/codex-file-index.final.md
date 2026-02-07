Implemented the file index generator and generated the index.

**Changed Files**
- `scripts/gen-file-index.sh`
- `.ai/file-index.json`

**What I added**
- `scripts/gen-file-index.sh` scans `src/**/*.rs` and generates pretty JSON with, per file:
- `path`
- `module` (top-level module bucket)
- `description` (single-sentence from leading `//!` docs, with fallback)
- `public_types` (`pub struct|enum|trait`)
- `test_files` (heuristic association from `tests/**` and internal `src` test modules)
- `coupled_with` (internal coupling from `use` + `mod` relationships, including reverse edges)

It writes to `.ai/file-index.json` (default) and supports an optional output path argument.

**Validation / Verification run**
1. `bash -n scripts/gen-file-index.sh` passed.
2. `scripts/gen-file-index.sh` passed and generated `.ai/file-index.json`.
3. Required JSON validation passed:
- `python3 -c 'import json; json.load(open(".ai/file-index.json"))'`
4. Sanity check: index has `391` entries, matching `391` Rust files in `src/`.

I also ran the repo gate:
- `cargo check` passed (with warnings).
- `cargo clippy --all-targets -- -D warnings` failed due many pre-existing lint errors outside this task.
- `cargo test` failed with pre-existing failures (12 failing tests, 9130 passed, 35 ignored), including failures in `actions` validation tests, `stdin_commands`, `window_state_tests`, and `prompts::markdown`.

**How to test**
1. Regenerate:
- `scripts/gen-file-index.sh`
2. Validate JSON:
- `python3 -c 'import json; json.load(open(".ai/file-index.json"))'`
3. Optional sanity count:
- `python3 - <<'PY'
import json, pathlib
d=json.load(open('.ai/file-index.json'))
print(len(d['files']), sum(1 for _ in pathlib.Path('src').rglob('*.rs')))
PY`

**Risks / Known gaps**
- `test_files` and `coupled_with` are heuristic (regex + use-tree expansion), not full Rust AST resolution, so a small number of associations may be over/under-inclusive.
- Nested/complex import patterns are handled better now, but macro-generated/module-reexport-only coupling can still be missed.

**Commits**
- No commits were made (repo-wide strict clippy/test gate is currently failing due unrelated pre-existing issues).