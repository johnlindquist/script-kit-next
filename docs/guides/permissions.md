# Permissions

Local search, scripts, forms and notes work without blanket privacy grants. If the setup list opens, press Esc to return to Kit. Enable access when you want a feature that needs it, under **System Settings → Privacy & Security**.

## Which feature needs what

| Feature | Needs | While it's missing |
| --- | --- | --- |
| Snippets (keyword expansion) | Accessibility | Snippets don't expand. Script Kit starts them on its own once Accessibility is on; no restart is needed. |
| Pasting an AI result or history item | Accessibility or Post Events | Kit offers the text on the clipboard instead of claiming it was pasted. |
| Dictating into the original field of another app | Microphone and Accessibility | Kit cannot verify an external field without Accessibility. Dictating in Kit is still available with microphone access. |
| Selected text, window commands | Accessibility | These commands are off. |
| Dictation | Microphone | macOS asks the first time you dictate. If you said no, dictation can't hear you. |
| Screenshots, optional screen spelling and screen context | Screen Recording | Ordinary dictation does not need this. macOS may ask at first use; if it does not, check Settings. |
| Browser tab activation | Automation for the particular browser | Approval for Safari is not approval for Chrome. If denied, the tab cannot be verified or activated. |

Accessibility unlocks cross-app actions, not basic launcher use. It is the permission highlighted by first-run setup. **Input Monitoring** and **Post Events** appear in the list too, but Accessibility already covers them.

## Set Up Permissions

Search **Set Up Permissions** in the launcher. Each row shows whether a permission is on and what stops working while it's off. The list updates while you change things in System Settings.

| Key | Does |
| --- | --- |
| ↵ | Ask macOS for the selected permission, or open its System Settings page |
| ⌘R | Restart Script Kit |
| ⌘⇧R | Reset Script Kit's entry for the selected permission, then ask again |

**Check Permissions** reports which features aren't ready and opens the list when something needs attention.

## When a grant doesn't take

Sometimes System Settings shows Script Kit switched on, but the feature still says the permission is missing. This happens when the entry belongs to an older copy of the app (after an update, a rebuild, or moving the app), or when a change has not yet taken effect in the running process. If Script Kit asked for a permission and it is still off a little later, its row in **Set Up Permissions** says so and offers the two fixes:

1. **⌘R — Restart Script Kit.** This is enough for Screen Recording and often for Accessibility.
2. **⌘⇧R — Reset the entry and ask again.** This removes Script Kit's entry for that permission (with macOS's own `tccutil` command). macOS then asks again and records the current copy of the app.

To fix it by hand instead, remove Script Kit from the list in System Settings with **−**, add it again with **+**, and restart Script Kit.

## You're told when something doesn't work

Script Kit says when it can't do what you asked, and why:

- **Cursor dictation without a verified field.** Kit does not paste into a different field. If recording completed and delivery was withheld, review the retained draft in Kit; do not expect a clipboard copy. If delivery is **unknown**, check the original field before inserting anything again.
- **An app ignores an ordinary paste.** Some apps never take the text. Follow the specific copy or recovery notice; a receipt that the destination changed is not a verified paste.
- **Snippets can't run.** If macOS stops Script Kit from watching typing (Accessibility turned off, or a stale entry), a notification says so. Script Kit keeps retrying and starts snippets again on its own when it can.
- **At startup**, Script Kit reminds you only about permissions your own setup depends on. Accessibility is always worth a reminder. A missing microphone is mentioned only if you use a dictation hotkey. Screen Recording never triggers a startup reminder.

While a password field is focused, macOS hides typing from every app ("Secure Input"), so snippets pause until you leave the field.

## Dictation Setup checks

**Dictation Setup** also shows:

- **Allow typing into other apps.** This row appears when Accessibility is off, because Kit cannot verify an external cursor without Accessibility.
- **Test microphone.** It records three seconds and says whether sound comes through, is very quiet, or is silent. A muted or wrong input then shows up before you dictate.
- **A closed-lid warning.** This appears when the built-in microphone is selected and the laptop lid is closed.

### Checking a release on a clean Mac

Use a fresh test account or test Mac and the actual downloaded release, not a dev build. Don’t reset an everyday account’s privacy grants. Record the release tag, archive SHA-256, macOS version and Mac model. Open the installed app from Finder; check the first model download, the system’s microphone grant or denial, and **Test microphone** with a real input. Then try the physical shortcut in a disposable field, with Accessibility granted, and verify one transcript lands in the original field once. Keep recordings and transcript text out of the report. An upgrade is a separate check: note the old and new versions and verify existing settings and scripts survive.

The build’s first-run and permission fixtures exercise seeded files and simulated states. A packaged usage description or signed entitlement makes a macOS prompt possible; it does not prove the OS showed it, a device recorded audio, or the paste reached its target.

## For agents

`getState` includes a `permissions` object that is read live:

- `permissions[]` lists each permission with:
  - `status`: `granted`, `denied`, `notAsked` or `unknown`;
  - `observation`: the OS evidence behind that status. A false Accessibility or CoreGraphics preflight means **not granted**, not proof that a person clicked Don’t Allow. Microphone can distinguish not determined from denied/restricted;
  - `requirement`;
  - `requestedAtUnixMs` and `changedAtUnixMs`;
  - `grantLooksStuck`;
  - `settingsUrl`.
- `features[]` lists each feature with:
  - `readiness`: `ready`, `degraded`, `blocked`, `askOnUse` or `unknown`;
  - the `missing` permissions;
  - the `message` the user would see;
  - a `fix` (`grant` a permission, or `restartScriptKit`).
- `textExpansion` is the live snippet watcher state: `waitingForAccessibility`, `running`, `secureInput` (with the owning app), or `failed` (with the reason).
- `canRestart` and `bundleIdentifier` show whether the restart and reset fixes are available.

Feature readiness here describes privacy prerequisites, not the presence of an input device, a loaded model, or successful delivery. A request timestamp is not an observation of a visible macOS dialog.

The scripting SDK's `checkAccessibility()` and the built-in commands above use the same status.
