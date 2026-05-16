# 026 Clipboard, Selected Text, and Accessibility APIs

This chapter maps system clipboard helpers, selected-text read/write, and Accessibility permission APIs exposed through the SDK and automation protocol.

Raw Oracle reference: [answer](../raw-oracle/026-clipboard-selected-text-accessibility/answer.md), [prompt](../raw-oracle/026-clipboard-selected-text-accessibility/prompt.md), [bundle map](../raw-oracle/026-clipboard-selected-text-accessibility/bundle-map.md), [full log](../raw-oracle/026-clipboard-selected-text-accessibility/output.log), [session metadata](../raw-oracle/026-clipboard-selected-text-accessibility/session.json).

## Executive Summary

Feature 026 covers:

- `copy()` / `paste()`.
- `clipboard.readText()` / `clipboard.writeText()`.
- `clipboard.readImage()` / `clipboard.writeImage()`.
- `getSelectedText()` / `setSelectedText()`.
- `hasAccessibilityPermission()` / `requestAccessibilityPermission()`.

The implementation is split across the SDK, protocol types, executor clipboard handling, stdin selected-text/accessibility handlers, and macOS selected-text helpers.

Clipboard text read/write is implemented through executor-side `Message::Clipboard` and `arboard::Clipboard`. Error signaling is weak: read failures and genuinely empty clipboard both resolve as `""`, and `writeText` resolves on any response.

Clipboard image support is not fully production-grade in the captured source. `readImage()` base64-decodes raw `arboard` image bytes into a `Buffer` without width/height/pixel-format metadata. `writeImage()` sends base64 with `format:"image"`, but the shown Rust write path writes `content` as text and does not decode/set an image clipboard payload.

Selected-text APIs are macOS-first and require Accessibility permission. The SDK hides Script Kit, waits 20ms for focus to return to the previous app, then sends the request. Rust selected-text reads use an AX-first helper with clipboard fallback; writes use clipboard + Core Graphics Cmd+V and best-effort clipboard restoration.

Automation over stdin uses typed response variants:

| Request | Response |
|---|---|
| `getSelectedText` | `selectedText` |
| `setSelectedText` | `textSet` |
| `checkAccessibility` | `accessibilityStatus` |
| `requestAccessibility` | `accessibilityStatus` |

Source-audit tests pin request-id correlation, typed response constructors, and privacy logging rules. Selected text logs must use `text_len`, never raw selected or replacement text.

## What Users Can Do

| Capability | Entry | Result |
|---|---|---|
| Copy text. | `await copy(text)` or `clipboard.writeText(text)` | Writes text to system clipboard. |
| Paste/read text. | `await paste()` or `clipboard.readText()` | Resolves current clipboard text or `""`. |
| Read image bytes. | `await clipboard.readImage()` | Resolves a `Buffer` from base64 raw image bytes; format metadata absent. |
| Attempt image write. | `await clipboard.writeImage(buffer)` | SDK sends base64, but Rust write path appears text-only. |
| Read selected app text. | `await getSelectedText()` | Hides Script Kit, reads focused app selection, resolves text or rejects on SDK executor error. |
| Replace selected app text. | `await setSelectedText(text)` | Hides Script Kit, pastes text into focused app through clipboard/Cmd+V. |
| Check Accessibility. | `await hasAccessibilityPermission()` | Resolves current Accessibility trust status. |
| Request Accessibility. | `await requestAccessibilityPermission()` | May trigger macOS permission prompt, resolves trust status. |
| Use template variables. | `template()` with `$SELECTION` / `$CLIPBOARD`. | SDK expands from `getSelectedText()` / `clipboard.readText()`. |

## Core Concepts

### Clipboard API

SDK interface:

```ts
interface ClipboardAPI {
  readText(): Promise<string>
  writeText(text: string): Promise<void>
  readImage(): Promise<Buffer>
  writeImage(buffer: Buffer): Promise<void>
}
```

SDK message shape:

```ts
interface ClipboardMessage {
  type: "clipboard"
  id: string
  action: "read" | "write"
  format: "text" | "image"
  content?: string
}
```

Rust protocol shape:

```rust
Message::Clipboard {
    id: Option<String>,
    action: ClipboardAction,
    format: Option<ClipboardFormat>,
    content: Option<String>,
}
```

With an `id`, the executor sends `Message::Submit { id, value }`. Without an `id`, a write is treated as best-effort/no-response text write.

### Aliases

`copy(text)` is `clipboard.writeText(text)`. `paste()` is `clipboard.readText()`.

