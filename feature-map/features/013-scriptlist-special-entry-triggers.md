# 013 ScriptList Special Entry Triggers / First-character Route Handoffs

This chapter maps the narrow ScriptList trigger handoffs for `~`, `/`, `@`, `>`, and `?`.

Raw Oracle reference: [answer](../raw-oracle/013-scriptlist-special-entry-triggers/answer.md), [prompt](../raw-oracle/013-scriptlist-special-entry-triggers/prompt.md), [bundle map](../raw-oracle/013-scriptlist-special-entry-triggers/bundle-map.md), [full log](../raw-oracle/013-scriptlist-special-entry-triggers/output.log), [session metadata](../raw-oracle/013-scriptlist-special-entry-triggers/session.json).

## Executive Summary

ScriptList special entries are route handoffs typed through the main launcher filter. They are not ordinary root search, source filters, menu syntax, capture syntax, or a general command parser.

The owner is `ScriptListSpecialEntry` in `src/app_impl/filter_input_core.rs`. It has five committed variants:

| Variant | User input | Destination |
|---|---|---|
| `FileSearchMini` | `~`, `~/...` | Mini File Search. |
| `AcpSlashPicker` | `/` | Embedded/detached Agent Chat with slash picker staged. |
| `AcpMentionPicker` | `@` | Embedded/detached Agent Chat with mention/context picker staged. |
| `QuickTerminal` | `>` | PTY-backed Quick Terminal. |
| `ActionsHelp` | `?` | Shared actions/help dialog when actions are available. |

The runtime handoff happens in `ScriptListApp::handle_filter_input_change` in `src/app_impl/filter_input_change.rs`. When a special entry matches, the handler logs `script_list_special_entry_routed`, dispatches the destination helper, and returns before normal menu-syntax mode processing continues.

The negative space is load-bearing. `/tmp`, `@browser`, `>deploy -- prod`, `:`, `+`, `!`, `#`, capture syntax, task/calendar-style menu syntax, and source filters must not route through this classifier.

## What Users Can Do

| User capability | Exact input | Result |
|---|---|---|
| Open mini File Search at home. | `~` | Opens File Search Mini with query `~/`. |
| Open mini File Search at a home-relative path. | `~/src` | Opens File Search Mini with query `~/src`. |
| Open ACP slash commands. | `/` | Opens Agent Chat and stages the slash picker. |
| Open ACP mention/context picker. | `@` | Opens Agent Chat and stages the mention/context picker. |
| Open Quick Terminal. | `>` | Opens `QuickTerminalView`, focuses the terminal prompt, and uses PTY-owned key behavior. |
| Open actions/help. | `?` | Opens or toggles actions only when `has_actions()` is true. |
| Type literal/power syntax safely. | `/tmp`, `@browser`, `>deploy -- prod`, `:`, `+`, `!`, `#`, `;todo ...`, `todo: ...`, `cal: ...` | Falls through to ordinary ScriptList, menu syntax, capture syntax, or another owner. |

Transient trigger text is exactly `~`, `/`, `@`, `>`, and `?`. These are control gestures, not durable launcher queries, and should not persist when returning to ScriptList.

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Narrow classifier | `special_entry_from_script_list_filter`. | Only routes exact committed trigger shapes. |
| First-character handoff | A one-character launcher filter that changes surface. | Dispatches before ordinary menu-syntax update. |
| Tilde prefix exception | `~` and `~/...` route to File Search Mini. | Bare `~` normalizes to `~/`; `~foo` is not a route by predicate. |
| Exact slash | `/` only. | Opens ACP slash picker; `/tmp` is not this route. |
| Exact mention | `@` only. | Opens ACP mention picker; `@browser` is not this route. |
| Exact terminal | `>` only. | Opens Quick Terminal; `>deploy -- prod` is not this route. |
| Conditional actions | `?` only. | Consumed by feature 013, but opens actions only when `has_actions()` is true. |
| Destination ownership | The opened surface owns behavior after handoff. | Feature 013 routes; it does not own File Search, ACP, Quick Terminal, or Actions internals. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| Main launcher filter input | `AppView::ScriptList`, shared filter focused. | `handle_filter_input_change` calls the special-entry classifier. |
| Programmatic filter changes | Any path that reaches the same filter-change handler. | Same classifier and route behavior. |
| Built-in Quick Terminal command | Built-in/triggerBuiltin utility path. | Calls the same `open_quick_terminal` helper, but is not the `>` special-entry classifier. |
| ACP entry request | Slash/mention helper after route. | Uses ACP entry/open logic and stages the picker trigger. |
| Actions toggle | `?` branch or Cmd+K elsewhere. | `?` is special-entry route; Cmd+K is normal actions routing and belongs to Actions. |

