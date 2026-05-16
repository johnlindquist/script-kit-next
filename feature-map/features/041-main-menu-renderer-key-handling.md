# 041 Main Menu Renderer Key Handling

This chapter maps the key-routing ladder inside the main ScriptList renderer and its protocol mirror.

Raw Oracle reference: [answer](../raw-oracle/041-main-menu-renderer-key-handling/answer.md), [prompt](../raw-oracle/041-main-menu-renderer-key-handling/prompt.md), [bundle map](../raw-oracle/041-main-menu-renderer-key-handling/bundle-map.md), [full log](../raw-oracle/041-main-menu-renderer-key-handling/output.log), [session metadata](../raw-oracle/041-main-menu-renderer-key-handling/session.json).

## Executive Summary

Main Menu Renderer Key Handling owns the dispatch order for physical GPUI keys while `AppView::ScriptList` is active and the stdin `simulateKey` mirror that agents use to prove behavior.

This slice fills the renderer gap left by the broad main-menu chapter: exact `Cmd+Enter`, non-file `Tab`, action shortcut execution, popup-first routing, fallback execution, and stale-selection avoidance. It does not own result ranking, action catalog contents, shortcut persistence, ACP internals, File Search, or Quick Terminal after a route handoff.

## What Users Can Do

- Press `Cmd+K` from ScriptList to open the actions dialog for the current visible row.
- Press `Cmd+K` again while actions are open to close the actions dialog instead of immediately reopening it.
- Use Up, Down, Enter, Escape, Backspace, printable characters, and action-row shortcuts while actions are open without leaking those keys to the parent launcher.
- Press `Cmd+Enter` outside actions to route the current launcher context into Agent Chat context capture when eligible.
- Press Enter outside popups to execute the visible selected ScriptList row.
- Press Enter while fallback rows are active to execute the selected fallback row.
- Press Escape to close menu-syntax trigger popups before clearing filters or closing the launcher.
- Use automation `simulateKey` to drive the same popup-first actions route, then prove the postcondition with `getState` or `getElements`.

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Renderer key listener | `handle_key` listener inside `src/render_script_list/mod.rs`. | Owns physical key dispatch while ScriptList is active. |
| Shared actions router | `route_key_to_actions_dialog(...)`. | Popup-owned keys are handled before generic launcher Cmd shortcuts. |
| `ActionsRoute` | Shared router result. | `Execute` delegates to host execution, `Handled` stops parent handling, `NotHandled` continues. |
| Generic Cmd block | Renderer branch after actions routing. | Handles launcher-level shortcuts such as `Cmd+K`, `Cmd+Shift+K`, logs, design cycle, and quit paths. |
| Fallback state | `main_menu_fallback_state`. | When active, fallback Up/Down/Enter/Escape run before normal selected-row execution. |
| Menu-syntax proposal | `pending_menu_syntax_ai_proposal`. | Tab or plain Enter accepts; Escape dismisses before final Enter/Escape routing. |
| SDK action shortcuts | `action_shortcuts`. | Run before shared actions routing in the visible physical renderer slice; this is a sharp edge. |
| Protocol mirror | `ExternalCommand::SimulateKey`. | Routes actions popup first, then falls through to current-view simulation. |

## Entry Points

| Entry point | Source | Scope |
|---|---|---|
| Physical keydown | `src/render_script_list/mod.rs` `handle_key`. | Main ScriptList renderer. |
| Actions dialog key route | `src/app_impl/actions_dialog.rs#route_key_to_actions_dialog`. | Shared actions hosts, including MainList. |
| MainList action toggle | `src/app_impl/actions_toggle.rs#handle_cmd_k_actions_toggle`. | `Cmd+K` and footer Actions. |
| Current-view action dispatch | `dispatch_actions_toggle_for_current_view(...)`. | Resolves root unified subject before legacy script actions. |
| Protocol simulateKey | `src/main_entry/runtime_stdin_match_simulate_key.rs`. | Automation parity path. |
| Root action execution | `execute_actions_route_action(...)`. | Runs captured host/action ids. |
| ACP context capture | `try_route_global_cmd_enter_to_acp_context_capture(...)`. | `Cmd+Enter` outside actions. |

## User Workflows

### Open And Close Actions With Cmd+K

