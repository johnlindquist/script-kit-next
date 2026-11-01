# Native form foundation

[Light/dark screenshot walkthrough and measured outcomes](https://ivory-sierra-cm9e.here.now/)

The label tells you what the value means. The accent outline tells you where typing goes. An error tells you what to change. These are independent: an invalid field keeps its focus outline and gains a warning icon and actionable text. Color is not the only error signal.

| Surface | What owns it |
|---|---|
| SDK HTML form, ordered Fields, multiline inputs, checkboxes | [FormPromptState](../../src/form_prompt.rs) owns traversal, reveal and submission. [Form fields](../../src/components/form_fields.rs) own drawing and editing. Prefer an explicit label; an unlabeled HTML field gets a persistent label from its name, not a disappearing placeholder. HTML still returns a named object; Fields returns an ordered array. |
| Naming and template variables | The same native text editor owns caret placement, selection and insertion. The host supplies the current value, validation and preview. Its key route applies a widget edit to the domain model in the same event. Keep the actual focused editor as the focus target; don't focus the outer shell again. |
| Credential/environment input | The shared field shell wraps the protected input. Prose uses the UI font; the value uses monospace. For a nonempty secret, Show or Control-Shift-R reveals the typed value for up to five seconds. Show/Hide have different semantic IDs, and inspecting the field still gives masked content. |
| One-window command forms | Keep the one main input. Rows show separately aligned labels and values even when not selected. Click or select a field, or press Tab/Shift-Tab; edits belong to the current field immediately, not to a debounced launcher search. Use the field’s stable slot ID for selection. |
| Handler forms; preset editor | Handler controls reuse the field shell without nesting a second border. The preset editor uses native fields too: Return adds a newline in Instructions, ⌘Return saves the preset. The preset launcher command is intentionally unregistered; its owned evaluation fixture is not evidence of a new launcher entry. |
| Previews and specialized controls | A filename, destination or rendered template is read-only output, not a focusable-looking form field. Theme sliders, pickers, shortcut recorders, and filters retain their own control behavior. |

**Adding a form:** start with the [field shell](../../src/components/form_fields/shell.rs) and [field colors and spacing](../../src/components/form_fields/colors.rs). Reuse the real editors for editable text, not a label inside a rectangle with a key listener. A surface-specific editor should supply borderless content to that same shell. For read-only information use `prompt_preview_style`. Agents should be able to find the same enabled, focused and invalid controls through `getElements`, and submit via the same domain operation. Do not return private input in semantic labels or error payloads.

**Regression routes:** `p4-command-fields` exercises reminder field picking and switching; `p4-template-editor` checks middle insertion and bidirectional traversal. Both are in [the one-window chapters](../../scripts/agentic/one-window/chapters/). [The SDK round trip](form-correction.md) checks actual delivery and cancellation. The application test `long_form_reveals_the_keyboard_field_and_returns_to_the_invalid_one` checks that the focused field is actually in the viewport—not merely selected in state.
