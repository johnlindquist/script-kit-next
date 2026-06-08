---
name: agy-script-kit-devtools
description: >-
  Use the local agy CLI as a fast Script Kit GPUI app inspector by prompting it
  to drive existing script-kit-devtools primitives, capture logs, and produce a
  compact investigation result from a user bug report or inspection prompt.
---

# agy Script Kit DevTools

This skill is the fast-agent companion to `$script-kit-devtools`. It does not
replace the direct DevTools contract; it wraps it with an `agy` prompt so a
faster model can infer the likely surface, run the existing `scripts/devtools/*`
CLI primitives, and return a compact inspection report.

Use this when the user wants `agy` to inspect Script Kit GPUI from a prompt, or
when a quick first-pass app investigation is useful before deeper Codex or
Oracle work.

## Front Door

Run the wrapper from the repo root:

```bash
scripts/agentic/agy-devtools.sh run --prompt "The actions dialog shortcuts overlap on notes" --surface actions-dialog
```

Important options:

- `--prompt <text>`: required user report or inspection request.
- `--prompt-file <path>`: read the user request from a file.
- `--surface <id>`: optional hint such as `main`, `actions-dialog`, `notes`, `dictation`, `agent_chat`, `file-search`, or `settings`.
- `--session <name>`: DevTools session name hint passed to `agy`; defaults to `agy-devtools`.
- `--trust-repo`: pass `--dangerously-skip-permissions` to `agy`; use only after reviewing the prompt and safety gates.
- `--agy-sandbox on|off`: keep sandbox on by default; turn it off only when `agy` needs to run local DevTools commands.
- `--fast`: use a short command-budget prompt for known action flows, especially demos where source exploration would be noise.
- `--allow-act`, `--allow-submit`, `--allow-native`, `--allow-mic`, `--allow-real-data`: explicit opt-ins for actions, submit/Enter flows, native input, live microphone, or real-data mutation.

The wrapper writes each run under `.test-output/agy-devtools/<run-id>/`:

- `input.txt`: the raw user request.
- `inference.json`: deterministic surface, target, safety, and primitive-stack inference.
- `prompt.md`: the exact prompt sent to `agy`.
- `agy.log`: the CLI log from `--log-file`.
- `agy.stdout.md` and `agy.stderr.log`: raw `agy --print` output streams.
- `result.json`, `result.md`, and `compact.md`: final result files.
- `receipts/`: expected location for any DevTools JSON receipts that `agy` creates.

## Subcommands

- `run`: build prompt, invoke `agy`, save logs/results, and print compact output.
- `infer`: infer surface, target, safety gates, and primitive stack as JSON.
- `prompt`: write and print the exact `agy` prompt without invoking `agy`.
- `compact`: print compact summary for an existing `--run-dir`.
- `cleanup`: stop a named wrapper-owned DevTools session.

## agy Contract

`agy` must stay inside the existing DevTools layer:

1. Infer the likely surface and target from the user prompt and any `--surface`
   hint.
2. Start with source-backed orientation commands such as
   `bun scripts/devtools/investigate.ts`, `inspect.ts`, `targets.ts`,
   `elements.ts`, `layout.ts`, `focus.ts`, `text.ts`, `scroll.ts`,
   `keyboard.ts`, `events.ts`, or `coverage.ts`.
3. Use `scripts/agentic/devtools-session.sh` when a real app session is needed.
4. Write JSON receipts to the provided `receipts/` directory.
5. Produce a final Markdown report with classification, commands run, receipt
   paths, key findings, likely owner files, and next verification.

## Safety Rules

- Default mode is inspect-only. Do not submit forms, press Enter on selected
  rows, invoke destructive built-ins, or use native input unless the wrapper
  prompt includes the matching allow flag.
- If a required primitive is missing, classify the run as
  `blocked-by-missing-primitive` and name the exact missing primitive.
- Do not call a screenshot-only observation green for a behavior bug. Green
  means the same failed user-path measurement now passes.
- Treat user interruption, hidden windows, missing target identity, and stale
  sessions as explicit classifications, not generic failures.

## Verification

For wrapper changes, run:

```bash
scripts/agentic/agy-devtools.sh --help
scripts/agentic/agy-devtools.sh infer --prompt "Cmd-K opens the actions popup but it is clipped" --surface auto
scripts/agentic/agy-devtools.sh run --dry-run --prompt "inspect the main menu filter" --surface main
scripts/agentic/agy-devtools.sh run --dry-run --fast --prompt "Open Agent Chat and submit what's for lunch" --surface agent_chat --allow-act --allow-submit
scripts/agentic/agy-devtools.sh run --dry-run --fast --prompt "Open the theme designer, change the accent color to red" --surface theme --allow-act --allow-submit --allow-real-data
```

If behavior changes beyond prompt assembly, add the smallest source or runtime
proof that can fail for the changed contract.
