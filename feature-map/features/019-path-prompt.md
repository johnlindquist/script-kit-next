# 019 Path Prompt / path()

This chapter maps the SDK-backed file and directory selection prompt and its path-action boundary.

Raw Oracle reference: [answer](../raw-oracle/019-path-prompt/answer.md), [prompt](../raw-oracle/019-path-prompt/prompt.md), [bundle map](../raw-oracle/019-path-prompt/bundle-map.md), [full log](../raw-oracle/019-path-prompt/output.log), [session metadata](../raw-oracle/019-path-prompt/session.json).

## Executive Summary

`path(options?)` is the SDK-backed filesystem selection prompt. It is a compact prompt entity routed through `AppView::PathPrompt`, not the dedicated File Search surface.

The SDK API visible in the bundle is intentionally small:

```ts
interface PathOptions {
  startPath?: string
  hint?: string
}

function path(options?: PathOptions): Promise<string>
```

Runtime flow: the SDK sends `Message::Path { id, start_path, hint }`, Rust maps it to `PromptMessage::ShowPath`, builds `PathPrompt`, installs `AppView::PathPrompt`, focuses `FocusTarget::PathPrompt`, and resolves the SDK promise with either the selected path string or cancellation.

Path Prompt owns its footer. Native Run is labeled `Select`, dispatches to `PathPrompt::handle_enter`, and must not fall through to launcher selection/execution. The footer keeps `Cmd+K Actions` and omits launcher AI unless a path-specific route is intentionally designed.

Path actions are typed through `PathAction`, not stringly matched. Known ids include `select_file`, `open_directory`, `copy_path`, `copy_filename`, `open_in_finder`, `open_in_editor`, `open_in_quick_terminal`, and `move_to_trash`.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Open a path picker. | `await path()`. | Opens `AppView::PathPrompt`, rooted at home by default. |
| Start in a directory. | `await path({ startPath: "/tmp" })`. | Opens at the requested start path. |
| Show prompt hint copy. | `await path({ hint: "Select a file" })`. | Hint is carried to `PathPrompt`; exact rendering needs source proof. |
| Browse rows. | Arrow keys/click. | Moves selected path entry. |
| Navigate into directory. | Right arrow or Enter on directory path depending route. | Updates current directory and reloads entries. |
| Navigate to parent. | Left arrow. | Moves up one directory. |
| Filter entries. | Type printable characters. | Updates prompt input/filter and visible rows. |
| Submit selected path. | Enter or footer Select. | Resolves SDK promise with selected path. |
| Submit typed fallback. | Non-empty filter with no selected row. | Resolves SDK promise with typed filter text as path. |
| Open path actions. | Cmd+K or footer Actions. | Opens action menu for selected path. |
| Inspect with automation. | `getState`, `getElements`, `simulateKey`. | Agents can identify path prompt, rows, current directory, and filter. |
| Prove filesystem edge state. | `getState.path`, `getElements` `path-status`. | Agents can distinguish missing, non-directory, permission-denied, empty, filtered-empty, hidden-dotfile policy, and symlink rows. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `path()` SDK API | Script-facing async filesystem picker. | Sends id, `startPath`, and `hint`; resolves one string. |
| `PathPrompt` entity | Rust prompt state for browsing a directory. | Owns current path, entries, filtered rows, selected index, and prompt input. |
| `AppView::PathPrompt` | Active app view for SDK path prompt. | Separate from `AppView::FileSearchView`. |
| Current directory | Directory whose entries are shown. | Starts from `startPath`, home dir, or `/` fallback. |
| Path entries | Directory/file rows. | Sorted directories first, then files, alphabetically. |
| Load status | Prompt-owned filesystem edge receipt. | Stable kinds cover ready, empty, filtered-empty, missing, non-directory, permission-denied, and read-error states. |
| Filter input | Typed query in the prompt. | Filters visible rows; exact matching algorithm was not in the tight bundle. |
| Path actions | Selected-path command menu. | Uses typed `PathAction` ids and path-scoped action dialog. |
| Footer ownership | Native footer for PathPrompt. | `Run` is labeled `Select`; `Actions` maps to Cmd+K. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `globalThis.path` in `scripts/kit-sdk.ts`. | Script calls `path(options?)`. | Generates id, sends path prompt message, resolves selected path string. |
| `Message::Path`. | Protocol message from SDK. | Converts to `PromptMessage::ShowPath`. |
| `PromptMessage::ShowPath`. | Rust prompt handler. | Creates `PathPrompt`, subscribes to events, installs app view, sets focus. |
| `render_path_prompt`. | App view render dispatch. | Renders PathPrompt wrapper and action overlay handling. |
| `PathPrompt::handle_enter`. | User/automation submit. | Selects row path or typed fallback; emits submit callback. |
| Native footer. | Main-window footer. | Run dispatches to PathPrompt select; Actions opens/toggles path actions. |
| `PathAction`. | Path action dispatcher. | Parses action ids and prevents typo-only silent no-op actions. |
| `collect_elements`. | Protocol element collection. | Exposes current directory, filter, entry list, and row choices. |
| `simulateKey`. | Protocol key driving. | Routes arrows, Enter, Escape, left/right, and Cmd+K to PathPrompt. |