The user presses `Cmd+K` in ScriptList. The renderer first gives any open actions dialog a chance to handle the key. If no actions popup is open, the generic Cmd block calls `handle_cmd_k_actions_toggle(window, cx)`. If an actions popup is open, the shared actions router closes it and returns `Handled`, so the generic Cmd block never reopens it.

### Navigate Actions Without Parent Leakage

With actions open, Up and Down move the selected action row and notify the actions window. Printable characters and Backspace update the actions search field and resize/notify the popup. Enter activates the selected action or drills into nested action routes. Unknown keys while the popup is open are still swallowed by the popup owner.

### Execute A Visible Action Shortcut

When actions are open, the shared router converts the keystroke into a normalized shortcut and matches only current filtered action rows. Hidden filtered-out actions must not win. Matching rows activate through the same action id route as Enter.

### Route Cmd+Enter To Agent Chat

Outside the actions popup, final renderer handling treats `Cmd+Enter` as an ACP context-capture route when no Shift, Alt, or Control modifiers are present. Inside an actions popup, `Cmd+Enter` belongs to the shared actions router, which may build an explicit ACP target chip for the selected action and preserve return origin.

### Execute Fallback Rows

When fallback rows are active, fallback Up/Down/Enter/Escape are handled before normal selected-row execution. Enter runs the selected fallback only when `gpui_input_focused` is false. Escape clears the filter and exits fallback mode.

### Close Popup Before Launcher State

Escape checks menu-syntax trigger popup state before clearing the filter, going back, or hiding the launcher. A trigger popup should close first, leaving the launcher filter/window state intact until a later Escape.

### Simulate The Same Route

Automation `simulateKey` normalizes key/modifier state, routes actions popups first when `show_actions_popup` and `current_actions_host()` exist, and then falls through to ScriptList-specific simulation. The send itself is not proof; agents must read follow-up state.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open actions. | ScriptList. | No actions popup. | `Cmd+K`. | Generic Cmd block -> `handle_cmd_k_actions_toggle`. | Actions dialog opens. | `getState.actionsDialog`. |
| Close actions. | Actions dialog. | Popup open. | `Cmd+K`. | `route_key_to_actions_dialog`. | Dialog closes; no reopen. | `actionsDialog` absent after key. |
| Move action selection. | Actions dialog. | Popup open. | Up/Down. | Shared actions router. | Selected action moves once. | selected action id changes one step. |
| Filter actions. | Actions dialog. | Popup open. | Printable / Backspace. | Shared actions router. | Visible actions narrow; parent filter stable. | `visibleActions`, launcher `filterText`. |
| Execute selected action. | Actions dialog. | Popup open. | Enter. | `activate_selected` -> `ActionsRoute`. | Drilldown, execute, or handled no-selection. | route side effect or action receipt. |
| Execute action shortcut. | Actions dialog. | Filtered actions. | Matching shortcut. | `keystroke_to_shortcut` -> `activate_action_id`. | Matching visible action executes. | filtered `visibleActions` plus action effect. |
| ACP action target. | Actions dialog. | AI-targetable action. | `Cmd+Enter`. | Shared router ACP handoff. | ACP opens with explicit target. | ACP state and return origin. |
| ACP current context. | ScriptList. | No popup. | `Cmd+Enter`. | `try_route_global_cmd_enter_to_acp_context_capture`. | ACP context capture. | ACP surface/context receipt. |
| Execute row. | ScriptList. | No popup/input focus. | Enter. | `execute_selected`. | Selected visible row runs. | selected stable key and transition/receipt. |
| Execute fallback. | ScriptList. | Fallback active. | Enter. | `execute_selected_fallback`. | Selected fallback runs. | fallback state exits or action receipt. |
| Close trigger popup. | ScriptList. | Menu-syntax popup open. | Escape. | trigger popup close intent. | Popup closes first. | popup absent; filter unchanged. |
| Clear filter. | ScriptList. | Filter text present. | Escape. | `clear_filter`. | Query becomes empty. | `filterText == ""`. |
| Close/go back. | ScriptList. | Empty filter. | Escape. | `go_back_or_close` or `close_and_reset_window`. | Return or hide/reset. | surface/window state. |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| ScriptList idle | Launcher open/reset. | Actions, fallback, ACP, execution, close. | `current_view == AppView::ScriptList`. |
| Shortcut recorder active | Add/Edit Shortcut. | Save/cancel/close. | Renderer returns immediately while recorder owns keys. |
| Global shortcut handled | Physical keydown. | Side-effect owner. | `handle_global_shortcut_with_options` claims the key. |
| SDK action shortcut handled | Physical keydown. | SDK action result. | Shortcut maps to `action_shortcuts` and triggers successfully. |
| Actions popup open | `Cmd+K` or footer action. | Close, route pop, action execute. | Shared router owns keys before generic Cmd block. |
| Fallback active | No normal rows / fallback state. | Fallback execute, filter clear. | Runs before final Enter/Escape behavior. |
| Proposal pending | Menu-syntax AI proposal. | Accept/dismiss. | No platform, Alt, or Control modifier. |
| Final Enter/Cmd+Enter/Escape | No earlier owner handled. | Execute, ACP capture, clear, go back, close. | `gpui_input_focused` blocks row execution. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| ScriptList idle | Launcher rows and filter. | Main filter/list. | `mainWindowPreflight`, selected key, filter text. |
| Actions popup Mini | Top-centered compact actions popup. | Actions dialog. | `actionsDialog`, active popup contract. |
| Actions popup Full | Bottom-right/full-style actions popup. | Actions dialog. | window position / popup metadata. |
| Shortcut recorder | Recorder popup/modal. | Shortcut recorder. | recorder automation window/popup. |
| Fallback active | Fallback rows. | Fallback selection. | fallback state/selected index. |
| Proposal pending | Proposal affordance. | Proposal handler. | proposal state and accept/dismiss receipt. |
| Trigger popup | Menu-syntax rows. | Trigger popup. | popup state/elements. |
| Embedded ACP | Agent Chat. | ACP composer. | ACP surface and return origin. |

