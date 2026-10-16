# Permissions

Several Script Kit features need a macOS privacy permission. You give it once in **System Settings → Privacy & Security**. This page lists which features need which permission, what happens while a permission is missing, and how to fix a grant that doesn't take.

## Which feature needs what

| Feature | Needs | While it's missing |
| --- | --- | --- |
| Snippets (keyword expansion) | Accessibility | Snippets don't expand. Script Kit starts them on its own once Accessibility is on; no restart is needed. |
| Pasting into other apps (dictation, AI results, history) | Accessibility | Text is copied to the clipboard instead, and Script Kit tells you so. |
| Selected text, window commands | Accessibility | These commands are off. |
| Dictation | Microphone | macOS asks the first time you dictate. If you said no, dictation can't hear you. |
| Screenshots, screen context for AI | Screen Recording | macOS asks the first time you use them. |

Accessibility is the only permission Script Kit needs for most of what it does. It is the one the first-run setup asks for. **Input Monitoring** and **Post Events** appear in the list too, but Accessibility already covers them.

## Set Up Permissions

Search **Set Up Permissions** in the launcher. Each row shows whether a permission is on and what stops working while it's off. The list updates while you change things in System Settings.

| Key | Does |
| --- | --- |
| ↵ | Ask macOS for the selected permission, or open its System Settings page |
| ⌘R | Restart Script Kit |
| ⌘⇧R | Reset Script Kit's entry for the selected permission, then ask again |

**Check Permissions** reports which features aren't ready and opens the list when something needs attention.

## When a grant doesn't take

Sometimes System Settings shows Script Kit switched on, but the feature still says the permission is missing. This happens when the entry belongs to an older copy of the app (after an update, a rebuild, or moving the app), and Screen Recording always needs a restart. If Script Kit asked for a permission and it is still off a little later, its row in **Set Up Permissions** says so and offers the two fixes:

1. **⌘R — Restart Script Kit.** This is enough for Screen Recording and often for Accessibility.
2. **⌘⇧R — Reset the entry and ask again.** This removes Script Kit's entry for that permission (with macOS's own `tccutil` command). macOS then asks again and records the current copy of the app.

To fix it by hand instead, remove Script Kit from the list in System Settings with **−**, add it again with **+**, and restart Script Kit.

## You're told when something doesn't work

Script Kit says when it can't do what you asked, and why:

- **Dictation without Accessibility.** Your words are put on the clipboard, and a message says why. If the Script Kit window is hidden, the message is a system notification.
- **An app ignores a paste.** Some apps, or a click outside a text field, never take the text. Dictation and AI results are left on the clipboard so you can press ⌘V. A snippet gives you your clipboard back, and a notification names the snippet.
- **Snippets can't run.** If macOS stops Script Kit from watching typing (Accessibility turned off, or a stale entry), a notification says so. Script Kit keeps retrying and starts snippets again on its own when it can.
- **At startup**, Script Kit reminds you only about permissions your own setup depends on. Accessibility is always worth a reminder. A missing microphone is mentioned only if you use a dictation hotkey. Screen Recording never triggers a startup reminder.

While a password field is focused, macOS hides typing from every app ("Secure Input"), so snippets pause until you leave the field.

## Dictation Setup checks

**Dictation Setup** also shows:

- **Allow typing into other apps.** This row appears when Accessibility is off, so you know dictation will copy instead of type.
- **Test microphone.** It records three seconds and says whether sound comes through, is very quiet, or is silent. A muted or wrong input then shows up before you dictate.
- **A closed-lid warning.** This appears when the built-in microphone is selected and the laptop lid is closed.

## For agents

`getState` includes a `permissions` object that is read live:

- `permissions[]` lists each permission with:
  - `status`: `granted`, `denied`, `notAsked` or `unknown`;
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

The scripting SDK's `checkAccessibility()` and the built-in commands above use the same status.
