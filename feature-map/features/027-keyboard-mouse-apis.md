# 027 Keyboard and Mouse APIs

This chapter maps the SDK-visible keyboard and mouse helpers, their current unsupported runtime status, and the reliable automation alternatives.

Raw Oracle reference: [answer](../raw-oracle/027-keyboard-mouse-apis/answer.md), [prompt](../raw-oracle/027-keyboard-mouse-apis/prompt.md), [bundle map](../raw-oracle/027-keyboard-mouse-apis/bundle-map.md), [full log](../raw-oracle/027-keyboard-mouse-apis/output.log), [session metadata](../raw-oracle/027-keyboard-mouse-apis/session.json).

## Executive Summary

Feature 027 covers:

- `keyboard.type(text)`.
- `keyboard.tap(...keys)`.
- `mouse.move(positions)`.
- `mouse.leftClick()`.
- `mouse.rightClick()`.
- `mouse.setPosition(position)`.

These helpers are SDK-visible unsupported boundaries, not reliable input controls. Each helper now rejects with `UnsupportedSdkFeatureError` before sending any protocol message. They are useful as API-surface evidence, but not as proof that text was typed, keys were pressed, the cursor moved, a click landed, or native macOS input occurred.

The captured source also records the historical shape mismatch between old SDK payloads and Rust protocol variants. The SDK no longer emits those payloads while unsupported; the reserved Rust protocol variants still expect `keys: Option<String>` for keyboard and `data: Option<MouseData>` for mouse.

Operator truth: do not use SDK keyboard or mouse helpers as verification steps unless the test is specifically proving that the helpers remain unsupported. Use `batch.setInput`, `batch.forceSubmit`, `simulateKey` plus state receipts, `getState`, `getElements`, and `waitFor` for reliable automation.

## What Users Can Do

| Capability | Entry | Current result |
|---|---|---|
| Request text typing. | `await keyboard.type("hello")` | Rejects with `UnsupportedSdkFeatureError`; no protocol message is sent. |
| Request key tap/chord. | `await keyboard.tap("enter")` or `keyboard.tap("command", "k")` | Rejects with `UnsupportedSdkFeatureError`; no key delivery is attempted. |
| Request mouse movement. | `await mouse.move([{ x, y }])` | Rejects with `UnsupportedSdkFeatureError`; no cursor movement is attempted. |
| Request left click. | `await mouse.leftClick()` | Rejects with `UnsupportedSdkFeatureError`; no click is attempted. |
| Request right click. | `await mouse.rightClick()` | Rejects with `UnsupportedSdkFeatureError`; no click is attempted. |
| Request cursor position. | `await mouse.setPosition({ x, y })` | Rejects with `UnsupportedSdkFeatureError`; no cursor movement is attempted. |

What users cannot currently rely on:

- Text insertion into a prompt, editor, terminal, ACP composer, or external app.
- Arrow-key selection movement.
- Enter submission.
- Escape dismissal.
- Cmd+K action dialog opening.
- Cursor movement.
- Left or right click delivery.
- Native macOS event generation.
- Focus or target guarantees.
- Any receipt proving delivery.

## Core Concepts

### SDK Exposure Is Not Implementation

The SDK defines helper functions and global objects. That only proves scripts can call them. It does not prove the GPUI app handles those messages or turns them into real key and mouse events.

### Unsupported Rejection Is The Contract

The helpers reject with `ERR_UNSUPPORTED_SDK_FEATURE` before `send(...)`. They do not register pending callbacks, expect a response message, or validate resulting UI state.

### Protocol Visibility Is Not Native Input

Rust defines `Message::Keyboard` and `Message::Mouse`, plus action enums and constructors. That establishes protocol shapes, not an execution path that performs native input or in-app simulation.

### Payload Shapes Diverge

| Helper | SDK payload | Rust protocol shape | Gap |
|---|---|---|---|
| `keyboard.type(text)` | Unsupported rejection; no send. | `Message::Keyboard { action, keys: Option<String> }` | Historical SDK `text` payload is no longer emitted while unsupported. |
| `keyboard.tap(...keys)` | Unsupported rejection; no send. | `Message::Keyboard { action, keys: Option<String> }` | Historical SDK `keys: string[]` payload is no longer emitted while unsupported. |
| `mouse.move(positions)` | Unsupported rejection; no send. | `Message::Mouse { action, data: Option<MouseData> }` | Historical SDK `positions` payload is no longer emitted while unsupported. |
| `mouse.leftClick()` | Unsupported rejection; no send. | `Message::Mouse { action, data: Option<MouseData> }` | Historical SDK `button` payload is no longer emitted while unsupported. |
| `mouse.rightClick()` | Unsupported rejection; no send. | `Message::Mouse { action, data: Option<MouseData> }` | Historical SDK `button` payload is no longer emitted while unsupported. |
| `mouse.setPosition(position)` | Unsupported rejection; no send. | `Message::Mouse { action, data: Option<MouseData> }` | Historical SDK `position` payload is no longer emitted while unsupported. |

