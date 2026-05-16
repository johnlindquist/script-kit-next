# 028 Window Control And Visual Inspection APIs

```text
[window-control-visual-inspection-atlas]




- SDK contracts, argument shapes, payload shapes, return values, auto-submit behavior, response handling, and error behavior.
- Existing tests/smokes/source-audits that use or pin these APIs, especially `getWindowBounds`, `captureScreenshot`, `getLayoutInfo`, `showGrid`, `hideGrid`, show/hide no-envelope behavior, and visual proof scripts.
- Error, empty, loading, unsupported, target-rejected, no-response, stale-window, and permission states.


- Feature 004 covers the broad protocol automation model; this feature should zoom into the window/visual APIs.
- Feature 014/015 cover terminal windows and terminal prompt behavior; only mention them when window bounds or screenshots prove layout.
- Feature 025 covers HUD visibility decoupling; mention only where `show()`/`hide()`/HUD restore intent intersect.
- Feature 026 covers selected-text focus handoff and Accessibility; do not conflate those APIs with generic `blur()`.
- Feature 027 covers unsupported keyboard/mouse helpers; do not use those helpers as proof for visual/window behavior.
- Mark uncertain claims as inference and name exact proof gaps.




Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


## 028 Window Control and Visual Inspection APIs

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
