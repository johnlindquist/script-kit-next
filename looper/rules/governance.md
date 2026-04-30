# Governance Toggles

Optional rules layered on top of the base loop. Adopt selectively based on what prior runs revealed. **More rules = less drift but less throughput.** Start minimal and tune up.

## Start with (run 1)

- Kill switch (`.audit-pause`)
- Budget + buffer (from [budget-edge.md](budget-edge.md))
- Verification gate (every Fix/Add/Extend re-runs the same verification before commit)
- Subject-verb discipline (from [discipline.md](discipline.md))

Don't pre-optimize. Observe drift patterns in run 1's log.md before adding toggles.

## Add after run 1 review

Review `log.md` for the following drift patterns and add the matching toggle:

| Drift pattern | Toggle to add |
|---|---|
| Same surface dominates ≥3 consecutive passes | **Surface rotation** |
| Clustering on 2–3 surfaces across 10 passes | **Surface breadth floor** |
| Mostly source-audit pins, few behavior tests | **Pin cap** + **Bug-yield floor** |
| Story list grows faster than it drains | **Drain mode** |
| Bugs never surface in golden-path passes | **Attacker mode** |
| Prose-only `tool-*` mentions never become stories | **Tool-gap auto-promotion** |

Add one toggle per run, not all at once. Track which toggles fire and how often — a toggle that fires every pass is too tight; one that never fires is too loose. Aim for 10–25% fire rate.

---

## Surface rotation

Don't run the same surface twice in a row.

**Rule**: if the previous pass's surface matches the next candidate's, skip the candidate and pick the next `[ ]` story with a different surface.

**When to adopt**: log shows 3+ consecutive passes on the same surface.

**Check (the tick runs this in Step 4)**:

```bash
LAST_SURFACE=$(grep -E '^- \*\*Surface\*\*:' audits/afk/log.md | tail -1 | sed 's/.*: //')
# Candidate surface MUST NOT equal $LAST_SURFACE
```

---

## Surface breadth floor

Require ≥N unique surfaces in the last M passes.

**Default threshold**: ≥6 unique in last 10.

**Enforcement**: when below floor, filter candidate stories to surfaces not yet covered in the window.

**Check**:

```bash
DISTINCT=$(grep -E '^- \*\*Surface\*\*:' audits/afk/log.md | head -10 | sort -u | wc -l)
# If DISTINCT < 6, the next pass's surface MUST be one not in the last 10
```

---

## Bug-yield floor

Require at least 1 Fix / Add / Extend commit per K-pass window.

**Default threshold**: ≥1 per 5-pass window.

**Enforcement**: if below floor and next story would be a Pin/Verify, skip it; pick a behavior test.

**Check**:

```bash
YIELD=$(git log -5 --format="%s" | grep -cE '^Prompt: (Fix|Add|Extend) ')
# If YIELD == 0, next pass MUST be Fix/Add/Extend (no Pin/Verify/Probe)
```

**Rationale**: prevents drift into pure contract-test pinning that doesn't catch real bugs. Pins have value but can dominate the log without exercising real behavior.

---

## Pin cap (contract-test cap)

If more than T passes in the last W have been `Prompt: Pin` commits, force behavior/integration tests for the next pass.

**Default threshold**: ≤7 pins in last 20.

**Check**:

```bash
PINS=$(git log -20 --format="%s" | grep -c '^Prompt: Pin ')
# If PINS >= 7, Pin is FORBIDDEN this tick
```

**Required fields on pin commits** (when this toggle is on, always):

- `Falsifier:` (already universal)
- `Refactor threat:` (specific contributor action — see [discipline.md](discipline.md))
- Subject: `Prompt: Pin <invariant> against <named refactor>`

**Banned phrases on pin rationales** — reject the pin if any appear without a concrete drift vector in the same paragraph:

- "already correct by construction"
- "pins existing behavior"
- "prevents silent drift"
- "locks down current shape"
- "belt-and-suspenders"

See [discipline.md](discipline.md) for the full consolation-pin filter.

---

## Drain mode

If open story count > D, stop generating new stories until open count drops.

**Default threshold**: >8 open.

**Enforcement**: the tick prompt's story-generation step (after the seed list drains) is gated by open-count.

**Rationale**: prevents backlog growth during unattended runs. The loop focuses on closing existing stories, not inventing more.

---

## Tool-gap auto-promotion

When a pass discovers a missing test surface (RPC command, state field, helper), append a `tool-<slug>` story to the backlog.

**Drain priority**: `tool-*` stories drain BEFORE new seed stories.

**Automation**: `promote-tool-gaps.sh` runs every tick (Step 3) and promotes prose-only `tool-*` mentions in log.md to actionable `[ ]` items.

**Rationale**: fixing test infrastructure multiplies future pass throughput. Worth prioritizing.

See [scripts/promote-tool-gaps.sh](../scripts/promote-tool-gaps.sh).

---

## Attacker mode cadence

Every Kth pass runs attacker mode. See [attacker-mode.md](attacker-mode.md) for composition rules.

**Default cadence**: every 4th pass (passes #4, #8, #12, …).

**When to adopt**: base loop is producing clean passes reliably. Don't start here — attacker mode is noisy and catches false positives if the base flow isn't solid.

---

## Threshold tuning across runs

All numbers above are defaults. After each run, review `log.md` for threshold-hit frequency:

- **Never triggered** → threshold too loose (or the problem doesn't exist; remove the toggle)
- **Triggered every pass** → threshold too tight (loosen or the toggle is redundant with another)
- **10–25% fire rate** → well-tuned

Track threshold hits in the log (e.g., `- Notes: bug-yield floor forced Fix (YIELD=0 in last 5)`) so you can tune across multiple runs.

## When rules conflict

If two toggles want different things:

1. Pin cap trumps bug-yield floor (a forced non-Pin may still need to be Fix/Add/Extend, which the bug-yield floor wants anyway — usually they align).
2. Surface rotation trumps tool-gap priority (if the only `tool-*` story's surface just ran, skip it this tick).
3. Attacker cadence is unconditional (every 4th pass is attacker mode even if the discipline checks want something else).

When genuinely deadlocked, log `failed` for the pass with a `Notes:` line naming the conflict, and let the user untangle on return.
