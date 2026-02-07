Implemented the script creation UX polish in these files:

- `src/prompts/template.rs:26`  
  Added richer template inputs (`label/group/required`), inline validation state, and friendly parsing helpers.
- `src/prompts/template.rs:232`  
  Added input validation (`required` + slug-style naming rules) and blocked submit until valid.
- `src/prompts/template.rs:504`  
  Updated render UX with grouped sections, required `*` markers, inline errors, and naming guidance text.
- `src/prompts/template.rs:678`  
  Added tests for human-friendly labels/groups and slug validation behavior.
- `src/script_creation.rs:106`  
  Improved new script starter with a structured “Template Guide” and clearer first prompt.
- `src/script_creation.rs:139`  
  Improved new scriptlet bundle starter with YAML frontmatter and guidance comments.
- `src/script_creation.rs:375`  
  Updated template tests to cover new starter content.
- `src/builtins.rs:917`  
  Made “New” entries more discoverable/prominent:
  - `New Script (Template)` with stronger keywords/icon
  - `New Scriptlet Bundle` with frontmatter/scriptlet-focused keywords
- `src/builtins.rs:1585`  
  Added discoverability test coverage for those new entries.

Validation run:

- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ blocked by unrelated workspace issues (mainly action validation test files + clippy items outside scope)
- `cargo test` ❌ blocked by unrelated unresolved chat imports in `src/prompts/chat.rs`
- Stdin protocol checks ✅  
  Ran `show` and `setFilter` via stdin JSON; logs confirm parsing/processing and “new script” search flow.

How to test:

1. Build:
   ```bash
   cargo check
   ```
2. Protocol smoke:
   ```bash
   timeout 10 bash -c '(printf "%s\n" "{\"type\":\"show\"}" "{\"type\":\"setFilter\",\"text\":\"new script\"}") | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'
   ```
3. Manual UX check in app:
   1. Open main menu, search `new script`.
   2. Run `New Script (Template)`.
   3. Verify grouped fields, required markers, placeholders, and inline validation.
   4. Enter invalid name like `My Cool Script` and confirm validation message.
   5. Run `New Scriptlet Bundle` and confirm generated file starts with YAML frontmatter guidance.

Risks / known gaps:

- Full `clippy -D warnings` and `cargo test` are currently blocked by unrelated in-progress changes outside this scope.
- `src/builtins.rs` has concurrent broader edits in this branch; my functional changes are limited to New-entry discoverability + its test.