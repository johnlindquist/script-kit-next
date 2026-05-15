# 025 System Feedback and Prompt Control APIs

This chapter maps SDK utility APIs that either show lightweight user feedback or mutate an already-running prompt: `beep()`, `say()`, `notify()`, `hud()`, `setStatus()`, `menu()`, `setActions()`, and `setInput()`.

Raw Oracle reference: [answer](../raw-oracle/025-system-feedback-and-prompt-control/answer.md), [prompt](../raw-oracle/025-system-feedback-and-prompt-control/prompt.md), [bundle map](../raw-oracle/025-system-feedback-and-prompt-control/bundle-map.md), [full log](../raw-oracle/025-system-feedback-and-prompt-control/output.log), [session metadata](../raw-oracle/025-system-feedback-and-prompt-control/session.json).

## Executive Summary

Feature 025 splits into two truth classes:

| Class | APIs | Current truth |
|---|---|---|
| Implemented app behavior | `hud()`, `setActions()`, `setInput()`, `beep()`, `say()`, `notify()` | These have Rust-side behavior: HUD overlays, SDK action/shortcut updates, prompt/input mutation, and platform feedback dispatch. |
| Explicit unsupported surfaces | `setStatus()`, `menu()` | The SDK returns typed unsupported results before sending because there is no visible GPUI status surface, tray/menu mutation handler, or receipt contract. |

Operationally: use `hud()` for visible in-launcher feedback, `notify()` for OS notifications, `beep()`/`say()` for receipt-backed platform dispatch requests, `setActions()` for prompt-level actions and shortcuts, and `setInput()` or automation `batch.setInput` for prompt mutation. Do not promise visible status chrome or tray/menu mutation from `setStatus()` or `menu()` until backend behavior is added and verified.

## What Users Can Do

| Capability | Entry | Result |
|---|---|---|
| Show lightweight feedback. | `hud("Saved", { duration })` | Standalone overlay appears and auto-dismisses without revealing the launcher. |
| Replace prompt actions. | `setActions(actions)` | SDK stores JS handlers, Rust updates action state/shortcuts, open actions dialog refreshes. |
| Trigger action handlers. | Shortcut or actions dialog. | SDK receives `actionTriggered` and calls local `onAction`. |
| Submit via action value. | Action with `value` and no `onAction`. | SDK sends `forceSubmit` with that value. |
| Set active prompt input. | `setInput("query")` | Fire-and-forget message mutates supported prompt input. |
| Set target input with receipt. | Automation `batch` `setInput`. | Target-specific mutation returns `batchResult`. |
| Dispatch platform feedback. | `beep`, `say`, `notify`. | Message is sent with a request id and resolves from `systemFeedbackResult`; delivery remains OS dependent. |
| Attempt unsupported status/menu calls. | `setStatus`, `menu`. | Promise resolves to `ERR_UNSUPPORTED_SDK_FEATURE`; no protocol message is sent. |

## Core Concepts

### Fire-And-Forget Calls

These APIs must distinguish app-originated receipts from local serialization. `hud()` and direct `setInput()` remain fire-and-forget, while `beep()`, `say()`, and `notify()` wait for dispatch receipts.

Automation should not infer success from the SDK call returning. Verify with `getState`, `getElements`, `waitFor`, batch results, smoke tests, or visual/window proof as appropriate.

### Protocol Message Families

The relevant protocol messages include:

- `hud`
- `setInput`
- `setActions`
- `actionTriggered`
- `forceSubmit`
- `notify`
- `beep`
- `say`
- `setStatus`
- `menu`

Typed protocol support means the message can be serialized. It does not prove that the app performs a native side effect unless the runtime dispatch path is also source- or runtime-proven.

### Runtime Prompt Messages

Captured Rust prompt-handler-facing variants:

```rust
PromptMessage::SetInput { text }
PromptMessage::ShowHud { text, duration_ms }
PromptMessage::SetStatus { status, message }
PromptMessage::SetActions { actions }
```

Observed backend behavior:

