# 006 Notes Window / Notes Browse / Notes-hosted ACP

Notes is a dedicated floating host with its own editor, storage, keyboard routing, actions, and embedded Agent Chat surface. Treat it as an independent product surface, not a launcher panel variant.

## Executive Summary

The Notes feature lets users create, edit, search, organize, delete, restore, and export Markdown notes in a separate window. The same host can switch into a Notes-owned embedded ACP chat, stage notes or note carts as context, show local ACP actions/history, and return to the editor without borrowing the main launcher ACP state.



## What Users Can Do

- Open Notes as a separate floating window and keep it independent from main launcher hide/focus behavior.
- Create an empty note with `Cmd+N` or a clipboard-backed note with `Cmd+Shift+N`.
- Type Markdown, use formatting commands, preview rendered Markdown, and rely on debounced saves.
- Switch notes, search across notes, find within the current note, pin notes, sort notes, and move through note history.
- Move notes to Trash, view Trash, restore a note, permanently delete a note, or empty Trash.
- Copy Markdown, copy a note deeplink, create quicklinks, or export note content.
- Send the current note, selection, or note cart into embedded Agent Chat without opening the main ACP surface.
- Use ACP actions, model/agent changes, and ACP history from inside Notes.
- Use Notes Browse as an attachment portal when another ACP surface requests a note target.

## Core Concepts

| Concept | Meaning | Owned by |
|---|---|---|
| `NotesApp` | Floating Notes host, editor state, note list, actions, search, trash, and embedded ACP cache. | `src/notes/window.rs` |
| Notes Browse | Expanded portal for selecting a note as ACP context. | `src/render_builtins/notes_browse.rs` |
| Note cart | Stored note-scoped context payloads staged into ACP and consumed after successful handoff. | `src/notes/storage.rs`, `src/notes/window/acp_host.rs` |
| Notes automation target | Stable runtime target for state/protocol receipts. | `tests/automation/notes_window_targeting.rs` |

## Entry Points

| Entry point | User intent | Expected target |
|---|---|---|
| Notes command/hotkey | Open or focus the floating Notes host. | `open_notes_window` in `src/notes/window/window_ops.rs` |
| Root Note row | Open a specific note from launcher search. | `open_note_in_notes_window`, not the toggle helper |
| `Cmd+P` | Open Notes note switcher in the floating host. | Notes `CommandBar` note switcher |
| `Cmd+Enter` / `Cmd+Shift+A` in Notes | Open embedded ACP and stage note context. | `open_or_focus_embedded_acp` |
| `Cmd+K` in ACP mode | Open Notes-hosted ACP actions. | `toggle_acp_actions` |

## User Workflows

### Open And Edit A Note


### Switch, Search, And Preview

Use `Cmd+P` to open the note switcher, type a filter, then arrow or press Enter to select. Use `Cmd+Shift+F` for cross-note storage search, and `Cmd+F` for window-local find inside the current note. Use `Cmd+Shift+P` to toggle Markdown preview while keeping the editor content as source of truth.

### Trash And Restore

Delete starts as a soft-delete into Trash. `Cmd+Shift+T` toggles the Trash view, `Cmd+Z` restores the selected trash note, and permanent delete removes the row. Confirmation dialogs must receive Enter, Escape, and Tab through Notes-owned keyboard routing because GPUI focus may not be in the rendered dialog tree yet.

### Send Notes To Embedded ACP

From the Notes editor, use `Cmd+Enter` or `Cmd+Shift+A`. Notes opens or reuses its embedded ACP view, stages the selected note or note-cart payload as inline context, dedupes cart items, and consumes note-cart rows only after staging succeeds. Reopening embedded ACP should preserve draft text when reuse is intended.

### Change Agent From Notes ACP

In Notes ACP mode, `Cmd+K` opens a Notes-parented ACP actions popup. Agent/model actions capture the originating `AcpChatView` weak target plus `notes_acp_generation`, refresh models before snapshot, relaunch the embedded view when needed, and restore the draft snapshot rather than mutating a stale replacement view.

### Attach A Note Through Notes Browse

