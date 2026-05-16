# 002 File Search / Root Files / Directory Browse / File Actions


## Executive Summary


The main risks are active-frame instability, stale source-filter decorations, wrong host ownership for actions, file action target reuse, and portal return mistakes. Use state receipts and source audits before screenshots.

## What Users Can Do

- Type `~`, `~/...`, or absolute paths to enter mini/dedicated File Search.
- Open full File Search from a built-in command or root continuation row.
- Search files through Spotlight first, with bounded filesystem fallback when dedicated File Search has no Spotlight rows.
- Browse directories inline with `Tab`, `Shift+Tab`, double-click, and path fragments.
- Show dotfiles by typing dot-prefixed directory fragments.
- Preview selected files in the full split view, including guarded image thumbnails.
- Open selected files or directories with Enter.
- Use Cmd+K actions for open, reveal, copy path/name, Quick Look, attach, sort, and current-directory actions.
- Use direct root-file shortcuts such as Cmd+Y, Cmd+Shift+F, and Cmd+Shift+C.
- Drag dedicated File Search rows to Finder or other native apps.
- Attach a file to ACP or open ACP Explain/Plan flows from File Search context.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Dedicated File Search | Routed file browser surface with list, optional preview, actions, drag-out, and portal behavior. | `src/render_builtins/file_search.rs`, `src/file_search/` |
| Mini File Search | Compact file-search presentation entered from `~` / path handoff. | `src/render_builtins/file_search.rs` |
| Full File Search | Expanded split list-plus-preview browser. | `src/render_builtins/file_search.rs`, `src/render_builtins/file_search_preview.rs` |
| Root Files | Passive ScriptList section for eligible filename/path queries. | `src/app_impl/root_file_search.rs`, `src/file_search/mod.rs` |
| Root Recent Files | Frecency-backed provider-free root rows seeded after successful opens. | `src/file_search/mod.rs`, `src/scripts/grouping.rs` |
| Root directory browse | Direct-child root Files mode for explicit directory path queries. | `src/file_search/directory.rs`, `src/app_impl/root_file_search.rs` |
| File action subject | Captured selected file path for actions and direct shortcuts. | `src/app_actions/handle_action/files.rs`, `src/app_impl/root_unified_result_actions.rs` |
| Attachment portal | ACP-owned return flow that asks File Search for a file/context target. | `src/app_impl/attachment_portal.rs` |

## Entry Points

| Entry point | User input | Result |
|---|---|---|
| Main menu path handoff | `~`, `~/dev`, `/tmp` | Mini File Search or root directory browse with stale menu decorations cleared. |
| File Search built-in | Built-in command / triggerBuiltin | Full dedicated File Search. |
| Root continuation row | `Search Files for "<query>"` | Dedicated File Search for query. |
| Root directory continuation | `Open File Search in "<folder>"` | Dedicated File Search scoped to folder. |
| Root file row | Enter, Cmd+K, direct shortcut | OS open or root file action. |
| Dedicated selected row | Enter, Tab, Shift+Tab, Cmd+K, drag | Open, browse, action, or native drag. |
| ACP portal | File attachment request | File Search opens as attachment portal and returns selected file part. |

## User Workflows

### Enter Mini File Search From The Launcher

The user types `~` or a home/absolute path. The main menu hands off to File Search and clears any menu-syntax/source-chip decorations before first paint. Directory rows should be seeded before async replacement so the surface does not flash blank.

### Browse A Directory In Dedicated File Search

The user selects a directory and presses Tab or double-clicks. Dedicated File Search opens that directory inline. The previous rows remain visible until the directory stream completes and then one stable replacement batch lands. Shift+Tab moves to the parent. Plain Enter still OS-opens the selected item, including directories.

### Search A Filename From Root

The user types an eligible root query. Root Files appears as a passive section below primary commands and before fallback rows. A loading header and continuation row are stable while the provider warms a cache. Provider completion must not mutate the active visible frame for the same filter text.



### Run File Actions

Root file rows and dedicated File Search rows both support file actions, but through different hosts. Root rows open MainList root actions with captured `RootUnifiedActionSubject`. Dedicated rows open FileSearch-hosted actions based on `file_search_actions_path`. Action execution must not reuse stale targets after the selected row changes or after popup close.

### Attach A File To ACP

From an ACP attachment flow or dedicated File Search action, File Search returns a file context part to the originating ACP composer. Escape in portal mode cancels the portal and returns to ACP; Enter accepts the selected file instead of OS-opening it.

### Drag A File Out