## Keystrokes And Commands

| Key | Context | Behavior |
|---|---|---|
| `Cmd+K` | ScriptList, no popup. | Opens actions through current-view actions toggle. |
| `Cmd+K` | Actions popup. | Closes actions through shared router. |
| Up/Down | Actions popup. | Moves action selection once. |
| Enter | Actions popup. | Activates selected action or drilldown. |
| Escape | Actions popup. | Pops nested action route or closes dialog. |
| Backspace | Actions popup. | Updates action filter. |
| Printable char | Actions popup. | Filters visible actions. |
| Action shortcut | Actions popup. | Matches filtered visible action rows only. |
| `Cmd+Enter` | Actions popup. | ACP target handoff when selected action supports it. |
| `Cmd+Enter` | ScriptList, no popup. | Current context ACP capture when eligible. |
| Enter | ScriptList, no popup. | Executes selected row when input is not focused. |
| Enter | Fallback active. | Executes selected fallback row. |
| Tab / Enter | Proposal pending. | Accepts pending proposal when unmodified. |
| Escape | Proposal pending. | Dismisses pending proposal. |
| Escape | Trigger popup open. | Closes popup before clearing/closing launcher. |
| Escape | Filter text present. | Clears filter. |
| Escape | Empty filter. | Returns/closes according to launch origin. |

## Actions And Menus

This feature owns key delivery into actions, not the action catalog itself.

| Area | Owner |
|---|---|
| Main renderer key order. | Feature 041 / `keyboard-focus-routing`. |
| Root unified action subject and execution. | Feature 011. |
| Shared actions filtering, drilldown, close/focus restore. | `actions-popups`. |
| Main menu filtering and fallback rows. | Feature 001. |
| Special filter entries `~`, `/`, `@`, `>`, `?`. | Feature 013. |
| Shortcut recorder persistence and config mutation. | Feature 022. |
| ACP composer after handoff. | ACP context composer. |
| File Search / Quick Terminal after handoff. | Their dedicated features. |

Root actions must operate on the same focused ScriptList row identity as Enter. The current-view actions toggle resolves the selected ScriptList result, asks the root unified owner whether it has a root subject, and opens root unified result actions before falling back to existing script actions.

## Automation And Protocol Surface