When ACP asks for a Notes portal, the main window shows `NotesBrowseView` with a filter input, note list, preview pane, and footer hints. Arrow keys, wheel, click, and double-click keep selection and preview synchronized. Enter attaches a note target with stable note identity; Escape cancels the portal before clearing the filter.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Create note | Floating Notes | Editor | `Cmd+N` | `src/notes/window/notes.rs#create_note` | Blank note selected and saved through debounce | Notes storage row, editor state |
| Create from clipboard | Floating Notes | Editor | `Cmd+Shift+N` | `create_note_from_clipboard` | New note seeded from clipboard | Storage row content |
| Switch note | Floating Notes | Note switcher | `Cmd+P`, filter, Enter | Notes `CommandBar` / `select_note_internal` | Selected note changes | Notes selected id/editor state |
| Cross-note search | Floating Notes | Search field | `Cmd+Shift+F` | `refresh_notes_for_search_query` | Matching active notes shown | `notes_search_refresh_*` logs |
| Delete note | Floating Notes | Confirm dialog | Delete shortcut, Enter | `request_delete_selected_note` | Soft delete or permanent delete | Deleted state / Trash view |
| Restore note | Floating Notes | Trash | `Cmd+Z` | `restore_note` | Note returns to active list | Active note list |
| Open embedded ACP | Floating Notes | Editor | `Cmd+Enter` | `open_or_focus_embedded_acp` | ACP replaces editor body | Notes target state, ACP mode |
| Dismiss ACP local popup | Floating Notes | ACP popup | Escape | Notes ACP keyboard branch | Popup closes before editor return | Escape ordering tests/logs |
| Cancel ACP stream | Floating Notes | Streaming ACP | Escape | Notes ACP keyboard branch | Stream cancels, ACP stays open | `notes_acp_escape_cancelled_streaming` |
| Open ACP actions | Floating Notes | ACP | `Cmd+K` | `toggle_acp_actions` | Notes-parented actions popup | `parent="notes"` action receipt |
| Open ACP history | Floating Notes | ACP | ACP action / `@history` | `open_embedded_acp_history_popup` | Notes-local history popup | History portal contract tests |
| Attach note | Notes Browse | Expanded portal | Enter / double-click | `src/render_builtins/notes_browse.rs` | Focused note target attached | ACP pending context receipt |
| Cancel attach | Notes Browse | Expanded portal | Escape | Notes Browse key handler | Portal cancel before filter clear | Notes Browse source contract |
| Close Notes | Floating Notes | Editor or ACP | `Cmd+W` | Notes keyboard close path | Save, close dialogs, persist bounds, remove window | Close-order source tests |

## State Machine

| State | Enters from | Exits to | Important guards |
|---|---|---|---|
| Notes editor | Open Notes, ACP close, Trash back | ACP, Trash, close | Editor owns formatting, save, switcher, local search, and note actions. |
| Trash view | `Cmd+Shift+T` or action | All Notes, close | Restore and permanent delete replace normal edit/delete actions. |
| Confirm dialog | Delete/permanent delete | Prior editor/trash state | Enter confirms, Escape cancels, Tab cycles before editor key handling. |
| Note switcher | `Cmd+P` | Selected note or editor | Replacement mode can replace an active `@note` token before normal switching. |
| Cross-note search | `Cmd+Shift+F` | Editor/list | Storage errors log and render empty rows instead of panicking. |
| Markdown preview | `Cmd+Shift+P` | Editor | Source content remains the editor value. |
| Focus mode | User action | Editor | Chrome/footer fade; editor focus remains primary. |
| Notes-hosted ACP | `Cmd+Enter`, `Cmd+Shift+A`, note/cart handoff | Editor or close | Escape order is local popup, streaming cancel, then editor return. |
| Notes ACP actions | `Cmd+K` in ACP mode | ACP | Parent id is `notes`; stale generation dispatch is rejected. |
| Notes ACP history | ACP history action/portal | ACP | Only `AcpHistory` is locally supported in Notes. |
| Notes Browse portal | ACP note attachment request | Attach, cancel, or back | Main-window portal, not the floating Notes note switcher. |

## Visual And Focus States


## Keystrokes And Commands

