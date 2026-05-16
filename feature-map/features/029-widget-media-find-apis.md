# 029 Widget, Media, and Find APIs

This chapter maps widget controller stubs, unsupported media helpers, eye dropper status, and the SDK-visible `find()` surface.

Raw Oracle reference: [answer](../raw-oracle/029-widget-media-find-apis/answer.md), [prompt](../raw-oracle/029-widget-media-find-apis/prompt.md), [bundle map](../raw-oracle/029-widget-media-find-apis/bundle-map.md), [full log](../raw-oracle/029-widget-media-find-apis/output.log), [session metadata](../raw-oracle/029-widget-media-find-apis/session.json).

## Executive Summary

Feature 029 covers:

- `widget()`.
- `webcam()`.
- `mic()`.
- `eyeDropper()`.
- `find()`.

This cluster is mostly a support-status and negative-contract map. The SDK exposes these names, but current GPUI behavior is not equivalent to legacy Script Kit behavior.

`widget()` returns a controller object and records local event handlers, but the runtime route is a coming-soon toast. No visible floating widget surface, renderer, widget-event producer, or backend `widgetAction` handler was proven in the bundled context.

`webcam()`, `mic()`, and `eyeDropper()` throw immediately in the SDK before sending any protocol message. Rust still contains coming-soon message variants for webcam and mic, but those paths are not reached through the current SDK globals. `eyeDropper()` has TypeScript protocol shapes, but no proven Rust route.

`find()` now rejects before send with `UnsupportedSdkFeatureError` / `ERR_UNSUPPORTED_SDK_FEATURE`. It does not create a pending submit resolver, emit a `find` protocol message, or interpret `onlyin`; use `fileSearch(query, { onlyin })` for non-interactive Spotlight/mdfind results.

## What Users Can Do

| Capability | Entry | Current result |
|---|---|---|
| Request a floating widget. | `await widget(html, options)` | SDK returns a controller; GPUI shows coming-soon behavior. |
| Register widget callbacks. | `controller.onClick(...)`, `onInput(...)`, `onClose(...)`, `onMoved(...)`, `onResized(...)` | Local callback registration exists; no event producer was proven. |
| Send widget state. | `controller.setState(state)` | Sends `widgetAction` with `action:"setState"`; no backend handler was proven. |
| Close widget controller. | `controller.close()` | Sends `widgetAction` with `action:"close"` and deletes SDK handler record. |
| Use webcam capture. | `await webcam()` | Throws immediately before sending. |
| Use microphone capture. | `await mic()` | Throws immediately before sending. |
| Pick a screen color. | `await eyeDropper()` | Throws immediately before sending. |
| Ask for legacy interactive find. | `await find(options)` | Rejects before send with `ERR_UNSUPPORTED_SDK_FEATURE`; use `fileSearch(query, { onlyin })` for supported file results. |

## Core Concepts

### SDK Surface Is Not Runtime Support

All five names are script-visible, but visibility is not support. The feature map must preserve whether a function sends a protocol message, throws before send, reaches a coming-soon toast, or has a proven rendered surface.

### Widget Controller Shape

`widget()` builds an id, logs that widgets are not yet implemented in GPUI, sends a `widget` message, stores handlers in SDK-local maps, and returns a controller with methods for state, close, and event registration.

### Media Helpers Throw Before Send

`webcam()` and `mic()` throw with explicit unsupported messages before any protocol line is emitted. Their current SDK text says media streaming is not feasible with the JSONL message-passing architecture.

### Eye Dropper Is Type-Shaped, Not Implemented

`eyeDropper()` throws immediately. TypeScript message and result shapes exist, but the captured Rust code did not prove a route, prompt handler, color picker UI, screenshot flow, or permission handling.

### Find Is An Explicit Unsupported Boundary

`find()` rejects immediately with `UnsupportedSdkFeatureError` before `nextId`, `addPending`, or `send`. It has no GPUI prompt route, renderer, submit contract, or `onlyin` prompt semantics.

## Entry Points