## User Workflows

### Open Mini File Search With `~`

The user types `~` in the main launcher. `special_entry_from_script_list_filter("~")` returns `FileSearchMini { query: "~/" }` because `normalize_mini_file_search_query("~")` converts bare tilde to `~/`.

`handle_filter_input_change` dispatches:

```text
open_file_search_view(query, FileSearchPresentation::Mini, cx)
```

File Search Mini takes over. It owns directory listing, path filtering, hidden-file behavior, selection, file actions, and later File Search-to-ACP handoffs. The tilde route seeds directory rows before first paint so the mini window does not show a zero-row flash while the async stream catches up.

### Open Mini File Search With `~/...`

The user types `~/src`. The classifier routes because `should_enter_file_search_from_script_list` accepts strings starting with `~/`. The query is preserved as `~/src`; only bare `~` normalizes.

Absolute paths like `/tmp` must not use this route. They fall through to ordinary ScriptList or another owner.

### Open ACP Slash Picker With `/`

The user types `/` by itself. The classifier returns `AcpSlashPicker`; the input-change handler calls:

```text
open_tab_ai_acp_with_slash_picker(window, cx)
```

The ACP helper stages `tab_ai_harness_script_list_trigger = Some('/')`, opens ACP through the normal ACP entry machinery, and defers embedded picker opening with `schedule_embedded_acp_picker_open`. Inside ACP, slash mode is command-oriented; context attachments and mention tokens belong to `@` mention mode.

### Open ACP Mention Picker With `@`

The user types `@` by itself. The classifier returns `AcpMentionPicker`; the input-change handler calls:

```text
open_tab_ai_acp_with_mention_picker(window, cx)
```

The ACP helper stages `tab_ai_harness_script_list_trigger = Some('@')`, opens ACP, and defers the mention/context picker. The ACP context composer then owns provider filtering, inline tokens, context parts, file/folder entries behind explicit `@file:` intent, and portal entries.

`@browser` is not a ScriptList handoff. It remains ordinary input at this layer.

### Open Quick Terminal With `>`

The user types `>` by itself. The classifier returns `QuickTerminal`; the input-change handler calls:

```text
open_quick_terminal(None, cx)
```

Quick Terminal switches the main view to `AppView::QuickTerminalView`, clears ordinary text-input focus, requests `FocusTarget::TermPrompt`, refills the warm PTY pool, and notifies GPUI.

From that point on, Quick Terminal owns behavior. It is not ACP Chat. Tab and Shift+Tab go to the PTY, Escape belongs to the terminal rather than global cancel, Cmd+W closes through the Quick Terminal close path, and Cmd+Enter applies back only when the terminal apply predicate says Apply is available.

`>deploy -- prod` is not a Quick Terminal handoff. It is deliberately left to ordinary/power-syntax handling.

### Open Actions/Help With `?`

The user types `?` by itself. The classifier returns `ActionsHelp`; the input-change handler checks `has_actions()`. When true, it calls `toggle_actions(cx, window)`.

When `has_actions()` is false, the route is consumed and no actions dialog opens. This is a disabled/no-op state, not literal `?` search.

### Preserve Power Syntax And Literal Text

