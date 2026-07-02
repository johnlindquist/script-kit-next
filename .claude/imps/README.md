# Script Kit Project Imps

Project imps are feature-bound Codex specialists for this repository, built on
the vendored `codex-imps` runtime (warm daemons, hot reload, evolution
suggestions, transcripts). The vendored runtime files and their hashes are
tracked in `imps.manifest.json`; re-vendor from `~/dev/codex-imps` and refresh
the manifest rather than patching `lib/*.ts` in place. Project-owned code is
`lib/project-config.ts`, `bin/`, `registry.json`, and the `imps/` shims.

Every imp is configured for `gpt-5.5` with `medium` reasoning. If that model is
unavailable, the imp must fail visibly instead of silently downgrading.

## Fleet

Feature owners: `imp-sk-launcher`, `imp-sk-components`, `imp-sk-agent-chat`,
`imp-sk-brain`, `imp-sk-clipboard`, `imp-sk-prompts`, `imp-sk-actions`,
`imp-sk-hotkeys`, `imp-sk-builtins`, `imp-sk-terminal`, `imp-sk-execution`,
`imp-sk-platform`, `imp-sk-mcp`, `imp-sk-ai-core`, `imp-sk-settings`.

Role imps (cross-cutting): `imp-sk-scout` (read-only routing/intake),
`imp-sk-devex` (repo process/probes), `imp-sk-build-doctor` (build/toolchain
failures), `imp-sk-devtools` (DevTools operator — the imp form of the
`script-kit-devtools` skill: driver/inspect/investigate primitives, fail-closed
receipts that feed `oracle-packx-conversation` bundles), `imp-sk-auditor`
(read-only audit sweeps), `imp-sk-tests` (test authorship/policy),
`imp-sk-release` (tag/clippy-gate pipeline).

`registry.json` is the single source of truth for owners, edit globs, routing
triggers, verification gates, and per-imp command-map/error-recovery extras.
Prompts are generated from it in `lib/project-config.ts` following the
upstream prompt standard (Mission → trust boundary → operating rule → command
map → workflow → mutation policy → worked examples → error recovery → command
rules → output).

## Commands

```bash
cd .agents/imps
bun install
bun imps/project-imps list                 # roster + lesson status
bun imps/project-imps lessons <imp-name>   # show local lessons
bun imps/project-imps paths                # owner globs per imp
bun imps/project-imp --which "<task>"      # routing dry-run
bun imps/project-imp "<task>"              # route + run (non-interactive)
bun imps/imp-sk-agent-chat --run "<task>"  # direct, non-interactive
bun imps/imp-sk-agent-chat "<task>"        # direct, interactive Codex TUI
bun imps/imp-sk-agent-chat evolve          # review evolution suggestions
bun run check                              # bundle-check router/fleet/sample imps
```

Direct `imp-sk-*` invocations open an interactive Codex TUI by default (new
upstream behavior). Agents and scripts must pass `--run` (streaming) or `-q`
(quiet) — the `project-imp` router always adds `--run` itself. `--effort
<level>` and `--no-warm` are also available per invocation.

`project-imp` routes by registry triggers and owner paths. It is an advisory
entry point with a progress watchdog: the routed specialist may run as long as
it keeps emitting useful stdout/stderr progress, but a silent/stuck run is
stopped after the progress timeout. Direct `imp-sk-*` commands skip routing and
run the named specialist without the router's watchdog.

Timeouts can be tuned per invocation:

```bash
SCRIPT_KIT_IMP_PROGRESS_TIMEOUT_MS=600000 bun imps/project-imp "review Agent Chat handoff"
bun imps/project-imp --progress-timeout-ms 0 "let the routed specialist run even if silent"
bun imps/project-imp --max-runtime-ms 600000 "hard-stop after ten minutes"
```

Warm-runtime knobs (bridged to the upstream `CODEX_IMP_*` variables by
`lib/project-config.ts`):

- `SCRIPT_KIT_IMP_PROGRESS_TIMEOUT_MS` controls how long the routed imp may be silent before `project-imp` stops it (default `600000`). Legacy `SCRIPT_KIT_IMP_ADVISORY_TIMEOUT_MS` and `--timeout-ms` are accepted as aliases.
- `SCRIPT_KIT_IMP_MAX_RUNTIME_MS` optionally caps total routed imp runtime (default `0`, disabled).
- `SCRIPT_KIT_IMP_READY_TIMEOUT_MS` controls warm daemon startup readiness (default `120000`; `CODEX_IMP_READY_TIMEOUT_MS` also works).
- `SCRIPT_KIT_IMP_START_TIMEOUT_MS` controls app-server JSON-RPC handshakes such as `thread/start` (default `180000`; `CODEX_IMP_START_TIMEOUT_MS` also works).
- `SCRIPT_KIT_IMP_TURN_TIMEOUT_MS` controls a warm imp turn (project default `1800000` — cargo gates here routinely exceed the upstream 300s default; `CODEX_IMP_TURN_TIMEOUT_MS` also works).

Router receipts are written to `receipts/<imp>.jsonl` with status, elapsed time,
progress count, timeout settings, and prompt hash.

## Learning: evolution + reviewed lessons

The upstream runtime replaced automatic self-improve lesson writing with the
review-owned **evolution** system: runs record reviewable suggestions under
`~/.imp/<imp-name>/`, a status line reports pending suggestions, and
`imp-sk-<name> evolve` opens an interactive review walkthrough. Prefix a prompt
with `+reason` to flag a turn for evolution review, or start it with `^` to run
a maintainer-instruction turn.

Local lessons under `lessons/local/*.lessons.md` remain supported as **reviewed
overlays**: `lib/project-config.ts` folds them into the imp's developer
instructions at launch (capped by `lessonDefaults.maxLessonBytes`, most recent
kept). Write lessons deliberately — from evolution reviews, receipts, or user
feedback — instead of expecting the runtime to append them automatically.

Promotion is manual and reviewed:

- repeated command or workflow failure -> permanent registry `commandMap`/`errorRecovery` entry
- cross-cutting repo rule -> `AGENTS.md`
- user-visible regression -> focused test or runtime probe
- durable product/domain assumption -> owning docs or `.notes`
- one-off local failure -> stays in `lessons/local/` until pruned

Lessons and receipts are git-ignored. Tracked examples live in `evals/cases/`.
