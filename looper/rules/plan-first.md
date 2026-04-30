# Plan-First Before Arming

The gate between "I'm going AFK for N hours" and any file edit is a written plan the user approves. The plan exists so the user can redirect the loop's shape *before* the scaffolding lands — once stories.md is committed and the cron is armed, changing the shape is expensive. Produce the plan; get approval; then execute it mechanically.

## When this rule fires

- "go AFK for N hours", "run overnight", "audit unattended", "loop through these stories"
- Any request that would result in `CronCreate` or the first `ScheduleWakeup` of a new audit run
- Any request to set up an audit loop in a **new** project (no prior `audits/afk/` directory)

Non-triggers: continuing an armed run (no re-plan needed — scope.md is the plan), one-off tasks under ~30 min (loop overhead isn't worth it), resuming after a kill-switch pause (rearm only — the prior plan still stands).

## The plan must cover seven sections

Reject any plan document that skips one of these. They are the sections the grader reads when deciding "is this actually ready to arm?"

1. **Context** — one paragraph on what the user asked for, and the core safety principle (bounded autonomy, file-based scope). No ambition creep — the plan is the scope.
2. **Driver choice** — cron vs ScheduleWakeup, with the rationale tied to budget (≤2h ScheduleWakeup, ≥3h cron — see [cadence.md](cadence.md)). Name the specific trade-off the user is accepting ("dies if agent crashes" for ScheduleWakeup; "fixed cadence" for cron).
3. **Pre-flight** — `git status --porcelain` result. Any pre-existing drift (rustfmt, uncommitted experiments, user or other-agent work) must be named and classified. The loop may run with dirty external paths present, but it must ignore them and use explicit path-scoped staging. Never sweep unknown edits into audit commits.
4. **Directory layout + files to create** — exact paths. `audits/afk/{scope,stories,log}.md`, `.gitignore` append for `.audit-pause`, `promote-tool-gaps.sh` copy. See [../checklists/arming.md](../checklists/arming.md).
5. **Story seed** — 5–15 concrete, falsifiable stories. Each: `slug: one-line verifiable behavior`. If the budget implies more passes than the seed list holds, name the generation policy (tool-gap queue first, then surface-rotation).
6. **Loop protocol + commit template** — either inline the STEPS 0–9 from [../tick-prompt.txt](../tick-prompt.txt) or reference it. The commit subject-verb taxonomy MUST appear (see [discipline.md](discipline.md)).
7. **Verification gate** — one supervised pass before the user leaves. [arming.md §8](../checklists/arming.md).

A plan with only 1–4 is a checklist, not a plan. A plan with 1–7 can be approved and executed.

## Where the plan document lives

```
~/.claude/plans/<auto-slug>.md   # ephemeral, not committed
```

The plan file is alignment scaffolding, not audit trail. The audit trail is `git log` + `audits/afk/log.md`. Do not commit the plan file into the repo — it will decay as scope.md becomes the truth.

If the plan needs to be referenced later (retrospective, postmortem), copy the relevant excerpts into `audits/afk/log.md` under the run's section header. The plan file itself can be discarded.

## What blocks approval

Reject-worthy in a plan (ask the user to amend, don't execute):

- **No pre-flight.** Working-tree state unexamined; any commit could sweep in user work.
- **Seed list < 5 stories and no generation policy.** Loop enters drain mode on tick #2.
- **Budget inconsistent with driver.** A 90-min budget with cron (overkill; use ScheduleWakeup). A 7-hour budget with ScheduleWakeup (agent will crash; requires cron).
- **Missing falsifier discipline.** The plan doesn't say that every pass commit will carry `Falsifier:`. The loop will ship untestable pins.
- **Forbidden actions undefined.** No explicit list of "don't do X" (push, force-push, rm -rf, cargo clean, disabling hooks). Scope.md later can't be written without this.
- **No kill-switch line.** Plan doesn't name `.audit-pause` — user has no emergency stop.
- **"Supervised pass" skipped.** Plan says "arm and leave". Don't. The supervised pass catches 80% of arming bugs (wrong minute offset, bad epoch math, session dead).

A plan that drifts from these is not ready. Say "plan needs X before we arm" — don't soften by arming a partial plan.

## Plan-first anti-patterns

These are the failure modes plan-first exists to prevent. Each one was observed in a prior run.

- **Arm-then-plan.** `CronCreate` fires before scope.md exists. First tick fails; cron keeps firing against a broken setup.
- **Seed-list drift.** Stories added ad-hoc after arming instead of seeded upfront. Surface rotation and bug-yield floor can't be computed on a moving backlog.
- **Pre-flight skip.** Rustfmt-only drift swept into first audit commit → revert is now ambiguous ("was this audit, or was this rustfmt?"). Dirty external files are allowed, but only when they are named and left unstaged.
- **Supervised-pass skip.** Loop arms, user leaves, first tick has a trivial discipline bug (e.g., commit subject missing `Run <N>`), and every subsequent tick inherits the bug. User returns to 30 malformed commits.
- **Plan-only-in-the-head.** Agent claims alignment but no written document exists. User has nothing to redirect — asks for changes mid-session, agent can't back out because the scaffolding is already in `main`.

## Executing the plan

Once approved:

1. Pre-flight coexistence snapshot — record dirty paths and ownership. Optional cleanup commits require explicit user approval; otherwise leave external paths unstaged and proceed with path-scoped commits.
2. Scaffolding commit — `audit(scaffolding): set up audit loop at audits/afk/` (the one commit that lands scope.md, stories.md, log.md, .gitignore, promote-tool-gaps.sh).
3. Run header commit — `audit(scheduler): Run <N> started — budget <M> min` (epoch math verified per [budget-edge.md](budget-edge.md)).
4. Arm — `CronCreate` (cron variant) or first `ScheduleWakeup` (self-paced variant), capture id.
5. Cron-id commit — `audit(scheduler): Run <N> cron id <id>` (cron variant only).
6. Supervised tick — run exactly one pass with the user watching. User confirms commit subject, body fields, diff scope, falsifier.
7. Hand over — user leaves. Loop takes over.

Steps 1–7 are mechanical once the plan is approved. Deviating from the sequence (skipping the cron-id commit, skipping supervised tick) is a [plan-first anti-pattern](#plan-first-anti-patterns).

## Re-planning mid-run

Don't. If the plan needs to change (budget extension, scope change, new forbidden action), the correct sequence is:

1. `CronDelete <id>` (pause the loop).
2. Append scheduler-stop entry to `log.md` naming the reason.
3. Commit `audit(scheduler): Run <N> paused for re-plan; CronDelete <id>`.
4. Produce a new plan document (same seven sections).
5. Open Run `<N+1>` with the new plan — old scope.md and stories.md carry over or are rewritten as the plan directs.

Mid-run scope edits under a live cron are forbidden — see [../README.md §Changing these rules](../README.md#changing-these-rules).
