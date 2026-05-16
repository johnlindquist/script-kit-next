# 005 Built-in Filterable Surfaces

This chapter maps the dedicated built-in list surfaces that share filterable rows, keyboard navigation, actions, and protocol receipts.

## Executive Summary

Built-in filterable surfaces are app-owned command views such as Clipboard History, App Launcher, Window Switcher, Browser Tabs, Emoji Picker, and Process Manager. They are opened by canonical `triggerBuiltin` routes and use a shared contract: the renderer, keyboard handlers, wheel handlers, state receipts, element receipts, and action execution must all resolve through the same visible-row projection.

This feature owns the dedicated list views, trigger registry/planner/dispatcher entry, route-local filters and selection, surface contracts, list semantic ids, built-in footer/focus/scroll behavior, and surface-specific activation such as paste, launch, focus window, activate tab, insert emoji, or stop process.

It does not own root passive unified-search sources, File Search portals, ACP/Notes portals, native AppKit enumeration internals, or the shared Actions Dialog implementation. Those systems are adjacent callers or hosts.

## Human Capabilities

| Surface | What a user can do | Default activation | Distinctive proof |
|---|---|---|---|
| Clipboard History | Browse, filter, preview, paste, attach to AI, pin, Quick Look, and run clipboard actions. | `Enter` copies selected entry and pastes into the frontmost app. | Dataset count vs filtered visible count, preview panel, clipboard action host. |
| App Launcher | Browse and filter installed macOS apps. | `Enter` launches the selected visible app. | Recursive `.app` scan with app bundles treated as leaves. |
| Window Switcher | Browse/filter open windows and focus one. | `Enter` activates the selected visible window. | Window cache seeded at route entry, filter on app/title display string. |
| Browser Tabs | Browse/filter open Safari/Chromium-family tabs. | `Enter` activates the existing selected tab. | Fuzzy tab ranking and metadata-only tab rows. |
| Emoji Picker | Browse categories, search emoji, navigate a grid, and paste. | `Enter` writes emoji to clipboard, hides, and delayed-pastes. | Grid-aware arrow movement and count asymmetry. |
| Process Manager | Browse active Script Kit child processes, filter, stop selected, and stop all. | `Enter` runs selected process action, usually stop. | Two-second active refresh, PID-aware visible-row helpers. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `TriggerBuiltinRegistry` | Canonical route table for app-owned built-ins and aliases. | Aliases and command ids validate once at startup; stdin and internal dispatch share the same resolver. |
| `FilterableRoutePlan` | Pure route plan for dedicated built-in surfaces. | Opening a built-in prepares route-local filter, selection, cache, prompt view, resize, and semantic re-key. |
| Visible-row helper | Surface-owned projection from dataset plus filter into visible rows. | Renderer, state, elements, keyboard, wheel, and actions must use this projection instead of raw backing indexes. |
| Dataset count | Full backing collection size. | Reported as `choiceCount`. |
| Visible count | Filtered row count. | Reported as `visibleChoiceCount` and must not exceed `choiceCount`. |
| Surface contract | Automation metadata for the active view. | `getState.surfaceContract` and `automationSemanticSurface` distinguish dedicated surfaces from generic `scriptList`. |
| List semantic id | Stable id for the main row list. | `getElements` uses ids such as `list:clipboard-history`, `list:apps`, `list:browser-tabs`, `list:emoji-results`, and `list:processes`. |
| Actions host | Host-specific context for Cmd+K action popup. | Dedicated view owns the subject; Actions Dialog owns popup presentation and search. |

## Entry Points

| Entry | Aliases | User result | Notes |
|---|---|---|---|
| Clipboard History | `clipboard-history`, `clipboard`, `clipboardhistory`. | Opens full preview-dense clipboard browser. | Root `clipboard:` / `c:` source is separate and metadata-only. |
| App Launcher | `apps`, `app-launcher`, `applauncher`. | Opens mini app launcher list. | Internal route can exist without launcher catalog entry. |
| Window Switcher | `window-switcher`, `windowswitcher`, `windows`. | Opens mini window switcher list. | Seeds `cached_windows` through window listing. |
| Browser Tabs | `browser-tabs`, `browsertabs`, `tabs`. | Opens mini browser tab picker. | Root `tabs:` / `t:` source is separate and passive. |
| Emoji Picker | `emoji`, `emoji-picker`, `emojipicker`. | Opens emoji grid/search view. | Frequent snapshot is frozen when opened. |
| Process Manager | `process-manager`, `processmanager`, `processes`. | Opens active process manager list. | Periodic refresh only while active. |