## User Workflows

### Basic Selection

A script calls:

```ts
const selected = await path()
```

The SDK creates a prompt id, sends a path message, and waits. Rust creates `PathPrompt` rooted at home unless `startPath` is present, installs `AppView::PathPrompt`, sets `FocusTarget::PathPrompt`, and renders directory/file rows. The user moves the selected row and presses Enter or footer Select. The SDK resolves with the selected filesystem path.

### Start Path

```ts
const selected = await path({ startPath: "/tmp" })
```

`startPath` crosses the SDK boundary as `start_path` and initializes the prompt current directory. Missing paths, non-directory starts, permission-denied directories, empty directories, and read errors stay on the PathPrompt surface with stable status kinds and copy instead of silently becoming indistinguishable empty lists.

### Filter And Submit

Typing updates the prompt input/filter. Filtered entries are exposed through `getElements`. If a visible row is selected, Enter submits that row path. If no selected row exists and the filter is non-empty, `handle_enter` can submit the typed filter text as the path.

### Directory Navigation

Users can navigate into selected directories and up to the parent directory. Simulated right/left keys route to PathPrompt navigation in the captured source. Directory rows are identified distinctly from file rows in element collection. Symlink rows are included and marked with `isSymlink:true` in state receipts and `kind:"symlink"` in element receipts; symlink directory targets remain navigable because directory detection follows the target.

### Path Actions

