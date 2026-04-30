---
name: looper
description: Canonical rules and artifacts for bounded autonomous audit loops. Durable "go AFK for N hours" pattern — scope/stories/log scaffolding, per-tick protocol, cache-aware cadence, cron + ScheduleWakeup drivers, budget-edge self-delete, subject-verb discipline, SHA backfill. Use when the user asks for a continuous audit, AFK loop, overnight run, or multi-hour auto-commit session. See README.md for the link map; tick-prompt.txt for the parameterized prompt.
---

# Looper

Rule source for bounded autonomous audit loops. See [README.md](README.md) for the full link map.

## When to use

- "Go AFK for N hours" / "run overnight" / "continuously audit and commit"
- Bounded autonomy: commits to `main`, time budget, verifiable stop conditions, greppable trail
- Any codebase with an agentic-testing RPC surface (state receipts > screenshots)

## Quick index

- [rules/principles.md](rules/principles.md) — the five invariants
- [rules/discipline.md](rules/discipline.md) — subject verbs + per-commit receipt fields
- [rules/cadence.md](rules/cadence.md) — cache TTL math + cron minute offsets
- [rules/budget-edge.md](rules/budget-edge.md) — deadline/buffer/self-delete math
- [rules/attacker-mode.md](rules/attacker-mode.md) — every-4th-pass adversarial sweep
- [rules/sha-backfill.md](rules/sha-backfill.md) — two-phase commit dance
- [rules/scheduler-stop.md](rules/scheduler-stop.md) — end-of-run log/commit format
- [rules/governance.md](rules/governance.md) — optional layered toggles
- [rules/run-transitions.md](rules/run-transitions.md) — Run N → Run N+1
- [templates/scope.md](templates/scope.md) — copy → `audits/afk/scope.md`
- [templates/stories.md](templates/stories.md) — copy → `audits/afk/stories.md`
- [templates/log.md](templates/log.md) — copy → `audits/afk/log.md`
- [tick-prompt.txt](tick-prompt.txt) — canonical per-tick prompt (parameterized)
- [scripts/arm-loop.sh](scripts/arm-loop.sh) — compute epochs, emit tick prompt
- [scripts/epoch-check.sh](scripts/epoch-check.sh) — drop-in buffer guard
- [scripts/promote-tool-gaps.sh](scripts/promote-tool-gaps.sh) — idempotent backlog promoter
- [checklists/arming.md](checklists/arming.md) — pre-flight before the user leaves

## Arming protocol (brief)

1. Confirm scaffolding. Copy `scripts/promote-tool-gaps.sh` into `audits/afk/` if missing.
2. Agree on budget minutes with user.
3. Append run header to `log.md`.
4. Round-trip verify deadline + buffer epochs (see [rules/budget-edge.md](rules/budget-edge.md)).
5. Pick safe minute offset (never `:00` / `:30` — see [rules/cadence.md](rules/cadence.md)).
6. `CronCreate` with the filled [tick-prompt.txt](tick-prompt.txt).
7. Run one supervised tick. User confirms. Cron takes over.

## Meta-rule: skill creation is forbidden during an active loop

Creating or editing `~/.claude/skills/` entries while a cron loop is firing interrupts the loop. Before running `/create-skill` or editing a skill, check:

```bash
# Active run header within deadline?
grep -E '^## Run [0-9]+ — started' audits/afk/log.md | tail -1
# Any cron in this project?
# (call CronList and filter by project)
```

If a run is active and in budget, defer skill changes until the run ends. This rule is in memory (`feedback_no_skills_during_afk_loop.md`) and repeated here because the rule matters for the loop itself.
