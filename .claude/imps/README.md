# Script Kit Project Imps

Project imps are feature-bound Codex specialists for this repository. They use
the copied `codex-imps` runtime so warm daemons, hot reload, local lesson
overlays, failure classification, pruning, and promotion workflows stay real.

Every feature imp is configured for `gpt-5.5` with `medium` reasoning. If that
model is unavailable, the imp should fail visibly instead of silently
downgrading.

## Commands

```bash
cd .agents/imps
npm install
bun imps/project-imps list
bun imps/project-imp --which "fix @file attachment in Agent Chat"
bun imps/project-imp "fix @file attachment in Agent Chat"
bun imps/imp-sk-agent-chat "fix @file attachment in Agent Chat"
```

`project-imp` routes by registry triggers and owner paths. It is an advisory
entry point with a progress watchdog: the routed specialist may run as long as
it keeps emitting useful stdout/stderr progress, but a silent/stuck run is
stopped after the progress timeout. Direct `imp-sk-*` commands skip routing and
run the named specialist without the router's watchdog.

Timeouts can be tuned per invocation:

```bash
SCRIPT_KIT_IMP_PROGRESS_TIMEOUT_MS=180000 bun imps/project-imp "review Agent Chat handoff"
bun imps/project-imp --progress-timeout-ms 0 "let the routed specialist run even if silent"
bun imps/project-imp --max-runtime-ms 600000 "hard-stop after ten minutes"
```

Additional warm-runtime knobs:

- `SCRIPT_KIT_IMP_PROGRESS_TIMEOUT_MS` controls how long the routed imp may be silent before `project-imp` stops it (default `120000`). Legacy `SCRIPT_KIT_IMP_ADVISORY_TIMEOUT_MS` and `--timeout-ms` are accepted as aliases for this progress timeout.
- `SCRIPT_KIT_IMP_MAX_RUNTIME_MS` optionally caps total routed imp runtime (default `0`, disabled).
- `SCRIPT_KIT_IMP_READY_TIMEOUT_MS` controls warm daemon startup readiness (default `30000`).
- `SCRIPT_KIT_IMP_START_TIMEOUT_MS` controls app-server JSON-RPC handshakes such as `thread/start` (default `60000`).
- `SCRIPT_KIT_IMP_TURN_TIMEOUT_MS` controls a warm imp turn before it returns `turn timeout` (default `120000`).

Router receipts are written to `receipts/<imp>.jsonl` with status, elapsed time,
progress count, timeout settings, and prompt hash.

## Self-Improvement

Local lessons live under `lessons/local/` and receipts under `receipts/`; both
are git-ignored. Lessons are folded into the imp developer instructions and are
included in the warm-daemon fingerprint, so the next run restarts with new
lessons active.

Promotion is manual and reviewed:

- repeated command or workflow failure -> permanent imp prompt
- cross-cutting repo rule -> `AGENTS.md`
- user-visible regression -> focused test or runtime probe
- durable product/domain assumption -> owning docs or `.notes`
- one-off local failure -> stays local until pruned

Tracked examples live in `evals/cases/`.
