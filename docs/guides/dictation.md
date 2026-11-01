# Dictation

Hold the main shortcut to dictate into the field where you started. Press ⇧Esc during recording to put the final words into Kit instead. Release finishes transcription; it does not execute a command. Tap twice for Today, or hold the second press to dictate there. [Shortcut and focus map](../specs/gesture-grammar.md). Transcription runs on-device.

## First-Time Setup

Open the launcher and search:

| Command | Use |
| --- | --- |
| **Dictation Setup** | Check model download, microphone permission, and device readiness |
| **Select Microphone** | Pick an input device (`None` = system default) |
| **Open Dictation History** | Browse saved transcripts |

Readiness requires a downloaded transcription model, macOS microphone permission, and at least one input device. Dictation Setup also shows whether Script Kit can type into other apps (direct cursor delivery needs Accessibility; if the field cannot be verified, the words are kept for review instead). It offers a three-second microphone test and warns when the built-in microphone is selected with the lid closed. See [Permissions](./permissions.md). Models are fetched on demand from Dictation Setup:

- `parakeet-tdt-0.6b-v3` (default, 740 MB) — auto-detects its supported languages
- `whisper-medium` (583 MB) — supports an explicit `language` hint

Both models run on the Mac's GPU through Metal. If the GPU can't load the model or a GPU pass fails, transcription continues on the CPU. Downloads resume if interrupted, and each file is checked against its published SHA-256 before it is used. Models live in `~/.scriptkit/models/`.

To check which processor is in use, read the passive dictation state (`getState`): its `engine` object reports `backend`, `device`, `gpu`, `cpuFallbackReason`, the last load time and the last transcription time. It appears after the first transcription.

## The Hotkey

The default dictation hotkey is `⌘⇧;` (Cmd+Shift+Semicolon), registered by default and configurable:

```typescript
export default {
  dictationHotkey: { modifiers: ["meta", "shift"], key: "Semicolon" },
  dictationHotkeyEnabled: true,
};
```

**Push-to-talk is the default**: hold the hotkey, speak, release to stop and transcribe. A quick tap keeps classic toggle behavior (tap to start, tap to stop).

**Dictate into Kit** (`builtin/dictate-into-kit`) is a separate bindable shortcut: start listening for Kit, then finish into editable launcher text. **Dictation Starting Destination** lets the usual shortcut start in Kit instead. The ⇧Esc switch is temporary: if its chord conflicts, choose a different `switchShortcut`. Configure a different chord when the hold shortcut already contains Shift; Kit does not guess whether an ambiguous Esc means switch or cancel. You can also bind **Cancel Dictation** (`builtin/cancel-cursor-dictation`). Outside cursor recording neither Escape combination belongs to dictation. ⌘K remains available to the foreground app.

## The “Talk” launcher action

The explicit Talk view is separate from direct cursor dictation. In that view, the list under the input is one header and two rows, each with Jev's percentage:

- **Type it into <app>** (`⌘1`) — paste the words at the cursor you were in. The line above the input reads "Typing into · <app>" while this leads.
- **The request** (`⌘2`) — run the words as something that should happen ("summarize what I copied").

Letting go acts on the final words only when Jev agrees and is at least 65 % sure. Otherwise the line reads "Pick one · ⌘1 or ⌘2". `Esc` drops everything.

## Spell names from the window

On macOS, dictation uses visible Accessibility text from the original window by default. It doesn’t take screenshots in this mode. For example, with `TypeScript` visible, “the type script project” can paste as “the TypeScript project”. Context applies to the final paste, not live partials. It fixes matching words, spacing and case; it does not guess that *Shawn* means *Sean*.

In **Dictation Setup**, choose **Use visible text and window OCR for screen spelling** for windows with little readable text. This option requires existing macOS Screen Recording permission as well as Accessibility; recording never opens a permission dialog. Only the original window is eligible, not the display. OCR can misread characters. Your personal vocabulary takes precedence.

Collection runs locally while you speak; if the terms aren't ready at delivery, dictation continues without them. No image, window text or term list is saved or sent to a model. The resulting dictated words follow your normal history settings. Password-manager exclusions and protected focused fields are skipped. Switching to Kit or Markdown discards the context. Choose **Turn off screen spelling** to turn it off.

