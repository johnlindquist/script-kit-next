# Cadence — Cache TTL and Cron Minute Offsets

How the loop paces itself without burning the prompt cache or clustering with other schedulers.

## The 5-minute cliff (for ScheduleWakeup)

The Anthropic prompt cache TTL is **5 minutes**. Sleeping past 300 s means the next wake-up reads your full conversation uncached — slower (multi-second first-token latency), more expensive, and the cache has to rebuild before your first tool call lands.

Two regimes:

- **Under 5 min** — cache stays warm. Cheap to wake. Use for active work between passes.
- **Over 5 min** — you pay the cache miss once. Only worth it if the wait is much longer than 5 min.

## Why 300 s is the trap

`delaySeconds=300` is worst-of-both: you pay the cache miss **and** barely gain any extra wait. If you're tempted to "wait 5 minutes", either drop to **270 s** (stay cached) or commit to **1200 s+** (one miss buys a real wait).

Never think in round minutes. Think in cache windows.

## ScheduleWakeup defaults by pass type

| Pass type | `delaySeconds` | Rationale |
|---|---|---|
| Source-level pin (no build) | **120** | Tight verify, warm session |
| RPC / stdin verification | **180** | Warm session, short commit |
| Build-heavy fix (cargo build + restart) | **240–270** | Just under cliff |
| Idle drain / waiting for signal | **1200–1800** | One miss buys 20–30 min |
| Don't know what to do | **1200** | Safer than burning cycles |

Runtime clamps to `[60, 3600]`. Don't schedule sub-60 s.

## Cron minute offsets (macOS launchd / most schedulers)

Schedulers batch-fire tasks on round minutes (`:00`, `:30`). Two observable failures:

1. **Late fire** — the `:00` batch is serialized; a tick scheduled at `:00` can land at `:00:12` or later. The extra latency eats into the buffer.
2. **Cluster collisions** — if multiple crons across projects all target `:00`, they compete for dispatcher attention.

Offset into the valleys between round numbers.

### Recommended offsets by cadence

| Cadence | Safe offsets | Fires/hour |
|---|---|---|
| 10 min | `:03,:13,:23,:33,:43,:53` **or** `:07,:17,:27,:37,:47,:57` | 6 |
| 6 min | `:03,:09,:15,:21,:27,:33,:39,:45,:51,:57` | 10 |
| 15 min | `:04,:19,:34,:49` | 4 |
| 30 min | `:17,:47` | 2 |

Rule of thumb: never `:00` or `:30`. When in doubt, offset by a small prime (3, 7, 11) from the natural round.

### Pass-duration matching

Pick a cadence ≥ 2× the typical pass duration so successive ticks don't overlap:

- Source-level pin passes: ~1 min → 6-min cadence fine.
- Fix + re-verify passes: ~3–5 min → 10-min cadence safer.
- Attacker passes (≥20 actions): ~5–8 min → 10-min or 15-min cadence.

A tick that fires while the previous is still running wastes itself — the
worktree coexistence check should detect owned-path overlap and avoid staging
the other tick's files. Cadence should be comfortably longer than the
95th-percentile pass duration.

## Deadline buffer

**Stop scheduling new fires 20 min before budget deadline.** The final pass needs runway for verify + commit + backfill + any hook overruns. See [budget-edge.md](budget-edge.md) for the math.

## Typical run shape

A well-paced 2-hour run (cron at 10-min cadence) produces:

- ~10–11 ticks (budget 120 min − 20 min buffer ÷ 10 min = 10)
- ~8–10 actual passes (some ticks may abort on path-overlap or discipline checks)
- Mix: ~60% source/RPC passes, ~25% fix passes, ~15% attacker passes (every 4th)

If ticks routinely abort before the story pick, the cadence is too tight — increase to 15 min. If ticks routinely complete in < 3 min, the cadence is OK.

## The `reason` field (ScheduleWakeup)

`ScheduleWakeup` takes a `reason` that ships to telemetry and shows to the user. Be specific:

- Bad: `"waiting"`, `"next iteration"`, `"loop continues"`
- Good: `"post-build, cache-warm tight pass"`, `"drain mode — wait for user"`

The user reads these to understand cadence without having to predict it in advance.

## Don't poll

If you started a background task with `Bash(run_in_background=true)`, you get a notification on completion. Don't sleep-poll checking on it — that burns cache cycles for no signal. Same for `Agent(run_in_background=true)` — notification on completion, no need to sleep.
