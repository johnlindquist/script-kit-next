# 017 Form and Fields Prompt / form() / fields()

This chapter maps structured SDK input collection through `form()` and `fields()`.

Raw Oracle reference: [answer](../raw-oracle/017-form-fields-prompt/answer.md), [prompt](../raw-oracle/017-form-fields-prompt/prompt.md), [bundle map](../raw-oracle/017-form-fields-prompt/bundle-map.md), [full log](../raw-oracle/017-form-fields-prompt/output.log), [session metadata](../raw-oracle/017-form-fields-prompt/session.json).

## Executive Summary

`form()` is an implemented SDK prompt surface. The TypeScript SDK sends a form message with a prompt id, HTML, and optional actions. Rust routes it to `PromptMessage::ShowForm`, builds `FormPromptState`, installs `AppView::FormPrompt`, focuses `FocusTarget::FormPrompt`, renders native GPUI form controls, validates email/number fields on keyboard submit, and resolves the SDK promise with a JSON object keyed by field names.

`fields()` is now a production GPUI prompt surface. The SDK exposes `fields(fieldDefs, actions?)`, normalizes string definitions, sends a fields message, and expects a JSON array response. Rust routes it to `PromptMessage::ShowFields`, builds shared `FormPromptState::from_fields`, installs `AppView::FormPrompt`, reports `promptType:"fields"` to automation, and resolves the SDK promise with a JSON array ordered by the submitted field definitions.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Render a native form from HTML. | `await form(html)` | Opens `AppView::FormPrompt` and resolves an object after submit. |
| Collect text-like inputs. | `<input name="first">`, `email`, `number`, `password`, `url`, `search`, `tel`, date/time-like types. | Values submit as named JSON properties. |
| Collect multiline text. | `<textarea name="bio">`. | Enter inserts a newline; Command+Enter submits. |
| Collect checkbox and select values. | `<input type="checkbox">`, `<select>`. | Parsed as form fields; exact wire value semantics need source/runtime proof for some types. |
| Use form actions. | `form(html, actions)`. | Opens a form-scoped actions dialog owned by `ActionsDialogHost::FormPrompt`. |
| Navigate fields. | Tab, Shift+Tab, click. | Moves field focus inside the form prompt. |
| Validate basic typed fields. | Submit invalid `email` or `number`. | Shows HUD validation and blocks keyboard submit. |
| Inspect or drive forms through automation. | `getState`, `getElements`, simulated keys, force submit. | Agents can identify form prompt state and visible fields. |
| Call `fields()`. | `await fields(["Name"])` or field definitions. | Opens the shared GPUI form prompt surface and resolves a JSON array in field-definition order. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `form()` SDK API | Developer-facing structured HTML form prompt. | Sends `type: "form"` with id, HTML, and optional serialized actions. |
| `fields()` SDK API | Developer-facing structured field-definition prompt. | Sends `type: "fields"` and resolves `string[]` ordered by definition position. |
| `FormPromptState` | Rust state created from form HTML and theme-derived field colors. | Owns parsed fields, focus movement, key handling, validation, and collected values. |
| `AppView::FormPrompt` | Active app view for implemented forms. | Must carry prompt id and entity identity through render, footer, automation, and submit. |
| `FocusTarget::FormPrompt` | Keyboard focus target for form prompts. | Prevents keystrokes from routing to launcher/arg input while a form is active. |
| Native footer surface | Footer identity for active form prompt. | `native_footer_surface()` returns `form_prompt`. |
| Form actions host | Actions dialog ownership for forms. | Uses `ActionsDialogHost::FormPrompt`, not arg/div/launcher hosts. |
| `FormPromptOutputMode` | Shared output contract for form and fields prompts. | `form()` submits object-by-name, while `fields()` submits array-by-order. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `globalThis.form` in `scripts/kit-sdk.ts`. | Script calls `form(html, actions?)`. | Sends `FormMessage`, waits for response, parses JSON object, resolves `{}` on invalid object response. |
| `globalThis.fields` in `scripts/kit-sdk.ts`. | Script calls `fields(fieldDefs, actions?)`. | Sends `FieldsMessage`, waits for response, parses JSON array, resolves `[]` on invalid array response. |
| `PromptMessage::ShowForm`. | Rust receives form prompt request. | Prepares window, stores actions, creates form state/entity, sets `AppView::FormPrompt`. |
| `PromptMessage::ShowFields`. | Rust receives fields prompt request. | Prepares window, stores actions, creates fields-backed form state/entity, and sets `AppView::FormPrompt`. |
| `render_impl.rs`. | App view render dispatch. | `AppView::FormPrompt` calls `render_form_prompt`. |
| `render_prompts/form/render.rs`. | Form UI render and event routing. | Handles field focus, Tab, Enter, Command+Enter, actions backdrop, and submit attempt. |
| `render_prompts/form/helpers.rs`. | Form behavior helpers. | Owns `FormEnterBehavior`, textarea detection, validation messages, and validation-error collection. |
| `app_layout/collect_elements.rs`. | Protocol element collection. | Delegates form prompt elements to `collect_form_prompt_elements`. |