| Shortcut | Context | Behavior |
|---|---|---|
| Enter | Confirm dialog | Confirm active dialog before editor handling. |
| Escape | Confirm dialog | Cancel active dialog. |
| Tab / Shift+Tab | Confirm dialog | Cycle dialog controls. |
| Escape | Notes ACP | Dismiss local popup, then cancel stream, then return to editor. |
| `Cmd+K` | Notes editor | Toggle Notes actions/command bar. |
| `Cmd+K` | Notes ACP | Toggle Notes-hosted ACP actions popup. |
| `Cmd+Enter` | Notes editor | Open embedded ACP and stage current note context. |
| `Cmd+Shift+A` | Notes editor | Send note/cart to embedded ACP. |
| `Cmd+W` | Notes editor or ACP | Save/prepare, close dialogs/actions, persist bounds, remove Notes window. |
| `Cmd+N` | Notes editor | Create blank note. |
| `Cmd+Shift+N` | Notes editor | Create note from clipboard. |
| `Cmd+P` | Notes editor | Toggle note switcher. |
| `Cmd+Shift+P` | Notes editor | Toggle Markdown preview. |
| `Cmd+F` | Notes editor | Window-local find in current note. |
| `Cmd+Shift+F` | Notes editor | Toggle cross-note search. |
| `Cmd+Shift+T` | Notes editor/trash | Toggle All Notes and Trash. |
| `Cmd+Z` | Trash | Restore selected note. |
| `Cmd+B`, `Cmd+I`, `Cmd+E` | Notes editor | Bold, italic, inline-code Markdown wrappers. |
| `Cmd+Shift+L` | Notes editor | Toggle checklist line. |
| `Cmd+L` | Notes editor | Select current line. |
| `Cmd+[`, `Cmd+]` | Notes editor | Note history back/forward. |
| `Cmd+1` to `Cmd+9` | Notes editor | Select pinned note by ordinal. |
| Alt+Up / Alt+Down | Notes editor | Move current line. |
| Alt+Shift+Up / Alt+Shift+Down | Notes editor | Duplicate line. |
| Arrow / wheel / click | Notes Browse | Change selected note and preview. |
| Enter / double-click | Notes Browse portal | Attach selected note. |
| Escape | Notes Browse portal | Cancel portal before clearing filter. |

## Actions And Menus

Notes uses shared command/action infrastructure but must pass the Notes parent when the action belongs to the floating host.

- Notes editor actions include new note, duplicate, browse/switch, find, copy Markdown, copy deeplink, quicklink/export, send to Agent Chat, trash actions, sort, pin, preview, and auto-size controls.
- Notes ACP actions reuse shared ACP action chrome but are filtered for the Notes host and close through Notes-owned helpers.
- Notes ACP action dispatch must use the originating ACP weak target and `notes_acp_generation`, not a fresh read of `embedded_acp_chat`.
- Notes Browse is a portal surface with attach/cancel semantics, not the same thing as the floating Notes note switcher.

## Automation And Protocol Surface

Use state-first receipts before screenshots.

| Surface | Target/proof | Notes |
|---|---|---|
| Notes editor | `getState`, `getElements`, `waitFor`, `batch` against Notes target | Use selected note/editor/search/trash state when exposed. |
| Notes ACP actions | Inspect parented actions popup and action ids | Parent must be `notes`; focus returns to ACP composer. |
| Notes Browse | `getState`/`getElements` on `NotesBrowseView` | Verify filter, selected row, preview, attach/cancel result. |

## Data, Storage, And Privacy Boundaries

- Notes persist in local SQLite tables with id, title, Markdown content, timestamps, deleted state, pinned state, sort order, and FTS indexes.
- Note bodies belong to Notes storage and editor state. Root search rows should expose metadata, not full note content.
- Note cart rows are note-scoped staged context payloads and should be deleted only after successful ACP staging.
- Trash is soft-delete first; permanent delete removes stored data.
- Embedded ACP context chips should be replaced when the host stages a new note target, not appended to stale chips from a previous note.

## Error, Empty, Loading, And Disabled States

- First launch may create or show a welcome note when active/deleted notes are empty.
- Empty All Notes shows a create affordance.
- Empty Trash shows a Trash-empty state and a path back to active notes.
- Empty Notes Browse shows no-notes or no-match copy depending on whether data or filter is empty.
- Notes Browse storage failures log `notes_browse_portal_load_failed` and render empty rows rather than panic.
- Missing/deleted note open from root should report failure instead of closing the launcher as if open succeeded.
- Unsupported portal kinds in Notes-hosted ACP are rejected or cancelled because Notes cannot host main-panel portals locally.

## Code Ownership

| Behavior | Owner files/tests |
|---|---|
| Floating host, modes, chrome, persistent window state | `src/notes/window.rs`, `src/notes/window/init.rs` |
| Open, close, toggle, root note open helper | `src/notes/window/window_ops.rs` |
| Note CRUD, search, delete/restore, view mode | `src/notes/window/notes.rs`, `src/notes/storage.rs`, `src/notes/model.rs` |
| Clipboard/export/format helpers | `src/notes/window/clipboard_ops.rs`, `src/notes/window/notes_actions.rs` |
| Keyboard routing | `src/notes/window/keyboard.rs` |
| Embedded ACP lifecycle and callbacks | `src/notes/window/acp_host.rs` |
| Notes Browse portal | `src/render_builtins/notes_browse.rs` |
| Automation target | `tests/automation/notes_window_targeting.rs` |
| ACP actions/draft/history/escape contracts | `tests/notes_acp_actions_parity_contract.rs`, `tests/notes_acp_agent_switch_draft_contract.rs`, `tests/notes_acp_history_portal_terminal_contract.rs`, `tests/notes_acp_escape_cmdw_contract.rs` |
| Context cart contracts | `tests/notes_context_cart_staging_contract.rs`, `tests/notes_context_cart_storage_contract.rs` |
| Notes Browse contracts | `tests/notes_browse_surface_contract.rs` |

