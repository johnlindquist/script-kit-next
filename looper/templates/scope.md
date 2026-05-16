# AFK Audit Scope — <run id>

This file is the canonical rule source for an active audit run. The tick prompt reads this file at the start of every tick (Step 1 onward). Editing it mid-run is ambiguous — prefer to end the current run and open a new one (see looper/rules/run-transitions.md).

## Allowed actions

- Read files, edit files within project root
- git add, git commit on main branch
- Project's own link check (e.g. `source checks`, `source search`, `source lookup`)
- CronList, CronCreate, CronDelete (for loop's own scheduling)

## Worktree coexistence

Dirty files may appear in `git status --porcelain` because the user or another
agent may be working concurrently. They do not block the loop.


- `.claude/scheduled_tasks.lock`
- `.audit-pause` (if present, Step 0 triggers before Step 2)


- Snapshot dirty paths at the start of each tick.
- Treat pre-existing dirty paths as external unless the chosen story explicitly owns them.
- Prefer stories whose write set does not overlap external dirty paths.
- Commit only this tick's owned files with explicit pathspecs.
- Never use `git add .` / `git add -A` in the loop.

## Forbidden actions

- git push, git push --force
- git reset --hard, git rebase, git checkout -- (destructive restore)
- git commit --amend (violates reversibility invariant)
- rm -rf, cargo clean
- Edits outside project root (including ~/.claude/skills/ during active run — see feedback_no_skills_during_afk_loop)
- Writes to ~/.config, ~/.ssh, .git/
- Disabling hooks (--no-verify, --no-gpg-sign)
- Killing a dev session the user started (only restart sessions the loop started)

## Budget + buffer


## Scheduling cutoff

1. CronDelete the loop's cron
2. Appends scheduler-stop entry to log.md

No "one more pass" exceptions. See looper/rules/budget-edge.md.

## Fix size policy


- If a fix feels like speculative cleanup unrelated to the story → pause, log `deferred`, record the finding.
- Cross-cutting fixes that the bug actually requires are fair game — three similar edits is better than a premature abstraction, but a genuine shared helper across N sites is worth extracting.
- Never bundle two stories into one pass. If the fix for story A reveals story B, commit A then log B as a new `[ ]` item for the next pass.

## Verification gate


## Subject-verb discipline (enforced every pass)


## Discipline thresholds (scope-tunable)




## Per-commit receipt fields (universal)





## Required log-entry fields (per pass)


- Surface
- Story
- Commit (sha-pending in Phase 1; backfilled in Phase 2)
- Files touched (count + list if > 1)
- Proof
- Falsifier
- Notes (short; names threshold triggers, deferred items, etc.)

## Kill switch

`touch .audit-pause` at project root. Next tick reads Step 0 and exits. Does NOT CronDelete — user may be inspecting. Remove with `rm .audit-pause` to resume.

## Post-pass close-up (always-on-top apps)


```
session.sh send --keys Escape
session.sh rpc hide
```
