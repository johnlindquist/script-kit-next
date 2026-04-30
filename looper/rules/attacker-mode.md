# Attacker Mode

Every 4th pass (passes #4, #8, #12, …) runs adversarially — no pre-specified acceptance criteria, intentionally exploratory. The goal is bugs the golden-path stories miss.

## Minimum composition (or it doesn't count)

- **≥20 actions** in the pass
- **≥3 distinct adversarial categories** from the menu
- Firing the same command 20× (one category, rapid-fire) does NOT qualify — retry

A pass that doesn't meet minimum composition is a discipline violation. Retry with different categories or log `failed` and move on.

## Recipe menu

| Category | Description |
|---|---|
| **Rapid-fire** | Same command ≥20× within 2 seconds |
| **Interrupted** | Start action A, cancel mid-effect, start action B |
| **Interleaved** | Ping-pong between ≥3 surfaces in random order |
| **Boundary** | Empty string, 10k+ char string, zero-width unicode, emoji, RTL, NUL, malformed UTF-8 |
| **Concurrent** | ≥5 RPCs within a 100 ms window |
| **Resurrection** | Invoke against a view supposedly dismissed |
| **Composition** | Sequences valid individually but unspecified in combination (detach → hide → reattach → show) |

Compose ≥3 of these per attacker pass.

## Near-anomaly filing

File a `[?]` story (not just a `[ ]`) when ANY of these happen, even if the final state looks right:

- Tracing-span ordering is unexpected
- Timing falls outside documented bounds
- Log spam scales with input size (hint: unbounded loop)
- State contradicts its own documentation
- A receipt field is missing that the code claims to emit
- A `warn!` or `error!` appears that isn't named in the story's expected diagnostics

Near-anomalies are worth as much as outright failures — they're leading indicators. `[?]` is cheap to file; resolving it later is how the loop produces real fixes. A clean attacker pass with no `[?]` after 3 consecutive attempts should escalate.

## Escalation rule

If 3 consecutive attacker passes yield **zero** `[?]` filings, the next attacker pass MUST escalate:

- ≥30 actions
- ≥4 surfaces
- ≥4 adversarial categories
- ≥1 payload outside documented input bounds

Zero anomalies across 3 passes usually means the attacker pass isn't biting — either the agent is playing it safe, or the surfaces are genuinely robust. Escalation forces the first case to reveal itself.

## Commit

Attacker-mode commits use:

- `Prompt: Probe <surface> <short summary>` — no anomaly found
- `Prompt: Reproduce <trigger> <short summary>` — anomaly filed as `[?]` or `[!]`

Never `Prompt: Verify` — that verb is reserved for golden-path baseline receipts. Never `Prompt: Fix` on an attacker pass either — if a fix came out of the exploration, split into two commits: the Probe that surfaced it, then the Fix (which can be a regular pass).

Attacker passes with no anomaly are still valid for pass counting as long as minimum composition was met.

## Picking the target surface

Prefer surfaces NOT exercised in the last 10 passes (surface rotation). If all surfaces have recent coverage, pick the surface with the fewest anomalies filed in the current run — that's either robust (boring) or under-attacked (bugs hiding).

## Required per-commit fields for attacker passes

In addition to the universal fields ([discipline.md](discipline.md)):

```
Adversarial categories: <≥3 from menu, comma-separated>
Actions: <count ≥ 20>
```

No Falsifier line for Probe (there's no acceptance criterion to falsify). Reproduce commits DO need a Falsifier, naming the receipt state that would prove the anomaly was not reproduced.
