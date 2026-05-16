# 027 Keyboard and Mouse APIs

```text
[keyboard-mouse-apis-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 027: Keyboard and Mouse APIs / `keyboard.type()` / `keyboard.tap()` / `mouse.move()` / `mouse.leftClick()` / `mouse.rightClick()` / `mouse.setPosition()`.

This atlas must cover:

- SDK contracts, argument shapes, payload shapes, return values, warning/stub copy, and current implementation truth for keyboard and mouse helpers.
- Protocol support: `Message::Keyboard`, `Message::Mouse`, `KeyboardAction`, `MouseAction`, `MouseData`, constructors, serialization names, and typed payload gaps.
- Current backend truth: distinguish SDK/protocol-visible typed messages from actual app-side/native input behavior. The captured source appears to warn that these APIs are not implemented and tests/smokes often treat them as unavailable/ignored.
- Relationship to other input paths: protocol `simulateKey`, `batch.setInput`, native macOS input in selected-text paste, MCP computer tools, and runtime UI key handling. Clarify that these are separate from SDK `keyboard.*`/`mouse.*`.
- Existing tests/smokes/scripts that attempt to use keyboard/mouse APIs, especially places that note keyboard.tap is ignored or expected to fail.
- Data/security boundaries: synthetic input is high risk; native event generation would require permission/focus guarantees and should not be inferred from message serialization.
- Error, unsupported, no-op, race, and verification states: SDK Promise resolution without backend action, warning-only behavior, lack of receipts, no response payload, focus ambiguity, and false-positive tests.

Important boundaries:

- Feature 025 covers `setInput` as the preferred receipt-backed way to mutate prompt text.
- Feature 026 covers selected-text Cmd+V simulation and Accessibility permission; do not conflate that specific Core Graphics paste path with general `keyboard.tap`.
- Protocol automation `simulateKey` and `batch` are separate agent automation surfaces and should be referenced as the current reliable alternatives.
- MCP computer tools are separate read/observe tools unless source proves mutation.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: platform-windowing-macos, keyboard-focus-routing, protocol-automation, agentic-testing, sdk-script-execution
- `lat.md`: protocol, design, verification
- Source: SDK keyboard/mouse objects and message interfaces, protocol system-control variants/constructors/primitives, smoke tests and SDK tests using keyboard/mouse, MCP resources references, generated API tests.

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 027 Keyboard and Mouse APIs

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
