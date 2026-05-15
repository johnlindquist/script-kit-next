# 046 Shortcut Assignment And Config Refresh

This chapter maps config-backed launcher shortcut assignment, removal, recorder UI states, command IDs, `config.ts` writes, live hotkey activation, and proof boundaries.

Raw Oracle reference: [answer](../raw-oracle/046-shortcut-assignment-config-refresh/answer.md), [prompt](../raw-oracle/046-shortcut-assignment-config-refresh/prompt.md), [bundle map](../raw-oracle/046-shortcut-assignment-config-refresh/bundle-map.md), [full log](../raw-oracle/046-shortcut-assignment-config-refresh/output.log), [session metadata](../raw-oracle/046-shortcut-assignment-config-refresh/session.json), [failed attempt log](../raw-oracle/046-shortcut-assignment-config-refresh/output-failed-thinking-chip.log).

## Executive Summary

Shortcut assignment is a config-backed launcher command feature. The durable user-owned source for command shortcuts is `~/.scriptkit/config.ts`, specifically `commands[commandId].shortcut`.

Users assign or update shortcuts through `configure_shortcut`, `add_shortcut`, or `update_shortcut` actions. Those actions resolve the selected launcher row to `SearchResult::launcher_command_id()`, reject unsupported item types, and open a detached shortcut recorder popup. Saving the recorder calls `scripts/update-config-shortcut.ts`, which delegates to `scripts/config-cli.ts set-command-shortcut`.

Removal uses `remove_shortcut`, resolves the same command ID, calls `scripts/remove-config-shortcut.ts`, unregisters the dynamic shortcut, shows a HUD, refreshes scripts, and hides/resets the main launcher. Removal preserves known sibling command fields such as `hidden` and `confirmationRequired`; empty command config entries are deleted.

Conflict handling is wired for the normal recorder entry path. The recorder checks the live hotkey route table and blocks shortcuts already owned by config-backed commands, script/scriptlet metadata routes, or top-level app hotkeys; OS/global shortcuts outside Script Kit's route table are allowed through capture and handled by save-time registration results.

## What Users Can Do

- Assign or update a shortcut for a selected launcher command.
- Remove a config-backed shortcut from a selected launcher command.
- Record a shortcut that includes at least one modifier plus one non-modifier key.
- Cancel the recorder with `Esc`, `Cmd+W`, Cancel, backdrop click in inline mode, or detached-popup margin click.
- Clear a partially recorded shortcut before saving.
- Get immediate activation when live hotkey registration succeeds.
- Keep the saved config shortcut even if live registration fails; the HUD reports that the shortcut is saved but not active now.
- Remove shortcuts from durable config and unregister matching live dynamic routes when present.
- Use the config CLI or wrapper scripts for automation-safe shortcut setup and removal.

## Core Concepts

| Concept | Contract |
|---|---|
| Durable source | `~/.scriptkit/config.ts`. |
| Command shortcut field | `commands[commandId].shortcut`. |
| Shortcut shape | `{ modifiers: ["meta", "shift"], key: "KeyK" }`. |
| Recorder shape | `RecordedShortcut { cmd, ctrl, alt, shift, key }`. |
| Metadata defaults | Script/scriptlet metadata shortcuts remain defaults. |
| Config priority | `config.ts.commands` wins over metadata for the same command ID. |
| Legacy store | `shortcuts.json` is legacy only and must not be active startup/display/recorder/removal state. |
| Live activation | Save attempts `hotkeys::update_script_hotkey(...)`. |
| Verification receipt | `getConfigFingerprint` can prove config file metadata changed. |

Command IDs are shared by display, action handling, config writes, hotkey registration, removal, and deeplinks.

| Command type | ID shape |
|---|---|
| Built-in | `builtin/{id}` |
| App | `app/{bundleId}` when available |
| Script | `script/{owner}:{name}` |
| Scriptlet | `scriptlet/{owner}:{name}` |

## Entry Points

| Entry point | Behavior |
|---|---|
| `configure_shortcut` action | Opens recorder for the selected launcher command. |
| `add_shortcut` action | Same assignment path. |
| `update_shortcut` action | Same assignment path; existing config shortcut is overwritten unless the CLI is called with `--skip-existing`, which the UI path does not pass. |
| `remove_shortcut` action | Removes only the config shortcut field for the selected command. |
| `scripts/update-config-shortcut.ts` | Compatibility wrapper around `config-cli.ts set-command-shortcut`. |
| `scripts/remove-config-shortcut.ts` | Compatibility wrapper around `config-cli.ts remove-command-shortcut`. |
| `scripts/config-cli.ts` | Owns config parsing, mutation, validation, formatting, and JSON output. |
| `getConfigFingerprint` | Protocol receipt for config file path/length/mtime proof. |

