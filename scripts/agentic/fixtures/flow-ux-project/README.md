# Flow UX fixture project

Deterministic mdflow project for Flow UX probes (docs/ai/flow-ux-protocol.md).
Fake engine executables in `bin/` stand in for real LLM CLIs — mdflow resolves
an engine name to any executable on PATH, so every flow here runs instantly
and reproducibly with zero tokens.

## Usage

```bash
FIXTURE="$(git rev-parse --show-toplevel)/scripts/agentic/fixtures/flow-ux-project"
export PATH="$FIXTURE/bin:$PATH"              # fake engines resolvable
export SCRIPT_KIT_FLOW_UX_CWD="$FIXTURE"      # app-side flow discovery seam

# CLI smoke, no app needed:
cd "$FIXTURE"
md roster --json | jq '.flows[].id'
md flows/fast-success.fasteng.md --events
```

Launch the app with both env vars set and the Flow UX built-ins
(`flow-ux-flash` / `dispatch` / `lens` / `mission-control`) list these flows.

## Flows

| Flow | Engine | Proves |
| --- | --- | --- |
| `fast-success.fasteng` | fasteng (exit 0, instant) | launch ack + terminal latency |
| `streaming-output.streameng` | streameng (10 lines / ~1s) | incremental `output.delta` |
| `slow-cancellable.sloweng` | sloweng (60s heartbeats + child sleeper) | cancel: SIGTERM→SIGKILL, dead process group |
| `failing-flow.faileng` | faileng (exit 42, stderr) | `run.error` exit-code surfacing |
| `input-matrix` | fasteng | all 5 input types; required password has NO default → `--events` must fail closed (input honesty) |
| `input-defaults` | fasteng | password WITH default `FIXTURE-SECRET-TOKEN-9F2` — that string must never surface in app state/elements |
| `workflow-dag` | fasteng (3-step `_steps`) | `step.started`/`step.completed` ordering |
| `stubborn-cancel.stubborneng` | stubborneng (traps SIGTERM) | 2s SIGKILL escalation still kills the group |
| `giant-line.gianteng` | gianteng (~128KB single line) | oversized lines truncated for display, never dropped |
