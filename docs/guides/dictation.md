# Dictation

Script Kit GPUI includes native dictation flows for Script Kit prompts, Agent Chat, notes, and saved transcript history.

## What Dictation Can Do

- capture audio from a selected macOS input device
- show a recording/transcribing overlay with target status
- deliver text into the active Script Kit surface when available
- quick-submit dictated text to Agent Chat
- save transcripts into searchable dictation history
- expose recent dictation over MCP as `kit://dictation` and `kit://dictation-history`

## First-Time Setup

Open the launcher and search for:

| Command | Use |
| --- | --- |
| `Dictation Setup` | Check model, microphone permission, selected device, and hotkey readiness |
| `Select Microphone` | Pick an input device |
| `Dictation History` | Browse saved transcripts |

Dictation readiness depends on:

1. a local transcription model being available
2. macOS microphone permission
3. at least one input device
4. an optional configured dictation hotkey if you want global voice entry

## Configure a Dictation Hotkey

Add a hotkey to `~/.scriptkit/config.ts`:

```ts
export default {
  dictationHotkey: {
    modifiers: ["meta", "shift"],
    key: "KeyD",
  },
  dictationHotkeyEnabled: true,
};
```

The hotkey registration flag does not create a shortcut by itself; set `dictationHotkey` too.

The global dictation shortcut routes to Agent Chat quick-submit. Contextual main-window and prompt dictation remains available from the launcher and active surfaces.

## Dictation Targets

Script Kit stores the intended target at dictation start so delivery is stable even if focus changes while you speak.

Targets include:

- main launcher filter
- active prompt input
- notes editor
- Agent Chat composer
- agent/harness terminal flows
- external/frontmost-app delivery paths where supported by the active route

Searchable commands include:

- **Start Dictation Here**
- **Start Dictation to AI**
- **Start Dictation to Notes**
- **Open Dictation History**

## Dictation History

Saved transcripts power both the launcher browser and MCP resources.

In the launcher, **Dictation History** lets you:

- search recent transcripts
- preview long transcript text
- paste or reuse a selected transcript
- hand a transcript to Agent Chat
- create notes from transcript content
- copy or delete entries through actions

In Agent Chat attachment flows, a saved transcript can return as a stable resource URI:

```text
kit://dictation-history?id=<entry-id>
```

## MCP Resources

External agents can read:

| Resource | Result |
| --- | --- |
| `kit://dictation` | most recent dictation provider envelope |
| `kit://dictation-history` | newest saved transcript summaries |
| `kit://dictation-history?limit=20` | newest 20 summaries |
| `kit://dictation-history?id=<entry-id>` | one transcript |

When no dictation exists, resources return explicit JSON envelopes instead of failing unexpectedly.

## Privacy Notes

- Dictation history is local user data under `~/.scriptkit/`.
- Treat transcripts as sensitive if they contain meetings, messages, code, or private names.
- Use the history browser's delete action for transcripts you do not want reused by Agent Chat or MCP readers.

## Troubleshooting

| Symptom | Try |
| --- | --- |
| No microphone detected | Open **Select Microphone** and confirm macOS sees the device. |
| Permission denied | Open **Dictation Setup** and grant microphone permission in macOS settings. |
| Hotkey does nothing | Confirm both `dictationHotkey` and `dictationHotkeyEnabled: true` are in `~/.scriptkit/config.ts`, then restart Script Kit. |
| Agent Chat did not receive the transcript | Use **Start Dictation to AI** or the configured global dictation hotkey. |
| History is empty | Complete a dictation session first, then reopen **Dictation History**. |
