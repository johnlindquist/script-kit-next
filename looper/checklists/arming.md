# Arming Checklist

Run through this before the user leaves. Skip none. This is the gate between "we have a loop" and "the loop is live".

## 1. Working tree coexistence

```bash
git status --porcelain
```

If non-empty:

- Name and classify every dirty path as user work, other-agent work, known audit prep, generated artifact, or unknown.
- Proceed with dirty external paths present. They are not blockers.
- Do NOT stash, revert, reset, restore, clean, or commit external paths.
- Plan the first story so its write set does not overlap dirty external paths.
- During commits, use explicit pathspecs only (`git add -- path...`), never `git add .` / `git add -A`.

## 2. Scaffolding in place

```
audits/afk/scope.md            # rules
audits/afk/stories.md          # backlog
audits/afk/log.md              # append-only findings
audits/afk/promote-tool-gaps.sh # copied from looper/scripts/
.gitignore                     # contains .audit-pause
looper/                        # rule source (committed)
```

Run the project's link check (`lat check`, `markdown-link-check`, etc.). Must pass.

## 3. Session healthy (for apps)

```bash
bash scripts/agentic/session.sh status default
```

- `alive:true` with no issues → ready.
- `alive:false` or any issues → restart. Verify the session responds to a no-op RPC before arming.

## 4. Budget is realistic

Rule of thumb: **~3 min per pass** assuming a healthy backlog and warm cache.

| Budget | Expected passes | Driver |
|---|---|---|
| 60 min | 15–20 | ScheduleWakeup |
| 120 min | 30–40 | ScheduleWakeup or cron |
| 180 min | 45–60 | cron (agent restarts become more likely) |
| 420 min (7h) | 100–130 | cron (required) |
| 720 min (12h overnight) | 180–240 | cron (required; consider 2 shifts) |

If the backlog has fewer stories than the expected pass count, either:

- Seed more stories before arming
- Accept the loop will enter drain mode and sleep out the remaining budget

Never arm a run whose budget is < 2× the worst-case pass duration (20 min). A 30-min budget buys 1–2 passes; not worth the arming overhead.

## 5. Deadline + buffer arithmetic

```bash
START_ISO=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
START_EPOCH=$(date -u -jf "%Y-%m-%dT%H:%M:%SZ" "$START_ISO" +%s)
DEADLINE_EPOCH=$(( START_EPOCH + BUDGET_MIN * 60 ))
BUFFER_EPOCH=$(( DEADLINE_EPOCH - 20 * 60 ))
DEADLINE_ISO=$(date -u -r "$DEADLINE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")
BUFFER_ISO=$(date -u -r "$BUFFER_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")

# Round-trip verify: round the epochs back to ISO, must match.
[ "$(date -u -r "$DEADLINE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")" = "$DEADLINE_ISO" ] && echo "deadline OK"
[ "$(date -u -r "$BUFFER_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")" = "$BUFFER_ISO" ] && echo "buffer OK"
```

A one-minute slip shipped to the cron becomes immortal. Verify first.

## 6. Record run header in log.md

Append below the prior run's section (or below the log intro if this is Run 1):

```markdown
## Run <N> — started <START_ISO>Z — budget <BUDGET_MIN> min — deadline <DEADLINE_ISO>Z — buffer-cutoff <BUFFER_ISO>Z
- Cron id: <fill in after CronCreate returns>
```

Commit as:

```
audit(scheduler): Run <N> started — budget <BUDGET_MIN> min
```

This commit's SHA is the anchor — the first pass's `Pass: #1` references the Run/start ISO in its body.

## 7. Arm the cron

Use `looper/scripts/arm-loop.sh` to emit the filled tick prompt:

```bash
bash looper/scripts/arm-loop.sh <BUDGET_MIN> audits/afk default "bash scripts/agentic/session.sh"
```

Copy the `--- BEGIN TICK PROMPT ---` block. Pick a minute offset per [cadence](../rules/cadence.md) (never `:00` or `:30`). `CronCreate` with the prompt + schedule.

After `CronCreate` returns the cron id:

1. Edit `audits/afk/log.md` — fill `- Cron id: <id>` on the run header.
2. Amend? NO — commit the update as `audit(scheduler): Run <N> cron id <id>`. (This is one of the few non-pass bookkeeping commits.)

## 8. One supervised tick

Before leaving, run **one full tick** in front of the user:

1. The tick reads scope.md + picks the first `[ ]` story.
2. Runs verification.
3. Logs the outcome.
4. Commits the pass.
5. Backfills the SHA.
6. User confirms: log format ✓, commit subject verb ✓, diff scope ✓, falsifier ✓.

Only after that supervised tick succeeds does the cron take over.

## 9. Brief the user on what to expect

Before they leave, say out loud:

- **Duration**: `<N> min`, deadline `<ISO>Z`, buffer cutoff `<ISO>Z`.
- **Expected passes**: `<range>`.
- **Kill switch**: `touch .audit-pause` in project root. Next tick exits; cron stays armed until you `CronDelete` or remove the switch.
- **On return, check**: `audits/afk/log.md` (end of Run <N> section) + `git log --oneline audits/afk/ looper/`.

They should leave knowing exactly where to look when they return.

## Optional: tell the user what *not* to do

Avoid editing during an active loop:

- `~/.claude/skills/` entries (see `feedback_no_skills_during_afk_loop` memory)
- `looper/rules/*.md` (would change mid-run discipline; pause the cron first)
- `audits/afk/scope.md` (same)

Editing is fine; the loop just can't reason about the change until it's committed and the next tick reads the new rules. For clarity, pause the cron before any of the above.
