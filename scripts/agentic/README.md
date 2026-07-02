# scripts/agentic — map of what's living vs. archived proof

~170 files live here, but they are two very different populations. A dozen or
so are **living infrastructure** every agent session depends on; the rest are
**one-shot probes** written to red/green-prove a specific bug or feature pass.
Probe file mtimes are unreliable (bulk reformats touch everything), so use
this manifest — not timestamps — to tell them apart.

The modern interactive tooling lives in `scripts/devtools/` (driver +
fail-closed receipt CLIs), not here. Read
`.agents/skills/script-kit-devtools/SKILL.md` first for the investigation
workflow; this directory is its supporting cast.

## Living infrastructure

Build / dev loop:

- `agent-cargo.sh` — the MANDATED cargo wrapper for agents (shared pool,
  visible lock, disk budget, APFS artifact clones). Never run bare cargo
  while `./dev.sh` may be running.
- `dev-cycle.sh`, `dev-relaunch.sh`, `dev-crash-watchdog.sh` — the build →
  relaunch → heartbeat loop `./dev.sh` delegates to. The crash watchdog is
  opt-in (`SCRIPT_KIT_DEV_CRASH_WATCHDOG=1`) so Quit during dev stays quit;
  use `bun scripts/devtools/events.ts crashes` to diagnose a dead app.
- `prune-cargo-targets.sh`, `disk-space-cargo-*` — disk-pressure subsystem
  (watcher + emergency clean). Note: <25GiB free can SIGTERM agent builds.
- `build-isolated-binary.sh`, `start-isolated.sh`, `preflight-isolated.sh` —
  isolated-binary path for runs that must not share `target/`.
- `ensure-pi-sidecar.sh` — resolves the Pi sidecar for non-bundle dev builds;
  `./dev.sh` runs it automatically. Without it, Agent Chat shows
  "Pi Agent Chat is unavailable" (an environment gap, not an app bug).
- `seed-sandbox-home.sh` — one-call sandbox auth seeding for live Agent
  Chat/brain probes (APFS-clones `~/.pi` + `~/.codex/{auth.json,config.toml}`);
  also available as `Driver.launch({ seedAgentAuth: true })`.

Session control (legacy named-session layer; prefer
`scripts/devtools/driver.ts` for anything multi-step):

- `session.sh` — start/send/status/stop a named app session over stdin FIFO.
  Sessions are addressed purely by name: parallel loops MUST use loop-unique
  names.
- `devtools-session.sh`, `devtools-session-lib.sh`, `session-state.ts`,
  `session-supervisor.py`, `wait-session-ready.sh`,
  `verify-devtools-session.sh` — session plumbing and health checks.
- `driver-benchmark.ts` — throughput receipt proving the driver's speedup
  over session.sh round trips.
- `agy-devtools.sh` — wrapper the `agy-script-kit-devtools` skill drives for
  fast-model first passes.
- `smoke-main-menu.sh` — quick launch smoke.

Shared engine modules imported by probes (large; edit with care):

- `scenario.ts` — scenario runner primitives most probes build on.
- `index.ts` — recipe orchestrator (the `agentic-testing` entry point).
- `verify-shot.ts` — screenshot capture + identity verification.
- `macos-input.ts`, `window.ts`, `macos-window-query.swift` — native input
  and window query helpers (escalation beyond protocol).
- `surface-navigator.ts`, `automation-window.ts`, `await-response.ts`,
  `target-thread.ts` — targeting/navigation/response helpers.
- `mock-pi-rpc.js` — mock Pi RPC endpoint for Agent Chat probes without a
  live agent.

## One-shot probes (everything else)

Named `<surface>-<behavior>-probe|proof|matrix|benchmark.ts`. Each exists
because some bug or feature pass needed a red/green receipt once; they are
kept as regression packs and as worked examples of driving a surface.

Families, by prefix:

- `brain-*` — brain capture/recall/inbox/substrate probes.
- `day-*`, `day-page-*` — Day Page editor, handoff, spine, perf.
- `notes-*` — Notes window, popups, embedded Agent Chat.
- `mini-*` — mini AI window sizing/dismiss/handoff.
- `main-*` — main window focus, hotkeys, actions sizing.
- `root-*` — main-menu perf benchmarks and source-filter matrices
  (`root-source-filter-matrix.ts` is the canonical template for porting a
  session.sh probe to the driver).
- `vibrancy-*` — window backdrop/material measurement.
- `verify-shot-*` — screenshot verification edge cases.
- Everything else: one surface each (actions popup, footer, HUD, confirm,
  clipboard, dictation, theme, process manager, ...).

Ground rules:

- Before writing a new probe, search this directory for the surface prefix —
  extending an existing probe beats minting a near-duplicate.
- New probes should import `Driver` from `scripts/devtools/driver.ts`
  (`sandboxHome: true`, `waitForSettle()` instead of sleeps, try/finally
  `close()`), print one JSON receipt, and exit nonzero on failure.
- Throwaway feature-verification scripts belong in your scratchpad, not
  here. Commit a probe only when it proves a behavior worth re-proving.
