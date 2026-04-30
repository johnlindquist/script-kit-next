# The Five Invariants

Everything in looper follows from these. When rules conflict, the invariant wins. When a new situation arises, derive the rule from the invariant that applies.

## 1. Every rule is grep-auditable

The run's truth lives in three files plus one command:

- `git log --oneline` (commit history)
- `audits/afk/log.md` (per-pass receipts)
- `audits/afk/stories.md` (backlog state)
- `CronList` (cron schedule)

No private agent state. If a rule can't be checked by reading one of those, it isn't a rule — it's a vibe.

**Test for this invariant**: a developer inspecting the run 6 months later must be able to answer every question ("why did this pass fail?", "why was this surface repeated?", "when did the loop stop?") from the artifacts above alone.

## 2. Every commit is reversible

One pass = one SHA. Enforced by:

- **No amend.** `git commit --amend` rewrites history; the SHA already referenced in `log.md` becomes invalid.
- **No force push.** Passes shipped to `main` stay on `main`. Revert with `git revert <sha>`.
- **SHA backfill is a separate commit.** The `audit(log): backfill …` commit updates the log with the main commit's SHA. Two commits per pass — intentional; see [sha-backfill.md](sha-backfill.md).
- **No bundled passes.** Two stories = two commits. Exception: the discipline-preflight bookkeeping commit (`audit(backlog): promote tool-* slugs …`) is not a pass.

**Test for this invariant**: `git revert <sha>` on any pass commit leaves the repo in a state the next tick can proceed from.

## 3. Every claim is falsifiable

Every pass commit body names a `Falsifier:` line — one concrete receipt state that would have proven the pass wrong. Rationales with no falsifier are not passes.

Good falsifiers (concrete, observable):

- "If the receipt had returned `{granted: null}` instead of `{granted: true|false}`."
- "If the second `/` keystroke had not reopened the slash picker (timeout 2s)."
- "If the post-fix `cargo test source_audits -- --exact` had still flagged any pin."

Bad falsifiers (rejected — rewrite or downgrade pass):

- "If the code were wrong." (not observable)
- "If the test failed." (tautological)
- "N/A." (reject outright)

**Test for this invariant**: an adversary reading only the falsifier can state what they would have to change to break the pass's validity.

## 4. Every rule is tunable per run

Thresholds live in `audits/afk/scope.md`. The tick prompt reads them; the agent does not invent numbers. When the user tunes a rule for a run, the change is committed alongside the run's scaffolding, not whispered to the agent.

Tunable knobs:

- Pin cap (default: `≤7 in last 20` passes)
- Bug-yield floor (default: `≥1 Fix/Add/Extend in last 5` passes)
- Surface rotation (default: next pass's surface not in last 10 unless breadth < 6)
- Attacker cadence (default: every 4th pass)
- Buffer cutoff (default: `deadline − 20 min`)
- Cron cadence (default: 10 min = `:07,:17,:27,:37,:47,:57` for this project)

Changing a knob mid-run: pause the cron, edit scope.md, log the change, arm a new run (same log.md, new `## Run <N+1>` header). Do not edit scope.md under a live cron — the tick between the edit and the next fire has ambiguous rules.

## 5. Every budget overshoot is a bug

Buffer exists to absorb worst-case pass duration. When `now_epoch ≥ buffer_epoch`, the tick:

- Calls `CronDelete` unconditionally (no "one more pass")
- Appends a scheduler-stop entry to `log.md`
- Commits with `audit(scheduler): …`
- Exits

The temptation to "squeeze one more pass in" is the failure mode of record. Run 2 overshot by 1h 51m because the loop started a pass 9 min before deadline. Run 4 caught a +16s-past-buffer fire and correctly self-deleted. The buffer is load-bearing.

Deadline math uses integer epoch-seconds, not ISO string compare. See [budget-edge.md](budget-edge.md).

---

## Where invariants map to rules

| Invariant | Primary enforcement | Secondary |
|---|---|---|
| Grep-auditable | Log format (log.md), commit subjects | Scope.md thresholds |
| Reversible | No-amend rule, SHA backfill | One pass = one commit |
| Falsifiable | Required `Falsifier:` body line | Banned-phrase reject list |
| Tunable | scope.md is canonical | Rule files are committed |
| No overshoot | Buffer + self-delete | Pre-emptive next-fire check |

When in doubt, read the invariant first. The rule below it exists because the invariant requires it.
