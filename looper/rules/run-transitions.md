# Run Transitions — Ending Run N, Starting Run N+1

Clean transitions between runs are how the log stays auditable across days. Each run's section in `log.md` is append-only; the next run gets its own `## Run <N+1> — started …` header below.

## Ending a run

A run ends three ways:

1. **Natural buffer cutoff** — Step 1 or Step 9 self-deletes the cron; scheduler-stop entry committed. See [scheduler-stop.md](scheduler-stop.md).
2. **Kill switch** — `.audit-pause` at project root. The next tick reads Step 0 and exits without scheduling. The cron stays armed (user may be inspecting); the user must `CronDelete` explicitly when ready.
3. **User request** — user asks to stop; agent runs `CronDelete` + scheduler-stop entry + commit, same as natural cutoff.

Kill-switch case (#2) leaves the cron armed on purpose — the user might just be reading the log and want the loop to resume when they remove the switch. If the user wants a full stop, they'd say so; the agent should confirm before `CronDelete`ing on kill-switch alone.

## Confirming the prior run ended cleanly

Before arming a new run:

```bash
# 1. No active cron for this project
# (call CronList and check; if any remain, ask user whether to delete)

# 2. Working tree coexistence snapshot
git status --porcelain
# (dirty external paths are allowed; they must not overlap setup files)

# 3. Last run has a scheduler-stop entry
tail -30 audits/afk/log.md | grep -E '^## Run [0-9]+ — scheduler-stop'
```

If any of these fail:

- Active cron → resolve with user (delete or let finish)
- Dirty setup-path overlap → ask user or defer opening Run `<N+1>`; unrelated dirty paths are allowed and must remain unstaged
- No scheduler-stop entry → prior run didn't stop cleanly (process crash, force-quit, etc.); append a retrospective note to log.md describing what happened before opening Run <N+1>

## Opening the next run

1. **Append the run header** to `log.md`:

   ```markdown
   ## Run <N+1> — started <ISO>Z — budget <mins> min — deadline <ISO>Z — buffer-cutoff <ISO>Z

   (No pass entries yet. First pass appends below this header.)
   ```

   Place the new header BELOW the prior run's scheduler-stop entry. Do not reorder or edit prior entries.

2. **Compute fresh deadline + buffer epochs** for the new budget (see [budget-edge.md](budget-edge.md)). Round-trip verify.

3. **Pick a fresh minute offset** for the cron. If Run N used `:03,:13,:23,…`, Run N+1 can use the same offset — offsets don't collide with themselves, only with other crons at `:00`/`:30`.

4. **`CronCreate`** with the filled tick prompt. Capture the returned cron id.

5. **Write the cron id into the run header in log.md**:

   ```markdown
   ## Run <N+1> — started <ISO>Z — budget <mins> min — deadline <ISO>Z — buffer-cutoff <ISO>Z
   - Cron id: <id returned by CronCreate>
   ```

   The tick prompt reads the cron id from here (not from a baked-in placeholder) — `CronList` confirms only one project cron is active. This avoids the chicken-and-egg of baking the id into the prompt before it exists.

6. **Commit the run header** as `audit(scheduler): Run <N+1> started — budget <mins> min`.

7. **Fire one supervised tick** (CronFire, or manually paste the prompt). User watches the first pass land cleanly.

8. Cron takes over. Agent goes AFK with the user.

## What changes between runs

Fresh each run:

- Run number (monotonic; don't reuse)
- Start/deadline/buffer ISO + epochs
- Cron id
- `log.md` run header

Carried over (don't reset):

- `stories.md` (closed stories stay `[x]`; the backlog continues)
- `scope.md` (rules are stable unless the user tuned a threshold)
- `promote-tool-gaps.sh` (same script; output grows over time)

If `scope.md` changed between runs, commit the change as `looper: <imperative>` before opening the new run header so the rule change's SHA is in git history, not buried in a pass commit.

## Kill-switch → resume

If the user paused with `.audit-pause` and wants to resume:

1. Confirm `.audit-pause` is removed (`rm .audit-pause`).
2. Check the cron state. Two cases:
   - **Cron still armed** → next fire will proceed normally. Nothing to do.
   - **Cron was deleted** (e.g., user deleted explicitly) → open Run <N+1> per "Opening the next run" above. Do NOT append passes to Run N's section; the run ended when the cron was deleted.

## Multiple active runs

Don't. A second cron for the same project will race the first. If the user asks for a parallel run on a different surface, either:

- Use a different `<AUDIT_ROOT>` and a different cron — but this is rarely what the user wants
- Or finish the current run first

`CronList` always has at most one project-scoped audit cron. If it shows two, the earlier run was not cleanly ended — pause both and reconcile with the user.

## Long-running runs (>1 day)

For multi-day runs, break into shifts of ≤8 hours. Each shift is its own Run <N>. Reasons:

- A budget much larger than 8 hours makes the buffer (20 min) negligible as a fraction, reducing its effectiveness.
- Log.md grows unwieldy; Run-scoped sections keep it navigable.
- User can inspect between shifts and tune scope.md if drift is emerging.

If the user insists on an 18-hour run, fine — but note in scope.md that the buffer should grow proportionally (e.g., 60 min for an 18-hour run).
