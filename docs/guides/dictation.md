# Dictation

Hold the main shortcut to dictate into the field where you started. Tap Shift during recording to put the final words into Kit instead. Release finishes transcription; it does not execute a command. Tap twice for Today, or hold the second press to dictate there. [Shortcut and focus map](../specs/gesture-grammar.md). Transcription runs on-device.

## First-Time Setup

Open the launcher and search:

| Command | Use |
| --- | --- |
| **Dictation Setup** | Check model download, microphone permission, and device readiness |
| **Select Microphone** | Pick an input device (`None` = system default) |
| **Open Dictation History** | Browse saved transcripts |

Readiness requires a downloaded transcription model, macOS microphone permission, and at least one input device. Dictation Setup also shows whether Script Kit can type into other apps (that needs Accessibility; without it your words are copied to the clipboard instead). It offers a three-second microphone test and warns when the built-in microphone is selected with the lid closed. See [Permissions](./permissions.md). Models are fetched on demand from Dictation Setup:

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

## The “Talk” launcher action

The explicit Talk view is separate from direct cursor dictation. In that view, the list under the input is one header and two rows, each with Jev's percentage:

- **Type it into <app>** (`⌘1`) — paste the words at the cursor you were in. The line above the input reads "Typing into · <app>" while this leads.
- **The request** (`⌘2`) — run the words as something that should happen ("summarize what I copied").

Letting go acts on the final words only when Jev agrees and is at least 65 % sure. Otherwise the line reads "Pick one · ⌘1 or ⌘2". `Esc` drops everything.

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

With `saveHistory` on, transcripts are stored locally and searchable via the **Dictation History** built-in: rows under the same input, newest first. `↵` pastes; `⌘K` stages an entry in the line to reuse it, hand it to Agent Chat, or delete it (the delete waits in the line with `⌘Z` to stop it). Saved transcripts are also exposed over MCP:

```text
kit://dictation                      # most recent dictation envelope
kit://dictation-history              # newest transcript summaries
kit://dictation-history?limit=20
kit://dictation-history?id=<entry-id>
```

## Privacy Notes

- Transcription is local; transcripts live under `~/.scriptkit/`.
- Set `dictation.saveHistory: false` to keep dictated text out of plaintext history.
- Use the history browser's delete action for transcripts you don't want reused by Agent Chat or MCP readers.

## Troubleshooting

| Symptom | Try |
| --- | --- |
| No microphone detected | **Select Microphone** — confirm macOS sees the device |
| Permission denied | **Dictation Setup** → grant microphone permission in macOS Settings |
| Words land on the clipboard instead of in the app | Script Kit needs Accessibility to type into other apps, or the app didn't take the paste: press ⌘V. See [Permissions](./permissions.md) |
| Dictation hears nothing | **Dictation Setup** → **Test microphone** |
| Hotkey does nothing | Check `dictationHotkeyEnabled` isn't `false`; restart Script Kit after config changes |
| Quiet mic drops dictations | Lower `dictation.silenceRms` (default 0.01) |
| Recording never stops | `maxDurationSecs` auto-stops at 600 s by default |

## Related

- [Feature Tour](./feature-tour.md) — where dictation fits with the other global hotkeys
- [MCP and Agent Context](./mcp-and-agent-context.md) — dictation history as agent context
