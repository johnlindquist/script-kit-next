# 028 Window Control And Visual Inspection APIs

```text
[window-control-visual-inspection-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 028: Window Control and Visual Inspection APIs / `show()` / `hide()` / `blur()` / `showGrid()` / `hideGrid()` / `getWindowBounds()` / `captureScreenshot()` / `getLayoutInfo()`.

This atlas must cover:

- SDK contracts, argument shapes, payload shapes, return values, auto-submit behavior, response handling, and error behavior.
- Fire-and-forget control surfaces: `show`, `hide`, `blur`, `showGrid`, and `hideGrid`. Explain which paths emit no response envelope and how agents must prove postconditions.
- Receipt-backed query surfaces: `getWindowBounds`, `captureScreenshot`, and `getLayoutInfo`. Explain request ids, response types, parsing, target behavior, and failure cases.
- Main-window visibility/focus behavior: show vs hide vs blur, reset semantics, script-requested hide, focus handoff, and independent secondary-window boundaries.
- Debug grid overlay behavior: options, defaults, protocol parsing, renderer ownership, visual testing purpose, and hide behavior.
- Screenshot behavior: SDK result shape, app/runtime capture path, hiDPI option, target support, blank/black capture risks, Screen Recording/privacy boundaries, and why pixel-content audit matters.
- Layout info behavior: current main-window limitation, component tree shape, layout bounds, component explanations, and relationship to screenshots and `getElements`.
- Existing tests/smokes/source-audits that use or pin these APIs, especially `getWindowBounds`, `captureScreenshot`, `getLayoutInfo`, `showGrid`, `hideGrid`, show/hide no-envelope behavior, and visual proof scripts.
- Data/security boundaries: screenshots can expose private data; layout info is structural; bounds reveal geometry; show/hide/blur affect focus and visibility.
- Error, empty, loading, unsupported, target-rejected, no-response, stale-window, and permission states.

Important boundaries:

- Feature 004 covers the broad protocol automation model; this feature should zoom into the window/visual APIs.
- Feature 014/015 cover terminal windows and terminal prompt behavior; only mention them when window bounds or screenshots prove layout.
- Feature 025 covers HUD visibility decoupling; mention only where `show()`/`hide()`/HUD restore intent intersect.
- Feature 026 covers selected-text focus handoff and Accessibility; do not conflate those APIs with generic `blur()`.
- Feature 027 covers unsupported keyboard/mouse helpers; do not use those helpers as proof for visual/window behavior.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: platform-windowing-macos, window-resizing, protocol-automation, agentic-testing, sdk-script-execution
- `lat.md`: windowing, automation, protocol, verification
- Source: SDK window/visual functions and message interfaces, protocol query/system-control variants and constructors, grid/layout types, executor/prompt handler routes, debug grid, layout builder, screenshot capture path, stdin show/hide/grid paths, tests and smoke scripts.

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

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