### Selected Text

The SDK focus handoff is part of the contract:

1. `await hide()`.
2. Wait 20ms.
3. Send `getSelectedText` or `setSelectedText`.

This gives the previous app a chance to regain focus before AX or Cmd+V operations run.

### Accessibility

`hasAccessibilityPermission()` is read-only. `requestAccessibilityPermission()` can prompt through macOS Accessibility trust APIs. These must not be collapsed into one behavior.

## Entry Points

| Entry | Payload | Backend path | Result |
|---|---|---|---|
| `copy(text)` | Clipboard write text. | SDK alias -> executor `ClipboardAction::Write`. | Promise resolves on response. |
| `paste()` | Clipboard read text. | SDK alias -> executor `ClipboardAction::Read`. | Text or `""`. |
| `clipboard.readText()` | `format:"text"` read. | `arboard::Clipboard::get_text`. | Text or `""`. |
| `clipboard.writeText(text)` | `format:"text"` write. | `arboard::Clipboard::set_text`. | `"ok"` submit on success, but SDK ignores value. |
| `clipboard.readImage()` | `format:"image"` read. | `arboard::Clipboard::get_image`, base64 raw bytes. | `Buffer`; empty on failure. |
| `clipboard.writeImage(buffer)` | `format:"image"` write. | SDK sends base64; captured executor writes `content` as text. | Known gap. |
| `getSelectedText()` | `requestId`. | SDK hide/delay -> selected-text probe. | Selected text or error/empty depending path. |
| `setSelectedText(text)` | `requestId`, text. | SDK hide/delay -> clipboard fallback paste. | Success or error. |
| `hasAccessibilityPermission()` | `checkAccessibility`. | Permission wizard check. | Boolean. |
| `requestAccessibilityPermission()` | `requestAccessibility`. | Permission wizard request/prompt. | Boolean. |

## User Workflows

### Copy Text

A script calls:

```ts
await copy("hello")
```

The SDK forwards to `clipboard.writeText`, sends a clipboard write message, and the executor calls `arboard::Clipboard::set_text`. On success Rust returns submit value `"ok"`. The SDK resolver ignores the response value and resolves on any response, so follow-up `paste()` is the real proof.

### Paste Text

A script calls:

```ts
const text = await paste()
```

The SDK sends a clipboard read text message. The executor calls `arboard::Clipboard::get_text`. On error, it logs and returns `""`. Empty clipboard and read failure are indistinguishable at the SDK surface.

### Read Image

`clipboard.readImage()` sends a clipboard read image message. The executor gets image data from `arboard`, base64-encodes `img.bytes`, and the SDK decodes to `Buffer`.

Do not assume PNG bytes. Width, height, row stride, pixel format, and encoded container metadata are absent in the captured contract.

### Write Image

`clipboard.writeImage(buffer)` sends base64 content with `format:"image"`. The captured Rust write branch does not branch on image format and writes `content` as text, so this should be documented as a gap until proven otherwise.

### Get Selected Text

The SDK hides the Script Kit window, waits 20ms, sends `getSelectedText`, and waits for a response. The macOS selected-text module checks Accessibility permission, then uses the `get-selected-text` crate, which tries AX selection paths and fallback clipboard simulation.

Executor-style SDK errors can reject with an `ERROR:` value. Stdin automation uses a typed `selectedText` response and logs error presence while returning empty text on probe failure.

### Set Selected Text

The SDK hides the window, waits 20ms, then sends `setSelectedText`. Rust checks Accessibility permission and calls `set_selected_text(&text)`.

The write strategy:

1. Snapshot every current `NSPasteboardItem` type/data representation.
2. Set clipboard to replacement plain text.
3. Simulate Cmd+V through Core Graphics.
4. Wait for paste.
5. Restore the full pasteboard snapshot, including rich text, image, file URL, and unknown item representations, unless the pasteboard changed during the paste window.
6. Return paste errors and explicit restore errors.

Snapshot failure aborts before mutating the clipboard. Restore failure is returned after the paste attempt so callers do not silently lose rich/image/file clipboard contents. If another process changes the pasteboard during the paste window, restore is skipped and the failure is explicit so Script Kit does not overwrite newer clipboard state.

### Check Or Request Accessibility