Inputs such as `:`, `+`, `!`, `#`, `;todo Renew passport tomorrow`, `todo: Renew passport tomorrow`, `cal: Lunch next friday`, `/tmp`, `@browser`, and `>deploy -- prod` return `None` from the classifier. The rest of ScriptList processing may treat them as root search, menu syntax, source filters, capture syntax, or another feature.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open mini File Search at home. | Main launcher filter. | `AppView::ScriptList`. | Type `~`. | `special_entry_from_script_list_filter` -> `FileSearchMini` -> `open_file_search_view(..., Mini)`. | Mini File Search opens with `~/`. | `test_normalize_mini_file_search_query`, `tests/file_search_tilde_entry.rs`. |
| Open mini File Search at subpath. | Main launcher filter. | `AppView::ScriptList`. | Type `~/src`. | `should_enter_file_search_from_script_list` sees `~/`. | Mini File Search opens with `~/src`. | `test_should_enter_file_search_from_script_list`. |
| Keep absolute path literal. | Main launcher filter. | `AppView::ScriptList`. | Type `/tmp`. | Classifier returns `None`. | No ACP slash picker. | `test_special_entry_from_script_list_filter`. |
| Open ACP slash picker. | Main launcher filter. | `AppView::ScriptList`. | Type `/`. | `AcpSlashPicker` -> `open_tab_ai_acp_with_slash_picker`. | ACP opens with slash picker staged/deferred. | `tests/file_search_tilde_entry.rs`, ACP trigger ordering tests. |
| Keep mention-like text literal. | Main launcher filter. | `AppView::ScriptList`. | Type `@browser`. | Classifier returns `None`. | No ACP mention picker. | `test_special_entry_from_script_list_filter`. |
| Open ACP mention picker. | Main launcher filter. | `AppView::ScriptList`. | Type `@`. | `AcpMentionPicker` -> `open_tab_ai_acp_with_mention_picker`. | ACP opens with mention picker staged/deferred. | `tests/file_search_tilde_entry.rs`, ACP popup/mention registry tests. |
| Open Quick Terminal. | Main launcher filter. | `AppView::ScriptList`. | Type `>`. | `QuickTerminal` -> `open_quick_terminal(None, cx)`. | `QuickTerminalView` opens and terminal prompt gets focus. | `tests/file_search_tilde_entry.rs`, `tests/quick_terminal_contracts.rs`. |
| Keep terminal command literal. | Main launcher filter. | `AppView::ScriptList`. | Type `>deploy -- prod`. | Classifier returns `None`. | No Quick Terminal handoff. | `test_power_syntax_prefixes_do_not_route_to_special_surfaces`. |
| Open actions/help. | Main launcher filter. | `AppView::ScriptList` with actions. | Type `?`. | `ActionsHelp` -> `has_actions()` -> `toggle_actions`. | Shared actions dialog opens. | `tests/file_search_tilde_entry.rs`, actions popup contracts. |
| Disabled actions/help. | Main launcher filter. | No actions available. | Type `?`. | `ActionsHelp` -> `has_actions() == false`. | No dialog, no crash. | Source branch; runtime proof recommended. |
| Preserve menu/power syntax. | Main launcher filter. | `AppView::ScriptList`. | Type `:`, `+`, `!`, `#`, `;todo ...`, `todo: ...`, `cal: ...`. | Classifier returns `None`. | Owned by menu syntax/capture/ordinary search. | `test_power_syntax_prefixes_do_not_route_to_special_surfaces`. |

## State Machine

| State | Trigger | Transition | Destination owner |
|---|---|---|---|
| ScriptList shared filter | `~` | Normalize to `~/`, open `FileSearchView { presentation: Mini }`. | File Search. |
| ScriptList shared filter | `~/...` | Open `FileSearchView { presentation: Mini }` with typed query. | File Search. |
| ScriptList shared filter | `/` | Stage slash trigger, open ACP, defer slash picker. | ACP context composer. |
| ScriptList shared filter | `@` | Stage mention trigger, open ACP, defer mention picker. | ACP context composer. |
| ScriptList shared filter | `>` | Open `QuickTerminalView`, focus terminal prompt. | Quick Terminal. |
| ScriptList shared filter | `?` and actions available | Toggle shared actions dialog. | Actions popups. |
| ScriptList shared filter | `?` and no actions | Consume route without opening dialog. | Feature 013 disabled state. |
| ScriptList shared filter | Any non-route | Fall through to normal processing. | Main menu, menu syntax, source filters, capture syntax, or other owner. |

