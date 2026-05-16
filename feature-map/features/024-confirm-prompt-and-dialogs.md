# 024 Confirm Prompt and Dialogs / confirm()



## Executive Summary


- SDK `confirm()` in `scripts/kit-sdk.ts`.
- The attached/native `confirm-popup` fallback route.
- Automation receipts for confirm state and popup buttons.


| Route | When used | Result |
|---|---|---|
| Parent popup fallback | Main window is hidden or the active GPUI context is not the main root. | Attached prompt popup opens and is registered under the prompt-popup automation family, commonly as `confirm-popup`. |


## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Ask for a default confirmation. | `await confirm()` | Shows "Are you sure?" and resolves a boolean. |
| Ask with a message. | `await confirm("Delete this?")` | Shows custom body copy. |
| Customize labels positionally. | `await confirm("Delete?", "Delete", "Keep")` | Uses custom confirm/cancel labels. |
| Customize labels by config. | `await confirm({ message, confirmText, cancelText })` | Same behavior with a config object. |
| Confirm or cancel by keyboard. | Enter, Escape, Tab. | Resolves according to focused button or cancel key. |
| Confirm destructive built-ins. | Quit, Empty Trash, Restart, Shut Down, Log Out, Stop All, Clear Suggested, Test Confirmation. | Built-in runs only after confirmation. |
| Confirm destructive app actions. | Clipboard delete, file trash, script removal, sharing install trust prompt, adjacent notes/chat actions. | Caller action runs only after confirmation. |
| Drive with automation. | `getState`, `getElements`, `listAutomationWindows`, `batch`. | Agents can inspect state and choose confirm/cancel deterministically. |

## Core Concepts

### SDK `confirm()`


```ts
```

`ConfirmConfig` carries `message`, `confirmText`, and `cancelText`. With no argument, the SDK uses `Are you sure?`. Missing confirm/cancel labels are omitted from the protocol message and default on the Rust side to `OK` and `Cancel`.


### Protocol Confirm Message


```json
{
}
```


### `ParentConfirmOptions`



| Field | Value |
|---|---|
| Title | `Confirm` |
| Body | SDK `message` |
| Confirm label | SDK `confirmText` or `OK` |
| Cancel label | SDK `cancelText` or `Cancel` |

### ConfirmPrompt Surface


```rust
}
```

`ConfirmFocusedButton` defaults to Confirm and toggles between Confirm and Cancel. The surface is explicit/no-editable-input feedback UI with automation semantic surface `confirmPrompt` and native footer surface `confirm_prompt`.

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `open_confirm_prompt` | Main window can host confirm. | Captures previous view, switches to `ConfirmPrompt`, focuses app root. |
| `open_parent_confirm_dialog` | Main cannot host confirm. | Opens attached/native popup fallback. |
| `open_parent_confirm_dialog_for_entity` | Entity-owned action flows. | Opens parent-owned popup with callbacks. |

## User Workflows

### SDK Default Confirm

A script calls `await confirm()`. The SDK sends a confirm message with body `Are you sure?`, Rust opens the in-window state or popup fallback, and the Promise resolves to `true` only when the confirm path is selected.

### SDK Custom Labels


```ts
const yes = await confirm("Delete?", "Delete", "Keep")
```

The SDK forwards `confirmText` and `cancelText`. Rust shows title `Confirm`, body `Delete?`, confirm label `Delete`, cancel label `Keep`, then submits `"true"` or `"false"`.

### In-Window Confirm


### Popup Fallback Confirm

When the main window is hidden or the current GPUI context is not the main root, `confirm_with_parent_dialog` falls back to the parent/native popup. The popup is attached when possible, appears as a prompt-popup automation surface, and exposes confirm/cancel semantic elements.

### Destructive Built-Ins

Confirmation-gated built-ins build `ParentConfirmOptions`, wait for `confirm_with_parent_dialog`, and run the built-in only on `Ok(true)`. Cancel logs a cancelled dispatch outcome; open failure shows/logs an error instead of proceeding.

Examples include Quit Script Kit, Empty Trash, Restart, Shut Down, Log Out, Sleep, Force Quit Apps, Stop All Processes, Clear Suggested, Sync to GitHub, and Test Confirmation.

### Destructive Action Callers