The exact visible menu construction that chooses labels such as Add/Update/Remove Shortcut was not included in the Oracle bundle. The handler behavior is proven; visible row exposure rules remain a follow-up source pass.

## User Workflows

### Assign Or Update

The user selects a launcher row, opens actions, and triggers `configure_shortcut`, `add_shortcut`, or `update_shortcut`. The handler checks that a result is selected, rejects unsupported result types, then asks the result for `launcher_command_id()`.

If a command ID exists, the handler resolves `launcher_command_name()`, logs `launcher_shortcut_recorder_requested`, and calls `show_shortcut_recorder(command_id, command_name, window, cx)`. The recorder opens as a detached popup centered over the parent, uses the command name as the header, and shows `ID: {commandId}` as the description.

The user presses modifiers and a key. Modifier-only input updates live feedback but does not complete the shortcut. A bare non-modifier key without modifiers is ignored. A complete shortcut requires at least one modifier and a non-modifier key.

Save or Enter is enabled only when the shortcut is complete and no conflict is present. Saving re-checks live conflicts, writes the config shortcut, attempts live hotkey registration, reports active-now or saved-not-active HUD text, and closes the recorder.

### Remove

The user selects a launcher row and triggers `remove_shortcut`. The handler resolves the launcher command ID, calls the removal wrapper, unregisters the dynamic shortcut, logs removal, shows `Shortcut removed`, calls `refresh_scripts(cx)`, and hides/resets the launcher.

The CLI removal path deletes only `shortcut`. If `hidden` or `confirmationRequired` remains, the command entry is preserved. If no known sibling fields remain, the whole command entry is deleted.

### Cancel Or Clear

Cancel sets a pending cancel action and closes the recorder. Closing clears recorder state/entity, removes the automation window, schedules focus back to the main filter, and notifies.

Clear is only shown once the shortcut is non-empty. It resets the recorded shortcut and conflict state, then resumes recording.

### Manual CLI Setup

Automation can write to a temp config path:

```bash
SCRIPT_KIT_CONFIG_PATH="$tmpdir/config.ts" \
  bun scripts/config-cli.ts set-command-shortcut script/main:do-in-current-app 1 true false false false
```

The expected normalized write is `key: "Digit1"` with `modifiers: ["meta"]`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Assign shortcut | `configure_shortcut` | Launcher row selected | Trigger action | `handle_shortcut_alias_action` -> `launcher_command_id()` -> `show_shortcut_recorder` | Detached recorder opens | Action handler and source-audit tests. |
| Add shortcut | `add_shortcut` | Launcher row selected | Trigger action | Same assignment arm | Recorder opens | Same action handler. |
| Update shortcut | `update_shortcut` | Row has existing or default shortcut | Trigger action | Same assignment arm -> config CLI overwrite | Config shortcut replaced | CLI writer behavior. |
| Reject unsupported row | Assignment action | Window, Skill, Note, BrowserTab, BrowserHistory, or Agent selected | Trigger action | Explicit unsupported match | Error: shortcuts not supported | Handler source. |
| Reject no command ID | Assignment/removal action | Selected row lacks command ID | Trigger action | `launcher_command_id()` returns `None` | Hide/reset plus error | Handler source. |
| Record modifiers only | Recorder | Recording | Hold modifiers | Recorder modifier update | Live feedback; Save disabled | Component state/tests. |
| Press bare key | Recorder | Recording | Press key without modifiers | `should_finish_recording` false | Key ignored | Component tests. |
| Complete shortcut | Recorder | Recording | Press modifier plus key | `handle_key_down` sets key and checks conflict | Save enabled if no conflict | Component logic/tests. |
| Save shortcut | Recorder | Complete/no conflict | Save or Enter | `handle_shortcut_save` -> live conflict re-check -> wrapper -> config CLI -> live hotkey update | Config written; HUD active-now or saved-not-active | App path and CLI tests. |
| Cancel recorder | Recorder | Any recorder state | Esc, Cmd+W, Cancel, backdrop/margin | Pending cancel -> close recorder | Popup closes; main filter focus restored | Render and close paths. |
| Clear recorder | Recorder | Non-empty recorded state | Clear | `clear()` | Empty recording state | Component logic. |
| Remove shortcut | `remove_shortcut` | Row has command ID | Trigger action | removal wrapper -> config CLI -> unregister dynamic shortcut -> refresh scripts | Shortcut removed and launcher reset | Handler and tests. |
| Verify config changed | Protocol | Running app | `getConfigFingerprint` before/after | protocol receipt | Path/len/mtime receipt | `get_config_fingerprint_contract`. |

