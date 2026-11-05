# Change how your launcher works

Type **Customize Kit** for examples. Pick one to put its words in the input, then edit the request or press Return on the proposed change. Picking an example does not save anything.

You can speak the request, too. With [Dictation](dictation.md) set up, choose **Dictate into Kit** and say **Keep my launcher minimal**. Stop recording to put the words in Kit’s input. Edit them if necessary, then press Return on **Show just scripts on the launcher home** to save. Saying the same words while dictating into a note just writes them in the note; it does not change the launcher.

Type **Make “Stays open” brighter** in the launcher. The **Customize Kit** row names the edit. Press Return to apply it. Pin the window with ⌘⇧P to see the label; the input and pin icon keep their usual appearance. **Make the pinned label bold only while dictating** changes its weight without losing the brighter color. It uses the recording indicator, not the microphone or transcript.

Want a quieter starting point? Type **Keep my launcher minimal** and press Return on **Show just scripts on the launcher home**. Open Kit with an empty input to see your scripts and scriptlets instead of subject tabs, apps and suggestions. Required setup still appears. Nothing is uninstalled: type a name to search normally, including apps and notes. This choice survives closing and restarting Kit. **Restore the standard launcher** brings the tabs and suggestions back without changing your other preferences. Agents can preview or set `launcher.home.mode` to `minimal` or `standard`.

To prefer filing highlighted text over asking about it, type **Put today’s note first for selected text**. This moves the existing note action on the Selection tab. The preference stores the action name, not the selected words. Opening Kit still uses the selection access you enabled. Nothing goes into today’s note until you run the action, and the Clipboard tab keeps its own order. An unavailable action is never added to a subject just because it was requested first.

These requests work offline. They take effect when you submit their launcher row, not when you dictate the same words into another app. Ask **Why is the pinned label brighter?** for the source. Press ⌘Z on empty input after a change, or type **Undo my last launcher change** after reopening Kit. Undo refuses to overwrite a newer revision.

**Disable my launcher customizations** uses stock behavior without deleting your rules. **Enable my launcher customizations** restores them. If the saved file cannot be verified, Kit uses stock behavior and leaves the file alone. Starting Kit with `SCRIPT_KIT_PERSONAL_SAFE_MODE=1` also ignores personal rules. Neither disabling rules nor Undo removes a note you already saved.

### Recover a damaged launcher file

If `--personalize inspect` reports `invalid_document`, Kit cannot verify the saved rules. Quit Kit and stop any other author editing this workspace. In the affected workspace’s `plugins/main/customizations/` folder, **move** `launcher.json` to a new, unused backup name. Leave `db/personal-changes/` alone. Run `--personalize inspect` again: `revision: "stock"` means there is no active custom file. You can reopen Kit with its defaults. This does not change your notes or scripts.

If you have a known-good copy of the original file, quit Kit before restoring it. Move any newer active file aside rather than overwriting it. Restoring the exact saved bytes in the same workspace can restore the matching Undo history; check `inspect.undo` before undoing. Don’t restore the damaged backup merely to re-enable a rule.

## Make a personal list

Type **Make a list of my pinned notes called Reading shelf**, then choose **Create “Reading shelf”**. Type **view reading shelf** to open it. For all notes with the latest changes first, try **Make a list of my notes newest first named Recently edited**. For a read-only list of panes, try **Make a list of Herdr panes that need me called Attention** and pair a space before opening it. These requests work without a model. A name already in use is not overwritten. A configured launcher author can compose other lists from the installed schema. The view uses Kit’s normal input and rows. Saving it does not open a note or read its contents. Type **my views** or **view reading** in the launcher, select the list and press Return. Type to narrow its titles and press Return to open the selected note. If the note was renamed, unpinned from a pinned-only list, or moved outside the result limit, Kit asks you to choose again instead of opening a different result. Escape returns from the editor to the same filtered list; Escape again returns to the launcher.

Kit shows the title you chose in the launcher. In passive `getElements` snapshots, an authored list row and its header expose private measurements rather than the title. Select a row by its current semantic ID. To read the declaration itself, use the workspace’s `--personalize inspect` or an authorized launcher reader.

Check `describe` for `views.schema` first. If it is absent, this version of Kit cannot create personal lists. Preview the following change before saving it. For `apply`, use the workspace and revision from `inspect` and generate a fresh version-4 request ID:

```json
{"op":"putView","view":{"id":"reading","title":"My pinned notes","limit":24,"list":{"source":"kit.noteTitles","filter":"pinned","sort":"title","detail":"lastChanged","enter":"openNote"}}}
```

A view name is not a filesystem path. A list can show up to 48 rows, and a workspace can keep eight views. Sorting and filtering happen before the row limit; typing in an open view narrows that bounded list. Saving a different declaration with the same ID updates the view. Undo restores its previous definition without changing any note. Pane recipes do not grant access to Herdr: an unconnected source shows an unavailable message, never a substitute list.

