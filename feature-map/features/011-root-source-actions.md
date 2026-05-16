# 011 Root Unified Search Result Actions

This chapter maps Cmd+K actions for root unified search rows in the main launcher.

## Executive Summary

Root result actions make Cmd+K operate on the same focused ScriptList row identity that Enter would execute. When a root unified row is selected, the MainList actions dialog captures a typed `RootUnifiedActionSubject` at open time and executes actions against that captured subject, not against later live selection.

This feature owns MainList root action resolution, captured root subjects, root file compatibility capture, typed root action ids, action execution routing, content-light `actionsDialog` receipts, and state-first proof for source-specific action palettes.

It does not own dedicated built-in action hosts, source storage/search internals, generic script/scriptlet actions, config-backed shortcut/alias/deep-link actions, or visual placement except when state receipts cannot prove behavior.

## Human Capabilities

| Row kind | Actions a user can access |
|---|---|
| Files | Open File, Reveal in Finder, Copy Path, Copy Name, Quick Look, directory-only Search Inside Folder, file-only Browse Parent Folder. |
| Notes | Open Note, Copy Note Title, Copy Note ID. |
| Clipboard History | Paste Clipboard, Copy to Clipboard, Attach to AI, Pin/Unpin, Quick Look, Delete Clipboard Entry. |
| Browser Tabs | Switch to Tab, Open URL in Browser, Copy URL, Copy Title, Copy Title and URL. |
| Browser History | Open Page, Copy URL, Copy Title, Copy Title and URL. |
| AI Conversations | Resume Conversation, Copy Conversation Title, Copy Session ID, Copy Preview. |
| Dictation History | Paste Dictation, Copy Transcript, Attach to AI, Create Note from Transcript, Delete Dictation. |
| Windows | Switch to Window, Copy Window Title, Copy App Name, Copy Window Descriptor. |
| Skills and script issues | Open/inspect, copy ids, copy summaries. |
| Scripts, scriptlets, config-backed commands/apps | Delegate to existing MainList action owner for SDK/config actions. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| MainList actions host | `ActionsDialogHost::MainList` for `AppView::ScriptList`. | Root result actions are scoped to the main launcher, not dedicated built-in views. |
| Captured subject | `pending_root_unified_actions_subject`. | Captured when popup opens; execution must not re-read live selection. |
| Root file compatibility capture | `pending_root_file_actions_file`. | Supports root file action paths and direct shortcuts. |
| Typed action catalog | `RootUnifiedResultAction`. | Known ids parse to typed actions; unknown ids warn/no-op. |
| Typed subject | `RootUnifiedActionSubject`. | Notes, clipboard, tabs, history, conversations, dictation, windows, files, skills, issues, etc. |
| Content-light receipt | `actionsDialog` state. | Exposes host, title, stable key, source, selected id, visible actions; not local payloads. |
| Generic fallback | Existing script/config MainList actions. | Runs only when the selected row is not owned by root unified actions. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| Cmd+K | ScriptList focused root unified row. | Resolve selected visible `SearchResult`, capture subject, open MainList actions dialog. |
| Cmd+K while popup open | Existing actions popup. | Toggle/close according to popup policy. |
| Enter in actions dialog | Physical keyboard. | Activate selected action via shared route. |
| `simulateKey Enter` | Protocol while popup open. | Same route as physical Enter before parent view handlers. |
| Action row shortcut | Matching normalized shortcut. | Activates action id through same execution path. |
| Escape | Actions dialog open. | Pop drill-down or close dialog, clear subjects, restore focus. |
| Printable/backspace | Actions dialog open. | Search/filter actions inside dialog and resize/notify. |
| Root file direct shortcuts | Selected root file row. | Route through shared root file action executor. |

## State Model

