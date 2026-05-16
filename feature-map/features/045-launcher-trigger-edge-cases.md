# 045 Launcher Trigger Edge Cases

This chapter maps the boundary between ScriptList first-token handoffs, source filters, menu syntax, capture syntax, ACP picker triggers, Quick Terminal, and Actions Help.

Raw Oracle reference: [answer](../raw-oracle/045-launcher-trigger-edge-cases/answer.md), [prompt](../raw-oracle/045-launcher-trigger-edge-cases/prompt.md), [bundle map](../raw-oracle/045-launcher-trigger-edge-cases/bundle-map.md), [full log](../raw-oracle/045-launcher-trigger-edge-cases/output.log), [session metadata](../raw-oracle/045-launcher-trigger-edge-cases/session.json).

## Executive Summary

`ScriptListSpecialEntry` is intentionally tiny. The committed special-entry routes are `~` and `~/...` to Mini File Search, exact `/` to the ACP slash picker, exact `@` to the ACP mention picker, exact `>` to Quick Terminal, and exact `?` to Actions Help when actions exist.

Everything else belongs to another owner: source filters, menu syntax, capture syntax, ordinary launcher search, path-like text, or ACP composer parsing after ACP owns focus. Bugs in this area usually come from broadening a narrow trigger, leaving stale decorations after a handoff, or treating parser-owned tokens as surface transitions.

## Token Matrix

| Input | Owner / behavior | Must not do |
|---|---|---|
| `~` | Special entry; normalizes to `~/` and opens Mini File Search. | Do not leave it as literal search. |
| `~/` | Special entry; opens Mini File Search with query `~/`. | Do not decorate it as a source head. |
| `~/src` | Special entry; preserves the typed home-path query. | Do not normalize beyond bare `~`. |
| `~foo` | Negative route; not accepted by the documented home-path rule. | Do not broaden tilde matching to every `~*`. |
| `/` | Special entry; opens ACP with slash picker staged. | Do not treat it as `/tmp` path text. |
| `/tmp` | Ordinary/path text at this layer. | Do not open the ACP slash picker. |
| `@` | Special entry; opens ACP with mention/context picker staged. | Do not treat it as ordinary search. |
| `@browser` | Ordinary ScriptList text; ACP mention logic owns this only after ACP is active. | Do not open the ACP mention picker from ScriptList. |
| `>` | Special entry; opens Quick Terminal. | Do not parse it as command invocation. |
| `>deploy -- prod` | Parser-owned command text, not Quick Terminal. | Do not treat `>` as a shell escape. |
| `?` | Special entry; toggles actions only when `has_actions()` is true. | Do not render literal `?` results after a no-op route. |
| `f:`, `files:` | Root source filter for Files. | Do not open Mini File Search. |
| `n:`, `notes:` | Root source filter for Notes. | Do not route to ACP or capture. |
| `c:`, `clipboard:` | Root source filter for Clipboard History. | Do not treat it as a capture alias. |
| `ai:`, `conversations:` | Root source filter for saved AI conversations. | Do not open ACP Chat. |
| `:` | Menu-syntax discovery/refine trigger. | Do not commit `:f` as source syntax. |
| `+` | Menu/capture power syntax when applicable, otherwise ordinary search. | Do not add it to transient ScriptList triggers. |
| `!` | Menu-syntax command discovery/picker input. | Do not open Quick Terminal. |
| `#` | Plain launcher search by itself; `:#tag` is tag-filter sugar. | Do not make top-level `#tag` a filter. |
| `;target` | Capture syntax / capture picker-composer owner. | Do not route via `ScriptListSpecialEntry`. |
| `todo:`, `cal:` | Capture keyword aliases only when registered. | Do not special-route or source-filter them. |
| `target:` | Capture alias only if registered; otherwise literal/ordinary text. | Do not assume every trailing-colon word is a source filter. |

## Precedence Model

The root input pipeline has four important ownership stages.

| Stage | Contract |
|---|---|
| Subview handling | Built-in subviews with their own filter state sync their query and return before root ScriptList parsing. |
| Special-entry classifier | Mini File Search eligibility runs first, then exact `/`, `@`, `>`, and `?`. A match dispatches and returns. |
| Menu syntax parser | Runs only after the special-entry branch misses. Source filters, `:`, `;`, `!`, command invocation, capture aliases, and decorations live here. |
| Ordinary search | Receives text after special and parser-owned paths decline ownership. |

