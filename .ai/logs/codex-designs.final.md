Implemented the `src/designs` split so oversized files are now sub-moduled and all `src/designs/*.rs` files are under 500 lines.

**Changed files**
- Modified wrappers:
  - `src/designs/mod.rs`
  - `src/designs/traits.rs`
  - `src/designs/separator_variations.rs`
  - `src/designs/icon_variations.rs`
  - `src/designs/retro_terminal.rs`
  - `src/designs/neon_cyberpunk.rs`
  - `src/designs/playful.rs`
  - `src/designs/glassmorphism.rs`
- Added:
  - `src/designs/core.rs`
  - `src/designs/core/` (`parts.rs`, `part_01.rs`..`part_04.rs`, `tests.rs`, `tests/part_01.rs`..`part_04.rs`)
  - `src/designs/traits/` (`parts.rs`, `part_01.rs`..`part_05.rs`)
  - `src/designs/separator_variations/` (`parts.rs`, `part_01.rs`..`part_05.rs`)
  - `src/designs/icon_variations/` (`parts.rs`, `part_01.rs`, `part_02.rs`)
  - `src/designs/retro_terminal/` (`parts.rs`, `part_01.rs`, `part_02.rs`)
  - `src/designs/neon_cyberpunk/` (`parts.rs`, `part_01.rs`, `part_02.rs`)
  - `src/designs/playful/` (`parts.rs`, `part_01.rs`, `part_02.rs`)
  - `src/designs/glassmorphism/` (`parts.rs`, `part_01.rs`, `part_02.rs`)

**How to test**
1. Line limits:
   - `find src/designs -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head`
2. Required gate:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
3. Optional scope sanity check:
   - `cargo check 2>&1 | rg "src/designs/"` (currently returns no `src/designs` errors)

**Verification result / known gaps**
- I ran the required gate command. It fails before reaching clippy/tests due unrelated repo-wide compile issues outside this scope, e.g.:
  - `src/protocol/io/parsing.rs:1`
  - `src/actions/builders/chat.rs:1`
  - `src/prompts/markdown/mod.rs:39`
- `src/designs` split issues were fixed (including include-boundary/doc-comment issues and missing import context in `src/designs/core/parts.rs`).

**Commits**
- None.