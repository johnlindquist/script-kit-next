# Form validation: follow the next character

[Screenshot walkthrough and public observations](https://brave-essence-5njm.here.now/) · First batch of the [behavior and style audit](behavior-and-style-polish-plan.md). An invalid email must block the response, show where to fix it, and send the next character to that field. Plain Return in Notes is a newline. The Submit button and ⌘Return are two ways to deliver the same values.

| Action | Before | After |
|---|---|---|
| Submit invalid Email with Notes focused | Keyboard: warning above the form. Button: status-line notice. Both left the caret in Notes. | Both focus Email, mark its field invalid and show an example address beneath it. No completion. |
| Correct Email without submitting again | The keyboard warning remained above the valid address. | Both the warning and invalid border clear. The field’s semantic error clears too. |
| Tab back to Notes, add a line, submit | The return value alone couldn’t distinguish the earlier feedback defects. | The SDK script receives the same named values once by keyboard or click. A repeat delivers no second response. Esc sends cancellation; no draft resolves. |

Validation belongs to the form and its shared field shell, not independently to the footer, key handler or automation dispatcher. Submit finds the first invalid field. Its supporting message, border and `getElements` invalid state follow its **current** value; a stored copy of the warning cannot go stale. A focused unit check covers two invalid fields, focusing them in order, without erasing the notes.

## Repeat the interaction

Use a new evidence directory and the repository's managed build; no daily app, screen capture, native input, live provider or user configuration required. The script uses a fixed HTML form, the production SDK, the ordinary prompt message handler, and an owned completion channel. The capture is the hidden GPUI framebuffer, not a macOS compositor or screen-reader test.

```sh
out=.test-output/form-audit-$(date +%Y%m%d-%H%M%S)
mkdir "$out"
bun scripts/devtools/devtools.ts build-ops act app-build --features owned-ui-evaluation --artifact-out "$out/app.ref.json"
for route in keyboard button cancel; do
  bun scripts/devtools/sdk-form-correction.ts "$out/app.ref.json" "$out/$route" "$route" || break
done
```

Each run checks the rendered field, semantic error, editing destination, and what the script actually received. Its `result.json` includes frame identities, steps, exit values and cleanup. The `fields()` neighbor was also exercised in the owned fixture: it preserved **array order**, as distinct from the HTML form’s **named object**. A separate matched comparison tested keyboard and click at the same scale and field values; both the invalid field area and the corrected field area matched pixel for pixel.

## Scope accounting for subsequent batches

The surface report’s source and generated mappings agree. Compared with the 37-kind/54-mapping source at `2ee4116a7`, the current 34-kind/49-mapping source no longer declares five independent views: `AgentChatView`, `AgentChatHistoryView`, `ConfirmPrompt`, `DictationHistoryView` and `HotkeyPrompt`. Three of these also retired a dedicated contract kind (chat, past chats and confirm). Their **features still belong in the audit**: exercise chat/history under the main input, questions and holds in the line, dictation history in the voice routes, and shortcut recording in place. A stale static coverage expectation is not evidence that five features vanished or that any passed at runtime. The global coverage binding remains blocked pending migration of its consumers; this batch grades only forms.

A separate broad DevTools safety check also flags the help inventory: 119 commands are listed against its minimum of 120. The last registry change (`583d93bc6`) retired the modal-stack scenario. The form changes do not touch that registry. Retain the failing check for the inventory reconciliation rather than suppressing it or restoring a retired modal.
