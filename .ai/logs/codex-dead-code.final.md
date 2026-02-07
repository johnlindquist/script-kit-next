Implemented the task by creating a comprehensive cleanup report at `plans/dead-code-cleanup.md`.

**Changed Files**
- `plans/dead-code-cleanup.md`

**What’s in the report**
- Compiler-confirmed unused imports (26 exact `file:line` findings)
- Dead-code suppression hotspots (`#[allow(dead_code)]` / `#[allow(unused_imports)]`)
- Unused-function and unused-struct-field candidates with specific references
- Unreachable macro usage (`src/schema_parser.rs:414`)
- Commented-out/stale code blocks
- Full TODO/FIXME/HACK inventory (26 references)
- Feature-flag usage audit (no unused feature flag found)
- Stub macro audit (`todo!`/`unimplemented!` none found)
- Prioritized cleanup order

**How To Test / Re-verify**
1. `cargo check --all-targets`
2. `cargo clippy --all-targets -- -W dead_code -W unused -W unreachable_code -W unreachable_patterns`
3. `cargo clippy --bin script-kit-gpui -- -W dead_code -W unused -W unreachable_code -W unreachable_patterns`
4. `cargo test`
5. Inspect `plans/dead-code-cleanup.md`

**Current Verification Results**
- `cargo check --all-targets`: fails due existing blockers:
  - `src/render_prompts/arg.rs:620` (`tests` module redefined, collides with `src/main.rs:3846`)
  - `src/ai/window.rs:8549` (`AiApp::message_body_content` missing)
- `cargo clippy --bin script-kit-gpui ...`: 1 warning (`src/render_prompts/other.rs:313` needless return)
- `cargo test`: fails due existing blocker at `src/ai/window.rs:8549`
- The report captures these blockers as audit limitations.

**Risks / Known Gaps**
- Full dead-code discovery is partially masked by `#[allow(dead_code)]` usage across the codebase.
- Because test targets currently fail to compile, additional compiler-detected dead code may remain hidden until blockers are fixed.

**Commits**
- No commits made.