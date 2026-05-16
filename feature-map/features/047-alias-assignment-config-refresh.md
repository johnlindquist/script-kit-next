# 047 Alias Assignment And Config Refresh

This chapter maps launcher alias assignment, AliasInput editing, alias override persistence, command-id ownership, refresh semantics, and current proof gaps.

Raw Oracle reference: [answer](../raw-oracle/047-alias-assignment-config-refresh/answer.md), [prompt](../raw-oracle/047-alias-assignment-config-refresh/prompt.md), [full log](../raw-oracle/047-alias-assignment-config-refresh/output.log), [session metadata](../raw-oracle/047-alias-assignment-config-refresh/session.json).

## Executive Summary

Launcher alias assignment is implemented separately from shortcut assignment, even though both share `src/app_actions/handle_action/shortcuts.rs` for action dispatch. Alias add/update/remove actions use launcher command IDs, open or mutate an AliasInput overlay, and persist user alias overrides in `~/.scriptkit/aliases.json`, not `config.ts`.

The safe product model is:

| Area | Current contract |
|---|---|
| User overrides | Stored as `command_id -> alias` in `~/.scriptkit/aliases.json`. |
| Metadata aliases | Read from script/scriptlet metadata and validated for duplicate bindings during catalog validation. |
| AliasInput prefill | Loads only persisted user overrides in the captured source. Metadata alias prefill is not proven. |
| Save refresh | Non-empty AliasInput save refreshes scripts after persistence. |
| Dedicated remove refresh | `remove_alias` action refreshes scripts after removing persistence. |
| Clear/empty removal gap | AliasInput Clear removes the override but does not call `refresh_scripts(cx)` in the captured source. |
| Runtime trigger gap | UI copy says type alias plus space to run, but this bundle does not prove the main-menu alias execution path. |

## What Users Can Do

| Capability | Entry | Result |
|---|---|---|
| Add an alias to a supported launcher row. | `add_alias` action. | Opens AliasInput for the row's launcher command ID. |
| Update an alias override. | `update_alias` action. | Opens AliasInput with the persisted override if one exists. |
| Remove an alias through actions. | `remove_alias` action. | Removes the override, shows HUD, refreshes scripts, and resets the main window. |
| Remove an alias from AliasInput. | Clear button or command/control Backspace/Delete. | Removes the override and closes the overlay; refresh is a gap. |
| Cancel editing. | Escape, Cancel, or backdrop click. | Closes AliasInput and returns focus intent to the main filter. |
| Type alias trigger. | Main-menu alias plus space. | Intended by help copy; dispatch implementation is not proven in this bundle. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Launcher command ID | Stable identity for alias persistence. | Alias overrides are keyed by the same command-id family used by shortcuts and command deeplinks. |
| Alias override | User-owned alias for a command ID. | Stored in `~/.scriptkit/aliases.json` and cached in memory. |
| Metadata alias | Alias declared by script/scriptlet metadata. | Duplicate metadata aliases are fatal catalog validation issues. |
| AliasInput | Modal overlay for editing one command alias. | Owns text validation, Save/Cancel/Clear, local keyboard handling, and focus. |
| Command deeplink | `scriptkit://commands/{commandId}` identity path. | Uses command IDs; aliases do not replace deeplinks. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `add_alias`. | Action selected for supported launcher row. | Opens AliasInput after resolving command ID/name. |
| `update_alias`. | Action selected for supported launcher row. | Same dispatch as add; preloads persisted override if present. |
| `remove_alias`. | Action selected for supported launcher row. | Removes persisted override and refreshes scripts. |
| `show_alias_input`. | App integration. | Loads overrides, creates `AliasInputState`, clears actions popup state, notifies UI. |
| `AliasInputAction::Save`. | Valid text saved. | Calls `save_alias_override`, shows HUD, refreshes scripts, closes overlay. |
| `AliasInputAction::Clear`. | Existing alias cleared. | Calls `remove_alias_override` via empty save; refresh is not shown. |
| `load_alias_overrides`. | Modal prefill and cache use. | Reads `~/.scriptkit/aliases.json`; missing file returns empty map. |
| `save_alias_override`. | Persist alias. | Creates `~/.scriptkit`, writes pretty JSON, invalidates cache. |
| `remove_alias_override`. | Remove alias. | Removes command key, writes JSON, invalidates cache. |

## User Workflows

### Add Alias

The user selects a supported launcher row and invokes `add_alias`. `handle_shortcut_alias_action` checks that there is a selected row, rejects unsupported row types, resolves `launcher_command_id()` and `launcher_command_name()`, logs `launcher_alias_input_requested`, and calls `show_alias_input(command_id, command_name, cx)`.