| Prompt message | Runtime behavior |
|---|---|
| `ShowHud` | Clears script-requested hide restore intent if needed, then delegates to HUD manager. |
| `SetStatus` | Logs receipt only in captured source. |
| `SetInput` | Delegates to `set_prompt_input`. |
| `SetActions` | Updates SDK action state/shortcuts and refreshes an open actions dialog. |

### SDK Action Map

The SDK keeps action handler functions in process memory using `globalThis.__kitActionsMap`. Since functions cannot cross the protocol boundary, `setActions()` serializes only metadata and a `hasAction` boolean.

When `actionTriggered` arrives, the SDK looks up the action by name:

- If `onAction` exists, it calls the JS handler.
- If only `value` exists, it sends `forceSubmit` with the value.

Action names should be unique in practice because the map is keyed by name.

## Entry Points

| Entry | SDK payload | Backend truth | Return/receipt |
|---|---|---|---|
| `beep()` | `{ type: "beep", requestId }` | Rust dispatches macOS `afplay /System/Library/Sounds/Tink.aiff`; unsupported platforms return typed unsupported. | `systemFeedbackResult`; delivery not verified. |
| `say(text, voice?)` | `{ type: "say", requestId, text, voice }` | Rust dispatches macOS `say [-v voice] <text>` for non-empty text; unsupported platforms return typed unsupported. | `systemFeedbackResult`; delivery not verified. |
| `notify(string)` | `{ type: "notify", requestId, body }` | Rust normalizes title/body and dispatches `notify-rust` on a dedicated thread. | `systemFeedbackResult`; delivery not verified. |
| `notify({ title, body })` | `{ type: "notify", requestId, title, body }` | Rust normalizes title/body and dispatches `notify-rust`; empty payloads return typed invalid. | `systemFeedbackResult`; delivery not verified. |
| `hud(message, options?)` | `{ type: "hud", text, duration_ms }` | Implemented HUD overlay. | Fire-and-forget; verify visually/source. |
| `setStatus(options)` | None. | SDK returns unsupported before `send(...)`; no visible status surface exists. | `SystemFeedbackResult` with `ERR_UNSUPPORTED_SDK_FEATURE`. |
| `menu(icon, scripts?)` | None. | SDK returns unsupported before `send(...)`; no tray/menu mutation handler exists. | `SystemFeedbackResult` with `ERR_UNSUPPORTED_SDK_FEATURE`. |
| `setActions(actions)` | `{ type: "setActions", actions }` | Implemented for action state, shortcuts, and open dialog refresh. | Fire-and-forget; later `actionTriggered`. |
| `setInput(text)` | `{ type: "setInput", text }` | Implemented for supported active views; batch has target-specific receipts. | Direct call no receipt; batch returns result. |

## User Workflows

### Show HUD Feedback

A script calls:

```ts
hud("Saved", { duration: 2000 })
```

The SDK sends a `hud` message. The prompt handler receives `PromptMessage::ShowHud`, clears a pending script-hide restore intent if present, and calls `show_hud`. The HUD manager allocates a slot, creates a standalone overlay window, schedules dismissal, and queues overflow HUDs when slots are full.

HUD is feedback UI, not prompt UI. It must not reveal the launcher, request main-window show, or use prompt window preparation.

### Add Runtime Actions

A script calls:

```ts
await setActions([
  {
    name: "Copy",
    description: "Copy current value",
    shortcut: "cmd+c",
    onAction: async (input, state) => {
      await copy(input)
      hud("Copied")
    },
  },
  {
    name: "Submit",
    shortcut: "cmd+enter",
    value: "submitted",
  },
])
```

The SDK clears `__kitActionsMap`, stores handlers by name, serializes metadata, and sends `setActions`. Rust receives `PromptMessage::SetActions`, calls `set_sdk_actions_and_shortcuts`, recalculates visible shortcuts, and updates an open actions dialog if present.

### Trigger Handler Action

The user presses a shortcut or selects a row in the actions dialog. Rust sends an action-trigger message back to the SDK. The SDK finds the action by name and awaits `onAction(input, state)`.

The exact full wire payload shape needs follow-up because the supplied Rust and SDK snippets expose slightly different `actionTriggered` fields.

