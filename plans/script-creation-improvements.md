# Script Creation Flow Audit (`src/script_creation.rs`)

## Scope
- Primary file: `src/script_creation.rs`
- Adjacent flow checked for UX impact: `src/app_execute.rs` (how creation is triggered) and `src/builtins.rs` (user-facing labels)

## Current behavior snapshot
1. Input name is sanitized (`sanitize_name`) and converted to `*.ts` or `*.md`.
2. Target directory is created with `fs::create_dir_all`.
3. Existence is checked via `path.exists()`.
4. Template content is generated and written with `fs::write`.
5. File is opened by spawning configured editor command.

This is straightforward, but there are several UX and robustness gaps.

## Findings

### 1) New script creation is collision-prone in the default flow (High)
- Evidence: `src/app_execute.rs:773-776` calls `create_new_script("untitled")` every time.
- `src/script_creation.rs:208-210` then hard-fails with "Script already exists" after the first run.
- Result: the built-in "New Script (Template)" feels broken on second use unless user manually renames the first file.

Improvement:
- Add unique-name behavior (`untitled.ts`, `untitled-1.ts`, `untitled-2.ts`) or interactive naming before create.

### 2) Editor command handling fails for commands with flags/paths (High)
- Evidence: `open_in_editor` uses `Command::new(&editor).arg(path)` (`src/script_creation.rs:304`).
- If config editor is `"code -r"` or an app path with embedded args, spawn will fail because the full string is treated as executable.

Improvement:
- Parse editor command into executable + args (e.g., `shlex/shell-words`) and append file path.
- Add clear error context showing parsed command parts.

### 3) File creation has TOCTOU race risk and weaker overwrite safety (Medium)
- Evidence: explicit `exists()` check then `fs::write()` in `create_new_script` (`src/script_creation.rs:207-215`) and `create_new_extension` (`src/script_creation.rs:262-270`).
- Between check and write, another process can create the file.

Improvement:
- Use `OpenOptions::new().write(true).create_new(true)` to make creation atomic.
- Keep user-friendly "already exists" error mapping.

### 4) Filename validation is incomplete for filesystem edge cases (Medium)
- Evidence: only "empty after sanitization" is rejected (`src/script_creation.rs:190-191`, `src/script_creation.rs:245-246`).
- Missing checks:
  - max filename length (255 bytes)
  - reserved names (important for cross-platform repos: `con`, `nul`, etc.)
  - accidental hidden files / trailing dots behavior

Improvement:
- Add `validate_filename` after sanitization with explicit, actionable error messages.
- Include both original and sanitized names in error context.

### 5) Template quality is functional but not optimized for fast authoring (Medium)
- Script template (`src/script_creation.rs:106-134`) is minimal and includes a long static guide comment.
- Extension template (`src/script_creation.rs:139-169`) has one bash block but limited real-world scriptlet patterns.

Improvements:
- Offer starter variants (quick prompt, list picker, AI/chat, background task).
- Include stronger metadata defaults (`description`, `author`, optional `shortcut` comment stub).
- For bundles, include 2-3 runnable scriptlet examples with different tool types and a clearer "edit me first" marker.

### 6) Integration tests are not validating the production create functions (Medium)
- Evidence: `test_create_script_integration` and `test_create_extension_integration` manually write files (`src/script_creation.rs:436-479`) instead of calling `create_new_script`/`create_new_extension`.
- This misses behavior in directory creation, collision detection, and error mapping.

Improvements:
- Refactor path resolution so tests can inject temp dirs, then test real APIs end-to-end.
- Add explicit tests for duplicate names, long names, atomic create errors, and editor command parsing.

### 7) Error UX can better guide users after partial success (Low)
- `create_*` errors are clear, but "created file, editor failed" recovery remains generic at call site.

Improvement:
- Return richer creation result info (path, sanitized_name, maybe template_kind).
- Provide user follow-up actions: reveal in Finder, copy path, retry open with fallback editor.

## Recommended implementation plan

### Phase 1 (low-risk, high impact)
1. Add unique-name generation helper for script/extension creation.
2. Switch to atomic `create_new` file writes.
3. Add filename validation (length + reserved names).
4. Improve `open_in_editor` to support editor args safely.
5. Expand tests to cover real `create_new_*` behavior (not manual writes).

### Phase 2 (authoring UX)
1. Add template presets (quick, list, async/background).
2. Improve extension template with practical multi-scriptlet examples.
3. Add better inline "next steps" comments (run, debug, metadata edits).

### Phase 3 (flow-level polish, outside this file)
1. Replace hardcoded `"untitled"` creation path in `src/app_execute.rs:775` with interactive naming or auto-suffix strategy.
2. Keep window open on recoverable creation failures and show actionable options.

## Proposed tests (naming style aligned with repo conventions)
- `test_create_new_script_generates_unique_name_when_base_exists`
- `test_create_new_script_fails_when_name_exceeds_filename_limit`
- `test_create_new_script_uses_atomic_create_new`
- `test_open_in_editor_parses_editor_with_flags`
- `test_create_new_extension_generates_unique_name_when_base_exists`
- `test_create_new_script_integration_uses_real_api_with_temp_dir`

## Risks / tradeoffs
- Auto-suffix naming changes current behavior; some users may prefer explicit failures.
- Parsing editor commands must avoid shell-injection pitfalls; parse tokens without invoking shell.
- Reserved-name validation may reject existing niche workflows unless made platform-aware.
