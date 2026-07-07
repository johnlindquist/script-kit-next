# Dictation

Script Kit GPUI includes native, local dictation: hold a hotkey, speak, and the transcript lands in the surface you chose — a prompt, the frontmost app, Notes, or Agent Chat. Transcription runs on-device.

## First-Time Setup

Open the launcher and search:

| Command | Use |
| --- | --- |
| **Dictation Setup** | Check model download, microphone permission, and device readiness |
| **Select Microphone** | Pick an input device (`None` = system default) |
| **Open Dictation History** | Browse saved transcripts |

Readiness requires a downloaded transcription model, macOS microphone permission, and at least one input device. Models are fetched on demand from Dictation Setup:

- `parakeet-tdt-0.6b-v3` (default) — auto-detects its supported languages
- `whisper-medium` — supports an explicit `language` hint

## The Hotkey

The default dictation hotkey is `⌘⇧;` (Cmd+Shift+Semicolon), registered by default and configurable:

```typescript
export default {
  dictationHotkey: { modifiers: ["meta", "shift"], key: "Semicolon" },
  dictationHotkeyEnabled: true,
};
```

**Push-to-talk is the default**: hold the hotkey, speak, release to stop and transcribe. A quick tap keeps classic toggle behavior (tap to start, tap to stop).

## Live Partials

While recording, the overlay shows a live partial transcript so you can see recognition in flight (on by default; `dictation.livePreview: false` disables it).

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
| **Start Dictation to Current App** | The active Script Kit surface / current app |
| **Start Dictation to App** | The frontmost macOS app |
| **Start Dictation to Notes** | The Notes editor |
| **Start Dictation to Agent Chat** | Agent Chat quick-submit |

## Dictation History

With `saveHistory` on, transcripts are stored locally and searchable via the **Dictation History** built-in — preview, paste, reuse, hand to Agent Chat, or delete entries. Saved transcripts are also exposed over MCP:

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
| Hotkey does nothing | Check `dictationHotkeyEnabled` isn't `false`; restart Script Kit after config changes |
| Quiet mic drops dictations | Lower `dictation.silenceRms` (default 0.01) |
| Recording never stops | `maxDurationSecs` auto-stops at 600 s by default |

## Related

- [Feature Tour](./feature-tour.md) — where dictation fits with the other global hotkeys
- [MCP and Agent Context](./mcp-and-agent-context.md) — dictation history as agent context