| Caller | Confirm copy/result |
|---|---|
| Clipboard delete matching | Confirms matching count, label `Delete`; cancel leaves entries untouched. |
| Clipboard delete all | Confirms unpinned count, label `Delete All`; cancel leaves entries untouched. |
| File move to trash | Confirms selected file name, label `Move to Trash`; cancel clears action target and restores focus. |
| Script removal | Verifies path still exists, asks `Move to Trash`, then moves only on confirm. |
| Sharing trust prompt | Shows shared item kind, title, plugin label, file count; Install proceeds, Ignore cancels. |

Notes and ACP/chat flows are adjacent callers. The bundle confirms the fallback route exists for non-main contexts, but exact labels/callbacks need targeted expansion before treating them as fully mapped.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Ask default confirm. | `confirm()` | In-window or popup. | Confirm/cancel. | SDK -> `ShowConfirm`. | Boolean Promise. | `scripts/kit-sdk.ts`. |
| Ask with labels. | `confirm("Delete?", "Delete", "Keep")` | In-window or popup. | Confirm/cancel. | Positional labels -> Rust options. | Boolean Promise. | SDK/protocol variant. |
| Ask with config. | `confirm({ message, confirmText, cancelText })` | In-window or popup. | Confirm/cancel. | `ConfirmConfig` -> message. | Boolean Promise. | `ConfirmConfig`. |
| Toggle focus. | In-window confirm. | Native footer selected flag. | Tab. | `toggle_confirm_prompt_focus`. | Confirm/Cancel focus flips. | `ConfirmFocusedButton`. |
| Confirm built-in. | Confirmation-gated built-in. | In-window or popup. | Confirm. | Built-in confirmation gate. | Built-in executes. | `builtin_execution.rs`. |
| Cancel built-in. | Confirmation-gated built-in. | In-window or popup. | Cancel/Escape. | Confirmation gate returns false. | Built-in does not execute. | `builtin_execution.rs`. |
| Move file to trash. | File action. | In-window or popup. | Confirm. | `move_to_trash` action. | File moves to Trash. | `files.rs`. |
| Cancel file trash. | File action. | In-window or popup. | Cancel/Escape. | Cancel branch. | Target cleared, focus restored. | `files.rs`. |
| Inspect popup. | Automation. | PromptPopup target. | `getElements`. | Confirm popup snapshot. | Panel + two buttons. | `automation_surface_collector.rs`. |
| Select popup cancel. | Automation batch. | PromptPopup target. | `selectByValue("cancel")`. | Confirm batch helper. | Cancels. | `prompt_handler/mod.rs`. |
| Hide while popup open. | Hide/reset path. | Main hidden. | Hide. | Remove `confirm-popup`. | No stale registry entry. | `hide_path_confirm_popup_registry_teardown_contract.rs`. |

## State Machine

### SDK Confirm

| State | Trigger | Transition |
|---|---|---|
| Script call. | `confirm(...)`. | SDK creates id and pending resolver. |
| Route decision. | Main/root availability. | In-window `ConfirmPrompt` or popup fallback. |
| User choice. | Confirm/cancel/open failure. | Rust sends submit `"true"` or `"false"`. |
| Promise resolution. | SDK pending handler. | Resolves `true` only for `"true"`. |

### In-Window Confirm

| State | Trigger | Transition |
|---|---|---|
| Previous view. | `open_confirm_prompt`. | Stored in `previous`. |
| Confirm visible. | App view transition. | `focused_button` starts at Confirm. |
| Focus change. | Tab. | Toggles Confirm/Cancel. |
| Confirm. | Enter on Confirm or footer Apply. | Sends true. |
| Cancel. | Escape, Enter on Cancel, footer Close. | Sends false. |
| Close. | Resolution. | Restores previous view. |

### Popup Fallback

| State | Trigger | Transition |
|---|---|---|
| Main cannot host. | Hidden main or non-root context. | Parent popup opens. |
| Popup visible. | Attached window registered. | Automation sees prompt popup / `confirm-popup`. |
| Confirm. | Button/key/batch. | Callback true. |
| Cancel. | Cancel/Escape/close/batch. | Callback false. |
| Close. | Popup dismissed. | Registry entry removed. |

## Visual And Focus States

| State | Visible result | Focus/automation signal |
|---|---|---|
| In-window body. | Main content area shows title and body. | No editable input. |
| Confirm focused. | Confirm footer button selected. | Enter resolves true. |
| Cancel focused. | Cancel footer button selected. | Enter resolves false. |
| Popup destructive. | Destructive verb is error-colored. | Danger lives on label, not keycap glyph. |
| Popup focused keycap. | Focused keycap uses accent selected styling. | `focused_semantic_id` points at confirm/cancel button. |

