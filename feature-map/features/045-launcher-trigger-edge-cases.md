# 045 Launcher Trigger Edge Cases

This chapter maps the boundary between ScriptList first-token handoffs, source filters, menu syntax, capture syntax, ACP picker triggers, Quick Terminal, and Actions Help.


## Executive Summary

`ScriptListSpecialEntry` is intentionally tiny. The committed special-entry routes are `~` and `~/...` to Mini File Search, exact `/` to the ACP slash picker, exact `@` to the ACP mention picker, exact `>` to Quick Terminal, and exact `?` to Actions Help when actions exist.


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
| `+` | Menu/capture power syntax when applicable, otherwise ordinary search. | Do not add it to transient ScriptList triggers. |
| `!` | Menu-syntax command discovery/picker input. | Do not open Quick Terminal. |
| `;target` | Capture syntax / capture picker-composer owner. | Do not route via `ScriptListSpecialEntry`. |

## Precedence Model

The root input pipeline has four important ownership stages.

| Stage | Contract |
|---|---|
| Subview handling | Built-in subviews with their own filter state sync their query and return before root ScriptList parsing. |
| Special-entry classifier | Mini File Search eligibility runs first, then exact `/`, `@`, `>`, and `?`. A match dispatches and returns. |
| Ordinary search | Receives text after special and parser-owned paths decline ownership. |


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



| Source | Heads |
|---|---|


## Menu Syntax And Capture



The menu-syntax hint is read-only panel state, not runnable list data. Automation should inspect `stateResult.menuSyntaxMainHint` and popup elements instead of attempting to select hint rows.

## Visual And Focus States

Decoration rendering must be replace-not-merge. The tokenizer computes the full span set on each input update, and the render path must install that set even when it is empty.


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

## Verification Recipes


```bash
cargo test --test file_search_tilde_entry -- --nocapture
cargo test --test menu_syntax_source_filters -- --nocapture
cargo test --test tab_ai_routing -- --nocapture
cargo test --test acp_main_menu_skill_launch_contract -- --nocapture
cargo check --lib
cargo fmt --check
git diff --check
source checks
```


```bash
bun scripts/agentic/root-source-filter-stability.ts
bun scripts/agentic/root-source-filter-clipboard.ts
bun scripts/agentic/root-source-filter-history-up.ts --timeout 12000
bun scripts/agentic/source-chip-pagination-proof.ts --timeout 16000
bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000
```



## Open Gaps

- Add runtime decoration-transition proof from decorated text into `~/`, `/`, `@`, `>`, and `?`.
- Add exact-id detached ACP proof for `/` and `@`, including picker visibility, focus owner, initial input, return origin, and focused-row context suppression.
- Add embedded ACP proof that validates deferred picker opening after first paint using state receipts.
- Add a no-actions fixture proving `?` is consumed without opening actions or falling through to literal search.

## Agent Notes

- Do not say `/tmp` opens ACP, Mini File Search, or a filesystem browser.
- Do not say `@browser` opens the ACP mention picker from ScriptList.
- Do not say `>` is a shell escape or that `>deploy -- prod` opens Quick Terminal.
- Do not say `?` always opens Actions; `has_actions() == false` is a consumed no-op.
- Do not say source-chip status is a selectable row or action subject.
- Do not call Quick Terminal Agent Chat or ACP.
- To verify decoration clearing, inspect live `filterInputDecorations`, not only parser output.
- Screenshots are only needed after state receipts cannot answer a visual or focus question.