## Shared Surface Contract

| Area | Required behavior | Receipt |
|---|---|---|
| Route dispatch | `triggerBuiltin` resolves through the canonical registry, builds a route plan, and updates `current_view`. | `getState.promptType`, `surfaceContract`, `automationSemanticSurface`. |
| Filter input | Typing writes the variant-owned filter, recomputes visible rows, and clamps selection. | `getState.inputValue`, `visibleChoiceCount`. |
| Row projection | All render, state, element, navigation, scroll, and action paths use the visible-row helper. | `getElements` list count matches visible rows. |
| Selection | Arrow, wheel, and row clicks move selected visible rows. | `selectedIndex`, `focusedSemanticId`, selected element value/id. |
| Scroll | Wheel uses selected-row ownership and `ScrollStrategy::Nearest`; render-time reanchor must not fight live selection. | Source audits and matrix tests. |
| Escape | Clears non-empty filter first, otherwise returns to launcher or hides according to origin. | Input clears, surface rekeys, or `windowVisible:false`. |
| Actions | Cmd+K opens only when a valid host exists or the renderer explicitly routes it. | `activePopupContract` with host and action ids. |
| Sizing | Mini list for compact list surfaces; expanded/full only when preview is required. | `surfaceContract.visualPolicy`, chrome audits. |

## Shared Keystrokes

| Key | Popup state | Filter state | Behavior |
|---|---|---|---|
| Character input | No popup. | Any. | Appends to the surface filter and recomputes visible rows. |
| Backspace | No popup. | Non-empty filter. | Removes text and restores browse rows as the filter empties. |
| Escape | Actions dialog open. | Any. | Actions Dialog handles route pop/close before host filter logic. |
| Escape | No popup. | Non-empty filter. | Clears the filter and resets visible rows. |
| Escape | No popup. | Empty filter, launcher-origin layer. | Returns to ScriptList and restores launcher sizing/focus. |
| Escape | No popup. | Empty filter, direct hide path. | Resets/hides main and rekeys semantic surface to `scriptList`. |
| ArrowDown/ArrowUp | No popup. | Any. | Moves selected visible row, scrolls nearest, and updates focused row state. |
| Wheel | No popup. | Any. | Moves selected visible row through shared wheel target helpers. |
| Enter | No popup. | Selected row exists. | Executes surface-specific activation for the selected visible row. |
| Cmd+K | No popup. | Selected row exists and host supports actions. | Opens Actions Dialog for the dedicated host. |

## Surface-Specific States

### Clipboard History

Clipboard History is an expanded preview browser because the user often needs to inspect clipboard payloads before acting.

| State | Trigger | Visual/state result | User actions |
|---|---|---|---|
| Empty dataset | No cached entries. | Empty message `No clipboard history`, `choiceCount=0`, `visibleChoiceCount=0`, preview empty. | Type filter or Escape. |
| Filter miss | Dataset exists, filter matches no `text_preview`. | Empty message `No entries match your filter`, dataset count remains non-zero. | Clear filter. |
| Text row selected | Visible text entry. | Document icon, content preview or cached fallback, content type, timestamp. | Enter paste, Cmd+K actions, Cmd+Y Quick Look. |
| Link/file/color row selected | Visible metadata entry. | Preview resolves full content where available, falls back to text preview. | Paste/copy, Quick Look, reveal/open/share where actions expose them. |
| Image row selected | Visible image entry. | Image cache/icon, dimensions, preview image where cached. | Paste/copy, Quick Look temp preview. |
| Full payload unavailable | `get_entry_content(id)` returns none. | Preview warns it is showing cached preview only. | Continue browse/action safely. |
| Attachment portal | ACP portal flag active. | Footer/actions switch from ordinary paste to context attachment. | Attach selected entry as `kit://clipboard-history?id=...`. |
| Root passive source | `clipboard:` / `c:` in launcher. | Metadata-only rows, capped, no raw content read. | Enter loads/copies/pastes existing entry through root action path. |

