**Changed files**
- `src/main.rs`: removed the duplicate test module declaration `#[cfg(test)] mod keyboard_routing_tests;` (the module is already declared inside `include!("app_impl/mod.rs")`, which caused same-scope duplication).

**How to test**
1. Confirm only one declaration remains:
   - `rg -n "mod\\s+keyboard_routing_tests" src/main.rs src/app_impl/mod.rs src/keyboard_routing_tests.rs`
   - Expected: only `src/app_impl/mod.rs:49`.
2. Run lint gate:
   - `cargo clippy --all-targets -- -D warnings`

**Verification results**
- Duplicate declaration check passed (single declaration found).
- `cargo clippy --all-targets -- -D warnings` still fails, but on unrelated parallel-agent issues (for example `src/actions/tests/*` doc-comment/import errors and `src/app_actions/tests.rs` duplicate-mod), not on `keyboard_routing_tests`.

**Risks / known gaps**
- Workspace-wide clippy is currently red due unrelated files being edited in parallel, so I could not produce a fully green global `clippy` run from this branch state.