`show_alias_input` loads user overrides, preloads the override for the command ID if one exists, stores `AliasInputState`, clears action popup state, and causes the AliasInput overlay to render. The user types a valid alias and saves. The app writes `~/.scriptkit/aliases.json`, shows `Alias set: ...`, calls `refresh_scripts(cx)`, and closes the overlay.

### Update Alias

`update_alias` uses the same handler path as `add_alias`. The captured prefill source is only `load_alias_overrides().get(&command_id)`, so a metadata alias is not proven to prefill the input unless another omitted merge layer has already created an override or state value.

### Remove Alias Through Actions

The user invokes `remove_alias` on a supported row. The handler resolves the command ID and calls `remove_alias_override(&command_id)`. On success, it logs removal, shows `Alias removed`, calls `refresh_scripts(cx)`, then hides and resets the main window. On failure, it logs an error and returns an action failure.

### Clear Alias From AliasInput

If AliasInput opens with a current override, it renders a Clear button and accepts command/control Backspace/Delete. Clear produces `AliasInputAction::Clear`, which calls `save_alias_with_text(Some(String::new()), cx)`. Empty text removes the alias override and closes the overlay. The captured code does not call `refresh_scripts(cx)` on this path, so stale launcher alias state is a real gap.

### Cancel Editing

Escape, Cancel, and backdrop click produce cancel behavior. The app clears `alias_input_state`, clears the `alias_input_entity`, and sets pending focus back to the main filter.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Add alias. | `add_alias`. | Supported row selected. | Action activation. | `handle_shortcut_alias_action` -> `show_alias_input`. | AliasInput opens. | `src/app_actions/handle_action/shortcuts.rs`, `src/app_impl/alias_input.rs`. |
| Update alias. | `update_alias`. | Supported row selected. | Action activation. | Same as add. | AliasInput opens with persisted override if present. | `src/app_impl/alias_input.rs`. |
| Remove alias. | `remove_alias`. | Supported row selected. | Action activation. | `remove_alias_override` -> HUD -> `refresh_scripts`. | Override removed and launcher refreshed. | `src/app_actions/handle_action/shortcuts.rs`. |
| Save valid alias. | AliasInput. | Valid input. | Save or Enter. | `validate_alias_input` -> `save_alias_override` -> refresh. | Override written. | `src/components/alias_input/*`, `src/app_impl/alias_input.rs`. |
| Reject invalid alias. | AliasInput. | Invalid input. | Save disabled / Enter ignored. | `validate_alias_input` fails. | Error copy shown; no save. | `src/components/alias_input/component.rs`, tests. |
| Clear alias. | AliasInput. | Current alias exists. | Clear or modifier+Backspace/Delete. | `AliasInputAction::Clear` -> empty save -> `remove_alias_override`. | Override removed; refresh gap. | `src/app_impl/alias_input.rs`. |
| Cancel editing. | AliasInput. | Overlay open. | Escape, Cancel, backdrop. | `close_alias_input`. | Overlay closes; main filter focus intent restored. | `src/app_impl/alias_input.rs`. |
| Unsupported add/update. | Action surface. | Window/Skill/Note/Browser/Agent row. | Action activation. | Unsupported match arm. | Error outcome. | `src/app_actions/handle_action/shortcuts.rs`. |
| No selected row. | Action surface. | Nothing selected. | Action activation. | Selection-required helper. | Action-specific no-selection error. | `src/app_actions/helpers.rs`. |
| Duplicate metadata alias. | Script reload. | Catalog validation. | File watcher/startup validation. | `detect_binding_collisions`. | Colliding scripts excluded. | `src/scripts/validation.rs`, tests. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| Launcher idle. | No alias action active. | User selects rows/searches. | Alias actions are action-surface entries. |
| Action requested. | `add_alias`, `update_alias`, or `remove_alias`. | Validate selection and command ID. | Unsupported rows and no-selection short-circuit. |
| AliasInput opening. | Add/update with command ID. | Load overrides and create state. | Prefill reads user override store only in captured source. |
| Editing alias. | User types. | Validate on render/save. | Valid input enables Save; invalid input shows copy. |
| Saving alias. | Save valid alias. | Write override, invalidate cache, refresh scripts, close. | Non-empty save refreshes. |
| Clearing alias. | Clear existing alias. | Remove override, invalidate cache, close. | Refresh missing in captured source. |
| Removing alias action. | `remove_alias`. | Remove override, refresh scripts, reset launcher. | Dedicated remove path is stronger than modal clear. |
| Cancelled. | Escape/Cancel/backdrop. | Close state and return focus intent. | No persistence change. |
| Metadata validation conflict. | Duplicate metadata alias. | Fatal validation issue. | Distinct from user override duplicate behavior. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| AliasInput overlay. | Modal with command name, alias field, help copy, buttons. | Alias input focus handle. | `alias-input-overlay`, `alias-input-field`. |
| Empty alias input. | Save disabled; error/help copy. | Alias input. | Validation state rejects empty/whitespace. |
| Existing override. | Clear button visible. | Alias input. | `current_alias.is_some()`. |
| Invalid alias. | Error text. | Alias input. | Save disabled. |
| Saving/removing. | Overlay closes after persistence result. | Main filter pending focus. | `aliases.json` and cache invalidation. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Enter. | AliasInput with non-empty valid text. | Saves alias. |
| Escape. | AliasInput. | Cancels and closes overlay. |
| Command/Ctrl + Backspace/Delete. | AliasInput with existing alias. | Clears alias. |
| Backdrop click. | AliasInput overlay. | Cancels and closes overlay. |
| `add_alias` / `update_alias`. | Actions surface. | Opens AliasInput. |
| `remove_alias`. | Actions surface. | Removes persisted override. |