Clipboard action states include paste selected, attach to AI, pin/unpin, Quick Look, Open With, Reveal in Finder, copy, share, delete, clear, and paste sequentially. The raw Oracle pass flagged the exact action id/shortcut catalog as a next-pass source expansion.

### App Launcher

App Launcher is a compact mini list of installed macOS apps.

| State | Trigger | Visual/state result | User actions |
|---|---|---|---|
| Empty app dataset | App scan/cache returns no apps. | Empty mini list with launch/back footer hints. | Escape or wait for scan/cache refresh. |
| Unfiltered browse | Empty filter. | Rows from `app_launcher_visible_row_names`. | Type filter, Enter launch. |
| Filtered browse | Non-empty filter. | Visible rows narrow through app launcher helpers. | Enter launches selected visible app. |
| Vendor folder app | Recursive scan finds `/Applications/<Vendor>/*.app`. | `.app` is indexed as a leaf; `Contents/` is not descended into. | Launch app by bundle path. |
| Cache/icon state | App catalog and icons seeded from scan/cache. | App row can show icon where renderer resolves one. | Launch or refresh next route. |
| Cmd+K | Dedicated app launcher view. | Renderer does not advertise dedicated actions. | No dead actions popup should appear. |

MainList app-row actions such as Add/Edit Shortcut, Add/Edit Alias, and Copy Deep Link are root launcher actions, not dedicated App Launcher renderer actions.

### Window Switcher

Window Switcher is a compact mini list of current native windows.

| State | Trigger | Visual/state result | User actions |
|---|---|---|---|
| Preload success | Route calls `list_windows()`. | `cached_windows` seeded, `WindowSwitcherView`, mini surface. | Type filter, Enter focus. |
| Filtered rows | Non-empty filter. | Rows match formatted app/title string. | Clear filter or Enter focus. |
| Preload failure | Window listing unavailable. | Route fails closed instead of opening a broken list. | Stay prior view or show error path. |
| Deterministic agent rows | Test provider seeds metadata windows. | Root/source action receipts can avoid macOS AX dependence. | Agentic proof without native window state. |

Tile/manage/window action catalog details were not fully expanded by the raw pass and remain a next-pass gap.

### Browser Tabs

Browser Tabs is a compact picker over currently open Safari and Chromium-family tabs.

| State | Trigger | Visual/state result | User actions |
|---|---|---|---|
| Preload success | `triggerBuiltin browser-tabs`. | Running browser tab metadata is cached for the view. | Type filter, Enter activate existing tab. |
| No tabs | Empty tab cache. | Empty message `No open browser tabs`. | Open browser tabs or Escape. |
| Filter miss | Filter has no fuzzy match. | Empty message `No browser tabs match your filter`. | Clear filter. |
| Fuzzy filtered | Non-empty filter. | `fuzzy_search_browser_tabs`, rows from `display_title()`. | Enter activates selected tab. |
| Root passive source | `tabs:` / `t:` in launcher. | Metadata-only cached rows, disabled by default. | Selecting row switches existing tab. |
| Preload failure | Provider unavailable or script failure. | Route fails closed instead of stale rows. | Stay prior view or error path. |

Root Browser Tabs must not read page content, favicons, cookies, downloads, or network data. Dedicated Browser Tabs also activates existing tabs rather than opening duplicate URLs.

### Emoji Picker

Emoji Picker is a grid-oriented mini utility surface with category/frequency behavior.

| State | Trigger | Visual/state result | User actions |
|---|---|---|---|
| Empty search | Open picker with no filter/category pin. | Frequently Used section appears above category grid. | Arrow/click/Enter, type search. |
| Category pin | `selected_category` set. | Ordered emoji list restricted to category. | Change category or search. |
| Search | Non-empty filter. | `visibleChoiceCount=search_emojis(filter)`, `choiceCount=EMOJIS`. | Enter/click paste. |
| Arrow grid navigation | Grid with headers/cells. | Up/Down use grid layout and skip headers; Left/Right single-step. | Continue navigation or paste. |
| Paste commit | Selected emoji exists. | Clipboard receives emoji, main hides/resets, delayed Cmd+V. | Frontmost app receives emoji. |
| Frequent use | Commit path records use. | Frequent order changes on next open. | Reopen sees updated frequent row. |
| Cmd+K | Emoji host available. | Actions Dialog opens with `EmojiPicker` host. | Execute host-specific action or close. |