## User Workflows

### Show And Submit A Form

A script calls:

```ts
const result = await form(`<input name="first" placeholder="First name">`)
```

The SDK creates a prompt id, sends a form message, and waits for a response. Rust handles `ShowForm`, prepares the window, creates `FormPromptState`, installs `AppView::FormPrompt`, clears launcher input focus, sets pending focus to `FocusTarget::FormPrompt`, sizes the window from field count, and notifies GPUI. The user fills fields and submits. The SDK receives a JSON string, parses it as an object, and resolves that object.

### Textarea Editing

When the focused field is a textarea, plain Enter is forwarded to the textarea so the user can enter a newline. Command+Enter submits the form. This is a form-specific key contract; do not generalize arg/select Enter behavior onto textareas.

### Validation Failure

Keyboard submit runs form submit validation first. Email fields must pass email validation, and number fields must pass number validation. If validation errors exist, the app shows a HUD message for a longer duration and does not submit. Other visible field types are accepted by the captured validation helper.

### Form Actions

If the script passes actions to `form(html, actions)`, the SDK serializes action descriptors and Rust stores them for the active prompt. The form render path uses the shared prompt key preamble with `ActionsDialogHost::FormPrompt`. The actions backdrop id is `form-actions-backdrop`; clicking it dismisses the action popup without handing ownership to another surface.

### Automation Submit

Protocol automation can inspect the active form with `getState` and `getElements`. `ForceSubmit` includes `AppView::FormPrompt` among supported prompt views in the captured handler. The forced value is routed through generic prompt response by id. Because this appears separate from keyboard validation, treat validation bypass as a likely but unproven behavior until runtime receipts prove it.

### Calling fields()

A script can call:

```ts
const values = await fields(["First name", { name: "email", label: "Email" }])
```