## Share a read-only Herdr pane list

To see which panes need you without exposing terminal contents, share one **local** Herdr space with one Kit workspace. This requires macOS, a direct Bun executable, and a Herdr version with the pane-metadata mod API. Kit does not search for a running terminal app, install a mod into it, or change your layout.

1. In an **operator shell**, select the Kit workspace with `SK_PATH` (use its physical, absolute path). Choose the Herdr app-home directory and the ID of the local space you intend to share. If you need the ID, use `herdr-app ls` in your own Herdr session; do not send that listing to the launcher author. Run `scriptkit panes offer "$herdr_home" "$space_id"`. The command creates a new private `kit-pane-share-v1` directory there; it refuses to overwrite an existing offer. Keep the returned `marker` path for pairing and the `offerId` for withdrawing this offer. The offer names this Kit workspace as well as the space: another workspace cannot claim it, even if it has that path or later occupies the same folder name.
2. In Herdr, install the optional `herdr-pane-metadata` mod from the Kit workspace’s `share/herdr-pane-metadata/` as a personal mod. Review it as you would any Herdr mod: mods run with Herdr’s own privileges. Open it once in Herdr.
3. Run `scriptkit panes pair "$marker" "$space_id"`. A fresh reply from that space is required, even if it has no live panes. `scriptkit panes status` reports the saved pairing, **not** whether Herdr is online. `scriptkit panes list needsAttention attentionFirst 8` makes a fresh read; `state: "ready"` with no rows means no pane reported blocked or done. A missing reply is not an empty list.
4. Ask a launcher author for a read-only list, or preview this declaration:

```json
{"op":"putView","view":{"id":"needs-me","title":"Panes that need me","limit":8,"list":{"source":"herdr.paneMetadata","filter":"needsAttention","sort":"attentionFirst","detail":"status","enter":"none"}}}
```

Open **view needs-me** in Kit. Rows use generated labels such as **Pane 4**, never the terminal’s title, command line, folder or output. While the list is open, Kit checks for changes about every four seconds. You can filter and select a row. Return cannot focus or zoom it. An unavailable or changed pairing clears the rows; reopen the list for another read.

If `panes status` says `renewalRequired`, the saved pairing predates workspace-bound offers. Unpair its displayed `pairId`, retire the old offer as described below and use the current mod with a fresh offer. An old listening file cannot authorize the new connection. The workspace check prevents accidental cross-pairing by Kit; it is not a sandbox against software already running with your account’s file access.

To stop Kit reading and discard an in-flight read, run `scriptkit panes unpair "$pair_id"` with the current `pairId` from `panes status`. This detaches Kit; it does not withdraw the provider’s offer. To stop sharing from Herdr, use the **same Kit workspace** and the Herdr home you chose:

```sh
scriptkit panes offer-status "$herdr_home"
scriptkit panes withdraw "$herdr_home" "$offer_id"
```

`offer-status` reads the local grant, not the terminal or the running mod. Check its space before using its `offerId`. Withdrawing sets that grant’s capability to `null`; it does not delete any files, close a pane, or silently unpair Kit. It works before pairing, while Herdr is offline, and in Kit safe mode. Repeating it for the same withdrawn offer makes no further change. Kit creates each new offer in a fresh exchange, so an old ID cannot withdraw a replacement made by Kit. Do not edit the grant in place from another program while withdrawing it. If the result is uncertain, inspect the offer again before doing anything else. If the original Kit workspace is gone or the grant cannot be verified, close the Herdr mod; do not switch to a neighboring workspace to force withdrawal. A withdrawn grant stops fresh provider reads, including after restart; it cannot recall a reply already produced or immediately clear a row already on screen.

To share again, unpair Kit, close the old mod and move its **stopped** exchange aside. Then make a new offer and pair deliberately; do not empty a live exchange or re-enable an old grant by editing its file. Kit does not sync the private pairing with personal rules. None of these operations confers terminal control, including on a launcher author or an agent that can receive handoffs.

## Give one agent access

A custom Agent Chat profile can list `tools: launcher_read, launcher_apply`. Neither tool is added to a built-in profile. These tools can change the named launcher preferences, not scripts, desktop permissions, or the contents of a selection. Give a profile only `launcher_read` if it should propose rather than apply. A profile with only these launcher tools works outside your project directory. Kit skips project and global hint files, does not attach notes or selected text, and checks the engine’s actual tool list before sending. The profile’s own instructions and chosen model still apply. With that profile selected, type **Customize Kit: …** followed by the change you want. A known offline request shows its local edit. For a novel request, **Review request** opens a clean draft; Return in chat sends only your words to the selected launcher author. If the selected profile is missing, changes before sending, or includes wider tools, Kit does not fall back to a general assistant. You can also open a clean chat with ⌘↵. Use the model you already configured; the direct launcher requests above do not require a provider.