Emoji count asymmetry is intentional: `choiceCount` is all emoji, while `visibleChoiceCount` is search/category narrowed. Tests should fail if these slots are swapped.

### Process Manager

Process Manager is the reference visible-row owner for destructive mini built-in lists.

| State | Trigger | Visual/state result | User actions |
|---|---|---|---|
| Active processes | Active child processes exist. | Rows show process metadata and formatted running duration. | Stop selected, Stop All, filter. |
| No processes | Active process list empty. | Centered empty state. | Escape or wait for new process. |
| Filtered processes | Non-empty filter. | PID-aware `process_manager_visible_rows`. | Clear filter or stop selected visible process. |
| Periodic refresh | View remains active. | Poll every 2 seconds, update cache only on changes, clamp selection. | Continue browse or navigate away. |
| Stop selected | Selected visible process. | Termination flow, escalation if still running after timeout. | Row removed after refresh. |
| Stop all/cleanup | Bulk action visible. | Clicks stop propagation so destructive action is not double-handled. | All child processes terminated/cleaned. |

Exact row actions, Stop All shortcuts/placement, copy-details, cleanup, confirmations, and error copy need a full process-manager source/action pass.

## Actions Matrix

| Surface | Action | Trigger | Outcome |
|---|---|---|---|
| Clipboard History | Paste selected. | Enter or selected click/double-click. | Copy selected entry, hide/reset, delayed paste. |
| Clipboard History | Attach to AI. | Cmd+Enter, Ctrl+Cmd+A, or action row. | Adds a `kit://clipboard-history?id=...` context part. |
| Clipboard History | Pin/unpin. | Cmd+P or action row. | Toggle pinned state and update ordering/persistence. |
| Clipboard History | Quick Look. | Cmd+Y or action row. | Native Quick Look or HUD/toast on failure. |
| Clipboard History | Open/reveal/copy/share/delete/clear. | Actions row. | File/clipboard/history side effect. Exact ids need source pass. |
| App Launcher | Launch app. | Enter or selected click. | Opens/focuses selected app. |
| Window Switcher | Activate window. | Enter or selected click. | Focuses selected window. |
| Browser Tabs | Activate tab. | Enter or selected click. | Switches to existing tab. |
| Emoji Picker | Paste emoji. | Enter or row click. | Writes emoji, hides, delayed paste. |
| Emoji Picker | Actions popup. | Cmd+K. | Opens host-specific action popup. |
| Process Manager | Stop selected. | Enter, click, or action row. | Terminates selected child process. |
| Process Manager | Stop all. | Visible bulk action. | Terminates/cleans active child processes. |
| Shared Actions Dialog | Search actions. | Type in popup. | Refilters action rows and resizes popup. |
| Shared Actions Dialog | Navigate actions. | Arrow keys. | Skips section headers and selects actionable rows. |
| Shared Actions Dialog | Execute/drill down. | Enter. | Executes terminal action or pushes drill-down route. |

## Visual And Protocol Matrix

| Surface | Chrome | State/elements expectations |
|---|---|---|
| Shared mini surfaces | Minimal list shell and native footer. | Stable `surfaceContract`, filter input id, list semantic id, selected row. |
| Clipboard History | Expanded scaffold with preview pane. | `PromptChromeAudit::expanded("clipboard_history")`, preview content/fallback message, full vs visible counts. |
| App Launcher | Minimal list. | No dead Cmd+K action hint, app visible-row helper used by state/elements. |
| Window Switcher | Minimal list. | Filter on display string, trigger route seeds window cache. |
| Browser Tabs | Compact single-column shell. | `browser-tabs-filter`, `browser-tabs` list, fuzzy order mirrored in elements. |
| Emoji Picker | Grid chrome. | `emoji-results` list, grid row height, arrow interception, count asymmetry. |
| Process Manager | Minimal list with destructive affordances. | PID-aware visible rows, refresh clamps selection, click propagation stopped. |
| Attached actions popup | Parent-linked popup. | `activePopupContract` identifies host, context, selected action, visible actions. |

Fallback collector warnings such as `collector_used_current_view_fallback` are failures for this feature. A surface that renders rows must expose those same rows semantically.

## Root Source Boundaries