## Entry Points

| Entry | Current behavior | Proof status |
|---|---|---|
| `keyboard.type(text)` | Rejects with `UnsupportedSdkFeatureError`; no keyboard message is sent. | Explicit unsupported receipt at SDK boundary. |
| `keyboard.tap(...keys)` | Rejects with `UnsupportedSdkFeatureError`; no keyboard message is sent. | Explicit unsupported receipt at SDK boundary. |
| `mouse.move(positions)` | Rejects with `UnsupportedSdkFeatureError`; no mouse message is sent. | Explicit unsupported receipt at SDK boundary. |
| `mouse.leftClick()` | Rejects with `UnsupportedSdkFeatureError`; no mouse message is sent. | Explicit unsupported receipt at SDK boundary. |
| `mouse.rightClick()` | Rejects with `UnsupportedSdkFeatureError`; no mouse message is sent. | Explicit unsupported receipt at SDK boundary. |
| `mouse.setPosition(position)` | Rejects with `UnsupportedSdkFeatureError`; no mouse message is sent. | Explicit unsupported receipt at SDK boundary. |
| `simulateKey` | Protocol automation command, separate from SDK keyboard helpers. | Fire-and-forget; prove with follow-up state. |
| `batch.setInput` | Receipt-backed input mutation command. | Preferred proof path for prompt text. |

## User Workflows

### Type Into A Prompt

A script may call:

```ts
await keyboard.type("ru")
```

Current source truth: the SDK rejects with `UnsupportedSdkFeatureError` before sending a keyboard message. No state changes are attempted. The reliable alternative is `batch.setInput` followed by `waitFor`, `getState`, or `getElements`.

### Submit With Enter

A script may call:

```ts
await keyboard.tap("enter")
```

Current source truth: the SDK rejects with `UnsupportedSdkFeatureError` before sending a keyboard message. Rust still has a reserved keyboard protocol shape that expects an optional string, but no focus, route, submit, or selected-row proof exists. Use `batch.forceSubmit` for direct submit behavior, or `simulateKey` plus state receipts when testing key routing.

### Open Actions With Cmd+K

A script may call:

```ts
await keyboard.tap("command", "k")
```

Current source truth: this is not a reliable way to open the Actions dialog. Tests that use it often include manual fallback or permissive outcomes. Use the supported popup/action route when available, or use `simulateKey` and then inspect `activePopupContract` and elements.

### Move Or Click The Mouse

A script may call:

```ts
await mouse.move([{ x: 100, y: 100 }])
await mouse.leftClick()
```

Current source truth: the SDK rejects with `UnsupportedSdkFeatureError` before sending mouse messages. The captured context does not prove cursor movement, coordinate-space handling, click delivery, or native event generation.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Type text. | `keyboard.type("abc")` | Any prompt. | Synthetic typing request. | SDK rejects before send. | Unsupported; explicit SDK error. | Source audit only. |
| Tap Enter. | `keyboard.tap("enter")` | Prompt/list. | Enter request. | SDK rejects before send. | Unsupported; explicit SDK error. | Source audit only. |
| Navigate choices. | `keyboard.tap("down")` | List prompt. | Arrow request. | SDK rejects before send. | Unsupported; explicit SDK error. | Use `simulateKey` + `getState`. |
| Dismiss prompt. | `keyboard.tap("escape")` | Prompt/popup. | Escape request. | SDK rejects before send. | Unsupported; explicit SDK error. | Use `simulateKey` + `windowVisible` or popup state. |
| Open actions. | `keyboard.tap("command", "k")` | Prompt/list. | Cmd+K request. | SDK rejects before send. | Unsupported; explicit SDK error. | Use popup state receipts. |
| Move cursor. | `mouse.move([{x,y}])` | Any window. | Mouse move request. | SDK rejects before send. | Unsupported; explicit SDK error. | Source audit only. |
| Click. | `mouse.leftClick()` | Any window. | Left click request. | SDK rejects before send. | Unsupported; explicit SDK error. | Source audit only. |
| Set cursor. | `mouse.setPosition({x,y})` | Any display/window. | Cursor position request. | SDK rejects before send. | Unsupported; explicit SDK error. | Source audit only. |
| Mutate prompt input. | `batch([{ type:"setInput", text }])` | Target prompt. | No native key. | Protocol batch executor. | Receipt-backed state mutation. | `batchResult`, `getState`, `getElements`. |
| Test key routing. | `simulateKey` | Target surface. | Simulated key. | Stdin/protocol automation. | Fire-and-forget. | Follow-up `getState`, `getElements`, or `waitFor`. |

