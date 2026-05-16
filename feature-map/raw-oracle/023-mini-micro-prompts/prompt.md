# 023 Mini and Micro Prompts

```text
[mini-micro-prompts-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 023: Mini and Micro Prompts / `mini()` / `micro()` / compact arg-like choice prompts.

This atlas must cover:

- TypeScript SDK APIs: `mini(placeholder, choices)` and `micro(placeholder, choices)`, return shapes, current warnings/stub comments, and any real message send behavior.
- Current product truth: reconcile SDK warning comments with Rust `PromptMessage::ShowMini`, `ShowMicro`, `AppView::MiniPrompt`, and `AppView::MicroPrompt`.
- MiniPrompt internals: compact minimal-list shell, choices, placeholder, input/filter, selection, Enter submit, focus, native footer, sizing via `ViewType::MiniPrompt`, and app/automation proof.
- MicroPrompt internals: ultra-compact prompt, footerless behavior, choices, selection, filter, Enter submit, sizing, and why it must remain distinct from microphone/media stubs.
- Shared arg-like behavior: choice filtering, selected values, state reporting, getElements, selectByValue/selectFirst if supported, simulateKey Enter/Escape, inputValue contract, and Tab AI context.
- Footer and windowing: Mini has native footer surface / minimal list footer; Micro stays footerless/off native-footer routing; MiniPrompt sizing must not inherit full ArgPrompt width.
- Relationship to Mini main window and Mini AI: distinguish SDK mini() prompt from Mini main-window mode and Mini ACP/AI sizing.
- Protocol and automation receipts: getState, getElements, simulateKey, waitFor, choice selection, stateResult inputValue, visibleChoiceCount, and layout info.
- Data/privacy boundaries: typed input/filter values, choices/metadata exposure in automation, no persistence unless script handles result.
- Error, empty choices, loading, cancellation, unsupported SDK behavior, stale warnings, no-footer states, and sizing regressions.

Important boundaries:

- Feature 016 covers core `arg()`, `select()`, `div()`, and `md()`; this feature should focus on Mini/Micro distinct contracts.
- Mini main-window mode and Mini AI are separate windowing/ACP features; include only boundary notes needed to avoid confusion.
- `mic()` media stub is not `micro()`; do not conflate them.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: prompt-runtime, sdk-script-execution, protocol-automation, agentic-testing, window-resizing, keyboard-focus-routing
- `lat.md`: scripting, protocol, surfaces, verification, design, windowing, tests/mini-window-contract
- Source: SDK mini/micro, prompt handler mini/micro routes, app view state, render dispatch, mini/micro renderers, arg helpers shared behavior, native footer ownership, window resize, collect_elements, simulateKey, layout info, Tab AI context
- Tests/reports: SDK editor/mini/micro tests, mini window sizing contract, dictation setup MiniPrompt contract, minimal chrome audits, tab AI coverage, autonomous prompt transitions

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 023 Mini and Micro Prompts / mini() / micro()

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
