Panel-specific reasoning contract:
Panel role: architect
Focus on the complete design, tradeoffs, implementation shape, and how the pieces fit together.

Return your answer with these headings:
## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score

Original task:
# Context

We are in /Users/johnlindquist/dev/script-kit-gpui. The repo has advisory "project imps" under `.agents/imps`: small feature-bound Codex specialists that the main agent can call before editing an owned surface. `AGENTS.md` says they are advisory, not blockers — continue normally if unavailable or too slow.

The imp harness is in `.agents/imps/lib/*.ts` and the router is `.agents/imps/bin/project-imp.ts`. Observed behavior:

- `cd .agents/imps && bun imps/project-imp --which "<task>"` returns the selected imp (e.g. `imp-sk-components`) quickly.
- Actually running the selected imp sometimes appears to time out / not return in time during a Codex task, so the main agent continues without it.
- The prior run did not leave useful receipts under `.agents/imps/receipts/`.

## Relevant source facts

- `.agents/imps/bin/project-imp.ts` uses Node `spawnSync(command, [prompt], { cwd: repoRoot, stdio: "inherit", env: process.env })` and does not set its own timeout.
- Each imp executable delegates to `runImp(makeProjectImpConfig("imp-sk-..."))` in `.agents/imps/lib/isolated.ts`.
- `.agents/imps/lib/isolated.ts` warm path auto-starts/reuses a background app-server imp unless `--no-warm` is passed. It falls back to a cold in-process SDK run if warm routing fails.
- `.agents/imps/lib/appserver.ts` has `awaitResponse(id, timeoutMs = 60000)` for JSON-RPC handshake / `thread/start`, and `runTurn` hard-codes `setTimeout(..., 120000)` that rejects with `turn timeout`.
- `.agents/imps/lib/project-config.ts` starts threads with config including `model_reasoning_effort`, disables skills/apps/environment/collab/permissions instructions, sets `project_doc_max_bytes: 0`, disables memories/MCP/web_search, and turns off features (plugins, hooks unless self-improve, memories, apps, image_generation, tool_search, tool_suggest).
- `.agents/imps/lib/codex-runtime.ts` writes an isolated `config.toml` only when self-improve is enabled: `bypass_hook_trust = true`, `[features] hooks = true`, and a `hooks.json` with a Stop hook `timeout: 10`.
- `.agents/imps/lib/isolated.ts` interactive flags include `--dangerously-bypass-approvals-and-sandbox`, `-c model_reasoning_effort=...`, `-c sandbox_mode=...`, `-c approval_policy="never"`.

## Questions

1. What are the most likely timeout causes here? Consider both harness-side (spawnSync, app-server warm start, JSON-RPC handshake, hard-coded turn timeout, model reasoning effort, self-improve hooks) and Codex-side (tool-call timeouts, turn limits, model/provider behavior, warm vs cold path).
2. What concrete repo changes should we make to avoid imp timeouts while preserving imp usefulness and keeping the main Codex unblocked? Do not make imps blockers. Prefer repo-local fixes with small verification.
3. Are there Codex TOML / `-c` configuration keys related to tool-call timeouts, turn timeouts, hooks, app-server, or model/tool behavior that should be added to the imp isolated config or thread/start config? If you know exact key names, give them. If not, say which local command/doc source to inspect to verify. We already see `startup_timeout_sec` and `tool_timeout_sec` in the Codex binary strings; clarify what they control and where they belong.
4. Should we increase the hardcoded 120s turn timeout, add env-configurable timeouts, force `--no-warm`, change reasoning effort, add quiet/structured short-response prompts, run imps asynchronously, add receipts, or use a bounded advisory wrapper from the main agent? Evaluate each option against the constraint that imps must remain advisory and not block the main agent.
5. Propose a small implementation plan with verification commands.

## Constraints

- Do not recommend narrowing Fusion providers; full panel is expected.
- Do not make imps blockers. `AGENTS.md` says continue if unavailable/too slow.
- Prefer repo-local fixes with small verification.
- Preserve dirty work. Avoid unrelated refactors.

Please answer as an independent panel member with the role **edge-case-tester**: focus on boundary cases, regressions, weird inputs, operational failure modes, and test coverage gaps. Return findings under these headings:

## Role Findings
## Evidence And Assumptions
## Failure Modes
## Recommendation
## Self Score