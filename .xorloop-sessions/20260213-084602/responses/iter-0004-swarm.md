The test compilation errors are all pre-existing (clipboard_history, unrelated modules) — not from our changes. Production code compiles and passes clippy cleanly.

---

## Swarm Results Summary

Both security hardening tasks completed successfully:

### `agent-exec-hardening` — **Done** (commit `7017b521`)
- **Environment allowlist scrub**: `env_clear()` + safe allowlist applied to all 3 spawn functions (`execute_agent`, `explain_agent`, `dry_run_agent`). Frontmatter `_env` values applied on top of scrubbed base.
- **Agent path canonicalization**: Validates agent path resolves to a real `.md` file, rejects path traversal.
- **Argv hardening**: Rejects reserved mdflow flags, control chars/newlines in values, inserts `--` before positional args.

### `frontmatter-sanitize-fields` — **Done** (commit `fd73eb0c`)
- `sanitize_cwd()`: Rejects empty, control chars, NUL, `..` components
- `sanitize_command()`: Rejects empty, control chars, NUL bytes
- `is_valid_env_key()`: Rejects empty, NUL, `=`, control chars
- `is_valid_env_val()`: Rejects NUL bytes
- Raw frontmatter map preserved; only extracted fields sanitized

### Verification
- `cargo check` — passes
- `cargo clippy -- -D warnings` — passes (pre-existing test compilation issues in unrelated modules are not from these changes)
- Both commits already pushed by workers
