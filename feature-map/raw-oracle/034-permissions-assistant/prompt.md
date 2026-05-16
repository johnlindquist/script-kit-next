# 034 Permissions and Permission Assistant

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

## Feature Scope

Map Script Kit GPUI permission setup, passive permission detection, and the native Permission Assistant:

- Built-in Accessibility and Screen Recording assistant entry points.
- Permission Assistant native overlay over live System Settings privacy panes.
- Passive status checks for Accessibility, Screen Recording, and Microphone.
- Non-prompting guarantees: no TCC mutation, no prompting APIs, no System Settings automation, no activation-policy changes.
- Native overlay lifetime, retained handle, teardown, AppKit panel behavior, refresh/repositioning, and drag source.
- Host `.app` bundle URL drag payload and why it must not point to the executable inside `Contents/MacOS`.
- System Settings window locator behavior and coordinate conversion.
- `computer/list_permissions` and `computer/get_permission` read-only MCP surfaces.
- Dictation setup microphone preflight boundary.
- Screenshot/screen recording preflight boundary.
- Tests/source audits that pin this behavior.

## Required Output Shape

```markdown
## 034 Permissions and Permission Assistant

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

## Specific Questions To Answer

1. How do users enter the Accessibility and Screen Recording Permission Assistant?
2. What does the assistant open in System Settings, and what native overlay does it show?
3. How does the drag-source row work, and exactly what file URL payload is dragged?
4. What passive APIs are used for Accessibility, Screen Recording, and Microphone?
5. Which APIs or behaviors are explicitly forbidden because they would prompt, mutate TCC, or activate the app?
6. How does the overlay locate and track the System Settings window?
7. What does retaining one active `PermisoHandle` accomplish?
8. What does teardown do, and in what order?
9. How do `computer/list_permissions` and `computer/get_permission` expose permission state to agents?
10. What can agents verify without prompting for permissions or opening System Settings?
11. What must be proven with source audits vs runtime receipts vs screenshots?
12. What are the boundaries with dictation setup, screenshot capture, Accessibility menu actions, and SDK clipboard/selected-text APIs?
