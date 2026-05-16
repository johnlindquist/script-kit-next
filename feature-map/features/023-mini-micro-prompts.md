# 023 Mini and Micro Prompts / mini() / micro()

This chapter maps compact SDK choice prompts and the boundary between MiniPrompt, MicroPrompt, Mini window mode, and microphone stubs.


## Executive Summary


```ts
```


Treat the SDK warning as stale product copy, not as the actual capability boundary.


`MicroPrompt` is also choice-backed and arg-like internally, but visually ultra-compact and footerless. It stays off native-footer routing and must remain distinct from `mic()` / microphone media stubs.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Open compact choice prompt. | `await mini("Pick", ["A", "B"])`. | Shows MiniPrompt and resolves selected value. |
| Open ultra-compact prompt. | `await micro("Pick", ["A", "B"])`. | Shows MicroPrompt and resolves selected value. |
| Use structured choices. | Choice objects with `name` and `value`. | Submitted value is `choice.value`. |
| Filter choices. | Type into prompt. | Visible choices filter by choice name in captured code. |
| Submit selected choice. | Enter or automation submit. | SDK resolves a string. |
| Cancel. | Escape. | SDK cancellation collapses to `""` in current shape. |
| Inspect state. | `getState`, `getElements`. | Agents see prompt type, input, choice counts, visible choices, selected value. |
| Select by automation. | Batch selection helpers. | Mini/Micro are included in shared choice helper support. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| SDK warning | Stale warning text. | User-visible but contradicts Rust support. |
| `MiniPrompt` | Compact arg-like choice prompt. | Has native footer and compact MiniPrompt sizing. |
| `MicroPrompt` | Ultra-compact arg-like choice prompt. | Footerless and off native footer routing. |
| Shared arg state | `arg_input`, selected index, filtered choices. | Mini/Micro reuse arg-like filtering and selection. |
| Mini window mode | Main-window mode. | Separate from SDK `mini()`. |
| `mic()` | Media/microphone stub. | Separate from SDK `micro()`. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `render_mini_prompt`. | Render dispatch. | Minimal list prompt shell with footer. |
| `render_micro_prompt`. | Render dispatch. | Ultra-compact no-footer prompt. |
| `collect_elements`. | Automation. | Shared choice elements for Mini and Micro. |

## User Workflows

### Mini Choice Prompt


```ts
const value = await mini("Pick fruit", ["Apple", "Banana", "Cherry"])
```


### Micro Choice Prompt


```ts
const value = await micro("Pick fruit", ["Apple", "Banana", "Cherry"])
```


### Filtering

Mini/Micro filter over shared arg-like choice state. The captured matching path filters lowercased `choice.name` against the prompt input. This means `choice.value`, metadata, or description matching should not be assumed without source proof.

### Cancellation

