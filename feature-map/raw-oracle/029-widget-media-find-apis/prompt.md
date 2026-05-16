# 029 Widget Media And Find APIs

```text
[widget-media-find-apis-atlas]

Project: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun SDK and agentic automation scripts. This repo uses `lat.md/` as a source-of-truth architecture graph. Repo rules require `lat search`/`lat expand` before work, `lat.md/` updates when behavior changes, and `lat check` after tasks. The local agent will write files; Oracle should return text only.

Goal: Produce a complete operator-grade feature atlas for feature 029: Widget, Media, and Find APIs / `widget()` / `webcam()` / `mic()` / `eyeDropper()` / `find()`.

This atlas must cover:

- SDK contracts, argument shapes, payload shapes, return values, thrown errors, warning/stub copy, event handling, controller methods, and current implementation truth.
- `widget()` behavior: warning-only SDK status, message send, returned controller, `setState`, `close`, event handler registration, widgetEvent dispatch, Rust `WidgetComingSoon` route, and false-positive tests that prove controller shape but not visible widget behavior.
- `webcam()`, `mic()`, and `eyeDropper()` behavior: SDK throws before sending for the current globals, historical/protocol Rust variants or coming-soon routes if present, and why media streaming / color picking are not currently reliable GPUI features.
- `find()` behavior: SDK prompt shape with `placeholder` and `onlyin`, pending submit handling, auto-submit fallback, cancellation behavior, and current backend/protocol proof gaps if Rust does not expose a corresponding message route.
- Relationship to adjacent features: file search and path prompt, debug screenshots / screen recording, dictation/microphone preflight, unsupported SDK reference, and Script Kit media/dictation surfaces.
- Existing tests/smokes/source-audits that use or pin these APIs, especially `tests/sdk/test-widget.ts`, generated API tests, minimal chrome audits, SDK reference unsupported metadata, and prompt coming-soon routes.
- Data/security boundaries: widget HTML, event payloads, microphone/camera/screen capture, screenshot/color-pick permissions, file path search scope, and unsupported API warnings.
- Error, unsupported, no-op, false-positive, throw-before-send, coming-soon-toast, and missing-backend-route states.

Important boundaries:

- Feature 002/019 cover file search and `path()`; this feature should explain how `find()` differs or fails to connect.
- Feature 026 covers Accessibility and clipboard-selected text; do not conflate it with `eyeDropper`.
- Feature 028 covers screenshot capture and Screen Recording; mention only where eye dropper/color picking would need screen pixels.
- Feature 009 covers dictation history and media-adjacent transcription; do not treat `mic()` as implemented dictation.
- Mark uncertain claims as inference and name exact proof gaps.

Bundle map:

- Process context: `AGENTS.md`, `CLAUDE.md`
- Skills: sdk-script-execution, prompt-runtime, file-search-portals, dictation-media, protocol-automation, agentic-testing
- `lat.md`: scripting, surfaces, protocol, design, verification, tests/dictation-setup-nux
- Source: SDK widget/media/find functions and message interfaces, protocol prompt/media variants and constructors, execute_script/prompt_handler coming-soon routes, SDK reference support metadata, minimal chrome audits, SDK tests, generated API tests.

Deliverable: Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Required output shape:

## 029 Widget, Media, and Find APIs

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