`hasAccessibilityPermission()` sends `checkAccessibility` and resolves `true` or `false`. `requestAccessibilityPermission()` sends `requestAccessibility`, calls the prompting trust API, and resolves the resulting trust status.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Copy text. | `copy(text)` | None. | SDK call. | SDK alias -> clipboard write text. | Clipboard text set, weak success signal. | `scripts/kit-sdk.ts`, `src/execute_script/mod.rs`. |
| Paste text. | `paste()` | None. | SDK call. | SDK alias -> clipboard read text. | Text or `""`. | `src/execute_script/mod.rs`. |
| Read image. | `clipboard.readImage()` | None. | SDK call. | Clipboard read image -> base64 raw bytes. | Buffer, format ambiguous. | `src/execute_script/mod.rs`. |
| Write image. | `clipboard.writeImage(buffer)` | None. | SDK call. | Sends base64 with image format. | Gap: write path appears text-only. | `scripts/kit-sdk.ts`, `src/execute_script/mod.rs`. |
| Read selected text. | `getSelectedText()` | Previous app focused. | Hide + delay. | SDK -> selected text probe. | Selected text or failure/empty. | `src/selected_text.rs`. |
| Replace selected text. | `setSelectedText(text)` | Previous app focused. | Hide + delay + Cmd+V. | Clipboard fallback paste. | Text inserted or error. | `src/selected_text.rs`. |
| Check permission. | `hasAccessibilityPermission()` | None. | SDK call. | `checkAccessibility`. | Boolean trust status. | `src/prompt_handler/mod.rs`. |
| Request permission. | `requestAccessibilityPermission()` | macOS prompt possible. | SDK call. | `requestAccessibility`. | Boolean trust status. | `src/permissions_wizard.rs`. |
| Stdin selected text. | `session.sh rpc getSelectedText`. | Frontmost app. | RPC. | Typed stdin arm. | `selectedText` receipt. | source-audit tests. |
| Stdin set text. | `session.sh rpc setSelectedText`. | Frontmost app. | RPC. | Typed stdin arm. | `textSet` receipt. | source-audit tests. |

## State Machine

### Clipboard Text Read/Write

| State | Trigger | Transition |
|---|---|---|
| SDK call. | `copy`, `paste`, `clipboard.*`. | SDK creates `id`, registers pending resolver. |
| Protocol message. | `Message::Clipboard`. | Executor handles without UI. |
| Read text. | `ClipboardAction::Read`, text/none. | `get_text`; submit text or `""`. |
| Write text. | `ClipboardAction::Write`. | `set_text`; submit `"ok"` or `""`. |
| SDK resolution. | Submit received. | Read resolves value; write resolves void. |

### Selected Text Read

| State | Trigger | Transition |
|---|---|---|
| SDK call. | `getSelectedText()`. | Hide Script Kit. |
| Focus yield. | 20ms delay. | Previous app should regain focus. |
| Probe. | Rust selected-text helper. | Check Accessibility, then AX/clipboard fallback. |
| Response. | Success/failure. | SDK resolves text or rejects; stdin returns typed receipt. |

### Selected Text Write

| State | Trigger | Transition |
|---|---|---|
| SDK call. | `setSelectedText(text)`. | Hide Script Kit. |
| Focus yield. | 20ms delay. | Previous app should regain focus. |
| Permission gate. | `set_selected_text`. | Bail if Accessibility missing. |
| Clipboard fallback. | Save text clipboard, set replacement. | Clipboard now contains replacement text. |
| Paste. | Core Graphics Cmd+V. | Target app receives paste. |
| Restore. | Rebuild all saved pasteboard items/types/data. | Original text, rich text, image, file URL, and unknown representations restored when possible. |
| Response. | Paste result. | Success or error receipt. |

## Visual And Focus States

| State | Visible/focus behavior |
|---|---|
| Clipboard read/write | No UI required. |
| `getSelectedText()` | Script Kit hides so previous app can be queried. |
| `setSelectedText()` | Script Kit hides so previous app receives Cmd+V. |
| Permission request | macOS may show a system Accessibility prompt/settings flow. |
| Adjacent paste flows | Emoji/Clipboard History hide or keep windows according to their own contracts before simulated paste. |

## Keystrokes And Commands

| Command/key | Owner |
|---|---|
| Cmd+C fallback | `get-selected-text` crate may use clipboard simulation internally. |
| Cmd+V simulation | `set_selected_text` / paste finalization uses Core Graphics. |
| `session.sh rpc checkAccessibility --expect accessibilityStatus` | Stdin automation. |
| `session.sh rpc requestAccessibility --expect accessibilityStatus` | Stdin automation, may prompt. |
| `session.sh rpc getSelectedText --expect selectedText` | Stdin automation. |
| `session.sh rpc setSelectedText --expect textSet` | Stdin automation. |