Source filters are advanced-query filters, not surface transitions. Capture aliases outrank source filters when a registered capture target owns the input; the `note: meeting f:` parser case preserves capture ownership instead of making `f:` dominant.

## Special Handoffs

Special-entry handoffs change the active surface. Source filters and menu syntax do not.

| Handoff | Destination | Notes |
|---|---|---|
| `~`, `~/...` | Mini File Search | Bare `~` becomes `~/`; home-path queries are preserved. |
| `/` | ACP slash picker | Stages `tab_ai_harness_script_list_trigger = Some('/')` before ACP launch. |
| `@` | ACP mention picker | Stages `tab_ai_harness_script_list_trigger = Some('@')` before ACP launch. |
| `>` | Quick Terminal | Calls the Quick Terminal opener with no command text. |
| `?` | Actions dialog or consumed no-op | Opens actions only if the current host has actions. |

## ACP Slash And Mention

Bare `/` and bare `@` in ScriptList are launch handoffs into ACP. Once ACP owns focus, slash and mention parsing belong to ACP composer/context-picker behavior.

For embedded ACP, the picker open is deferred until after first paint. The helper logs deferred picker opening, waits for the ACP first-paint delay, updates the parent window, and opens the slash or mention picker inside the ACP view.

Detached ACP is still a proof gap. The launch path captures the pending ScriptList trigger and passes it into detached ACP launch input, but the exact visible picker timing, focus owner, and selected row need targeted receipts before the atlas can claim them as settled.

## Source Filters

Root source filters keep `AppView::ScriptList` active. They strip source heads from free text, record include/exclude source filters, suppress unrelated sources, and expose status metadata rather than navigable status rows.

Committed source heads:

| Source | Heads |
|---|---|
| Files | `files:`, `f:` |
| Notes | `notes:`, `n:` |
| Clipboard | `clipboard:`, `c:` |
| Browser tabs | `tabs:`, `t:` |
| History | `history:`, `h:` |
| Apps | `apps:`, `a:` |
| Scripts | `scripts:`, `s:` |
| Commands | `commands:`, `cmd:` |
| Conversations | `conversations:`, `ai:` |
| Vault | `vault:`, `v:` |
| Dictation | `dictation:`, `d:` |
| Windows | `windows:`, `w:` |

`processes:` and `p:` are intentionally uncommitted until root process rows exist. Attached source queries such as `c:skip`, `f:s`, `files: sc`, and `h:https://example.com` are valid source-filter inputs. Source-only inputs with trailing space keep an empty stripped query and show browse/default rows.

## Menu Syntax And Capture

Leading `:` owns advanced-query discovery and refinement. Bare and partial states such as `:`, `:typ`, `:type:`, `:type:s`, `:has:sh`, and `:#` remain discovery states; complete predicates such as `:type:skill review` or `:#work notes` become advanced queries.

Capture syntax is parser-owned. `;target` enters capture target or composer behavior. `todo:` and `cal:` are capture keyword aliases only when registered by capture targets. Generic `target:` remains ordinary text unless a capture alias or known source/property head owns it.

The menu-syntax hint is read-only panel state, not runnable list data. Automation should inspect `stateResult.menuSyntaxMainHint` and popup elements instead of attempting to select hint rows.

## Visual And Focus States

Decoration rendering must be replace-not-merge. The tokenizer computes the full span set on each input update, and the render path must install that set even when it is empty.

This matters for transitions such as `f: report` to `~/`, `c:skip` to `/`, `n:` to `@`, `files:s` to `>`, and `:type:script` to `?`. The target surface should not inherit source chips, power-syntax accents, stale popup rows, or stale menu hints from the previous input.

Do not claim stale decorations are impossible. The proof source is live `filterInputDecorations` from GPUI state, not a recomputed tokenizer result in a test helper.

## Automation Receipts