| State | Meaning |
|---|---|
| Parent surface | `AppView::ScriptList`. |
| Actions host | `ActionsDialogHost::MainList`. |
| Popup surface | Detached/shared Actions Dialog window with `show_actions_popup=true`. |
| Mini placement | Actions search at top, top anchor, TopCenter position. |
| Full placement | Actions search at bottom, bottom anchor, BottomRight position. |
| Captured subject | Root unified row subject stored while popup is open. |
| Pending root file | File subject stored for root file actions. |
| Actions receipt | `open`, `host`, `contextTitle`, `contextStableKey`, `contextSource`, `selectedActionId`, `visibleActions`. |
| Dedicated hosts | File Search, Clipboard History, Dictation History, Browser Tabs/History, App Launcher, Window Switcher, ACP History keep their own hosts. |

## Action Routing

| Step | Behavior | Failure mode |
|---|---|---|
| Resolve selected row. | Use grouped visible selection from ScriptList. | No selected row falls back only if generic actions exist. |
| Determine owner. | Root unified owner runs before generic `has_actions()`. | Rows with owner `None` use generic fallback. |
| Capture subject. | Store typed subject and stable key before popup opens. | Missing subject later logs and returns. |
| Open popup. | MainList Actions Dialog receives context title/source/stable key. | Recent close debounce suppresses immediate reopen. |
| Filter/search actions. | Dialog owns search text and visible rows. | Parent surface keys are swallowed. |
| Activate. | Enter/click/shortcut returns `ActionsRoute::Execute { action_id, should_close }`. | Unknown/mismatched ids warn/no-op. |
| Execute. | Handler routes through `execute_root_unified_result_action` with captured subject. | Does not fall through to generic `handle_action` for root ids. |
| Close. | Clear pending subjects and restore focus. | Detached actions-window `on_close` must also clear context. |

## Interaction Matrix

| Interaction | Expected behavior | Proof |
|---|---|---|
| Cmd+K on root passive row. | MainList actions opens with matching context stable key and source-specific actions. | `actionsDialog.open`, host `MainList`, context key equals selected key. |
| Parent selection changes while popup open. | Action still applies to captured subject. | Captured subject source path and stable key receipt. |
| Physical Enter in popup. | Shared action route executes selected action. | Source audit and action result receipt. |
| Simulated Enter in popup. | Same route as physical Enter. | Stdin simulation path audit. |
| Row shortcut in popup. | Activates same action id path. | `visibleActions[].shortcut` and action receipt. |
| Escape in popup. | Pop route or close, clear context, focus main filter. | `actionsDialog` absent/closed and pending subjects cleared. |
| Known root id without subject. | Warn and return, no generic fallback. | Source audit. |
| Unknown root id. | Warn and no-op. | Action id parser/round-trip tests. |
| Dedicated built-in view Cmd+K. | Dedicated host handles actions. | `actions_host_for_view`/host-specific receipts. |
| Matrix across source filters. | Each source row shows expected action ids. | `root-source-actions-matrix.ts`. |

## File Actions

| Action | Applies to | Result |
|---|---|---|
| Open File | Files/directories where OS open is valid. | Uses shared root file open helper and records success where appropriate. |
| Reveal in Finder | Files/directories. | OS helper reveals path. |
| Copy Path | Files/directories. | Copies full path. |
| Copy Name | Files/directories. | Copies basename. |
| Quick Look | Files/directories where helper supports it. | Uses captured root file path; missing path returns controlled error. |
| Search Inside Folder | Directory rows only. | Opens dedicated File Search at trailing-slashed directory query. |
| Browse Parent Folder | Regular file rows only. | Clears stale MainList selection and opens dedicated File Search at parent folder. |

Root file direct shortcuts must route through the same shared executor as action rows. The raw pass pinned Cmd+Y for Quick Look; other shortcut glyphs should be read from `visibleActions.shortcut` or the full action catalog.

## Source Action Boundaries

| Source | Root action boundary |
|---|---|
| Notes | Metadata actions and Open Note; full Notes editor actions remain Notes window-owned. |
| Clipboard History | Root row actions can load/copy/attach/pin/delete explicitly, but receipts stay content-light. Dedicated clipboard browser owns full preview/action UI. |
| Browser Tabs | Switch/copy/open metadata actions; browser provider internals remain separate. |
| Browser History | Open/copy metadata actions; history snapshot/search internals remain separate. |
| ACP History | Resume/copy metadata only; no attach-summary action in this pass. |
| Dictation History | Paste/copy/attach/create note/delete can load transcript only after explicit action. |
| Windows | Switch/copy metadata actions; test provider can avoid AX dependency. |
| Skills/issues | Open/inspect/copy metadata actions. |
| Scripts/config commands/apps | Preserve existing MainList action owner and config-backed shortcut/alias/deep-link behavior. |

