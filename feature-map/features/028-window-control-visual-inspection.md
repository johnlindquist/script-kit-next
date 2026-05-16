# 028 Window Control and Visual Inspection APIs

This chapter maps main-window visibility controls, debug grid commands, window bounds, screenshot capture, and layout inspection APIs.

Raw Oracle reference: [answer](../raw-oracle/028-window-control-visual-inspection/answer.md), [prompt](../raw-oracle/028-window-control-visual-inspection/prompt.md), [bundle map](../raw-oracle/028-window-control-visual-inspection/bundle-map.md), [full log](../raw-oracle/028-window-control-visual-inspection/output.log), [session metadata](../raw-oracle/028-window-control-visual-inspection/session.json).

## Executive Summary

Feature 028 covers:

- `show()`.
- `hide()`.
- `blur()`.
- `showGrid(options?)`.
- `hideGrid()`.
- `getWindowBounds()`.
- `captureScreenshot(options?)`.
- `getLayoutInfo()`.

These APIs split into two families. `show`, `hide`, `blur`, `showGrid`, and `hideGrid` are fire-and-forget controls. They send a command and resolve after the SDK message is written, not after the UI visibly changes. Agents must prove postconditions with follow-up state, elements, automation-window inspection, bounds, or screenshots.

`getWindowBounds`, `captureScreenshot`, and `getLayoutInfo` are request-backed queries. They allocate request ids, register pending SDK callbacks, and expect correlated responses. Bounds and layout are structural proof surfaces; screenshots are high-sensitivity pixel evidence and require content audit before they count as visual proof.

The main ambiguity Oracle flagged is show-response drift: `lat.md/protocol.md` and `tests/stdin_show_hide_simulatekey_no_response_envelope_contract.rs` describe stdin show/hide/simulateKey as no-envelope commands, while one bundled `app_run_setup.rs` snippet includes a `window_visibility_ack` send in a Show branch. Treat `show()`/`hide()` as unreceipted at the SDK level and verify the active dispatcher before relying on any visibility ack.

## What Users Can Do

| Capability | Entry | Current result |
|---|---|---|
| Reveal Script Kit. | `await show()` | Sends `show`; no SDK receipt. |
| Hide the main window. | `await hide()` | Sends `hide`; no SDK receipt. |
| Defocus/blur Script Kit. | `await blur()` | Sends `blur`; exact runtime path is a proof gap in the captured context. |
| Show visual debug grid. | `await showGrid({ gridSize: 16 })` | Sends grid options; visual proof requires screenshot. |
| Hide visual debug grid. | `await hideGrid()` | Sends grid hide command; no SDK receipt. |
| Read main bounds. | `await getWindowBounds()` | Resolves `{ x, y, width, height }`; SDK may mask error JSON as zero values. |
| Capture pixels. | `await captureScreenshot({ hiDpi })` | Resolves base64 PNG data, width, and height, or rejects typed screenshot errors. |
| Inspect layout structure. | `await getLayoutInfo()` | Resolves window size, prompt type, and component tree. |

## Core Concepts

### Fire-And-Forget Controls

`show`, `hide`, `blur`, `showGrid`, and `hideGrid` do not call `addPending`. A resolved Promise proves the SDK function sent a line to stdout; it does not prove visibility, focus, grid rendering, or hidden state.

### Receipt-Backed Queries

`getWindowBounds`, `captureScreenshot`, and `getLayoutInfo` call `nextId()` and `addPending(...)`. They are the right surface when an agent needs a direct response.

### Main Window Is Special

The main Script Kit window is tracked as automation window id `main`. Main-window hide paths are supposed to dismiss the main panel only, preserve independent secondary hosts, and reset/rekey so a later `show()` returns to ScriptList.

### Screenshots Are Pixel Evidence

Screenshots can expose private user data and can fail because of Screen Recording permission or stale/ambiguous native windows. A PNG existing on disk is not enough; screenshot proof needs non-empty data, positive dimensions, PNG validity, and pixel-content audit.

### Layout Info Is Structural

