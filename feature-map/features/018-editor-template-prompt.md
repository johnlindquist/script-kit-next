# 018 Editor and Template Prompt / editor() / template()

This chapter maps the full-height SDK editor prompt and the dedicated template placeholder prompt.

Raw Oracle reference: [answer](../raw-oracle/018-editor-template-prompt/answer.md), [prompt](../raw-oracle/018-editor-template-prompt/prompt.md), [bundle map](../raw-oracle/018-editor-template-prompt/bundle-map.md), [full log](../raw-oracle/018-editor-template-prompt/output.log), [session metadata](../raw-oracle/018-editor-template-prompt/session.json).

## Executive Summary

Feature 018 has two related but distinct surfaces:

| SDK API | Rust app view | Runtime meaning | Submit shape |
|---|---|---|---|
| `editor()` | `AppView::EditorPrompt` | Full-height code/text editor prompt, with optional SDK actions and snippet/tabstop mode. | Edited content string. |
| `template()` | `AppView::TemplatePrompt` | Dedicated placeholder prompt that parses template variables into fields and preview. | Rendered template string. |

The overlap is easy to misread. `EditorPrompt::with_template(...)` means snippet-enabled full editor mode, not SDK `template()`. SDK `template()` uses `TemplatePrompt::new(...)`, a separate prompt entity with different sizing, focus, footer, validation, element collection, and automation risks.

The main parity gaps in the captured source are TemplatePrompt automation and actions: `simulateKey` has a rich EditorPrompt branch but no dedicated TemplatePrompt branch, `ForceSubmit` supports EditorPrompt but not TemplatePrompt, and TemplatePrompt footer UI advertises an Actions button while the visible actions host map does not include `ActionsDialogHost::TemplatePrompt`.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Open a full-height editor. | `await editor(content, language, actions?)`. | Opens `AppView::EditorPrompt`; user edits and submits a string. |
| Edit code or text with language metadata. | `editor("// code", "typescript")`. | Shows editor prompt with language surfaced in state/elements. |
| Use editor SDK actions. | `editor(content, language, actions)`. | Opens actions dialog under `ActionsDialogHost::EditorPrompt`. |
| Submit editor content. | Cmd+Enter or Cmd+S in physical path; simulateKey Cmd+Enter in protocol path. | Resolves the SDK promise with editor content. |
| Use editor snippet/tabstops. | Content with explicit tabstops or ShowEditor template field. | Enters EditorPrompt snippet mode; Tab advances tabstops. |
| Open a template field prompt. | `await template("Hello {{name}}")`. | Opens `AppView::TemplatePrompt`; user fills fields and submits rendered string. |
| Preview rendered template. | Fill template fields. | Preview updates from placeholder values. |
| Validate template values. | Fill slug-like variables such as `script_name`. | Invalid values show validation; valid values submit. |
| Inspect surfaces through automation. | `getState`, `getElements`. | Agents can distinguish `editor` and `template` prompt types when mappings are current. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| EditorPrompt | Full-height editor child-content prompt. | Uses `ViewType::EditorPrompt`, `FocusTarget::EditorPrompt`, native footer surface `editor_prompt`. |
| TemplatePrompt | Dedicated template placeholder prompt. | Uses `AppView::TemplatePrompt`, `FocusTarget::TemplatePrompt`, native footer surface `template_prompt`, and DivPrompt sizing. |
| Editor snippet mode | Snippet/tabstop mode inside EditorPrompt. | Triggered by explicit ShowEditor template or tabstop-bearing editor content. |
| Template parser | Placeholder parser for `template()`. | Extracts variables, skips unsafe/control forms, renders a single-pass preview/result. |
| Prompt id | SDK id carried through app view and submit callback. | Editor uses submit callback `"editor"`; template uses submit callback `"template"`. |
| Actions host | Action menu owner. | Editor maps to `ActionsDialogHost::EditorPrompt`; TemplatePrompt actions are not proven. |
| Focus target | Keyboard routing owner. | Editor and TemplatePrompt have separate focus targets and pending focus application. |
| Full-height sizing | Editor-specific window sizing. | `ViewType::EditorPrompt` maps to max height; TemplatePrompt uses `ViewType::DivPrompt`. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `globalThis.editor` in `scripts/kit-sdk.ts`. | Script calls `editor(content = "", language = "text", actions?)`. | Sends editor prompt request and resolves a content string. |
| `globalThis.template` in `scripts/kit-sdk.ts`. | Script calls `template(templateStr, options?)`. | Preprocesses template text, sends template request, resolves rendered string. |
| `PromptMessage::ShowEditor`. | Rust receives editor request. | Stores actions, builds `EditorPrompt`, installs `AppView::EditorPrompt`, focuses editor, schedules guarded editor resize. |
| `PromptMessage::ShowTemplate`. | Rust receives template request. | Builds `TemplatePrompt`, installs `AppView::TemplatePrompt`, focuses template, resizes to div prompt. |
| `render_editor_prompt`. | Renders editor prompt wrapper. | Provides editor prompt shell, actions host, clickable hint strip, and editor entity body. |
| `TemplatePrompt::render`. | Renders template prompt. | Shows source, input fields, preview, validation, and footer behavior. |
| `collect_elements`. | Protocol element collection. | Exposes editor and template prompt elements, including template input semantic ids. |
| `runtime_stdin_match_simulate_key.rs`. | Protocol simulated key routing. | Has detailed EditorPrompt handling; TemplatePrompt handling is a gap in captured source. |

