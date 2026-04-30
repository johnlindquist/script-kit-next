# Budget Edge — Deadline, Buffer, Self-Delete

The loop's failure mode of record is NOT missing bugs — it's overshooting the agreed budget. The buffer exists to prevent that; the self-delete is unconditional for the same reason.

## Buffer definition

```
buffer_epoch = deadline_epoch − 20 minutes
```

20 min is the observed worst-case pass duration (fix + rebuild + re-verify + commit + SHA backfill) rounded up. Tune per project, never below 10 min.

## Why self-delete is unconditional

The temptation is "I'm only 16 s past buffer, one more pass should be fine". Run 4 cron fired 16 s past buffer; the correct action was to self-delete. The pass that would have started would have committed at or past deadline — the entire 20-min buffer exists to absorb that worst case. Eating into the buffer on the first pass that hits the edge negates the buffer.

### Data points (why the rule is unconditional)

| Run | Buffer cutoff | Actual last fire | Delta | Action |
|---|---|---|---|---|
| 2 | n/a (no buffer rule) | overshot deadline +1h 51m | — | rule added post-hoc |
| 4 | 2026-04-18 06:06:52Z | 2026-04-18 06:07:08Z | +16 s | `CronDelete`, log, commit |

The Run 2 overshoot is the reason for the buffer. The Run 4 +16-s stop is the reason for "unconditional" — 16 s felt like nothing, but honoring the rule cleanly ended the run with no partial passes.

## Epoch math (macOS BSD date)

Budget comparisons are integer epoch-seconds. Never ISO string compare — `"13:27Z" < "13:28Z"` works by luck within a day but breaks across day/month/year boundaries.

### ISO → epoch

```bash
date -u -jf "%Y-%m-%dT%H:%M:%SZ" "2026-04-18T13:27:28Z" +%s
# 1776518848
```

Linux coreutils uses `-d` instead of `-jf`:

```bash
date -u -d "2026-04-18T13:27:28Z" +%s
```

### Current time

```bash
now_epoch=$(date -u +%s)
```

### Deadline + buffer

```bash
start_epoch=$(date -u -jf "%Y-%m-%dT%H:%M:%SZ" "$START_ISO" +%s)
budget_sec=$(( BUDGET_MIN * 60 ))
deadline_epoch=$(( start_epoch + budget_sec ))
buffer_epoch=$(( deadline_epoch - 20 * 60 ))
```

### Round-trip verify before arming the cron

```bash
date -u -r "$deadline_epoch" +"%Y-%m-%dT%H:%M:%SZ"
# must match the ISO you wrote to log.md
```

A one-minute slip shipped to the tick prompt becomes immortal — the tick can't know whether its embedded `<BUFFER_EPOCH>` is the correct one. Verify first.

## The comparison the tick runs

```bash
if [ "$now_epoch" -ge "$buffer_epoch" ]; then
  # stop: CronDelete, log, commit, exit
fi
```

Use `-ge`, not `-gt`. Equality means "exactly at buffer" — too late to start a pass.

## Pre-emptive next-fire check (Step 9)

The tick that fires AT `buffer − 8 min` still passes the Step 1 budget check — but if the cadence is 10 min, the next fire would be at `buffer + 2 min`. Waiting for the next tick to self-delete means an entire cadence window of idle cron fires.

Step 9 catches this edge:

```bash
# Compute next fire by aligning now_epoch to the next matching minute in the cron schedule.
# For cadence :07,:17,:27,:37,:47,:57, find the smallest :MM ≥ current minute + 1.
# next_fire_epoch = start of that minute in UTC
if [ "$next_fire_epoch" -ge "$buffer_epoch" ]; then
  CronDelete <id>
  # scheduler-stop entry, note: pre-emptive pre-next-fire stop
  # commit: audit(scheduler): Run <N> pre-emptive stop before next-fire past buffer
fi
```

Pre-emptive stop uses the same log format as the Step 1 stop but with a `Notes:` line naming "pre-emptive" and the next-fire epoch that would have been.

## Why self-delete instead of "just skip this tick"

If the tick only skips, the cron keeps firing every 10 min forever, each one logging scheduler-stop, each one a small churn commit. Deleting the cron ends the run cleanly. The user can inspect the final log entry and decide whether to arm a new run.

## Scheduler-stop receipt

When Step 1 or Step 9 trips, the tick writes a scheduler-stop entry to `log.md`. Format: see [scheduler-stop.md](scheduler-stop.md).
