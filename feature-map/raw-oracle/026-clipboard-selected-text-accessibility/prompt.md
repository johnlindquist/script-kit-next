# 026 Clipboard, Selected Text, and Accessibility APIs

```text
[clipboard-selected-text-accessibility-atlas]




- SDK contracts, payload shapes, request ids, return values, auto-submit fallbacks, aliases, and error handling for clipboard and selected-text/accessibility APIs.
- `copy()` / `paste()` aliases and their relationship to `clipboard.writeText` / `clipboard.readText`.


- Feature 008 covers root unified clipboard history; do not remap clipboard history storage/list UI here except as a related feature.
- Feature 025 covers `hud`, `setActions`, and `setInput`; include only the shared fire-and-forget/receipt lessons if needed.
- Keyboard/mouse/window APIs are separate later features; include only the Cmd+V Core Graphics boundary used by `setSelectedText`.
- Permission Assistant is an adjacent setup workflow; include its permission context but not its full UI.
- Mark uncertain claims as inference and name exact proof gaps.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


## 026 Clipboard, Selected Text, and Accessibility APIs

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
