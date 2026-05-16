# 016 Prompt Runtime Core / arg() / select() / div() / md()

This chapter maps the core SDK prompt surfaces for typed input, choice lists, rendered HTML, and Markdown-to-HTML helper output.


## Executive Summary


| SDK API | Runtime surface | Return shape | Primary use |
|---|---|---|---|
| `md()` | SDK helper only | `string` HTML | Converts Markdown to HTML for `div()` or other HTML-consuming surfaces. |


SDK TermPrompt, Quick Terminal, and ACP Chat are adjacent features, not part of this chapter.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Ask for typed text. | `await arg("Name")` | Opens an arg prompt; user types; Enter returns a string. |
| Ask from choices. | `await arg("Pick one", choices)` | Opens arg prompt with filterable choices; Enter returns selected choice value or typed text depending on state. |
| Use arg actions. | `await arg(configOrPlaceholder, choices, actions)` | Opens arg-scoped actions via Cmd+K/action routing. |
| Select multiple values. | `await select("Pick", choices)` | Opens select prompt; toggles choices; submit returns string array. |
| Render HTML. | `await div("<h1>Hello</h1>")` | Opens div prompt with rendered content and no text input focus. |
| Render Markdown. | `await div(md(markdown))` | Converts Markdown to HTML, then renders in div prompt. |
| Use div actions. | `await div(htmlOrConfig, actions)` | Opens div-scoped action menu and returns submitted value or void. |
| Automate prompts. | `getState`, `getElements`, `simulateKey`, semantic selectors. | Agents inspect prompt type/id, input/selection state, visible choices/elements, and submit through safe paths. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Prompt id | SDK `nextId()` id stored in pending resolver and app view. | Submit must resolve the matching id only. |
| App view | Rust prompt route installed for active prompt. | `ArgPrompt`, `SelectPrompt`, and `DivPrompt` have distinct focus/render/submit behavior. |
| Submit callback | Rust callback from prompt to SDK response. | Sends prompt response to pending SDK resolver. |
| Receipts | Protocol/test state and element outputs. | Prompt type, prompt id, input value, selected value, and semantic elements should be asserted directly. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `globalThis.arg`. | Script requests one text/choice value. | Sends arg prompt message and resolves `Promise<string>`. |
| `globalThis.div`. | Script requests rendered HTML panel. | Sends div prompt message and resolves submitted string or void. |
| `globalThis.md`. | Script converts Markdown to HTML. | Returns HTML synchronously; does not send a runtime prompt message. |
| Render dispatch. | Main view renders prompt app view. | Dispatches to `render_arg_prompt`, `render_select_prompt`, or `render_div_prompt`. |
| Automation. | Protocol drives or inspects prompt. | Uses prompt type/id state, elements, simulated keys, semantic selectors, and force submit where supported. |

## User Workflows

### Arg Free-text Submit


```ts
const name = await arg("Name")
```


### Arg Choice Submit


```ts
const value = await arg("Pick one", choices)
```

The prompt opens with choices and text input focus. Typing filters the list. If a filtered choice is selected, Enter submits the selected choice value. Automation can use `selectByValue`, `selectBySemanticId`, or `selectFirst` on choice-backed arg/mini/micro prompts.

### Arg No Choices


### Arg Actions


### Select Multi-select


```ts
const values = await select("Pick", choices)
```


### Select Internal Single-select


### Div HTML Prompt


```ts
const result = await div("<h1>Hello</h1>")
```


### Markdown To Div


```ts
await div(md(markdown))
```

