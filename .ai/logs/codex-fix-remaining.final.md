**Changed files**
- `src/main.rs:308`  
  Kept `main()` as a single `include!("main_entry/app_run_setup.rs");` expression and removed the separate preflight include path that caused include-expression/scope issues.
- `src/main_entry/app_run_setup.rs:1`  
  Wrapped content as a block expression and inlined the preflight setup so referenced variables are in scope.
- `src/render_prompts/arg/render.rs:1`  
  Replaced impl-position include usage with direct merged impl methods to avoid include parse errors.
- `src/app_impl/startup.rs`  
  Fixed split-induced brace/scope issues and visibility needed by cross-module calls.
- `src/config/editor/tests/part_01.rs:471`  
  Removed dangling trailing doc comments with no following item.
- `src/config/config_tests/part_01.rs:407`  
  Removed orphan trailing `#[test]`.
- `src/config/config_tests/part_03.rs:386`  
  Removed orphan trailing `#[test]`.
- `src/config/config_tests/part_04.rs:1`  
  Added missing `#[test]` before the first test function.
- `src/menu_bar/part_001.rs:192`  
  Fixed test module path to `../menu_bar_tests.rs`.
- `src/menu_executor/part_001.rs:75`  
  Fixed test module path to `../menu_executor_tests.rs`.
- `src/scriptlets/part_003.rs:353`  
  Fixed test module path to `../scriptlet_tests.rs`.
- `src/terminal/alacritty/handle_creation.rs:154`  
  Changed `detect_shell` visibility to `pub(crate)` for tests.
- `src/terminal/pty/lifecycle.rs:116`  
  Changed `detect_shell` visibility to `pub(crate)` for tests.

**How to test**
1. `cargo check 2>&1 | grep -v src/protocol | grep -v src/actions | grep -v src/prompts | grep -v src/components | rg "error\\[|error:" -n`  
   Result: no matches (remaining non-excluded compile errors resolved for requested check loop).
2. `cargo check`  
   Result: passes.
3. `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`  
   Result: fails due broad existing clippy/test failures mostly in excluded/other-agent areas (`src/actions/*`, `src/prompts/*`, `src/components/*`, plus general warning debt).

**Risks / known gaps**
- Full verification gate is still red because the repo currently has many nontrivial clippy/test failures outside this assignment scope (and ongoing parallel-agent edits).
- I did not resolve excluded-area failures by design.
- No commits were made.