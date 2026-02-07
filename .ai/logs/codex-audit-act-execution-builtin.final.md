# Audit: Built-in Action Execution Path

## Scope
- `src/app_impl/*`
- Related execution code: `src/app_execute/builtin_execution.rs`, `src/app_execute/builtin_confirmation.rs`

## Execution Path (built-ins)
1. Built-ins are materialized at startup via `builtins::get_builtin_entries(...)` in `src/app_impl/startup.rs:72` / `src/app_impl/startup_new_prelude.rs:56`.
2. They flow into search/grouped results via `self.builtin_entries` in `src/app_impl/filtering_cache.rs:61` and grouped result construction.
3. User selection reaches execution via:
- Main list selection: `src/app_impl/selection_fallback.rs:179` -> `self.execute_builtin(...)`
- Alias + trailing space: `src/app_impl/filter_input_change.rs:438` -> `self.execute_builtin(...)`
- Hotkey/deeplink/command-id execution: `src/app_impl/execution_scripts.rs:345` -> `self.execute_builtin(...)`
4. Actual dispatch is the feature match in `src/app_execute/builtin_execution.rs:122`.
5. Confirmation flow for dangerous commands:
- Confirmation open/check: `src/app_execute/builtin_execution.rs:26`
- Modal result handling: `src/main_sections/render_impl.rs:42` -> `src/app_execute/builtin_confirmation.rs:4`

## Findings (ordered by severity)

### 1. Hotkey/deeplink execution can incorrectly re-show the main window for built-ins that open AI/Notes windows
- Severity: High
- Evidence:
  - `NO_MAIN_WINDOW_BUILTINS` omits `builtin-open-ai` and `builtin-open-notes` in `src/app_impl/execution_scripts.rs:317`.
  - These IDs are valid built-ins in `src/builtins/part_001_entries/entries_002.rs:46` and `src/builtins/part_001_entries/entries_002.rs:10`.
  - `execute_by_command_id_or_path` returns `needs_main_window` from that list (`src/app_impl/execution_scripts.rs:355`), and callers use that to decide whether to show the main window (e.g. `src/hotkey_pollers.rs:273`).
- Impact:
  - Command-id flows (`builtin/<id>`) for `builtin-open-ai` / `builtin-open-notes` can reopen main UI even though those actions intentionally transition to secondary windows.

### 2. Confirmation modal open failure logs an error but provides no user-visible feedback
- Severity: Medium
- Evidence:
  - On `open_confirm_window(...)` failure, code only logs and skips execution (`src/app_execute/builtin_execution.rs:87`-`107`), with no toast/HUD.
- Impact:
  - User sees no action and no reason when a dangerous builtin cannot open its confirmation dialog.

### 3. Several AI command failure paths only log `Failed to open AI` without toast/HUD feedback
- Severity: Medium
- Evidence:
  - `SendScreenToAi` branch: `src/app_execute/builtin_execution.rs:634`-`636`
  - `SendFocusedWindowToAi` branch: `src/app_execute/builtin_execution.rs:677`-`679`
  - `SendSelectedTextToAi` branch: `src/app_execute/builtin_execution.rs:709`-`711`
  - `SendBrowserTabToAi` branch: `src/app_execute/builtin_execution.rs:752`-`754`
  - Preset placeholder branch: `src/app_execute/builtin_execution.rs:796`-`798`
- Impact:
  - Action appears to fail silently from the user perspective in those error cases.

### 4. Orphaned built-in command variants exist (defined and handled, but not constructible from built-in entries)
- Severity: Low
- Evidence:
  - Defined `AiCommandType` variants include: `ClearConversation`, `SendScreenAreaToAi`, `CreateAiPreset`, `ImportAiPresets`, `SearchAiPresets` (`src/builtins/part_000.rs:95`-`114`).
  - Built-in entries only construct `OpenAi`, `NewConversation`, `SendScreenToAi`, `SendFocusedWindowToAi`, `SendSelectedTextToAi`, `SendBrowserTabToAi` (`src/builtins/part_001_entries/entries_002.rs:45`-`123`).
  - Command-id execution only resolves IDs through `get_builtin_entries(...)` (`src/app_impl/execution_scripts.rs:348`).
  - `BuiltInFeature::AppLauncher` and `BuiltInFeature::App(String)` are also defined (`src/builtins/part_000.rs:189`-`191`) but not produced by current entry builders.
- Impact:
  - Some defined/handled actions are unreachable from built-in selection and command-id execution.

## Checkpoint Summary
- (1) Every `BuiltInFeature` variant has an execution match arm: **Pass** (exhaustive `match` in `src/app_execute/builtin_execution.rs:122`).
- (2) Error handling for failures: **Partial** (several failures are log-only).
- (3) User feedback on success/failure: **Partial** (many paths have toast/HUD, but gaps above).
- (4) No orphaned actions: **Fail** (unreachable defined variants and legacy feature variants).