`getLayoutInfo()` reports component names, bounds, hierarchy, prompt type, and explanations. It does not prove colors, text rendering, clipping, blur/vibrancy, z-order, or grid overlay pixels.

## Entry Points

| Entry | Payload | Response | Notes |
|---|---|---|---|
| `show()` | `{ type:"show" }` | None at SDK level. | Prove with follow-up state/window inspection. |
| `hide()` | `{ type:"hide" }` | None at SDK level. | Prove `windowVisible:false` or automation registry state. |
| `blur()` | `{ type:"blur" }` | None at SDK level. | Runtime side effect needs focused-window proof. |
| `showGrid(options?)` | `{ type:"showGrid", ...options }` | None at SDK level. | Visual grid proof requires screenshot. |
| `hideGrid()` | `{ type:"hideGrid" }` | None at SDK level. | Visual absence requires screenshot if it matters. |
| `getWindowBounds()` | `{ type:"getWindowBounds", requestId }` | SDK script path expects JSON bounds through `submit`; stdin path has typed `windowBounds`. | Zero bounds can mean error or missing registry data. |
| `captureScreenshot(options?)` | `{ type:"captureScreenshot", requestId, hiDpi }` | `screenshotResult`. | SDK wrapper shown does not expose protocol `target`. |
| `getLayoutInfo()` | `{ type:"getLayoutInfo", requestId }` | `layoutInfoResult`. | Protocol has target, but handler is main-window limited in captured context. |

## User Workflows

### Show The Main Window

A script calls:

```ts
await show()
```

The SDK sends `{ type:"show" }` and resolves. Runtime show paths activate/focus the main panel and sync the main automation-window record. To prove it, query `getState`, `listAutomationWindows`, or `inspectAutomationWindow({ id:"main" })`. Capture a screenshot only when visual proof is required.

### Hide The Main Window

A script calls:

```ts
await hide()
```

The SDK sends `{ type:"hide" }` and resolves. Main hide should be a main-panel-only dismissal, not a broad app hide that conceals Notes or other independent hosts. For reset-sensitive flows, prove `windowVisible:false`, then show again and assert the surface returned to ScriptList.

### Blur Or Defocus

`blur()` sends `{ type:"blur" }`, but the bundled context did not prove the exact Rust/AppKit dispatch path. Treat it as a focus-affecting, unreceipted command. Prove focus with automation-window focused id, frontmost app observation, or the appropriate platform receipt.

### Show Or Hide Debug Grid

`showGrid()` forwards grid options to the debug grid path. Options include grid size, bounds, box model, alignment guides, dimensions, depth, and color scheme. Stdin ExternalCommand snippets construct `color_scheme: None`, so custom grid colors need path-specific proof.

Use screenshots to prove that the grid rendered or disappeared. `getLayoutInfo()` can prove the expected structural bounds, but not overlay pixels.

### Read Window Bounds

`getWindowBounds()` differs by runtime path. The SDK script-execution path reads main NSWindow bounds and returns JSON through a submit-style response. Error JSON like `{"error":"Main window not found"}` can parse into zero-like bounds because the SDK defaults missing numeric fields to `0`.

The stdin automation path reads `list_automation_windows()`, filters id `main`, emits typed `windowBounds`, and logs request-scoped `get_window_bounds_result`.

### Capture Screenshot

`captureScreenshot({ hiDpi })` returns base64 PNG data and dimensions from `screenshotResult`, or rejects typed screenshot errors. The Rust protocol supports screenshot targets, but the SDK wrapper shown only passes `hiDpi`.

Screenshot capture uses Screen Recording preflight and rejects blank/black/solid-like images. Treat screenshot failures as possible infrastructure or permission failures before treating them as UI regressions.

### Get Layout Info