Cmd+K emits a `PathPromptEvent::ShowActions(PathInfo)` for the selected row. The wrapper creates an actions dialog for that path, hides search, matches the main-window background, marks the popup open, and synchronizes shared action state. Escape closes actions before canceling the prompt.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open path prompt. | `path(options?)`. | `AppView::PathPrompt`. | SDK call. | `globalThis.path` -> `Message::Path` -> `ShowPath` -> `PathPrompt::new`. | Path picker opens. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `src/prompts/path/prompt.rs`. |
| Move selection. | Active PathPrompt. | Selected row. | Up/Down. | `simulateKey`/render key path -> prompt selection methods. | Selected index changes. | `src/main_entry/runtime_stdin_match_simulate_key.rs`, `src/prompts/path/prompt.rs`. |
| Navigate into directory. | Selected directory. | Directory row selected. | Right/Enter route. | PathPrompt navigation updates current path and reloads entries. | Directory contents shown. | `src/prompts/path/prompt.rs`. |
| Navigate parent. | Active PathPrompt. | Current directory. | Left. | PathPrompt parent navigation. | Parent directory shown. | `src/main_entry/runtime_stdin_match_simulate_key.rs`, `src/prompts/path/prompt.rs`. |
| Filter rows. | Active PathPrompt. | Filter input. | Type/backspace. | `handle_char`, `handle_backspace`, `set_input`, `update_filtered`. | Visible rows change. | `src/prompts/path/prompt.rs`, `src/prompts/path/render.rs`. |
| Submit selected path. | Active PathPrompt. | Row selected. | Enter/footer Select. | `PathPrompt::handle_enter`. | SDK resolves selected path. | `src/prompts/path/prompt.rs`, `src/app_impl/ui_window.rs`. |
| Submit typed path fallback. | Active PathPrompt. | No row selected, filter non-empty. | Enter. | `handle_enter` fallback branch. | SDK resolves typed text. | `src/prompts/path/prompt.rs`. |
| Open path actions. | Active PathPrompt. | Row selected. | Cmd+K/footer Actions. | `PathPromptEvent::ShowActions` -> `ActionsDialog::with_path`. | Path action popup opens. | `src/render_prompts/path.rs`, `src/app_impl/path_action.rs`. |
| Close path actions. | Path action popup. | Popup open. | Escape/backdrop route. | Shared action key helper closes popup first. | Focus returns to PathPrompt. | `src/render_prompts/path.rs`. |
| Inspect state. | Protocol. | Active PathPrompt. | `getState`. | Prompt state mapping. | Prompt type `path`, active id, path status, counts, selected row. | `src/prompt_handler/mod.rs`. |
| Inspect rows. | Protocol. | Active PathPrompt. | `getElements`. | PathPrompt element collector. | Current directory, filter, entries, selected row, path-status row. | `src/app_layout/collect_elements.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK idle. | No path prompt active. | Script continues. | No pending path id. |
| Path request created. | `path(options?)`. | Create id, send path message. | Options are `startPath` and `hint` in visible SDK. |
| Path route handled. | `ShowPath`. | Build `PathPrompt`, subscribe to events, install app view. | Focus moves to PathPrompt. |
| Entries loaded. | Prompt construction or directory change. | Current entries copied to filtered entries and render rows. | Dirs first, files second. |
| Browsing. | User moves/filters/navigates. | Selected index/current path/filter mutate. | Element receipts should reflect each state. |
| Actions open. | Cmd+K. | Path action dialog opens for selected path. | Escape should close actions first. |
| Submit. | Enter/footer Select. | Selected row or typed fallback submitted. | SDK resolves string. |
| Cancel. | Escape with no popup. | Submit callback receives cancellation. | Exact SDK cancellation value needs direct test. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| PathPrompt active. | Compact path picker with directory/file rows. | `FocusTarget::PathPrompt`. | Prompt type `path`; native footer path prompt. |
| Filtered. | Filter text visible and row list narrowed. | PathPrompt. | `path-filter` element value. |
| Directory row selected. | Row marked selected, directory identity. | PathPrompt. | Row choice with directory flag/trailing slash. |
| File row selected. | Row marked selected, file identity. | PathPrompt. | Row choice path/name. |
| Actions open. | Path actions popup. | Actions dialog. | Shared path action state; popup contract. |
| Empty/no matches. | Stable empty or no-match copy. | PathPrompt. | `getState.path.status.kind` and `getElements` `path-status.statusKind`. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Printable text. | PathPrompt. | Appends to filter input. |
| Backspace. | PathPrompt. | Removes filter character. |
| Up/Down. | PathPrompt. | Moves selected row. |
| Right. | Selected directory. | Navigates into directory. |
| Left. | PathPrompt. | Navigates to parent directory. |
| Enter. | Selected row. | Submits selected path. |
| Enter. | No selection and filter non-empty. | Submits typed filter text. |
| Escape. | Actions popup open. | Closes actions first. |
| Escape. | No popup. | Cancels path prompt. |
| Cmd+K. | PathPrompt. | Opens/toggles path actions. |

## Actions And Menus

| Action id | Meaning |
|---|---|
| `select_file`. | Select the path and resolve prompt. |
| `open_directory`. | Open/navigate directory action. |
| `copy_path`. | Copy full path. |
| `copy_filename`. | Copy basename. |
| `open_in_finder`. | Reveal/open in Finder. |
| `open_in_editor`. | Open path in configured editor. |
| `open_in_quick_terminal`. | Open Quick Terminal at path location. |
| `move_to_trash`. | Destructive trash action; confirmation boundary needs proof in full source/tests. |

The typed parser strips optional `file:` prefixes and rejects unknown ids. Keep actions in `PathAction::ALL` and tests in sync when adding or renaming actions.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getState`. | Reports prompt type `path`, active prompt id, and `path` payload with current path, filter, counts, selected metadata, and load status. |
| `getElements`. | Exposes `path-current-directory`, `path-filter`, `path-entries`, `path-status`, and per-row choices with file/directory/symlink kind. |
| simulateKey Up/Down. | Moves selected row. |
| simulateKey Right/Left. | Navigates into selected directory or up to parent. |
| simulateKey Enter. | Submits selected path or typed fallback. |
| simulateKey Cmd+K. | Opens path action popup for selected row. |
| simulateKey Escape. | Cancels prompt unless action popup intercepts first. |
| ForceSubmit. | Not fully proven for PathPrompt in the tight bundle; verify before relying on it. |

