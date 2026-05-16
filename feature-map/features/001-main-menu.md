# 001 Main Menu / ScriptList / Menu Syntax / Actions / Shortcut Assignment

The main menu is the launcher-owned ScriptList surface where users search commands, scripts, apps, files, passive local sources, power syntax, and row actions from one filter input.

## Executive Summary



## What Users Can Do

- Type plain text to find scripts, scriptlets, built-ins, apps, skills, windows, and fallback commands.
- Use root Files as a passive section for eligible filename queries, explicit directory paths, and recent-file rows.
- Use `;` or `+` capture syntax to open capture target rows, filter targets, and compose structured capture payloads.
- Use `@`, `~`, `/`, `>`, and `?` legacy triggers without letting menu syntax claim them incorrectly.
- Press Enter to execute the visible selected row, including root files, passive rows, fallback continuation rows, and normal launcher rows.
- Press Cmd+K to open row-specific MainList actions, search those actions, and execute an action against the captured selected row.
- Add, edit, remove, or inspect command shortcuts through config-backed `config.ts.commands[commandId].shortcut`.
- Use action-row shortcuts such as root-file reveal/copy/Quick Look without opening the actions dialog.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| ScriptList | Default launcher `AppView` with main filter input and grouped results. | `src/render_script_list/mod.rs` |
| Main filter | The single-line input that drives launcher search, source filters, triggers, and handoffs. | `src/app_impl/filter_input_change.rs`, `src/app_impl/filter_input_updates.rs` |
| Grouped results | Stable, role-aware projection of scripts, commands, files, passive sources, and fallback rows. | `src/scripts/grouping.rs`, `src/scripts/types.rs` |
| Menu syntax popup | Attached prompt popup for capture, filter, and command discovery rows. | `src/app_impl/menu_syntax_trigger_popup.rs`, `src/app_impl/menu_syntax_trigger_popup_window.rs` |
| Actions dialog | Attached Cmd+K popup whose host and subject come from the focused surface/row. | `src/app_impl/actions_dialog.rs`, `src/app_impl/actions_toggle.rs` |
| Root action subject | Captured row identity for root files/passive rows so execution does not re-read changing selection. | `src/app_impl/root_unified_result_actions.rs` |
| Shortcut recorder | Attached popup that records and writes command shortcuts to `config.ts`. | `src/app_impl/shortcut_recorder.rs` |
| Command ID | Stable config/deeplink/hotkey identity such as `script/...`, `scriptlet/...`, `builtin/...`, or `app/...`. | `src/scripts/types.rs`, `removed-docs` |

## Entry Points

| Entry point | User input | Result |
|---|---|---|
| Type plain text | `deploy`, `calendar`, `raycast` | Grouped fuzzy results from primary launcher sources and eligible passive rows. |
| Type path | `~`, `~/dev/`, `/tmp` | Mini File Search or root directory browse handoff; no source-chip decoration. |
| Type capture sigil | `;`, `;todo`, `+todo` | Capture target picker or capture composer. |
| Type legacy trigger | `@`, `/`, `>`, `?`, `~` | Legacy special-entry route or literal launcher handling, not generic menu-syntax popup ownership. |
| Press Cmd+K | Focused ScriptList row | MainList actions dialog for selected row. |
| Choose shortcut action | Add/Edit/Remove Shortcut | Shortcut recorder or config shortcut removal path. |

## User Workflows

### Search And Execute A Launcher Row

The user opens the launcher, types plain text, arrows to a row, and presses Enter. ScriptList updates `filter_text`, installs computed text/decorations, rebuilds grouped results, preserves selection by stable key where possible, and executes the visible selected result. Fallback and continuation rows must execute the row that is visibly selected, not a stale legacy cursor.

### Search Files From Root

For eligible filename queries, root Files appears below primary launcher rows and before fallbacks. Provider work may warm a cache and update loading receipts, but it must not stream new rows into the current query frame. Cached rows can appear when a future frame is built. Explicit directory paths intentionally switch to bounded direct-child browse and may replace the active direct-child batch.



### Search A Passive Source


### Use Advanced Filter Discovery


### Use Capture Syntax

Typing `;` opens capture target rows. Typing `;todo` filters target rows; typing `;todo Buy milk #errands` enters capture composer state. Unknown capture heads remain normal launcher search until metadata registers them. Footer actions can open help, captures, create a handler, or scaffold a handler through ACP.

### Open And Execute Actions

