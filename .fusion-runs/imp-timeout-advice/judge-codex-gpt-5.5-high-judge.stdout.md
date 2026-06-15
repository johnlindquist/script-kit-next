## Consensus

Most usable outputs agree that the main timeout risk is layered blocking: router-level `spawnSync` has no timeout, warm startup/RPC waits can consume 30-60s, warm `runTurn` rejects at 120s, and the cold SDK fallback is not bounded by a comparable repo-local timeout. Local source supports this in [project-imp.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/bin/project-imp.ts:35), [appserver.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/appserver.ts:110), [appserver.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/appserver.ts:180), and [isolated.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/isolated.ts:318).

The strongest shared recommendation is to preserve imps as advisory by bounding the outer call, adding lifecycle receipts independent of model success, and making warm/RPC/turn/cold timeouts configurable. There is also broad support for concise advisory prompts and not treating imp failure as a blocker.

Most useful agents agree that `startup_timeout_sec` and `tool_timeout_sec` should not be added blindly to imp thread config. The best-supported position is that they are MCP-server config keys, not general Codex turn/app-server timeouts.

## Contradictions

Codex says self-improve hooks can add tail latency, while Kimi’s partial output notes the Stop hook is off by default unless `stopHook` is true. Source supports the Kimi nuance: `project-config.ts` enables selfImprove, but `self-improve.ts` sets `stopHook` only when explicitly true, and `codex-runtime.ts` writes hooks config only when `resolved.stopHook` is true. So hooks are a possible opt-in edge case, not a likely default cause.

Codex recommends defaulting advisory warm turn timeout to around 90s, while the prompt asks whether to increase the 120s timeout. The better-supported synthesis is: do not simply increase 120s for the main advisory path; put an outer advisory budget first, then allow direct/manual imp runs to use a longer env-configured budget.

Claude refuses to make source claims because its environment lacked tools. That is correct for its run, but it underdelivers on the requested independent answer. Its verification checklist is useful but not a substitute for a recommendation.

## Partial Coverage

Codex covers the implementation shape best: bounded wrapper, env-configurable timeouts, lifecycle receipts, warm default retained, concise source-grounded prompt suffix, and schema verification before adding config keys.

Claude contributes useful skepticism about unverified claims and gives precise commands to falsify receipt/timeout assumptions.

Kimi’s failed output still contains useful edge-case reasoning in stderr: outer Codex tool timeout may kill the main call while the imp child continues, `stdio: "inherit"` may reduce useful captured receipts, and default Stop hooks are likely not active.

The visible GLM output suggests it attempted source inspection, but the requested artifact is not present in the provided panel text, so it cannot be credited much.

Gemini’s output is off-task and should be ignored.

## Unique Insights

Kimi uniquely calls out that the process may continue after the main Codex tool call times out, creating orphaned or still-running imp work unless the wrapper kills the child on advisory timeout.

Kimi also identifies a subtle correction: self-improve lesson mode being enabled is not the same as Stop hook execution being enabled.

Claude uniquely suggests checking whether the 120s reject path aborts/kills the underlying Codex subprocess. Local source shows the reject path removes the handler and finishes the observer, but does not obviously kill the app-server or cancel the turn, making this a valid follow-up edge-case test.

Codex uniquely frames receipts as lifecycle receipts rather than success receipts, which is the right operational fix for “prior run left no useful receipts.”

## Blind Spots

The panel does not sufficiently specify child cleanup behavior for `spawnSync` timeout or async `spawn` timeout. The final plan should explicitly kill the child process on advisory timeout and record signal/duration.

No panel gives a concrete test for “outer timeout does not fall through to cold and exceed the budget.” This is important because warm failure currently falls back to cold at [isolated.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/isolated.ts:342).

The panel does not fully separate CLI modes: direct human imp runs may tolerate longer latency, but AGENTS-driven advisory pre-edit calls need a short budget.

No one clearly proposes preserving `--which` fast path behavior while changing run behavior. That should be included in implementation.

## Failure Notes

`kimi-code-high` failed with exit code 1, but its stderr includes meaningful partial reasoning. Treat it as partial evidence, not a completed panel answer.

`agy-gemini-flash-high` returned irrelevant content and did not answer the task.

`opencode-glm-5.2-high` is marked ok, but the provided visible output lacks the requested artifact. Unless a hidden artifact is available to the synthesizer, score it as non-responsive.

Claude’s tool limitations reduce its evidence value, but its skepticism and verification table are still useful.

## Recommended Synthesis

Recommend a small repo-local change set:

