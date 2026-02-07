**Changed Files**
- `scripts/agent-check.sh`
- `scripts/agent-scope.sh`

**What I Implemented**
- Added `scripts/agent-check.sh`:
  - Accepts changed files as args.
  - Supports `--quick` (runs only `cargo check`).
  - Full mode runs `cargo check`, related test filters first, then `cargo clippy --all-targets -- -D warnings`, then `cargo test`.
  - Derives related test filters from changed file path tokens (example: `src/actions/dialog.rs` -> `actions`, `dialog`).
  - Prints clear `[PASS]/[FAIL]` statuses and per-step + total timing.
- Added `scripts/agent-scope.sh`:
  - Accepts a module name (e.g., `actions`, `prompts`, `theme`).
  - Prints sorted file list for that module scope via directory/file + name-pattern matching.
- Set both scripts executable with `chmod +x`.

**Validation Run**
- `bash scripts/agent-check.sh` (before creation): failed as expected (`No such file or directory`) for TDD baseline.
- `bash -n scripts/agent-check.sh scripts/agent-scope.sh`: pass.
- `bash scripts/agent-scope.sh actions | head`: pass, lists expected action files.
- `bash scripts/agent-scope.sh prompts | head`: pass.
- `bash scripts/agent-scope.sh theme | head`: pass.
- `bash scripts/agent-check.sh --quick src/actions/dialog.rs`: pass.
- Required command `bash scripts/agent-check.sh`: script works; run fails at repo-level clippy errors already present outside these script changes.
- `bash scripts/agent-check.sh src/actions/dialog.rs`: confirmed related-test prepass runs first (`actions`, `dialog`), then fails on existing repo test failures.

**Risks / Known Gaps**
- Related-test detection is heuristic/token-based; it may run broader test filters than intended in some modules.
- `agent-scope.sh` uses path/name matching and may include extra files sharing the same token.
- Full verification currently fails due pre-existing repository clippy/test issues unrelated to these new scripts.

**Commits**
- No commits were made.