Storybook documents `confirm-popup-states` as canonical state coverage, including the live in-window state plus destructive popup treatments.

## Keystrokes And Commands

| Input | In-window behavior | Popup behavior | Notes |
|---|---|---|---|
| Enter | Resolves according to focused button. | Implied by design/smoke coverage, exact popup handler needs expansion. | Stdin simulateKey parity for in-window confirm was not proven by this bundle. |
| Escape | Resolves false. | `route_key_to_confirm_popup("escape", cx)` is used from popup-aware paths. | Cancel/fail-closed. |
| Tab | Toggles focused button. | Implied by smoke coverage, exact popup handler needs expansion. | Stops propagation in in-window route. |
| Space | Not proven. | Not proven. | Smoke copy mentions Space, but included source did not prove handler support. |
| Arrow keys | Not proven. | Not proven. | Treat as open until source-expanded. |
| Footer Apply | Resolves true. | N/A. | Native footer only for in-window route. |
| Footer Close | Resolves false. | N/A. | Native footer only for in-window route. |

## Actions And Menus

Confirm is a gate, not the owner of destructive side effects. The caller owns the actual operation after a true result.

| Domain | Confirm responsibility | Adjacent owner responsibility |
|---|---|---|
| Built-ins | Ask before a confirmation-gated system command. | Execute Quit/Restart/etc. |
| Clipboard | Ask before delete matching/delete all. | Matching, deletion, pinning/cache refresh. |
| Files | Ask before moving selected file to Trash. | File resolution, trash operation, focus restoration. |
| Scripts | Ask before moving script/scriptlet path to Trash. | Path checks, refresh, HUD/error state. |
| Sharing | Ask before installing shared URI content. | URI parsing, validation, install, refresh. |
| Notes/ACP/chat | Provide popup fallback for non-main contexts. | Domain-specific labels, delete/rename/session behavior. |

## Automation And Protocol Surface

### `getState`


| Field | Expected shape |
|---|---|
| `promptType` | `confirmPrompt` |
| `inputValue` / filter | Empty string. |
| `choiceCount` | `0`. |
| `visibleChoiceCount` | `0`. |
| Selected index | `-1`. |
| Selected value | The confirm options title in the captured state path. |

### `getElements`


| Semantic id | Type | Value |
|---|---|---|


### `batch`

PromptPopup batch selection tries mention/model-selector helpers before confirm helpers, because `PromptPopup` is a union family. Prefer exact target identity when available.

| Batch command | Result |
|---|---|
| `selectByValue("confirm")` | Confirm. |
| `selectByValue("cancel")` | Cancel. |

### `listAutomationWindows`

The confirm popup fallback registers as an attached prompt popup, commonly `confirm-popup`. Hide/reset paths must remove stale `confirm-popup` entries after closing actions-dialog siblings.

## Data, Storage, And Privacy Boundaries


- SDK prompt message.
- Title/body.
- Confirm/cancel labels.
- Destructive action labels.
- Action-specific names/counts, such as file names or clipboard counts.

Confirm itself does not persist decisions. The caller may persist, delete, install, or execute after a true result. Runtime logs and automation receipts may include confirm text; screenshots expose the visible text. Do not place secrets in confirm messages or labels when logs/receipts may be collected.

## Error, Empty, Loading, And Disabled States

| Case | Behavior |
|---|---|
| User cancellation | Resolves/behaves as false. |
| Escape | Resolves false. |
| SDK `null` submit | Resolves false. |
| Dialog-open failure in SDK prompt path | Logs error and sends false. |
| Dialog-open failure in built-in/action paths | Shows/logs failure and does not proceed. |
| Missing confirm label | Defaults to `OK`. |
| Missing cancel label | Defaults to `Cancel`. |
| Empty message | SDK does not prove non-empty validation; empty body would be sent. Layout needs proof. |
| Long message | Wrapping/truncation needs visual proof. |
| Sender/channel closed | Does not imply confirmation; no retry path was visible. |
| Main hidden/wrong root | Uses popup fallback. |
| App hides while popup open | Must remove `confirm-popup` registry entry. |
| Detached popup open | Main footer buttons are blocked while confirm popup is open. |

## Code Ownership

| Owner | Responsibility |
|---|---|
| `actions-popups` | Confirm popup, parent popup routing, popup registry, attached popup behavior. |
| `keyboard-focus-routing` | Enter/Escape/Tab handling, popup-first routing, propagation. |
| `protocol-automation` | `getState`, `getElements`, `batch`, automation target identity. |
| `agentic-testing` | State-first proof and visual proof only when needed. |