Escape cancels Mini through the captured simulateKey path. SDK Promise cancellation collapses to `""`, so cancellation is not distinguishable from an intentional empty value with the current return shape. Micro cancellation parity needs direct proof.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Filter choices. | Mini/Micro active. | Input text. | Type/setInput. | Shared arg input and filtered choices. | visibleChoiceCount changes. | `src/app_impl/prompt_ai.rs`, `src/prompt_handler/mod.rs`. |
| Submit Mini. | MiniPrompt active. | Selected choice. | Enter. | `submit_arg_prompt_from_current_state`. | SDK resolves value. | `src/render_prompts/mini.rs`, simulateKey route. |
| Submit Micro. | MicroPrompt active. | Selected choice. | Batch helper / physical path. | Shared submit helpers include Micro. | SDK resolves value. | `src/prompt_handler/mod.rs`; simulateKey gap noted. |
| Inspect Mini. | Protocol. | Mini active. | `getState`, `getElements`. | State and choice collectors. | Prompt type `mini`, choices, footer. | `src/app_layout/collect_elements.rs`. |
| Inspect Micro. | Protocol. | Micro active. | `getState`, `getElements`. | State and choice collectors. | Prompt type `micro`, choices, no footer. | `src/app_layout/collect_elements.rs`. |
| Cancel Mini. | Mini active. | Prompt active. | Escape. | simulateKey Mini arm. | Prompt cancels. | `runtime_stdin_match_simulate_key.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK call. | `mini()` / `micro()`. | Normalize choices, send message. | Stale warning logs first. |
| Rust route. | `ShowMini` / `ShowMicro`. | Reset shared arg state and install app view. | Mini/Micro share choice state. |
| Visible. | Render dispatch. | Mini footer shell or Micro no-footer shell. | Sizing differs. |
| Filtering. | Input changes. | Filtered choices recomputed. | `inputValue` should remain verbatim. |
| Selection. | Arrow/batch/click. | selected index/value updates. | Choice counts remain distinct. |
| Submit. | Enter/batch/physical. | Submit selected or current value. | SDK resolves string. |
| Cancel. | Escape/cancel. | Submit None/cancel script. | SDK collapses to `""`. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Mini filtered. | Compact list narrowed. | Mini prompt. | `inputValue`, `visibleChoiceCount`, selected value. |
| Micro filtered. | Ultra-compact filtered choices. | Micro prompt. | Shared choice elements. |
| No matches. | Empty visible list / typed fallback possible. | Prompt focus. | Runtime proof needed per prompt. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Text input. | Mini/Micro. | Updates prompt input/filter. |
| Enter. | MiniPrompt. | Submits current prompt state. |
| Enter. | MicroPrompt. | Submit supported by shared helpers; simulateKey parity not proven. |
| Escape. | MiniPrompt. | Cancels in simulateKey path. |
| Escape. | MicroPrompt. | Needs direct proof. |
| Cmd+K. | MiniPrompt. | May route into shared arg actions if actions exist; proof depends on source path. |
| Batch select by value/id. | Mini/Micro. | Shared choice helpers include both. |

## Actions And Menus

Mini/Micro are arg-like choice prompts. Mini can participate in native footer/hints and shared action handling where the arg-like action infrastructure includes it. Micro is footerless and should not grow a native footer or action strip without a product decision. Any actions support should be proven separately because the public SDK signatures do not expose actions.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getElements`. | Shared choice row elements for both. |
| `inputValue`. | Verbatim prompt input, subject to stdin command byte cap only. |
| Batch select by value/semantic id. | Shared helper supports Mini and Micro. |
| simulateKey Mini Enter. | Proved to submit current state. |
| simulateKey Micro Enter. | Gap in captured source. |
| Layout info. | Mini maps to mini; Micro maps to micro. |
| Active footer. | Mini has `mini_prompt`; Micro has none. |

## Data, Storage, And Privacy Boundaries

- Mini/Micro typed input is transient prompt input and exposed through automation state.
- Choices and choice metadata can be exposed through elements.
- Submitted values return to the script; there is no persistence unless the script persists the result.
- Cancellation currently resolves indistinctly from empty value in the SDK.
- InputValue is intentionally verbatim and should not be truncated or normalized by the app beyond stdin line caps.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| SDK warning. | Stale warning says not implemented even though Rust support exists. |
| Empty choices. | SDK fallback result is `""`; runtime no-choice behavior needs proof. |
| No filter matches. | Shared arg helpers imply fallback/invalid behavior; Mini/Micro need runtime proof. |
| Cancellation. | Collapses to `""` in SDK. |
| Micro no footer. | Expected state, not missing UI. |
| Mini sizing mismatch. | ShowMini immediate resize may use ArgPrompt view types; runtime proof needed. |
| Micro simulateKey. | Not proven. |
| Loading. | Static choices only; no loading state proven. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK APIs. | `scripts/kit-sdk.ts` owns `mini()`, `micro()`, warnings, choice normalization, response parsing. |
| Protocol/routing. | `src/execute_script/mod.rs`, `src/prompt_handler/mod.rs`, prompt messages. |
| App view identity. | `src/main_sections/app_view_state.rs` owns Mini/Micro variants and native footer mapping. |
| Rendering. | `src/render_prompts/mini.rs`, `src/render_prompts/micro.rs`, `render_impl.rs`. |
| Shared choice behavior. | `src/render_prompts/arg/helpers.rs`, `src/app_impl/prompt_ai.rs`, prompt handler state helpers. |
| Footer/window sizing. | `src/app_impl/ui_window.rs`, `src/window_resize/mod.rs`, `tests/mini_window_sizing_contract.rs`. |
| Automation. | `src/app_layout/collect_elements.rs`, `build_layout_info`, `runtime_stdin_match_simulate_key.rs`, Tab AI tests. |

