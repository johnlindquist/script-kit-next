# Dictation: finish, cancel, restore, forget

The fast path is still **hold, speak, release, keep working**. Cancel means “don’t deliver this”; Forget means “destroy Kit’s copy.” Restore gets words back without writing into another app.

## While speaking

Defaults below refer to the configurable [main and dictation shortcuts](gesture-grammar.md). The listening line names the destination: for example, **Writing in Mail**, **Dictating into Script Kit**, or **Review in Markdown**. During a recording, the line shows the available keyboard controls; ⌘K and bare Shift stay with the foreground app. The switch can be reassigned or disabled.

| Intent | Default gesture | Contract |
|---|---|---|
| Open Kit / open Today | Tap `⌘;` from another app / tap twice | The first release opens the launcher, not a stale note. Two presses open Today; holding the second dictates there. A single tap in focused Kit retains its existing window toggle. |
| Finish in the original field | Hold `⌘;`, speak, release | Insert the final text once at the captured selection if it is still safe. Either release order means finish. No confirmation on a safe direct write; no interpretation as a command, even for “send this email.” |
| Dictate hands-free | Tap `⌘⇧;` to start; tap again to finish | No need to keep a key down. The contextual route uses the same finish/cancel rules as a hold. Show the actual destination, including any explicitly configured target. |
| Dictate into Kit | Bind **Dictate into Kit** | Start hands-free in Kit; press its shortcut again to finish. **Dictation Starting Destination** changes where the usual shortcut starts; explicit Today stays Today. |
| Use in Kit | Press **⇧Esc** while listening | Toggle between the original destination and editable launcher input for the **whole** recording. Keep holding the main shortcut while adding ⇧Esc. Neither bare Shift nor modifier release is an action. Release does not run a reading; a separate deliberate `↵` does. |
| Review in Markdown | **Review Dictation in Markdown** (bind if desired) | Finish into an editable draft instead of the original field. It is not Today. Length, wording and pauses never select this route. |
| Cancel | `Esc` while recording or finishing, before dispatch | Stop the mic and revoke delivery for this take immediately. Ignore its later hotkey release and late results. Don’t paste, copy, send, open an editor, or steal focus. Say what actually survived: **Cancelled · Restore in Kit**, **Cancelled · No recording kept**, or **Couldn't save the recording**. If dispatch already occurred, don’t claim it was cancelled. |

## After recording

Use the launcher and its existing `⌘K` list rather than another set of global modifiers. **Last Dictation**, **Open Dictation History**, and **Recently Cancelled** are searchable commands and can take personal bindings. Last Dictation opens the latest finished take for review; it does not silently paste it. Cancelled takes have their own list. Ordinary history searches original and edited words and identifies which matched.

| Verb | Keyboard path | Effect |
|---|---|---|
| **Restore / Open** | `⌘Z` in Kit’s empty launcher **only while the line offers Restore**; otherwise `↵` on a selected take | Open an editable draft. If only audio survived, offer **Transcribe** instead of pretending text exists. Opening doesn’t promote a private take to normal history. Show its expiry; **Keep in History** explicitly saves it and its edits. Outside Kit, `⌘Z` stays with the active app; inside an editor, it stays text undo. |
| **Copy / Paste** | `⌘K` on the take → **Copy Text** or **Paste into <named target>** → `↵` | Copy changes only the clipboard. Paste explicitly targets a field and rechecks it; a different target requires a new choice. No silent clipboard fallback and no automatic second paste if the first result is unknown. |
| **Retry / Regenerate** | `⌘K` → **Retry Transcription** on a failure, or **Regenerate from Audio** on a completed take | Use that take’s saved audio, never the newest unrelated take. Preserve existing raw text and user edits; open a new candidate for comparison. If there is no audio, explain why retry isn’t available. A paste failure needs the saved **text**, not another transcription. Neither verb delivers text. |
| **Forget** | `⌘K` → **Forget This Recording** → `↵` | Start Kit’s **8-second cancellable countdown** in the line; don’t require the key to be held. Use `⌘Z` on empty input or the stop row before expiry. After the countdown and successful removal there is no Kit undo. No one-key permanent deletion. |

## What “protected” means

- **Accidental cancellation:** with recovery enabled, keep each cancelled take privately on-device for **24 hours from cancellation**, not just the last one. No transcription, remote reading, ordinary search result, agent resource or learned command from a cancelled take without an explicit next action. Expire access on time and purge at next startup if Kit was closed. With saving disabled, use memory only and explicitly say it cannot be recovered after closing.
- **App failure:** when saving is enabled, journal audio in short, durable chunks from the start of capture. After a crash recover only what was committed; label it **Interrupted**, never queue it for delivery. Disk full or a failed write is not a “saved” receipt. Keep audio, raw text, revisions and delivery state together under a stable take ID; don’t publish text or audio in logs or passive diagnostics.
- **Wrong or uncertain destination:** check the original app, window, non-secure field and selection before writing. A change, unsupported target, or missing permission leaves a draft, not a best-guess paste. A dispatched paste without proof of the outcome is **Delivery unknown**, not “failed, retrying.” Partial speech is local preview only. Restoring words never restores an old permission to paste or run them.
- **Intentional erasure:** Forget must invalidate in-flight work and remove all Kit-owned audio, raw and edited text, drafts, indexes and agent projections for that ID. Don’t report **Forgotten** until cleanup succeeds. This cannot recall a paste, someone’s clipboard copy, or an independent backup.

## Boundary and acceptance

Protection applies to cursor dictation and **Dictate into Kit**. The separate Talk, legacy recordings and dedicated chat actions have different lifecycles. Unlinked legacy WAV files cannot reliably be identified from a historical transcript; forgetting a legacy row does not guess which WAV to remove. An external paste is not an atomic operation with its AX checks. Native hotkey delivery and insertion in a particular app need qualification on that desktop. See the [dictation guide](../guides/dictation.md) and [comparison of installed Mac apps](https://faint-tangle-3qxj.here.now/).

A distinguishing test: dictate into Mail, press Escape, release the shortcut, restart Kit, and restore. Mail and the clipboard must be unchanged. Only the cancelled take should open, with its surviving words or audio. Reprocess it, change focus, and forget it while a callback is pending: no automatic delivery and no resurrection. Check the no-saving case separately: no durable take should exist to restore.