1. Keep `--which` unchanged.
2. Add an advisory timeout to `project-imp.ts`, preferably via async `spawn` or `spawnSync({ timeout })`, with `CODEX_IMP_ADVISORY_TIMEOUT_MS`.
3. On timeout, kill the child, exit non-zero, and write a lifecycle receipt.
4. Add env-configurable warm ready, RPC, turn, and cold SDK timeouts. Do not merely increase the hardcoded 120s for all paths.
5. Keep warm enabled by default, but make it fail within the advisory budget; `--no-warm` should remain a diagnostic escape hatch, not the default.
6. Add concise/structured advisory prompt framing to reduce wandering.
7. Do not add `startup_timeout_sec` / `tool_timeout_sec` to imp thread config. They are MCP server settings; verify exact current behavior with `codex --help`, `codex app-server --help`, `codex debug app-server --help`, `codex --strict-config`, or local Codex source/schema before adding any non-MCP timeout key.
8. Verify with a forced tiny timeout, receipt tail, `--which`, and the narrowest Bun/TS check available.

## Judge JSON

```json
{
  "scores": {
    "codex-gpt-5.5-high": {
      "correctness": 8,
      "task_fit": 9,
      "evidence": 8,
      "specificity": 9,
      "constraint_following": 9,
      "novelty": 7,
      "risk_awareness": 8,
      "cost_complexity": 8,
      "rationale": "Best complete answer; aligns with local source and advisory constraint, but overstates default hook risk slightly."
    },
    "claude-opus-4.8-high": {
      "correctness": 6,
      "task_fit": 4,
      "evidence": 5,
      "specificity": 6,
      "constraint_following": 7,
      "novelty": 5,
      "risk_awareness": 7,
      "cost_complexity": 6,
      "rationale": "Honest about lack of tools and gives useful falsification checks, but does not produce the requested substantive panel answer."
    },
    "agy-gemini-flash-high": {
      "correctness": 1,
      "task_fit": 1,
      "evidence": 1,
      "specificity": 1,
      "constraint_following": 1,
      "novelty": 1,
      "risk_awareness": 1,
      "cost_complexity": 1,
      "rationale": "Off-task output; no useful analysis of the imp timeout problem."
    },
    "kimi-code-high": {
      "correctness": 7,
      "task_fit": 5,
      "evidence": 6,
      "specificity": 6,
      "constraint_following": 4,
      "novelty": 7,
      "risk_awareness": 8,
      "cost_complexity": 6,
      "rationale": "Failed run, but stderr contains useful edge-case reasoning, especially default hook nuance and orphaned-child risk."
    },
    "opencode-glm-5.2-high": {
      "correctness": 3,
      "task_fit": 2,
      "evidence": 3,
      "specificity": 2,
      "constraint_following": 2,
      "novelty": 2,
      "risk_awareness": 3,
      "cost_complexity": 3,
      "rationale": "Visible output only shows process notes, not the requested artifact; cannot credit hidden analysis not supplied."
    }
  },
  "consensus": [
    "Unbounded router spawn plus layered warm/RPC/turn/cold waits are the likely operational cause.",
    "Imps should remain advisory with a bounded outer wrapper rather than becoming blockers.",
    "Lifecycle receipts should be written for startup, routing, timeout, fallback, exit, and errors, not only successful model turns.",
    "Do not blindly add startup_timeout_sec or tool_timeout_sec to imp thread config; they are best understood as MCP server timeout settings unless current Codex schema proves otherwise."
  ],
  "contradictions": [
    "Hook latency was treated by some as likely, but source supports it only as an opt-in Stop hook edge case, not the default path.",
    "Some suggestions imply increasing the 120s turn timeout, but the best-supported position is to bound advisory calls first and make longer timeouts opt-in for direct imp runs.",
    "Forcing --no-warm conflicts with preserving latency benefits; best-supported position is keep warm default but make warm failure bounded and observable."
  ],
  "unsupported_claims": [
    "Any exact non-MCP Codex config key for general turn timeout was not proven by the panel.",
    "Claims that receipts only write on success need source confirmation beyond the empty receipts directory and current observer behavior.",
    "Claims about provider/model behavior causing the timeout are plausible but not directly proven."
  ],
  "unique_insights": [
    "A main Codex tool-call timeout may leave the spawned imp child still running unless the wrapper kills it.",
    "Self-improve lesson mode is not equivalent to Stop hook execution; stopHook must be true.",
    "The 120s app-server turn reject path should be checked for whether it aborts the underlying turn or only rejects locally."
  ],
  "failure_notes": [
    "kimi-code-high failed but emitted useful partial reasoning in stderr; confidence is reduced only slightly because other evidence covers the core.",
    "agy-gemini-flash-high was non-responsive and should not influence synthesis.",
    "opencode-glm-5.2-high did not provide a visible requested artifact in the supplied panel output.",
    "Claude lacked source tools and therefore contributes checks, not verified conclusions."
  ],
  "confidence": "medium",
  "escalation_needed": false,
  "synthesis_instructions": [
    "Base the final answer primarily on Codex plus local source verification.",
    "Use Kimi's default-hook and orphaned-child edge cases as refinements.",
    "Treat Claude's table as a verification checklist, not a final recommendation.",
    "Ignore Gemini and do not rely on absent GLM artifact content.",
    "Recommend a small advisory-timeout wrapper, env-configurable inner timeouts, lifecycle receipts, and narrow forced-timeout verification."
  ]
}
```


