# 013 ScriptList Special Entry Triggers Prompt

```text
[scriptlist-special-triggers-atlas]




- `~` and `~/...` mini File Search handoff.
- `/` ACP slash picker handoff.
- `@` ACP mention/context picker handoff.
- `>` Quick Terminal handoff.
- `?` actions/help handoff.

Map what happens when each trigger is typed in the main launcher, which routes open, what view/state/focus/window size changes occur, which exact variants and helper functions own the behavior, which adjacent features take over after handoff, what should remain ordinary literal text, and which power-user/menu-syntax prefixes must NOT route through this special-entry classifier.


- Special entries are narrow first-character handoffs owned by `ScriptListSpecialEntry`, not general query parsing.
- Bare `~` normalizes to `~/` and opens mini File Search; `~/src` also routes to mini File Search, while `/tmp` must not route through this classifier.
- `/` opens Agent Chat with the slash picker, while non-bare slash/path strings are not this route.
- `@` opens Agent Chat with the mention/context picker, while `@browser` must stay ordinary/literal for later handling.
- `>` opens Quick Terminal only when the input is exactly `>`. Longer strings such as `>deploy -- prod` are power syntax/literal search and must not route through this classifier.
- `?` opens actions/help only when `has_actions()` allows actions.
- Transient trigger text (`~`, `/`, `@`, `>`, `?`) should not persist when returning to ScriptList.
- Handoff routes must clear/avoid stale menu-syntax decorations where needed and preserve focus/automation surfaces for the destination view.
- Quick Terminal is not ACP Chat; it is the PTY-backed terminal wrapper with its own Tab, Escape, Cmd+W, apply-back, native footer, and cleanup behavior.
- ACP slash and mention handoffs should be mapped through the ACP entry/composer/context-picker contracts and popup automation receipts.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


## 013 ScriptList Special Entry Triggers / First-character Route Handoffs

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
