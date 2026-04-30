# AFK Audit Log — <project name>

Append-only. One entry per pass. Scheduler-stop and bookkeeping commits may also produce entries (non-pass).

Each run gets its own `## Run <N>` section with a header line naming the budget, deadline, and buffer. Passes append below the header in order.

---

## Run 1 — started <ISO>Z — budget <N> min — deadline <ISO>Z — buffer-cutoff <ISO>Z

- Cron id: <id returned by CronCreate>

(First pass entry appends below this header.)

---

## Pass entry template

```markdown
## Run <N> — Pass #<K> — <ISO timestamp>Z

- Surface: <main-menu | acp | builtin | stdin | attacker | refactor | stdin/protocol-router | …>
- Story: <slug>
- Outcome: pass | fix-committed | skipped | failed | deferred
- Commit: <sha-pending>  (← Phase 1; Phase 2 backfills the short SHA)
- Files: <n> changed, +<I>/−<D>
- Proof: <one-line state receipt fields or screenshot path>
- Falsifier: <one concrete state that would have proven this pass wrong>
- Notes: <short — names threshold triggers, deferred items, attacker composition, etc.>
```

### Pin-only additional fields

```
- Refactor threat: <specific contributor action — e.g. "a contributor deduping the three stdin dispatchers by extracting a shared helper could drop the post-match re-key call in one arm">
```

### Attacker-only additional fields (Probe / Reproduce)

```
- Adversarial categories: <≥3 from menu — comma-separated>
- Actions: <count ≥ 20>
```

## Scheduler-stop entry template

```markdown
## Run <N> — scheduler-stop — <ISO timestamp>Z

- **Reason**: budget buffer cutoff hit (now_epoch=<now> ≥ buffer_epoch=<buf>, delta=+<secs>s past buffer)
- **Cron id**: <cron-id>
- **Action**: CronDelete <cron-id>
- **Deadline**: <deadline-ISO>Z (epoch <deadline-epoch>)
- **Buffer cutoff**: <buffer-ISO>Z (epoch <buffer-epoch>)
- **Last pass committed**: #<K> sha <short-sha>
- **Notes**: <free text; "Step 1 stop", "Step 9 pre-emptive stop", "kill switch", etc.>
```

## Loop-status summary (optional, end-of-run)

```markdown
## Run <N> — summary — <ISO timestamp>Z

- Passes committed: <total> (<fix-committed>/<pass>/<failed>/<skipped>/<deferred>)
- Distinct surfaces: <count>
- Threshold triggers: <bug-yield forced Fix: 2x, pin cap triggered: 0x, rotation violated: 1x>
- Cron fires: <total> (<aborted-preflight>/<full-pass>/<scheduler-stop>)
- Open stories at end: <count>
- Notes: <free text, lessons for scope.md tuning>
```

## Stop reasons (append at run end if not scheduler-stop)

- `Budget exhausted` — elapsed ≥ budget (rare; usually scheduler-stop fires first)
- `Kill switch` — `.audit-pause` present at tick
- `Unrecoverable` — scope.md missing or malformed; tick can't read rules
- `User interrupt` — session ended externally (no tick got to write the stop entry; retrospective note added later)
