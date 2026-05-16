# AFK Audit User Stories











User reported two bugs after Run 12 stop. Both fixed in Run 13 Pass 1+2 (single commit) using the AFK loop pattern (oracle → inline impl → screenshot).








## Power Syntax completion (focus for Run 11)










## Cmd+K actions menu consistency (focus for Run 7+)









## Completed baselines


## Cross-surface workflows (state survives boundary crossings)


## ACP lifecycle and streaming


## Detached / popup surface threading (capture identity, mutation routing)


## Portal contracts (round-trip preserves identity)


## Edge cases (empty, overflow, invalid)


## Concurrency and race conditions


## Tool self-tests (verify the harness itself)


## Generated

(Loop appends new stories below this line once the above are exhausted.)


These stories are deliberately ordered so the first ~3 close the highest-leverage agentic-testing gaps from Run 1 before verification work continues. Each tool-extension story is paired with a verification story so the new tool is exercised end-to-end.


### Run 2 ambitious additions — lifecycle, concurrency, edge



























































---

## Legend

- `[ ]` — not yet attempted.
- `[x]` — pass proven (possibly with a committed fix).
- `[!]` — story could not be verified with current tools; tooling gap recorded in log.
- `[-]` — story skipped (would require a forbidden action per scope.md).

### Tool-gap backlog (promoted from log)




### Attacker-mode anomalies



## Run 10 — Cross-Surface Workflow Hardening seed (2026-04-22)