## Actions And Menus

This feature owns the SDK/system clipboard and selected-text APIs. Adjacent surfaces use clipboard mechanics but own their domain behavior:

| Adjacent surface | Relationship |
|---|---|
| Clipboard History | Stores/list/searches clipboard history; paste/copy actions reuse clipboard + simulated paste patterns. |
| Emoji Picker | Writes selected emoji to clipboard and simulates paste. |
| Sharing watcher | Watches clipboard share URIs and asks trust prompt before install. |
| AI/chat | Copy/export/paste-image actions use clipboard. |
| Menu/action effects | Some actions write clipboard content as an effect. |

## Automation And Protocol Surface

| Surface | Receipt shape |
|---|---|
| SDK clipboard | Executor `Submit { id, value }`. |
| SDK selected text executor path | Executor `Submit` shape with values/errors. |
| Stdin `getSelectedText` | `SelectedText { text, requestId }`. |
| Stdin `setSelectedText` | `TextSet { success, error?, requestId }`. |
| Stdin `checkAccessibility` / `requestAccessibility` | `AccessibilityStatus { granted, requestId }`. |

Source-audit tests require:

- Explicit dispatcher arms in `handle_stdin_protocol_message`.
- Shared helper calls (`selected_text::*`, `permissions_wizard::*`).
- Constructor helpers for responses.
- `response_sender.try_send(response)`.
- Request-scoped tracing event types.
- Length-only selected-text logging.

## Data, Storage, And Privacy Boundaries

| Data | Boundary |
|---|---|
| Clipboard text | Can be private; SDK read returns raw text to script. |
| Clipboard images | Can be large/private; current Buffer lacks metadata. |
| Selected text | Must not be logged raw; source tests pin `text_len` logging. |
| Replacement text | Must not be logged raw; source tests pin `text_len` logging. |
| Request ids | Echoed for correlation. |
| Accessibility status | Boolean only. |
| Clipboard restoration | Full pasteboard item/type/data snapshot before mutation; restore failures are explicit. |
| Clipboard History | May persist content; not owned by this SDK API chapter. |

Privacy regressions include logging `text = %text`, returning raw selected text in broad state receipts, or content-heavy generic action receipts.

## Error, Empty, Loading, And Disabled States

| Case | Behavior/risk |
|---|---|
| Empty clipboard | `readText` resolves `""`. |
| Clipboard read failure | Also resolves `""`; indistinguishable from empty. |
| Clipboard write failure | Executor can return `""`; SDK `writeText` still resolves on response. |
| `readImage` failure | Resolves empty `Buffer`. |
| `writeImage` | Known gap; likely writes base64 as text in captured source. |
| No Accessibility permission | Selected-text read/write fails or returns error/empty depending path. |
| Permission request denied/pending | Resolves false. |
| Focus handoff race | 20ms delay may be insufficient on slow systems. |
| Clipboard snapshot failure | Fails before clipboard mutation. |
| Clipboard restore failure | Logs content-light snapshot metadata and returns an explicit error after paste attempt. |
| Clipboard changed during paste | Skips restore and returns an explicit error to avoid overwriting newer clipboard state. |
| Non-macOS selected text | Module bails/returns unsupported behavior. |
| Response sender missing | Stdin arm logs warning; caller times out. |
| Auto-submit fallback | SDK defaults can mask missing handlers in generated tests. |

## Code Ownership

| Owner | Responsibility |
|---|---|
| `platform-windowing-macos` | AX, Core Graphics paste, focus handoff, permission prompt behavior. |
| `storage-cache-security` | Clipboard persistence/cache boundaries and privacy. |
| `sdk-script-execution` | SDK globals, executor message handling, script pending responses. |
| `protocol-automation` | Stdin typed receipts and request-id correlation. |
| `agentic-testing` | Receipt-first verification and native focus proof when needed. |

Key files include `scripts/kit-sdk.ts`, `src/execute_script/mod.rs`, `src/selected_text.rs`, `src/prompt_handler/mod.rs`, `src/permissions_wizard.rs`, protocol variants/constructors, and source-audit tests for selected-text/accessibility wiring.

## Invariants And Regression Risks