## User Workflows

### Normal Editor

A script calls:

```ts
const code = await editor("// edit me", "typescript")
```

The SDK creates a prompt id and sends an editor prompt. Rust handles `ShowEditor`, stores actions if provided, creates a submit callback, builds `EditorPrompt::with_height(...)`, installs `AppView::EditorPrompt`, clears text-input focus, sets pending focus to `FocusTarget::EditorPrompt`, and schedules a state-guarded resize to `ViewType::EditorPrompt`. The user edits content and submits. The SDK promise resolves with the current editor content.

### Editor Snippet Mode

Editor snippet mode is reached when ShowEditor carries an explicit template field, or when editor content contains explicit tabstops detected by snippet analysis. Rust builds `EditorPrompt::with_template(...)`, which parses the snippet template. Tab advances through tabstops, physical Shift+Tab can move backward, and Escape exits snippet mode. Leaving snippet mode does not change the app view; it remains `EditorPrompt`.

### Editor Actions

If `editor()` receives actions, the SDK serializes action descriptors and Rust stores them in the SDK action map. `AppView::EditorPrompt` maps to `ActionsDialogHost::EditorPrompt`. Actions must not steal reserved editor shortcuts such as submit, undo/redo, find, copy/paste, or selection commands.

### Dedicated Template Prompt

A script calls:

```ts
const rendered = await template("Hello {{name}}, your email is {{email}}")
```

The SDK preprocesses special tokens such as `$SELECTION` and starts a template prompt. Rust handles `ShowTemplate`, builds `TemplatePrompt::new(...)`, installs `AppView::TemplatePrompt`, focuses `FocusTarget::TemplatePrompt`, and sizes as `ViewType::DivPrompt`. The prompt parses placeholders into editable inputs, shows a preview, validates field values, and submits a rendered string.

### Template Validation