ACP and terminal close/return state matter after handoff. Agents should inspect `tab_ai_harness_script_list_trigger`, `pending_script_list_trigger`, `tab_ai_harness_return_view`, and `tab_ai_harness_return_focus_target` when changing return behavior.

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| ScriptList before handoff. | Main launcher filter and rows. | Main filter/list. | `filterText` before route, route logs. |
| Mini File Search from `~`. | Compact File Search with path query and seeded rows. | Shared filter/File Search selection. | `semanticSurface`/surface contract for File Search Mini, file-search receipts. |
| ACP slash picker. | Agent Chat with slash picker popup. | ACP composer/popup. | `promptPopup` / ACP picker state, expected popup id. |
| ACP mention picker. | Agent Chat with mention/context picker popup. | ACP composer/popup. | `promptPopup`, context picker rows, `acp-mention-popup` target. |
| Quick Terminal. | Compact terminal wrapper. | Terminal prompt. | `AppView::QuickTerminalView`, `FocusTarget::TermPrompt`, native footer surface `quick_terminal`. |
| Actions/help. | Shared actions dialog. | Actions search/list. | `actionsDialog`, popup target `actions-dialog`, parent surface metadata. |
| Non-route literal. | ScriptList remains active. | Main filter/list. | Classifier `None`, no `script_list_special_entry_routed` log. |

The special-entry branch returns before `set_menu_syntax_mode_from_filter(&new_text)` for the special trigger. Still verify stale decorations when transitioning from an existing decorated query to a special entry, because Oracle flagged this as a possible gap not fully proven by the retrieved chunks.

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| `~` | ScriptList | Opens mini File Search with `~/`. |
| `~/...` | ScriptList | Opens mini File Search with typed home-relative path. |
| `/` | ScriptList | Opens ACP slash picker. |
| `@` | ScriptList | Opens ACP mention/context picker. |
| `>` | ScriptList | Opens Quick Terminal. |
| `?` | ScriptList | Opens actions/help only if actions exist. |
| `Tab` / `Shift+Tab` | Quick Terminal after `>` | Written to PTY, not GPUI focus traversal. |
| `Escape` | Quick Terminal after `>` | Belongs to PTY; Quick Terminal wrapper uses Cmd+W/native close. |
| `Cmd+W` | Quick Terminal after `>` | State-first Quick Terminal close path. |
| `Cmd+Enter` | Quick Terminal after `>` | Apply-back only when Apply is available. |
| `Enter` in ACP picker | ACP after `/` or `@` | Accepts slash command or mention/context row. |
| `Cmd+K` | Any actions-capable host | Normal actions route, adjacent to but not owned by feature 013. |

## Actions And Menus

Feature 013 only invokes actions through the `?` special entry. It does not own action discovery, target capture, action row filtering, action execution, or Cmd+K.

When `?` routes and `has_actions()` is true, `toggle_actions(cx, window)` opens the shared actions dialog for the current host. When `has_actions()` is false, no dialog opens.

ACP slash and mention pickers are not ScriptList actions menus. They are ACP composer popups owned by the context-picker subsystem. Quick Terminal is also not an ACP menu; it is a terminal wrapper.

## Automation And Protocol Surface

| Surface | What to assert |
|---|---|
| Route logs | `script_list_special_entry_routed` with `entry_kind` for successful handoffs. |
| `getState` / preflight | Active view/semantic surface after route, filter text where exposed, focused input, popup state. |
| `getElements` | ACP prompt popup rows, actions dialog rows, Quick Terminal/native footer elements, File Search rows. |
| ACP popup proof | Use ACP runtime/popup receipts for slash and mention, not screenshots unless layout is the risk. |
| Quick Terminal proof | Assert `QuickTerminalView`, terminal focus, native footer surface, Cmd+W/Tab contracts. |
| File Search proof | Assert `FileSearchPresentation::Mini`, query normalization, seeded rows, mini sizing. |
| Actions proof | Assert `actionsDialog` only when `has_actions()` allows it and parent surface metadata is preserved. |

Recommended runtime smoke:

```text
1. Type "~" and assert mini File Search query "~/".
2. Return to ScriptList and assert "~" is not left as launcher query.
3. Type "~/src" and assert mini File Search query "~/src".
4. Type "/tmp" and assert ACP does not open.
5. Type "/" and assert ACP slash picker.
6. Type "@browser" and assert ACP does not open.
7. Type "@" and assert ACP mention picker.
8. Type ">" and assert Quick Terminal focus/terminal state.
9. Type ">deploy -- prod" and assert Quick Terminal does not open.
10. Type "?" with actions and assert actions dialog.
11. Type "?" without actions and assert no dialog/no crash.
12. Type ":", "+", "!", "#", "todo:", "cal:", ";todo ..." and assert no special-entry route.
```