## Invariants And Regression Risks

- Do not treat stale SDK warnings as actual absence of Mini/Micro.
- Do not conflate SDK `mini()` with Mini main-window mode or Mini AI.
- Do not conflate `micro()` with `mic()`.
- Micro must remain footerless and off native footer routing.
- `inputValue` must remain verbatim.
- `choiceCount` and `visibleChoiceCount` must remain distinct.
- Mini simulateKey Enter must submit current prompt state.
- Micro simulateKey gap should be fixed or explicitly documented.
- Cancellation semantics are weak because SDK returns `""`.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| Source checks. | Mini/Micro SDK sends real messages; Rust routes install app views. |
| Mini runtime. | `mini("Pick", ["Apple","Banana"])` -> getState type mini, filter to Banana, Enter returns Banana. |
| Mini sizing. | Layout/window bounds show compact Mini width, not full ArgPrompt width. |
| Micro runtime. | `micro("Pick", ["Apple","Banana"])` -> getState type micro, getElements choices, batch submit returns value. |
| Micro footer. | Active footer is absent/none; no reserved footer space. |
| inputValue contract. | setInput text with punctuation/whitespace; getState returns exact text. |
| state counts. | choiceCount remains total; visibleChoiceCount changes with filter. |
| SDK warning audit. | Decide whether to remove stale warning or document why it remains. |
| simulateKey gap. | Mini Enter proof exists; Micro Enter should be added/proven before claiming parity. |

## Agent Notes

Use `getState`, `getElements`, `getLayoutInfo`, `batch`, and script result assertions before screenshots. Screenshots are only useful for compactness/double-padding/footer visual regressions.


Do not reserve native footer space for MicroPrompt.

Do not use Mini main-window mode receipts as proof of SDK `mini()`.

Do not use microphone/media tests as proof of SDK `micro()`.

## Related Features

| Feature | Relationship |
|---|---|
| [016 Prompt Runtime Core](./016-prompt-runtime-core.md). | General arg/select prompt concepts live there. |
| Window Resizing. | MiniPrompt compact sizing is pinned by window contracts. |
| Protocol Automation. | getState/getElements/batch/simulateKey are the proof surfaces. |
| Dictation Media. | Dictation can input into Mini/Micro; `mic()` is separate. |
| Mini AI / ACP. | Uses Mini window concepts, not SDK MiniPrompt. |
| Tab AI context. | Mini/Micro must produce rich prompt context. |

## Open Questions And Gaps

- SDK warning text is stale relative to Rust support.
- `removed-docs` mentions micro but may omit mini in one prompt-family list.
- `ShowMini` immediate resize appears to use ArgPrompt view types, conflicting with compact sizing contract.
- MicroPrompt lacks a visible simulateKey arm.
- Micro physical key handling was not fully visible.
- Public `selectFirst` command name is not proven, though internal helper exists.
- Empty/no-match submit behavior needs runtime proof.
- Disabled choice semantics are unknown.
- Cancellation is collapsed to `""` in the SDK.
- Micro focus/input sync needs direct verification because render sync visibly includes Mini/Arg but not Micro.
