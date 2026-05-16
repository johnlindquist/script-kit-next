# 025 System Feedback and Prompt Control APIs

```text
[system-feedback-prompt-control-atlas]




- SDK API contracts, argument shapes, return values, fire-and-forget semantics, warnings/stub comments, and message payloads for each function.


- Feature 016 covers core `arg()`, `select()`, `div()`, and `md()` prompt lifecycle; this feature should focus on utility/control calls that mutate prompt/UI feedback after or outside initial prompt creation.
- Feature 011 covers root result actions and actions dialog behavior broadly; include `setActions()` only as the SDK/script-facing way to populate prompt actions.
- Tray/menu bar observation/action APIs are separate later features; include `menu()` only as the SDK fire-and-forget message for tray icon/scripts if source proves it.
- Clipboard, selected text, keyboard/mouse, windowing, media, chat, and AI APIs are separate later features. Mention only boundary notes.
- Mark uncertain claims as inference and name exact proof gaps.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


## 025 System Feedback and Prompt Control APIs

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