## Data, Storage, And Privacy Boundaries

- The classifier reads only the current launcher filter text.
- Route logs include `filter_text`; `~/...` may include local path fragments and should be treated as path-bearing diagnostic data.
- The `~` route crosses into local filesystem browsing, but does not attach files to ACP unless a later explicit File Search action does so.
- `/` and `@` cross into Agent Chat. Slash mode is command-only; mention mode can expose context providers, file/folder rows for explicit file intent, and portal items.
- ACP entry can be blocked by an existing launcher attachment portal to avoid overwriting staged portal state.
- `>` crosses into a local PTY-backed shell. The bare `>` route passes `None` for cwd, unlike path actions that may open Quick Terminal at a directory.
- `?` crosses into shared actions metadata and must preserve parent surface metadata for automation.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Mini File Search first paint. | Seeded rows and display-index sizing avoid a zero-row flash. |
| File Search hidden entries. | Dot-prefixed filters such as `~/.` are File Search-owned after handoff. |
| ACP portal already open. | ACP entry request is blocked; no new slash/mention picker should overwrite portal state. |
| ACP first-run/setup. | ACP setup/onboarding state belongs to ACP after handoff. |
| Quick Terminal warm PTY unavailable. | Warm pool fails open; cold spawn can proceed. |
| Quick Terminal creation failure. | Logs error and shows a failure toast. |
| `?` without actions. | Route is consumed, no actions dialog opens, no crash. |
| Prompt popup proof failure. | Treat wrong-window/blank/ambiguous capture as infrastructure failure, not product proof. |
| Stale menu-syntax decoration. | Should not leak into destination surfaces; verify when touching this path. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Special-entry enum and classifier | `src/app_impl/filter_input_core.rs#ScriptListSpecialEntry`, `special_entry_from_script_list_filter`, `is_transient_script_list_trigger`, `should_enter_file_search_from_script_list`, `normalize_mini_file_search_query` |
| Runtime dispatch | `src/app_impl/filter_input_change.rs`, special-entry branch and `script_list_special_entry_routed` logs |
| Mini File Search handoff | `open_file_search_view`, `FileSearchPresentation::Mini`, mini sizing and seeded directory results |
| ACP slash/mention | `src/app_impl/tab_ai_mode/mod.rs#open_tab_ai_acp_with_slash_picker`, `open_tab_ai_acp_with_mention_picker`, `schedule_embedded_acp_picker_open` |
| ACP entry request | `src/app_impl/tab_ai_mode/acp_entry.rs#AcpEntryRequest` |
| ACP context picker | `src/ai/window/context_picker/mod.rs`, `src/ai/window/context_picker/types.rs` |
| Quick Terminal | `src/app_execute/utility_views.rs#open_quick_terminal`, `src/render_prompts/term.rs`, Quick Terminal close/apply/native-footer paths |
| Actions/help | `src/app_impl/actions_toggle.rs`, `src/app_impl/actions_dialog.rs` |
| Automation receipts | `src/main_window_preflight/types.rs`, `src/main_window_preflight/build.rs`, automation surface/popup collectors |
| Source contracts | `tests/file_search_tilde_entry.rs`, `tests/tab_ai_routing.rs`, `tests/quick_terminal_contracts.rs`, ACP popup/mention tests |

## Invariants And Regression Risks

- `ScriptListSpecialEntry` must stay narrow.
- `~` normalizes to `~/`; `~/...` preserves the typed path.
- `/tmp` must not open ACP slash picker.
- `/` and `@` are exact-only at the ScriptList layer.
- `@browser` must not open ACP mention picker from ScriptList.
- `>` is exact-only; `>deploy -- prod` must not open Quick Terminal.
- `?` must not open an empty actions dialog when `has_actions()` is false.
- Transient triggers must not persist as launcher search text after returning.
- Power syntax and menu syntax prefixes must remain owned by their own features.
- Slash/mention handoffs must stage the pending trigger before ACP opens.
- Embedded ACP picker opening must be deferred.
- Quick Terminal must remain distinct from ACP Chat.
- Quick Terminal opened from launcher must stay compact, not grow to SDK terminal height.
- Actions popup parent semantic surface must be preserved.
- Automation target ids for ACP prompt popups, actions dialogs, and Quick Terminal footer controls must remain stable.

