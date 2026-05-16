# 036 Tray Menu and Global App Entry Points Oracle Prompt

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code, create downloadable artifacts, or ask the user to open files. Return text only.

## Feature Scope

Feature id: `036-tray-menu-global-entry-points`

Feature name: Tray Menu and Global App Entry Points

Cover these Script Kit GPUI capabilities:

- macOS tray/status-bar menu as secondary entry point.
- `TrayManager`, tray menu construction, menu sections, stable action ids, icon/template rendering, version/update label mutation, and current-app command row refresh.
- Tray actions: Open Script Kit, Current App Commands, Open Notes, Open Agent Chat, Send Feedback, Follow Us, GitHub, Discord, Settings, Reload Scripts, Check for Updates, Version row, About Script Kit, Quit Script Kit.
- Global hotkey display and routing: tray `Open Script Kit` key equivalent mirrors the configured launcher hotkey, global launcher hotkeys, notes hotkey, AI hotkey, dictation tray/hotkey boundaries where relevant.
- Update checker behavior and why worker threads cannot mutate the native menu directly.
- About surface opened from tray and automation-only `openAbout`.
- MCP read-only tray menu tools: `computer/list_tray_menu`, `computer/get_tray_menu_item`, `computer/get_tray_menu_item_by_id`, including closed schemas, status values, stable ids, section/item indexing, and no-action boundaries.
- Distinguish Script Kit's own tray menu model from frontmost app menu-bar observation (`computer/list_menus`, current-app commands) and from global menu extras/status-item discovery, which is explicitly not implemented.
- Verification/test surfaces: tray URL constants, action id roundtrip, icon rendering/template tests, source audits for MCP observation-only tools, protocol docs, runtime tray hotkey tests, and launcher startup entrypoint tests.

Explicitly cover special states:

- dynamic Version row before/after update state changes.
- `Current App Commands` label when a last real app is tracked versus when no app is tracked.
- configured built-ins or menu rows missing/unavailable.
- no-op/non-action Version row behavior.
- failed SVG/icon rendering fallback.
- update check pending/available/no-update/error states where source pins them.
- MCP section/item not found and id not found statuses.
- bad MCP arguments rejected by closed schemas.

## Required Output Shape

```markdown
## 036 Tray Menu and Global App Entry Points

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