The SDK normalizes string entries to `{ name, label }`, sends a fields message, and waits for a JSON array response. Rust converts the field definitions into shared `FormPromptState` so the visible surface has the same focus, validation, native footer, actions host, and automation affordances as `form()`, while `collect_values()` returns an array ordered by the original definitions instead of an object keyed by name.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Show HTML form. | `form(html)`. | `AppView::FormPrompt`. | SDK call. | `globalThis.form` -> `FormMessage` -> `PromptMessage::ShowForm` -> `FormPromptState::new`. | Native GPUI form appears. | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`. |
| Type into field. | Active form. | Field focused. | Text input. | `render_form_prompt` forwards key input to form entity. | Field value updates. | `src/render_prompts/form/render.rs`. |
| Move next field. | Active form. | Any form field. | Tab. | Form render key handler -> `focus_next`. | Focus advances. | `src/render_prompts/form/render.rs`. |
| Move previous field. | Active form. | Any form field. | Shift+Tab. | Form render key handler -> `focus_previous`. | Focus moves back. | `src/render_prompts/form/render.rs`. |
| Submit from normal field. | Active form. | Non-textarea field focused. | Enter. | `form_enter_behavior` -> validation -> submit response. | SDK resolves JSON object if valid. | `src/render_prompts/form/helpers.rs`, `src/render_prompts/form/render.rs`. |
| Insert textarea newline. | Active form. | Textarea focused. | Enter. | `form_enter_behavior` returns forward-to-field. | Newline is handled by textarea; no submit. | `src/render_prompts/form/helpers.rs`, `src/render_prompts/form/tests.rs`. |
| Submit textarea. | Active form. | Textarea focused. | Command+Enter. | `form_enter_behavior` returns submit. | Form submits if valid. | `src/render_prompts/form/helpers.rs`, `src/render_prompts/form/tests.rs`. |
| Reject invalid email/number. | Active form. | Invalid typed value. | Submit. | `collect_form_submit_validation_errors` -> HUD. | Submit blocked. | `src/render_prompts/form/helpers.rs`. |
| Open form actions. | `form(html, actions)`. | Active form with actions. | Action shortcut/menu. | Shared prompt key preamble with `ActionsDialogHost::FormPrompt`. | Form-scoped actions dialog opens. | `src/render_prompts/form/render.rs`, `src/main_sections/app_view_state.rs`. |
| Dismiss form actions. | Form actions popup. | Backdrop visible. | Click backdrop. | `form-actions-backdrop` dismissal. | Actions popup closes. | `src/render_prompts/form/render.rs`. |
| Inspect state. | Protocol. | Active form. | `getState`. | `current_prompt_type()` and app-view state. | Prompt is identifiable as `form`. | `src/prompt_handler/mod.rs`. |
| Inspect elements. | Protocol. | Active form. | `getElements`. | `collect_form_prompt_elements`. | Field elements are returned. | `src/app_layout/collect_elements.rs`. |
| Force submit. | Protocol. | Active form. | `ForceSubmit`. | Current view id match -> prompt response. | SDK promise resolves with forced value. | `src/prompt_handler/mod.rs`; validation behavior needs proof. |
| Call `fields()`. | `fields(fieldDefs)`. | `AppView::FormPrompt` reporting `fields`. | SDK call. | `globalThis.fields` -> `FieldsMessage` -> `PromptMessage::ShowFields` -> `FormPromptState::from_fields`. | Native GPUI fields prompt appears and resolves a JSON array. | `scripts/kit-sdk.ts`, `src/main_sections/prompt_messages.rs`, `src/prompt_handler/mod.rs`, `src/form_prompt.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK idle. | No form/fields call active. | Script continues normally. | No pending structured-input prompt id. |
| Form request created. | `form(html, actions?)`. | Create id, serialize actions, send `FormMessage`. | Pending resolver expects object-like JSON. |
| Form message handled. | Rust receives `ShowForm`. | Prepare window and create `FormPromptState`. | Theme-derived `FormFieldColors` are applied. |
| Form view installed. | Entity created. | `current_view = AppView::FormPrompt { id, entity }`. | Focus is moved away from launcher input. |
| Editing. | User types/clicks/tabs. | Form entity updates field focus and values. | Textarea Enter is special. |
| Validation. | Keyboard submit. | Collect email/number validation errors. | Errors block submit and show HUD. |
| Submit. | Valid Enter, Command+Enter, click/automation submit. | Prompt response sent with active id. | SDK parses object, falls back to `{}` on invalid object. |
| Fields request created. | `fields(fieldDefs, actions?)`. | Create id, normalize defs, send `FieldsMessage`. | Pending resolver expects array JSON. |
| Fields message handled. | Rust receives `ShowFields`. | Prepare window and create `FormPromptState::from_fields`. | Theme-derived `FormFieldColors` are applied. |
| Fields view installed. | Entity created. | `current_view = AppView::FormPrompt { id, entity }`. | `getState` reports prompt type `fields` even though the native footer surface remains `form_prompt`. |
| Fields submit. | Valid Enter or click/automation submit. | Prompt response sent with active id. | SDK parses an array, falling back to `[]` on invalid array responses. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Form with fields. | Native GPUI form controls. | `FocusTarget::FormPrompt`. | Prompt type `form`; form elements in `getElements`. |
| Form without discovered fields. | Form prompt still active, sized like div prompt. | `FocusTarget::FormPrompt`. | App view remains `FormPrompt`; field count may be zero. |
| Textarea focused. | Multiline field. | Form field focus. | Plain Enter should not submit. |
| Invalid submit. | HUD validation message. | Form remains active. | No SDK response should resolve from keyboard submit. |
| Form actions open. | Actions popup over form. | Actions dialog host. | Host is `ActionsDialogHost::FormPrompt`; backdrop id `form-actions-backdrop`. |
| Password field. | Masked visual input. | Form field focus. | Submitted value is still sensitive prompt data. |
| Fields prompt. | Native GPUI form controls built from field definitions. | `FocusTarget::FormPrompt`. | Prompt type `fields`; `fields-fields` and `input:fields-*` elements in `getElements`. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Text input. | Focused form field. | Updates the field. |
| Click field. | Form prompt. | Focuses clicked field and synchronizes parent focus state. |
| Tab. | Form prompt. | Moves to next field. |
| Shift+Tab. | Form prompt. | Moves to previous field. |
| Enter. | Non-textarea field. | Attempts validation and submit. |
| Enter. | Textarea field. | Forwards newline/editing input to field. |
| Command+Enter. | Textarea field. | Attempts validation and submit. |
| Action shortcut. | Form prompt with actions. | Opens form-scoped actions dialog. |
| Backdrop click. | Form actions open. | Closes action dialog. |
| Protocol `ForceSubmit`. | Form prompt. | Submits by prompt id through generic prompt response path. |