Cmd+K on a ScriptList row opens the MainList actions popup. Script/scriptlet/built-in/app rows delegate to existing script action owners so shortcut, alias, and deeplink actions are preserved. Root file and passive rows capture a root action subject so action execution remains correct after focus resync, cache warming, or arrow movement.

### Assign A Shortcut

The user opens actions for a command row, chooses Add/Edit Shortcut, records a key, and saves. The recorder writes `config.ts.commands[commandId].shortcut` through `scripts/config-cli.ts set-command-shortcut`, updates the live hotkey table, closes the popup, restores main filter focus, and shows HUD feedback. Removal deletes only the `shortcut` field and preserves sibling command config.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Search commands | Main filter | ScriptList | Type text | `filter_input_change` / grouping | Primary rows recomputed | `mainWindowPreflight.visibleResults` |
| Reject newline | Main filter | ScriptList | Paste newline | Filter input owner | Canonical filter unchanged | `filter_change.newline_ignored` log/state |
| Execute selected primary | Main list | ScriptList | Enter | Selected `SearchResult` execution | Command/script/app runs | Selected stable key receipt |
| Execute fallback | Main list | Fallback selected | Enter | Grouped fallback owner | Visible fallback runs | Fallback stable-selection proof |
| Search root file | Main filter | Root Files | Type eligible filename | `root_file_search` + grouping | Files section appears | Root file state/preflight |
| Browse path | Main filter | Root directory browse | Type `~/dev/` | Directory browse branch | Direct children list | File rows + continuation |
| Enter directory | Main list | Root directory row | Tab | Root Files directory owner | Query rewrites to folder | Directory-browse proof |
| Move parent | Main list | Directory browse | Shift+Tab | Root Files directory owner | Query moves parent / clears fragment | Directory-browse proof |
| Source status | Main list | Source-filter mode | Query source | Source status metadata | Non-selectable status row | `getElements` role `status` |
| Capture target | Main filter | Prompt popup | `;` / `+` | Trigger picker snapshot | Capture rows | Menu syntax rows |
| Legacy mention trigger | Main filter | Special route | `@` | Special entry detector | ACP mention route or literal handling | `script_list_special_entry_routed` |
| Path handoff | Main filter | File Search mini | `~` / `/...` | File Search special entry | Mini File Search | No stale source chip |
| Open actions | Main list | ScriptList | Cmd+K | `handle_cmd_k_actions_toggle` | Actions dialog opens | `actionsDialog.host=MainList` |
| Execute action | Actions dialog | Popup | Enter / shortcut | Actions activation callback | Captured action runs | Physical/simulated parity proof |
| Add shortcut | Actions dialog | Shortcut action | Enter | `show_shortcut_recorder` | Recorder popup opens | `shortcut-recorder-popup` |
| Save shortcut | Recorder | Popup | Save | `config-cli.ts set-command-shortcut` | `config.ts` updated, hotkey refresh | Shortcut config source tests |
| Remove shortcut | Actions dialog | Shortcut action | Enter | `config-cli.ts remove-command-shortcut` | Shortcut field removed only | Config CLI tests/source audit |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Empty root ScriptList | Launcher open/reset | Plain search, trigger, source filter, child surface | Suggested/default rows and optional capped Recent Files. |
| Plain search | Character input | Empty, source mode, popup, child surface, execution | Selection uses stable keys, not input-history keys. |
| Root Files loading | Eligible filename query | Warm future frame, query change, continuation | Provider completion cannot mutate same-query visible frame. |
| Root directory browse | Explicit path query | Child fragment, Tab, Shift+Tab, File Search | Intentional direct-child browse may update active rows. |
| Source-filter mode | Known source head | Delete source head, execute row, source-only browse | Disallowed sources suppressed; Up/Down stay list navigation. |
| Capture body composer | Exact capture target plus body | Enter, Cmd+K, Escape, text edit | Body text is payload, not fuzzy target search. |
| Actions closed | Normal main list | Cmd+K | Host and selected row decide action catalog. |
| Actions open | Cmd+K | Escape, Cmd+K, action, blur/backdrop | Action execution uses captured subject before close clears context. |
| Shortcut recorder | Add/Edit Shortcut | Save, cancel, close | Main list key handling yields while recorder state exists. |
| Child special route | File Search, ACP mention, Quick Terminal, Actions Help | Return/close | Special route owns input until it returns. |

## Visual And Focus States


