# Local search and Jev readings

The launcher works locally by default. For optional remote readings, set `jevIntentEnabled: true` in `~/.scriptkit/config.ts` and supply `TYPESAFE_API_KEY` through the environment or Script Kit’s secret store. Restart Kit after changing credentials. `SCRIPT_KIT_JEV_INTENT=0` disables readings; `=1` explicitly opts in for that launch. No checkout of the Jev demos is needed.

**What leaves the machine:** when typing pauses, TypeSafe can receive the words typed so far, the front app name, app/menu choices and shortlists of command and script names and descriptions. With voice readings enabled by the same setting, it can receive partial and final transcripts. A sentence can incur several paid requests; a compound request can require one per clause. The live launcher does not put window titles, clipboard or selection contents, stored facts or prior action summaries in the screen description. Labels themselves can be private. Recognizable credentials are withheld, but no pattern detector catches every secret. Leave remote readings disabled for sensitive work. Opening chat or explicitly giving work to an agent is a separate action.

Typed launcher syntax and exact local name matches do not need a remote reading. An unsupported service cannot connect or send a message from the launcher. Saving a fact writes it to today’s note; a fact-recall reading opens the existing local `brain:` search. A match is not a generated answer. Quit requests for another app wait in the cancellable hold before dispatch.

## Measuring readings

The default model is `jev-1.14.0`; `TYPESAFE_MODEL` is an explicit override. The historical replay measures deterministic composition, not current model accuracy. A separate ten-sentence synthetic holdout checks the selected app, menu, window action, capture body, quit hold classification or unavailable result. No private lessons file is uploaded. A successful test run that only generates requests is not a live score.

```sh
export SCRIPT_KIT_JEV_HOLDOUT_DIR="$(mktemp -d)"
./scripts/agentic/agent-cargo.sh test --lib intent::holdout_tests
# Inspect requests.json first. This next command uses your TypeSafe account.
bun scripts/agentic/one-window/record-jev-answers.ts --score-holdout "$SCRIPT_KIT_JEV_HOLDOUT_DIR"
./scripts/agentic/agent-cargo.sh test --lib intent::holdout_tests
```

To sample typing rather than complete sentences, the same test emits `typing/requests.json` from a synthetic reducer trace: 51 characters at 30 ms per key, with three 250 ms pauses. `bun scripts/agentic/one-window/record-jev-answers.ts --score-typing "$SCRIPT_KIT_JEV_HOLDOUT_DIR/typing"` makes the three paid prefix requests and writes their timings separately.

`score.json` contains the expected and observed results; `timing.json` contains per-request time, input tokens and nearest-rank p50/p95. These are decision round trips for complete sentences in a small synthetic catalog, not per-keystroke billing estimates or end-to-end UI latency. Use a fresh directory for a deliberately new paid sample. To evaluate personal shortcuts, curate and anonymize examples separately rather than sending your learning history.