Dedicated built-in views must not be collapsed with root passive sources:

- Dedicated Clipboard History can inspect full clipboard entries, preview content, paste, pin, Quick Look, and attach.
- Root Clipboard History is capped, metadata-only, excludes raw content, and appears only through root source configuration or `clipboard:` / `c:`.
- Dedicated Browser Tabs opens a compact browser-tab picker and activates existing tabs.
- Root Browser Tabs is disabled by default, metadata-only, cache-backed, and selected rows switch existing tabs through the root activation path.
- Root action palettes for files, passive rows, windows, apps, scripts, and skills are owned by MainList root result actions, not by these dedicated built-in renderers.

## Data, Storage, And Privacy Boundaries

- Clipboard exclusion hooks must keep excluded clips out of dedicated history and root rows.
- Root clipboard rows must not read raw clipboard content during grouping.
- Root browser tab rows must not read page content, favicons, cookies, downloads, or network data.
- App Launcher scans configured macOS app roots recursively but treats `.app` bundles as leaves.
- Process Manager acts on Script Kit child processes and must show destructive actions with clear state, propagation, refresh, and clamp behavior.
- Hidden built-ins can stay resolvable by canonical id for hotkeys and protocol callers without appearing in launcher search.
- Config gates can hide launcher catalog entries; canonical `triggerBuiltin` routes remain governed by the registry and feature gates.

## Error, Empty, Loading, And Disabled States

| Area | State | Expected behavior |
|---|---|---|
| Trigger registry | Unknown builtin id. | Rate-limited unknown-name warning and protocol counter. |
| Trigger registry | Alias or command id collision. | Startup validation failure. |
| Route planner | Preload failure for windows/tabs. | Fail closed instead of showing stale/broken rows. |
| Filter | No visible matches. | Empty-filter-miss message distinct from empty dataset where surface supports it. |
| Counts | Filtered rows. | `visibleChoiceCount <= choiceCount`. |
| Elements | Missing collect arm. | Fallback warning, treated as coverage failure. |
| Clipboard | Missing full payload. | Preview fallback copy, not false full-content claim. |
| Clipboard | Quick Look failure. | HUD/toast or structured error. |
| Browser Tabs | Provider unavailable. | Fail closed or empty/error state without duplicate tab open. |
| Emoji | Arrow in grid. | Key handled by emoji grid, not text cursor movement. |
| Process Manager | Stop fails or times out. | Escalation/status/error, then refresh and clamp. |
| Actions popup | No valid host. | Do not advertise dead Cmd+K actions. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Built-in catalog | `src/builtins/mod.rs` |
| Trigger registry/resolution | `src/builtins/trigger_registry.rs`, `src/builtins/trigger_resolve.rs` |
| Built-in renderers | `src/render_builtins/clipboard*.rs`, `src/render_builtins/app_launcher.rs`, `src/render_builtins/window_switcher.rs`, `src/render_builtins/browser_tabs.rs`, `src/render_builtins/emoji_picker.rs`, `src/render_builtins/process_manager.rs`, `src/render_builtins/common.rs` |
| Clipboard storage/actions | `src/clipboard_history/mod.rs`, `src/clipboard_history/types.rs`, `src/clipboard_history/macos_paste.rs`, `src/clipboard_history/quick_look.rs` |
| App launcher scan/launch | `src/app_launcher/scanning.rs`, `src/app_launcher/launch.rs`, `src/app_launcher/core_types.rs` |
| Browser tabs | `src/browser_tabs.rs` |
| Emoji data/frecency | `src/emoji/mod.rs` |
| Process manager | `src/process_manager/mod.rs` |
| Surface contracts | `lat.md/builtins.md`, `lat.md/surfaces.md`, `lat.md/automation.md` |

## Verification Recipes

### Filterable Matrix

Run:

```bash
cargo test filterable_surface_agentic_matrix_contract
cargo test filterable_subviews_getelements_filter_aware_contract
```

Check:

- Each supported surface opens through real `triggerBuiltin`.
- `surfaceContract` and `automationSemanticSurface` match the active view.
- `visibleChoiceCount` matches rendered/element rows and never exceeds `choiceCount`.
- No collector fallback warning appears for migrated surfaces.

### Clipboard History

Run:

```bash
cargo test clipboard_history_getelements_filter_aware_contract
cargo test clipboard_history_state_filter_receipt_contract
```

Check:

- Filtered state/elements share the same visible-row projection.
- Empty dataset and filter miss remain distinguishable.
- Clipboard attach/paste routes are not confused.

### App Launcher / Window Switcher / Browser Tabs

Run:

```bash
cargo test app_launcher_visible_rows_contract
cargo test window_switcher_triggerbuiltin_contract
cargo test collect_elements_browser_tabs_arm_contract
```

Check:

- App Launcher state/elements use app visible-row helpers.
- Window Switcher trigger route seeds `WindowSwitcherView` through the registry.
- Browser Tabs elements use fuzzy-ranked browser-tab rows and stable semantic ids.

### Emoji Picker

Run:

```bash
cargo test emoji_picker_arrow_up_down_contract
cargo test emoji_picker_state_choice_count_asymmetry_contract
cargo test emoji_picker
```

Check:

- Up/Down are intercepted by grid navigation.
- `choiceCount` remains full emoji dataset while `visibleChoiceCount` is narrowed.
- Footer/grid source audits still match the renderer.

### Process Manager

Run:

```bash
cargo test trigger_builtin_process_manager_contract
cargo test process_manager_visible_rows_contract
```

Check:

- Process Manager trigger route resolves through canonical built-in ids.
- Renderer, state, elements, stop actions, and refresh all use visible-row helpers.

## Agent Notes

- Always open these views with `triggerBuiltin` rather than simulating launcher clicks when proving dedicated behavior.
- Prefer `getState` and `getElements` after setting a filter with protocol/batch input.
- Treat list-count mismatches as real behavior failures, not harmless automation drift.
- For Clipboard History and Browser Tabs, keep dedicated-view proof separate from root passive source proof.
- For destructive Process Manager actions, prove selection from visible rows before sending Enter or action execution.
- For Emoji Picker, use grid-aware proof and do not assume ordinary one-row arrow behavior.
- Do not advertise Cmd+K for a surface unless its renderer or action host actually supports it.

## Related Features

- [001 Main Menu](./001-main-menu.md) owns root launcher grouping, source filters, fallback rows, and MainList actions.
- [002 File Search](./002-file-search.md) owns dedicated filesystem browsing and file attachment portals.
- [003 Agent Chat Context](./003-agent-chat-context.md) owns ACP attachment destinations for clipboard and other context parts.
- [004 MCP / SDK / Protocol Automation](./004-mcp-sdk-protocol.md) owns `triggerBuiltin`, `getState`, `getElements`, `waitFor`, and `batch` proof surfaces.

## Raw Oracle References

- [Prompt](../raw-oracle/005-built-in-filterable-surfaces/prompt.md)
- [Bundle map](../raw-oracle/005-built-in-filterable-surfaces/bundle-map.md)
- [Answer](../raw-oracle/005-built-in-filterable-surfaces/answer.md)
- [Full output log](../raw-oracle/005-built-in-filterable-surfaces/output.log)
- [Session metadata](../raw-oracle/005-built-in-filterable-surfaces/session.json)

## Open Questions And Gaps

- Full `src/app_impl/trigger_builtin_dispatch.rs`, `src/app_impl/routes.rs`, `src/main_sections/app_view_state.rs`, `src/app_layout/collect_elements.rs`, and `src/prompt_handler/mod.rs` bodies should be expanded before calling route/state/elements mapping complete.
- Clipboard Actions Dialog full catalog/action ids/shortcuts for Open With, Reveal in Finder, Copy, Share, Delete, Clear, Attach, Pin/Unpin need a source pass.
- Emoji Picker action catalog beyond paste behavior needs the `ActionsDialogHost::EmojiPicker` owner mapped.
- Process Manager exact row actions, Enter semantics, Stop All placement/shortcut, copy-details, cleanup, confirmation, and error states need full renderer/action context.
- App Launcher scan/cache/icon details need a full source pass across scanning, launch, core types, and renderer rows.
- Window Switcher tile/manage/focus action catalog and permission/error UI need full window-control and action-host context.
- Browser Tabs provider-specific errors, ranking score fields, provider labels/icons, and activation failure toasts need provider source expansion.
- Paste Sequentially visible states and receipts need the clipboard sequence execution path mapped.