## Visual States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| ScriptList root row selected. | Grouped root rows with selected row focus styling. | Main filter/list. | `mainWindowPreflight.selectedResultKey`, visible results, selected role. |
| MainList root actions open. | Actions window with context title, action rows, icons, sections. | Actions dialog. | `actionsDialog` receipt and targetable actions dialog elements. |
| Mini-mode actions dialog. | Search input at top, top-anchored list. | Actions dialog. | Same state receipt; screenshot only for placement issues. |
| Full-mode actions dialog. | Search input at bottom, bottom-anchored list. | Actions dialog. | Same state receipt. |
| Destructive action visible. | Delete actions in Danger section. | Actions dialog. | `visibleActions[].destructive == true`. |
| Closed dialog. | ScriptList restored or window hidden by policy. | Main filter or focus coordinator. | Actions dialog absent; pending subjects cleared. |

## Data, Storage, And Privacy Boundaries

- This feature owns no persistent action map storage.
- Actions can mutate source-owned stores, such as clipboard pin/delete, dictation delete/create note, notes open, file frecency, app/window OS actions.
- `actionsDialog` receipts must not expose raw clipboard text, note bodies, dictation transcripts, browser page contents, or other local payloads.
- Attach/copy/delete actions may access local content as explicit side effects, but proof receipts remain content-light.
- Captured subjects preserve ids, previews, stable keys, source names, and metadata needed for action execution.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| No selected MainList row. | Generic actions only if `has_actions()` applies. |
| Root owner `None`. | Generic fallback only. |
| Known root id with no subject. | Warn `root_unified_result_action_missing_subject`, return/no-op. |
| Unknown root action id. | Warn `unknown_root_unified_result_action`, no-op. |
| Subject/action mismatch. | Warn `root_unified_result_action_subject_mismatch`, no-op. |
| Recently closed popup. | Suppress immediate reopen to avoid focus/footer race. |
| Quick Look missing path. | Controlled OS-helper error, no panic. |
| Destructive actions. | Visible as Danger/destructive; confirmation policy remains source/action specific. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Actions dialog/toggle | `src/app_impl/actions_dialog.rs`, `src/app_impl/actions_toggle.rs`, `src/render_builtins/actions.rs` |
| Root actions | `src/app_impl/root_unified_result_actions.rs#RootUnifiedResultAction`, `RootUnifiedActionSubject`, `root_unified_actions_for_subject`, `execute_root_unified_result_action` |
| ScriptList selection | `selected_main_list_search_result_owned`, grouped result cache, selected index |
| Captured subjects | `pending_root_unified_actions_subject`, `pending_root_file_actions_file` |
| Keyboard routing | `route_key_to_actions_dialog`, `execute_actions_route_action`, `runtime_stdin_match_simulate_key.rs`, `app_run_setup.rs` |
| File handoffs | `execute_root_file_action`, root file Quick Look/Browse Parent/Search Inside paths |
| Tests/source audits | `root_unified_source_actions_contract.rs`, `root_file_action_enter_routes_activation_before_close`, `root_file_quick_look`, `root_file_browse_parent_folder`, `root_unified_result_actions` |
| Runtime matrix | `scripts/agentic/root-source-actions-matrix.ts` |

## Invariants And Regression Risks

- Cmd+K acts on the same focused ScriptList row identity as Enter.
- MainList root owner resolution happens before generic `has_actions()` fallback.
- Subject is captured on open; execution never re-reads live selection for root unified actions.
- Physical Enter, simulated Enter, shortcuts, and click callbacks share activation handling.
- Known root action id without subject never falls through to generic `handle_action`.
- Unknown or mismatched root action ids warn and no-op.
- Root action receipts are content-light.
- Script/scriptlet and config-backed command/app rows preserve existing script action ownership.
- Dedicated built-in views keep their own action hosts.
- Root file action executor is shared by Enter, direct shortcuts, and action rows.
- File handoffs clear stale MainList selection before dedicated File Search owns selection.
- Windows proof can use metadata-only providers in hidden sessions.

