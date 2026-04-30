# Scheduler-Stop — End-of-Run Receipt

When the buffer cutoff trips, the tick writes a scheduler-stop entry to `log.md` and commits. This is a first-class record — it matches the `CronDelete` action with a durable receipt and makes the run's endpoint visible in `log.md`.

## When it fires

Two triggers:

1. **Step 1 (budget check)** — `now_epoch ≥ buffer_epoch`. The current tick would start a pass that overshoots.
2. **Step 9 (pre-emptive next-fire check)** — this tick is fine, but the next scheduled fire would be past buffer. Stop now to avoid one cadence window of idle fires.

Both use the same log format; only the `Notes:` line differs.

## Log entry format

```markdown
## Run <N> — scheduler-stop — <ISO timestamp>Z

- **Reason**: budget buffer cutoff hit (now_epoch=<now> ≥ buffer_epoch=<buf>, delta=+<secs>s past buffer)
- **Cron id**: <cron-id>
- **Action**: CronDelete <cron-id>
- **Deadline**: <deadline-ISO>Z (epoch <deadline-epoch>)
- **Buffer cutoff**: <buffer-ISO>Z (epoch <buffer-epoch>)
- **Last pass committed**: #<K> sha <short-sha>
- **Notes**: <free text — e.g. "final pass landed 4 min before buffer; cron fired 16s past" or "pre-emptive pre-next-fire stop; next fire at <ISO>Z (epoch <next>) would be past buffer">
```

## Commit

Subject for Step 1 stop:

```
audit(scheduler): Run <N> stop at buffer cutoff; CronDelete <cron-id>
```

Subject for Step 9 pre-emptive stop:

```
audit(scheduler): Run <N> pre-emptive stop before next-fire past buffer
```

Body (both):

```
Run <N> deadline <deadline-ISO>Z, buffer cutoff <buffer-ISO>Z.
Tick fired at <fire-ISO>Z (epoch <now>); buffer epoch <buf>; delta +<secs>s.
Per scope §"Scheduling cutoff", self-deleted cron <cron-id> without picking a story.

Last pass committed: #<K> sha <short-sha>.
```

For Step 9 pre-emptive, add:

```
Next fire would have been <next-ISO>Z (epoch <next>), which is <delta>s past buffer.
```

## Why this shape

- **Epoch deltas named explicitly** — post-mortem doesn't have to recompute.
- **Cron id named** — the next run can confirm it's gone via `CronList`.
- **Last committed pass referenced** — the revert range of the run is bounded (first pass SHA → last pass SHA).
- **`audit(scheduler):` prefix** (not `Prompt:`) — signals non-pass to quota checks.

## Alternate scheduler commits (non-stop)

Two other scheduler-class commits exist:

### Path-overlap abort

When Step 2 finds that every eligible story would need a pre-existing external
dirty path that the loop does not own:

```
audit(scheduler): path overlap blocked — Run <N>.<K> aborted
```

Body names the blocked story candidates and dirty paths. Tick exits without
staging external work; cron stays armed.

### Session-restart

When Step 3 finds a dead session and restarts it, the restart itself is not a commit — it's a bash side-effect. But if the restart fails twice, the tick logs and commits:

```
audit(scheduler): session restart failed twice — Run <N>.<K> aborted
```

Body names the session name and the `session.sh status` output.

Both abort-class commits leave the cron armed — the next fire tries again. Only Step 1 and Step 9 `CronDelete`.

## Rules for the scheduler-stop log entry's position in log.md

Append below the last pass entry for the same run. The run's section ends there; the next run's `## Run <N+1> — started …` header goes below.

Do not edit old run headers or entries. Each run's section is append-only.