`getLayoutInfo()` returns a structural component tree. Captured mappings include prompt types such as `mainMenu`, `arg`, `div`, `form`, `term`, `editor`, `select`, `path`, `env`, and `drop`. Non-main targets are rejected to default layout in the captured handler path.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Reveal main window. | `show()` | Main hidden/behind. | None. | SDK `ShowMessage`; runtime show/focus/sync path. | Main should be visible/focused. | Follow-up state or automation-window inspection. |
| Hide main window. | `hide()` | Main visible. | None. | SDK `HideMessage`; `PromptMessage::HideWindow`; hide/reset paths. | Main hidden; next show should return to ScriptList. | `windowVisible:false`, registry state, next-show surface. |
| Defocus. | `blur()` | Main focused. | None. | SDK `BlurMessage`. | Focus changes, exact route unproven. | Focus/frontmost observation. |
| Show grid. | `showGrid(options)` | Visible prompt/window. | None. | SDK/protocol grid message -> `show_grid`. | Overlay rendered. | Screenshot with content audit. |
| Hide grid. | `hideGrid()` | Grid active. | None. | SDK/protocol grid hide -> `hide_grid`. | Overlay removed. | Screenshot if visual absence matters. |
| Read bounds. | `getWindowBounds()` | Main registered. | None. | SDK submit JSON or stdin typed `windowBounds`. | Bounds returned. | Positive bounds plus trace/typed response. |
| Capture pixels. | `captureScreenshot({ hiDpi })` | Visible target. | None. | Screenshot protocol/native capture. | Base64 PNG and dimensions. | PNG/content audit, permission ok. |
| Inspect structure. | `getLayoutInfo()` | Main window. | None. | Layout info request -> `build_layout_info`. | Component tree. | `layoutInfoResult` with expected prompt/components. |

## State Machine

### Fire-And-Forget Control

| State | Trigger | Transition |
|---|---|---|
| Script call. | `show`, `hide`, `blur`, `showGrid`, `hideGrid`. | SDK builds message. |
| Send. | `send(message)`. | Promise resolves. |
| UI effect. | Runtime dispatcher handles command. | No SDK receipt proves completion. |
| Proof. | Follow-up query. | Agent asserts postcondition. |

### Query Request

| State | Trigger | Transition |
|---|---|---|
| Script call. | `getWindowBounds`, `captureScreenshot`, `getLayoutInfo`. | SDK allocates request id. |
| Pending callback. | `addPending(id, ...)`. | SDK waits for correlated response. |
| Runtime response. | `windowBounds`, `screenshotResult`, `layoutInfoResult`, or submit JSON. | SDK resolves or rejects. |
| Verification. | Caller inspects data. | Agent checks dimensions, layout, or PNG content. |

## Visual And Focus States

| State | How it appears | Proof path |
|---|---|---|
| Main visible. | Launcher/prompt panel visible. | `getState`, automation window registry, optional screenshot. |
| Main hidden. | Main panel dismissed. | `windowVisible:false`, registry visible flag. |
| Main focused. | Script Kit accepts input. | focused automation-window id or platform frontmost/focus receipt. |
| Main blurred. | Focus returned elsewhere. | frontmost/focus observation. |
| Grid visible. | Overlay lines/bounds/dimensions on prompt. | Screenshot content audit. |
| Screenshot valid. | PNG captures real nonblank pixels. | positive dimensions, PNG bytes, pixel audit. |
| Layout known. | Component tree available. | `layoutInfoResult`. |

## Keystrokes And Commands

This feature is command/API driven, not keyboard-helper driven. Do not use SDK `keyboard.*` or `mouse.*` from feature 027 to prove these behaviors.

| Command | Proof rule |
|---|---|
| `show` | Parse/send is not enough; inspect main visibility. |
| `hide` | Parse/send is not enough; inspect hidden state and reset if relevant. |
| `blur` | Inspect focused window or frontmost app. |
| `showGrid` | Screenshot if visual overlay matters. |
| `hideGrid` | Screenshot if visual absence matters. |

## Actions And Menus

These APIs do not own Actions dialog rows or menu actions. They are supporting surfaces for showing/hiding the main panel, visual debugging, and inspection. If an action workflow uses screenshots or layout info, the action behavior belongs to the action-owning feature and the screenshot/layout proof belongs here.

## Automation And Protocol Surface

