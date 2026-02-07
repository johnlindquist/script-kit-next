Created the keyboard/focus audit report at `plans/keyboard-ux-improvements.md`.

**Changed Files**
- `plans/keyboard-ux-improvements.md`

**Verification Run**
- `cargo check` ✅ passed
- `cargo clippy --all-targets -- -D warnings` ❌ failed due existing workspace issues outside this report task (examples: `src/render_prompts/arg.rs:620`, `src/main.rs:3846`, `src/ai/window.rs:8549`)
- `cargo test keyboard_routing_tests -- --nocapture` ❌ failed for the same current workspace compile issue (`src/ai/window.rs:8549`)

**How To Test**
1. Open and review `plans/keyboard-ux-improvements.md`.
2. Re-run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test keyboard_routing_tests -- --nocapture`

**Risks / Known Gaps**
- This task delivered analysis/report only; no production Rust behavior was changed.
- Clippy/test are currently blocked by pre-existing compile/lint errors in other in-progress files, so full green verification is not possible on this snapshot.

**Commits**
- None.