The user drags a dedicated File Search row. The renderer starts a native AppKit file drag, logs handoff success/failure, then defers GPUI active-drag cleanup until after GPUI stores the row drag payload. Header re-entry restores focus and clears stale drag/hover state.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open mini File Search | Main filter | ScriptList | `~` / path | File Search handoff | Mini file browser | No stale source chips |
| Open full File Search | Built-in | Full route | Enter / triggerBuiltin | Built-in execution | Split browser | `surfaceContract` full file search |
| Browse directory dedicated | File Search row | Dedicated | Tab / double-click | File Search key/row handler | Child directory rows | Directory-nav smoke/source tests |
| Move parent dedicated | File Search | Dedicated | Shift+Tab | File Search parent nav | Parent rows | Directory-nav proof |
| Open selected item | File Search | Dedicated | Enter | `open_file` / `open_directory` | OS opens item | Open success receipt/HUD |
| Show hidden files | File Search | Directory query | Dot fragment | Directory source with hidden mode | Dotfiles visible | Hidden-file source tests |
| Preview selected file | Full File Search | Split view | Selection move | Preview/thumbnail loader | Preview updates | Thumbnail state/logs |
| Open root file | ScriptList | Root file row | Enter | Shared root open helper | OS opens file and records frecency | Root file open source audit |
| Browse root directory inline | ScriptList | Root directory row | Tab | Root directory browse | Filter rewrites to folder | Root directory proof |
| Move root parent | ScriptList | Root directory browse | Shift+Tab | Root directory navigation | Parent query | Source audit |
| Open continuation | ScriptList | Continuation row | Enter | Stable fallback row | Dedicated File Search | Fallback stable key proof |
| Expand source page | ScriptList | Files source filter | Arrow near bottom | Source-chip pagination | More rows, selected visible | `mainListScroll` receipt |
| Open root actions | ScriptList | Root file selected | Cmd+K | MainList root action owner | Captured actions dialog | `actionsDialog.contextStableKey` |
| Root Quick Look | ScriptList | Root file selected | Cmd+Y | Root file action executor | Quick Look or controlled error | Quick Look tests |
| Dedicated actions | File Search | Dedicated row selected | Cmd+K | `toggle_file_search_actions` | FileSearch actions popup | `activePopupContract` |
| Attach file | Portal/File Search | Portal mode | Enter / action | Attachment portal | ACP context part attached | ACP pending context receipt |
| Cancel portal | File Search portal | Portal mode | Escape | `close_attachment_portal_cancel` | Return to ACP | Portal cancel receipt |
| Native drag | Dedicated row | File Search | Drag | `begin_file_search_native_drag` | External app receives file | Drag logs/source tests |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Dedicated loading | Query starts with no cached rows | Results/cached rows | Six-row skeleton preserves columns. |
| Dedicated cached rows | Existing directory rows while stream starts | Stable replacement batch | Prevents blank flash during Tab navigation. |
| Dedicated directory browse | Selected directory Tab/double-click | Child/parent/open/action | Enter remains OS open, not browse. |
| Dedicated hidden disabled | Normal directory listing | Dot-prefixed filter | Dotfiles hidden by default. |
| Dedicated hidden enabled | Dot-prefixed fragment | Normal fragment/parent | Hidden mode participates in cache/source key. |
| Dedicated preview | Full presentation | Selection change/close | Thumbnail loads are size/dimension/format guarded. |
| Dedicated portal | ACP attachment request | Attach/cancel | Enter attaches; Escape cancels, not OS-open. |
| Root empty Recent Files | Empty ScriptList | Query/source/filter | Provider-free, capped, app-bundle internals suppressed. |
| Root global Files loading | Eligible uncached root query | Future frame/query change | Same-query provider completion cannot alter active rows. |
| Root global cached | Frame built after warm cache | Query change/open/action | Cached rows eligible at frame build only. |
| Root directory browse | Explicit directory path | Tab/Shift+Tab/continuation | Direct children only; no recursive/global search. |
| Root actions | Cmd+K on root file | Execute/close | Captured subject survives selection/focus changes. |

## Visual And Focus States


## Keystrokes And Commands

| Key | Context | Behavior |
|---|---|---|
| `~` / path text | Main menu | File Search handoff or root directory browse. |
| Character input | Dedicated File Search | Search, directory fragment, or hidden-file restream. |
| Up/Down | Dedicated File Search | Move selected file row. |
| Enter | Dedicated selected file/dir | OS open selected item; portal mode attaches instead. |
| Tab | Dedicated selected directory | Browse into directory inline. |
| Shift+Tab | Dedicated File Search | Browse parent directory. |
| Cmd+K | Dedicated File Search | Open FileSearch-hosted actions. |
| Cmd+K | Root file row | Open MainList root actions. |
| Escape | Portal mode | Cancel portal and return to ACP. |
| Cmd+Y | Root file row | Quick Look through root action executor. |
| Cmd+Y | Dedicated file row | Quick Look if supported; directories excluded. |
| Cmd+Shift+F | Root file row | Reveal in Finder. |
| Cmd+Shift+C | Root file row | Copy full path. |
| Cmd+Enter | Dedicated File Search | ACP Explain route with selected context. |
| Cmd+Shift+Enter | Dedicated File Search | ACP Plan route with selected context. |
| Drag | Dedicated row | Native file drag-out. |