`md()` converts Markdown to HTML on the SDK side. It is synchronous and not a prompt. `div()` then sends the HTML to the app.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Ask for one choice. | `arg("Prompt", choices)`. | Arg with choices, selected index 0. | Filter, Enter. | Filtered arg choices -> submit outcome. | Selected choice value resolves. | Prompt handler state helpers; arg helpers. |
| Submit arg by automation. | Protocol/test helper. | Choice-backed prompt. | `selectByValue(..., submit=true)`. | `select_by_value` supports arg/mini/micro. | Value submitted safely. | `src/prompt_handler/mod.rs`. |
| Dismiss arg actions. | Arg actions dialog. | Actions popup. | Escape/backdrop. | `mark_actions_popup_closed`; backdrop helper. | Menu closes, arg focus restored. | simulateKey route; arg backdrop tests. |
| Toggle selected row. | Select prompt. | Focused list row. | Space/intent/click. | `ToggleFocusedSelection`; row mouse handler. | Selection changes without submit in multi mode. | `src/prompts/select/prompt.rs`; `src/prompts/select/render.rs`. |
| Single-select row activation. | Internal select with `multiple=false`. | Select list. | Click row. | Row handler branches to `submit()`. | Immediate submit. | `src/prompts/select/render.rs`; public API exposure unproven. |
| Render Markdown panel. | `div(md(markdown))`. | Div prompt HTML. | Submit/link as div. | `md()` returns HTML; `div()` sends HTML. | Markdown appears as HTML. | `scripts/kit-sdk.ts`; `tests/sdk/test-md.ts`. |
| Inspect active prompt. | Automation. | Any active core prompt. | `getState`/probe. | `current_prompt_type`, `current_input_value`, `current_selected_value`. | Receipt includes type/id/input/selection. | `src/prompt_handler/mod.rs`. |
| Enumerate visible elements. | Automation. | Arg/div/select. | `getElements`. | `collect_elements`. | Arg choices, div panel, select elements. | `src/app_layout/collect_elements.rs`. |
| Force-submit. | Protocol. | Arg/div/form/term/editor. | ForceSubmit value. | Current view id match then prompt response. | Pending SDK promise resolves. | Select not proven supported in visible match. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK idle. | No prompt call active. | Script continues normally. | No pending prompt id. |
| SDK request created. | `arg`, `select`, or `div` called. | Create id, normalize input, register pending resolver, send typed message. | `md()` does not enter this state. |
| Rust message received. | Prompt handler receives typed message. | Creates submit callback and prompt-specific app view. | Id must match SDK pending resolver. |
| Actions open. | Prompt actions shortcut/menu. | Opens host-scoped actions dialog. | Arg/div hosts proven; select actions unproven. |
| Submit. | Enter/click/protocol submit. | Submit active prompt id with value. | SDK pending promise resolves. |
| Cancel. | Escape/cancel route. | May submit `None` for arg in visible simulateKey path. | Exact SDK cancellation result needs tests. |
| Stale async guard. | Delayed work fires after prompt changes. | Current view/id matching protects active prompt. | Prevents resolving/mutating wrong prompt. |

## Visual And Focus States

| Surface | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Div prompt. | Rendered HTML panel. | App root / no text input. | Prompt type `div`; element `div-prompt`; empty current input value. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Text input | Arg prompt. | Updates arg input and filters choices. |
| Enter | Arg prompt. | Submits current arg outcome. |
| Cmd+K | Arg prompt. | Toggles arg actions in visible simulateKey route. |
| Escape | Arg actions. | Closes action dialog and restores arg focus. |
| Escape | Arg prompt. | Visible simulateKey path submits/cancels with `None`; SDK result needs verification. |
| Arrow/list movement | Select prompt. | Select-owned keyboard intent moves focus. |
| Toggle focused selection | Select prompt multi-select. | Toggles row selection without submit. |
| Toggle all filtered | Select prompt multi-select. | Toggles filtered rows. |
| Submit | Select prompt. | Resolves selected string array. |
| Click row | Select multi-select. | Toggles. |
| Click row | Select single-select internal mode. | Submits immediately. |
| Div keys/clicks | Div prompt. | Host-owned prompt key preamble and rendered HTML submit/link behavior; exact details need div source expansion. |

## Actions And Menus

| Surface | Host | Backdrop | Notes |
|---|---|---|---|
| Select | Unproven in tight bundle. | Unproven. | Do not claim select SDK actions until source confirms. |

Action host confusion is high-risk. If arg actions use div host, or div actions use arg host, shortcuts, backdrop dismissal, focus restoration, and action execution can target the wrong prompt.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `current_prompt_type`. | Returns `arg`, `div`, `select`, and adjacent prompt names. |
| `current_input_value`. | Returns arg input for arg/mini/micro; returns empty string for div/select. |
| `current_selected_value`. | Computes filtered selected arg choice. |
| Arg `selectByValue` / `selectBySemanticId` / `selectFirst`. | Visible helpers support choice-backed arg/mini/micro prompts. |
| Select automation. | Use select element receipts and select-owned key/click behavior; do not assume arg selection helpers apply. |
| Div `getElements`. | Exposes panel element with semantic id/name `div-prompt`. |
| ForceSubmit. | Visible match supports arg and div but not select. |
| Simulated Enter/Escape. | Arg/mini prompt routing is visible; select/div parity should be proven by tests/receipts before relying on it. |

## Data, Storage, And Privacy Boundaries

- Arg typed text is transient prompt input in `self.arg_input` and exposed through automation state while active.
- Choice labels/values/metadata cross the SDK/app boundary and may be exposed through element collection.
- Select state lives in `SelectPrompt` and returns `string[]` to SDK.
- Div HTML is script-provided rendered content; it may include links or submit-bearing elements.
- Markdown source passed to `md()` is converted in SDK; the resulting HTML is what `div()` sends.
- SDK actions serialize action metadata and handler ids while the prompt is active.
- Return values resolve the SDK pending resolver for the active prompt id.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Empty arg choices. | Explicit no-choice layout; typed input can still be valid unless disabled elsewhere. |
| Empty select choices. | Uses a no-list visual layout; verify with SDK/smoke tests because naming overlaps arg no-choice sizing. |
| Empty div content. | Likely valid blank panel because `div()` accepts optional input; exact default HTML needs source proof. |
| Loading choices. | Not proven in bundle. |
| Disabled choices. | Not proven in bundle. |
| Stale prompt id. | Submit/force-submit should match active current-view id. |
| Cancellation. | Arg simulateKey Escape submits `None`; SDK cancellation result needs prompt-flow tests. |

