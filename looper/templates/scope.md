# AFK Audit Scope — <run id>

This file is the canonical rule source for an active audit run. The tick prompt reads this file at the start of every tick (Step 1 onward). Editing it mid-run is ambiguous — prefer to end the current run and open a new one (see looper/rules/run-transitions.md).

## Allowed actions

- Read files, edit files within project root
- Run: cargo check / cargo build / cargo test — always with ≥30s timeout (see memory: feedback_timeout_commands)
- Run: bun / npm / pnpm / yarn scripts — always with timeout
- Run: bash scripts/agentic/session.sh (start/send/rpc/stop/status)
- git add, git commit on main branch
- Project's own link check (e.g. `lat check`, `lat search`, `lat locate`)
- CronList, CronCreate, CronDelete (for loop's own scheduling)

## Worktree coexistence

Dirty files may appear in `git status --porcelain` because the user or another
agent may be working concurrently. They do not block the loop.

Files that are known scheduler artifacts:

- `.claude/scheduled_tasks.lock`
- `.audit-pause` (if present, Step 0 triggers before Step 2)

Rules:

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

- Total wall clock: <N> minutes from start timestamp in log.md header
- Buffer: deadline − 20 min (scope-tunable; never below 10 min)
- Per-pass timeout: 8 minutes. Over → log `failed`, move on.

## Scheduling cutoff

When `now_epoch >= buffer_epoch`, Step 1 of the tick unconditionally:
1. CronDelete the loop's cron
2. Appends scheduler-stop entry to log.md
3. Commits `audit(scheduler): Run <N> stop at buffer cutoff; CronDelete <id>`

No "one more pass" exceptions. See looper/rules/budget-edge.md.

## Fix size policy

No hard LOC or file cap. Trust judgment. But:

- If a fix feels like speculative cleanup unrelated to the story → pause, log `deferred`, record the finding.
- Cross-cutting fixes that the bug actually requires are fair game — three similar edits is better than a premature abstraction, but a genuine shared helper across N sites is worth extracting.
- Never bundle two stories into one pass. If the fix for story A reveals story B, commit A then log B as a new `[ ]` item for the next pass.

## Verification gate

Every Fix / Add / Extend MUST pass a re-run of the story's verification before commit. No green receipt → no commit. Commit the verification proof (state receipt fields or screenshot path) in the log entry's `Proof:` line.

## Subject-verb discipline (enforced every pass)

Every pass commit starts `Prompt: <verb>` where verb is one of: Fix, Add, Extend, Pin, Verify, Probe, Reproduce. See looper/rules/discipline.md. Non-pass commits use `audit(log):`, `audit(backlog):`, `audit(scheduler):`, or `looper:`.

## Discipline thresholds (scope-tunable)

Thresholds the tick prompt's Step 4 enforces:

- **Pin cap**: ≤7 `Prompt: Pin` commits in last 20 `Prompt:` commits
- **Bug-yield floor**: ≥1 `Prompt: Fix/Add/Extend` in last 5 `Prompt:` commits
- **Surface breadth**: ≥6 distinct Surface fields in last 10 log entries
- **Surface rotation**: next pass's surface MUST differ from previous pass's surface
- **Attacker cadence**: every 4th pass (#4, #8, #12, …) is attacker mode

Change a threshold by editing this line and committing as `looper: …`. Do not edit under a live cron.

## Per-commit receipt fields (universal)

Every `Prompt:` commit body includes:

- `User story: <slug> — <exact text from stories.md>`
- `Pass: #<K> of AFK audit Run <N> <START_ISO>`
- `Proof: <receipt fields or screenshot path>`
- `Falsifier: <concrete state that would have proven the pass wrong>`

Pin-only additional:
- `Refactor threat: <specific contributor action>`

Attacker-only additional:
- `Adversarial categories: <≥3 from menu>`
- `Actions: <count ≥ 20>`

## Required log-entry fields (per pass)

Every pass entry in log.md has:

- Surface
- Story
- Outcome: pass | fix-committed | skipped | failed | deferred
- Commit (sha-pending in Phase 1; backfilled in Phase 2)
- Files touched (count + list if > 1)
- Proof
- Falsifier
- Notes (short; names threshold triggers, deferred items, etc.)

## Kill switch

`touch .audit-pause` at project root. Next tick reads Step 0 and exits. Does NOT CronDelete — user may be inspecting. Remove with `rm .audit-pause` to resume.

## Post-pass close-up (always-on-top apps)

For apps whose main window is always-on-top (e.g. non-activating panels), every pass ends with:

```
session.sh send --keys Escape
session.sh rpc hide
session.sh rpc getState  # confirm windowVisible:false
```

This prevents the app from covering the user's workspace when they return. See memory: feedback_afk_close_app_when_done.
