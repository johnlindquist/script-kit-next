# 022 Hotkey Prompt / hotkey()

This chapter maps the SDK-visible `hotkey()` capture prompt and separates it from the implemented shortcut recorder/config system.

Raw Oracle reference: [answer](../raw-oracle/022-hotkey-prompt/answer.md), [prompt](../raw-oracle/022-hotkey-prompt/prompt.md), [bundle map](../raw-oracle/022-hotkey-prompt/bundle-map.md), [full log](../raw-oracle/022-hotkey-prompt/output.log), [session metadata](../raw-oracle/022-hotkey-prompt/session.json).

## Executive Summary

Feature 022 has two related but separate systems:

| System | Status | Meaning |
|---|---|---|
| SDK `hotkey(placeholder?)` | Implemented transient capture. | Exposed in TypeScript, opens `HotkeyPrompt`, and returns `HotkeyInfo` without writing config or registering shortcuts. |
| Shortcut recorder | Implemented. | Persistent app shortcut assignment UI used from action menus; writes `config.ts`, updates live hotkey registration, and refreshes visible scripts. |

Current product truth: SDK `hotkey()` sends a `type:"hotkey"` prompt message, Rust opens `AppView::HotkeyPrompt`, and capture resolves through prompt submission with a JSON `HotkeyInfo` payload. The implemented shortcut recorder remains a separate app feature that mutates `config.ts` and live hotkey registrations.

The implemented shortcut recorder is a separate app feature. It is a compact popup/modal for assigning/removing persistent shortcuts. It mutates `~/.scriptkit/config.ts` via `update-config-shortcut.ts` / `remove-config-shortcut.ts`, registers command hotkeys live through `src/hotkeys/mod.rs`, and refreshes script/scriptlet data.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Call SDK `hotkey()`. | `await hotkey()` or `await hotkey("Press shortcut")`. | GPUI opens transient capture and resolves `HotkeyInfo`. |
| Receive SDK autosubmit shape in tests. | SDK test harness with autosubmit enabled. | Object has `HotkeyInfo` fields, but runtime capture remains the proof of product behavior. |
| Assign command shortcut. | Action menu shortcut action. | Opens shortcut recorder, saves to config, registers live hotkey. |
| Remove command shortcut. | `remove_shortcut` action. | Removes config shortcut, refreshes script data, shows feedback. |
| Capture shortcut in recorder. | Modifier + non-modifier key. | Recorder accepts a valid shortcut. |
| Cancel recorder. | Escape, Cmd+W, backdrop/margin/focus-loss dismissal. | Recorder closes and returns focus. |
| Use registered shortcuts. | Global hotkey. | Config-backed command hotkeys run matching command. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `HotkeyInfo` | SDK return shape. | `key`, `command`, `shift`, `option`, `control`, `shortcut`, `keyCode`. |
| SDK `hotkey()` | Transient capture API. | Captures one shortcut and returns `HotkeyInfo`; should not persist or register anything. |
| `HotkeyPrompt` | Rust host route. | Displays transient capture and submits `HotkeyInfo` or null on cancel. |
| Shortcut recorder | Implemented persistent shortcut UI. | Captures modifier-based shortcuts for commands. |
| `HotkeyConfig` | Persistent config shape. | `modifiers: KeyModifier[]`, `key: KeyCode`. |
| `config.ts` shortcuts | Source of truth for command shortcuts. | Replaces legacy `shortcuts.json`. |
| Global hotkey registry | Live OS shortcut routing. | Config-backed command shortcuts register before inline metadata shortcuts. |
| Transactional rebind | Hotkey update safety. | Register new hotkey before unregistering old one. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `globalThis.hotkey` in `scripts/kit-sdk.ts`. | Script calls SDK hotkey. | Sends `type:"hotkey"` and resolves submitted `HotkeyInfo`. |
| `PromptMessage::ShowHotkey`. | Rust receives hotkey request. | Opens `HotkeyPrompt` with transient capture state. |
| Shortcut action aliases. | Actions menu rows. | `configure_shortcut`, `add_shortcut`, `update_shortcut` open recorder. |
| `remove_shortcut`. | Actions menu row. | Removes config-backed command shortcut. |
| `show_shortcut_recorder`. | App implementation. | Opens detached native recorder popup. |
| `update-config-shortcut.ts`. | Save shortcut. | Writes command shortcut into config. |
| `remove-config-shortcut.ts`. | Remove shortcut. | Deletes command shortcut from config. |
| `hotkeys::update_script_hotkey`. | Live update. | Registers new shortcut and handles old binding. |
| `refresh_scriptlets`. | Scriptlet metadata changes. | Registers, updates, or unregisters scriptlet shortcuts on refresh. |