## Code Ownership

| Area | Files |
|---|---|
| SDK prompt globals | `scripts/kit-sdk.ts` |
| Prompt routing/state helpers/force submit | `src/prompt_handler/mod.rs` |
| App view identity/native footer mapping | `src/main_sections/app_view_state.rs` |
| Top-level render dispatch | `src/main_sections/render_impl.rs` |
| Arg rendering/helpers | `src/render_prompts/arg/render.rs`, `src/render_prompts/arg/helpers.rs` |
| Div wrapper rendering | `src/render_prompts/div.rs` |
| Select prompt internals | `src/prompts/select/prompt.rs`, `src/prompts/select/render.rs`, `src/prompts/select/search.rs`, `src/prompts/select/types.rs` |
| Div internals | `src/prompts/div/prompt.rs`, `src/prompts/div/render.rs`, `src/prompts/div/render_html.rs` |
| Markdown internals | `src/prompts/markdown/`, plus SDK `md()` helper in `scripts/kit-sdk.ts` |
| Element collection | `src/app_layout/collect_elements.rs` |
| Simulated input | `src/main_entry/runtime_stdin_match_simulate_key.rs` |
| Tests | `tests/sdk/test-arg.ts`, `tests/sdk/test-select.ts`, `tests/sdk/test-div.ts`, `tests/sdk/test-md.ts`, `tests/sdk/test-prompt-flow.ts`, and related smoke tests. |

## Invariants And Regression Risks

- Prompt ids must remain stable from SDK send to Rust submit.
- Arg must clear input/filter and reset selected index on entry.
- Arg focus must restore after actions close.
- Arg submit parity must hold across real Enter, simulated Enter, and safe automation helpers.
- Select multi-select row clicks must toggle, not submit.
- Div must not steal text-input focus or mutate hidden arg input state.
- Arg/div action hosts and backdrop ids must stay distinct.
- ForceSubmit support is not uniform; do not assume select support.
- Element semantic ids must remain stable for automation.
- `md()` is SDK-side HTML generation, not a runtime prompt.

## Verification Recipes


```bash
bun tests/sdk/test-arg.ts
bun tests/sdk/test-select.ts
bun tests/sdk/test-div.ts
bun tests/sdk/test-md.ts
bun tests/sdk/test-prompt-flow.ts
bun tests/smoke/test-arg-actions-cmdk.ts
bun tests/smoke/test-arg-text-submit.ts
bun tests/smoke/test-div-submit-links.ts
bun tests/smoke/test-select-actions-cmdk.ts
bun tests/smoke/test-md-div-integration.ts
source checks
```


1. Open arg, assert prompt type `arg`, id, current input, selected choice, and Enter submit.
2. Open arg with actions, assert Cmd+K opens arg actions and Escape/backdrop restores arg focus.
3. Open select, assert prompt type `select`, list elements, toggle behavior, selected array result, and empty choices behavior.
4. Open div, assert prompt type `div`, `div-prompt` element, no current input value, and submit/link result.
5. Open `div(md(...))`, assert Markdown converted to expected HTML rendering.
6. Prove ForceSubmit only on supported surfaces unless expanded source confirms select.

## Agent Notes

Do not mix core prompt runtime with SDK terminal, Quick Terminal, or ACP Chat.

When automating arg, prefer semantic selection helpers over coordinates. When automating select, do not assume arg helpers apply. When automating div, treat it as a panel plus submit/link surface, not a text prompt.

When using ForceSubmit, use it for arg/div from this bundle; select support is unproven.

When documenting tests, distinguish "test exists in bundle map" from "test was run."

## Related Features

- [015 SDK TermPrompt](./015-sdk-term-prompt.md) is adjacent through prompt handler submit/force-submit logic, but terminal behavior is out of scope.
- [014 Quick Terminal](./014-quick-terminal-pty.md) is unrelated to arg/select/div/md except for shared app infrastructure.
- ACP Chat is a separate AI/chat surface.
- Form, editor, path, drop, env, template, emoji, and naming prompts share prompt-routing infrastructure but need separate chapters.
- Actions dialogs and protocol automation are cross-cutting dependencies.

## Open Questions And Gaps

- Full `ArgConfig`, `DivConfig`, `Choice`, and `SerializableAction` field inventories need a wider source pass.
- Div HTML rendering internals, link sanitization, allowed HTML, submit attributes, and Markdown rendering rules need a dedicated pass.
- `md()` full Markdown support beyond fenced-code conversion needs source/test expansion.
- Select ForceSubmit support is not shown.
- Select actions/Cmd+K are not proven despite a smoke-test name; expand select action code before claiming parity.
- Cancellation semantics are partially visible only for arg.
- Loading and disabled choice states are not proven.