TemplatePrompt validation is field-specific. Tests pin slug-like validation for fields such as `script_name` and `extension_name`, required field behavior, empty optional description behavior, and single-pass substitution. Invalid values should block submit and show validation rather than resolving a bad rendered string.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open editor prompt. | `editor(content, language)`. | `AppView::EditorPrompt`. | SDK call. | `globalThis.editor` -> `ShowEditor` -> `EditorPrompt::with_height`. | Full-height editor opens. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `src/editor/mod.rs`. |
| Open editor snippet mode. | Editor content with tabstops or ShowEditor template. | `AppView::EditorPrompt` with snippet state. | SDK/protocol call. | `contains_explicit_tabstops` / template field -> `EditorPrompt::with_template`. | Editor opens with tabstop navigation. | `src/prompt_handler/mod.rs`, `src/editor/mod.rs`. |
| Edit editor content. | Active editor. | Editor focused. | Type/paste. | Embedded editor entity handles input. | Content changes. | `src/editor/mod.rs`. |
| Submit editor. | Active editor. | Editor focused. | Cmd+Enter or Cmd+S. | `EditorPrompt::submit` / simulateKey Cmd+Enter -> submit response. | SDK resolves string. | `src/editor/mod.rs`, `src/main_entry/runtime_stdin_match_simulate_key.rs`. |
| Cancel editor by protocol. | Active editor. | Editor focused. | simulateKey Escape. | Editor simulateKey arm submits/cancels with `None`. | Prompt cancelled in protocol path. | `src/main_entry/runtime_stdin_match_simulate_key.rs`; physical parity uncertain. |
| Open editor actions. | `editor(..., actions)`. | Editor prompt with actions. | Cmd+K/action shortcut. | `ActionsDialogHost::EditorPrompt`. | Editor-scoped actions popup opens. | `src/app_impl/actions_dialog.rs`, `src/render_prompts/editor.rs`. |
| Open template prompt. | `template("Hello {{name}}")`. | `AppView::TemplatePrompt`. | SDK call. | `globalThis.template` -> `ShowTemplate` -> `TemplatePrompt::new`. | Field/preview prompt opens. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `src/prompts/template/prompt.rs`. |
| Fill template field. | Active TemplatePrompt. | Template input focused. | Type/click. | `TemplatePrompt::set_input` path and render state. | Preview updates. | `src/prompts/template/prompt.rs`, `src/prompts/template/render.rs`. |
| Move template field. | Active TemplatePrompt. | Template input list. | Next Field / Tab path. | Footer special-case / template focus navigation. | Next placeholder receives focus. | `src/app_impl/ui_window.rs`, `src/prompts/template/render.rs`. |
| Submit template. | Active TemplatePrompt. | Valid inputs. | Submit footer/Enter path. | `TemplatePrompt::filled_template` -> submit callback. | SDK resolves rendered string. | `src/prompts/template/prompt.rs`, `src/prompts/template/tests.rs`. |
| Inspect editor. | Protocol. | Editor active. | `getState` / `getElements`. | State mapping and editor collector. | Prompt type `editor`; editor elements. | `src/prompt_handler/mod.rs`, `src/app_layout/collect_elements.rs`. |
| Inspect template. | Protocol. | Template active. | `getState` / `getElements`. | State mapping and template collector. | Prompt type `template`; template input elements. | `src/prompt_handler/mod.rs`, `src/app_layout/collect_elements.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK idle. | No editor/template prompt active. | Script continues. | No pending prompt id. |
| Editor request created. | `editor()`. | Create id, serialize actions, send editor message. | SDK default language is `text`; Rust fallback is `markdown` only if language absent. |
| Editor route handled. | `ShowEditor`. | Build normal editor or snippet editor. | Explicit template/tabstops choose `with_template`. |
| Editor view installed. | Entity created. | `AppView::EditorPrompt`, `FocusTarget::EditorPrompt`, guarded `ViewType::EditorPrompt` resize. | Must remain full height. |
| Editor editing. | User types/navigates. | Editor content/snippet state mutates. | Actions popup may temporarily own keys. |
| Editor submit/cancel. | Cmd+Enter/Cmd+S/protocol/cancel. | Submit callback resolves SDK promise. | Protocol Escape cancellation differs from physical Escape uncertainty. |
| Template request created. | `template()`. | Preprocess template string, send template message. | `$SELECTION` and `$CLIPBOARD` preprocessing is visible but truncated. |
| Template route handled. | `ShowTemplate`. | Build `TemplatePrompt::new`, install app view. | Uses `ViewType::DivPrompt`. |
| Template editing. | User fills fields. | Preview and validation state update. | Placeholder values substitute single-pass. |
| Template submit. | Valid submit action. | Render filled template and resolve SDK promise. | ForceSubmit support is not proven. |

## Visual And Focus States

| Surface | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| EditorPrompt normal. | Full-height editor body plus wrapper footer/hints. | `FocusTarget::EditorPrompt`. | Prompt type `editor`, native footer `editor_prompt`. |
| EditorPrompt snippet mode. | Editor with tabstop/choice overlay state. | `FocusTarget::EditorPrompt`. | Tabstop-rich state in Tab AI/context receipts. |
| Editor actions. | Actions popup over editor. | Actions dialog host. | `ActionsDialogHost::EditorPrompt`. |
| TemplatePrompt. | Template source, editable placeholders, preview, footer. | `FocusTarget::TemplatePrompt`. | Prompt type `template`, native footer `template_prompt`. |
| Template no placeholders. | Template prompt with no editable input rows. | Template prompt focus. | Element count may be zero for inputs; source/preview remain. |
| Invalid template value. | Validation UI/error state. | Template prompt focus. | Submit should not resolve until corrected. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Text input. | EditorPrompt. | Updates editor content through embedded editor. |
| Tab. | EditorPrompt snippet mode. | Advances to next tabstop; may interact with choice popup. |
| Shift+Tab. | Physical EditorPrompt snippet mode. | Moves to previous tabstop; simulateKey parity is not proven. |
| Escape. | EditorPrompt snippet mode. | Exits snippet mode. |
| Escape. | EditorPrompt protocol simulateKey. | Cancels/submits `None` in visible simulateKey arm. |
| Cmd+Enter. | EditorPrompt. | Submits current content. |
| Cmd+S. | Physical EditorPrompt. | Submits/saves by editor path; simulateKey Cmd+S is not proven. |
| Cmd+K. | EditorPrompt with actions. | Opens editor actions. |
| Text input. | TemplatePrompt field. | Updates placeholder value and preview. |
| Next Field footer. | TemplatePrompt. | Advances to next template input. |
| Submit footer/Enter path. | TemplatePrompt. | Validates and submits rendered output. |

## Actions And Menus

| Surface | Host | Notes |
|---|---|---|
| EditorPrompt | `ActionsDialogHost::EditorPrompt`. | Proven host mapping and focus restore path. |
| Editor action popup | Actions dialog surface. | Should route Up/Down/Enter to popup, not editor cursor. |
| TemplatePrompt | `ActionsDialogHost::TemplatePrompt`. | Footer Actions now routes through a live TemplatePrompt shared-actions host with TemplatePrompt focus restore. |

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| Editor `getState`. | Prompt type `editor` and current prompt id match SDK id. |
| Editor `getElements`. | Includes editor prompt metadata such as language and editor prompt identity. |
| Editor simulateKey Cmd+Enter. | Submits current content. |
| Editor simulateKey Tab. | Calls `next_tabstop` in visible arm. |
| Editor simulateKey Escape. | Cancels the editor prompt in protocol path. |
| Editor ForceSubmit. | Visible support includes `AppView::EditorPrompt`. |
| Template `getState`. | Main state mapping should report prompt type `template`. |
| Template `getElements`. | Includes template source, input collection, and input semantic ids such as `template-name`. |
| Template simulateKey. | Dedicated TemplatePrompt arm handles Enter submit, Escape cancel, Tab/Shift+Tab navigation, text editing, Backspace, and Cmd+K actions. |
| Template ForceSubmit. | Direct and batch ForceSubmit support active TemplatePrompt by submitting the provided value. |

## Data, Storage, And Privacy Boundaries

- Editor content may contain source code, secrets, or selected text. Avoid logging full content or exposing screenshots unnecessarily.
- The captured route logs enough editor/template content during some setup paths that additional logging should be treated as sensitive.
- SDK actions serialize descriptors and handler ids while the prompt is active.
- Template input values are user-entered prompt data and appear in rendered preview/results.
- `$SELECTION` and `$CLIPBOARD` preprocessing can inject sensitive local data into `template()` prompts.
- Template substitution is single-pass, so user-entered placeholder-like text should not recursively expand.
- Prompt response payloads return to the SDK resolver and should be considered script-visible.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| Empty editor content. | Opens EditorPrompt with empty content string. |
| Unsupported editor language. | No validation/fallback beyond storing/displaying language is proven. |
| Editor stale resize. | Deferred resize is guarded by current prompt id and current view. |
| Editor snippet no tabstops. | Normal EditorPrompt path unless explicit template field is present. |
| Editor physical Escape outside snippet mode. | Uncertain; visible editor code propagates while protocol Escape cancels. |
| Template no placeholders. | Prompt should render source/preview with zero editable inputs. |
| Template invalid required/slug field. | Validation blocks submit and reports error. |
| Template optional description empty. | Tests allow empty optional description. |
| Template actions. | Ambiguous; footer copy exists but host support is not proven. |
| Template language option. | SDK signature accepts options, but Rust ShowTemplate route does not show language consumption. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK APIs. | `scripts/kit-sdk.ts` owns `editor()` and `template()` call shapes, defaults, preprocessing, and response parsing. |
| Editor route. | `src/prompt_handler/mod.rs` owns `ShowEditor`, actions storage, editor construction, focus, submit callback, and guarded resize. |
| Template route. | `src/prompt_handler/mod.rs` owns `ShowTemplate`, template entity construction, focus, and div-style sizing. |
| Editor entity. | `src/editor/mod.rs` owns editor content, language, snippet parsing/state, tabstop movement, focus, submit, and render body. |
| Editor wrapper. | `src/render_prompts/editor.rs` owns editor prompt shell, action host wiring, and footer/hint strip integration. |
| Template entity. | `src/prompts/template/prompt.rs` owns placeholder parsing, input state, validation, preview, and filled output. |
| Template render. | `src/prompts/template/render.rs` owns template prompt UI and focusable render behavior. |
| App-view contracts. | `src/main_sections/app_view_state.rs`, `src/app_impl/ui_window.rs`, and `src/focus_coordinator/mod.rs` own surface kind, footer, focus target, and sizing integration. |
| Protocol receipts. | `src/app_layout/collect_elements.rs`, `src/main_entry/runtime_stdin_match_simulate_key.rs`, and protocol constructors own state/elements/simulated input coverage. |
| Contract tests. | `tests/tab_ai_input_coverage.rs`, `tests/minimal_chrome_audit.rs`, `tests/source_audits/resize_presentation_contract.rs`, and editor/template smoke tests. |

## Invariants And Regression Risks

- Do not collapse `template()` into editor snippet mode.
- Keep `EditorPrompt` full-height through `ViewType::EditorPrompt`.
- Keep ShowEditor deferred resize guarded by prompt id/current view.
- Do not add prompt-local editor footers; footer ownership belongs to the wrapper/native footer slot.
- Preserve reserved editor shortcuts over SDK actions.
- Do not claim TemplatePrompt actions until a real host mapping and focus/action routing exist.
- Do not assume simulateKey parity with physical editor keys; verify Shift+Tab, Cmd+S, and Escape separately.
- Keep TemplatePrompt substitution single-pass.
- Keep TemplatePrompt parser defensive around control tags and JavaScript-like expressions.
- Keep TemplatePrompt footer Ai behavior local to Next Field, not Agent Chat.
- Do not rely on ForceSubmit for TemplatePrompt unless implemented and tested.
- Keep automation semantic ids stable, especially template input ids.
- Watch helper drift: one captured helper reports `editor` but may omit `template` even though main state mapping handles TemplatePrompt.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| Editor source contract tests. | Minimal chrome and resize audits keep editor footer and full-height resize ownership pinned. |
| Editor launch receipt. | `editor("hello", "text")` -> `getState` reports prompt type `editor`; `getElements` reports editor metadata. |
| Editor submit receipt. | simulateKey Cmd+Enter resolves the SDK promise with current content. |
| Editor snippet receipt. | `editor("Hello ${1:name}", "text")` -> Tab moves tabstop state; physical Shift+Tab tested separately. |
| Editor actions receipt. | `editor(content, lang, actions)` -> Cmd+K opens actions dialog with editor host; reserved shortcuts still submit/edit. |
| Template launch receipt. | `template("Hello {{name}}")` -> `getState` reports `template`; `getElements` includes `template-source`, `template-inputs`, and `template-name`. |
| Template no-placeholder receipt. | `template("Hello world")` shows no editable inputs and submits original text. |
| Template validation tests. | Slug-like fields reject invalid values and accept valid slugs; optional description can be empty. |
| Template single-pass test. | User-entered placeholder-looking values are not recursively expanded. |
| Template parity contract. | `cargo test --test template_prompt_parity_contract -- --nocapture` pins TemplatePrompt simulateKey, ForceSubmit, and Actions footer host coverage. |
| Remaining gap checks. | Explicitly test `template(..., { language })` before claiming language-option behavior. |

## Agent Notes

When you see `EditorPrompt::with_template`, read it as snippet-enabled full editor mode. It is not the same as SDK `template()`.

For editor automation, prefer `getState`, `getElements`, and simulateKey Cmd+Enter. Do not use generic prompt-input setters for editor content unless source proves support.

For template automation, prefer semantic ids from `getElements` and the input-setting paths that call `TemplatePrompt::set_input`. `simulateKey` now owns field submit/cancel/navigation/editing, and ForceSubmit submits the explicit provided value.

If adding TemplatePrompt actions, update the full stack: `ActionsDialogHost`, host detection, focus restore, action toggle, active popup routing, simulateKey/generic routing, footer tests, and automation receipts.

When changing template parsing, update tests for control tags, JavaScript expressions, duplicate placeholders, slug validation, and single-pass substitution. Parser broadening can accidentally capture code expressions.

## Related Features

| Feature | Relationship |
|---|---|
| [016 Prompt Runtime Core](./016-prompt-runtime-core.md). | Shares prompt ids, submit callbacks, actions patterns, and prompt state. |
| [017 Form and Fields Prompt](./017-form-fields-prompt.md). | TemplatePrompt is field-like but has separate parser, preview, validation, and output semantics. |
| [015 SDK TermPrompt](./015-sdk-term-prompt.md). | Shares full-height prompt sizing concepts through child-content prompts. |
| Actions Popups. | EditorPrompt has a proven actions host; TemplatePrompt support is ambiguous. |
| Protocol Automation. | State/elements/simulateKey/ForceSubmit define practical agent support. |
| Window Resizing. | EditorPrompt is max-height; TemplatePrompt is div-sized. |
| ScratchPad and Notes editor. | Adjacent editor-like surfaces, but separate unless a task is explicitly about shared editor runtime behavior. |

## Open Questions And Gaps

- The SDK `template()` body is truncated in the bundle; full `$CLIPBOARD` replacement and final send shape need inspection.
- `template(..., { language })` is not proven to affect Rust behavior.
- TemplatePrompt actions, simulateKey, and ForceSubmit parity are covered by `tests/template_prompt_parity_contract.rs`; runtime receipts should still prove live submit, cancel, actions, and ForceSubmit behavior before release.
- EditorPrompt simulateKey parity is incomplete for Shift+Tab, Cmd+S, and physical-vs-protocol Escape.
- Editor physical Escape outside snippet mode is uncertain.
- Editor footer Run dispatch is not fully visible.
- Exact editor element shape, content details, and tabstop element receipts need local inspection if agents rely on them.
- Dedicated TemplatePrompt support for `${1:name}` positional snippet syntax is not pinned by the visible parser tests.
- Unsupported editor language behavior is unproven.
- Template validation UI and disabled footer behavior require runtime proof beyond validation unit tests.