## Actions And Menus

| Action | Root row | Dedicated File Search | Notes |
|---|---|---|---|
| Open | Yes | Yes | Root frecency records only after successful OS open. |
| Reveal in Finder | Yes | Yes | Root direct shortcut is Cmd+Shift+F. |
| Copy full path | Yes | Yes | Root direct shortcut is Cmd+Shift+C. |
| Copy name | Yes | Yes | Copies basename. |
| Quick Look | Yes | Files only in dedicated | Root/dedicated both use controlled OS helper. |
| Search Inside Folder | Directories only | Directory browse path | Root opens dedicated File Search scoped to folder. |
| Browse Parent Folder | Regular files only | Parent navigation | Root clears stale MainList selection and shortens home display. |
| Attach to AI | Adjacent/context action | Yes | Failure HUD should prefix "Failed to attach". |
| Sort current directory | No | Current directory actions | Modified-time sort compares folders/files together. |
| Delete/mutation | Not fully mapped | Dedicated action path | Needs focused mutation-refresh pass. |

## Automation And Protocol Surface

| Receipt | What it proves |
|---|---|
| `surfaceContract` | Mini/full File Search surface and proof/visual policies. |
| `semanticSurface=fileSearch` | Routed File Search identity after entry. |
| `mainWindowPreflight.visibleResults` | Root file/passive roles, stable keys, action kinds, source names. |
| `getElements` source status | Source-filter status is metadata, not selectable row. |
| `mainListScroll` | Source-chip lazy page selected row remains above footer. |
| `actionsDialog` | Captured root or FileSearch action context. |
| File Search logs | Loading/cached/display selected rows, thumbnail state, drag handoff, action failure. |

## Data, Storage, And Privacy Boundaries

- Root Files reads metadata needed for row display and OS open, not file contents.
- Root Recent Files are frecency-backed and seeded only after successful opens.
- Root global Files suppress `.app` bundles and nested `.app` internals; explicit directory browse can show them intentionally.
- Browser/source filter status is display metadata, not executable row data.
- Attachment flows should return a stable file context part to ACP, not leak arbitrary file content in launcher receipts.
- Quick Look uses OS helpers and must report missing-path or launch failures without blocking or panicking.
- Native drag-out hands file paths to AppKit; failures log warnings and leave GPUI state clean.

## Error, Empty, Loading, And Disabled States

- Dedicated loading shows six skeleton rows, not a collapsing spinner.
- Dedicated no-results/help copy stays bounded inside the results pane.
- Spotlight no-results in dedicated search can fall back to bounded filesystem scan.
- Root global uncached query keeps stable loading/continuation rows while providers warm cache.
- Ineligible root queries, advanced queries, and noisy short queries do not start global root file search.
- Source status rows for capped/loading/empty/exhausted states are not selectable.
- Quick Look missing path returns controlled error/HUD.
- File action failures log action/path/error and clear action targets appropriately.
- Platform native drag failure logs warning and avoids stale internal drag state.

## Code Ownership

| Behavior | Owner files/tests |
|---|---|
| Dedicated File Search rendering/key handling | `src/render_builtins/file_search.rs` |
| File search query/ranking/types | `src/file_search/mod.rs` |
| Directory listing/browse | `src/file_search/directory.rs` |
| Spotlight provider | `src/file_search/mdfind.rs` |
| OS open/reveal/Quick Look | `src/file_search/os_open.rs` |
| File Search layout/list/preview helpers | `src/render_builtins/file_search_layout.rs`, `src/render_builtins/file_search_list.rs`, `src/render_builtins/file_search_preview.rs` |
| Root Files | `src/app_impl/root_file_search.rs` |
| File actions | `src/app_actions/handle_action/files.rs`, `src/actions/builders/file_path.rs` |
| Root file actions | `src/app_impl/root_unified_result_actions.rs` |
| Attachment portal | `src/app_impl/attachment_portal.rs` |
| Simulated key parity | `src/main_entry/runtime_stdin_match_simulate_key.rs` |
| Contract tests | `tests/file_search_tilde_entry.rs`, `tests/file_search_ai_routing.rs`, `tests/file_search_drag_and_verbs.rs`, `tests/file_search_mutation_refresh.rs`, `tests/source_audits/root_file_search_contract.rs`, `tests/source_audits/root_unified_source_actions_contract.rs` |