| Invariant | Risk |
|---|---|
| Clipboard text read/write stays executor-side and response-correlated. | SDK promises resolve without a real clipboard operation. |
| Empty/error clipboard reads remain understood as ambiguous. | Agents overclaim clipboard state. |
| Image clipboard support remains documented as incomplete. | Users treat raw bytes or text-write gap as production image support. |
| SDK hides before selected-text operations. | Script Kit reads/pastes into itself instead of previous app. |
| Accessibility check/request remain distinct. | Read-only checks unexpectedly prompt, or request stops prompting. |
| Stdin uses typed receipts, not executor `Submit`. | `session.sh rpc --expect ...` breaks. |
| Selected-text logs never include raw text. | Sensitive user content leaks to app logs. |
| Clipboard restore occurs after paste attempt and restores all saved pasteboard item representations. | Target app may miss paste or clipboard may not be restored; restore failure must be explicit. |
| Non-macOS selected text stays explicit unsupported. | Cross-platform callers get false confidence. |

## Verification Recipes

| Recipe | Proof |
|---|---|
| Clipboard text roundtrip | `await copy(value); const actual = await paste(); assert(actual === value)`. |
| SDK focus handoff | `cargo test --test config_contract_alignment kit_sdk_yields_focus_before_set_selected_text -- --nocapture`. |
| Stdin selected text wiring | `cargo test --test source_audits stdin_get_selected_text_wired -- --nocapture`. |
| Stdin set text wiring | `cargo test --test source_audits stdin_set_selected_text_wired -- --nocapture`. |
| Selected-text rich clipboard restore | `cargo test --test source_audits selected_text_clipboard_restore -- --nocapture`. |
| Stdin check accessibility wiring | `cargo test --test source_audits stdin_check_accessibility_wired -- --nocapture`. |
| Stdin request accessibility wiring | `cargo test --test source_audits stdin_request_accessibility_wired -- --nocapture`. |
| Selected-text unit timing/restore | `cargo test selected_text -- --nocapture`. |
| Clipboard action adjacency | `cargo test --test source_audits clipboard_actions -- --nocapture`. |
| Manual selected text | Select text in TextEdit, run `getSelectedText`, then `setSelectedText("REPLACED")`. |
| Stdin receipts | `session.sh rpc checkAccessibility --expect accessibilityStatus`, etc. |
| Image clipboard | Test `readImage` only as non-empty raw buffer; treat `writeImage` as a gap until source proves image write. |

## Agent Notes

Do not assume `clipboard.readText()` returning `""` means the clipboard is empty.

Do not assume `clipboard.writeText()` succeeded just because the SDK Promise resolved; verify with a readback when it matters.

Do not assume `clipboard.readImage()` returns PNG. Do not document `clipboard.writeImage()` as working without a source/runtime fix.

Do not log raw selected text, replacement text, pasteboard type names, or pasteboard bytes. Length-only and content-light summary logging is deliberate and pinned.

Do not collapse `checkAccessibility` and `requestAccessibility`. One is read-only; one can prompt.

For selected-text runtime proof, native focus matters. Use a real macOS app target and verify the frontmost app receives the operation.

## Related Features

| Feature | Relationship |
|---|---|
| 008 Root Unified Search Clipboard History | Clipboard history storage/list/search is separate. |
| Sharing Clipboard Trust Prompt | Watches clipboard for share URIs and opens trust prompt. |
| Emoji Picker | Uses clipboard write + simulated paste pattern. |
| AI/Chat copy/export | Uses clipboard for exported content and image paste. |
| Keyboard/Mouse APIs | General input APIs are separate; this feature only owns Cmd+V simulation used here. |
| Permission Assistant | Setup UI for permissions is adjacent; this feature owns API calls. |
| MCP computer permission tools | Separate observation-only permission surfaces. |

## Open Questions And Gaps

| Gap | Why it matters |
|---|---|
| `clipboard.writeImage()` appears incomplete. | SDK sends base64 image content, but executor writes text. |
| `clipboard.readImage()` payload is underspecified. | No width/height/format metadata; cannot safely treat as PNG. |
| Clipboard operation errors collapse to `""` or ignored values. | Scripts cannot distinguish empty clipboard from failure. |
| SDK auto-submit fallbacks can mask missing handlers. | Generated tests can pass without real behavior. |
| Stdin vs executor selected-text errors diverge. | Stdin softens read errors to empty text; SDK path can reject. |
| Focus timing is heuristic. | 20ms/100ms waits can race on slow systems. |
| Non-macOS behavior differs by path. | Needs explicit docs/tests before cross-platform claims. |
| No image clipboard contract test in bundle. | Good candidate before advertising image clipboard support. |
| Response sender absence still times out callers. | Automation harness should watch logs for missing sender warnings. |
