Implemented prompt-scope compile fixes so `src/prompts` no longer emits compile **errors**.

**Changed files**
- `src/prompts/div/mod.rs:36`  
  Added public re-exports for `ContainerOptions` and `ContainerPadding`.
- `src/prompts/path/mod.rs:33`  
  Re-export `PathPrompt` from `types` (`pub use types::PathPrompt` path).
- `src/prompts/select/types.rs:4`  
  Made `SelectChoiceIndex` and `ChoiceDisplayMetadata` fields `pub(super)` so sibling prompt modules can access them.
- `src/prompts/select/render.rs:166`  
  Made shortcut trailing content conversion explicit (`SharedString::from(shortcut)`).
- `src/prompts/markdown/mod.rs:39`  
  Changed internal markdown helper bindings to non-re-export `use` bindings.
- `src/prompts/markdown/code_table.rs:25`  
  Visibility adjusted to module-parent scope (`pub(super)`).
- `src/prompts/markdown/inline_render.rs:87`  
  Visibility adjusted to module-parent scope (`pub(super)`).
- `src/prompts/markdown/parse.rs:3`  
  Visibility adjusted to module-parent scope (`pub(super)`).
- `src/prompts/markdown/render_blocks.rs:3`  
  Visibility adjusted to module-parent scope (`pub(super)`).

**Verification run**
- `cargo check 2>&1 | grep src/prompts`  
  Result: prompt warnings only, no prompt errors.
- `cargo check > /tmp/cargo-check-final-prompts.out 2>&1; rg -n "error\\[|error:" /tmp/cargo-check-final-prompts.out | rg "src/prompts"`  
  Result: no matches (no prompt error lines).

**Full gate status (attempted, blocked by unrelated repo issues)**
- `cargo check` fails outside prompt scope (`src/app_impl/prompt_ai.rs:315` unexpected closing delimiter).
- `cargo clippy --all-targets -- -D warnings` fails due many pre-existing non-prompt warnings/errors across repo.
- `cargo test` fails due pre-existing parse errors in `src/config/config_tests/*` and `src/config/editor/tests/*`.

**Risks / known gaps**
- `src/prompts` still has warnings (existing glob-import/private-interface style warnings), but prompt compile errors are resolved.
- Full repo verification gate cannot pass until unrelated non-prompt build/test issues are fixed.

Commits made: none.