## Invariants And Regression Risks

- Root Files is passive and must not displace primary command/script/app/window intent unless the explicit promotion policy allows it.
- Root provider completion must not mutate visible rows for the same filter text.
- Root directory browse is intentional direct-child browsing, not recursive global search.
- Source status must never become an executable row or affect list count/sizing/scroll.
- Dedicated File Search owns Tab/Shift+Tab directory navigation; root files use separate inline directory browse rules.
- Plain Enter in dedicated File Search OS-opens selected items except when in attachment portal mode.
- Actions execute against captured file path/subject, not the current selection after popup drift.
- Dedicated and MainList actions must preserve their host semantic surface.
- Quick Look must be non-blocking and controlled on missing paths.
- Native drag-out must clean GPUI active-drag and hover/focus state after AppKit handoff.
- Attachment portal Escape cancels the portal before treating Escape as generic File Search back/close.

## Verification Recipes


```bash
cargo test --test file_search_tilde_entry
cargo test --test file_search_ai_routing
cargo test --test file_search_drag_and_verbs
cargo test --test file_search_mutation_refresh
cargo test --test source_audits root_file_search_contract -- --nocapture
cargo test --test source_audits root_unified_source_actions_contract -- --nocapture
cargo test --test source_audits shortcut_alias_file_actions -- --nocapture
cargo check --lib
cargo fmt --check
git diff --check
source checks
```


```bash
bun scripts/agentic/root-search-frame-stability.ts
bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000
bun scripts/agentic/root-source-filter-lazy-scroll.ts --query s --timeout 20000
bun scripts/agentic/root-source-actions-matrix.ts
bun tests/smoke/test-file-search-actions.ts
bun tests/smoke/test-file-search-directory-nav.ts
```


- `~` enters File Search without stale source-chip decorations.
- Dedicated File Search keeps rows visible during directory stream replacement.
- Root provider warming does not change selected stable key for the active query.
- Root file actions capture the file subject and direct shortcuts route through the same executor.
- File Search actions preserve `semanticSurface=fileSearch`.
- Portal mode Enter attaches and Escape cancels.
- Native drag logs handoff and cleans active drag state.

Screenshots are only needed for visual acceptance of mini/full layout, preview pane, thumbnail rendering, row chrome, or popup placement.

## Agent Notes

- Do not merge root Files and dedicated File Search behavior. Root Files is a passive ScriptList section; dedicated File Search is a routed browser.
- To verify root behavior, prefer `mainWindowPreflight`, `filterInputDecorations`, source status receipts, and `mainListScroll`.
- To verify dedicated behavior, inspect File Search surface state, row elements, action host, and logs before screenshots.
- If a file action hits the wrong path, inspect captured action target state before checking row labels.
- If a selected file disappears below the footer during source-chip pagination, inspect `mainListScroll` and deferred reveal logic.
- If File Search hides or shows dotfiles incorrectly, inspect hidden-mode source keys and dot-fragment parsing.
- This belongs to `file-search-portals` unless the bug is purely root grouping, generic actions popup lifecycle, or ACP inline token replacement.
- Screenshots are only needed when visual layout or thumbnail rendering is the behavior under test.

## Related Features

- [001 Main Menu](./001-main-menu.md)
- [003 Agent Chat Context Composer](../raw-oracle/003-agent-chat-context/answer.md)
- [011 Root Source Actions](../raw-oracle/011-root-source-actions/answer.md)

## Raw Oracle References

- [Prompt](../raw-oracle/002-file-search/prompt.md)
- [Bundle map](../raw-oracle/002-file-search/bundle-map.md)
- [Full answer](../raw-oracle/002-file-search/answer.md)
- [Full output log](../raw-oracle/002-file-search/output.log)
- [Session metadata](../raw-oracle/002-file-search/session.json)

## Open Questions And Gaps

- Exact dedicated File Search `getState` / `getElements` field names should be mapped from runtime receipts.
- Dedicated file action catalog strings for reveal/copy/open-in-Finder/open-in-editor/open-in-Quick-Terminal/delete need a focused action-builder pass.
- Mutation action IDs and refresh receipt fields need `tests/file_search_mutation_refresh.rs` or full action handler context.
- File attachment context URI/token schema and portal return receipts need full `src/app_impl/attachment_portal.rs` plus ACP composer context.
- Exact Escape behavior inside non-portal `FileSearchView` needs a focused key-handler pass.