## Keystrokes And Commands

| Key | Context | Behavior |
|---|---|---|
| Character input | Main filter | Updates filter, computed text, grouped rows, and selection where owned. |
| Backspace/Delete | Source filter | Removing source head clears source chips and source-filter history block. |
| ArrowUp/ArrowDown | Main list | Moves selected row; source-filter mode blocks input-history recall. |
| ArrowUp/ArrowDown | Menu popup | Moves popup row selection and visible window. |
| ArrowUp/ArrowDown | Actions dialog | Moves selected action exactly once. |
| Enter | Main list | Executes selected visible row. |
| Enter | Actions dialog | Executes selected action before close clears captured subject. |
| Enter | Menu popup | Accepts selected trigger/qualifier/footer outcome. |
| Tab | Selected root directory | Rewrites query to browse into directory. |
| Shift+Tab | Root directory browse | Clears child fragment or moves to parent. |
| Cmd+K | Main list | Opens/toggles MainList actions. |
| Cmd+K | Actions dialog | Closes/toggles actions. |
| Escape | Menu popup | Closes popup and reconciles ScriptList. |
| Escape | Actions dialog | Closes actions and restores focus. |
| Escape | Shortcut recorder | Cancels recorder and restores main filter. |
| Cmd+Shift+F | Selected root file | Reveal in Finder. |
| Cmd+Shift+C | Selected root file | Copy full file path. |
| Cmd+Y | Selected root file | Quick Look selected file. |
| `;` / `+` | Main filter | Capture picker when configured. |
| `@` | Main filter | Legacy ACP mention/special-entry path, not generic source syntax. |
| `~` / `/...` | Main filter | Path/file-search handoff. |

## Source Filter And Trigger Reference

| Input | Classification | Behavior |
|---|---|---|
| `;target` | Capture syntax | Registered capture target composer. |
| `+target` | Legacy capture alias | Same target-gated behavior as `;target`. |
| `@` | Legacy trigger | Closes menu-syntax popup and may route to ACP mention picker. |
| `~` | Legacy file trigger | Opens mini File Search/path handoff. |
| `>` | Command invocation / legacy head | Parser-owned argv composer for registered command heads; exact legacy behavior needs focused follow-up. |
| `?` | Legacy trigger/help | Special-entry route, not generic source filter. |

## Actions And Shortcut Assignment

MainList actions resolve from the selected visible row.

- Script and scriptlet rows keep script-specific actions plus global actions.
- Built-in and app rows keep config-backed Add/Edit Shortcut, Add/Edit Alias, and Copy Deep Link through `launcher_command_id()`.
- Root file rows expose Open, Reveal, Copy Path, Copy Name, Quick Look, Search Inside Folder for directories, and Browse Parent Folder for regular files.
- Passive rows expose typed root actions through `RootUnifiedActionSubject` with content-light metadata.
- Unknown root action IDs no-op and must not fall through to generic script handling.


| Operation | Path | Contract |
|---|---|---|
| Add/Edit Shortcut | Recorder save -> `scripts/config-cli.ts set-command-shortcut` | Writes `config.ts.commands[commandId].shortcut`. |
| Live refresh | `src/hotkeys/mod.rs` update after write | New shortcut can work before restart when refresh succeeds. |
| Remove Shortcut | `scripts/config-cli.ts remove-command-shortcut` | Removes only `shortcut`; preserves sibling command fields. |
| Display shortcut | `get_command_shortcut(command_id)` | Reads config-backed shortcut display. |
| Startup registration | Hotkey listener | App hotkeys first, config command shortcuts second, inline metadata third if not overridden. |
| Legacy guard | Source audits | `shortcuts.json` is never active startup/display/recorder/removal source. |

## Automation And Protocol Surface

| Receipt | What it proves |
|---|---|
| `getState.surfaceContract` | Current main surface, focus/actions/proof policy, and ScriptList identity. |
| `mainWindowPreflight.visibleResults` | Visible row roles, stable keys, ranks, sources, action kinds, and selected identity. |
| `filterInputDecorations` | Live rendered source/power-syntax spans; replacement semantics clear stale chips. |
| `promptPopup` / `getElements(target)` | Menu-syntax popup rows, tokens, footer actions, highlights, enabled state. |
| `actionsDialog` | Host, context title/stable key/source, selected action id, visible action metadata. |
| `mainListScroll` | Source-chip pagination, selected-row visibility, footer-safe reveal. |
| `shortcut-recorder-popup` | Attached recorder bounds/focus/parent identity. |

