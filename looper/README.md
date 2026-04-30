# Looper — Bounded Autonomous Audit Loops

Durable rules and artifacts for AFK audit loops: the user walks away; the agent picks stories from a backlog, verifies them via agentic testing, ships minimal fixes on `main`, and commits every action with a greppable receipt. When the user returns, `git log` + `audits/afk/log.md` tell the whole story.

This directory is the canonical source. The `~/.claude/skills/afk-audit-loop/` and `~/.claude/skills/cron-audit-loop/` entries are thin pointers that redirect here — editing the rules means editing files in this repo, not files in `~`.

## Read this first: five invariants

Everything below follows from these. Break one and the loop stops being auditable.

1. **Every rule is grep-auditable.** No private agent state. The run's truth lives in `git log --oneline` + `audits/afk/log.md` + `audits/afk/stories.md`. If you can't grep it, it isn't a rule.
2. **Every commit is reversible.** One pass = one SHA. No amend. No force push. Each pass can be `git revert`ed in isolation; backfill commits carry the same property.
3. **Every claim is falsifiable.** Every pass commit names a `Falsifier:` line — one concrete receipt state that would have proven the pass wrong. Rationales with no falsifier are not passes.
4. **Every rule is tunable per run.** Thresholds live in `audits/afk/scope.md` (pin cap, bug-yield floor, surface breadth, attacker cadence). The tick prompt reads them; the agent does not invent numbers.
5. **Every budget overshoot is a bug.** Deadline − 20 min = buffer. When `now ≥ buffer`, the cron self-deletes unconditionally. "Just one more pass" is the anti-pattern the buffer exists to prevent.

## File map

```
looper/
├── README.md                   # this file — entry point + link map
├── SKILL.md                    # re-export for Claude's skill discovery
├── tick-prompt.txt             # canonical per-tick prompt (parameterized)
├── rules/
│   ├── principles.md           # the five invariants above, expanded
│   ├── plan-first.md           # plan document → approval → execute (before any arming)
│   ├── discipline.md           # subject-verb taxonomy + per-commit receipt fields
│   ├── cadence.md              # cache TTL math + cron minute offsets
│   ├── budget-edge.md          # deadline/buffer/self-delete math
│   ├── attacker-mode.md        # every-4th-pass adversarial sweep
│   ├── sha-backfill.md         # two-phase commit dance
│   ├── scheduler-stop.md       # end-of-run log/commit format
│   ├── governance.md           # optional layered toggles (breadth, rotation, caps)
│   └── run-transitions.md      # ending Run N, starting Run N+1
├── templates/
│   ├── scope.md                # copy → audits/afk/scope.md
│   ├── stories.md              # copy → audits/afk/stories.md
│   └── log.md                  # copy → audits/afk/log.md
├── scripts/
│   ├── arm-loop.sh             # compute epochs, emit the tick prompt
│   ├── epoch-check.sh          # drop-in buffer guard
│   └── promote-tool-gaps.sh    # idempotent backlog promotion (copy into audits/afk/)
└── checklists/
    └── arming.md               # pre-flight before the user leaves
```

## Which driver — cron vs ScheduleWakeup

| Run length | Driver | Rationale |
|---|---|---|
| ≤ 2 hours | `ScheduleWakeup` (dynamic) | Self-paced, cache-warm between ticks, dies if agent crashes (acceptable at this length). |
| ≥ 3 hours | `CronCreate` | Survives agent-process restarts. Fixed cadence means picking a minute offset that avoids scheduler collisions (see [rules/cadence.md](rules/cadence.md)). |
| Mixed | Start with cron | Simpler reasoning; cron cadence matches typical pass duration anyway. |

## Arming a new run (condensed)

1. Confirm scaffolding exists: `audits/afk/{scope,stories,log}.md`, `.audit-pause` in `.gitignore`, `promote-tool-gaps.sh` in `audits/afk/` (copy from `looper/scripts/`). See [checklists/arming.md](checklists/arming.md).
2. Agree on budget minutes with the user.
3. Append the run header to `log.md`: `## Run <N> — started <ISO>Z — budget <mins> min — deadline <ISO>Z — buffer-cutoff <ISO>Z`.
4. Round-trip verify deadline/buffer epochs via `date -u -jf "%Y-%m-%dT%H:%M:%SZ" … +%s`. See [rules/budget-edge.md](rules/budget-edge.md).
5. Pick a safe minute offset (never `:00` / `:30`). See [rules/cadence.md](rules/cadence.md).
6. `CronCreate` the per-tick prompt (copy from `tick-prompt.txt`, fill placeholders). The cron id does not need to be baked into the prompt — the prompt reads the id from `log.md` and confirms via `CronList`.
7. Run **one supervised tick** with the user watching. Only after that does the cron take over.

## Per-tick protocol (condensed)

Every tick runs this sequence exactly. See `tick-prompt.txt` for the full parameterized text.

0. **Kill switch** — `.audit-pause` exists → exit without `CronDelete` (user may be inspecting).
1. **Budget check** — `now_epoch ≥ buffer_epoch` → `CronDelete`, scheduler-stop entry, commit, exit.
2. **Worktree coexistence check** — snapshot `git status --porcelain`, ignore external dirty paths, and stage only this tick's owned files.
3. **Session + tool-gap** — agentic session alive; `promote-tool-gaps.sh` drains prose-only `tool-*` slugs into backlog items.
4. **Discipline preflight** — pin cap, bug-yield floor, surface rotation. All three honored before story pick.
5. **Story pick** — top-down first `[ ]`, `tool-*` slugs first, every 4th pass attacker mode.
6. **Verify / fix / re-verify** — one fix attempt max; still red → `failed`, move on.
7. **Commit** — one pass = one commit. Subject verb matches the work (see [rules/discipline.md](rules/discipline.md)). Body has user story, pass number, proof, falsifier.
8. **SHA backfill** — replace `<sha-pending>` in log.md with the just-landed short SHA; commit as `audit(log): backfill Run <N> Pass #<K> sha <short>`.
9. **Pre-emptive next-fire check** — if next scheduled fire ≥ buffer, self-delete now rather than waiting for the next tick to do it.

## Kill switch

`touch .audit-pause` at project root. The next tick reads step 0 and exits without rescheduling. The switch is gitignored; remove it (`rm .audit-pause`) to re-arm (but arm a new cron — the existing one self-deleted or is waiting).

## What "A+" means for this system

- **Nothing is whispered.** Every rule is in one of these files; no implicit norms.
- **Every verb is disciplined.** The subject verb on a pass commit is the single authoritative label for what happened; `Pin ≠ Verify ≠ Extend ≠ Fix`.
- **Every failure mode observed in prior runs has a named fix.** Budget overshoot, pin-only drift, surface clustering, skill-creation-during-loop interrupts, cron-id bake conflicts, amend-instead-of-backfill confusion — all have explicit rules.
- **The rules live with the code they govern.** Editing a rule is a normal PR; the audit trail of rule changes is the same audit trail as the codebase.

## Changing these rules

Edit the file in-place, commit as `looper: <imperative>`, and the next tick picks up the change. Do not branch or version these rules — the run-in-progress honors whatever `scope.md` and the linked rule files say at tick time.

If a rule change would affect an active run's discipline (e.g. lowering the pin cap mid-run), pause the cron first (`CronDelete`), land the rule change, append a scheduler-stop log entry noting the rule change, and arm a new run. Do not rewrite scope.md under a live cron.