## Verification Recipes

### Classifier

Run:

```bash
cargo test --lib filter_input_core::test_special_entry_from_script_list_filter -- --nocapture
cargo test --lib filter_input_core::test_normalize_mini_file_search_query -- --nocapture
cargo test --lib filter_input_core::test_is_transient_script_list_trigger -- --nocapture
cargo test --lib filter_input_core::test_power_syntax_prefixes_are_not_transient_triggers -- --nocapture
```

### ScriptList Dispatch

Run:

```bash
cargo test --test file_search_tilde_entry -- --nocapture
```

This checks the shared classifier call, Mini File Search route, ACP slash/mention helper calls, Quick Terminal route, `?` actions route, and seeded mini File Search sizing.

### ACP Slash/Mention

Run targeted source contracts if changing ACP handoff ordering:

```bash
cargo test --lib script_list_trigger_routes_stage_trigger_before_acp_open_contract -- --nocapture
cargo test --lib script_list_trigger_routes_defer_embedded_picker_contract -- --nocapture
```

Runtime proof path:

```bash
bun scripts/agentic/tx_wait_for_acp_runtime_semantics.ts
```

For popup layout or attachment proof, use the attached-popup matrix with `triggerBuiltin tab-ai`, `setAcpInput "/"`, and expected target `acp-mention-popup`.

### Quick Terminal

Run:

```bash
cargo test --test quick_terminal_contracts -- --nocapture
cargo test --test tab_ai_routing quick_terminal -- --nocapture
```

Check terminal-owned Tab/Shift+Tab, Escape pass-through, Cmd+W close, apply-back visibility, native footer, compact launcher height, and no regression into ACP Chat.

### Actions/Help

Run or adapt:

```bash
cargo test --test source_audits actions_popup_contract -- --nocapture
```

If parent-surface preservation is the risky behavior, locate the current named test in `tests/source_audits/actions_popup_contract.rs` or the adjacent actions popup contract tests before substituting.

### Hygiene

Run after atlas/doc updates:

```bash
lat check
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md
```

## Agent Notes

- Do not broaden `special_entry_from_script_list_filter` without a product decision, tests, and updated docs.
- Do not send `>deploy -- prod` to Quick Terminal from this classifier.
- Do not treat `@browser` as an ACP mention until ACP is active.
- Do not call Quick Terminal ACP Chat; it is PTY-backed and has terminal-owned input.
- Do not add generic action buttons to the Quick Terminal footer because launcher footer patterns happen to exist.
- If slash/mention handoff changes, verify embedded and detached ACP behavior separately.
- If return paths change, inspect `tab_ai_harness_script_list_trigger`, `pending_script_list_trigger`, `tab_ai_harness_return_view`, and `tab_ai_harness_return_focus_target`.
- If automation surfaces change, preserve the current parent/subview surface instead of hardcoding `scriptList`.
- Screenshots are only needed for popup placement, stale decoration, or strict target-capture regressions that state receipts cannot answer.

## Related Features

- [001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment](./001-main-menu.md)
- [002 File Search / Browser / Attachment Portals](./002-file-search.md)
- [003 Agent Chat Context Composer](./003-agent-chat-context.md)
- [011 Root Unified Search Result Actions](./011-root-source-actions.md)
- [012 Root Unified Source Filters / Source Chips / Lazy Paging](./012-root-source-filters.md)

## Open Questions And Gaps

- The raw pass proves early return before menu-syntax update but did not expose an explicit stale-decoration clear in the special-entry branch. Verify stale decoration behavior if this route is touched.
- Embedded ACP deferred picker opening is proven by the retrieved context; detached ACP slash/mention picker behavior should be inspected before edits.
- The bundle included preflight files, but Oracle did not receive enough direct snippets to map exact 013-specific receipt fields. Inspect `src/main_window_preflight/*` before adding more granular receipt claims.
- `?` no-actions behavior has a clear branch, but user-facing feedback is not visible in the retrieved context. Treat any toast or literal fallback as a behavior change.
