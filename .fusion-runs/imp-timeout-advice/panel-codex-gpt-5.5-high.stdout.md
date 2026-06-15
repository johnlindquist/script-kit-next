## Role Findings

As edge-case tester: the biggest risk is not one timeout, but stacked unbounded waits. `project-imp.ts` blocks on `spawnSync` with no timeout, warm startup can spend 30s before falling back, app-server RPC waits can spend 60s, warm turns have a 120s timeout, and cold SDK fallback has no local deadline. If the main Codex tool call has its own timeout, it can kill the whole attempt before the imp harness writes a useful receipt.

Most likely causes:

- Outer tool timeout: `spawnSync(..., stdio: "inherit")` means the main agent waits until the imp exits.
- Warm startup failure: `ensureWarmImp(..., 30000)` plus app-server `initialize` can miss the caller’s patience window.
- Warm turn timeout: `runTurn` hard-codes 120s, likely tight for `gpt-5.5` + `medium` + mandated repo inspection.
- Cold fallback hang: if warm path fails, SDK quiet/stream path has no repo-local timeout.
- Queueing: warm server serializes turns; one slow turn delays later callers.
- Self-improve hooks: Stop hook is only 10s, but still adds tail latency and can hide “model finished but process not done” cases.
- Receipts gap: receipts appear tied to self-improve observer events, not harness lifecycle events, so startup/connect/spawn/cold-fallback failures can leave no useful record.

## Evidence And Assumptions

Current source supports the above:

- [project-imp.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/bin/project-imp.ts:36) uses blocking `spawnSync` without a timeout.
- [imp.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/imp.ts:302) defaults warm readiness to 30s.
- [appserver.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/appserver.ts:110) defaults JSON-RPC response waits to 60s.
- [appserver.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/appserver.ts:180) hard-codes the warm turn timeout to 120s.
- [isolated.ts](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/lib/isolated.ts:355) cold fallback starts after warm failure and has no equivalent bounded timer.
- [registry.json](/Users/johnlindquist/dev/script-kit-gpui/.agents/imps/registry.json:3) sets `gpt-5.5` with `medium` reasoning.
- Receipts directory currently exists but is empty.

`startup_timeout_sec` and `tool_timeout_sec` were found in the user config under `[mcp_servers.*]`, so I would treat them as MCP server startup/tool-call settings, not app-server turn settings. They belong in MCP server config blocks such as `[mcp_servers.node_repl]`, not in imp `thread/start` config unless Codex schema confirms otherwise.

## Failure Modes

Increasing only the 120s turn timeout can make main Codex more likely to hit its own tool timeout with no better receipt.

Forcing `--no-warm` avoids warm socket/app-server bugs, but makes every imp pay cold startup and still lacks a cold timeout.

Lowering reasoning effort improves latency, but may reduce advisory quality. Better default: keep registry effort configurable and allow bounded wrapper overrides like `--effort low` for advisory preflight.

Async imps are useful only if they produce a discoverable receipt. Fire-and-forget without receipt state creates silent failures.

Hooks should stay disabled except self-improve. Even self-improve hook failures need lifecycle receipt entries so hook stalls are distinguishable from model stalls.

## Recommendation

Implement a bounded advisory wrapper around `project-imp` and make timeouts env-configurable.

Concrete repo changes:

1. Add router flags/env:
   - `CODEX_IMP_ADVISORY_TIMEOUT_MS`, default maybe `45000`.
   - `CODEX_IMP_WARM_READY_TIMEOUT_MS`, default `10000` or `15000`.
   - `CODEX_IMP_RPC_TIMEOUT_MS`, default `30000`.
   - `CODEX_IMP_TURN_TIMEOUT_MS`, default `90000` for advisory mode, maybe `180000` for direct imp CLI.
   - `CODEX_IMP_COLD_TIMEOUT_MS`, default bounded too.

2. Change `project-imp.ts` from unbounded `spawnSync` to `spawnSync(..., { timeout })`, or use async `spawn` plus timer so it can write a receipt before killing.

3. Add harness lifecycle receipts independent of self-improve:
   - routed imp
   - started warm/cold
   - warm startup timeout
   - turn timeout
   - outer advisory timeout
   - exit code/signal/duration
   Put these in `.agents/imps/receipts/project-imp.jsonl` or per-imp files.

4. Keep warm default, but fail fast to cold or fail advisory within the wrapper budget. Do not make imps blockers.

5. Add a prompt suffix for advisory calls: “return concise source-grounded findings, no edits, stop after owner files and verification suggestions.” This preserves usefulness while reducing exploratory drift.

6. Do not add `startup_timeout_sec` / `tool_timeout_sec` to imp thread config until verified against Codex schema/source. Use `codex app-server generate-json-schema`, `codex app-server --help`, `codex debug app-server --help`, and `--strict-config` smoke tests to confirm accepted keys.

Verification commands:

```bash
cd .agents/imps
bun imps/project-imp --which "shared UI button timeout smoke"
CODEX_IMP_ADVISORY_TIMEOUT_MS=1000 bun imps/project-imp "shared UI button timeout smoke"
ls -la receipts && tail -20 receipts/*.jsonl
CODEX_IMP_WARM_READY_TIMEOUT_MS=1 bun imps/imp-sk-devex -q "Say one sentence about AGENTS.md"
```

Then run the smallest TS check available for changed imp files, for example:

```bash
bun --check lib/appserver.ts lib/imp.ts lib/isolated.ts bin/project-imp.ts
```

## Self Score

8/10. The timeout architecture is clear from local source, but exact Codex config-key semantics need Codex schema/source verification before adding non-MCP timeout keys to imp config.


