# Arming Checklist

Run through this before the user leaves. Skip none. This is the gate between "we have a loop" and "the loop is live".

## 1. Working tree coexistence

```bash
git status --porcelain
```


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

Run the project's link check (`source checks`, `markdown-link-check`, etc.). Must pass.

## 3. Session healthy (for apps)

```bash
bash scripts/agentic/session.sh status default
```


## 4. Budget is realistic


| Budget | Expected passes | Driver |
|---|---|---|
| 60 min | 15–20 | ScheduleWakeup |
| 120 min | 30–40 | ScheduleWakeup or cron |
| 180 min | 45–60 | cron (agent restarts become more likely) |
| 420 min (7h) | 100–130 | cron (required) |
| 720 min (12h overnight) | 180–240 | cron (required; consider 2 shifts) |


- Seed more stories before arming
- Accept the loop will enter drain mode and sleep out the remaining budget

Never arm a run whose budget is < 2× the worst-case pass duration (20 min). A 30-min budget buys 1–2 passes; not worth the arming overhead.

## 5. Deadline + buffer arithmetic

```bash
DEADLINE_EPOCH=$(( START_EPOCH + BUDGET_MIN * 60 ))
BUFFER_EPOCH=$(( DEADLINE_EPOCH - 20 * 60 ))

```

A one-minute slip shipped to the cron becomes immortal. Verify first.

## 6. Record run header in log.md


```markdown
## Run <N> — started <START_ISO>Z — budget <BUDGET_MIN> min — deadline <DEADLINE_ISO>Z — buffer-cutoff <BUFFER_ISO>Z
```


```
```


## 7. Arm the cron


```bash
bash looper/scripts/arm-loop.sh <BUDGET_MIN> audits/afk default "bash scripts/agentic/session.sh"
```




## 8. One supervised tick


1. The tick reads scope.md + picks the first `[ ]` story.
2. Runs verification.
3. Logs the outcome.
4. Commits the pass.
5. Backfills the SHA.

Only after that supervised tick succeeds does the cron take over.

## 9. Brief the user on what to expect



They should leave knowing exactly where to look when they return.



- `~/.claude/skills/` entries (see `feedback_no_skills_during_afk_loop` memory)
- `looper/rules/*.md` (would change mid-run discipline; pause the cron first)
- `audits/afk/scope.md` (same)

Editing is fine; the loop just can't reason about the change until it's committed and the next tick reads the new rules. For clarity, pause the cron before any of the above.
