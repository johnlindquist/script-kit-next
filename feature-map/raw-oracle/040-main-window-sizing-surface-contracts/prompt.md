# 038 Main Window Sizing and Surface Contracts Oracle Prompt

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code, create downloadable artifacts, or ask the user to open files. Return text only.

## Feature Scope

Feature id: `038-main-window-sizing-surface-contracts`

Feature name: Main Window Sizing and Surface Contracts

Cover these Script Kit GPUI capabilities:

- Main-window presentation modes: Full launcher, Mini main window, MiniPrompt, Mini AI chat, full prompt surfaces, and split preview/detail surfaces.
- `MainWindowMode`, `ViewType`, `resize_to_view_sync`, `defer_resize_to_view`, `update_window_size_deferred`, `calculate_window_size_params`, and content-aware mini sizing receipts.
- Entry paths that open or resize surfaces: main menu, triggerBuiltin, tray/current-app commands, special-character handoffs, prompt creation, hide/reset/go-back, Escape/Cmd-W, filter changes, and protocol commands.
- Built-in filterable sizing policy: Mini for single-column command surfaces, Full for list-plus-preview/detail panes, and special exceptions like File Search Mini vs Full.
- Surface identity contracts: `AppView`, `SurfaceKind`, `surface_contract`, native footer surface ids, generated `surface-contracts.json`, generated current-view transition inventory, `getState.surfaceContract`, and `activePopupContract`.
- Automation semantic surface re-keying after route transitions and `triggerBuiltin` dispatch.
- Dismiss/focus/keyboard/actions/proof/visual policy vocabulary carried by `LauncherSurfaceContract`.
- Native footer and Mini layout height relationships, including footer spacer, hint strip, and fixed Mini height.
- Verification: source-audit tests, generated-contract checks, surface navigator/filterable matrix receipts, state-first bounds receipts, and when screenshots are or are not appropriate.

Explicitly distinguish:

- Main-window sizing from secondary window sizing.
- Mini main window from MiniPrompt, MicroPrompt, inline Mini AI, and Quick Terminal.
- Surface identity from render implementation files.
- State-first surface-contract receipts from visual screenshots.
- Initial open sizing from follow-up deferred resize paths.
- Product-facing window behavior from generated agent-readable contract artifacts.

## Required Output Shape

```markdown
## 038 Main Window Sizing and Surface Contracts

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
```

## Context Notes

This repository requires `lat check` after atlas updates. The maintained atlas preserves the complete Oracle answer under `feature-map/raw-oracle/<feature-id>/answer.md` and distills a readable chapter under `feature-map/features/<feature-id>.md`.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