## Invariants And Regression Risks

- Notes is a separate floating host and must not be hidden by main launcher hide paths.
- Root Note rows must use `open_note_in_notes_window`, not the toggle-style `open_notes_window`, so an already-open Notes window is focused/selected rather than closed.
- `close_notes_window` must release global locks before updating/removing the window and must unregister both automation and runtime window handles.
- Notes-hosted ACP and main ACP have separate cached `embedded_acp_chat` ownership.
- `ensure_embedded_acp_view` reuses ACP without overwriting draft; explicit relaunch paths restore snapshots.
- Notes-hosted ACP only supports the portal kinds Notes can host locally; `AcpHistory` is supported, broad main-panel portals are not.
- Escape in Notes ACP must run local popup, stream cancel, editor return in that order.
- Notes ACP actions must dispatch through the originating view and generation token.
- Actions popups owned by Notes must use parent automation id `notes`.
- Note cart handoff must dedupe, stage replacement context, and consume rows only after successful staging.
- Notes Browse must attach stable note identity, not display index.
- Notes Browse Escape must cancel the portal before treating Escape as filter clear/back.
- Auto-resize must tolerate restored window height above the default max.
- Notes Markdown editor disables the dynamic bottom scroll margin so deleting trailing lines collapses scroll extent.

## Verification Recipes

Use the smallest proof that can fail for the touched behavior.


```bash
cargo test --test notes_acp_agent_switch_draft_contract
cargo test --test notes_acp_actions_parity_contract
cargo test --test notes_acp_history_portal_terminal_contract
cargo test --test notes_acp_escape_cmdw_contract
cargo test --test notes_hosted_acp_host_isolation_contract
cargo test --test notes_context_cart_staging_contract
cargo test --test notes_browse_surface_contract
cargo test --test notes_ai_routing
cargo check --lib
cargo fmt --check
git diff --check
source checks
```


```bash
bun scripts/agentic/notes-embedded-acp-context-cart.ts
bun scripts/agentic/notes-acp-draft-agent-switch-replay.ts
bun scripts/agentic/notes-acp-actions-originating-view.ts
```


- Note-cart handoff produces one deduped ACP chip and consumes cart rows only after staging.
- ACP draft text survives reuse or agent-switch relaunch byte-for-byte.
- Notes ACP Escape closes local popup before cancelling stream and returns to editor only when idle.
- Notes Browse filter, selected row, preview, attach, and cancel are state-verifiable without screenshots.


## Agent Notes

- Do not use main-window focus as a proxy for Notes actions popup focus. Notes-owned popups must use the `notes` parent.
- Do not call the toggle open helper when opening a root Note row. Use the non-toggle note-open path.
- To verify embedded ACP identity, target the Notes automation window first and combine state receipts with Notes ACP logs; current ACP state may not expose a host field directly.
- If context chips look stale, inspect note target staging, note-cart consumption, `select_note_internal`, and `clear_notes_hosted_acp_context_for_note`.
- If Escape returns to the editor too early, inspect ACP-local popup state and streaming cancellation before the surface-mode branch.
- If Notes Browse attaches the wrong note, inspect stable note id construction and selection/preview synchronization.
- This belongs to `notes-window` unless the failing behavior is generic ACP composer, generic actions popup chrome, or root search ranking.
- Screenshots are only needed when visual layout is the behavior under test; most Notes contracts should be proven with state, logs, source contracts, or agentic receipts.

## Related Features

- [003 Agent Chat Context Composer](../raw-oracle/003-agent-chat-context/answer.md)
- [007 Root Unified Search Notes](../raw-oracle/007-root-notes/answer.md)
- [011 Root Unified Search Result Actions](../raw-oracle/011-root-source-actions/answer.md)

## Raw Oracle References

- [Prompt](../raw-oracle/006-notes-window/prompt.md)
- [Bundle map](../raw-oracle/006-notes-window/bundle-map.md)
- [Full answer](../raw-oracle/006-notes-window/answer.md)
- [Full output log](../raw-oracle/006-notes-window/output.log)
- [Session metadata](../raw-oracle/006-notes-window/session.json)

## Open Questions And Gaps

- `Cmd+Shift+D` may have a display/behavior conflict between copy deeplink metadata and date/time insertion. Verify live behavior before documenting it as canonical.
- The floating Notes note switcher and main-window Notes Browse portal are easy to confuse; keep them separate in future chapters.
- Include ACP portal test specs in future Oracle bundles when changing Notes-hosted history or mention replacement behavior.