| Surface | Status | Notes |
|---|---|---|
| `show` / `hide` | Fire-and-forget at SDK level. | Follow with state/window inspection. |
| `blur` | Fire-and-forget at SDK level. | Exact runtime route is a proof gap in this bundle. |
| `showGrid` / `hideGrid` | Fire-and-forget. | Visual proof requires screenshot. |
| `getWindowBounds` | Request-backed. | SDK submit JSON vs stdin typed `windowBounds` drift. |
| `captureScreenshot` | Request-backed typed result. | Protocol target support exists; SDK wrapper shown only exposes `hiDpi`. |
| `getLayoutInfo` | Request-backed typed result. | Protocol target exists; captured handler is main-only. |
| `getElements` / `inspectAutomationWindow` | Adjacent proof surfaces. | Prefer for secondary windows and semantic inspection. |

## Data, Storage, And Privacy Boundaries

Screenshots can contain private user content, filenames, clipboard/history text, browser/app surfaces, and debug overlays. Store screenshots only when the proof requires pixels, and reject blank/black captures as infrastructure failures.

Layout info reveals structural state: prompt type, component names, bounds, hierarchy, and explanations. It is less sensitive than pixels but still exposes active UI shape.

Bounds reveal screen geometry and window placement. Hide paths may persist per-display main-window position. Show/hide/blur change visible focus state and can interrupt the user.

## Error, Empty, Loading, And Disabled States

| API | Failure or ambiguous state |
|---|---|
| `show()` | No SDK receipt; active dispatcher ambiguity around `window_visibility_ack`. |
| `hide()` | No SDK receipt; must not hide independent secondary windows; reset must be proven where relevant. |
| `blur()` | No SDK receipt; runtime route not proven by captured bundle. |
| `showGrid()` | Overlay may not be visible if window is hidden; stdin color scheme handling may drop custom colors. |
| `hideGrid()` | Likely harmless if grid inactive, but explicit idempotence was not proven. |
| `getWindowBounds()` | SDK can collapse error JSON or parse failure into zero bounds. |
| `captureScreenshot()` | Permission denied, blank/black capture, stale target, target resolution failure, xcap failure, or empty data. |
| `getLayoutInfo()` | Non-main target returns default layout; empty components can mean mock, target rejection, or real default state. |

## Code Ownership

| Area | Owner skill | Files and references |
|---|---|---|
| SDK functions and types. | `sdk-script-execution` | `scripts/kit-sdk.ts`. |
| Main window and screenshots. | `platform-windowing-macos` | `src/platform/screenshots_window_open.rs`, main-window platform paths. |
| Resize and bounds proof. | `window-resizing` | `src/window_resize/`, `src/app_impl/ui_window.rs`, bounds tests. |
| Protocol messages and receipts. | `protocol-automation` | `src/protocol/message/variants/query_ops.rs`, `src/protocol/message/variants/system_control.rs`, stdin/runtime dispatch. |
| Runtime UI dispatch. | `gpui-ui-foundation`, adjacent. | `src/prompt_handler/mod.rs`, `src/execute_script/mod.rs`. |
| Layout tree. | `protocol-automation`, UI layout adjacent. | `src/app_layout/build_layout_info.rs`, `src/app_layout/build_component_bounds.rs`. |
| Debug grid. | `agentic-testing`, UI adjacent. | `src/debug_grid/mod.rs`, grid smoke tests. |
| Visual proof. | `agentic-testing` | `scripts/agentic/verify-shot.ts`, `tests/verify_shot_strict_window_contract.rs`. |

## Invariants And Regression Risks

- Do not add or rely on `showResult`, `hideResult`, `showGridResult`, `hideGridResult`, or `blurResult` without updating contracts and callers.
- Hide must remain a main-panel-only dismissal and must not app-hide independent secondary windows.
- Hide/reset/rekey order matters; stale subviews after next `show()` are a regression.
- `getWindowBounds` stdin handling must read the automation registry and return main-window bounds.
- Screenshot proof must reject blank/black or permission-failed captures.
- `hiDpi` must affect screenshot dimensions.
- `getLayoutInfo` must not pretend to support secondary layouts until implemented and tested.
- Grid overlay changes need visual proof, not only state or layout proof.
- SDK `getWindowBounds()` zeroes are suspicious unless corroborated.