## Actions And Menus

| Action id | Meaning | Notes |
|---|---|---|
| `add_alias`. | Add alias for selected supported row. | Opens AliasInput. |
| `update_alias`. | Update alias for selected supported row. | Same handler as add. |
| `remove_alias`. | Remove alias override. | Calls persistence and refreshes scripts. |

Scriptlet context actions expose `update_alias` and `remove_alias` when `script.alias.is_some()`, otherwise `add_alias`. Full script/app/builtin action exposure is not completely mapped by the Oracle bundle and should be verified in the omitted builders before making broader claims.

## Alias Persistence And Refresh Semantics

| Operation | Store | Cache | Refresh |
|---|---|---|---|
| Save non-empty alias from AliasInput. | Inserts/updates `~/.scriptkit/aliases.json`. | Invalidated. | Calls `refresh_scripts(cx)`. |
| Remove alias from dedicated action. | Removes command key. | Invalidated. | Calls `refresh_scripts(cx)`. |
| Clear alias from AliasInput. | Removes command key. | Invalidated. | No refresh call shown. |
| Load override for prefill. | Reads aliases JSON. | Uses load path directly in shown code. | No refresh. |

The persistence module creates the `~/.scriptkit` directory when saving, writes pretty JSON, and stores a simple object keyed by command ID. Save uses `load_alias_overrides().unwrap_or_default()`, while remove propagates load errors. Malformed JSON may therefore behave differently between save and remove.

## Command IDs And Source Priority

Alias overrides use launcher command IDs. The command-id helper supports categories such as builtin, app, script, and scriptlet, and command deeplinks use `scriptkit://commands/{commandId}`.

The included snapshot does not prove the effective merge priority between user overrides in `aliases.json` and metadata aliases on scripts/scriptlets. Safe wording: user overrides are persisted by command ID; metadata aliases are validated separately; final effective search/display priority is a proof gap until the merge layer is inspected.

## Automation And Protocol Surface

No alias-specific protocol verb is shown. Automation should drive the real UI/action route:

| Receipt | Expected proof |
|---|---|
| Open actions and invoke `add_alias`. | AliasInput overlay appears. |
| `getElements`. | `alias-input-overlay`, modal content, and alias field are visible. |
| Type valid alias and save. | `aliases.json` contains `{ commandId: alias }`. |
| Remove alias. | `aliases.json` no longer contains command ID. |
| Refresh proof. | Non-empty save and dedicated remove refresh scripts; modal clear refresh remains a gap. |

Screenshots are secondary. Prefer state receipts, file-content assertions, and action/log receipts.

## Data, Storage, And Privacy Boundaries

- Alias overrides are local user data in `~/.scriptkit/aliases.json`.
- The file stores command IDs and aliases, not script bodies, shortcut keys, or command output.
- AliasInput keeps command ID, command name, and alias text in memory while editing.
- Raw Oracle bundles and output may contain repo-sensitive context and should be preserved but handled as internal artifacts.

## Error, Empty, Conflict, And Disabled States

| State | Behavior |
|---|---|
| No selected row. | Action-specific selection-required error. |
| Unsupported row. | Add/update rejects Window, Skill, Note, BrowserTab, BrowserHistory, and Agent. |
| Missing command ID. | Add/update/remove hide/reset and return command-id-specific error. |
| Empty alias. | UI validation rejects and disables Save. |
| Invalid characters. | UI accepts only alphanumeric, `-`, and `_`. |
| Whitespace. | UI rejects spaces/whitespace. |
| Too long. | UI validation rejects aliases over 32 characters. |
| Persistence failure. | Save/clear show error toast; dedicated remove returns action failure. |
| Duplicate metadata alias. | Fatal catalog validation issue; colliding scripts excluded. |
| Duplicate user override alias. | Not proven blocked; `save_alias_override` inserts by command ID without scanning values. |
| Hidden command alias behavior. | Not proven. |

