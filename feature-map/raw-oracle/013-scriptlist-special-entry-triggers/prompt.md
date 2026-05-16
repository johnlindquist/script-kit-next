# 013 ScriptList Special Entry Triggers Prompt

```text
[scriptlist-special-triggers-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 013: ScriptList Special Entry Triggers / first-character route handoffs.

This atlas must cover every committed first-character ScriptList route that is distinct from ordinary root search and distinct from source filters:

- `~` and `~/...` mini File Search handoff.
- `/` ACP slash picker handoff.
- `@` ACP mention/context picker handoff.
- `>` Quick Terminal handoff.
- `?` actions/help handoff.

Map what happens when each trigger is typed in the main launcher, which routes open, what view/state/focus/window size changes occur, which exact variants and helper functions own the behavior, which adjacent features take over after handoff, what should remain ordinary literal text, and which power-user/menu-syntax prefixes must NOT route through this special-entry classifier.

Important known requirements from current docs and source:

- Special entries are narrow first-character handoffs owned by `ScriptListSpecialEntry`, not general query parsing.
- Bare `~` normalizes to `~/` and opens mini File Search; `~/src` also routes to mini File Search, while `/tmp` must not route through this classifier.
- `/` opens Agent Chat with the slash picker, while non-bare slash/path strings are not this route.
- `@` opens Agent Chat with the mention/context picker, while `@browser` must stay ordinary/literal for later handling.
- `>` opens Quick Terminal only when the input is exactly `>`. Longer strings such as `>deploy -- prod` are power syntax/literal search and must not route through this classifier.
- `?` opens actions/help only when `has_actions()` allows actions.
- Transient trigger text (`~`, `/`, `@`, `>`, `?`) should not persist when returning to ScriptList.
- Power-user syntax prefixes such as `:`, `+`, `!`, `#`, capture syntax, and source filters belong to other feature chapters and must not be misclassified as special entries.
- Handoff routes must clear/avoid stale menu-syntax decorations where needed and preserve focus/automation surfaces for the destination view.
- Quick Terminal is not ACP Chat; it is the PTY-backed terminal wrapper with its own Tab, Escape, Cmd+W, apply-back, native footer, and cleanup behavior.
- ACP slash and mention handoffs should be mapped through the ACP entry/composer/context-picker contracts and popup automation receipts.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: main-menu-search-selection, acp-context-composer, quick-terminal-pty, file-search-portals, actions-popups, protocol-automation, agentic-testing
- `lat.md`: surfaces, acp-chat, ai-context, automation, builtins, menu-syntax, verification
- Source: ScriptList special-entry classifier and input-change routing, ScriptList render/key handling, AppView/surface contracts, ACP entry helpers, Quick Terminal openers, actions toggling, ACP context picker, main-window preflight receipts
- Tests/scripts: tilde/special-entry source contract, ACP main menu/popup registry contracts, tab AI routing contracts, Quick Terminal contracts, action popup contracts, and ACP runtime proof script

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

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
