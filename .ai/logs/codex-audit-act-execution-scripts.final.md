# Audit: Script-context action execution

## Scope
- `src/execute_script/mod.rs`
- `src/execute_script/*.rs`
- Call-chain cross-checks in `src/actions/**`, `src/app_impl/**`, `src/app_actions/handle_action.rs`

## Trace summary
1. Script-context actions are defined in `get_script_context_actions()` (`src/actions/builders/script_context.rs:6`).
2. Actions are inserted into the actions dialog (`src/actions/dialog/part_02/part_02.rs:152`).
3. Choosing an action emits its `id` (`src/actions/dialog/part_02/part_03.rs:50`, `src/actions/dialog/part_02/part_03.rs:83`).
4. App routing forwards the selection as `ActionsRoute::Execute { action_id }` (`src/app_impl/actions_dialog.rs:83`, `src/app_impl/actions_dialog.rs:100`, `src/app_impl/actions_dialog.rs:162`).
5. Startup routing dispatches to `handle_action` for script-list hosts (`src/app_impl/startup.rs:1340`, `src/app_impl/startup.rs:1352`, `src/app_impl/startup.rs:1401`).
6. Execution is implemented by `handle_action` match arms (`src/app_actions/handle_action.rs`).

## Verification (1): every script-context action ID has an execution handler

Action IDs from `get_script_context_actions()`:
- `run_script`
- `update_shortcut`
- `remove_shortcut`
- `add_shortcut`
- `update_alias`
- `remove_alias`
- `add_alias`
- `edit_script`
- `view_logs`
- `reveal_in_finder`
- `copy_path`
- `copy_content`
- `edit_scriptlet`
- `reveal_scriptlet_in_finder`
- `copy_scriptlet_path`
- `copy_deeplink`
- `reset_ranking`

All IDs above are handled in `src/app_actions/handle_action.rs`:
- `run_script` (`:444`)
- `view_logs` (`:448`)
- `reveal_in_finder` (`:452`)
- `copy_path` (`:485`)
- `copy_deeplink` (`:563`)
- `add_shortcut`/`update_shortcut` (`:623`)
- `remove_shortcut` (`:689`)
- `add_alias`/`update_alias` (`:758`)
- `remove_alias` (`:806`)
- `edit_script` (`:873`)
- `edit_scriptlet` (`:1927`)
- `reveal_scriptlet_in_finder` (`:1959`)
- `copy_scriptlet_path` (`:1992`)
- `copy_content` (`:2083`)
- `reset_ranking` (`:2196`)

Result: **PASS**. No orphan script-context action IDs found.

## Verification (2): missing script / permission error handling

### `run_script` path
- `run_script` calls `execute_selected` (`src/app_actions/handle_action.rs:444`).
- Script selection dispatches to `execute_interactive` (`src/app_impl/selection_fallback.rs:172`).
- In `execute_interactive_merged`, launch failures are surfaced in-app via `last_output` + `cx.notify()`:
  - failed split/session setup (`src/execute_script/part_001_body/execute_interactive_merged.rs:56`)
  - execution start failure (`src/execute_script/part_001_body/execute_interactive_merged.rs:1540`)

### Other script-context actions
- `copy_content` handles read failures with explicit HUD error (`src/app_actions/handle_action.rs:2169`).
- `copy_path` / `copy_scriptlet_path` check script/scriptlet presence and show HUD errors (`src/app_actions/handle_action.rs:541`, `src/app_actions/handle_action.rs:2029`).
- `edit_script` / `edit_scriptlet` rely on `open::that_detached` and only log failures (`src/app_impl/shortcut_recorder.rs:15`); no user-facing HUD on failure.
- `reveal_in_finder` / `reveal_scriptlet_in_finder` execute reveal in detached async tasks; errors are logged only (`src/app_actions/handle_action.rs:17`, `src/app_actions/handle_action.rs:27`) while success HUD is shown immediately.

Result: **PARTIAL PASS**. Missing-script handling exists in key paths, but permission/open failures are not consistently surfaced to users.

## Verification (3): user feedback on action completion

- Positive completion HUD exists for most actions (copy, alias/shortcut updates, reset ranking, reveal actions).
- `run_script` feedback is output-panel based (interactive script output / launch error text), not HUD.
- **Feedback correctness gap:** reveal actions can display success HUD even when reveal fails asynchronously.
- **Feedback absence gap:** editor-launch failures for `edit_script` / `edit_scriptlet` are not surfaced to user.

Result: **PARTIAL PASS**.

## Notes on `src/execute_script/*` responsibilities

- `src/execute_script/*` handles interactive script execution protocol (stdin/stdout JSONL loop, command processing, session handling).
- Script-context actions like `edit_script`, `duplicate`-style operations, copy/reveal/reset are handled in `handle_action`, not in `src/execute_script/*`.
- `open_in_editor` is a path-context command (`src/app_impl/execution_paths.rs:220`), not a script-context action from `get_script_context_actions()`.

## Findings

1. **[Medium] Reveal actions show optimistic success before async reveal result**
   - Evidence: `reveal_in_finder` and `reveal_scriptlet_in_finder` trigger async reveal and immediately show success HUD (`src/app_actions/handle_action.rs:475`, `src/app_actions/handle_action.rs:1967`), while reveal failures are only logged (`src/app_actions/handle_action.rs:17-31`).
   - Impact: users can get incorrect completion feedback.

2. **[Medium] Editor-launch failures are log-only, no user-facing error**
   - Evidence: `edit_script` / `edit_scriptlet` call `open_script_in_editor`, which logs errors from `open::that_detached` without HUD notification (`src/app_impl/shortcut_recorder.rs:15-19`).
   - Impact: action appears to do nothing when open fails (permissions/app association issues).

3. **[Info] Script-context action coverage is complete**
   - Evidence: all IDs from `get_script_context_actions()` are matched in `handle_action`.