## Code Ownership

| Area | Owner |
|---|---|
| Alias action dispatch. | `src/app_actions/handle_action/shortcuts.rs`. |
| Selection-required messages. | `src/app_actions/helpers.rs`. |
| AliasInput app integration. | `src/app_impl/alias_input.rs`. |
| Alias persistence/cache. | `src/aliases/mod.rs`, `src/aliases/persistence.rs`. |
| AliasInput component. | `src/components/alias_input/*`. |
| Command IDs/deeplinks. | `src/config/command_ids.rs`. |
| Metadata duplicate validation. | `src/scripts/validation.rs`. |
| Scriptlet action exposure. | `src/actions/builders/scriptlet.rs`. |
| Source audits. | `tests/source_audits/action_shortcut_alias.rs`, `tests/source_audits/shortcut_alias_file_actions.rs`. |

## Invariants And Regression Risks

- Alias overrides must remain in `~/.scriptkit/aliases.json`, not `config.ts`.
- Alias actions must resolve launcher command IDs before opening AliasInput or mutating persistence.
- Unsupported row types must not open AliasInput.
- UI validation must enforce empty, whitespace, character, and max-length rules.
- App-level save currently does not enforce max length for future non-UI callers.
- Clear-from-modal and dedicated remove have different refresh semantics; this should stay visible until fixed.
- Duplicate metadata alias validation must not be generalized to user overrides without proof.
- Do not claim alias-trigger execution from help copy alone.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| `cargo test alias_actions_open_alias_input`. | Add/update alias actions open AliasInput. |
| `cargo test alias_actions_show_error_when_no_selection`. | No-selection message stays action-specific. |
| `cargo test alias_actions_reject_window_items`. | Unsupported rows are rejected. |
| `cargo test remove_alias_calls_persistence_and_shows_hud`. | Dedicated remove uses persistence and HUD. |
| `cargo test alias_remove_refreshes_scripts_after_success`. | Dedicated remove refreshes scripts. |
| `cargo test test_validate_alias_rejects_empty_and_whitespace`. | UI validation rejects empty/whitespace. |
| `cargo test test_validate_alias_accepts_trimmed_valid_input`. | UI trims valid aliases. |
| `cargo test test_validate_alias_rejects_invalid_characters`. | UI rejects invalid characters. |
| `cargo test test_alias_clear_shortcut_requires_modifier_and_existing_alias`. | Clear shortcut is gated. |
| `cargo test duplicate_alias_normalizes_case`. | Metadata duplicate aliases normalize case. |
| `bun tests/smoke/test-alias-conflict.ts`. | Duplicate metadata alias conflict is surfaced at runtime. |
| Runtime AliasInput proof. | Open add alias, assert overlay/elements, save, inspect `aliases.json`, remove, inspect file again. |
| Atlas gates. | `lat check`, `git diff --check`, and `feature_explorer` build after index/chapter updates. |

## Agent Notes

Do not assume shortcut and alias persistence share a store. Shortcuts use config-backed command entries; aliases use `aliases.json`.

Do not claim user override duplicates are blocked unless the effective alias merge/execution layer is inspected.

If AliasInput opens empty for a row that visibly has a metadata alias, inspect the merge layer and decide whether metadata aliases should prefill the modal or only persisted overrides should.

If cleared aliases appear stale, inspect the missing `refresh_scripts(cx)` call in the empty-alias branch of `save_alias_with_text`.

## Related Features

| Feature | Relationship |
|---|---|
| [001 Main Menu](./001-main-menu.md). | Alias actions are launched from the main action surface and affect launcher search/run behavior. |
| [046 Shortcut Assignment And Config Refresh](./046-shortcut-assignment-config-refresh.md). | Shares action-dispatch handler and command ID concepts, but persistence differs. |
| Script metadata catalog. | Metadata aliases are validated with other script bindings. |
| Actions dialogs. | Alias add/update/remove are action-surface commands. |
| Command deeplinks. | Aliases and deeplinks both rely on command IDs. |

## Open Questions And Gaps

- Effective alias override priority over metadata aliases is not proven.
- Duplicate user override aliases are not proven blocked.
- AliasInput clear/remove does not refresh scripts in the captured code.
- App-level save does not enforce AliasInput's max-length rule.
- Full script/app/builtin action exposure needs omitted builder inspection.
- Hidden-command alias behavior is absent.
- Alias plus space runtime dispatch is represented by help text but not proven by included source.
- Malformed `aliases.json` save-vs-remove behavior differs and needs explicit tests.