## Data, Storage, And Privacy Boundaries

- PathPrompt exposes filesystem names and full paths in UI, element receipts, and `stateResult.path`; use temporary fixture paths for proof.
- Logs use path-ending copy for PathPrompt load/navigation events rather than writing the full directory path.
- Hidden dotfiles are intentionally skipped and reported through `hiddenPolicy:"omitDotfiles"` and `hiddenCount`.
- Screenshot tests can capture private path names; prefer temporary fixture directories.
- Path action payloads carry full selected paths.
- Destructive actions such as `move_to_trash` require confirmation/registry teardown proof before changing behavior.
- Clipboard actions intentionally put path data on the system clipboard.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| Missing/invalid `startPath`. | `stateResult.path.status.kind:"missing"` with "Path not found.". |
| Permission denied. | `stateResult.path.status.kind:"permission_denied"` with "Permission denied.". |
| Non-directory `startPath`. | Opens the parent directory and preselects the file when possible; direct non-directory loads report `not_directory` with "Path is not a folder.". |
| Empty directory. | `stateResult.path.status.kind:"empty"` with "This folder is empty.". |
| No filter matches. | `stateResult.path.status.kind:"filtered_empty"` with "No matching files or folders."; typed fallback submit may apply. |
| Hidden files. | Dotfiles are skipped by policy and counted in receipts. |
| Unsupported SDK options. | TypeScript exposes only `startPath` and `hint`; extra keys likely ignored but not proven. |
| Action popup open. | Escape should close popup before canceling prompt. |
| `move_to_trash`. | Typed action exists; confirmation details need direct source/test proof. |
| Loading. | No separate async loading state is proven in the tight bundle. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK API. | `scripts/kit-sdk.ts` owns `path()`, `PathOptions`, id generation, and response resolution. |
| Prompt routing. | `src/prompt_handler/mod.rs` owns `Message::Path`, `ShowPath`, app-view installation, focus, and state reporting. |
| PathPrompt state. | `src/prompts/path/prompt.rs` and `src/prompts/path/types.rs` own current path, entries, filtering, selection, navigation, submit/cancel, and events. |
| PathPrompt rendering. | `src/prompts/path/render.rs` owns inner GPUI rendering and key handling. |
| Path wrapper/actions. | `src/render_prompts/path.rs` owns action popup overlay, shared action state, and wrapper focus behavior. |
| Path actions. | `src/app_impl/path_action.rs` owns typed action ids and parser tests. |
| Footer. | `src/app_impl/ui_window.rs` owns native footer labels and PathPrompt Run/Actions dispatch. |
| Automation. | `src/app_layout/collect_elements.rs` and `src/main_entry/runtime_stdin_match_simulate_key.rs` own element receipts and simulated keys. |
| File Search boundary. | `src/render_builtins/file_search*.rs`, `src/app_impl/root_file_search.rs`, and file-search tests own the separate File Search product. |

## Invariants And Regression Risks