| Entry | Payload or behavior | Response | Notes |
|---|---|---|---|
| `widget(html, options?)` | Sends `{ type:"widget", id, html, options }`. | Controller object. | Runtime maps `Message::Widget` to coming-soon toast. |
| `controller.setState(state)` | Sends `widgetAction` with `action:"setState"`. | Fire-and-forget Promise. | Backend handler not proven. |
| `controller.close()` | Sends `widgetAction` with `action:"close"` and deletes local handler entry. | Fire-and-forget Promise. | No closed flag; later `setState` can still send. |
| `controller.onClick(handler)` | Stores local handler. | Controller. | Requires incoming `widgetEvent`; producer not proven. |
| `controller.onInput(handler)` | Stores local handler. | Controller. | Requires incoming `widgetEvent`; producer not proven. |
| `controller.onClose(handler)` | Stores local handler. | Controller. | Requires incoming `widgetEvent`; producer not proven. |
| `controller.onMoved(handler)` | Stores local handler. | Controller. | Requires incoming `widgetEvent`; producer not proven. |
| `controller.onResized(handler)` | Stores local handler. | Controller. | Requires incoming `widgetEvent`; producer not proven. |
| `webcam(options?)` | Throws before send. | Rejected Promise/throw. | Rust coming-soon variant exists but SDK does not reach it. |
| `mic(options?)` | Throws before send. | Rejected Promise/throw. | Not dictation and not `micro()`. |
| `eyeDropper(options?)` | Throws before send. | Rejected Promise/throw. | No proven Rust route. |
| `find(options?)` | Rejects before send. | `UnsupportedSdkFeatureError` / `ERR_UNSUPPORTED_SDK_FEATURE`. | `onlyin` is not interpreted; use `fileSearch(query, { onlyin })`. |

## User Workflows

### Attempt A Widget

A script can call:

```ts
const w = await widget("<button>Run</button>")
w.onClick(() => {})
await w.setState({ count: 1 })
await w.close()
```

The SDK call returns a controller and callback registration works locally. The GPUI runtime path maps widget messages to `WidgetComingSoon`, logs a warning, and shows a coming-soon toast. Do not document widgets as visible, movable, resizable UI until a renderer, event producer, action handler, and automation proof exist.

### Attempt Webcam Or Microphone Capture

Scripts that call `webcam()` or `mic()` currently fail at the SDK layer before runtime dispatch. This is privacy-safe by default because no hardware capture starts and no permission prompt should appear from these SDK calls.

Do not confuse `mic()` with dictation. Dictation owns its own media setup, preflight, transcription, and history flows. `mic()` is a legacy SDK media helper that is currently unsupported.

### Attempt Eye Dropper

`eyeDropper()` fails immediately. Future implementation would cross screenshot/screen-recording boundaries and needs explicit permission, target, pixel sampling, color-space, and retention policies. The current implementation does not capture the screen or return `sRGBHex`.

### Attempt Find

`find()` is a legacy prompt name that GPUI does not implement. The SDK rejects before sending any JSONL protocol line, so scripts cannot hang waiting for a submit response from a nonexistent backend route.

`find()` does not apply `onlyin`. Use `fileSearch(query, { onlyin })` when the script needs non-interactive Spotlight/mdfind file results constrained to a root, or use `path({ startPath })` / `arg(...)` for supported prompt-driven selection.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Create widget. | `widget()` | Any script runtime. | None. | SDK sends `widget`; Rust maps to coming-soon toast. | Controller exists; visible widget unsupported. | Source audit plus toast route proof. |
| Update widget. | `setState()` | Controller exists. | None. | SDK sends `widgetAction`. | Send attempt only. | Source audit; backend handler required for support. |
| Close widget. | `close()` | Controller exists. | None. | SDK sends `widgetAction`, deletes handler record. | Local handler entry removed. | Source audit; visible close not proven. |
| Handle widget click/input/move/resize. | `onClick` etc. | Handler registered. | Hypothetical widget event. | SDK `widgetEvent` dispatcher. | Callback can run only if event arrives. | Need event producer proof. |
| Capture webcam. | `webcam()`. | None. | None. | SDK throws. | No capture. | Assert throw-before-send. |
| Capture microphone. | `mic()`. | None. | None. | SDK throws. | No capture. | Assert throw-before-send. |
| Pick screen color. | `eyeDropper()`. | None. | None. | SDK throws. | No color result. | Assert throw-before-send. |
| Run find prompt. | `find()`. | No GPUI prompt. | None. | SDK rejects before `nextId`, `addPending`, or `send`. | Typed unsupported error. | Source audit plus Bun rejection/no-stdout proof. |