Key files include `scripts/kit-sdk.ts`, `src/protocol/message/variants/prompts_media.rs`, `src/prompt_handler/mod.rs`, `src/confirm/parent_dialog.rs`, `src/confirm/window.rs`, `src/app_impl/about_route.rs`, `src/main_sections/app_view_state.rs`, `src/app_impl/ui_window.rs`, `src/windows/automation_surface_collector.rs`, `src/windows/automation_registry.rs`, built-in/action caller files, and `src/stories/popup_component_states.rs`.

## Invariants And Regression Risks

| Invariant | Risk if broken |
|---|---|
| `confirm()` fails closed. | Cancel/open failure could accidentally run destructive behavior. |
| Auto-submit fallback remains `"false"`. | Script cancellation could resolve true. |
| Previous view restores after resolution. | App can strand users in ConfirmPrompt. |
| Confirm keys stop propagation. | Enter/Escape/Tab can leak to launcher, ACP, or actions. |
| Popup fallback unregisters. | Automation can see phantom `confirm-popup` windows. |
| Destructive semantics stay on labels. | Visual danger signal can become inconsistent. |
| Semantic ids stay stable. | Agentic tests and automation recipes break. |
| Built-ins execute only after `Ok(true)`. | Destructive commands can bypass confirmation. |
| Response channel failure never means true. | Disconnected scripts can accidentally confirm. |

## Verification Recipes

| Recipe | Command/proof |
|---|---|
| SDK source check | Inspect `ConfirmConfig`, `ConfirmMessage`, default message, fallback `"false"`, `null` handling in `scripts/kit-sdk.ts`. |
| Protocol source check | Inspect `serde(rename = "confirm")`, `confirmText`, `cancelText` in `src/protocol/message/variants/prompts_media.rs`. |
| Popup automation check | Open popup fallback, run `listAutomationWindows`, then `getElements`, expect panel and two buttons. |
| Batch selection check | Use `selectBySemanticId` or `selectByValue` for confirm/cancel. |
| Smoke visual/focus check | Use `tests/smoke/test-confirm-screenshot.ts`, `test-confirm-focus.ts`, `test-confirm-tab.ts` when visual/focus behavior matters. |
| Hide teardown check | Verify no stale `confirm-popup`; static coverage in `tests/hide_path_confirm_popup_registry_teardown_contract.rs`. |
| Storybook check | Review `confirm-popup-states` and destructive variants. |

## Agent Notes

Treat confirm as a routing and decision contract. Do not treat it as the owner of destructive side effects.


Do not assume `PromptPopup` means confirm. It can also refer to ACP mention/model/history popups. Prefer exact popup ids or semantic ids when driving confirm automation.

## Related Features

| Feature | Relationship |
|---|---|
| 016 Prompt Runtime Core | Owns general prompt message lifecycle. |
| Actions Popups | Owns attached popup mechanics. |
| Keyboard Focus Routing | Owns Enter/Escape/Tab propagation and focus restoration. |
| Launcher Surface Contracts | Owns AppView/SurfaceKind/native footer contracts. |
| Protocol Automation | Owns state/elements/batch target resolution. |
| Built-ins/System Actions | Owns actual system command execution after confirmation. |
| Clipboard/File/Script/Notes/ACP/Sharing | Own domain-specific destructive actions and post-confirm effects. |
| Storybook | Owns visual state catalog and presenter fixtures. |

## Open Questions And Gaps

| Gap | Why it matters |
|---|---|
| Full `src/confirm/parent_dialog.rs` and `src/confirm/window.rs` were filtered by packx context windows. | Exact popup open/render/key/callback internals need a targeted follow-up. |
| Space-key support is not proven. | Smoke text mentions it, but bundled source did not prove a handler. |
| Arrow-key behavior is not proven. | Do not document arrow parity without source/runtime proof. |
| Stdin `simulateKey` parity for in-window ConfirmPrompt is not proven. | Automation should prefer state/batch receipts until verified. |
| Notes and ACP/chat concrete callers need expansion. | Adjacent labels/actions were referenced but not fully source-mapped. |
| Long/empty message layout needs visual proof. | Wrapping/truncation behavior should be screenshot/state verified before claiming. |
| Confirm-close automation re-keying after previous-view restore should be verified live. | Prevents stale `confirmPrompt` semantic surface after resolve. |