## State Machine

| State | Trigger | Guard | Transition | Side effect |
|---|---|---|---|---|
| Launcher idle | User selects row | Row exists | Selected result cached | Selection owner is main launcher. |
| Assignment requested | Shortcut action | Selected result exists | Validate row type | Unsupported rows fail clearly. |
| Command ID resolved | Supported row | `launcher_command_id()` exists | Open recorder | Clears actions popup state and opens detached popup. |
| Recorder opening | Popup creation succeeds | Parent window exists | Recorder visible | Registers automation popup and focuses recorder. |
| Recorder opening failed | Popup creation fails | Error | Back to launcher | Clears recorder state/entity; shows error toast. |
| Recording | Modifier/key input | Incomplete shortcut | Update display | Save disabled. |
| Complete | Modifier plus key | No conflict | Save-enabled | Enter/Save can create pending save action. |
| Conflict | Complete shortcut | Conflict checker returns conflict | Save-disabled warning | Live Script Kit route conflicts are blocked before save. |
| Save pending | Save/Enter | Complete/no conflict | Handle save | Runs wrapper and config CLI. |
| Write failed | Wrapper error | Non-zero/error | Close after error path | Error toast. |
| Write succeeded | Live registration succeeds | `update_script_hotkey` OK | Close recorder | HUD active-now. |
| Write succeeded | Live registration fails | `update_script_hotkey` error | Close recorder | HUD reports saved but not active now. |
| Removal requested | `remove_shortcut` | Command ID exists | Remove config shortcut | Unregister dynamic shortcut, refresh scripts, hide/reset. |
| Removal failed | Wrapper error | Error | Hide/reset | Dispatch error. |

## Visual And Focus States

The normal proven path opens a detached native popup, not an inline overlay. The popup is compact, centered over the parent window, uses popup window kind/chrome, and requests focus.

The recorder displays command name, command ID, modifier/key glyphs, Cancel, Save, and Clear when applicable. Save is disabled until a complete shortcut is captured and no conflict is present. Clear is hidden until some shortcut state exists.

`Esc` and `Cmd+W` cancel the recorder; they must not be captured as assignable shortcuts. Closing the recorder clears recorder entities and returns focus to the main filter.

## Keystrokes And Commands

| Input | Recorder behavior |
|---|---|
| Modifier only | Updates live modifier state; incomplete. |
| Bare key | Ignored while recording because no modifier exists. |
| Modifier plus key | Completes shortcut and checks conflict. |
| Enter | Saves only when complete and no conflict. |
| Esc | Cancels. |
| Cmd+W | Cancels. |
| Clear button | Resets recording state. |
| Save button | Persists shortcut when enabled. |

`cmd` in recorder state becomes `meta` in config. Display glyphs include platform symbols such as `⌘`, `⇧`, `⌥`, `⌃`, and special key glyphs.

## Config Write And Refresh Semantics

Assignment calls:

```bash
scripts/update-config-shortcut.ts <commandId> <key> <cmd> <ctrl> <alt> <shift>
```

That wrapper delegates to:

```bash
scripts/config-cli.ts set-command-shortcut <command_id> <key> <cmd> <ctrl> <alt> <shift>
```

Removal calls:

```bash
scripts/remove-config-shortcut.ts <commandId>
```

That wrapper delegates to:

```bash
scripts/config-cli.ts remove-command-shortcut <command_id>
```

After a successful assignment write, the app attempts live activation via `hotkeys::update_script_hotkey(&command_id, None, Some(&shortcut_str))`. If this succeeds, the HUD reports active-now. If it fails, the config write remains valid and the HUD reports that the shortcut is saved but not active now.