## State Machine

### Widget Lifecycle

| State | Trigger | Transition |
|---|---|---|
| Script requests widget. | `widget(html, options)`. | SDK creates id, stores handler bucket, sends widget message. |
| Runtime receives message. | `Message::Widget`. | Current GPUI route becomes `WidgetComingSoon`. |
| Controller returned. | SDK resolves immediately. | Script can register handlers or send actions. |
| Action sent. | `setState` or `close`. | SDK emits `widgetAction`; backend handling unproven. |
| Handler event. | Incoming `widgetEvent`. | SDK dispatches to registered callback if any event arrives. |

### Throw-Before-Send Media Helpers

| State | Trigger | Transition |
|---|---|---|
| Script calls helper. | `webcam`, `mic`, or `eyeDropper`. | SDK enters function. |
| Unsupported guard. | Current GPUI implementation. | SDK throws before `send(...)`. |
| Runtime state. | No message emitted. | No GPUI prompt, no hardware capture, no permission request from these calls. |

### Find Request

| State | Trigger | Transition |
|---|---|---|
| Script calls `find()`. | Options include placeholder and optional `onlyin`. | SDK constructs `UnsupportedSdkFeatureError`. |
| Unsupported boundary. | No implemented GPUI find route exists. | SDK rejects before pending-submit setup or protocol send. |
| Alternative route. | Script needs file search with a root. | Use `fileSearch(query, { onlyin })`, the supported non-interactive file-search API. |

## Visual And Focus States

| State | How it appears | Proof path |
|---|---|---|
| Widget unsupported. | Coming-soon toast or no visible widget. | Source route plus runtime toast/state proof. |
| Widget controller exists. | No direct UI required. | SDK unit/source proof of returned methods. |
| Webcam unsupported. | Throw in script output. | Runtime script assertion; no protocol message. |
| Mic unsupported. | Throw in script output. | Runtime script assertion; no protocol message. |
| EyeDropper unsupported. | Throw in script output. | Runtime script assertion; no protocol message. |
| Find unsupported. | No prompt appears. | Source audit plus Bun rejection/no-stdout proof. |

## Keystrokes And Commands

This feature has no stable keyboard or mouse interaction contract today. If a future widget or find UI is implemented, keyboard ownership must be documented in the owning prompt or widget surface. Current verification should use script assertions and protocol/state receipts, not simulated UI interaction.

| Command | Proof rule |
|---|---|
| `widget()` | Prove controller shape separately from visible UI support. |
| `webcam()` | Assert the exact unsupported throw. |
| `mic()` | Assert the exact unsupported throw and keep separate from dictation. |
| `eyeDropper()` | Assert the exact unsupported throw. |
| `find()` | Prove typed unsupported rejection before send; do not document prompt behavior until a real route exists. |

## Actions And Menus

No Actions dialog ownership was proven for these APIs. A future widget surface may need widget-specific actions, and a future find prompt may need prompt actions, but neither should be inferred from the current SDK names.

## Automation And Protocol Surface

| Surface | Status | Notes |
|---|---|---|
| `widget` message. | Routed to coming-soon. | SDK sends; GPUI handler does not implement rendered widgets. |
| `widgetAction` message. | SDK sends; backend unproven. | Required for `setState` and `close`. |
| `widgetEvent` handling. | SDK dispatcher exists. | No runtime producer proven. |
| `webcam` message. | Rust coming-soon route exists. | Current SDK throws before send. |
| `mic` message. | Rust coming-soon route exists. | Current SDK throws before send. |
| `eyeDropper` message. | TypeScript shape exists. | SDK throws before send; Rust route unproven. |
| `find` message. | Not emitted by SDK. | `find()` rejects before send; `fileSearch` / `fileSearchResult` remain the supported file-search protocol route. |

## Data, Storage, And Privacy Boundaries

Widgets would execute or render user-provided HTML if implemented. No sanitization, CSP, sandboxing, trusted event bridge, size limits, or storage policy was proven in the captured context. Treat widget HTML as a high-risk future surface.

Webcam and microphone helpers are currently safe by absence: they throw before capture. Future media support must define permissions, stream lifetime, retention, transcript/media storage, logging redaction, and user-visible capture state.

