# 027 Keyboard and Mouse APIs

```text
[keyboard-mouse-apis-atlas]




- SDK contracts, argument shapes, payload shapes, return values, warning/stub copy, and current implementation truth for keyboard and mouse helpers.
- Existing tests/smokes/scripts that attempt to use keyboard/mouse APIs, especially places that note keyboard.tap is ignored or expected to fail.


- Feature 025 covers `setInput` as the preferred receipt-backed way to mutate prompt text.
- Feature 026 covers selected-text Cmd+V simulation and Accessibility permission; do not conflate that specific Core Graphics paste path with general `keyboard.tap`.
- Protocol automation `simulateKey` and `batch` are separate agent automation surfaces and should be referenced as the current reliable alternatives.
- MCP computer tools are separate read/observe tools unless source proves mutation.
- Mark uncertain claims as inference and name exact proof gaps.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


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