A dedicated launcher writer asks about **each save**, even if ordinary chats have Full access. Read the proposed effect above **Save once** and **Don’t save**. For example, semibold whenever Kit is pinned also affects the time between recordings; semibold only while dictating does not. Choose **Don’t save** or press ⌘2 if the proposal is wrong. The existing rule stays in place. If the engine skips review or submits different arguments, `launcher_review_required` means that call cannot save. Check `status` with the original request ID after an uncertain reply; do not assume the edit was made. A review describes the declaration, not proof that a window adopted it or an action ran.

For an external agent, the operator can separately grant `launcher:read` and `launcher:apply` with `scriptkit grant-agent`. No existing credential gets either grant automatically. The agent's `scriptkit agent launcher describe` lists supported targets; `inspect` and `preview` report stored or proposed state. After a permitted `apply`, use `scriptkit agent launcher live` to compare the actual window’s adopted revision. A matching revision means the window adopted the declaration. It does not prove a data source is connected or that an action ran. This returns preferences, not the current search, selection, or clipboard. See `scriptkit --skill` for the request shapes.

## When Kit needs a native change

A request such as “Make the cold first key faster” needs a native comparison, not a launcher rule. A local development agent can ask the installed executable for a read-only handoff:

```sh
printf '%s\n' '{"request":"Make the cold first key faster","proof":"Compare time from the first key to the correct actionable row on the same offline catalogue"}' |
  "/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui" --personalize native-plan
```

The result identifies the invoked executable by version and SHA-256. Match the hash to the signed release manifest or a verified local build reference before creating a separate worktree. In that worktree, use `bun scripts/devtools/devtools.ts build-ops act app-build --features owned-ui-evaluation --artifact-out .test-output/candidate.reference.json` with a new reference path. Compare both versions using the same disposable workspace and the action named in the request. Keep the working installation unchanged. A plan does not build or switch anything; a compiled candidate is not an installer. A launcher author cannot grant itself build, installation, microphone or desktop access.

## Local authoring API

The installed executable accepts `--personalize describe` and `--personalize inspect` without opening a window. Other operations read one JSON object on stdin with `--personalize`. Use the currently selected Kit workspace (`SK_PATH` if configured). Use this route only with authority to write that local workspace. It is not a service credential, a workaround for a denied agent grant, or permission to inspect clipboard contents, run scripts or change security settings.

```sh
kit="/Applications/Script Kit.app/Contents/MacOS/script-kit-gpui"
"$kit" --personalize describe
"$kit" --personalize inspect
printf '%s\n' '{"op":"preview","change":{"op":"edit","edit":{"target":"launcher.subject.selection.firstAction","value":"note"}}}' | "$kit" --personalize
```

For a write, bind the request to the `workspace` and `revision` from `inspect`. Choose a fresh version-4 UUID. Here’s the request shape; replace the three placeholders with your own values:

```json
{"op":"apply","request":{"workspace":"<workspace>","expectedRevision":"<revision>","requestId":"<uuid>","change":{"op":"edit","edit":{"target":"launcher.header.pinned-label.textRole","value":"primary"}}}}
```

The reply reports the **stored** revision. The running app reports the revision it actually adopted in `getState.oneWindow.personalization.appliedRevision`. Use `{"op":"status","requestId":"<uuid>"}` after an uncertain result. `committed` means the request is in the active history; there is no need to send it again. Neither `unknown` nor `notInActiveHistory` proves the edit never happened. For example, moving the rules file aside while Kit is stopped can leave its journal entry outside the active history. Resending the original request is not a read: it can put the rule back. Reusing an ID for a different change returns `request_conflict`; a new edit based on an obsolete revision can return `stale_revision`.

If the agent CLI reports `launcher_apply_unverified`, it does not know whether the save happened. With `launcher:read`, pipe `{"requestId":"<original-uuid>"}` to `scriptkit agent launcher status` in the original workspace, then inspect the current preference. `launcher_read_unverified` means the reply did not establish a status; that is different from a valid `unknown` result. Do not submit another edit automatically. If a reply says your rules are disabled but still shows them as effective, or a new edit offers to undo a different change, do not use that reply’s Undo target. Check the original request’s status and inspect the workspace again. A later replay can legitimately show a newer edit as the one to undo.

`{"op":"why","target":"launcher.header.pinned-label.textRole"}` identifies the effective value and its origin. To undo, submit a new revision-bound request with `change: {"op":"undo","of":"<undo from inspect>"}`. `{"op":"disable"}` and `{"op":"enable"}` are also valid changes.

The readable declaration is `plugins/main/customizations/launcher.json`. The private undo journal lives under `db/personal-changes/`, outside personal GitHub backups. Restoring only the declaration restores the rules, not the original machine’s history.