## State Machine

### SDK Helper Current State

| State | Trigger | Transition |
|---|---|---|
| Script call. | `keyboard.*` or `mouse.*`. | SDK constructs `UnsupportedSdkFeatureError`. |
| Promise rejection. | Helper returns. | Promise rejects with `ERR_UNSUPPORTED_SDK_FEATURE`. |
| Message send. | Not reached. | No keyboard or mouse protocol message is emitted. |
| Backend effect. | Not reached. | No native input, cursor movement, or click delivery is attempted. |

### Reliable Automation State

| State | Trigger | Transition |
|---|---|---|
| Prompt mutation. | `batch.setInput`. | Batch registers request id and returns `batchResult`. |
| Key routing. | `simulateKey`. | Command is fire-and-forget. |
| Proof. | `getState`, `getElements`, or `waitFor`. | Agent asserts semantic state after command. |

## Visual And Focus States

The SDK keyboard and mouse helpers do not expose focus state, target identity, window identity, coordinate space, disabled state, loading state, or hover/click state. A future native implementation must define whether events target the Script Kit main window, focused Script Kit surface, frontmost macOS app, current cursor position, or an explicit automation target.

## Keystrokes And Commands

| Intent | Unsupported SDK helper | Reliable route today |
|---|---|---|
| Type prompt text. | `keyboard.type(text)` | `batch.setInput` plus `getState`/`waitFor`. |
| Submit prompt. | `keyboard.tap("enter")` | `batch.forceSubmit` when direct submit is desired; `simulateKey` plus state when testing routing. |
| Dismiss prompt/popup. | `keyboard.tap("escape")` | `simulateKey("escape")` plus close/popup receipts. |
| Open Actions. | `keyboard.tap("command", "k")` | Action/popup route where available; otherwise `simulateKey` plus `activePopupContract`. |
| Navigate list. | `keyboard.tap("down")` | `simulateKey` plus selected row state/elements. |

## Actions And Menus

The keyboard/mouse helpers do not own actions, menus, or command execution. Actions behavior belongs to the prompt runtime and actions popup surfaces. When a workflow needs an action menu, prove that menu through action/popup state and semantic elements rather than SDK keyboard or mouse stubs.

## Automation And Protocol Surface

| Surface | Status | How to prove |
|---|---|---|
| SDK `keyboard.*`. | Exposed but unsupported. | Source audit `UnsupportedSdkFeatureError` and no `send(...)` path. |
| SDK `mouse.*`. | Exposed but unsupported. | Source audit `UnsupportedSdkFeatureError` and no `send(...)` path. |
| Rust `Message::Keyboard`. | Typed protocol shape exists. | Source audit variants/constructors only. |
| Rust `Message::Mouse`. | Typed protocol shape exists. | Source audit variants/constructors only. |
| `simulateKey`. | Separate protocol automation command. | Follow with state/elements/wait receipts. |
| `batch.setInput`. | Receipt-backed prompt input mutation. | Assert `batchResult` and prompt state. |
| `getState` / `getElements` / `waitFor`. | Reliable observation surfaces. | Use after fire-and-forget commands. |

## Data, Storage, And Privacy Boundaries

The captured keyboard/mouse helpers do not show a storage path. They do transmit requested text, key names, mouse coordinates, and click intent through the SDK message channel. If native input is implemented later, the privacy and safety boundary becomes larger because text could contain secrets, shortcuts could trigger destructive behavior, and mouse coordinates could act outside the app if focus is wrong.

Native input implementation must define:

- Permission requirements.
- Focus ownership.
- Target window identity.
- Coordinate system.
- Result receipts or explicit fire-and-forget policy.
- Safe failure behavior.

## Error, Empty, Loading, And Disabled States

| State | Current behavior |
|---|---|
| Unsupported. | Explicit SDK rejection with `ERR_UNSUPPORTED_SDK_FEATURE`. |
| No-op. | Prevented at the SDK boundary because helper bodies reject before `send(...)`. |
| Serialization mismatch. | Historical SDK/Rust shapes diverged; SDK no longer emits those payloads while unsupported. |
| False-positive tests. | Method existence or no-throw tests prove API surface only. |
| Race. | Fixed sleeps do not prove helper delivery. |
| Focus ambiguity. | No target or focus receipt exists. |
| Disabled/loading. | No capability flag or disabled helper state is exposed in the captured surface. |