## Verification Recipes

### Source And Unit Contracts

Run:

```bash
cargo test --test source_audits root_unified_source_actions_contract -- --nocapture
cargo test --test source_audits copy_deeplink_prefers_command_namespace_for_config_backed_rows -- --nocapture
cargo test --test source_audits root_file_action_enter_routes_activation_before_close -- --nocapture
cargo test --test source_audits root_file_quick_look -- --nocapture
cargo test --test source_audits root_file_browse_parent_folder -- --nocapture
cargo test --lib quick_look_missing_path_returns_error_without_panic -- --nocapture
cargo test --lib root_file_actions_for_regular_file_displays_parent_folder_with_tilde_home -- --nocapture
cargo test --lib parent_folder_search_query_shortens_home_prefix_for_display -- --nocapture
cargo test --lib root_unified_result_actions -- --nocapture
```

Check:

- Root subjects are captured and used.
- Action ids round-trip and do not fall through incorrectly.
- Root file action paths use shared helpers.
- Config-backed command rows preserve command namespaces.

### Runtime State Proof

Run or adapt:

```bash
bun scripts/agentic/root-source-actions-matrix.ts
```

For each source filter, prove:

- `mainWindowPreflight.computedSearchText`
- `sourceFilters`
- selected row role, type label, source name, stable key
- `actionsDialog.open`
- `actionsDialog.host == "MainList"`
- `actionsDialog.contextStableKey == selected.stableKey`
- `visibleActions[].id`, label, section, shortcut, destructive, enabled

Use `SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN=1` only in proof harnesses so the detached actions window can be inspected before focus auto-close.

Use `SCRIPT_KIT_WINDOW_SEARCH_TEST_PROVIDER` for deterministic `w:` receipts in hidden sessions.

## Agent Notes

- Prefer `getState`, `getElements`, `waitFor`, and ActionsDialog-targeted `batch`; screenshots are unnecessary for catalog/routing/privacy proofs.
- Do not prove captured-subject semantics visually. Use state/action receipts.
- Run native/macOS proof only for real Quick Look, Finder reveal, app launch, real window switching, or AX focus behavior.
- Treat any raw payload in action receipts as a privacy regression.
- Treat action execution against live selection after popup open as a correctness regression.

## Related Features

- [001 Main Menu](./001-main-menu.md) owns grouped root rows and selection identity.
- [002 File Search](./002-file-search.md) owns dedicated File Search after root file handoff.
- [007 Root Notes](./007-root-notes.md), [008 Root Clipboard](./008-root-clipboard-history.md), [009 Root Dictation](./009-root-dictation-history.md), and [010 Root ACP History](./010-root-acp-history.md) own their source row semantics.

## Raw Oracle References

- [Prompt](../raw-oracle/011-root-source-actions/prompt.md)
- [Bundle map](../raw-oracle/011-root-source-actions/bundle-map.md)
- [Answer](../raw-oracle/011-root-source-actions/answer.md)
- [Full output log](../raw-oracle/011-root-source-actions/output.log)
- [Session metadata](../raw-oracle/011-root-source-actions/session.json)

## Open Questions And Gaps

- Built-in/App action ownership has tension: some rows may delegate to existing script/config actions while the matrix includes app/command root actions. Verify intended reachability.
- Direct root-file shortcut glyphs beyond Cmd+Y need the full action catalog or `visibleActions.shortcut` receipts.
- Destructive source actions need explicit confirmation/no-confirm policy mapping.
- Action result success/error feedback is unevenly specified and should be audited branch by branch.
- Privacy tests should cover receipts after attach/copy/delete actions without serializing local payloads.
- Dedicated built-in action hosts need host-specific audits to avoid root id reuse or host leakage.
