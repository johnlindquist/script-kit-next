# SHA Backfill — The Two-Phase Commit Dance

Each pass's log entry names the commit SHA that landed the work. But the SHA doesn't exist until `git commit` returns. Two-phase dance resolves the chicken-and-egg.

## The dance

### Phase 1: main pass commit

Includes, in one commit:

- scope.md (if touched)
- stories.md (outcome marker flipped `[ ]` → `[x]` or `[!]` or `[?]`)
- code/test files modified for the fix/pin/extension
- log.md with the pass entry — except the `Commit:` field contains the literal placeholder `<sha-pending>`

Subject: `Prompt: <Verb> <imperative>` per [discipline.md](discipline.md).

### Phase 2: SHA backfill commit

Immediately after Phase 1:

```bash
SHORT_SHA=$(git log -1 --format=%h)
# Edit log.md: replace the most recent "<sha-pending>" with "$SHORT_SHA"
git add audits/afk/log.md
git commit -m "audit(log): backfill Run <N> Pass #<K> sha $SHORT_SHA"
```

The backfill is its own SHA. The log now references a committed-and-final SHA.

## Why NOT amend

Amending Phase 1 would rewrite the main commit's SHA — which is the SHA you would then need to rewrite into log.md, requiring another amend… etc. You enter a fixed-point loop.

Fundamentally: scope.md forbids amending a published commit. Each pass getting its own SHA is how passes stay individually revertable. An amend couples the log update to the code change — `git revert` on that SHA would un-flip the story marker but not the log entry (or vice versa), depending on which amended version you reverted.

Two commits per pass is the canonical form.

## Why NOT write the SHA first

Can't — git computes the SHA from tree + parent + author + message + time. The message embeds the log.md content, which would have to embed the SHA. Impossible loop.

## Backfill is NOT a pass

Backfill commits have the `audit(log):` prefix. They:

- **do not** consume a story slot
- **do not** need a `Prompt:` verb
- **do not** trigger pin-cap or bug-yield floor checks
- **do not** require a Falsifier
- **do not** increment pass numbering (`#<K>` stays the same as the pass they back-reference)

The `audit(log):` prefix is the single-commit signal: "loop bookkeeping, not verification work".

## Exceptions (single-commit passes)

Some "passes" are log-only and never had a separate phase:

- Scheduler-stop (Step 1 or Step 9) — one commit, prefix `audit(scheduler): …`, no backfill
- Tool-gap promotion (Step 3) — one commit, prefix `audit(backlog): …`, no backfill
- Path-overlap abort (Step 2 dirty-path collision) — one commit, prefix `audit(scheduler): path overlap blocked …`, no backfill

These commits ARE the log commit, so there's nothing to backfill. They don't count as passes.

## Log entry format (before and after backfill)

Before (Phase 1):

```markdown
## Run 6 — Pass #3 — 2026-04-18T13:47:00Z

- Surface: stdin/protocol-router
- Story: tool-system-control-verbs-unwired
- Outcome: fix-committed
- Commit: <sha-pending>
- Files: 3 changed, +45/−0
- Proof: session.sh rpc checkAccessibility → accessibilityStatus {granted: true, requestId: "r1"} within 120ms
- Falsifier: if the receipt had returned null or timed out >2s
- Notes: one of 6 unwired verbs; see earlier audit
```

After (Phase 2):

```markdown
- Commit: 730bd3c02
```

(All other fields unchanged.)

The backfill commit's diff is one line: the placeholder → short SHA. Keeping diffs minimal makes the audit trail clean.

## If the backfill fails

If Phase 2's `git commit` fails (hook, pre-commit check, etc.), do not panic:

1. Investigate and fix the hook issue.
2. Re-stage log.md with the SHA already written.
3. Commit normally. The SHA in log.md is already correct; you're just landing it.

Never `--amend` Phase 1 to fold in the backfill — that violates the no-amend invariant.