### Trigger Submit-Value Action

If an action has `value` and no handler, the SDK sends `forceSubmit` with the action value. This completes the prompt through the normal submit path.

### Set Active Prompt Input

A script calls:

```ts
setInput("abc")
```

The SDK sends `setInput`. Rust calls `set_prompt_input(text, cx)`. Because the direct call has no receipt, scripts and agents should follow with state verification when correctness matters.

### Set Target Input With Batch

Automation can send `batch` with `setInput` to a resolved target. This is the preferred proof path because it returns command results and can be followed by `waitFor`.

Captured targets include main, detached ACP, Notes, and ActionsDialog. Notes routes to either the notes editor or embedded ACP composer depending on Notes mode. ActionsDialog `setInput` updates search text and then resizes the dialog.

### Dispatch Platform Feedback

Scripts can call `beep()`, `say()`, or `notify()` to request platform feedback. These helpers no longer warn as unsupported because the Rust runtime has dispatch paths, and each resolves from an app-originated dispatch receipt. The receipt proves spawn/thread dispatch only; it does not prove the user heard or saw the feedback.

### Attempt Unsupported Status Or Menu Calls

Scripts can call `setStatus()` or `menu()`, but both helpers return a typed unsupported result before sending. That keeps a missing status UI or tray/menu mutation handler from looking like success.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Show transient feedback. | `hud("Saved")`. | HUD overlay. | SDK call. | SDK `hud` -> `ShowHud` -> HUD manager. | Overlay appears and auto-dismisses. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `src/hud_manager/mod.rs`. |
| Keep launcher hidden during HUD. | Hidden main + `hud`. | HUD only. | SDK call. | `script_requested_hide` cleared. | Main window remains hidden. | `tests/hud_visibility_decoupled_contract.rs`. |
| Register actions. | `setActions([...])`. | Prompt/action state. | SDK call. | SDK map + serialized `setActions`. | Rust stores actions/shortcuts. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`. |
| Refresh open actions dialog. | `setActions` while dialog open. | ActionsDialog. | SDK call. | `d.set_sdk_actions(actions)`. | Open dialog sees new SDK actions. | `src/prompt_handler/mod.rs`. |
| Trigger handler. | Action shortcut/dialog. | Current prompt. | Shortcut/Enter. | `actionTriggered` -> SDK map. | `onAction` runs. | `tests/smoke/test-sdk-actions.ts`. |
| Trigger submit value. | Action with value. | Current prompt. | Shortcut/dialog. | SDK sends `forceSubmit`. | Prompt submits action value. | SDK handler and protocol variants. |
| Set prompt input. | `setInput("abc")`. | Active supported prompt. | SDK call. | `PromptMessage::SetInput` -> `set_prompt_input`. | Input changes if view supports it. | `src/prompt_handler/mod.rs`. |
| Set main input with receipt. | `batch.setInput`. | Main target. | Protocol command. | Main batch path. | Batch result success/failure. | `tests/sdk_automation_runtime/mod.rs`. |
| Filter actions dialog by automation. | `batch.setInput` target ActionsDialog. | Actions dialog search. | Protocol command. | `dialog.set_search_text`, resize. | Rows/filter height update. | `tests/actions_dialog_batch_setinput_resize_parity_contract.rs`. |
| Set Notes editor/composer. | `batch.setInput` target Notes. | Notes editor or embedded ACP. | Protocol command. | Notes mode dispatch. | Correct target text changes. | `lat.md/protocol.md`, `src/prompt_handler/mod.rs`. |
| Request notification. | `notify(...)`. | OS Notification Center. | SDK call. | SDK sends `notify`, Rust dispatches `notify-rust`. | `systemFeedbackResult` dispatch receipt. | `src/execute_script/mod.rs`. |
| Play sound. | `beep()`. | macOS sound. | SDK call. | SDK sends `beep`, Rust dispatches `afplay`. | `systemFeedbackResult` dispatch receipt. | `src/execute_script/mod.rs`. |
| Speak text. | `say(...)`. | macOS speech. | SDK call. | SDK sends `say`, Rust dispatches `say`. | `systemFeedbackResult` dispatch receipt. | `src/execute_script/mod.rs`. |
| Set status. | `setStatus(...)`. | None. | SDK call. | SDK returns unsupported before send. | Unsupported typed result. | `tests/sdk_system_feedback_contract.rs`. |
| Set tray menu. | `menu(...)`. | None. | SDK call. | SDK returns unsupported before send. | Unsupported typed result. | `tests/sdk_system_feedback_contract.rs`. |

## State Machine

### HUD

| State | Trigger | Transition |
|---|---|---|
| Script call. | `hud(text, options)`. | SDK sends `ShowHud` protocol message. |
| Handler receives. | `PromptMessage::ShowHud`. | Clears hide-restore intent if set. |
| HUD requested. | `show_hud`. | HUD manager allocates a slot or queues. |
| Visible. | HUD window created. | Non-focus overlay displays text. |
| Expiry. | Timer / cleanup. | HUD dismisses by id, pending HUD may show. |

### Actions

| State | Trigger | Transition |
|---|---|---|
| Script action list. | `setActions(actions)`. | SDK map cleared and repopulated. |
| Serialized actions. | Functions removed, `hasAction` set. | Protocol `setActions` sent. |
| Rust action state. | `PromptMessage::SetActions`. | Actions/shortcuts updated. |
| Dialog open. | Existing ActionsDialog. | Dialog action list refreshed. |
| User triggers action. | Shortcut/dialog row. | Rust sends `actionTriggered`. |
| SDK handles action. | Map lookup. | Calls handler or sends `forceSubmit`. |

### Input

| State | Trigger | Transition |
|---|---|---|
| Direct script call. | `setInput(text)`. | SDK sends fire-and-forget message. |
| Prompt handler. | `PromptMessage::SetInput`. | Calls `set_prompt_input`. |
| Supported view. | View-specific setter exists. | Input/composer/search changes. |
| Unsupported view. | No setter branch. | Warning/logging path, no guaranteed mutation. |
| Automation batch. | `BatchCommand::SetInput`. | Target-specific path returns batch result. |

## Visual And Focus States

| Surface | Visual/focus contract |
|---|---|
| HUD | Standalone overlay, independent from launcher visibility, auto-dismissed by duration. |
| Actions dialog | `setActions` can update dialog contents; batch `setInput` filters search and must resize. |
| Main prompt input | `setInput` can mutate supported prompt input without native typing. |
| Notes target | Batch input writes to Notes editor or embedded ACP composer based on Notes mode. |
| Detached ACP target | Batch input writes to ACP thread composer; setup mode fails. |
| Stub APIs | No visible native feedback proven. |

## Keystrokes And Commands

| Input | Behavior |
|---|---|
| Action shortcut | Triggers visible registered SDK action when shortcut is registered. |
| Cmd+K | Opens actions dialog, where SDK actions can be selected. |
| ActionsDialog filter typing / batch `setInput` | Updates action search text and resizes popup. |
| `setInput` protocol | Avoids native key delivery and focus issues for supported targets. |
| HUD | No keyboard ownership; it is feedback, not an interactive prompt. |

## Actions And Menus

`setActions()` is the script-facing bridge into prompt actions. It should be treated as separate from root result actions and built-in action coverage.

| Action property | Meaning |
|---|---|
| `name` | Map key and visible label. Duplicate names overwrite in SDK handler map. |
| `description` | Metadata shown in actions surfaces where supported. |
| `shortcut` | Rust registers visible action shortcuts. |
| `onAction` | JS handler kept in SDK memory; serialized as `hasAction`. |
| `value` | Submit value used when no handler exists. |
| `visible` | At minimum affects shortcut registration; full row behavior needs actions-dialog proof. |
| `close` | Serialized, but exact close behavior was not proven in this bundle. |

`menu(icon, scripts?)` is included in this feature only as an explicit unsupported SDK boundary. Real tray menu observation/action behavior belongs to a later tray/menu feature.

## Automation And Protocol Surface

| Surface | Agent strategy |
|---|---|
| Direct `setInput()` | Follow with `getState`/`waitFor`; no direct receipt. |
| Batch `setInput` main target | Use `batch` with `setInput` then `waitFor` expected input. |
| Batch `setInput` ActionsDialog | Verify row count/filter text/geometry after resize. |
| Batch `setInput` Notes | Verify editor value or embedded ACP composer value depending on Notes mode. |
| Batch `setInput` detached ACP | Expect failure when ACP is in setup/no-thread mode. |
| `setActions` | Verify via actions dialog `getElements`, shortcut behavior, or `actionTriggered` logs. |
| HUD | Source contracts and smoke/visual proof; not a normal prompt state. |
| Stub APIs | Verify warning/logging only; do not assert native side effects. |

## Data, Storage, And Privacy Boundaries

| Data | Exposure |
|---|---|
| HUD text | Visible overlay, logs/tests may capture it. |
| Status message | Logged by Rust in captured `SetStatus` arm. |
| Action names/descriptions/shortcuts | Sent to Rust and visible in actions UI. |
| Action values | Can become prompt submit values. |
| Action handlers | Stay in SDK process memory; not serialized. |
| Input text | Written to prompt/composer/search/editor state and may appear in automation receipts. |
| Batch traces | May include command metadata and should be treated as potentially sensitive. |

No persistence is inherent to HUD/actions/control calls, but adjacent targets may persist state after input mutation.

## Error, Empty, Loading, And Disabled States

| Case | Current behavior/risk |
|---|---|
| `setActions([])` | Should clear SDK map and Rust shortcuts/actions. Verify dialog clears. |
| Duplicate action names | SDK map is name-keyed; later action can overwrite handler lookup. |
| Invisible actions | Shortcuts are skipped for invisible actions; full row rendering needs source proof. |
| Action with no handler and no value | Serialized but cannot meaningfully run. |
| Handler error | Full catch/final behavior was not in the bundle. |
| Direct `setInput` before prompt/support | No receipt; unsupported view can log and do nothing. |
| PromptPopup batch input | Bundle showed a capability mismatch; do not assume support. |
| HUD while main hidden | Must remain independent and not restore main window. |
| HUD slots full | HUD manager queues pending HUDs. |
| Stub APIs | Message send is not proof of native behavior. |
| `setStatus` | Logs message text; no visible state proven. |

## Code Ownership

| Owner | Responsibility |
|---|---|
| `prompt-runtime` | SDK utility calls, prompt message arms, direct `setInput`, action bridge. |
| `actions-popups` | ActionsDialog refresh, action selection, dialog filtering/resizing. |
| `protocol-automation` | Batch `setInput`, target identity, transaction results/traces. |
| `agentic-testing` | State-first proof for input/action behavior and visual proof for HUD. |
| `platform-windowing-macos` | Future native notification/speech/beep/menu platform behavior. |
| `theme-config-preferences` | Status/theme/tray-related future visible state if implemented. |

Key files include `scripts/kit-sdk.ts`, protocol variants/constructors, `src/main_sections/prompt_messages.rs`, `src/prompt_handler/mod.rs`, `src/hud_manager/mod.rs`, `src/app_impl/actions_dialog.rs`, `src/actions/`, `src/tray/mod.rs`, and the HUD/actions/batch tests.

## Invariants And Regression Risks

| Invariant | Risk |
|---|---|
| HUD must not reveal main window. | Launcher appears unexpectedly after hidden-script feedback. |
| HUD must not use prompt preparation. | Feedback steals focus or changes prompt lifecycle. |
| Action handlers stay local. | Functions cannot cross protocol; losing map breaks `onAction`. |
| Action names should be unique. | Handler map collisions call the wrong action. |
| Visible shortcuts recalculate on every `setActions`. | Stale shortcuts remain active. |
| Open actions dialog refreshes after `setActions`. | Dialog shows stale action rows. |
| ActionsDialog batch `setInput` resizes after filter. | Popup height gets stuck after filtering. |
| Batch trace remains top-level and opt-in/conditional. | Receipts get too large or leak data unexpectedly. |
| Direct SDK `setInput` is not treated as a receipt. | Agents claim mutation without proof. |
| Stub APIs are not documented as implemented. | Users expect native notification/speech/beep/menu behavior that does not exist. |

## Verification Recipes

| Recipe | Proof |
|---|---|
| HUD visibility decoupling | `tests/hud_visibility_decoupled_contract.rs`; confirm `ShowHud` clears script-hide intent and does not show main. |
| HUD smoke | `tests/smoke/test-hud.ts`, `test-hud-multiple.ts`, `test-hud-auto-dismiss.ts`. |
| SDK action handler | `tests/smoke/test-sdk-actions.ts`; trigger shortcut/dialog action and inspect stderr/behavior. |
| Submit-value action | Register action with `value`, no handler; trigger and verify prompt submits that value. |
| Clear actions | Register actions, call `setActions([])`, verify actions/shortcuts disappear. |
| Direct setInput | Open supported prompt, call `setInput`, follow with `getState`/`waitFor`. |
| Batch main setInput | Send batch `setInput` + `waitFor`; expect `batchResult.success`. |
| ActionsDialog batch setInput | Run/source-check `tests/actions_dialog_batch_setinput_resize_parity_contract.rs`. |
| Notes setInput routing | Verify Notes editor vs embedded ACP mode writes correct target. |
| Detached ACP setup failure | Target detached ACP setup; expect batch failure. |
| Unsupported boundaries | Call `setStatus()` / `menu()`, confirm `ERR_UNSUPPORTED_SDK_FEATURE` and no protocol send. |

## Agent Notes

Prefer `hud()` for in-launcher feedback. Use `notify()` only when OS Notification Center delivery is the intent.

Prefer batch `setInput` for automation. Direct `setInput()` is script-friendly but receiptless.

Always verify receiptless calls with state, elements, logs, or visual proof. Message serialization is not a behavioral receipt; `systemFeedbackResult` proves dispatch only.

Keep HUD separate from prompt runtime. It is not a prompt, not launcher UI, and not an input surface.

Do not treat `PromptPopup` as input-capable without direct proof; the captured bundle shows conflicting signals.

## Related Features

| Feature | Relationship |
|---|---|
| 016 Prompt Runtime Core | Owns initial prompt creation/lifecycle; this feature owns post-creation control calls. |
| 011 Root Unified Search Result Actions | Owns root action behavior; this feature owns script-facing `setActions`. |
| Actions Dialog | Displays/executes actions and owns popup filtering/resizing. |
| Protocol Automation | Owns batch `setInput`, target resolution, and traces. |
| HUD Manager | Owns proven user-visible feedback. |
| Tray/Menu | `menu()` returns an unsupported result here; real tray/menu behavior is separate. |
| Platform macOS | Owns the current beep/say/notify dispatch boundaries. |
| ACP and Notes | Batch `setInput` can write into these targets; they own deeper state/persistence. |

## Open Questions And Gaps

| Gap | Why it matters |
|---|---|
| `beep()` / `say()` / `notify()` delivery proof absent. | Runtime dispatch is proven by `systemFeedbackResult`, but OS delivery remains unverified. |
| `menu()` backend absent. | No tray/menu mutation should be promised; SDK returns unsupported. |
| `setStatus()` visible surface absent. | Visible status UI is not proven; SDK returns unsupported. |
| `ActionTriggered` payload shape mismatch. | Full emitter/adapter needs inspection before exact wire docs. |
| `close` flag behavior not proven. | Serialized flag may not imply exact close behavior. |
| Invisible action row behavior incomplete. | Shortcut skipping is proven; rendering needs source proof. |
| Direct `set_prompt_input` support matrix incomplete. | Need full source/runtime proof before listing every supported prompt. |
| PromptPopup batch input contradictory. | Agentic scripts and Rust capabilities appear out of sync. |
| HUD display placement needs targeted source proof. | Bundle had mixed comments around mouse/main-window display choice. |
| Handler error behavior incomplete. | Need full SDK response-loop catch/final path. |
| Batch trace sensitivity needs schema review. | Tests prove inclusion rules, not full data contents. |
| Action dialog resize after `setActions()` not proven. | Open-dialog refresh is proven; post-list-change resize is not. |