- `path()` must install `AppView::PathPrompt`, not `FileSearchView`.
- PathPrompt primary footer label must be Select, not launcher Run.
- Native footer Run must dispatch to `PathPrompt::handle_enter`, never launcher selection.
- `Cmd+K Actions` must remain path-scoped and close before prompt cancellation.
- Unknown path action ids must fail at parse time, not become silent no-op branches.
- Dirs-first/files-second sorting is user-visible and automation-visible.
- Do not claim File Search preview, drag-out, attachment portals, or `~` trigger as SDK `path()` behavior.
- Treat full paths as private in logs and screenshots; path prompt automation receipts intentionally carry full paths, so use temporary fixtures for proof.
- Resolve the sizing ambiguity before changing layout: `ShowPath` uses `ViewType::ScriptList` in one path while sizing calculation maps PathPrompt to `ViewType::DivPrompt`.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| SDK smoke. | `path()` opens PathPrompt; `getState` reports `path`; Enter resolves a selected path. |
| Key events smoke. | `tests/smoke/test-path-key-events.ts` drives Down, Up, Enter, Cmd+K, and Escape. |
| Visual consistency. | Path prompt screenshot shows current directory, filter area, rows, Select footer, and Actions footer. |
| PathAction parser tests. | Every variant round-trips; `file:` prefix strips; unknown ids rejected; ids are unique snake_case. |
| Render wrapper tests. | Shared actions state updates only when changed; shared key helpers route Up/Down/Enter/Escape; dialog matches main-window background. |
| `getElements` receipt. | Current directory, filter text, entries list, row choices, selected row, and directory/file identity are visible. |
| Filesystem edge receipts. | `cargo test --lib prompts::path::prompt::tests:: -- --nocapture` and `cargo test --test path_prompt_filesystem_edges_contract -- --nocapture` prove stable missing, non-directory, permission-denied, empty, hidden-dotfile, symlink, state, and element contracts. |
| simulateKey receipt. | Right/Left navigate directories; Cmd+K opens actions; Escape closes actions or cancels prompt. |
| Footer regression. | Native footer has Select and Actions; Select dispatches `PathPrompt::handle_enter`. |
| File Search boundary. | `path()` never enters `FileSearchView`; File Search preview/drag/portal checks are excluded. |

## Agent Notes

When working on Path Prompt, start with `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `src/prompts/path/*`, `src/render_prompts/path.rs`, `src/app_impl/path_action.rs`, `src/app_impl/ui_window.rs`, `src/app_layout/collect_elements.rs`, and `src/main_entry/runtime_stdin_match_simulate_key.rs`.

Do not make File Search assumptions while editing PathPrompt. Dedicated File Search owns mini/full file browser modes, preview pane, thumbnail loading, drag-out rows, attachment portal behavior, and `~` launcher handoff.

When adding a path action, add the enum variant, canonical action id, `PathAction::ALL`, execution behavior, action menu definitions, tests, and docs/atlas updates.

When changing footer semantics, verify native footer and fallback GPUI hints together. PathPrompt Select is a product contract, not cosmetic copy.

Use temporary directories for runtime tests so receipts and screenshots do not expose private paths.

## Related Features

| Feature | Relationship |
|---|---|
| [002 File Search / Browser / Attachment Portals](./002-file-search.md). | Related filesystem browsing, but separate view with preview, drag-out, and portals. |
| [013 ScriptList Special Entry Triggers](./013-scriptlist-special-entry-triggers.md). | `~` opens Mini File Search, not SDK `path()`. |
| [016 Prompt Runtime Core](./016-prompt-runtime-core.md). | Shares prompt id/focus/submission concepts. |
| [017 Form and Fields Prompt](./017-form-fields-prompt.md). | Adjacent prompt entity, but different data model. |
| Quick Terminal. | Path actions can open Quick Terminal at selected path location. |

## Open Questions And Gaps

- Exact filter matching algorithm was not visible.
- `hint` rendering is not proven, though the value is carried and stored.
- Path action execution side effects are not fully visible, only typed ids/parser.
- `move_to_trash` confirmation behavior needs direct test/source proof.
- Window sizing has conflicting proof between `ViewType::ScriptList` and `ViewType::DivPrompt` paths.
- ForceSubmit behavior for PathPrompt is not fully proven.
- Direct protocol row selection by index/value was not shown; simulateKey navigation is proven.
- Runtime behavior for extra JS option keys is not shown.