| Surface | What it proves |
|---|---|
| `getState.actionsDialog` | Dialog open/closed state, host, selected action, visible actions. |
| `activePopupContract` | Shared actions popup owns keyboard/actions policy while attached. |
| `mainWindowPreflight.visibleResults` | Stable selected row identity before Enter or Cmd+K. |
| `simulateKey` | Fire-and-forget key injection; always follow with state proof. |
| `getElements` | Actions rows, popup rows, trigger popup rows, visible labels. |
| Logs/traces | `script_list.key_down`, actions route logs, simulateKey actions popup logs. |

Protocol `simulateKey` routes actions popup keys first when `show_actions_popup` and `current_actions_host()` are present. `Handled` and `Execute` both prevent view-specific simulation from also running.

## Data, Storage, And Privacy Boundaries

- Renderer key handling does not mutate durable storage by itself.
- Action execution can trigger source-owned side effects only after explicit key/action activation.
- Shortcut persistence belongs to shortcut/config features, not this renderer route.
- Logs may expose command ids, shortcut strings, action ids, and host names; treat them as workflow metadata.
- ACP handoff can move selected/current context into Agent Chat; prove it with content-light ACP state where possible.
- Action and root-source receipts should avoid raw note bodies, clipboard content, transcripts, browser contents, or similar local payloads.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Shortcut recorder active. | ScriptList key handler returns early; recorder owns capture/cancel. |
| Global shortcut consumes key. | Renderer returns before SDK action shortcuts or popup routing. |
| SDK action shortcut matches. | Renderer returns before shared actions routing. |
| `show_actions_popup == false`. | Shared actions router returns `NotHandled`. |
| `show_actions_popup == true` but `actions_dialog == None`. | Shared router returns `Handled`; parent keys are swallowed. |
| Unknown key while actions open. | Shared router returns `Handled`; parent ScriptList does not process it. |
| Actions Enter with no selection. | Returns `Handled`; parent selected row is not executed. |
| Actions Escape in nested route. | Pops route instead of closing root dialog. |
| Fallback active and unrecognized key. | Fallback block returns without normal row execution. |
| Enter while input is focused. | Does not execute selected/fallback row. |
| Trigger popup open on Escape. | Popup closes before filter/window state changes. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Physical ScriptList keydown | `src/render_script_list/mod.rs`. |
| Shared actions key routing | `src/app_impl/actions_dialog.rs`. |
| Actions open/close/focus | `src/app_impl/actions_toggle.rs`, `src/app_impl/actions_dialog.rs`. |
| Root unified subject routing | `src/app_impl/root_unified_result_actions.rs`. |
| Protocol key simulation | `src/main_entry/runtime_stdin_match_simulate_key.rs`. |
| Shortcut recorder guard | `src/app_impl/shortcut_recorder.rs`. |
| Shortcut action persistence | `src/app_actions/handle_action/shortcuts.rs`, config scripts. |
| Source audits | `src/app_impl/actions_dialog.rs` tests and root source action audits. |

## Invariants And Regression Risks

- `route_key_to_actions_dialog(...)` must stay before the generic Cmd shortcut block in `render_script_list`.
- Do not reintroduce a duplicate inline `show_actions_popup` key handler in the renderer.
- `Cmd+K` while actions are open must close the popup, not close and reopen it.
- Up/Down while actions are open must move one row, not double-step.
- Actions shortcuts must match only current filtered action rows.
- `ActionsRoute::Execute` must go through `execute_actions_route_action(...)`.
- Root unified actions must use captured subject state, not later live selection.
- Shortcut recorder state must keep ScriptList from stealing capture keys.
- Fallback mode must return before normal selected-row execution.
- Proposal accept/dismiss must run before final Enter/Escape behavior.
- Escape must close menu-syntax trigger popup before clearing filters or closing.
- `gpui_input_focused` must prevent Enter row execution.
- `simulateKey` must route actions popup first when an actions host exists.
- Do not claim screenshots prove key ownership; inspect state, stable keys, and popup contracts.

## Verification Recipes

Source audit checks recommended by Oracle:

```bash
cargo test --lib render_script_list_routes_popup_keys_before_generic_cmd_shortcuts -- --nocapture
cargo test --lib route_key_to_actions_dialog_notifies_after_arrow_navigation -- --nocapture
cargo test --lib route_key_to_actions_dialog_handles_cmd_k_close -- --nocapture
cargo test --lib route_key_to_actions_dialog_preserves_return_origin_for_explicit_acp_handoff -- --nocapture
cargo test --lib render_script_list_has_no_duplicate_popup_handler -- --nocapture
```

