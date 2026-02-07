Implemented API-surface cleanup from `plans/api-surface-cleanup.md` and released all swarm claims for `impl-api-surface`.

**Changed Files**
- `src/lib.rs`
- `src/window_control_enhanced/mod.rs`
- `src/theme/color_resolver.rs`
- `src/render_script_list.rs`
- `src/protocol/io.rs`
- `src/protocol/mod.rs`
- `src/notes/mod.rs`
- `src/icons/mod.rs`
- `src/icons/types.rs`
- `src/components/unified_list_item/mod.rs`

**What Changed**
- Reduced visibility for internal APIs (`pub` -> private / `pub(crate)`) in protocol parsing internals and notes internals.
- Made `window_control_enhanced` crate-private from the crate root.
- Removed wildcard re-exports and replaced with explicit `pub use` lists in protocol/icons/unified-list modules.
- Added compile-fail doctests documenting/enforcing private API boundaries (`protocol`, `notes`, `window_control_enhanced`, `theme/color_resolver`).
- Updated color/typography/spacing resolver usage to accessor methods after resolver field privatization.

**How To Test**
1. `cargo check`
2. `cargo test --doc protocol -- --nocapture`
3. `cargo test --doc notes -- --nocapture`
4. `cargo test --doc window_control_enhanced -- --nocapture`
5. `cargo test --doc color_resolver -- --nocapture`
6. Runtime sanity (stdin protocol):
   - `timeout 8 bash -c 'printf "%s\n" "{\"type\":\"show\"}" | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`

**Verification Status**
- `cargo check`: passed
- Targeted doctests above: passed
- `cargo clippy --all-targets -- -D warnings`: fails due unrelated existing workspace issues
- `cargo test`: fails due unrelated existing workspace failures (10 failing tests outside this task’s scope)

**Risks / Known Gaps**
- External consumers relying on previously leaked symbols (wildcard exports or internal structs/functions) may break; this is expected by the cleanup.
- `show_window_channel()` remains public; further narrowing is possible but currently constrained by cross-crate usage patterns and active parallel work in `src/main.rs`.

**Commits**
- No commits created.