After a successful removal, the app calls `hotkeys::unregister_dynamic_shortcut(&command_id)`, removes the app route before best-effort OS unregister, logs whether the live route was removed or absent, shows a HUD, calls `refresh_scripts(cx)`, and hides/resets the main launcher.

Assignment refreshes scripts after a successful config write, whether live registration succeeds immediately or reports saved-not-active. This keeps row hints aligned with durable config while preserving the recoverable live-registration failure policy.

## Automation And Protocol Surface

| Proof target | Receipt |
|---|---|
| Recorder popup exists | `getElements` for attached popup `shortcut-recorder-popup` with surface `shortcutRecorder`. |
| Config file changed | `getConfigFingerprint` before/after write. |
| Shortcut persisted | Inspect temp `SCRIPT_KIT_CONFIG_PATH` file contents. |
| Save enabled/disabled | Recorder elements and button state. |
| Unsupported row | Action error receipt. |
| Live activation | Trigger shortcut and observe command route; source tests prove registration/update paths, while full OS delivery remains runtime-only. |
| Removal refresh | State/row receipt after `refresh_scripts(cx)` if exposed. |

State-first proof should use an isolated config path. Screenshots are secondary and mainly useful for popup chrome/layout regressions.

## Error, Empty, Loading, Conflict, And Disabled States

| State | Behavior |
|---|---|
| No selection | Assignment/removal returns the shared selection-required message. |
| Unsupported assignment row | Error: `Shortcuts not supported for this item type.` |
| Selected row without command ID | Assignment/removal hides/resets and returns cannot-assign/cannot-remove error. |
| Bad shortcut key | CLI normalization throws `Invalid shortcut key: {key}`. |
| Bare key input | Ignored; no toast. |
| Save disabled | Until shortcut is complete and conflict is none. |
| Clear hidden | Until recorder state is non-empty. |
| Conflict | Normal recorder paths wire `shortcut_conflict_for_recording` and block save for live route conflicts. |
| Wrapper missing | Error says the script could not be found in `.scriptkit/sdk` or repo scripts. |
| Bun path problem | Falls back to `bun` unless configured path exists; process failure surfaces through write/removal error path. |
| Config parse/write failure | CLI errors bubble to wrapper failure path. |
| Remove absent shortcut | CLI returns success with `removed:false`; app still treats wrapper success as removal success. |
| Loading/saving spinner | No explicit loading UI was proven in the bundled recorder. |

Unknown sibling fields under `commands[commandId]` are not guaranteed. The formatter serializes `shortcut`, `hidden`, and `confirmationRequired`; unknown fields are likely dropped unless schema work proves otherwise.

## Code Ownership

| Area | Files |
|---|---|
| Shortcut contract | `lat.md/shortcuts.md` |
| Config CLI writer/remover | `scripts/config-cli.ts` |
| Compatibility wrappers | `scripts/update-config-shortcut.ts`, `scripts/remove-config-shortcut.ts` |
| App recorder integration | `src/app_impl/shortcut_recorder.rs` |
| Shortcut actions | `src/app_actions/handle_action/shortcuts.rs` |
| Recorder component | `src/components/shortcut_recorder/*` |
| Shortcut parsing/display | `src/shortcuts/*` |
| Command ID helpers | `src/config/command_ids.rs` |
| Source audits | `tests/source_audits/shortcut_config_source.rs`, `tests/source_audits/action_shortcut_alias.rs`, `tests/source_audits/shortcut_lookup_exports.rs` |
| Popup/window contracts | `tests/shortcut_recorder_popup_window_contract.rs` |
| Config fingerprint | `tests/get_config_fingerprint_contract.rs` |

## Invariants And Regression Risks

- `config.ts` is the only durable user-owned launcher shortcut source.
- Do not reintroduce `shortcuts.json` into startup registration, display, recorder save, or removal.
- User assignment writes config only; it must not mutate script metadata.
- Config-backed shortcuts override script/scriptlet metadata.
- Assignment must use `launcher_command_id()`, not ad hoc ID construction.
- Unsupported rows must fail clearly.
- Recorder must require at least one modifier plus one non-modifier key.
- `Esc` and `Cmd+W` cancel; they must not become captured shortcuts.
- Removal must preserve `hidden` and `confirmationRequired`.
- Conflict blocking is limited to live Script Kit routes; OS/global reservations outside that table remain save-time registration failures.
- Save attempts live activation but does not prove immediate row display refresh.