## Live Partials

While recording, the main input shows the words as they're recognized so you can see recognition in flight (on by default; `dictation.livePreview: false` disables it).

## Configuration Reference

All keys live under `dictation` in `~/.scriptkit/config.ts`:

```typescript
export default {
  dictation: {
    selectedDeviceId: "usb-mic",   // microphone; omit for system default
    model: "parakeet-tdt-0.6b-v3", // or "whisper-medium"
    language: "en",                // Whisper only; Parakeet auto-detects
    startInKit: false,             // usual cursor shortcut starts at the original field
    screenSpelling: "accessibility", // default; alternatives: "off", "accessibilityAndOcr"
    switchShortcut: "Shift+Escape", // use e.g. "Alt+F12" or "off"
    keepRecordings: true,          // protected audio; saveHistory:false disables it too
    pushToTalk: true,              // hold-to-record (default: true)
    livePreview: true,             // live partial transcripts (default: true)
    saveHistory: true,             // append to dictation history (default: true)
    silenceRms: 0.01,              // silence gate; lower for quiet mics
    maxDurationSecs: 600,          // runaway-recording guard; 0 disables
  },
};
```

## Dictation Targets

Script Kit records the intended target when dictation starts, so delivery is stable even if focus changes while you speak. Searchable variants:

| Command | Delivers to |
| --- | --- |
| **Start Dictation to Current App** | The active Script Kit view / current app |
| **Start Dictation to App** | The frontmost macOS app |
| **Start Dictation to Notes** | Today's note: the one row is "Add it to today's note", and letting go appends the words ("Added to today's note: …") |
| **Start Dictation to Agent Chat** | The model's chat: the one row is "Send it to the model", and letting go opens the chat with the words as the first message |

## Dictation History

With `saveHistory` on, transcripts are stored locally and searchable via the **Dictation History** built-in: rows under the same input, newest first. `↵` opens words for review. `⌘K` stages an entry: copy it, paste into the named field if available, review recording versions, hand it to Agent Chat or forget it after an eight-second countdown. **Last Dictation** opens the newest history entry without pasting. Saved transcripts are also exposed over MCP:

```text
kit://dictation                      # most recent dictation envelope
kit://dictation-history              # newest transcript summaries
kit://dictation-history?limit=20
kit://dictation-history?id=<entry-id>
```

**Recently Cancelled Dictations** is a separate, private list. Cancel with Esc before dispatch; use ⌘Z in Kit’s empty launcher when offered, or open the recovery list later. Cancelled takes expire after 24 hours. If only audio survived, choose **Transcribe saved audio**. A new version does not overwrite the draft. Choose **Keep draft in History** to retain a private draft and make it searchable. [Recovery and failure behavior](../specs/dictation-recovery.md).

## Privacy Notes

- Transcription is local; transcripts live under `~/.scriptkit/`.
- Set `dictation.saveHistory: false` to keep dictated text out of plaintext history.
- Use the history browser's delete action for transcripts you don't want reused by Agent Chat or MCP readers.

## Troubleshooting

| Symptom | Try |
| --- | --- |
| No microphone detected | **Select Microphone** — confirm macOS sees the device |
| Permission denied | **Dictation Setup** → grant microphone permission in macOS Settings |
| Delivery unknown or field changed | Check the original field first. Use **Recently Cancelled Dictations** or **Review Dictation in Markdown** to recover text. Kit does not repeat an uncertain paste. |
| Dictation hears nothing | **Dictation Setup** → **Test microphone** |
| Hotkey does nothing | Check `dictationHotkeyEnabled` isn't `false`; restart Script Kit after config changes |
| Quiet mic drops dictations | Lower `dictation.silenceRms` (default 0.01) |
| Recording never stops | `maxDurationSecs` auto-stops at 600 s by default |

## Related

- [Feature Tour](./feature-tour.md) — where dictation fits with the other global hotkeys
- [MCP and Agent Context](./mcp-and-agent-context.md) — dictation history as agent context