## User Workflows

### SDK hotkey() transient capture

A script calls:

```ts
const shortcut = await hotkey("Press a keyboard shortcut")
```

The SDK sends `type:"hotkey"` with an optional placeholder. Rust handles the request as `ShowHotkey`, opens `HotkeyPrompt`, and resolves the pending SDK call with `HotkeyInfo` JSON when a modifier chord is captured. Escape and Cmd+W submit null so the SDK follows its cancellation path.

### Assign Shortcut From Actions

The user selects a launcher command row, opens actions with Cmd+K, and chooses a shortcut action. The action handler resolves a stable launcher command id, clears action popup state, opens the shortcut recorder popup, and waits for capture. The user presses a modifier plus non-modifier key, then saves. The app writes `config.ts`, calls live hotkey update, refreshes visible script state, and shows HUD feedback.

### Remove Shortcut

The user opens actions for a command with a shortcut and chooses Remove Shortcut. The action path calls the config removal script, removes the live app route before best-effort OS unregister, refreshes script data, and shows success or error feedback.

### Startup Registration

On startup, global hotkey registration loads config-backed command shortcuts before inline script/scriptlet metadata shortcuts. If config defines a command shortcut, it wins over inline metadata for that command id.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Call SDK hotkey. | `hotkey()`. | HotkeyPrompt. | SDK call. | SDK message -> `ShowHotkey`. | Captured `HotkeyInfo`. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `tests/hotkey_prompt_contract.rs`. |
| Call with placeholder. | `hotkey("Press...")`. | HotkeyPrompt. | SDK call. | Placeholder labels capture surface. | Captured `HotkeyInfo`. | `tests/hotkey_prompt_contract.rs`, runtime proof. |
| Open recorder. | Shortcut action. | Detached shortcut recorder popup. | Cmd+K -> action row. | Action handler -> `show_shortcut_recorder`. | Recorder visible. | `src/app_impl/shortcut_recorder.rs`, source audits. |
| Capture shortcut. | Recorder popup. | Recording active. | Modifier + key. | `ShortcutRecorder` component. | Recorded shortcut. | `src/components/shortcut_recorder/*`. |
| Save shortcut. | Recorder with value. | Popup active. | Save. | `write_config_command_shortcut` -> `update-config-shortcut.ts` -> `update_script_hotkey`. | Config and live registry update. | `src/app_impl/shortcut_recorder.rs`, scripts. |
| Cancel recorder. | Recorder popup. | Popup active. | Escape/Cmd+W/backdrop. | Cancel callback/teardown. | Popup closes, focus returns. | `lat.md/design.md`, popup contract tests. |
| Remove shortcut. | Action menu. | Shortcut exists. | Remove action. | `remove_config_command_shortcut` -> refresh. | Config shortcut removed. | source audits, scripts. |
| Refresh metadata shortcut. | Scriptlet refresh. | App running. | File change/refresh. | `refresh_scriptlets` register/update/unregister paths. | Live shortcut registry changes. | `src/app_impl/refresh_scriptlets.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK hotkey call. | `hotkey()`. | HotkeyPrompt. | Host prompt opens. |
| Capture complete. | Modifier + key. | Submit `HotkeyInfo`. | No config or live registration mutation. |
| Actions popup. | Cmd+K on command row. | User chooses shortcut action. | Popup-first routing matters. |
| Recorder open. | `show_shortcut_recorder`. | Detached popup registered. | Main window should not blur-dismiss. |
| Capturing. | User presses keys. | Modifier state updates until modifier+non-modifier captured. | Bare keys rejected. |
| Saved. | User confirms. | Write config, update live hotkey, refresh. | Config write and live register are not fully atomic. |
| Cancelled. | Escape/Cmd+W/backdrop. | Teardown recorder, restore focus. | No config mutation. |
| Removed. | Remove action. | Config mutation, route removal, and refresh. | OS unregister failures are recoverable after app route removal. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| SDK hotkey prompt. | Compact capture surface. | HotkeyPrompt recorder focus. | `getState.promptType:"hotkey"`, `getElements` capture rows. |
| Shortcut recorder popup. | Compact modal/popup capture UI. | Detached recorder popup. | `listAutomationWindows` / popup registration. |
| Recording active. | Shortcut capture state visible. | Recorder component. | Physical key capture preferred for proof. |
| Recorder conflict/error. | Conflict/error copy when a live route already owns the shortcut. | Recorder. | Conflict checker is wired for app-owned routes; OS/global reservations surface at save-time registration. |
| Recorder cancelled. | Popup dismissed. | Main filter/focus restored. | Popup gone. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Modifier + non-modifier key. | Shortcut recorder. | Captures shortcut. |
| Bare key. | Shortcut recorder. | Should not finish recording. |
| Escape. | Shortcut recorder. | Cancels. |
| Cmd+W. | Shortcut recorder. | Cancels. |
| Backdrop/margin click. | Shortcut recorder. | Dismisses. |
| Cmd+K. | Main/actionable row. | Opens actions to reach shortcut actions. |
| SDK hotkey capture. | `hotkey()`. | Captures a modifier chord and resolves transient `HotkeyInfo`. |

## Actions And Menus

| Action | Behavior |
|---|---|
| `configure_shortcut`. | Opens shortcut recorder for supported command row. |
| `add_shortcut`. | Alias/path to open recorder. |
| `update_shortcut`. | Alias/path to open recorder with existing shortcut. |
| `remove_shortcut`. | Removes config-backed command shortcut. |

Action identity should use `SearchResult::launcher_command_id()`, not labels or display names. Unsupported row types should fail with clear user feedback.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| SDK hotkey. | Prove HotkeyPrompt state/elements, capture result, and no config fingerprint mutation. |
| `listAutomationWindows`. | Shortcut recorder popup is visible/registered. |
| `getConfigFingerprint`. | Config changes after save/remove. |
| `simulateKey`. | Fire-and-forget; follow with state/window inspection. |
| Recorder capture. | May require physical key/event injection; simulateKey popup targeting is not proven. |
| Menu shortcut agentic script. | Proves adjacent launcher shortcut filters, not SDK `hotkey()`. |

## Data, Storage, And Privacy Boundaries

- SDK `hotkey()` returns transient `HotkeyInfo`; it should not persist config or register global hotkeys.
- Shortcut recorder mutates `config.ts`, which is durable user configuration.
- Global hotkey registration affects OS-level shortcuts.
- Shortcut strings and command ids can reveal user workflow preferences.
- Config mutation scripts should be treated as source-of-truth operations and verified through config fingerprint/diff.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| SDK hotkey cancelled. | Escape/Cmd+W submits null and follows the SDK cancellation path. |
| Bare key in recorder. | Should not complete capture. |
| Escape/Cmd+W. | Cancels recorder. |
| Unsupported row. | Shortcut action should show clear failure. |
| Duplicate/conflict shortcut. | The normal recorder path blocks conflicts from live Script Kit routes; OS/global conflicts outside that table surface at live registration. |
| Reserved shortcut. | Global registration can fail; user-facing message needs proof. |
| Config write failure. | Should show error and avoid claiming active shortcut. |
| Live registration failure after config write. | Risk: config may already be mutated; rollback policy not proven. |
| Remove shortcut. | Config removal and immediate app route removal are source-audited; OS unregister failures remain recoverable. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK hotkey. | `scripts/kit-sdk.ts` owns `hotkey()` and `HotkeyInfo`. |
| Host prompt. | `src/prompt_handler/mod.rs`, `src/main_sections/prompt_messages.rs`, protocol constructor. |
| Shortcut recorder component. | `src/components/shortcut_recorder/*`. |
| Recorder app integration. | `src/app_impl/shortcut_recorder.rs`. |
| Config mutation scripts. | `scripts/update-config-shortcut.ts`, `scripts/remove-config-shortcut.ts`. |
| Action handlers. | `src/app_actions/handle_action/shortcuts.rs`, action alias audits. |
| Global hotkeys. | `src/hotkeys/mod.rs`, `src/shortcuts/*`. |
| Scriptlet refresh. | `src/app_impl/refresh_scriptlets.rs`. |
| Automation proof. | `runtime_stdin_match_simulate_key.rs`, `listAutomationWindows`, `getConfigFingerprint`, agentic menu shortcut script. |

## Invariants And Regression Risks

- Do not claim SDK `hotkey()` works until a real GPUI prompt exists.
- Do not use shortcut recorder as proof of SDK `hotkey()`.
- SDK `hotkey()` should remain transient and not mutate config/register global hotkeys.
- Shortcut assignment must persist in `config.ts`, not legacy `shortcuts.json`.
- Config command shortcuts override inline metadata shortcuts.
- Register new hotkey before unregistering old one.
- Recorder capture requires modifier plus non-modifier key.
- Escape and Cmd+W remain cancel controls.
- Opening recorder must not dismiss the main window.
- simulateKey remains fire-and-forget; proof requires follow-up state/window reads.
- Config save plus live registration is not proven atomic.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| SDK HotkeyPrompt source. | `cargo test --test hotkey_prompt_contract -- --nocapture`. |
| SDK test. | `tests/sdk/test-hotkey.ts` proves shape only, not real capture. |
| Popup contract. | `shortcut_recorder_popup_window_contract` protects detached popup, cancel keys, automation visibility. |
| Config source. | `shortcut_config_source` proves config-backed source of truth. |
| Action alias. | `action_shortcut_alias` proves aliases open recorder/remove shortcut. |
| Runtime save/remove. | Open recorder, capture shortcut, save, verify `getConfigFingerprint`, visible shortcut, live registration, remove, refresh. |
| Scriptlet refresh. | Verify register/update/unregister paths on metadata changes. |
| Global priority. | Config-backed command shortcuts register before inline metadata. |
| SDK HotkeyPrompt runtime. | `bun scripts/agentic/hotkey-prompt-transient.ts` proves state/elements, capture JSON, cancellation, and unchanged config fingerprint. |

## Agent Notes

Treat this as two tracks: SDK `hotkey()` is transient capture; shortcut recorder/config registration is persistent assignment.

For SDK `hotkey()` work, do not mutate `config.ts` and do not register global hotkeys. It should return transient `HotkeyInfo`.

For shortcut assignment work, keep `config.ts` as source of truth and verify with config fingerprint/diff plus visible UI.

Recorder capture proof may need physical key/event injection. Do not assume `simulateKey` can target the detached popup.

Do not treat `tests/sdk/test-hotkey.ts` as proof of real GPUI capture.

## Related Features

| Feature | Relationship |
|---|---|
| [001 Main Menu](./001-main-menu.md). | Main shortcut assignment is covered there; this chapter deepens recorder/config/hotkey boundaries. |
| Actions Popups. | Shortcut assignment is action-menu driven. |
| Theme Config Preferences. | `config.ts` owns persistent hotkeys and command shortcuts. |
| Keyboard Focus Routing. | Popup-first keys and cancel controls are critical. |
| Protocol Automation. | `simulateKey`, automation windows, and config fingerprint are proof tools. |
| Menu syntax shortcut filters. | Adjacent launcher filtering, not SDK `hotkey()`. |

## Open Questions And Gaps

- SDK fallback timing is not fully visible in the excerpt.
- Exact hotkey message conversion path is partly omitted.
- Conflict enforcement now covers live app routes for config commands, scripts, scriptlets, and top-level app hotkeys; reserved OS/global shortcuts outside the route table remain a save-time registration policy.
- Reserved shortcut UX is explicit policy: the recorder allows capture, then save reports saved-not-active if OS registration rejects it after config write.
- Remove shortcut immediate live-unregister is source-audited for config-backed dynamic routes; absent live routes are no-ops and app routes are removed before best-effort OS unregister.
- Config write plus live registration is not atomic in visible code.
- Detached recorder popup key-capture automation is under-specified.
- Inline recorder overlay path may be legacy or unreachable; detached popup appears current.