## Verification Recipes

### Prove Show

Send `show`, use a parse receipt only as command-acceptance proof, then query `getState`, `listAutomationWindows`, or `inspectAutomationWindow({ id:"main" })`. Assert visible/focused main state.

### Prove Hide And Reset

Put the main window in a non-ScriptList surface, send `hide`, assert hidden state, send `show`, then assert the main surface is ScriptList.

### Prove Bounds

Use `getWindowBounds()` and reject zero dimensions unless the test explicitly expects missing bounds. For stdin automation, prefer the typed `windowBounds` response and `get_window_bounds_result` trace.

### Prove Screenshot

Render visible content, prove state first, call `captureScreenshot`, then verify positive dimensions, base64 PNG bytes, no error, and nonblank pixel audit. Check Screen Recording permission before screenshot-heavy proof.

### Prove HiDPI

Capture once with `hiDpi:false` and once with `hiDpi:true`. Assert HiDPI dimensions are greater than or equal to the 1x dimensions.

### Prove Debug Grid

Call `showGrid({ gridSize: 16 })`, capture a screenshot, audit pixels, and inspect for grid overlay. Call `hideGrid()` and capture again if absence matters.

### Prove Layout Info

Render a known prompt, call `getLayoutInfo()`, assert positive `windowWidth`/`windowHeight`, expected `promptType`, and expected component names/bounds. Use screenshots only for visual claims.

## Agent Notes

- Do not wait for a `showResult`, `hideResult`, `showGridResult`, `hideGridResult`, or `blurResult`.
- Treat the `window_visibility_ack` evidence as a dispatcher-specific proof gap until verified in the active build.
- Prefer state, elements, automation-window inspection, and bounds before screenshots.
- Use screenshots only for pixel claims: grid overlay, clipping, rendered color, blur/vibrancy, or screenshot behavior.
- Always audit screenshots; a blank or black PNG is not visual proof.
- Treat zero bounds from SDK `getWindowBounds()` as likely infrastructure/error unless corroborated.
- Do not use targeted `getState` for secondary surfaces. Use `getElements(target)` and `inspectAutomationWindow(target)`.
- Remember that the SDK `captureScreenshot()` wrapper shown only exposes `hiDpi`, not protocol `target`.
- Hide the debug grid before ordinary visual proof.
- This belongs to `platform-windowing-macos`, `window-resizing`, `protocol-automation`, `agentic-testing`, and `sdk-script-execution`.

## Related Features

- [004 MCP Context Resources / SDK / Protocol Automation](./004-mcp-sdk-protocol.md) owns the broader protocol and proof model.
- [014 Quick Terminal PTY / TermPrompt / Warm Pool / Apply-back](./014-quick-terminal-pty.md) and [015 SDK TermPrompt / term() / Terminal Actions / Full-height Terminal](./015-sdk-term-prompt.md) own terminal-specific layout behavior.
- [025 System Feedback and Prompt Control APIs](./025-system-feedback-and-prompt-control.md) owns HUD and prompt-control interactions with hide restore intent.
- [026 Clipboard, Selected Text, and Accessibility APIs](./026-clipboard-selected-text-accessibility.md) owns selected-text focus handoff and Accessibility.
- [027 Keyboard and Mouse APIs](./027-keyboard-mouse-apis.md) documents why unsupported keyboard/mouse helpers should not be proof steps here.

## Open Questions And Gaps

- `blur()` runtime route and exact AppKit/GPUI effect were not proven by the captured context.
- Show response behavior has conflicting evidence between no-envelope contracts and one `window_visibility_ack` snippet.
- SDK `captureScreenshot()` does not expose protocol `target` in the shown wrapper.
- `getLayoutInfo(target)` exists at protocol level but returns default layout for non-main targets in the captured handler.
- Debug grid renderer internals were not fully present in the extracted context.
- SDK `getWindowBounds()` masks some errors as zeros.
- Stdin `showGrid` color scheme handling appears to drop custom colors.