State receipts should avoid raw local payloads. Root passive action receipts must not expose note bodies, clipboard text, dictation transcripts, browser page contents, or similar local content.

## Data, Storage, And Privacy Boundaries

- `config.ts` is the durable user-owned shortcut source.
- `config.ts.commands[commandId].shortcut` overrides script/scriptlet metadata shortcuts.
- `shortcuts.json` is legacy only and must not influence active display, startup, recorder, or removal behavior.
- Root passive rows carry metadata-only row content where needed; full local content loads only after explicit user action.
- Source status is metadata, not an executable row or action subject.
- Root file frecency is recorded only after successful OS open from Enter or equivalent root-file Open action.
- Menu-syntax capture payloads should not be written or spawned when schema validation blocks submission.

## Error, Empty, Loading, And Disabled States

- Empty root shows suggested/default rows and optional capped Recent Files without starting file providers.
- Empty Recent Files should not expose `.app` internals or direct `.app` bundles unless the user intentionally browses directories.
- Root Files loading shows `Files - Searching...` or `Files - Loading folder...` while selection remains stable.
- Source-filter no-results should show filter-specific recovery/status copy, not generic no-results guidance.
- Unsupported or quoted source-looking tokens remain literal.
- Disabled source heads can still opt into their source explicitly when the user types that source head.
- Config write failures should surface toast/HUD feedback instead of silently closing with a false success.
- Missing root-file Quick Look paths should report controlled HUD errors.

## Code Ownership

| Behavior | Owner files/tests |
|---|---|
| ScriptList rendering and key handling | `src/render_script_list/mod.rs` |
| Filter input change/update pipeline | `src/app_impl/filter_input_change.rs`, `src/app_impl/filter_input_updates.rs` |
| Source-filter parsing/highlighting | `src/menu_syntax/query.rs`, `src/menu_syntax/mode.rs` |
| Menu-syntax popup lifecycle | `src/app_impl/menu_syntax_trigger_popup.rs`, `src/app_impl/menu_syntax_trigger_popup_window.rs` |
| Advanced query/filter grammar | `src/menu_syntax/filter.rs`, `removed-docs` |
| Root Files | `src/app_impl/root_file_search.rs`, `src/file_search/mod.rs`, `src/scripts/grouping.rs` |
| Grouped selection identity | `src/scripts/types.rs`, `src/scripts/grouping.rs`, `src/main_window_preflight/` |
| Actions host/toggle/close | `src/app_impl/actions_dialog.rs`, `src/app_impl/actions_toggle.rs` |
| Root unified result actions | `src/app_impl/root_unified_result_actions.rs` |
| Shortcut recorder | `src/app_impl/shortcut_recorder.rs` |
| Shortcut action handlers | `src/app_actions/handle_action/shortcuts.rs` |
| Config shortcut CLI | `scripts/config-cli.ts`, `scripts/update-config-shortcut.ts`, `scripts/remove-config-shortcut.ts` |
| Hotkey registration | `src/hotkeys/mod.rs` |
| Verification contracts | `tests/source_audits/root_unified_source_filters_contract.rs`, `tests/source_audits/root_unified_source_actions_contract.rs`, `tests/source_audits/shortcut_config_source.rs` |

## Invariants And Regression Risks

- The main filter is single-line; newline input must not corrupt the root query.
- Async root file/passive providers must not mutate the current visible frame for the same query.
- Source filters are transparent refinements, not separate app modes, and source heads can appear anywhere as standalone tokens.
- Home/path input (`~`, `~/...`, `/tmp`) must not inherit stale source-chip decoration.
- Source-chip status must not affect executable row count, selection, mini sizing, scroll height, or action subjects.
- Source-filter mode blocks launcher input-history recall so arrows remain list navigation.
- Menu-syntax popup footer rows must not be default-selected as normal row actions.
- Actions execute against the captured subject before close/reset clears pending context.
- Physical Enter and protocol/simulated Enter should route action activation with the same ownership.
- Shortcut recorder state must intercept keys before the main list.
- Config-backed shortcuts override script/scriptlet metadata without mutating script files.
- Shortcut removal must preserve sibling command fields and never write `shortcuts.json`.
- MainList actions for built-ins/apps must use stable launcher command IDs, not display labels.

## Verification Recipes