## Actions And Menus

| Surface | Host | Backdrop | Notes |
|---|---|---|---|
| Form prompt. | `ActionsDialogHost::FormPrompt`. | `form-actions-backdrop`. | Actions must stay scoped to the active form prompt id. |
| Fields prompt. | `ActionsDialogHost::FormPrompt`. | `form-actions-backdrop`. | Fields reuses the form prompt actions host while preserving `promptType:"fields"`. |

Form actions share the broader prompt action infrastructure, but the host identity is part of the contract. Routing form actions through arg/div/launcher hosts risks wrong focus restoration, wrong action subject, and wrong dismissal behavior.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getState` on active form. | Prompt type should identify as `form`; active prompt id should match SDK id. |
| `getElements` on active form. | Should expose a form-fields collection and individual field elements. |
| Simulated typing. | Should update the focused field, not launcher input. |
| Simulated Tab/Shift+Tab. | Should move field focus inside the form prompt. |
| Simulated Enter. | Should submit normal fields and forward newline in textareas. |
| Simulated Command+Enter. | Should submit textareas. |
| `ForceSubmit`. | Should resolve the current form prompt id; whether it bypasses validation needs a dedicated receipt. |
| `fields()` automation. | Should prove prompt type `fields`, `fields-fields` elements, `input:fields-*` focus, validation blocking, submit/cancel, actions, and explicit array `ForceSubmit`. |

## Data, Storage, And Privacy Boundaries

- Form HTML is script-provided prompt-local data parsed into native form state.
- Submitted values resolve to the SDK as a JSON object keyed by field names.
- Password fields are masked visually, but their submitted values are still plain prompt response data and should be treated as sensitive.
- Validation errors should identify invalid field labels/types without logging sensitive values.
- SDK actions serialize action descriptors and handler ids while the prompt is active.
- `fields()` field definitions cross the SDK/app boundary and resolve as array values in definition order.
- The bundle does not show persistent storage for form HTML or field values.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| Invalid email. | Keyboard submit blocks and shows HUD validation. |
| Invalid number. | Keyboard submit blocks and shows HUD validation. |
| Malformed/unsupported form attributes. | Exact parser behavior is a proof gap; inspect parser source before documenting specifics. |
| Unsupported form types. | Parity report marks radio, range, and file as unsupported; hidden, submit, and button are intentionally skipped. |
| Text-like specialized types. | Date/time-like, URL/search/tel/color are accepted or passed through as text-field-like controls in captured reports/tests. |
| Empty/no-field form. | Still installs `AppView::FormPrompt`; window sizing may use div-style sizing. |
| Invalid SDK response for `form()`. | SDK resolves `{}` if response is missing, invalid JSON, or not an object. |
| Invalid SDK response for `fields()`. | SDK resolves `[]` if response is missing, invalid JSON, or not an array. |
| `fields()` backend. | Uses the shared form prompt surface, reports `promptType:"fields"`, and submits arrays by field-definition order. |
| Loading. | No explicit form loading state is visible; construction appears synchronous after message receipt. |
| Disabled submit. | No distinct disabled footer state is proven; submit/button input elements are skipped. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK APIs and response parsing. | `scripts/kit-sdk.ts` owns `globalThis.form`, `globalThis.fields`, message shapes, action serialization, and parse fallbacks. |
| Prompt routing. | `src/prompt_handler/mod.rs` owns `ShowForm`, `ShowFields`, focus setup, app-view installation, force submit inclusion, and prompt type reporting. |
| App view identity. | `src/main_sections/app_view_state.rs` owns `AppView::FormPrompt`, surface kind, native footer id, focus target, and actions host identity. |
| Render dispatch. | `src/main_sections/render_impl.rs` routes `AppView::FormPrompt` to `render_form_prompt`. |
| Form rendering. | `src/render_prompts/form/render.rs` owns form UI, key handling, actions backdrop, focus navigation, validation call, and submit attempt. |
| Form helpers. | `src/render_prompts/form/helpers.rs` owns Enter behavior, textarea detection, footer status text, and validation helpers. |
| Unit behavior tests. | `src/render_prompts/form/tests.rs` covers Enter and footer status behavior. |
| Protocol elements. | `src/app_layout/collect_elements.rs` delegates form element collection. |
| SDK and smoke coverage. | `tests/sdk/test-form-*`, `tests/sdk/test-fields-*`, `tests/sdk/FORM_FIELDS_PARITY_REPORT.md`, `tests/smoke/test-form-*`, and `tests/smoke/test-protocol-submit.ts`. |

## Invariants And Regression Risks

- `form()` must install `AppView::FormPrompt`, not div/arg/launcher fallback state.
- `current_prompt_type()` must report `form` for active forms.
- `native_footer_surface()` must return `form_prompt` for forms.
- `FocusTarget::FormPrompt` must be set when showing a form.
- Field clicks and focus movement must keep keyboard routing inside the form.
- Textarea Enter must remain newline/editing input, while Command+Enter submits.
- Email and number validation must block keyboard submit before SDK resolution.
- Form actions must use `ActionsDialogHost::FormPrompt`.
- Password masking is not data secrecy; avoid logging submitted values.
- `fields()` must keep its `promptType:"fields"` and array-by-order submit semantics even though it reuses `AppView::FormPrompt` and `ActionsDialogHost::FormPrompt`.
- Adding native pickers or richer field types changes UX, automation element shapes, validation, and SDK typing; update all layers together.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| Inspect SDK API shape. | `scripts/kit-sdk.ts` shows `form()` returns object and `fields()` returns array. |
| Inspect Rust form route. | `PromptMessage::ShowForm` creates `FormPromptState` and installs `AppView::FormPrompt`. |
| Inspect app-view contracts. | `FormPrompt` maps to prompt entity kind, `form_prompt` native footer, `FocusTarget::FormPrompt`, and `ActionsDialogHost::FormPrompt`. |
| Run form Enter unit tests. | Non-textarea Enter submits; textarea Enter forwards; textarea Command+Enter submits. |
| Run invalid email/number form. | HUD validation appears and SDK promise does not resolve until corrected. |
| Run form actions script. | Actions dialog opens under `ActionsDialogHost::FormPrompt`; backdrop id is `form-actions-backdrop`. |
| Run `getState` on active form. | Receipt reports prompt type `form` and active prompt id. |
| Run `getElements` on active form. | Receipt includes form-fields list and individual field elements. |
| Run protocol `ForceSubmit`. | SDK promise resolves by active id; separately record whether validation is bypassed. |
| Run minimal `fields(["Name"])`. | `getState.promptType` is `fields`, `getElements` exposes `fields-fields`, and SDK resolution is a JSON array. |

## Agent Notes

Before changing this feature, run `lat expand` and `lat search` for form prompt, fields prompt, prompt runtime, protocol automation, and verification. Keep `form()` and `fields()` status separate in docs and test plans.

Do not collapse `fields()` into object-style `form()` semantics. Its implementation deliberately uses shared `FormPromptState` for focus, actions, footer, validation, element collection, automation, and submit behavior, while `FormPromptOutputMode::ArrayByOrder` preserves the SDK contract.

The older parity report is historical. It described `fields()` falling through as unhandled; current source should route `Message::Fields` to `PromptMessage::ShowFields` and the shared form prompt state.

When adding a new field type, update SDK typings, wire message shape if needed, parser/conversion, renderer component, validation, element collection, tests, and `lat.md`.

## Related Features

| Feature | Relationship |
|---|---|
| [016 Prompt Runtime Core](./016-prompt-runtime-core.md). | Covers `arg()`, `select()`, `div()`, and `md()`; form shares prompt ids, submit callbacks, actions, and automation patterns but has its own entity. |
| [015 SDK TermPrompt](./015-sdk-term-prompt.md). | Shares SDK prompt response and force-submit patterns; terminal keyboard/rendering ownership is separate. |
| Actions dialogs. | Form actions must preserve `ActionsDialogHost::FormPrompt`. |
| Protocol automation. | Forms require state, element, key, and submit receipts because visual masking and validation are not enough. |
| Field-definition prompt. | `fields()` is the field-definition prompt and shares the form prompt shell while keeping array response semantics. |

## Open Questions And Gaps

- The exact conversion from SDK `Message::Fields` to `PromptMessage::ShowFields` should stay covered by the source audit because older reports described an unhandled route.
- The full HTML parser source was not included, so exact behavior for labels, ids, placeholders, defaults, duplicate names, malformed HTML, select options, and unsupported attributes needs direct inspection.
- Checkbox, select, color, date/time, and empty value serialization need runtime/source proof beyond the high-level object contract.
- Select rendering details are uncertain; the parity report says parsed/minimal and future dropdown work was noted.
- Exact native footer labels for `form_prompt` were not shown.
- Protocol `ForceSubmit` may bypass validation; this needs a dedicated runtime receipt.
- Richer field-definition options beyond the shared form-field subset need source and runtime receipts before documentation claims support.
