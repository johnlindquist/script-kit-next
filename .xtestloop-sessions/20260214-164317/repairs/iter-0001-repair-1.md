Confirmed — only references outside `src/` are in xorloop session logs. These two empty stubs have zero callers in the actual source code. They're dead code from a previous refactor.

No need for Oracle here — the gate failure is trivially diagnosable. Here's the swarm task list:

---

## SWARM TASK LIST

### Task 1: fix-dead-cursor-fns
**Scope:** `src/platform/cursor.rs`
**Task:**
Delete the two unused empty stub functions at lines 321-327:
- `pub fn claim_cursor_pointer() {}` (line 325)
- `pub fn apply_default_cursor() {}` (line 327)
- Also delete the comment block at lines 321-323 ("Layer 2 — Mouse-move cursor coordination")

These functions have zero callers anywhere in `src/`. They are leftover stubs from a previous cursor refactor. Clippy with `-D warnings` promotes `dead_code` warnings to hard errors, which is the sole cause of the current gate failure.

**Verification:** `cargo clippy --all-targets -- -D warnings` must complete with zero errors.

---

That's it — only 1 task needed. The gate output shows exactly 2 errors, both in the same file, both dead code.

**NEXT_AREA:** `src/theme` — add `tests.rs` for the highest-churn untested module (77 unwrap/expect calls to audit and harden).
