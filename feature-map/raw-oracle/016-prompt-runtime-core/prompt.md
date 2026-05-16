# 016 Prompt Runtime Core Prompt

```text
[prompt-runtime-core-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 016: Prompt Runtime Core / arg() / select() / div() / md() / prompt handler routing.

This atlas must cover the human and agent behavior for the core SDK prompt surfaces:

- TypeScript SDK APIs: `arg()`, `select()`, `div()`, `md()`.
- Rust prompt handler routing into `AppView::ArgPrompt`, `AppView::SelectPrompt`, and `AppView::DivPrompt`.
- Prompt ids, submit callbacks, focused input state, prompt type/state identity, and return values.
- Arg prompt text input, placeholder/config overloads, choices, actions, Enter submit behavior, empty choices, and actions menus.
- Select prompt list behavior, multi-select/toggle behavior, submit behavior, keyboard ownership, and state receipts.
- Div prompt HTML/Markdown rendering, link/submit behavior, actions, no text-input focus, and md-to-div integration.
- Prompt actions and `ActionsDialogHost` ownership for these surfaces.
- Automation and protocol receipts: `getState`, `getElements`, prompt type, semantic ids, current prompt id, `simulateKey`, Enter parity, and safe submit paths.
- Data/privacy boundaries: typed user text, choice metadata, rendered HTML/Markdown, action payloads, and return values.
- Error, empty, disabled, loading, and stale async states.

Important boundaries:

- SDK TermPrompt is feature 015 and should only be referenced as adjacent.
- Quick Terminal is feature 014 and should not be mixed into this prompt-runtime chapter.
- ACP Chat is separate from these core prompt surfaces.
- Keep recommended proofs marked as recommended unless the raw bundle proves they ran.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing
- `lat.md`: design, surfaces, protocol, scripting, verification
- Source: SDK globals, prompt handler route, app view state, arg/div/select renderers, select prompt internals, markdown/div internals, element collection, simulateKey routing
- Tests/scripts: SDK arg/select/div/md/prompt-flow tests and smoke tests for arg/div/select actions and submit behavior

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 016 Prompt Runtime Core / arg() / select() / div() / md()

### Executive Summary

### What Users Can Do

### Core Concepts

### Entry Points

### User Workflows

### Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|

### State Machine

### Visual And Focus States

### Keystrokes And Commands

### Actions And Menus

### Automation And Protocol Surface

### Data, Storage, And Privacy Boundaries

### Error, Empty, Loading, And Disabled States

### Code Ownership

### Invariants And Regression Risks

### Verification Recipes

### Agent Notes

### Related Features

### Open Questions And Gaps

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
```