```bash
cargo test --test menu_syntax_source_filters -- --nocapture
cargo test --test source_audits root_unified_source_filters_contract -- --nocapture
cargo test --test source_audits root_unified_source_actions_contract -- --nocapture
cargo test --test source_audits root_file_action_enter_routes_activation_before_close -- --nocapture
cargo test --test source_audits root_file_quick_look -- --nocapture
cargo test --test source_audits root_file_browse_parent_folder -- --nocapture
cargo test --test source_audits shortcut_config_source -- --nocapture
bun test scripts/config-cli.test.ts
cargo check --lib
cargo fmt --check
git diff --check
source checks
```


```bash
bun scripts/agentic/root-search-frame-stability.ts
bun scripts/agentic/root-passive-frame-stability.ts
bun scripts/agentic/root-source-filter-stability.ts
bun scripts/agentic/root-source-filter-clipboard.ts
bun scripts/agentic/root-source-filter-history-up.ts --timeout 12000
bun scripts/agentic/source-chip-pagination-proof.ts --timeout 16000
bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000
bun scripts/agentic/root-source-filter-lazy-scroll.ts --query s --timeout 20000
bun scripts/agentic/root-source-actions-matrix.ts
```


- `@` routes as a legacy/special entry and does not leave menu-syntax popup state behind.
- `~/...` clears stale source/power-syntax decorations before File Search paints.
- Cmd+K on root file/passive rows exposes content-light actions and captures context.
- Saving a shortcut updates `config.ts.commands[commandId].shortcut` and refreshes hotkeys.
- Removing a shortcut preserves sibling command fields and leaves no active `shortcuts.json` path.

Screenshots are only needed for visual acceptance of row chrome, popup placement, shortcut-recorder appearance, or menu-syntax highlight colors. State, source audits, and agentic receipts should cover normal behavior.

## Agent Notes

- Do not collapse `@`, `~`, `/`, `>`, and `?` into menu-syntax popup ownership; they are legacy/special-entry boundaries unless a focused parser path says otherwise.
- To verify selection stability, inspect `mainWindowPreflight.visibleResults` and stable keys instead of reading row labels from screenshots.
- If Enter runs the wrong thing, inspect grouped selection projection, fallback execution order, and same-query provider frame mutation.
- If Cmd+K runs the wrong action, inspect captured `RootUnifiedActionSubject` and close-before-execute ordering.
- If shortcut display differs from saved config, inspect `config.ts.commands`, `get_command_shortcut`, and hotkey registration order.
- This belongs to `main-menu-search-selection` unless the bug is inside a dedicated child built-in surface after route handoff.
- Actions presentation belongs to `actions-popups`; shortcut persistence belongs to `theme-config-preferences` or shortcut config code when storage is the failure.
- Screenshots are only needed when visual placement or paint is the asserted behavior.

## Related Features

- [002 File Search](../raw-oracle/002-file-search/answer.md)
- [005 Built-in Filterable Surfaces](../raw-oracle/005-built-in-filterable-surfaces/answer.md)
- [007 Root Unified Search Notes](../raw-oracle/007-root-notes/answer.md)
- [008 Root Clipboard History](../raw-oracle/008-root-clipboard-history/answer.md)
- [011 Root Source Actions](../raw-oracle/011-root-source-actions/answer.md)

## Raw Oracle References

- [Prompt](../raw-oracle/001-main-menu/prompt.md)
- [Bundle map](../raw-oracle/001-main-menu/bundle-map.md)
- [Full answer](../raw-oracle/001-main-menu/answer.md)
- [Full output log](../raw-oracle/001-main-menu/output.log)
- [Session metadata](../raw-oracle/001-main-menu/session.json)

## Open Questions And Gaps

- Full `src/render_script_list/mod.rs` key handling after the clipped bundle area should get a focused pass for exact Cmd+Enter, non-file Tab, action shortcut execution, and popup-first ordering.
- Exact source parser rows, qualifier value semantics, and grouped ranking implementation need a focused menu-syntax parser pass.
- Remove Shortcut live unregister/HUD behavior needs source expansion beyond the config write contract.
- Preview/info panel affordances and right-side shortcut glyph visual states need a renderer-focused pass.
- Exact action IDs/labels for script context actions need the full `src/actions/builders/script_context.rs` body.
- Agent Catalog or agent row shortcut assignment may need a dedicated feature slice.
- Menu-syntax mouse handling for row click, footer click, and mouse arm/disarm needs a focused popup-window pass.