| Surface / state | Receipt to assert |
|---|---|
| Special routes | `script_list_special_entry_routed` log with `entry_kind` and current view. |
| ScriptList decorations | `getState.stateResult.filterInputDecorations`. |
| Source filters | Main-window preflight or state result with stripped query, source filters, and filter indicators. |
| Menu syntax | `stateResult.menuSyntaxMainHint` and `getElements` for `menu-syntax-trigger-popup`. |
| Embedded ACP | `getAcpState`, ACP popup target elements, parent identity, picker rows, and focus. |
| Detached ACP | Exact `acpDetached` target with `getAcpState(target)` and `inspectAutomationWindow(target)`. |
| Actions dialog | `actionsDialog` target with action ids, labels, host/context, or absence when `has_actions()` is false. |
| Mini File Search | Active File Search Mini surface, `~/...` query, rows, selection/focus, and no stale ScriptList chips. |
| Quick Terminal | `QuickTerminalView`, `FocusTarget::TermPrompt`, terminal elements, and native footer controls. |

## Verification Recipes

Existing narrow source checks:

```bash
cargo test --lib filter_input_core::test_special_entry_from_script_list_filter -- --nocapture
cargo test --lib filter_input_core::test_normalize_mini_file_search_query -- --nocapture
cargo test --lib filter_input_core::test_is_transient_script_list_trigger -- --nocapture
cargo test --lib filter_input_core::test_power_syntax_prefixes_are_not_transient_triggers -- --nocapture
cargo test --test file_search_tilde_entry -- --nocapture
cargo test --test menu_syntax_source_filters -- --nocapture
cargo test --test tab_ai_routing -- --nocapture
cargo test --test acp_main_menu_skill_launch_contract -- --nocapture
cargo check --lib
cargo fmt --check
git diff --check
lat check
```

Existing adjacent runtime checks:

```bash
bun scripts/agentic/root-source-filter-stability.ts
bun scripts/agentic/root-source-filter-clipboard.ts
bun scripts/agentic/root-source-filter-history-up.ts --timeout 12000
bun scripts/agentic/source-chip-pagination-proof.ts --timeout 16000
bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000
```

Recommended new proof: `bun scripts/agentic/launcher-trigger-edge-cases.ts`.

It should drive `f: report -> ~/`, `c:skip -> /`, `n: -> @`, `files:s -> >`, and `:type:script -> ?`, then assert the target surface plus cleared prior decorations. It should also assert negative routes for `/tmp`, `@browser`, `>deploy -- prod`, `:`, `+`, `!`, `#`, `;todo`, `todo:`, `cal:`, `target:`, `f:`, `files:`, `n:`, `c:`, and `ai:`.

## Open Gaps

- Add direct classifier tests for `~/`, `~foo`, all committed source heads, `:`, `+`, `#`, `;todo`, `todo:`, `cal:`, and `target:`.
- Add runtime decoration-transition proof from decorated text into `~/`, `/`, `@`, `>`, and `?`.
- Add exact-id detached ACP proof for `/` and `@`, including picker visibility, focus owner, initial input, return origin, and focused-row context suppression.
- Add embedded ACP proof that validates deferred picker opening after first paint using state receipts.
- Add a no-actions fixture proving `?` is consumed without opening actions or falling through to literal search.
- Add capture alias boundary proof for registered and unregistered `todo:`, `cal:`, and `target:` cases.
- Add source-filter boundary proof that `f:`, `files:`, `n:`, `c:`, and `ai:` remain ScriptList source filters.

## Agent Notes

- Do not say `/tmp` opens ACP, Mini File Search, or a filesystem browser.
- Do not say `@browser` opens the ACP mention picker from ScriptList.
- Do not say `>` is a shell escape or that `>deploy -- prod` opens Quick Terminal.
- Do not say `?` always opens Actions; `has_actions() == false` is a consumed no-op.
- Do not say `f:` or `files:` opens Mini File Search.
- Do not say source-chip status is a selectable row or action subject.
- Do not say `processes:` or `p:` is committed source syntax until parser tests and root process rows align.
- Do not call Quick Terminal Agent Chat or ACP.
- To verify decoration clearing, inspect live `filterInputDecorations`, not only parser output.
- Screenshots are only needed after state receipts cannot answer a visual or focus question.
