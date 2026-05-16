# 036 Tray Menu, App Lifecycle, Distribution, and Updates Oracle Prompt

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code, create downloadable artifacts, or ask the user to open files. Return text only.

## Feature Scope

Feature id: `036-tray-lifecycle-distribution-updates`

Feature name: Tray Menu, App Lifecycle, Distribution, and Updates

Cover these Script Kit GPUI capabilities:

- macOS tray/status-bar menu sections, row labels, icons, stable action ids, accelerator display, current-app command row, social/help links, Settings/About routes, Reload Scripts, Check for Updates, Version row, and Quit.
- Tray action dispatch through startup/runtime event loops, including main-thread update-state refresh constraints.
- About route as the launcher-native companion to the tray menu: route contract, update card states, links, acknowledgements disclosure, keyboard handling, explicit dismissal, and prior-route restoration.
- Update checker: `UpdateState`, latest-release request, version comparison, release asset selection, manifest SHA handling, tray/about shared state, retry/error behavior, and user-visible update/open-release paths.
- App lifecycle: startup tray/hotkey registration, reload scripts, quit/shutdown paths, launch-at-login helper status, and removed/deferred launch-at-login UI.
- Distribution: local bundle path, CI artifact path, tagged release path, signing/notarization/stapling, release manifest, update asset boundaries, and human-only shipping gates.
- Automation/MCP proof surfaces for tray/about/update/distribution: tray menu model observations, tray item lookup by index/id, About open route, update source audits, release verification scripts, and packaging verification.

Explicitly distinguish:

- Tray model observation versus native status-item clicking.
- Tray menu action execution versus MCP read-only tray menu tools.
- Update state shared by tray and About versus actual installer/download behavior.
- Release manifest generation versus future installer integrity enforcement.
- Launch-at-login helper implementation versus removed/deferred tray UI.
- Source-audit proof versus runtime/native AppKit proof.

## Required Output Shape

```markdown
## 036 Tray Menu, App Lifecycle, Distribution, and Updates

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