## Code Ownership

| Area | Owner skill | Files and references |
|---|---|---|
| SDK helper definitions. | `sdk-script-execution` | `scripts/kit-sdk.ts`. |
| Keyboard/focus semantics. | `keyboard-focus-routing` | Key routing, popup-first behavior, focus restore, and `simulateKey` proof paths. |
| Protocol message shapes. | `protocol-automation` | `src/protocol/message/variants/system_control.rs`, `src/protocol/message/constructors/general.rs`, `src/protocol/types/primitives.rs`. |
| Native input/focus/windowing. | `platform-windowing-macos` | AppKit, Accessibility, focus, window targeting, and native event policy. |
| Runtime proof. | `agentic-testing` | State-first receipts, screenshots only when state cannot answer. |

## Invariants And Regression Risks

- Keep unsupported rejection until real backend behavior and proof exist.
- Do not treat `send(...)` success or `Promise<void>` resolution as input success.
- Reconcile SDK and Rust payload shapes before implementing runtime handling.
- Do not conflate SDK `keyboard.tap()` with protocol `simulateKey`.
- Keep `batch.setInput` as the preferred prompt text mutation proof path.
- Do not replace receipt-backed tests with SDK keyboard/mouse helper calls.
- Native OS input needs explicit permission, focus, target, and coordinate policy before it is safe to expose as supported.

## Verification Recipes

### Source-Audit Unsupported Status

Inspect `scripts/kit-sdk.ts` for `UnsupportedSdkFeatureError`, helper rejection paths, and lack of `send(...)` calls inside the keyboard and mouse helper bodies. Confirm the SDK response union has no keyboard or mouse result response.

### Source-Audit Protocol Shape Mismatch

Compare SDK keyboard/mouse interfaces with Rust `Message::Keyboard`, `Message::Mouse`, `KeyboardAction`, `MouseAction`, and constructor helpers. The expected finding is `text`/array/position fields on the SDK side versus `keys: Option<String>` and `data: Option<MouseData>` on the Rust side.

### Prove Prompt Input Reliably

Use `batch.setInput`, then assert the postcondition through `batchResult`, `getState`, `getElements`, or `waitFor`. Do not use `keyboard.type` for this proof.

### Prove Key Routing Reliably

Use protocol `simulateKey`, then inspect state. Good postconditions include selected index, selected semantic id, popup contract, window visibility, current view, or submit result.

### Prove Native Input Only After Implementation

Require source evidence for the native handler, permission checks, focus/target handling, and deterministic postconditions. Native proof should be a narrow test tier, not the default feature-map proof path.

## Agent Notes

- Do not assume SDK-visible keyboard and mouse helpers perform native input.
- Do not use these helpers to prove UI behavior unless the test is explicitly about unsupported API status.
- To verify prompt text, use `batch.setInput` and inspect state.
- To verify key routing, use `simulateKey` and follow with `getState`, `getElements`, or `waitFor`.
- If a benchmark uses `keyboard.tap`, treat its timing as helper-call overhead unless native delivery is separately proven.
- This belongs to `sdk-script-execution`, `protocol-automation`, `keyboard-focus-routing`, and `platform-windowing-macos`.
- This does not belong to Actions popup behavior except where a workflow falsely tries to use keyboard/mouse stubs to open or interact with actions.

## Related Features

- [025 System Feedback and Prompt Control APIs](./025-system-feedback-and-prompt-control.md) owns `setInput`, `setActions`, HUD, and prompt-control behavior.
- [026 Clipboard, Selected Text, and Accessibility APIs](./026-clipboard-selected-text-accessibility.md) owns selected-text/native paste behavior and must not be generalized into keyboard/mouse support.
- [004 MCP Context Resources / SDK / Protocol Automation](./004-mcp-sdk-protocol.md) owns `getState`, `getElements`, `waitFor`, `batch`, and protocol proof patterns.
- [011 Root Unified Search Result Actions](./011-root-source-actions.md) and [024 Confirm Prompt and Dialogs](./024-confirm-prompt-and-dialogs.md) cover action and confirm workflows that should be proven through semantic state, not mouse stubs.

## Open Questions And Gaps

- No app-side native handler was proven in the captured context.
- SDK and Rust payload shapes do not align.
- No keyboard or mouse result envelope exists in the captured SDK response surface.
- Mouse coordinate space is undefined.
- Focus and target policy are undefined.
- Permission policy for native input is undefined.
- Test suites that call these helpers may be false positives unless they assert independent state receipts.