State-first runtime proof:

```text
1. Open ScriptList.
2. Press or simulate Cmd+K.
3. Assert actionsDialog opens with expected host/context.
4. Press or simulate Down once.
5. Assert selected action changes exactly once.
6. Press or simulate Cmd+K again.
7. Assert actionsDialog closes and does not reopen.
8. Reopen actions, type a filter character, and assert visibleActions narrows while launcher filter text stays stable.
9. Press Enter and assert drilldown, action execution, or handled no-selection behavior.
10. Press Escape from nested route and root route and assert pop/close semantics.
```

Protocol parity proof:

```text
1. `simulateKey` Cmd+K from ScriptList, then `getState`.
2. `simulateKey` Down while actions are open, then assert one-row movement.
3. `simulateKey` Enter while actions are open, then assert shared route behavior.
4. `simulateKey` Escape while actions are open, then assert pop/close.
5. With actions closed, `simulateKey` Cmd+Enter and assert ACP context capture when eligible.
6. With fallback active, assert Up/Down/Enter/Escape fallback behavior.
7. With trigger popup open, assert Escape closes popup before filter/window mutation.
```

## Agent Notes

- Do not debug renderer key issues from screenshots first; inspect `show_actions_popup`, `actions_dialog`, `current_actions_host`, `main_menu_fallback_state`, `pending_menu_syntax_ai_proposal`, `filter_text`, `opened_from_main_menu`, and `gpui_input_focused`.
- If `Cmd+K` is wrong, inspect the shared actions router, `handle_cmd_k_actions_toggle(...)`, and root unified subject capture in that order.
- If Enter is wrong, separate actions-popup Enter, fallback Enter, proposal Enter, menu-syntax popup Enter, and ordinary selected-row Enter.
- If simulated behavior differs from physical behavior, inspect `runtime_stdin_match_simulate_key.rs` before assuming a renderer bug.
- If a key is swallowed while actions are open, that may be intentional because the popup owner returns `Handled`.
- If a shortcut executes while actions are open, check whether it came from the SDK `action_shortcuts` map before shared actions routing.

## Related Features

| Feature | Relationship |
|---|---|
| [001 Main Menu](./001-main-menu.md). | Launcher filtering, grouped rows, fallback rows, and stable selection proof. |
| [011 Root Unified Search Result Actions](./011-root-source-actions.md). | Captured action subjects and root action execution. |
| [013 ScriptList Special Entry Triggers](./013-scriptlist-special-entry-triggers.md). | `~`, `/`, `@`, `>`, and `?` handoffs before renderer semantics after route change. |
| [022 Hotkey Prompt](./022-hotkey-prompt.md). | Shortcut recorder, config mutation, and recorder keys. |
| Actions Popups. | Shared actions dialog and shortcut matching. |
| Protocol Automation. | `simulateKey`, `getState`, `getElements`, and receipt proof. |
| ACP Context Composer. | ACP behavior after `Cmd+Enter` or special-entry handoff. |
| Quick Terminal and File Search. | Own keys after `>` or `~` handoff. |

## Open Questions And Gaps

- The Oracle bundle clipped part of the generic Cmd block after visible `Cmd+K`; expand full `src/render_script_list/mod.rs` before documenting remaining Cmd shortcuts.
- Physical Enter acceptance for the menu-syntax trigger popup was not visible in this renderer slice, while `simulateKey` Enter explicitly accepts the trigger popup before selected-row execution.
- `action_shortcuts` currently run before `route_key_to_actions_dialog(...)`; this may be intended, but it weakens broad "popup-first" claims when SDK shortcuts overlap popup-owned keys.
- `handle_global_shortcut_with_options(...)` runs before shared actions routing; confirm it cannot consume popup-owned action keys when actions are open.
- The router excerpt recognizes Home/End/PageUp/PageDown, but the packed window elided the body; do not specify jump-key behavior without expanding full source.
- Normal physical Up/Down outside fallback/actions popup is not visible in this focused renderer excerpt and may be owned by another listener/component path.