## Verification Recipes

Config CLI mutation proof:

```bash
tmpdir="$(mktemp -d)"
cat > "$tmpdir/config.ts" <<'TS'
import type { Config } from "@scriptkit/sdk";
export default { hotkey: { modifiers: ["meta"], key: "Semicolon" }} satisfies Config;
TS
SCRIPT_KIT_CONFIG_PATH="$tmpdir/config.ts" \
  bun scripts/config-cli.ts set-command-shortcut script/main:do-in-current-app 1 true false false false
cat "$tmpdir/config.ts"
```

Removal preservation proof:

```bash
tmpdir="$(mktemp -d)"
cat > "$tmpdir/config.ts" <<'TS'
import type { Config } from "@scriptkit/sdk";
export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  commands: {
    "builtin/clipboard-history": {
      shortcut: {"modifiers":["meta"],"key":"KeyV"},
      hidden: true,
    },
  },
} satisfies Config;
TS
SCRIPT_KIT_CONFIG_PATH="$tmpdir/config.ts" \
  bun scripts/config-cli.ts remove-command-shortcut builtin/clipboard-history
cat "$tmpdir/config.ts"
```

Targeted tests:

```bash
bun test scripts/config-cli.test.ts
cargo test --test shortcut_recorder_popup_window_contract
cargo test --test shortcut_error_messages
cargo test --test get_config_fingerprint_contract
cargo test --test source_audits shortcut_config_source
cargo test --test source_audits action_shortcut_alias
cargo test shortcut_recorder
cargo test shortcuts
lat check
git diff --check
```

Recommended state-first runtime proof:

1. Start the app with an isolated `SCRIPT_KIT_CONFIG_PATH`.
2. Capture `getConfigFingerprint`.
3. Select a command row and invoke the real shortcut action.
4. Assert `shortcut-recorder-popup` exists with `getElements`.
5. Simulate or deliver a valid modifier-plus-key shortcut.
6. Assert Save is enabled, save it, and re-read `getConfigFingerprint`.
7. Inspect the temp config file for `commands[commandId].shortcut`.
8. For removal, invoke `remove_shortcut`, prove the shortcut field is gone, prove known sibling fields remain, and verify row/display refresh if exposed.

## Agent Notes

- Do not assume `shortcuts.json` participates in assignment, removal, startup, display, or recorder behavior.
- Duplicate shortcut conflicts are blocked in normal UI when the conflicting shortcut is already present in the live Script Kit hotkey route table.
- Do not claim assignment refreshes main-menu row shortcut hints immediately.
- Do not claim script or scriptlet metadata is edited.
- Do not claim every search result row supports shortcuts.
- Do not claim `open_config_for_shortcut` is user-reachable from this snapshot; Oracle found it marked dead code.
- Do not treat config template examples as the final command-ID contract where they conflict with `lat.md/shortcuts.md`.
- To verify config mutation, prefer temp `SCRIPT_KIT_CONFIG_PATH`, CLI JSON, file inspection, and `getConfigFingerprint`.
- To verify UI, prefer `getState`, `getElements`, `waitFor`, attached popup registration, and runtime action receipts.
- Screenshots are only needed for visual/chrome assertions.

## Related Features

- [001 Main Menu](./001-main-menu.md)
- [022 Hotkey Prompt](./022-hotkey-prompt.md)
- [035 Settings Theme Config Preferences](./035-settings-theme-config-preferences.md)
- [041 Main Menu Renderer Key Handling](./041-main-menu-renderer-key-handling.md)

## Open Questions And Gaps

- Visible action menu exposure is not fully mapped. The handler proves action IDs and behavior, but not exactly which rows show Add/Update/Remove Shortcut labels.
- Conflict detection is wired for the detached and legacy recorder entry paths.
- Assignment display refresh is source-audited after successful config writes; full row repaint proof is runtime-only.
- Config watcher/reload behavior is not shown.
- Hotkey registration internals, duplicate OS registration handling, and error mapping are outside this bundle.
- Shortcut display implementation is only source-audit visible in this pass.
- Inline overlay path appears unreachable in the bundled source; detached popup is the clearly active path.
- Hidden-command shortcut action exposure is not proven.
- Unknown command sibling fields are not guaranteed preserved.
- Config template examples may lag the command-ID contract for script/scriptlet IDs.
