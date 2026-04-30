# Subject-Verb Discipline and Per-Commit Receipts

The subject verb on a pass commit is the load-bearing label for what happened. Graders, pin caps, and bug-yield floors all read the verb. A `Pin` that should have been `Verify`, or a `Verify` that should have been `Extend`, is a discipline violation — reject or rewrite.

## Subject-verb taxonomy

Every pass commit starts `Prompt: <verb> …`. The verb is drawn from exactly this list:

| Verb | When to use | Anti-example (reject) |
|---|---|---|
| **Fix** | User-visible behavior change. The app/CLI/API does something new after the commit that it didn't before. | "Fix comment typo in dead code" (no behavior change → this is prose, not Fix). |
| **Add** | New RPC verb, new receipt field, new capability the surface didn't have. | "Add a test" — tests alone are not Add; see Pin/Verify. Adding a test with no corresponding production change is either Pin (if source-audit) or should not be a pass. |
| **Extend** | Existing verb/tool/helper now handles a case it didn't before. | "Extend the refactor" (what case?). "Extend X to handle Y" names the case. |
| **Pin** | Source-level contract test that fails if a specific plausible refactor breaks the invariant. Subject MUST name the defended refactor: `Prompt: Pin <invariant> against <named refactor>`. | "Pin current shape" — no named refactor, no falsifiable claim. Reject. |
| **Verify** | Pass-only. No code change. A fresh baseline receipt confirms the story still holds. | Source-audit test claiming to verify — that's Pin, not Verify. Verify is for behavior receipts. |
| **Probe** | Attacker pass with no pre-declared acceptance criteria. `≥20 actions, ≥3 categories`. | "Probe and found nothing" with 5 actions across 1 category. Does not qualify — retry. |
| **Reproduce** | Attacker anomaly captured as a reproducible trigger sequence. Names the steps a developer can replay. | "Reproduce the flakiness" — flakiness is not a reproducer. Name the exact steps. |

### Banned phrases on Pin rationales

Reject the pin if any of these appear without a concrete drift vector named in the same paragraph:

- "already correct by construction"
- "pins existing behavior"
- "prevents silent drift"
- "locks down current shape"
- "belt-and-suspenders"

These phrases signal a consolation pin — the agent couldn't ship real work, so it shipped a tautology. Escalate: ship the real extension, or file `[!]` and move on. Do not substitute.

### Consolation-pin tell (stronger filter)

If the pass's log entry reads as "story blocked on missing tool, so I pinned `<adjacent invariant>` instead", the pin is invalid regardless of falsifier quality. The correct action is:

1. File the original story as `[!]` with a tool-gap note.
2. Either ship the tool extension as the pass (`Prompt: Extend agentic-testing — add <verb> RPC`), or log-only and move on.

A pin defending an adjacent invariant the agent did not touch is not a pass.

## Per-commit receipt fields (universal)

Every pass commit body has these fields. Missing any one is a discipline violation.

```
User story: <slug> — <exact text from stories.md>
Pass: #<K> of AFK audit Run <N> <START_ISO>
Proof: <key state receipt fields OR screenshot path>
Falsifier: <one concrete receipt state that would have proven this pass wrong>
```

### Pin-only additional fields

```
Refactor threat: <specific contributor action — e.g. "a contributor deduping
  the three stdin dispatchers by extracting a shared helper could drop the
  post-match re-key call in one arm">
```

The threat MUST be a specific action a named role could take. Passive phrasing ("a refactor could …", "drift could …") is reject-worthy. If you can't name the action, the pin is too abstract — either narrow it or drop it.

### Attacker-only additional fields (Probe / Reproduce)

```
Adversarial categories: <≥3 from menu — rapid-fire, interrupted, interleaved,
  boundary, concurrent, resurrection, composition>
Actions: <count ≥ 20>
```

See [attacker-mode.md](attacker-mode.md) for the menu and composition minimums.

## Commit subject length

Keep the subject line ≤72 characters. The body carries the detail. A subject that needs more context is an Add masquerading as a Fix — promote the scope.

## Non-pass commits

These don't count toward pin cap, bug-yield floor, or pass numbering. Use a different prefix (`audit(…):`) to signal "bookkeeping":

| Prefix | Purpose |
|---|---|
| `audit(log): backfill Run <N> Pass #<K> sha <short>` | SHA backfill after a pass commit |
| `audit(backlog): promote tool-* slugs (tick <N>.<K>)` | `promote-tool-gaps.sh` output |
| `audit(scheduler): Run <N> stop at buffer cutoff; CronDelete <id>` | End-of-run scheduler stop |
| `audit(scheduler): Run <N> pre-emptive stop before next-fire past buffer` | Step 9 pre-emptive stop |
| `audit(scheduler): path overlap blocked — Run <N>.<K> aborted` | Aborted tick because every eligible story would need a pre-existing external dirty path |
| `looper: <imperative>` | Rule/template/script edit in looper/ |

`audit(…):` and `looper:` prefixes are the single-commit signal: "this is loop bookkeeping, not verification work". A tick that tries to dress up bookkeeping as a pass (by using `Prompt:`) is a discipline violation.

## Verb quick-reference (when to pick which)

```
Did user-visible behavior change?       → Fix (or Add / Extend if it's a new capability)
Is this a new verb / field / capability? → Add
Does an existing verb now cover a new case? → Extend
Did I only add a test asserting a source invariant? → Pin (must name defended refactor)
Did I only re-run a baseline story and it stayed green? → Verify
Did I attack a surface with ≥20 adversarial actions? → Probe (no anomaly) or Reproduce (anomaly filed)
Did I only edit log.md / stories.md / looper/? → Not a pass. Use audit(…): or looper:
```

If the answer crosses categories, split the work into multiple commits. One pass = one SHA = one verb.