Eye dropper would require screen pixels. Future implementation must align with screenshot and Screen Recording permission rules, reject blank/unauthorized captures, and avoid retaining private pixels beyond the color result.

`find()` does not interpret `onlyin` because it rejects before send. `fileSearch(query, { onlyin })` owns the supported root-constrained file-search behavior and any path/privacy semantics for this cluster.

## Error, Empty, Loading, And Disabled States

| API | Failure or ambiguous state |
|---|---|
| `widget()` | Controller return can be mistaken for real UI support; current runtime is coming-soon. |
| `setState()` | Sends after `close()` because no closed flag was proven. |
| Widget callbacks. | Registered handlers do nothing unless a `widgetEvent` arrives. |
| `webcam()` | Throws before send; Rust coming-soon route may mislead source readers. |
| `mic()` | Throws before send; unrelated to dictation. |
| `eyeDropper()` | Throws before send despite generated tests that may imply a color result. |
| `find()` | SDK rejects before send; callers get a typed unsupported error instead of hanging on an unhandled backend response. |

## Code Ownership

| Area | Owner skill | Files and references |
|---|---|---|
| SDK globals and controller shape. | `sdk-script-execution` | `scripts/kit-sdk.ts`. |
| Prompt route and unsupported surfaces. | `prompt-runtime` | `src/execute_script/mod.rs`, `src/prompt_handler/mod.rs`. |
| Find unsupported boundary and file-search alternative. | `sdk-script-execution`, `file-search-portals` | `scripts/kit-sdk.ts`; `fileSearch` remains the only supported `onlyin` route. |
| Media boundaries. | `dictation-media`, `platform-windowing-macos` | Media helpers are unsupported; dictation is separate. |
| Automation proof. | `protocol-automation`, `agentic-testing` | Negative tests and state/source receipts. |
| Storage/security review. | `storage-cache-security` | Future widget/media privacy and sandbox requirements. |

## Verification Recipes

### Widget Unsupported Contract

1. Run a script that calls `widget("<button>Hi</button>")`.
2. Assert the returned controller has `setState`, `close`, `onClick`, `onInput`, `onClose`, `onMoved`, and `onResized`.
3. Assert GPUI routes the widget message to coming-soon behavior, not a rendered floating widget.
4. Do not mark widget support as implemented until a visible surface, `widgetAction` backend, event producer, and automation receipts exist.

### Media Throw-Before-Send

1. Run script-level tests for `webcam()`, `mic()`, and `eyeDropper()`.
2. Assert each throws the documented unsupported error.
3. Assert no prompt, permission request, or hardware capture starts.
4. Keep these tests separate from dictation setup and `micro()` prompt coverage.

### Find Explicit Unsupported Boundary

1. Source-audit `scripts/kit-sdk.ts` to assert `globalThis.find` calls `rejectUnsupportedSdkFeature("find", ...)`.
2. Assert the body does not call `nextId`, `addPending`, or `send`, and does not construct `type:"find"`.
3. Assert the SDK Reference marks `find` as Unsupported and points to `fileSearch(query, { onlyin })`.
4. Run a Bun smoke call that `find("...", { onlyin:"/tmp" })` rejects with `ERR_UNSUPPORTED_SDK_FEATURE` and emits no stdout JSONL protocol writes.

## Known Gaps

- No proven visible GPUI widget renderer.
- No proven `widgetAction` backend handler.
- No proven runtime `widgetEvent` producer.
- No proven widget HTML sanitization, sandbox, or security model.
- `webcam()` and `mic()` Rust coming-soon routes are not reached through current SDK globals.
- No proven Rust `eyeDropper` route.
- `find()` remains unsupported as a GPUI prompt, but no longer hangs because the SDK rejects before send.
- Existing tests can pass while proving only SDK function shape, not real runtime behavior; `tests/sdk/test-widget.ts` now asserts the typed unsupported error shape for `find()`.

## Test And Documentation Risks

`tests/sdk/test-widget.ts` proves controller shape and unsupported boundaries, not visible widget behavior. Generated media and find tests can still create false confidence if they only check that names exist or that a body type compiles.

Any docs or agent prompts must distinguish:

- SDK name exists.
- SDK sends a message.
- SDK rejects before send with a typed unsupported error.
- Runtime reaches coming-soon.
- Runtime throws before send.
- Runtime renders a prompt or surface.
- Automation can prove the user-visible result.
